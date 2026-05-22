---
Sprint: 7  |  Date: 2026-05-22  |  Session: #1
---

> Sprint 7 (Phase 2 carry-over 해소) — T1 단독 세션.
> macOS Keychain 반복 다이얼로그 + startup 31초 Critical UX 해소.
> CredentialCache 도입으로 후속 keyring 호출 제거.
> 예상 5h. skill: systematic-debugging.

## 이전 세션 결과

Sprint 7 첫 세션 — 이전 작업 없음.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1** | macOS Keychain 호출 통합 캐싱 — CredentialCache + 5파일 캐시 경유 변경 | 5h |

> 사용자 결정(2026-05-22): Session #1 = T1 단독, T3 device.id = OS 로컬(app_config_dir).

## Systematic Debugging 적용 (T1 skill)

### 1단계: 증상 정의 (What)
- **실제 동작**: 비밀번호 입력 후 macOS Keychain 다이얼로그가 여러 차례 반복 표시, startup 31545ms (sprint6 시각 검증 로그 확인됨)
- **기대 동작**: 다이얼로그 최대 1회 (prod 빌드 + "Always Allow" 가정), startup < 3000ms (PRD §5.6)
- **재현 조건**: macOS dev 빌드 `pnpm tauri:dev`, salt + key 모두 keyring 저장된 상태에서 비밀번호 잠금 해제

### 2단계: 범위 파악 (Where)
keyring 호출처 (dev 빌드, cipher off):
- `auth.rs::check_auth_status` — salt 1회 조회 (line 242)
- `auth.rs::verify_password` — salt 1회 (line 292) + key 1회 (line 302)
- 합계: dev 빌드에서 startup 시 최대 3회 keyring 호출 — 각 호출이 별도 macOS Security Framework 다이얼로그 유발

cipher on 빌드 추가 호출:
- `db.rs::apply_cipher_key_if_enabled` — key 1회
- `backup.rs::create_backup` — key 1회 (백업 시점)
- `integrity.rs::check_integrity` / `auto_restore` — key 2곳
- prod 빌드 총 합산 ~7회 keyring 호출 가능

### 3단계: 가설 설정 (Why)
- **가설 A**: macOS Security Framework가 동일 process라도 다른 (service, user) 항목 access마다 별도 다이얼로그 → 캐싱 미적용 → 통합 캐시로 후속 호출 제거 가능 ✅
- **가설 B**: `OnceLock<Uuid>` device_id가 새 UUID라 audit 등에서 추가 keyring 호출 → device_id 영속화(T3) 가 더 효과적 (Sprint 7 T3에서 처리, 본 세션 외)
- **가설 C**: PBKDF2 600K iter 자체가 30초 → 측정 로그에 PBKDF2 만 ~500ms 라 가설 기각

→ **가설 A 채택**, 가설 B는 T3에서 추가 효과 (Session 외).

### 4단계: 최소 재현
- 단위 테스트로 캐시 hit/miss/invalidation 검증 (keyring 직접 호출 없이)
- `CredentialCache` 가 in-memory 구조이므로 keyring mock 불필요

### 5단계: 수정 → 검증
- CredentialCache 도입 → 5파일 캐시 경유 → cargo test + clippy

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/auth.rs | [10회 ⚠️] | CredentialCache 신규, verify/set/check 캐시 경유, delete 시 invalidate |
| src-tauri/src/commands/db.rs | [1회] | apply_cipher_key_if_enabled 캐시 경유 (cipher feature on 한정) |
| src-tauri/src/commands/backup.rs | [1회] | create_backup 캐시 경유 (cipher feature on 한정) |
| src-tauri/src/commands/integrity.rs | [2회] | check_integrity / auto_restore 2곳 캐시 경유 (cipher feature on 한정) |
| src-tauri/src/commands/recovery.rs | [4회 ⚠️] | reset_password_with_code 후 캐시 갱신 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src/` — 본 세션 프론트엔드 변경 없음
- [ ] `src-tauri/Cargo.toml` — 신규 의존성 없음
- [ ] `src-tauri/src/commands/paths.rs` — T2/T3에서 다룸
- [ ] `src-tauri/src/commands/lock.rs` — T3에서 다룸

## 완료 기준 (이번 세션)

### T1 — Keychain 캐싱 (PRD §5.5·§5.6, sprint7.md L46-75)
- ⬜ AC-T1-1: prod 빌드 다이얼로그 1회 검증 — **T10 통합 검증으로 이연** (본 세션은 코드 + 단위 테스트까지)
- ⬜ AC-T1-2: dev 빌드 startup 단축 검증 — **시각 검증으로 이연** (Session 외)
- ✅ AC-T1-3: `verify_password` 가 캐시 hit 시 keyring 0회, miss 시 `load_credentials_to_cache` 통합 호출 1회 — 직접 `retrieve_*_from_keyring` 호출 0
- ✅ AC-T1-4: `set_password` / `reset_password_with_code` 가 `cache_credentials` 호출로 캐시 즉시 갱신
- ✅ AC-T1-5: `CachedCredentials` 에 `#[derive(ZeroizeOnDrop)]` — Drop 시 메모리 zero
- ✅ AC-T1-6: 기존 11개 auth 테스트 + 신규 cache 테스트 5건 (cache_miss / cache_then_read / invalidate / overwrites / get_cached_or_load_key) → `cargo test 151 passed`

