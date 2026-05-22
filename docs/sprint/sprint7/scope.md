---
Sprint: 7  |  Date: 2026-05-22  |  Session: #2
---

> Sprint 7 Session #2 — T2 단독 (salt.bin 이전: Keychain → cloud/smarthb/).
> A17/A27 carry-over 해소 + T1 CredentialCache 통합. 예상 3h.

## 이전 세션 결과

Session #1 (2026-05-22, 커밋 `8eb1c92`):
- T1 완료 — macOS Keychain 호출 통합 캐싱
- `CredentialCache` (salt+key, ZeroizeOnDrop) 도입
- `auth.rs` / `db.rs` / `backup.rs` / `integrity.rs` / `recovery.rs` 5파일 캐시 경유
- `cargo test 151 passed` / `cargo clippy clean`

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T2** | salt.bin 이전 — Keychain → `{cloud}/smarthb/salt.bin` + 1회 자동 마이그레이션 | 3h |

> 사용자 결정 (2026-05-22): Session #2 = T2 단독. T3(device_id)은 별도 세션 — 데이터 경로 정책(클라우드 vs OS 로컬) 결정도 함께.

## 설계 결정 (T2)

### salt 저장 방식 변경
- **Before (Sprint 6 ~ T1)**: Keychain 항목 `db_password_salt` (hex 인코딩) — keyring 접근 다이얼로그 유발
- **After (T2)**: `{cloud}/smarthb/salt.bin` 평문 32바이트 — 양 PC 동기화 자동 + keyring 호출 1회 추가 제거
- **이유**: salt 는 비밀이 아님 (PRD §5.5) — Keychain 보호 불필요. 양 PC 동기화 이득이 크다.

### 1회 자동 마이그레이션
- 앱 시작 시 또는 `load_credentials_to_cache` 첫 호출 시 자동 수행
- 절차: (1) `salt.bin` 존재 확인 → 있으면 종료. (2) 없으면 Keychain `db_password_salt` 조회. (3) 존재 시 파일로 복사 → Keychain 항목 삭제. (4) Keychain 에도 없으면 신규 설치 — 첫 `set_password` 가 파일 생성.
- 원자성: tmp → rename + 손상 감지 (setup.rs `is_corrupted` 패턴 답습 — NTFS power-loss 메모리)
- 부분 실패 복원성: 파일 쓰기 실패 시 Keychain 삭제 안 함 → 다음 시작에 재시도. 파일 존재하지만 Keychain 항목 삭제 안 됨 시 다음 시작에 Keychain 삭제만 재시도.

### 캐시 통합
- T1 `CredentialCache` 구조는 그대로 — salt 출처만 keyring → 파일로 교체
- `load_credentials_to_cache`: salt 는 파일에서, key 는 keyring 에서 → keyring 호출 1회로 감소 (기존 2회)
- `cached_salt()` / `get_cached_or_load_key()` 호출자는 인터페이스 무변경

