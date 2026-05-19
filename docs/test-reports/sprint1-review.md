# 코드 리뷰 보고서 — Sprint 1 (2026-05-19)

> 검토 범위: `develop..sprint1` (14 commits, 43 files, +5467 / -57 lines)
> 검토 기준: `.claude/skills/code-review.md` + `backend.md` + `frontend.md`

---

## 전체 요약

| 등급 | 건수 |
|------|------|
| Critical | 0 |
| High | 0 |
| Medium | 3 |
| Low | 3 |
| Praise | 6 |

**종합 판정**: 배포 진행 가능. Medium 이슈는 후속 Sprint에서 해소 권장.

---

## AI 생성 코드 추가 체크

- 비즈니스 로직이 sprint1.md 계획과 일치하는가: **일치** (T1~T11 모두 계획 문서 목표 달성)
- 의도치 않은 파일 변경 없음: **확인** (`git diff --stat` 기준 범위 내)
- 하드코딩된 테스트 데이터·더미 값이 프로덕션 코드에 남아있지 않음: **확인**
- AI가 추가한 의존성 충돌 없음: **확인** (`Cargo.toml` + `pnpm` 의존성 모두 정상)

---

## 보안

### praise: SQLCipher 키 메모리 보호 (`src-tauri/src/commands/auth.rs:70-98`)

`DerivedKey([u8; KEY_LEN])` 에 `ZeroizeOnDrop` derive를 적용하여 Drop 시 자동 메모리 폐기가 구현되었다. `Debug` trait 수동 구현으로 `"DerivedKey([REDACTED])"` 마스킹도 철저하다. 비밀번호를 `Zeroizing<String>`으로 감싸 IPC 수신 즉시 보호하는 패턴도 일관되게 적용되어 있다.

### praise: 타이밍 공격 방어 (`src-tauri/src/commands/auth.rs:91-97`)

`DerivedKey::matches`에서 XOR 누적 constant-time 비교를 직접 구현하였다. `==` 비교가 아닌 모든 바이트를 순회하는 방식으로 비교 시간이 입력 내용에 무관하다.

### praise: PRAGMA key SQL 인젝션 방지 (`src-tauri/src/commands/paths.rs:36-38`)

`pragma_key_sql`이 hex 인코딩된 문자열(`[0-9a-f]`)만 허용하도록 설계되어 있으며, 주석에도 안전성 근거가 명시되어 있다. `format!` 사용이지만 hex 입력이 인젝션 통로가 될 수 없다.

### praise: 민감 데이터 audit 로그 미기록 (`src-tauri/src/commands/audit.rs:14-15`)

`try_record` 호출 7개 전체에서 비밀번호, 복구 코드 평문/해시, 암호화 키가 `details`에 기록되지 않는다. 모듈 헤더 주석에도 보안 원칙이 명시되어 있다.

### medium: `KEYRING_USER_SALT`의 Keychain 저장이 임시 설계임을 프로덕션까지 망각할 위험 (`src-tauri/src/commands/auth.rs:51`)

Salt는 비밀이 아니므로 Keychain 저장은 임시 방편이다. 코드 주석에 T9 마법사 통합 시 클라우드 폴더의 평문 파일로 이전한다고 명기되어 있어 의도는 분명하지만, 이전이 누락되면 양 PC에서 같은 Keychain 항목을 공유해야 해서 클라우드 동기화가 제대로 동작하지 않는다.

대응: Sprint 2 이후 초기 설정 마법사 구현 시 이전 작업을 명시적으로 계획에 포함할 것.

### medium: `release_lock`의 read-then-delete가 advisory lock 없이 실행 (`src-tauri/src/commands/lock.rs:240-253`)

`read_lock_info()`와 `std::fs::remove_file()` 사이에 fs2 advisory lock이 없다. `acquire_lock_atomic`은 read→판정→write를 단일 advisory lock 범위에서 수행하지만, `release_lock`은 별도 `File::open`+`try_lock_exclusive` 없이 파일을 삭제한다. 극단적 타이밍에서 다른 디바이스가 점유한 직후 우리 쪽이 삭제할 수 있다.

대응: 위험도는 낮다 (단일 사용자 모델, 정상 종료 경로). 후속 Sprint에서 release에도 advisory lock 획득 후 삭제하도록 개선 검토.

---

## 성능

### question: `derive_key` 600K PBKDF2가 spawn_blocking 없이 호출되는 경로 존재 여부 (`src-tauri/src/commands/auth.rs:120-125`)

`derive_key` 자체는 동기 함수이며 `derive_key_async`가 `spawn_blocking`으로 감싸는데, 테스트 코드에서 `derive_key`를 직접 호출한다. 프로덕션 IPC 경로는 모두 `derive_key_async`를 거치므로 문제없지만, 미래 contributor가 실수할 가능성이 있다.

대응: `derive_key`에 `#[cfg(not(test))]` 가시성 제한 또는 주석으로 호출 제약을 명시하는 것 검토.

### medium: audit_logs `list_logs` SQL 분기 — `since` 있을 때 `LIMIT` 위치가 다름 (`src-tauri/src/commands/audit.rs:113-124`)

`since` 없는 경우 `ORDER BY created_at DESC LIMIT ?`, `since` 있는 경우 `WHERE created_at >= ? ORDER BY created_at DESC LIMIT ?`. 두 쿼리 모두 `idx_audit_logs_created_at_desc` 인덱스를 활용할 수 있으나, `since` 필터 없이 전체 조회 시 데이터가 누적되면 성능이 저하될 수 있다. Sprint 1 범위에서는 이벤트 건수가 적으므로 문제없음.

