//! 키 관리 + OS Keychain 통합.
//!
//! ADR-004 구현 — `keyring` v3.x 로 macOS Keychain + Windows Credential Manager 통합 접근.
//! PRD §5.5: 사용자 비밀번호 → PBKDF2-HMAC-SHA256 → AES-256 키 → OS Keychain 보관.
//!
//! ## 보안 원칙
//!
//! - 키 메모리 노출 시간 최소화 — `DerivedKey` Drop 시 `zeroize` 로 자동 폐기
//! - 키 바이트 로그 출력 방지 — `Debug` trait 수동 구현 (`"DerivedKey([REDACTED])"`)
//! - 키체인 항목명 하드코딩 — 사용자 입력 비공개 (LDAP injection 류 회피)
//! - PBKDF2 600,000 iterations (OWASP 2024 권장) — 무차별 대입 비용 보장
//!
//! ## 후속 작업
//!
//! 본 모듈은 T4 (인증 IPC) 와 T5 (복구 코드 검증) 에서 사용된다.
//! Tauri IPC 커맨드는 T4 에서 본 모듈을 호출하는 형태로 추가된다.

use crate::error::AppError;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use serde::Serialize;
use sha2::Sha256;
use zeroize::{ZeroizeOnDrop, Zeroizing};

/// OWASP 2024 권장 — PBKDF2-HMAC-SHA256 최소 반복 횟수.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// PBKDF2 salt 길이 (바이트).
pub const SALT_LEN: usize = 32;

/// SQLCipher 키 길이 (AES-256 = 32바이트).
pub const KEY_LEN: usize = 32;

/// OS Keychain service 식별자. recovery 모듈 등 같은 crate 의 다른 인증 관련 모듈에서 재사용.
pub(crate) const KEYRING_SERVICE: &str = "SmartHB";

/// SQLCipher DB 암호화 키의 Keychain user 식별자.
///
/// 동일 OS 사용자 내 SmartHB 인스턴스는 같은 키체인 항목을 공유한다 (덮어쓰기 정책).
/// PRD 단일 사용자(원장 1인) 모델 가정이므로 인스턴스 격리는 불필요.
/// 향후 멀티 사용자가 필요해지면 본 상수에 사용자/디바이스 ID 를 부착해야 한다.
const KEYRING_USER_KEY: &str = "db_encryption_key";

/// PBKDF2 salt 의 Keychain user 식별자.
///
/// T4 임시 저장 위치 — T9 (초기 설정 마법사 + 클라우드 동기화 폴더) 통합 시점에
/// 클라우드 폴더의 평문 파일(`smarthb/salt.bin`)로 이전한다. salt 는 비밀이 아니므로
/// 평문 보관이 가능하며, 양 PC 시점 분리 사용 시 자동 동기화 이득이 크다.
const KEYRING_USER_SALT: &str = "db_password_salt";

/// 사용자 인증 상태.
///
/// 프론트엔드에서 잠금 화면을 "최초 설정" 또는 "잠금 해제" 모드로 분기하기 위해 사용한다.
/// `Unlocked` 는 본 enum 에 없다 — 메모리 상태로만 관리되며 IPC 응답에 포함되지 않는다.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthStatus {
    /// 비밀번호 미설정 — 최초 설정 마법사 진입 필요.
    NotInitialized,
    /// 비밀번호 설정 완료, 현재 잠금 상태.
    Locked,
}

/// PBKDF2 로 유도된 32바이트 키.
///
/// `ZeroizeOnDrop` derive 로 `Drop` 시 자동으로 메모리가 영(0)으로 덮어쓰여진다.
/// `Debug` 는 수동 구현되어 키 바이트가 로그·panic 메시지에 노출되지 않는다.
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey([u8; KEY_LEN]);

impl DerivedKey {
    /// SQLCipher `PRAGMA key` 적용 직전 1회 호출되도록 의도된 hex 인코딩.
    ///
    /// 반환된 `Zeroizing<String>` 은 호출자 스코프 종료 시 자동으로 영(0)으로 덮어쓰여진다.
    /// 평문 hex 가 메모리에 잔류하는 시간을 호출 사이트 한 줄로 제한하기 위함.
    ///
    /// T9 (SQLCipher DB pool 통합) 에서 사용 예정 — 현재 keyring 저장 경로는 내부 헬퍼
    /// `store_bytes_in_keyring` 가 직접 hex 인코딩한다.
    #[allow(dead_code)]
    pub fn to_hex(&self) -> Zeroizing<String> {
        Zeroizing::new(hex::encode(self.0))
    }