### 세션 종료 조건
- ✅ 단일 커밋 (5파일 수정 + scope.md, 본 커밋)
- ✅ Self-verify: `cargo test 151 passed` / `cargo clippy -- -D warnings` clean (cipher off + on 양쪽)
- ✅ simplify 검토 — CredentialCache 헬퍼 4개 + 캐시 경유 호출처 3곳 모두 단일 책임, 추상화 적절

## 설계 결정

### CredentialCache 구조
```rust
#[derive(Zeroize, ZeroizeOnDrop)]
struct CachedCredentials {
    salt: [u8; SALT_LEN],
    key: DerivedKey,  // 이미 ZeroizeOnDrop
}

static CRED_CACHE: OnceLock<Mutex<Option<CachedCredentials>>> = OnceLock::new();

fn cred_cache() -> &'static Mutex<Option<CachedCredentials>> {
    CRED_CACHE.get_or_init(|| Mutex::new(None))
}
```

### 캐시 로드 전략
- **Lazy load**: 첫 verify_password 호출 시 keyring 2회 호출(salt + key) → 캐시 채움
- 후속 호출(같은 process 내): 캐시 hit, keyring 호출 0회
- prod 빌드의 "Always Allow"가 적용되면 첫 호출 다이얼로그도 1회로 통합 가능

### check_auth_status 처리
- 캐시에 cred 있으면 → Locked 반환 (keyring 호출 X)
- 캐시 비어있으면 → keyring 1회 조회 (현재와 동일) — 다만 cred 전체를 미리 로드하지는 않음 (사용자가 비밀번호 미입력 상태)

### invalidate 시점
- `delete_key_from_keyring` 후
- 비밀번호 변경 (현재 IPC 없음, 후속 sprint)
- 로그아웃 시 (현재 없음, Phase 후속)

### Salt + Key 동시 로드 (전용 함수)
```rust
fn load_credentials_to_cache() -> Result<(), AppError> {
    let salt = retrieve_salt_from_keyring()?;
    let key = retrieve_key_from_keyring()?;
    *cred_cache().lock().unwrap() = Some(CachedCredentials { salt, key });
    Ok(())
}
```

### 캐시 경유 헬퍼
```rust
pub(crate) fn get_cached_or_load_key() -> Result<DerivedKey, AppError> {
    let mut guard = cred_cache().lock().unwrap();
    if let Some(c) = guard.as_ref() {
        return Ok(DerivedKey(c.key.0));  // Copy
    }
    drop(guard);
    load_credentials_to_cache()?;
    let guard = cred_cache().lock().unwrap();
    Ok(DerivedKey(guard.as_ref().unwrap().key.0))
}
```

### cipher feature on 호출처 (db.rs / backup.rs / integrity.rs)
- 기존: `retrieve_key_from_keyring()` 직접 호출 → keyring access
- 변경: `get_cached_or_load_key()` 호출 → 캐시 hit 시 keyring 호출 X

## 코드 패턴 SSOT

- 비밀 데이터 zeroize: `zeroize::Zeroize + ZeroizeOnDrop` derive
- OnceLock + Mutex: 기존 lock.rs / paths.rs 패턴 답습
- 에러: `AppError::Auth` 한국어 메시지 유지
- 테스트: `#[cfg(test)] mod tests` 안에서 cred_cache 리셋 가능하도록 helper 추가 (또는 #[cfg(test)] 한정 reset 함수)

## 발견된 이슈

> 진행 중 새 제약·충돌 발견 시 여기에 기록.
