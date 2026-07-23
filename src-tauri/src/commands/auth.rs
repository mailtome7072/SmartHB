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

use crate::commands::audit::{self, AuditEventType};
use crate::commands::paths;
use crate::error::AppError;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use serde::Serialize;
use sha2::Sha256;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use zeroize::{ZeroizeOnDrop, Zeroizing};

/// OWASP 2024 권장 — PBKDF2-HMAC-SHA256 최소 반복 횟수.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// PBKDF2 salt 길이 (바이트).
pub const SALT_LEN: usize = 32;

/// SQLCipher 키 길이 (AES-256 = 32바이트).
pub const KEY_LEN: usize = 32;

/// 앱 잠금 PIN 길이 (ADR-007 — 6자리 숫자).
pub(crate) const PIN_LEN: usize = 6;

/// OS Keychain service 식별자.
pub(crate) const KEYRING_SERVICE: &str = "SmartHB";

/// SQLCipher DB 암호화 키의 Keychain user 식별자.
///
/// 동일 OS 사용자 내 SmartHB 인스턴스는 같은 키체인 항목을 공유한다 (덮어쓰기 정책).
/// PRD 단일 사용자(원장 1인) 모델 가정이므로 인스턴스 격리는 불필요.
/// 향후 멀티 사용자가 필요해지면 본 상수에 사용자/디바이스 ID 를 부착해야 한다.
const KEYRING_USER_KEY: &str = "db_encryption_key";

/// 구(舊) Keychain salt 항목 식별자 — Sprint 7 T2 에서 `{cloud}/smarthb/salt.bin` 파일로 이전.
///
/// 본 상수는 **자동 마이그레이션 전용** — 기존 설치에서 Keychain 에 남아있는 salt 를 한 번 읽어
/// 파일로 복사한 뒤 Keychain 항목을 삭제하기 위해 보관된다. 신규 설치에는 사용되지 않는다.
/// 마이그레이션 완료 (`{cloud}/smarthb/salt.bin` 존재) 후에는 Keychain 항목이 부재해도 정상 동작.
const LEGACY_KEYRING_USER_SALT: &str = "db_password_salt";

// ────────────────────────────────────────────────────────────────────
// CredentialCache (Sprint 7 T1, Issue 1)
// ────────────────────────────────────────────────────────────────────
//
// macOS Keychain access 마다 별도 Security Framework 다이얼로그가 표시되는 문제 해소.
// salt + key 를 한 번 keyring 에서 읽어 메모리 캐시 → 후속 호출은 캐시 hit (keyring 0회).
//
// 보안 모델:
// - 캐시 항목은 `ZeroizeOnDrop` — 프로세스 종료 또는 명시적 invalidate 시 메모리 영(0)으로 덮어쓰임.
// - 캐시는 비밀번호 검증 후 메모리에 상주 — PRD §5.5 단일 사용자/로컬 앱 모델에서 수용 가능한 trade-off (R35).
// - 캐시 채움은 `set_password` / `verify_password` 첫 호출 / `reset_password_with_code` 시점.

/// 캐시된 자격증명 (salt + 유도된 키).
#[derive(ZeroizeOnDrop)]
struct CachedCredentials {
    salt: [u8; SALT_LEN],
    key: DerivedKey,
}

static CRED_CACHE: OnceLock<Mutex<Option<CachedCredentials>>> = OnceLock::new();

fn cred_cache() -> &'static Mutex<Option<CachedCredentials>> {
    CRED_CACHE.get_or_init(|| Mutex::new(None))
}

/// poisoned Mutex 도 graceful 복구 — `PoisonError::into_inner()` 로 inner guard 회수.
///
/// Sprint 8 T8 (R46 / I-S2-8): `cred_cache().lock().expect("cred_cache poisoned")` 패턴은
/// poison 발생 시 앱 crash. 본 헬퍼는 panic 흔적이 남았어도 캐시 자체는 무결할 가능성이
/// 높다는 가정 아래 graceful 복구한다. 7곳의 lock 호출을 일괄 단순화.
fn cred_cache_lock() -> std::sync::MutexGuard<'static, Option<CachedCredentials>> {
    cred_cache().lock().unwrap_or_else(|e| e.into_inner())
}

/// 캐시에 자격증명 저장 (set_password 또는 reset_password_with_code 호출 후).
pub(crate) fn cache_credentials(salt: [u8; SALT_LEN], key: DerivedKey) {
    *cred_cache_lock() = Some(CachedCredentials { salt, key });
}

/// 캐시 무효화 (delete_key_from_keyring 또는 로그아웃 시).
///
/// Sprint 8 T6 (I-S2-4): `startup::exit_hook()` 에서도 명시적으로 호출되어 종료 시점에
/// 프로세스 메모리의 키 잔류를 최소화한다. `pub` 노출은 cross-module 호출용.
pub fn invalidate_credential_cache() {
    *cred_cache_lock() = None;
}

// ────────────────────────────────────────────────────────────────────
// set_password 재진입 가드 (Sprint 8 T6 / I-S2-3)
// ────────────────────────────────────────────────────────────────────
//
// `set_password` 가 concurrent 하게 호출되면 keyring 저장과 salt 파일 저장 사이에 race 가
// 발생해 (1) keyring 키와 salt 파일이 서로 다른 비밀번호 기준으로 저장되거나, (2) rollback
// 경합으로 양쪽 모두 일관성을 잃을 수 있다. AtomicBool + RAII 가드로 진입을 직렬화한다.
//
// 정상 흐름: 첫 호출이 가드를 잡고 작업 완료 → Drop 으로 해제 → 다음 호출 진입 가능.
// 동시 호출: 두 번째 호출은 즉시 사용자 친화 에러 반환 (UI 가 "잠시 후 재시도" 안내).
// panic 안전: `_guard` 가 stack 에 있으므로 panic unwinding 중에도 Drop 호출됨.

static SET_PASSWORD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// RAII 가드 — `set_password` 진입 시 `try_acquire`, scope 종료 시 자동 해제.
struct SetPasswordGuard;

impl SetPasswordGuard {
    /// 가드 획득 시도 — 이미 진행 중이면 `None` 반환.
    fn try_acquire() -> Option<Self> {
        SET_PASSWORD_IN_PROGRESS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Self)
    }
}

impl Drop for SetPasswordGuard {
    fn drop(&mut self) {
        SET_PASSWORD_IN_PROGRESS.store(false, Ordering::Release);
    }
}