    /// 두 키가 동일한지 constant-time 비교로 검사한다.
    ///
    /// 비밀번호 검증 시 타이밍 공격 방어 — 일반 `==` 비교는 일찍 종료되어 비교 시간이
    /// 일치 바이트 수에 비례한다. 본 메서드는 모든 바이트를 XOR 누적하여 입력 길이와 무관한
    /// 일정한 시간에 종료한다. zeroize 보호 유지 — 바이트가 외부로 노출되지 않음.
    pub(crate) fn matches(&self, other: &Self) -> bool {
        let mut diff = 0u8;
        for i in 0..KEY_LEN {
            diff |= self.0[i] ^ other.0[i];
        }
        diff == 0
    }
}

#[cfg(test)]
impl DerivedKey {
    /// 테스트 전용 — 키 바이트 슬라이스.
    ///
    /// 프로덕션 코드에서 직접 노출하면 `ZeroizeOnDrop` 보호가 우회된다.
    /// SQLCipher 통합은 항상 [`DerivedKey::to_hex`] 를 사용한다.
    pub(crate) fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl std::fmt::Debug for DerivedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DerivedKey([REDACTED])")
    }
}

/// 사용자 비밀번호 + salt 로부터 32바이트 키를 유도한다 (PBKDF2-HMAC-SHA256, 600K iter).
///
/// 양 OS 에서 동일한 (password, salt) 입력에 대해 항상 같은 키를 반환해야 한다 (재현성 검증 — 테스트 보장).
pub fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> DerivedKey {
    let mut key_bytes = [0u8; KEY_LEN];
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key_bytes)
        .expect("PBKDF2 출력 길이는 컴파일 타임에 고정");
    DerivedKey(key_bytes)
}

/// 새로운 32바이트 salt 를 CSPRNG 로 생성한다.
///
/// `OsRng` 사용 — OS 의 cryptographically secure random source 직접 활용.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

/// Keychain `Entry` 핸들을 생성한다. `Entry::new` 자체는 OS 핸들만 생성하는 단순 객체이므로
/// 캐싱하지 않고 호출 시점마다 새로 만든다.
pub(crate) fn keyring_entry_for(user: &str) -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(KEYRING_SERVICE, user)
        .map_err(|e| AppError::Config(format!("Keychain 항목 생성 실패: {}", e)))
}

/// 항목 부재(`keyring::Error::NoEntry`) 와 실제 에러를 구분하여 조회한다.
///
/// `check_auth_status` 가 "Keychain 에 항목 없음" 을 `NotInitialized` 로 정확히 매핑하기 위해
/// 사용된다. 다른 에러는 그대로 전파. recovery 모듈 등 같은 crate 의 다른 사용처도 활용.
pub(crate) fn keyring_get_or_none(user: &str) -> Result<Option<Zeroizing<String>>, AppError> {
    match keyring_entry_for(user)?.get_password() {
        Ok(value) => Ok(Some(Zeroizing::new(value))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Auth(format!("Keychain 조회 실패: {}", e))),
    }
}

/// 바이트 슬라이스를 hex 인코딩하여 Keychain 에 저장한다 (덮어쓰기).
fn store_bytes_in_keyring(user: &str, bytes: &[u8], label: &str) -> Result<(), AppError> {
    let hex_value = Zeroizing::new(hex::encode(bytes));
    keyring_entry_for(user)?
        .set_password(&hex_value)
        .map_err(|e| AppError::Auth(format!("{} 저장 실패: {}", label, e)))?;
    Ok(())
}

/// Keychain 에서 hex 인코딩된 바이트 배열을 조회하여 고정 길이로 디코딩한다.
fn retrieve_bytes_from_keyring<const N: usize>(
    user: &str,
    label: &str,
) -> Result<[u8; N], AppError> {
    let hex_value = match keyring_get_or_none(user)? {
        Some(v) => v,
        None => return Err(AppError::Auth(format!("{} 항목이 존재하지 않습니다.", label))),
    };
    let decoded = Zeroizing::new(
        hex::decode(hex_value.as_str())
            .map_err(|e| AppError::Auth(format!("{} hex 디코딩 실패: {}", label, e)))?,
    );
    if decoded.len() != N {
        return Err(AppError::Auth(format!("{} 길이 불일치", label)));
    }
    let mut result = [0u8; N];
    result.copy_from_slice(&decoded);
    Ok(result)
}

