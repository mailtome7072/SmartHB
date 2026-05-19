//! 복구 코드 발급·검증·재설정 (PI-07 PRD v1.5.1).
//!
//! ## 흐름
//!
//! 1. **발급** (`generate_recovery_code`): 사용자가 설정 메뉴에서 명시적으로 요청 →
//!    12자리 영숫자 생성 → Argon2id 해시를 keyring 에 저장 → 평문 코드를 1회 반환.
//!    프론트엔드는 화면에 표시하면서 "비밀번호와 코드 모두 분실 시 데이터 영구 접근 불가"
//!    경고를 함께 노출. 평문은 표시 후 즉시 폐기 (Zeroizing).
//!
//! 2. **검증** (`verify_recovery_code`): 입력 코드를 Argon2id 로 해시 비교 (constant-time).
//!
//! 3. **비밀번호 재설정** (`reset_password_with_code`): 검증 성공 시 새 비밀번호로 salt + key
//!    재유도 후 keyring 갱신. 본 T5 에서는 SQLCipher DB rekey 는 포함하지 않는다 — T9
//!    (DB pool + 시작 시퀀스 통합) 시점에 통합된다.
//!
//! ## 보안
//!
//! - **알파벳**: Crockford Base32 변형 (`23456789ABCDEFGHJKLMNPQRSTUVWXYZ`) — 32자.
//!   0/O/1/I/L 혼동 문자 제거로 사용자가 종이에 받아 적을 때 오독 방지.
//! - **엔트로피**: 32^12 ≈ 1.15×10^18 ≈ 60비트 — Argon2id 메모리-하드 해시와 결합으로
//!   오프라인 무차별 대입 비용 충분.
//! - **해시 파라미터**: Argon2id m=19456 KiB, t=2, p=1 (OWASP 2024 권장).
//! - **PHC 문자열**: salt + 파라미터 + 해시가 단일 문자열에 포함 → keyring 1개 항목으로 충분.
//! - **평문 폐기**: `Zeroizing<String>` 으로 IPC 응답 직후 메모리 영(0) 덮어쓰기.

use crate::commands::auth::{
    derive_key_async, generate_salt, keyring_entry_for, keyring_get_or_none, store_key_in_keyring,
    store_salt_in_keyring,
};
use crate::error::AppError;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand::Rng;
use zeroize::Zeroizing;

/// 복구 코드 PHC 해시 Keychain user 식별자.
const KEYRING_USER_RECOVERY: &str = "recovery_code_hash";

/// 영숫자 알파벳 — Crockford Base32 변형 (혼동 문자 0/O/1/I/L 제거).
///
/// **실제 31자** (B"..." 바이트 리터럴 길이는 31). 32^12 (60비트) 대비 31^12 ≈ 59.4비트 —
/// 약 0.5비트 엔트로피 감소, 무차별 대입 비용 영향 미미.
const CODE_ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ";

/// 복구 코드 길이.
const CODE_LEN: usize = 12;

/// Argon2id 인스턴스 생성 (OWASP 2024 권장 파라미터).
fn argon2_instance() -> Argon2<'static> {
    // Params::new(m_cost, t_cost, p_cost, output_len)
    let params = Params::new(19_456, 2, 1, None)
        .expect("Argon2 파라미터는 컴파일 타임에 고정");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// 12자리 복구 코드를 CSPRNG 로 생성한다.
///
/// 반환된 `Zeroizing<String>` 은 IPC 응답 직렬화 직후 메모리에서 폐기된다.
fn generate_code_plaintext() -> Zeroizing<String> {
    let mut rng = rand::rngs::OsRng;
    let mut code = String::with_capacity(CODE_LEN);
    for _ in 0..CODE_LEN {
        let idx = rng.gen_range(0..CODE_ALPHABET.len());
        code.push(CODE_ALPHABET[idx] as char);
    }
    Zeroizing::new(code)
}

/// 정규화 — 사용자가 입력한 코드에서 공백·하이픈을 제거하고 대문자로 통일한다.
///
/// 표시 형식은 `XXXX-XXXX-XXXX` (4-4-4) 이지만 저장 형식은 dash 없는 12자.
fn normalize_input_code(raw: &str) -> Zeroizing<String> {
    let cleaned: String = raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    Zeroizing::new(cleaned)
}

/// Keychain 에 PHC 해시 문자열을 저장 (덮어쓰기).
///
/// `auth::keyring_entry_for` 헬퍼를 재사용 — service 식별자 + 에러 처리 일관성 보장.
/// PHC 문자열은 hex 가 아니므로 `store_bytes_in_keyring` 대신 직접 `set_password` 사용.
fn store_recovery_hash(phc: &str) -> Result<(), AppError> {
    keyring_entry_for(KEYRING_USER_RECOVERY)?
        .set_password(phc)
        .map_err(|e| AppError::Auth(format!("복구 코드 해시 저장 실패: {}", e)))?;
    Ok(())
}

/// Keychain 에서 PHC 해시 문자열을 조회.
///
/// 항목 부재 시 `AppError::Auth` — 발급되지 않은 상태에서 검증 요청이 들어왔다는 의미.
fn retrieve_recovery_hash() -> Result<Zeroizing<String>, AppError> {
    keyring_get_or_none(KEYRING_USER_RECOVERY)?.ok_or_else(|| {
        AppError::Auth(
            "복구 코드가 발급되어 있지 않습니다. 설정 메뉴에서 먼저 발급해주세요.".to_string(),
        )
    })
}