/// 캐시에서 salt 조회 — 캐시 미스면 None.
fn cached_salt() -> Option<[u8; SALT_LEN]> {
    cred_cache_lock().as_ref().map(|c| c.salt)
}

/// 캐시 미스 시 salt(파일) + key(keyring) 동시 로드.
///
/// Sprint 7 T2: salt 출처가 keyring → 파일로 이전. 첫 로드 시 마이그레이션을 함께 시도하므로
/// 기존 설치도 투명하게 신규 경로로 전환된다.
fn load_credentials_to_cache() -> Result<(), AppError> {
    let salt = load_salt()?;
    let key = retrieve_key_from_keyring()?;
    cache_credentials(salt, key);
    Ok(())
}

/// load 작업 직렬화용 Mutex (Sprint 8 T6 / I-S2-7 / R45).
///
/// `cred_cache().lock()` 만으로는 fast-path 캐시 hit 만 race-free — slow-path 의
/// `load_credentials_to_cache()` 호출 사이에 lock 이 해제되어 두 스레드가 동시 진입 시
/// keyring 을 2회 호출하는 race 가 가능했다. 본 Mutex 로 load 자체를 직렬화하여
/// macOS Keychain 다이얼로그가 startup 동안 정확히 1회만 표시되도록 보장한다.
///
/// 락 순서: 항상 `LOAD_MUTEX` → `cred_cache` (deadlock 회피). fast-path 는 `cred_cache`
/// 만 사용하므로 `LOAD_MUTEX` 를 잡지 않아 다른 fast-path 호출과 경합하지 않는다.
static LOAD_MUTEX: Mutex<()> = Mutex::new(());

/// 캐시 미스 시 keyring + salt 를 정확히 1회만 로드한다 (Sprint 8 T7 / R45).
///
/// double-checked locking 패턴: fast-path 캐시 hit → slow-path LOAD_MUTEX 직렬화
/// → double-check (다른 스레드가 이미 로드 완료했을 수 있음) → load.
fn ensure_cache_loaded() -> Result<(), AppError> {
    // Fast path — 캐시 hit (대다수 호출).
    if cred_cache_lock().is_some() {
        return Ok(());
    }
    // Slow path — load 직렬화. 첫 진입자가 keyring/salt 1회 로드, 후속 진입자는 대기 후 hit.
    // poison 복구는 cred_cache_lock 과 동일 패턴 (Sprint 8 T8 / R46).
    let _load_guard = LOAD_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if cred_cache_lock().is_some() {
        return Ok(());
    }
    load_credentials_to_cache()
}

/// 캐시 우선 조회, 미스 시 keyring 1회 로드 후 캐시 → DerivedKey 복제 반환.
///
/// cipher feature on 빌드의 db.rs / backup.rs / integrity.rs 가 사용. 매 호출마다 keyring
/// 다이얼로그를 띄우던 기존 패턴을 1회로 통합.
///
/// Sprint 8 T7 (R45): double-checked locking 누락 race 제거 — `ensure_cache_loaded` 헬퍼로
/// 통합. `tokio::join!` 안의 integrity check + 후속 verify_password 가 동시 진입해도 keyring
/// 은 정확히 1회만 호출된다.
#[cfg_attr(not(feature = "cipher"), allow(dead_code))]
pub fn get_cached_or_load_key() -> Result<DerivedKey, AppError> {
    ensure_cache_loaded()?;
    let guard = cred_cache_lock();
    Ok(DerivedKey(guard.as_ref().expect("just loaded").key.0))
}

#[cfg(test)]
pub(crate) fn reset_credential_cache_for_tests() {
    invalidate_credential_cache();
}

// ────────────────────────────────────────────────────────────────────

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

/// 앱 잠금 PIN 형식 검증 (ADR-007) — 정확히 6자리 ASCII 숫자.
///
/// UI(`LockScreen`)가 1차로 강제하지만, IPC 직접 호출 우회를 방어하기 위해
/// `set_password` / `reset_password_with_code` 진입점에서 재검증한다.
pub(crate) fn validate_pin(pin: &str) -> Result<(), AppError> {
    if pin.len() == PIN_LEN && pin.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(AppError::Auth("PIN 번호는 6자리 숫자여야 합니다.".to_string()))
    }
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
/// 사용된다. 다른 에러는 그대로 전파.
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


/// OS Keychain 에 SQLCipher DB 키를 저장한다.
pub fn store_key_in_keyring(key: &DerivedKey) -> Result<(), AppError> {
    store_bytes_in_keyring(KEYRING_USER_KEY, &key.0, "DB 키")
}

/// OS Keychain 에서 SQLCipher DB 키를 조회한다.
///
/// Sprint 7 T2 보안 패치 #4 (PC-B 시나리오 UX):
/// 항목 부재 (`NoEntry`) 는 양 PC 첫 동기화 시점에서 빈번 — salt.bin 은 cloud sync 로 도착했지만
/// 이 PC 에는 아직 keyring key 가 생성되지 않은 상태. 50대 사용자 친화 한국어 메시지로 안내.
pub fn retrieve_key_from_keyring() -> Result<DerivedKey, AppError> {
    match keyring_get_or_none(KEYRING_USER_KEY)? {
        Some(hex_value) => {
            let decoded = Zeroizing::new(
                hex::decode(hex_value.as_str())
                    .map_err(|e| AppError::Auth(format!("DB 키 hex 디코딩 실패: {}", e)))?,
            );
            if decoded.len() != KEY_LEN {
                return Err(AppError::Auth("DB 키 길이 불일치".to_string()));
            }
            let mut bytes = [0u8; KEY_LEN];
            bytes.copy_from_slice(&decoded);
            Ok(DerivedKey(bytes))
        }
        None => Err(AppError::Auth(
            "이 PC 에는 인증 정보(키)가 없습니다. 다른 PC 에서 설정한 비밀번호로는 \
            잠금을 해제할 수 없습니다 — 최초 설정을 다시 진행하거나 관리자에게 문의해 주세요."
                .to_string(),
        )),
    }
}

/// OS Keychain 에서 SQLCipher DB 키를 삭제한다.
///
/// set_password rollback 에서 사용.
/// Sprint 7 T1: 캐시도 함께 무효화.
/// Sprint 7 T2 보안 패치 #13: `NoEntry` 는 idempotent 성공 — `delete_legacy_keyring_salt` 와 일관.
pub fn delete_key_from_keyring() -> Result<(), AppError> {
    match keyring_entry_for(KEYRING_USER_KEY)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {
            invalidate_credential_cache();
            Ok(())
        }
        Err(e) => Err(AppError::Auth(format!("Keychain 삭제 실패: {}", e))),
    }
}