/// OS Keychain 에 SQLCipher DB 키를 저장한다.
pub fn store_key_in_keyring(key: &DerivedKey) -> Result<(), AppError> {
    store_bytes_in_keyring(KEYRING_USER_KEY, &key.0, "DB 키")
}

/// OS Keychain 에서 SQLCipher DB 키를 조회한다.
pub fn retrieve_key_from_keyring() -> Result<DerivedKey, AppError> {
    let bytes = retrieve_bytes_from_keyring::<KEY_LEN>(KEYRING_USER_KEY, "DB 키")?;
    Ok(DerivedKey(bytes))
}

/// OS Keychain 에서 SQLCipher DB 키를 삭제한다.
///
/// T5 (PI-07 복구 코드 재발급) 에서 사용 예정 — 현재는 미호출이라 dead_code 허용.
#[allow(dead_code)]
pub fn delete_key_from_keyring() -> Result<(), AppError> {
    keyring_entry_for(KEYRING_USER_KEY)?
        .delete_credential()
        .map_err(|e| AppError::Auth(format!("Keychain 삭제 실패: {}", e)))?;
    Ok(())
}

/// Salt 를 OS Keychain 에 저장한다.
pub(crate) fn store_salt_in_keyring(salt: &[u8; SALT_LEN]) -> Result<(), AppError> {
    store_bytes_in_keyring(KEYRING_USER_SALT, salt, "Salt")
}

/// Salt 를 OS Keychain 에서 조회한다.
fn retrieve_salt_from_keyring() -> Result<[u8; SALT_LEN], AppError> {
    retrieve_bytes_from_keyring::<SALT_LEN>(KEYRING_USER_SALT, "Salt")
}

// ----------------------------------------------------------------------------
// Tauri IPC commands
// ----------------------------------------------------------------------------

/// CPU-bound PBKDF2 600K iter 를 tokio blocking 스레드 풀에서 실행한다.
///
/// PRD §5.6 (앱 시작 < 3초) 보장 — async runtime 이벤트 루프가 PBKDF2 ~500ms 동안 다른
/// IPC 요청을 처리할 수 있도록 한다. `tokio::task::spawn_blocking` 은 dedicated worker
/// thread 풀을 사용.
pub(crate) async fn derive_key_async(
    password: Zeroizing<String>,
    salt: [u8; SALT_LEN],
) -> Result<DerivedKey, AppError> {
    tokio::task::spawn_blocking(move || derive_key(&password, &salt))
        .await
        .map_err(|e| AppError::Auth(format!("키 유도 작업 실패: {}", e)))
}

/// 현재 인증 상태를 반환한다.
///
/// Keychain 에 salt 항목이 존재하면 `Locked`, `NoEntry` 응답이면 `NotInitialized`.
/// 다른 keyring 에러(권한 부족, daemon 오류 등)는 그대로 전파하여 "최초 설정" 으로
/// 잘못 해석되지 않도록 한다.
#[tauri::command]
pub async fn check_auth_status() -> Result<AuthStatus, String> {
    match keyring_get_or_none(KEYRING_USER_SALT).map_err(String::from)? {
        Some(_) => Ok(AuthStatus::Locked),
        None => Ok(AuthStatus::NotInitialized),
    }
}

/// 최초 비밀번호를 설정한다.
///
/// 흐름: salt 생성 → key 유도 (blocking 스레드) → keyring 에 salt + key 저장.
/// 이미 설정된 경우 `store_salt_in_keyring` 이 덮어쓰기를 수행하는데, 본 IPC 호출자는
/// LockScreen 이 `not-initialized` 상태에서만 호출하도록 분기하므로 중복 검증은 생략.
/// 비밀번호 변경(설정 메뉴) 은 별도 IPC 에서 명시적 흐름으로 처리한다.
#[tauri::command]
pub async fn set_password(password: String) -> Result<(), String> {
    let password = Zeroizing::new(password);
    if password.is_empty() {
        return Err(AppError::Auth("비밀번호가 비어있습니다.".to_string()).into());
    }
    let salt = generate_salt();
    let key = derive_key_async(password, salt).await.map_err(String::from)?;
    store_salt_in_keyring(&salt).map_err(String::from)?;
    store_key_in_keyring(&key).map_err(String::from)?;
    Ok(())
}