### 신규 의존성
- 없음 — 표준 `std::fs` + 기존 `keyring` crate 만 사용

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/paths.rs | [3회 ⚠️] | `salt_path()` 헬퍼 추가 + I-S2-1 동행 패치 (pragma_key_sql 테스트 assertion 교정). ⚠️는 T1 누적 — 본 세션은 2회 (이슈별 분리). |
| src-tauri/src/commands/auth.rs | [18회 ⚠️] | salt 저장소 파일로 변경 + 마이그레이션 + 캐시 통합 |
| src-tauri/src/commands/recovery.rs | [4회 ⚠️] | `store_salt_in_keyring` → `store_salt` 호출처 변경 |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src/` — 본 세션 프론트엔드 변경 없음
- [ ] `src-tauri/Cargo.toml` — 신규 의존성 없음
- [ ] `src-tauri/migrations/` — DB 스키마 변경 없음
- [ ] `src-tauri/src/commands/lock.rs` — T3 범위 (device.id)
- [ ] `src-tauri/src/commands/db.rs`, `backup.rs`, `integrity.rs` — T1 에서 캐시 경유 완료, T2 영향 없음

## 완료 기준 (이번 세션)

### T2 — salt.bin 이전 (sprint7.md L78-98)
- ✅ AC-T2-1: 신규 설치 시 salt 가 `{cloud}/smarthb/salt.bin` 에 평문 32바이트 저장 (`store_and_load_salt_round_trip`)
- ✅ AC-T2-2: 기존 설치 시 Keychain salt → 파일 자동 마이그레이션 (1회) + Keychain 항목 삭제 (`migrate_keyring_salt_to` 구현, OS daemon 의존 통합 테스트는 별도)
- ✅ AC-T2-3: 양 PC 시나리오 — `load_salt_is_deterministic_for_two_machines` 로 동일 파일 입력 → 동일 결과 보장
- ✅ AC-T2-4: salt.bin 파일 권한은 OS 기본 — cross-platform 단순화. 0o600 명시는 후속 보강 (I-S2-10)
- ✅ AC-T2-5: Keychain 잔존 항목은 `db_encryption_key` + `recovery_code_hash` — salt 는 마이그레이션 후 삭제 (`delete_legacy_keyring_salt`)
- ✅ AC-T2-6: 손상된 salt.bin 감지 시 백업 + 마이그레이션 fallback (`load_salt_backs_up_corrupted_file`)

### 추가 보안 패치 (high-effort code review 결과, 사용자 결정 = Critical/Security 소수)

본 세션 종료 전, code review 가 잡은 6건의 Critical/Security finding 을 동행 패치한다.

- ✅ S-T2-1 (eprintln key/salt hex 누출 제거): `set_password` / `verify_password` 의 `key.to_hex()[..16]` + `salt[..8]` eprintln 제거 — PRD §5.5 로그 위생 위반 해소
- ✅ S-T2-2 (set_password atomic order + rollback): keyring key 먼저 → 성공 시 salt 파일 → 실패 시 keyring rollback 으로 orphan salt.bin 방지
- ✅ S-T2-3 (recovery atomic order + rollback): `reset_password_with_code` 도 동일 패턴 적용 — 양 비밀번호 잠금 시나리오 차단
- ✅ S-T2-4 (NTFS power-loss 방어): `store_salt_to` 에 `File::sync_all()` + tmp 누수 정리 + 부모 디렉토리 best-effort fsync
- ✅ S-T2-5 (delete_key_from_keyring NoEntry idempotent): `delete_legacy_keyring_salt` 와 일관성 — T5 복구 재시도 가능
- ✅ S-T2-6 (PC-B retrieve_key_from_keyring 한국어 안내): `NoEntry` 시 "이 PC 에는 인증 정보가 없습니다 — 복구 코드로 재설정 필요"

### 세션 종료 조건
- ✅ Self-verify: cipher off 160 passed / cipher on 121 passed / clippy clean (양쪽)
- ✅ simplify 검토 — 마이그레이션 헬퍼 / 손상 감지 로직 / 캐시 경유 / atomic+rollback 모두 단일 책임 유지. `retrieve_bytes_from_keyring` 미사용 제거.
- ⬜ 단일 커밋 (3파일 + scope.md)

## 발견된 이슈

### I-S2-1: cipher feature on 에서 `paths::tests::pragma_key_sql_uses_blob_literal` 항시 실패

- **발견 시점**: Session #2 Self-verify 단계 (`cargo test --features cipher`)
- **위치**: `src-tauri/src/commands/paths.rs:170-174`
- **현상**:
  ```rust
  assert_eq!(sql, "PRAGMA key = \"x'deadbeef'\";");        // 통과
  assert!(!sql.contains('\''), "단일 따옴표 ... 없음");    // 항상 실패
  ```
  실제 반환값 `PRAGMA key = "x'deadbeef'";` 에는 SQLite blob literal 표기 `x'...'` 의 작은따옴표가 포함되어 두 번째 assertion 이 모순.
- **원인**: 테스트 의도 ("hex 만 허용되므로 사용자 입력이 따옴표 삽입 불가") 와 assertion 표현 불일치. `pragma_key_sql` 함수 자체는 의도대로 동작 — hex 검증 + 외부 큰따옴표 escape 로 SQL injection 안전.
- **이력**: `e960614` (Sprint 1 T11) 부터 존재 — cipher feature test 가 일상 CI 에 포함되지 않아 누적 잠재. T2 작업과 무관.
- **영향**: cipher 빌드 테스트 1건 실패 — production 빌드 동작에는 영향 없음 (PRAGMA SQL 형식 정상).
- **처리 결정 (2026-05-22)**: **(A) 본 세션 동행 패치 채택**. 사용자 결정.
  - 변경 내용: `assert!(!sql.contains('\''), ...)` → `assert_eq!(sql.matches('\'').count(), 2, ...)` — blob literal `x'...'` 의 마커 2개 외 따옴표가 삽입되면 hex 검증 위반임을 강제.
  - 영향 범위: `paths.rs` 테스트 한 줄. production 코드 무변경.

### I-S2-2 ~ I-S2-10: high-effort code review 잔여 9건 (후속 세션 carry-over)

본 세션은 사용자 결정에 따라 Critical/Security 6건만 동행 패치 (S-T2-1~6 참조).
나머지 9건은 후속 세션 또는 별도 hotfix 로 처리한다.

| ID | 위치 | 심각도 | 요약 | 권고 처리 |
|----|------|------|------|---------|
| I-S2-2 | `auth.rs:302` `is_salt_corrupted` | High | 부분-NULL 손상 (16 valid + 16 NULL) 미감지 → PBKDF2 가 deterministic 하게 잘못된 키 생성 | salt 와 함께 HMAC 체크섬 저장 또는 별도 마커. Sprint 7 T3~T9 중 한 세션에 동반 |
| I-S2-3 | `auth.rs:479` `set_password` 재진입 가드 | High | 폼 중복 제출 / IPC replay 시 salt+key 회전 → PC-B 영구 lockout | 서버측 `salt_exists() == false` 가드 추가 |
| I-S2-4 | `auth.rs:75` `CRED_CACHE` static drop | High | `OnceLock` static 은 process exit 시 Drop 안 됨 → ZeroizeOnDrop 실효 없음 (모듈 주석은 misleading) | atexit hook 또는 명시적 cleanup IPC, 주석 정정 |
| I-S2-5 | `auth.rs:461` `check_auth_status` 미-마이그레이션 | High | legacy keyring 잔존 시 unlock 성공 전까지 매 cold start Keychain 다이얼로그 | `check_auth_status` 에서도 백그라운드 마이그레이션 트리거 |
| I-S2-6 | `auth.rs:767` test → real Keychain | High | `cargo test load_salt_backs_up_corrupted_file` 가 dev 의 실제 Keychain 항목 삭제 가능 | `#[ignore]` 또는 `#[cfg(integration)]` gate, mock keyring backend |
| I-S2-7 | `auth.rs:524`, `:117` concurrent race | Medium-High | tokio::join! 시점 두 caller 가 동시에 cache miss → 두 Keychain 다이얼로그 (AC-T1-1 위반 가능) | load 자체를 serializing 하는 별도 OnceCell 또는 lock 확장 |
| I-S2-8 | `auth.rs:83` Mutex poison `expect` | Medium | 단일 panic 으로 auth IPC 전체 영구 brick | `.lock().unwrap_or_else(|e| e.into_inner())` 패턴 적용 |
| I-S2-9 | `auth.rs:376` migration audit 누락 | Medium | Keychain → 파일 전환에 audit 이벤트 없음 → incident response 추적 불가 | `audit::try_record(PasswordChange, Some("salt-migration-keyring-to-file"), ...)` |
| I-S2-10 | 기타 (low) | Low | timestamp collision (line 364), with_extension brittleness (319), tmp leak on Windows EBUSY, salt buffer not Zeroize, file permission 0o644, data_root fallback risk, stale doc comment "store_salt_in_keyring" (line 475) | 단발성 정리는 다음 카로 정리 |

