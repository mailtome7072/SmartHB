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

// T3 시점에는 외부 사용처가 단위 테스트뿐이라 dead_code 경고가 발생한다.
// T4 인증 IPC 커맨드가 본 모듈 함수를 호출하면서 자연 해소된다 — T4 진입 시 본 attribute 제거.
#![allow(dead_code)]

use crate::error::AppError;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use sha2::Sha256;
use zeroize::{ZeroizeOnDrop, Zeroizing};

/// OWASP 2024 권장 — PBKDF2-HMAC-SHA256 최소 반복 횟수.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// PBKDF2 salt 길이 (바이트).
pub const SALT_LEN: usize = 32;

/// SQLCipher 키 길이 (AES-256 = 32바이트).
pub const KEY_LEN: usize = 32;

/// OS Keychain service 식별자.
const KEYRING_SERVICE: &str = "SmartHB";

/// SQLCipher DB 암호화 키의 Keychain user 식별자.
///
/// 동일 OS 사용자 내 SmartHB 인스턴스는 같은 키체인 항목을 공유한다 (덮어쓰기 정책).
/// PRD 단일 사용자(원장 1인) 모델 가정이므로 인스턴스 격리는 불필요.
/// 향후 멀티 사용자가 필요해지면 본 상수에 사용자/디바이스 ID 를 부착해야 한다.
const KEYRING_USER_KEY: &str = "db_encryption_key";

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
    pub fn to_hex(&self) -> Zeroizing<String> {
        Zeroizing::new(hex::encode(self.0))
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

/// SmartHB SQLCipher 키 항목의 keyring `Entry` 핸들을 반환한다.
///
/// `Entry::new` 호출 자체는 OS 핸들 생성에 해당하며 단순 객체이므로 캐싱하지 않는다 —
/// 호출 시점 일관성과 에러 메시지 통일을 위해 헬퍼로 분리.
fn keyring_entry() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_KEY)
        .map_err(|e| AppError::Config(format!("Keychain 항목 생성 실패: {}", e)))
}

/// OS Keychain 에 SQLCipher DB 키를 저장한다 (hex 인코딩 후 즉시 zeroize).
///
/// 동일 service+user 항목이 이미 존재하면 덮어쓰기.
pub fn store_key_in_keyring(key: &DerivedKey) -> Result<(), AppError> {
    let hex_str = key.to_hex();
    keyring_entry()?
        .set_password(&hex_str)
        .map_err(|e| AppError::Auth(format!("Keychain 저장 실패: {}", e)))?;
    Ok(())
}

/// OS Keychain 에서 SQLCipher DB 키를 조회한다.
///
/// 평문 hex 표현과 임시 decode 버퍼는 모두 `Zeroizing` 으로 감싸 함수 종료 시 즉시 폐기.
/// 키가 등록되어 있지 않으면 `AppError::Auth` 반환 — T4 인증 흐름에서 "최초 실행이라
/// 비밀번호 설정 필요" 분기로 처리한다.
pub fn retrieve_key_from_keyring() -> Result<DerivedKey, AppError> {
    let hex_key = Zeroizing::new(
        keyring_entry()?
            .get_password()
            .map_err(|e| AppError::Auth(format!("Keychain 조회 실패: {}", e)))?,
    );
    let decoded = Zeroizing::new(
        hex::decode(hex_key.as_str())
            .map_err(|e| AppError::Auth(format!("키 hex 디코딩 실패: {}", e)))?,
    );
    let mut key_bytes = [0u8; KEY_LEN];
    if decoded.len() != KEY_LEN {
        return Err(AppError::Auth("키 길이 불일치".to_string()));
    }
    key_bytes.copy_from_slice(&decoded);
    let result = DerivedKey(key_bytes);
    // key_bytes 는 result 가 소유하므로 별도 zeroize 불필요 — DerivedKey Drop 시 폐기.
    Ok(result)
}

/// OS Keychain 에서 SQLCipher DB 키를 삭제한다 (재발급/로그아웃 시).
pub fn delete_key_from_keyring() -> Result<(), AppError> {
    keyring_entry()?
        .delete_credential()
        .map_err(|e| AppError::Auth(format!("Keychain 삭제 실패: {}", e)))?;
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

    // Keychain 통합 테스트는 OS Keychain daemon 의존 — 환경에 따라 실패할 수 있어
    // 단위 테스트에서 제외한다. T11 사용자 환경 검증 또는 통합 테스트(별도 모듈)에서 다룬다.
}