/// 비밀번호로 DB 잠금을 해제한다.
///
/// 흐름: keyring 에서 salt 조회 → 입력 비밀번호로 key 유도 (blocking) → 저장된 key 와 비교.
/// 일치하지 않으면 `AppError::Auth`.
///
/// 본 함수는 SQLCipher DB 연결을 직접 열지 않는다 — T9 (마법사 + 시작 시퀀스 통합)
/// 시점에 실제 DB pool 초기화 흐름이 추가된다. 현재는 비밀번호 정합성만 검증.
#[tauri::command]
pub async fn unlock_db(password: String) -> Result<(), String> {
    let password = Zeroizing::new(password);
    if password.is_empty() {
        return Err(AppError::Auth("비밀번호를 입력해주세요.".to_string()).into());
    }
    let salt = retrieve_salt_from_keyring().map_err(String::from)?;
    let candidate = derive_key_async(password, salt).await.map_err(String::from)?;
    let stored = retrieve_key_from_keyring().map_err(String::from)?;
    if !candidate.matches(&stored) {
        return Err(AppError::Auth("비밀번호가 일치하지 않습니다.".to_string()).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_produces_consistent_output() {
        let salt = [42u8; SALT_LEN];
        let key1 = derive_key("test_password", &salt);
        let key2 = derive_key("test_password", &salt);
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn derive_key_different_salts_produce_different_keys() {
        let salt1 = [1u8; SALT_LEN];
        let salt2 = [2u8; SALT_LEN];
        let key1 = derive_key("test_password", &salt1);
        let key2 = derive_key("test_password", &salt2);
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn derive_key_different_passwords_produce_different_keys() {
        let salt = [0u8; SALT_LEN];
        let key1 = derive_key("password1", &salt);
        let key2 = derive_key("password2", &salt);
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn derived_key_debug_does_not_leak_bytes() {
        let key = derive_key("test", &[0u8; SALT_LEN]);
        let debug_str = format!("{:?}", key);
        assert!(
            debug_str.contains("REDACTED"),
            "Debug 출력에 REDACTED 마커 누락: {}",
            debug_str
        );
        // 키 바이트가 hex 로도 노출되어서는 안 됨
        let hex = key.to_hex();
        assert!(
            !debug_str.contains(&hex.as_str()[..16]),
            "Debug 출력에 키 hex 일부 누출: {}",
            debug_str
        );
    }

    #[test]
    fn derived_key_hex_format_is_64_chars() {
        let key = derive_key("test", &[0u8; SALT_LEN]);
        let hex = key.to_hex();
        assert_eq!(hex.len(), KEY_LEN * 2, "32바이트 키는 64자 hex");
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_salt_produces_unique_values() {
        let s1 = generate_salt();
        let s2 = generate_salt();
        // 32바이트 random 두 값이 같을 확률 = 2^-256 ≈ 0 (실효적 보장)
        assert_ne!(s1, s2);
    }

    #[test]
    fn generate_salt_returns_correct_length() {
        let salt = generate_salt();
        assert_eq!(salt.len(), SALT_LEN);
    }

    #[test]
    fn matches_returns_true_for_identical_keys() {
        let salt = [7u8; SALT_LEN];
        let key1 = derive_key("password", &salt);
        let key2 = derive_key("password", &salt);
        assert!(key1.matches(&key2));
    }

    #[test]
    fn matches_returns_false_for_different_keys() {
        let salt = [7u8; SALT_LEN];
        let key1 = derive_key("password1", &salt);
        let key2 = derive_key("password2", &salt);
        assert!(!key1.matches(&key2));
    }

    #[test]
    fn matches_returns_false_for_different_salts() {
        let key1 = derive_key("password", &[1u8; SALT_LEN]);
        let key2 = derive_key("password", &[2u8; SALT_LEN]);
        assert!(!key1.matches(&key2));
    }

    // Keychain 통합 테스트는 OS Keychain daemon 의존 — 환경에 따라 실패할 수 있어
    // 단위 테스트에서 제외한다. T11 사용자 환경 검증 또는 통합 테스트(별도 모듈)에서 다룬다.
}