// ----------------------------------------------------------------------------
// Tauri IPC commands
// ----------------------------------------------------------------------------

/// 복구 코드를 발급한다.
///
/// 이미 발급된 코드가 있으면 무효화하고 새 코드를 생성한다 (재발급 정책 — PRD v1.5.1).
/// 반환된 12자리 평문 코드는 프론트엔드가 화면에 1회 표시 후 즉시 폐기해야 한다.
#[tauri::command]
pub async fn generate_recovery_code() -> Result<String, String> {
    let plaintext = generate_code_plaintext();
    let phc = tokio::task::spawn_blocking({
        let plaintext_owned = (*plaintext).clone();
        move || -> Result<String, AppError> {
            let salt = SaltString::generate(&mut rand::rngs::OsRng);
            let hash = argon2_instance()
                .hash_password(plaintext_owned.as_bytes(), &salt)
                .map_err(|e| AppError::Auth(format!("Argon2id 해시 실패: {}", e)))?;
            Ok(hash.to_string())
        }
    })
    .await
    .map_err(|e| AppError::Auth(format!("해시 작업 실패: {}", e)))
    .and_then(|r| r)
    .map_err(String::from)?;

    store_recovery_hash(&phc).map_err(String::from)?;
    Ok((*plaintext).clone())
}

/// 사용자가 입력한 복구 코드를 검증한다.
///
/// 입력 코드는 공백·하이픈을 제거하고 대문자로 통일하여 비교한다 (사용자 가독성).
/// Argon2id `verify_password` 가 constant-time 비교를 보장.
#[tauri::command]
pub async fn verify_recovery_code(code: String) -> Result<bool, String> {
    let normalized = normalize_input_code(&code);
    let phc = retrieve_recovery_hash().map_err(String::from)?;
    let result = tokio::task::spawn_blocking({
        let normalized_owned = (*normalized).clone();
        let phc_owned = (*phc).clone();
        move || -> Result<bool, AppError> {
            let parsed = PasswordHash::new(&phc_owned)
                .map_err(|e| AppError::Auth(format!("저장된 해시 파싱 실패: {}", e)))?;
            Ok(argon2_instance()
                .verify_password(normalized_owned.as_bytes(), &parsed)
                .is_ok())
        }
    })
    .await
    .map_err(|e| AppError::Auth(format!("검증 작업 실패: {}", e)))
    .and_then(|r| r)
    .map_err(String::from)?;
    Ok(result)
}

/// 복구 코드로 비밀번호를 재설정한다.
///
/// 흐름: 코드 검증 → 새 비밀번호로 salt + key 재생성 → keyring 갱신.
/// SQLCipher DB rekey 는 T9 시점에 통합된다 (현재는 keyring 만 갱신).
#[tauri::command]
pub async fn reset_password_with_code(code: String, new_password: String) -> Result<(), String> {
    let new_password = Zeroizing::new(new_password);
    if new_password.is_empty() {
        return Err(AppError::Auth("새 비밀번호가 비어있습니다.".to_string()).into());
    }
    let valid = verify_recovery_code(code).await?;
    if !valid {
        return Err(AppError::Auth("복구 코드가 일치하지 않습니다.".to_string()).into());
    }
    let salt = generate_salt();
    let new_key = derive_key_async(new_password, salt)
        .await
        .map_err(String::from)?;
    store_salt_in_keyring(&salt).map_err(String::from)?;
    store_key_in_keyring(&new_key).map_err(String::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_code_has_correct_length() {
        let code = generate_code_plaintext();
        assert_eq!(code.len(), CODE_LEN);
    }

    #[test]
    fn generated_code_uses_only_allowed_alphabet() {
        let code = generate_code_plaintext();
        for c in code.chars() {
            assert!(
                CODE_ALPHABET.contains(&(c as u8)),
                "허용되지 않은 문자: {} (코드: {})",
                c,
                code.as_str()
            );
        }
    }

    #[test]
    fn generated_code_excludes_confusing_chars() {
        // 100번 생성하여 혼동 문자 0/O/1/I/L 이 한 번도 등장하지 않음을 확인
        for _ in 0..100 {
            let code = generate_code_plaintext();
            for c in code.chars() {
                assert!(
                    !matches!(c, '0' | 'O' | '1' | 'I' | 'L'),
                    "혼동 문자 등장: {} (코드: {})",
                    c,
                    code.as_str()
                );
            }
        }
    }

    #[test]
    fn generated_codes_are_unique() {
        // 32^12 엔트로피에서 두 번 생성이 같을 확률 = 2^-60 ≈ 0
        let c1 = generate_code_plaintext();
        let c2 = generate_code_plaintext();
        assert_ne!(c1.as_str(), c2.as_str());
    }

    #[test]
    fn normalize_input_strips_spaces_and_dashes() {
        let raw = "abcd-efgh ijkl";
        let normalized = normalize_input_code(raw);
        assert_eq!(normalized.as_str(), "ABCDEFGHIJKL");
    }

    #[test]
    fn normalize_input_uppercases() {
        let normalized = normalize_input_code("abcdef");
        assert_eq!(normalized.as_str(), "ABCDEF");
    }

    // Argon2id 해시 + Keychain 통합 테스트는 OS Keychain daemon 및 ~50ms 해시 비용 의존 —
    // 본 단위 테스트에서는 제외하고 T11 통합 테스트 또는 사용자 환경 검증에서 다룬다.
}