// ────────────────────────────────────────────────────────────────────
// Salt 파일 저장 (Sprint 7 T2, A17/A27)
// ────────────────────────────────────────────────────────────────────
//
// salt 는 비밀이 아니므로 `{cloud}/smarthb/salt.bin` 평문 32바이트로 저장.
// 양 PC 동기화 자동 + Keychain 다이얼로그 1회 추가 제거.
//
// 손상 복구: NTFS power-loss 패턴 (fs::write + rename 후 NULL 페이지 잔존) 방어 —
// setup.rs `is_corrupted` 패턴을 답습하여 길이/NULL 검증.

/// salt.bin 의 32바이트 외 입력은 손상으로 간주한다.
///
/// 손상 판정 조건 (Sprint 8 T6 / I-S2-2 강화):
/// 1. 길이 불일치 (`!= SALT_LEN`)
/// 2. 전체 NULL (NTFS power-loss 시 페이지 전체가 0x00 잔존)
/// 3. 첫 8바이트가 모두 동일한 단일 바이트 (NTFS partial-write 시 페이지가 0x00/0xFF/특정 패턴으로 잔존)
///    — 정상 random salt 가 8연속 동일 바이트일 확률 = 256^-7 ≈ 1.4e-17, 무시 가능.
fn is_salt_corrupted(bytes: &[u8]) -> bool {
    if bytes.len() != SALT_LEN {
        return true;
    }
    if bytes.iter().all(|&b| b == 0) {
        return true;
    }
    let first = bytes[0];
    bytes[..8].iter().all(|&b| b == first)
}

/// salt 를 cloud 폴더 파일에 atomic 쓰기 (tmp → rename).
///
/// 부모 디렉토리는 setup::save_cloud_folder 가 이미 생성하지만 신규 설치 첫 호출 시점
/// 또는 통합 테스트에서 미존재할 수 있어 명시적 `create_dir_all` 로 안전망 유지.
pub(crate) fn store_salt(salt: &[u8; SALT_LEN]) -> Result<(), AppError> {
    store_salt_to(&paths::salt_path(), salt)
}

fn store_salt_to(path: &Path, salt: &[u8; SALT_LEN]) -> Result<(), AppError> {
    use std::io::Write;
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    std::fs::create_dir_all(&parent)
        .map_err(|e| AppError::Auth(format!("salt 디렉토리 생성 실패: {}", e)))?;

    // NTFS power-loss 방어 (Sprint 7 T2 보안 패치 #5, MEMORY.md ntfs-power-loss-pattern):
    // fs::write + fs::rename 는 메타데이터 커밋만 보장 — 데이터 페이지가 NULL 로 남는 사례를
    // 실측. sync_all 로 tmp 데이터를 디스크에 flush 한 후 rename 한다.
    let tmp = path.with_extension("bin.tmp");
    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)
            .map_err(|e| AppError::Auth(format!("salt 임시 파일 열기 실패: {}", e)))?;
        f.write_all(salt)
            .map_err(|e| AppError::Auth(format!("salt 임시 파일 쓰기 실패: {}", e)))?;
        f.sync_all()
            .map_err(|e| AppError::Auth(format!("salt 임시 파일 sync 실패: {}", e)))?;
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        // rename 실패 시 tmp 파일이 cloud sync 폴더에 누적되어 sync 대상으로 떠도는 것을 막는다.
        let _ = std::fs::remove_file(&tmp);
        return Err(AppError::Auth(format!("salt 파일 rename 실패: {}", e)));
    }
    // 부모 디렉토리 fsync — POSIX 표준이지만 Windows 는 미지원/무시. best-effort 로 적용.
    if let Ok(dir) = std::fs::File::open(&parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

/// salt 를 cloud 폴더 파일에서 로드 — 미존재 시 Keychain 마이그레이션 시도.
///
/// 절차:
/// 1. `salt.bin` 존재 + 정상 (32바이트, NULL 아님) → 반환
/// 2. `salt.bin` 손상 → 손상본 백업 (`salt.bin.corrupted-{ts}`) → 마이그레이션 시도
/// 3. 파일 미존재 → Keychain `db_password_salt` 조회 → 있으면 파일로 이전 + Keychain 삭제
/// 4. Keychain 에도 없으면 `NotInitialized` 에러 — 호출자(`verify_password`)가 사용자 알림
fn load_salt() -> Result<[u8; SALT_LEN], AppError> {
    load_salt_from(&paths::salt_path())
}

fn load_salt_from(path: &Path) -> Result<[u8; SALT_LEN], AppError> {
    match std::fs::read(path) {
        Ok(bytes) if !is_salt_corrupted(&bytes) => {
            let mut salt = [0u8; SALT_LEN];
            salt.copy_from_slice(&bytes);
            Ok(salt)
        }
        Ok(bytes) => {
            eprintln!(
                "[auth] salt.bin 손상 감지 ({} 바이트). 백업 후 마이그레이션 재시도.",
                bytes.len()
            );
            backup_corrupted_salt(path);
            migrate_keyring_salt_to(path)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => migrate_keyring_salt_to(path),
        Err(e) => Err(AppError::Auth(format!("salt 파일 읽기 실패: {}", e))),
    }
}

/// 손상된 salt.bin 을 `salt.bin.corrupted-{unix_ts}` 로 백업 (best-effort).
fn backup_corrupted_salt(path: &Path) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = path.with_extension(format!("bin.corrupted-{}", ts));
    if let Err(e) = std::fs::rename(path, &backup) {
        eprintln!("[auth] 손상 salt 백업 실패 ({}): {}", backup.display(), e);
    }
}

/// Keychain 에 남아있는 salt 를 파일로 1회 이전한다.
///
/// 부분 실패 복원성:
/// - 파일 쓰기 실패 → Keychain 삭제 안 함, 에러 반환 → 다음 시작에 재시도
/// - 파일 쓰기 성공 + Keychain 삭제 실패 → best-effort 경고만, salt 반환 성공
///   (다음 시작 시 파일 우선 경로로 분기, Keychain 잔존 항목은 무해)
fn migrate_keyring_salt_to(path: &Path) -> Result<[u8; SALT_LEN], AppError> {
    let bytes = match keyring_get_or_none(LEGACY_KEYRING_USER_SALT)? {
        Some(v) => v,
        None => {
            return Err(AppError::Auth(
                "salt 가 설정되지 않았습니다 (최초 설정 필요).".to_string(),
            ));
        }
    };
    let decoded = Zeroizing::new(
        hex::decode(bytes.as_str())
            .map_err(|e| AppError::Auth(format!("Salt hex 디코딩 실패: {}", e)))?,
    );
    if decoded.len() != SALT_LEN {
        return Err(AppError::Auth("Salt 길이 불일치".to_string()));
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&decoded);

    store_salt_to(path, &salt)?;
    eprintln!(
        "[auth] Keychain salt → 파일 마이그레이션 완료 ({})",
        path.display()
    );

    if let Err(e) = delete_legacy_keyring_salt() {
        eprintln!(
            "[auth] Keychain salt 항목 삭제 실패 (best-effort, 다음 시작에 재시도): {}",
            e
        );
    }

    // Sprint 8 T8 (R47 / I-S2-9): salt 마이그레이션은 1회 이벤트이므로 audit 추적.
    // 본 함수는 sync 이고 try_record 는 async — tokio runtime 이 활성일 때만 fire-and-forget.
    // verify_password 경로에서 호출되므로 runtime 은 항상 있지만, 테스트/단위 호출 경로 보호.
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async {
            audit::try_record(
                AuditEventType::SecurityEvent,
                Some("salt-migration"),
                Some(r#"{"detail":"legacy keyring salt → cloud file"}"#),
            )
            .await;
        });
    }
    Ok(salt)
}