대응: 감사 로그 건수가 수천 건을 초과하는 Sprint에서 keyset 페이지네이션 도입 검토.

---

## 코드 품질

### praise: `AppError::user_message` 패턴 (`src-tauri/src/error.rs:58-80`)

`From<AppError> for String`이 기술 디테일 없이 한국어 사용자 메시지만 반환하도록 일관되게 구현되었다. `Display` trait은 기술 상세를 보존하여 로그용으로 분리된 설계가 명확하다.

### praise: `app_err!` 매크로 통합 (`src-tauri/src/commands/runtime.rs:47-52`)

backup/integrity/lock 모듈에 흩어져 있던 `AppError::Variant(format!(...))` 패턴을 단일 매크로로 통합한 리팩토링이 깔끔하다. 모든 variant에서 컴파일 가능성을 테스트로 보장하는 점도 좋다.

### nitpick: `src-tauri/src/commands/db.rs:21` `#![allow(dead_code)]` 모듈 레벨 허용

모듈 전체에 `dead_code`를 허용하면 실제 미사용 코드가 섞여들 경우 경고를 놓칠 수 있다. T10 통합 완료 후 이 허용을 제거하고 필요한 항목만 `#[allow(dead_code)]`로 개별 지정하는 것이 권장된다.

### nitpick: `src-tauri/src/commands/sync.rs:43` `DateTime::from_timestamp` deprecation

`chrono 0.4.31+` 에서 `from_timestamp`는 deprecated. `DateTime::from_timestamp(secs, 0)` 대신 `DateTime::from_timestamp_opt(secs, 0).unwrap_or_default()` 또는 `Utc.timestamp_opt(secs, 0).single()` 사용 권장. clippy가 현재 경고를 발생시키지 않는다면 해당 버전이 미적용일 수 있다.

### low: `src/lib/tauri/index.ts:93` 개발 모드 복구 코드 mock `'DEVM-ODEX-TEST'`

`generateRecoveryCode()` fallback이 `'DEVM-ODEX-TEST'`를 반환하는데, `verifyRecoveryCode()` fallback도 이 값과만 일치하도록 하드코딩되어 있다. 개발 모드 테스트에서는 문제없지만 일관성 문서화가 없어 이후 다른 개발자가 혼동할 수 있다.

### low: `src/components/LockScreen.tsx:184-186` Unicode 이모지 아이콘 사용

`'🙈'` / `'👁'`을 비밀번호 토글 버튼에 사용하는 것은 임시방편으로 코멘트에도 명기되어 있다. lucide-react 도입 시 교체 예정이나 백로그 등록 여부를 명확히 할 것.

---

## 테스트

cargo test 통과: **74건 전체 통과 (0 실패)**

### 모듈별 테스트 커버리지

| 모듈 | 테스트 건수 | 비고 |
|------|------------|------|
| `auth.rs` | 11건 | 재현성, 솔트 유일성, constant-time 비교, Debug 마스킹 |
| `backup.rs` | 9건 | 파일명 파싱, 순환 삭제, 빈 디렉토리 처리 |
| `integrity.rs` | 8건 | PRAGMA 결과 분류, rollback 파일명, serde |
| `lock.rs` | 8건 | is_self, is_stale, JSON serde, OnceLock 안정성 |
| `audit.rs` | 5건 | event_type codes, serde, silent fail |
| `error.rs` | 6건 | user_message 한국어, 기술 디테일 비노출 |
| `runtime.rs` | 5건 | run_blocking, app_err! 매크로 |
| `sync.rs` | 5건 | mtime 분기, serde |
| `startup.rs` | 3건 | 상수, serde, fail-soft |
| `recovery.rs` | 5건 | 코드 생성, 알파벳, 정규화 |
| `db.rs` (cipher off) | 4건 | URL 생성, 마이그레이션, UNIQUE 제약 |
| `paths.rs` | 2건 | 경로 조합 |

### 미수행 테스트 (의도적 제외)

- Keychain 통합 테스트 (OS daemon 의존): 사용자 환경 검증으로 위임
- Argon2id 해시 + Keychain 통합: 사용자 환경 검증으로 위임
- cipher on 빌드의 전체 시작 시퀀스 통합 테스트: 사용자 환경 검증으로 위임 (`DEPLOY.md ⬜` 항목)

---

## 패턴 준수

- Tauri IPC 래퍼 (`src/lib/tauri/index.ts`): 전체 커맨드 추상화 완료, 컴포넌트에서 `invoke()` 직접 호출 없음
- 파일/디렉토리 구조: `src-tauri/src/commands/` 하위 모듈 분리 준수
- TypeScript 타입: `src/types/index.ts`에 모든 IPC 타입 중앙화 완료
- SSR 가드: `src/lib/tauri/index.ts:21`에 `typeof window !== 'undefined'` 가드 적용
- 접근성: 잠금 화면 56px 입력 필드, 44×44px 버튼, role="alert" 에러 메시지

---

## 배포 준비도 사전 확인

| 항목 | 결과 |
|------|------|
| CHANGELOG.md `[Unreleased]` 섹션 | 업데이트됨 (Sprint 1 변경사항 전체 기재) |
| 하드코딩 시크릿 패턴 | 없음 (변경된 .rs/.ts/.tsx 파일 전수 스캔) |