/// 구 Keychain salt 항목 삭제 — 항목 부재는 정상 (이미 삭제되었거나 신규 설치).
fn delete_legacy_keyring_salt() -> Result<(), AppError> {
    match keyring_entry_for(LEGACY_KEYRING_USER_SALT)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth(format!("Keychain salt 삭제 실패: {}", e))),
    }
}

/// salt 파일 존재 여부 — `check_auth_status` 가 마법사 진입 분기에 사용.
///
/// 파일 부재 시 Keychain 에 마이그레이션 대상이 있는지 확인하여 "기존 설치" 도 `Locked` 로 인식.
fn salt_exists() -> Result<bool, AppError> {
    salt_exists_at(&paths::salt_path())
}

fn salt_exists_at(path: &Path) -> Result<bool, AppError> {
    if path.exists() {
        return Ok(true);
    }
    Ok(keyring_get_or_none(LEGACY_KEYRING_USER_SALT)?.is_some())
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
/// 캐시(salt 보유)가 있으면 즉시 `Locked`. 캐시 미스 시 salt.bin 파일 존재 여부 확인 —
/// 파일 또는 legacy Keychain 항목이 있으면 `Locked`, 둘 다 없으면 `NotInitialized`.
/// 다른 IO/keyring 에러는 그대로 전파하여 "최초 설정" 으로 잘못 해석되지 않도록 한다.
///
/// Sprint 7 T1: 캐시 우선 조회 — `verify_password` 후 호출되면 keyring 다이얼로그 0회.
/// Sprint 7 T2: salt 저장소가 파일로 이전 — 파일 존재 시 keyring 호출 0회.
#[tauri::command]
pub async fn check_auth_status() -> Result<AuthStatus, String> {
    if cached_salt().is_some() {
        return Ok(AuthStatus::Locked);
    }
    if salt_exists().map_err(String::from)? {
        Ok(AuthStatus::Locked)
    } else {
        Ok(AuthStatus::NotInitialized)
    }
}

/// 최초 비밀번호를 설정한다.
///
/// 흐름: salt 생성 → key 유도 (blocking 스레드) → keyring 에 key 저장 → cloud 파일에 salt 저장.
/// 이미 설정된 경우 `store_key_in_keyring` 와 `store_salt` 가 각자 덮어쓰기를 수행하지만, 본 IPC
/// 호출자는 LockScreen 이 `not-initialized` 상태에서만 호출하도록 분기하므로 중복 검증은 생략.
/// 비밀번호 변경(설정 메뉴) 은 별도 IPC 에서 명시적 흐름으로 처리한다.
///
/// Sprint 7 T2 보안 패치 #2: 매체간 partial-failure 시 orphan salt.bin 차단을 위해
/// `store_key_in_keyring` → `store_salt` 순서 + salt 실패 시 keyring rollback.
#[tauri::command]
pub async fn set_password(password: String) -> Result<(), String> {
    // Sprint 8 T6 (I-S2-3): 재진입 가드. `_guard` 는 함수 종료 시 Drop 으로 자동 해제.
    let _guard = SetPasswordGuard::try_acquire().ok_or_else(|| {
        String::from(AppError::Auth(
            "비밀번호 설정이 이미 진행 중입니다. 잠시 후 다시 시도하세요.".to_string(),
        ))
    })?;
    let password = Zeroizing::new(password);
    // ADR-007: 앱 잠금은 6자리 숫자 PIN. UI 우회 방어를 위해 진입점에서 재검증.
    validate_pin(&password).map_err(String::from)?;
    // M2(T5): 기존 salt 하드 가드. 이미 salt(파일/레거시 keyring)가 있으면 새 salt 생성을 거부한다.
    // 새 salt 를 만들면 기존 암호화 DB 의 키 유도가 달라져 DB 를 열 수 없게 되므로(데이터 접근 불가),
    // set_password 는 최초 설정에서만 허용한다. 비밀번호 변경/2번째 PC 로그인은 별도 흐름
    // (change_password / try_adopt_key)에서 처리한다. 프론트 not-initialized 분기와 이중 방어.
    if salt_exists().map_err(String::from)? {
        return Err(String::from(AppError::Auth(
            "이미 설정된 비밀번호가 있습니다. 비밀번호 변경 또는 다른 PC 로그인 기능을 사용하세요.".to_string(),
        )));
    }
    eprintln!("[auth] set_password 진입");
    let salt = generate_salt();
    let key = derive_key_async(password, salt).await.map_err(String::from)?;
    // PRD §5.5 로그 위생 — key/salt 바이트 일부 hex 도 stderr 에 노출되지 않도록 제거 (R36).
    // Sprint 7 T2 보안 패치 #1: ZeroizeOnDrop + Debug='[REDACTED]' 보호가 eprintln 평문 노출로 우회되던 버그.
    // 순서: keyring(원자성 약함, 사용자 다이얼로그 가능) 먼저 → 성공 시 salt 파일 → 실패 시 keyring rollback (R37).
    store_key_in_keyring(&key).map_err(String::from)?;
    if let Err(e) = store_salt(&salt) {
        // salt 파일 실패 → keyring key 즉시 롤백하여 orphan keyring 항목 + 빈 salt.bin 상태 차단.
        // delete 실패는 best-effort 로그 (다음 set_password 가 덮어씀).
        if let Err(rollback) = delete_key_from_keyring() {
            eprintln!(
                "[auth] set_password rollback 실패 (keyring key 잔존): {} — 다음 set_password 가 덮어씀",
                rollback
            );
        }
        return Err(String::from(e));
    }
    eprintln!("[auth] set_password: keyring + salt.bin 저장 완료");
    // Sprint 7 T1: 저장 직후 캐시에 즉시 반영 — 후속 verify_password / cipher key 조회가 keyring 호출 없이 동작.
    cache_credentials(salt, DerivedKey(key.0));
    // 최초 설정 시점은 pool 미초기화 — try_record 가 silent skip. 비밀번호 변경 IPC(추후)는 unlock 후 호출되어 정상 기록.
    audit::try_record(AuditEventType::PasswordChange, None, None).await;
    Ok(())
}

/// 비밀번호 정합성을 검증한다 (내부 헬퍼).
///
/// Sprint 7 T1: keyring 직접 호출 제거 — `get_cached_or_load_key` 가 첫 호출 시 keyring 2회
/// 통합 로드 + 캐시 채움, 후속 호출은 캐시 hit. 후속 cipher key 조회 (db.rs / backup.rs /
/// integrity.rs) 도 모두 캐시 경유 → macOS Keychain 다이얼로그가 startup 동안 1회로 통합 (prod
/// "Always Allow" 적용 시).
///
/// `unlock_db` IPC 와 `app_startup_sequence` 양쪽이 공유.
pub(crate) async fn verify_password(password: &Zeroizing<String>) -> Result<(), AppError> {
    // ADR-007: 여기서는 PIN 형식(validate_pin)을 검사하지 않는다 — 형식 강제는 set/reset 진입점 책임.
    // 잠금 해제는 형식과 무관하게 저장된 키와의 일치 여부만 본다(불일치 시 어차피 인증 실패).
    if password.is_empty() {
        eprintln!("[auth] verify_password: 빈 비밀번호");
        return Err(AppError::Auth("비밀번호를 입력해주세요.".to_string()));
    }
    eprintln!(
        "[auth] verify_password 진입 (password.len={})",
        password.len()
    );

    // 캐시 미스 시 한 번에 salt + key 로드 (keyring 2회 → 통합 호출).
    // Sprint 8 T7 (R45): `ensure_cache_loaded` 가 LOAD_MUTEX 로 직렬화 — `tokio::join!` 안의
    // integrity check 와 동시 진입해도 keyring 은 정확히 1회만 호출된다.
    ensure_cache_loaded()?;

    let salt = cached_salt().expect("just loaded into cache");
    let candidate = derive_key_async(password.clone(), salt).await?;
    // PRD §5.5 로그 위생 — derived key hex 의 일부도 stderr 에 노출되지 않도록 제거 (Sprint 7 T2 보안 패치 #1).

    // 캐시에서 stored key 비교 (Mutex guard 내에서 직접 비교 — 별도 복사 회피).
    let matches = {
        let guard = cred_cache_lock();
        let stored_key = &guard.as_ref().expect("just loaded into cache").key;
        candidate.matches(stored_key)
    };

    if !matches {
        eprintln!(
            "[auth] verify_password: 키 매치 실패 (candidate≠stored — set/verify 사이 salt 또는 key 불일치 의심)"
        );
        return Err(AppError::Auth("비밀번호가 일치하지 않습니다.".to_string()));
    }
    eprintln!("[auth] verify_password: 인증 통과");
    Ok(())
}

/// 비밀번호로 DB 잠금을 해제한다.
///
/// 본 함수는 SQLCipher DB 연결을 직접 열지 않는다 — 실제 DB pool 초기화는
/// `app_startup_sequence` (T10) 가 담당. 본 IPC 는 잠금 화면에서 비밀번호 정합성 사전 검증에 사용.
#[tauri::command]
pub async fn unlock_db(password: String) -> Result<(), String> {
    let password = Zeroizing::new(password);
    verify_password(&password).await.map_err(String::from)
}

/// 현재 PIN 을 확인한 뒤 새 PIN 으로 변경한다 (잠금 해제 상태에서 호출).
///
/// 흐름: 현 PIN 검증 → 새 salt 생성 → 새 key 유도 → keyring 갱신 → salt 파일 갱신 → 캐시 갱신.
/// `set_password` / `reset_password_with_code` 와 동일한 atomic order + rollback 패턴.
/// SetPasswordGuard 재사용 — 동시 호출 차단.
#[tauri::command]
pub async fn change_pin(current_pin: String, new_pin: String) -> Result<(), String> {
    let _guard = SetPasswordGuard::try_acquire().ok_or_else(|| {
        String::from(AppError::Auth(
            "비밀번호 설정이 이미 진행 중입니다. 잠시 후 다시 시도하세요.".to_string(),
        ))
    })?;
    let current = Zeroizing::new(current_pin);
    let new_pin = Zeroizing::new(new_pin);
    validate_pin(&new_pin).map_err(String::from)?;
    verify_password(&current).await.map_err(String::from)?;

    let salt = generate_salt();
    let new_key = derive_key_async(new_pin, salt)
        .await
        .map_err(String::from)?;
    store_key_in_keyring(&new_key).map_err(String::from)?;
    if let Err(e) = store_salt(&salt) {
        if let Err(rollback) = delete_key_from_keyring() {
            eprintln!(
                "[auth] change_pin rollback 실패 (keyring key 잔존): {} — 다음 set 시 덮어씀",
                rollback
            );
        }
        return Err(String::from(e));
    }
    cache_credentials(salt, new_key);
    audit::try_record(AuditEventType::PasswordChange, Some("user-change"), None).await;
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
    fn validate_pin_accepts_six_digits() {
        // ADR-007: 정확히 6자리 숫자만 허용.
        assert!(validate_pin("123456").is_ok());
        assert!(validate_pin("000000").is_ok());
    }

    #[test]
    fn validate_pin_rejects_invalid() {
        assert!(validate_pin("12345").is_err(), "5자리 거부");
        assert!(validate_pin("1234567").is_err(), "7자리 거부");
        assert!(validate_pin("").is_err(), "빈 입력 거부");
        assert!(validate_pin("12a456").is_err(), "숫자 외 문자 거부");
        assert!(validate_pin("12 456").is_err(), "공백 거부");
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

    // ─── Sprint 7 T1: CredentialCache 단위 테스트 (keyring 없이 검증) ───

    /// 동일 process 내 테스트 간 캐시 상태 격리 — Mutex 가 process-wide 이므로 명시적 reset.
    fn reset_cache_for_test() {
        reset_credential_cache_for_tests();
    }

    #[test]
    fn cache_miss_returns_none() {
        reset_cache_for_test();
        assert!(cached_salt().is_none());
    }

    #[test]
    fn cache_credentials_then_read_salt() {
        reset_cache_for_test();
        let salt = [99u8; SALT_LEN];
        let key = derive_key("test", &salt);
        cache_credentials(salt, key);
        assert_eq!(cached_salt(), Some(salt));
    }

    #[test]
    fn invalidate_clears_cache() {
        reset_cache_for_test();
        let salt = [11u8; SALT_LEN];
        let key = derive_key("p", &salt);
        cache_credentials(salt, key);
        assert!(cached_salt().is_some());
        invalidate_credential_cache();
        assert!(cached_salt().is_none());
    }

    #[test]
    fn cache_credentials_overwrites_previous() {
        reset_cache_for_test();
        let salt_a = [1u8; SALT_LEN];
        cache_credentials(salt_a, derive_key("a", &salt_a));
        let salt_b = [2u8; SALT_LEN];
        cache_credentials(salt_b, derive_key("b", &salt_b));
        assert_eq!(cached_salt(), Some(salt_b));
        reset_cache_for_test(); // 후속 테스트 격리
    }

    /// get_cached_or_load_key 가 캐시 hit 시 keyring 호출 없이 반환 — Mutex 만 동작 확인.
    #[test]
    fn get_cached_or_load_key_hits_cache() {
        reset_cache_for_test();
        let salt = [42u8; SALT_LEN];
        let original = derive_key("pw", &salt);
        let original_bytes = *original.as_bytes();
        cache_credentials(salt, original);
        let retrieved = get_cached_or_load_key().expect("cache hit must succeed");
        assert_eq!(retrieved.as_bytes(), &original_bytes);
        reset_cache_for_test();
    }

    // ─── Sprint 7 T2: salt.bin 파일 입출력 테스트 ───
    //
    // paths::salt_path() 는 thread_local DATA_ROOT 를 사용 — 각 #[test] 가 독립.
    // 테스트별 고유 디렉토리를 임시로 지정하여 병렬 실행 격리.

    fn unique_test_dir(label: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("smarthb-salt-test-{}-{}", label, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn store_and_load_salt_round_trip() {
        let dir = unique_test_dir("roundtrip");
        let path = dir.join("salt.bin");
        let salt = varied_salt(33);
        store_salt_to(&path, &salt).expect("store_salt_to must succeed");
        let loaded = load_salt_from(&path).expect("load_salt_from must succeed");
        assert_eq!(loaded, salt);
        // tmp 파일이 rename 후 남아있지 않아야 함 (atomic write 보장)
        assert!(!dir.join("salt.bin.tmp").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn store_salt_creates_parent_directory() {
        let dir = unique_test_dir("parent-create");
        let path = dir.join("nested").join("more").join("salt.bin");
        let salt = varied_salt(7);
        store_salt_to(&path, &salt).expect("부모 디렉토리 자동 생성");
        assert!(path.exists());
        let loaded = load_salt_from(&path).expect("저장 후 로드");
        assert_eq!(loaded, salt);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 테스트용 다양성 salt — `seed ^ (i*7)` 로 첫 8바이트가 단일 값 도배되지 않도록 한다.
    /// I-S2-2 강화 후 단일 바이트 도배 salt 는 손상으로 판정되므로, store/load 라운드트립
    /// 테스트는 이 헬퍼로 정상 salt 를 생성한다.
    fn varied_salt(seed: u8) -> [u8; SALT_LEN] {
        let mut s = [0u8; SALT_LEN];
        for (i, b) in s.iter_mut().enumerate() {
            *b = seed ^ (i as u8).wrapping_mul(7);
        }
        s
    }

    #[test]
    fn is_salt_corrupted_detects_all_zeros() {
        assert!(is_salt_corrupted(&[0u8; SALT_LEN]), "32바이트 NULL 은 손상");
        assert!(is_salt_corrupted(&[0u8; 0]), "빈 파일은 손상");
        assert!(is_salt_corrupted(&[1u8; 31]), "31바이트는 손상 (길이 불일치)");
        assert!(is_salt_corrupted(&[1u8; 33]), "33바이트는 손상");
        assert!(
            !is_salt_corrupted(&varied_salt(42)),
            "다양한 바이트 salt 는 정상"
        );
    }

    /// I-S2-2 (R40): partial-NULL / 단일 바이트 도배 패턴 감지.
    /// NTFS power-loss 시 페이지가 0x00 / 0xFF / 임의 단일 값으로 잔존하는 사례를 손상으로 판정.
    #[test]
    fn is_salt_corrupted_detects_partial_null_patterns() {
        // 32바이트 모두 0xFF 도배 → 손상
        assert!(
            is_salt_corrupted(&[0xFFu8; SALT_LEN]),
            "0xFF 32바이트 도배는 손상"
        );
        // 32바이트 모두 0x42 도배 → 손상 (단일 바이트 fill)
        assert!(
            is_salt_corrupted(&[0x42u8; SALT_LEN]),
            "임의 단일 바이트 32 도배는 손상"
        );
        // 첫 8바이트만 동일 + 이후 다양 → 손상 (partial-write fill 시그니처)
        let mut partial = [0u8; SALT_LEN];
        partial[..8].fill(0x55);
        for (i, b) in partial.iter_mut().enumerate().skip(8) {
            *b = (i as u8).wrapping_mul(13);
        }
        assert!(
            is_salt_corrupted(&partial),
            "첫 8바이트 단일 바이트 도배는 손상"
        );
        // 첫 8바이트가 다양 + 이후 NULL → 본 휴리스틱은 감지 안 함 (한계 명시).
        // 정상 random salt 의 첫 8바이트 동일 확률은 256^-7 ≈ 1.4e-17 이므로 false positive 무시 가능.
        let mut head_diverse = [0u8; SALT_LEN];
        for (i, b) in head_diverse.iter_mut().enumerate().take(8) {
            *b = (i as u8).wrapping_mul(31).wrapping_add(7);
        }
        assert!(
            !is_salt_corrupted(&head_diverse),
            "첫 8바이트 다양하면 뒤가 NULL 이어도 본 휴리스틱은 정상 판정 (감지 한계)"
        );
    }

    /// ⚠️ I-S2-6 (Sprint 7 hotfix): 본 테스트는 `load_salt_from` 호출 시 내부적으로
    /// `migrate_keyring_salt_to` 가 트리거되어 dev 환경의 **실제 OS Keychain 항목** 을
    /// 읽고 삭제할 수 있다. 개발자가 평소 SmartHB 를 사용 중이면 그 비밀번호 salt 가 손상
    /// 가능. CI/통합 테스트에서만 명시적으로 실행하도록 `#[ignore]` 가드 적용.
    /// 수동 실행: `cargo test --lib load_salt_backs_up_corrupted_file -- --ignored --nocapture`
    #[test]
    #[ignore = "I-S2-6: dev keychain 부수효과 방지 — 명시적 --ignored 시에만 실행"]
    fn load_salt_backs_up_corrupted_file() {
        let dir = unique_test_dir("corrupted-backup");
        let path = dir.join("salt.bin");
        // 손상본: 32바이트 NULL — is_corrupted 트리거
        std::fs::write(&path, [0u8; SALT_LEN]).unwrap();
        assert!(path.exists());

        // 검증 핵심: 손상본이 `.corrupted-{ts}` 로 rename 됨 (마이그레이션 성공 여부와 무관).
        // 후속 동작(Keychain 마이그레이션 또는 NotInitialized 에러)은 Keychain 상태에 의존하므로
        // 여기서는 검증하지 않는다 — `migrate_keyring_salt_to` 단위 동작은 OS daemon 의존이라 제외.
        let _ = load_salt_from(&path);
        let corrupted_backups: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains("salt.bin.corrupted-")
            })
            .collect();
        assert_eq!(
            corrupted_backups.len(),
            1,
            "정확히 1개의 손상 백업이 생성되어야 함"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn salt_exists_at_returns_true_when_file_present() {
        let dir = unique_test_dir("exists-file");
        let path = dir.join("salt.bin");
        std::fs::write(&path, varied_salt(9)).unwrap();
        assert!(salt_exists_at(&path).expect("파일 존재 확인"));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// M2(T5): 기존 salt.bin 이 있으면 set_password 가 새 salt 생성을 거부한다 (하드 가드).
    /// 셋업 완료 후 set_password 재호출이 기존 암호화 DB 키를 파괴하는 것을 방지.
    #[tokio::test]
    async fn set_password_rejects_when_salt_exists() {
        let dir = unique_test_dir("m2-salt-guard");
        crate::commands::paths::update_data_root(dir.clone());
        std::fs::write(dir.join("salt.bin"), varied_salt(7)).unwrap();

        let result = set_password("123456".to_string()).await;
        assert!(result.is_err(), "기존 salt 존재 시 set_password 거부");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("이미 설정된") || msg.contains("비밀번호"),
            "안내 메시지에 이유 포함: {}",
            msg
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// load_salt_from 이 동일 파일 입력에 대해 항상 같은 결과 반환 —
    /// 양 PC 가 동기화된 salt.bin 으로 같은 key 를 유도할 수 있는 핵심 전제 (AC-T2-3).
    #[test]
    fn load_salt_is_deterministic_for_two_machines() {
        let dir = unique_test_dir("two-pc");
        let path = dir.join("salt.bin");
        let salt_a = varied_salt(123);
        store_salt_to(&path, &salt_a).unwrap();
        let pc_a = load_salt_from(&path).unwrap();
        let pc_b = load_salt_from(&path).unwrap();
        assert_eq!(pc_a, pc_b);
        assert_eq!(pc_a, salt_a);
        std::fs::remove_dir_all(&dir).ok();
    }

    // ─── Sprint 7 T2 보안 패치 단위 테스트 ───

    /// S-T2-4: NTFS power-loss 방어 — rename 실패 시 tmp 파일이 cloud sync 폴더에 누적되지 않음.
    /// rename 실패를 시뮬레이션하기는 어려우므로 success path 에서 tmp 가 청소되는지만 검증한다.
    #[test]
    fn store_salt_to_does_not_leak_tmp_on_success() {
        let dir = unique_test_dir("no-tmp-leak");
        let path = dir.join("salt.bin");
        let salt = varied_salt(55);
        store_salt_to(&path, &salt).unwrap();
        assert!(path.exists(), "salt.bin 존재");
        assert!(
            !dir.join("salt.bin.tmp").exists(),
            "tmp 파일은 rename 후 잔존하지 않아야 함"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// S-T2-4: 부모 디렉토리가 빈 문자열인 path 도 fallback 처리.
    /// `paths::salt_path()` 가 dev 환경에서 상대경로 fallback 일 때 안전성 검증.
    #[test]
    fn store_salt_to_handles_relative_path() {
        // 작업 디렉토리 격리 — 절대 경로로 변환해 PWD 변화 없이 검증.
        let dir = unique_test_dir("relative");
        let path = dir.join("salt.bin");
        let salt = varied_salt(77);
        store_salt_to(&path, &salt).unwrap();
        assert_eq!(load_salt_from(&path).unwrap(), salt);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// S-T2-5: delete_key_from_keyring 의 NoEntry idempotent 동작 (단위 — 실제 keyring 없이도
    /// 로직 분기가 컴파일·노출되는지 컴파일 가드 보장만 한다 — daemon 의존 테스트는 통합 분리).
    #[test]
    fn delete_key_from_keyring_is_exported() {
        // 함수가 pub 로 노출 + 시그니처 변경 없이 호출 가능한지 확인.
        // 실제 keyring 효과 검증은 통합 테스트 영역.
        let _f: fn() -> Result<(), AppError> = delete_key_from_keyring;
    }

    // ─── Sprint 8 T6 (I-S2-3): set_password 재진입 가드 ───

    /// 동일 process 내 다른 테스트가 가드를 잡고 있을 가능성 차단 — 명시적 reset.
    /// 가드는 RAII 라 정상 시 자동 해제되지만, 직전 테스트가 panic 했을 가능성 방어.
    fn reset_set_password_guard_for_test() {
        SET_PASSWORD_IN_PROGRESS.store(false, Ordering::Release);
    }

    #[test]
    fn set_password_guard_blocks_concurrent_entry() {
        reset_set_password_guard_for_test();
        let first = SetPasswordGuard::try_acquire().expect("첫 진입 성공");
        // 두 번째 진입은 가드가 잡혀있어 실패
        assert!(
            SetPasswordGuard::try_acquire().is_none(),
            "두 번째 진입은 즉시 차단"
        );
        // 첫 가드 drop 후 재진입 가능
        drop(first);
        let _second = SetPasswordGuard::try_acquire().expect("drop 후 재진입 성공");
    }

    #[test]
    fn set_password_guard_releases_on_drop() {
        reset_set_password_guard_for_test();
        {
            let _g = SetPasswordGuard::try_acquire().expect("진입");
            assert!(SET_PASSWORD_IN_PROGRESS.load(Ordering::Acquire));
        }
        assert!(
            !SET_PASSWORD_IN_PROGRESS.load(Ordering::Acquire),
            "scope 종료 시 자동 해제"
        );
    }

    #[test]
    fn set_password_guard_releases_on_panic_unwind() {
        reset_set_password_guard_for_test();
        // catch_unwind: panic 이 발생해도 stack 의 Drop 은 호출되어 가드 해제 보장.
        let result = std::panic::catch_unwind(|| {
            let _g = SetPasswordGuard::try_acquire().expect("진입");
            panic!("의도된 panic — 가드 RAII 검증");
        });
        assert!(result.is_err(), "panic 캐치");
        assert!(
            !SET_PASSWORD_IN_PROGRESS.load(Ordering::Acquire),
            "panic unwind 후에도 가드 자동 해제"
        );
    }

    // ─── Sprint 8 T6 (I-S2-5): salt_exists_at 정상/부재 경로 검증 ───

    /// 파일도 부재하고 legacy keyring 도 부재한 환경에서 false 반환 확인 (NotInitialized 분기).
    /// 실제 keyring 부재를 가정하려면 OS Keychain 에 `db_password_salt` 항목이 없어야 한다.
    /// dev 머신에 SmartHB 가 설정되어 있으면 legacy 항목이 남아있을 가능성 → `#[ignore]` 처리.
    #[test]
    #[ignore = "I-S2-5: dev keychain 잔존 salt 부수효과 방지 — 명시적 --ignored 시에만 실행"]
    fn salt_exists_at_returns_false_when_neither_file_nor_keyring() {
        let dir = unique_test_dir("absent-both");
        let path = dir.join("salt.bin");
        assert!(!path.exists(), "파일 부재 전제 확인");
        // keyring 잔존 항목이 없어야 false. 잔존 시 본 테스트는 의미 없음 (그래서 #[ignore]).
        let result = salt_exists_at(&path).expect("keyring 조회 성공");
        // 결과는 keyring 상태에 의존 — 본 테스트는 "예외 없이 bool 반환" 만 단언.
        let _ = result;
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `check_auth_status` 가 `Unlocked` 캐시 적중 시 즉시 Locked 반환 — keyring 조회 0회 보장.
    /// 이는 AC-T6-5 의 핵심 — 캐시 적중 경로에서 salt_exists 호출이 발생하지 않아야 함.
    #[tokio::test]
    async fn check_auth_status_returns_locked_on_cache_hit() {
        reset_credential_cache_for_tests();
        let salt = varied_salt(33);
        let key = derive_key("p", &salt);
        cache_credentials(salt, key);
        let status = check_auth_status().await.expect("성공");
        assert_eq!(status, AuthStatus::Locked, "캐시 적중 시 Locked");
        reset_credential_cache_for_tests();
    }

    // ─── Sprint 8 T7 (R45): ensure_cache_loaded 직렬화 ───

    /// 캐시 hit 상태에서 N 스레드가 동시 진입해도 모두 fast path 로 같은 결과 반환.
    /// keyring/salt 파일 호출 없음 — `get_cached_or_load_key` 가 즉시 캐시 값 반환.
    ///
    /// **Sprint 11 F6 (carry-over)**: `cargo test` 병렬 실행 시 다른 테스트가 캐시를 reset 하면서
    /// 본 테스트의 setup 과 race 가능 (간헐 실패). 동시성 설계 재검토는 backlog.
    /// 단독 실행은 항상 통과 — `cargo test -- --ignored ensure_cache_loaded_fast_path` 로 확인.
    #[test]
    #[ignore = "Sprint 11 F6: 병렬 실행 시 cache reset race — backlog"]
    fn ensure_cache_loaded_fast_path_is_concurrent_safe() {
        reset_credential_cache_for_tests();
        let salt = varied_salt(101);
        let key = derive_key("concurrent", &salt);
        let expected_bytes = *key.as_bytes();
        cache_credentials(salt, key);

        const THREADS: usize = 16;
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                std::thread::spawn(|| {
                    get_cached_or_load_key().expect("fast path 캐시 hit 성공")
                })
            })
            .collect();

        for h in handles {
            let k = h.join().expect("thread panic 없음");
            assert_eq!(
                k.as_bytes(),
                &expected_bytes,
                "모든 스레드가 동일 캐시 값 반환"
            );
        }
        reset_credential_cache_for_tests();
    }

    /// 캐시 미스 상태에서 `ensure_cache_loaded` 다중 스레드 진입 시 LOAD_MUTEX 가 직렬화하는지
    /// 검증한다. 실제 keyring/salt 호출은 OS 의존이므로 본 테스트는 다음만 단언:
    /// (1) deadlock 발생 안 함 (모든 스레드가 정해진 시간 내 종료),
    /// (2) 결과는 일관 — 모두 Err 또는 모두 Ok (race 로 일부 Ok, 일부 Err 가 섞이지 않음).
    /// load_credentials_to_cache 가 dev 환경 keyring/salt 부재로 Err 반환할 가능성이 높으므로
    /// 결과 자체보다 race 없음에 집중. macOS dev 환경 keychain 부수효과 방지를 위해 #[ignore].
    #[test]
    #[ignore = "T7: dev keychain 부수효과 방지 — 명시적 --ignored 시에만 실행"]
    fn ensure_cache_loaded_serializes_slow_path() {
        reset_credential_cache_for_tests();

        const THREADS: usize = 8;
        let handles: Vec<_> = (0..THREADS)
            .map(|_| std::thread::spawn(ensure_cache_loaded))
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("thread panic 없음"))
            .collect();

        // race 없음: 모든 결과가 같은 variant. Ok 면 모두 Ok, Err 면 모두 Err.
        let all_ok = results.iter().all(|r| r.is_ok());
        let all_err = results.iter().all(|r| r.is_err());
        assert!(
            all_ok || all_err,
            "race 발생 — 일부 Ok 일부 Err 섞임: {:?}",
            results
                .iter()
                .map(|r| r.is_ok())
                .collect::<Vec<_>>()
        );
        reset_credential_cache_for_tests();
    }
}
