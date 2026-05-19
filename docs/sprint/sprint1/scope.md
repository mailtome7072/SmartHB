---
Sprint: 1  |  Date: 2026-05-19  |  Session: #10 (T10 진입)
---

## 세션 진행 기록

- **Session #1** (T1 SQLCipher PoC + ADR-001): ✅ 완료. commit `10bf105`.
- **Session #2** (T2 에러 처리 기반): ✅ 완료. commit `9ba7f6a`.
- **Session #3** (T3 Keychain + PBKDF2 + ADR-004): ✅ 완료. commit `8e17324`.
- **Session #4** (T4 인증 IPC + 잠금 화면 UI): ✅ 완료. commit `c00fa7e`.
- **Session #5** (T5 PI-07 복구 코드 Argon2id): ✅ 완료. commit `80eb975`.
- **Session #6** (T6 app.lock + ADR-002): ✅ 완료. commit `eba6456`.
- **Session #7** (T7 4계층 백업 + ADR-003): ✅ 완료. commit `67be493`.
- **Session #8** (T8 무결성 검증 + 자동 복원): ✅ 완료. commit `b82d678`.
- **Session #9** (T9 동기화 + 감사 로그 + 코드 테이블): ✅ 완료. commit `01737c3`.
- **Session #10** (T10 앱 시작 시퀀스 통합 + 성능 검증): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T10 — Day 9 두 번째 작업)

**앱 시작 시퀀스 + 성능 검증 + audit 호출 통합 + 백그라운드 task**

### 시작 시퀀스 (PRD §5.6 < 3초 목표)

```
(UI) checkSyncStatus → Ready 까지 폴링 (예산 외)
    ↓
(UI) 사용자 비밀번호 입력
    ↓
app_startup_sequence(password, force_lock=false)
    ├── 측정 시작 (Instant::now)
    ├── [tokio::join!] acquire_lock(force) + check_integrity(quick) 병렬
    ├── 비밀번호 검증 (salt → derive_key → matches)
    ├── db::initialize(db_path) — pool + PRAGMA key + WAL + cache_size + migrate
    ├── audit::cleanup_older_than(365) (best-effort)
    ├── 백그라운드 spawn: lock heartbeat (60초) + hourly 백업 (1시간)
    └── 측정 종료 → StartupResult 반환
```

- `tokio::join!` 으로 락 확인 + 무결성 검증 병렬화
- PRAGMA 설정: `journal_mode=WAL`, `cache_size=-8000`, `foreign_keys=ON` (db::initialize 내부)
- 성능 측정: 시작 ~ 메인 진입 < 3초 (동기화 대기·사용자 입력 제외)
- IPC 1개: `app_startup_sequence`
- 산출물: `src-tauri/src/startup.rs` (sprint1.md 계획대로 commands/ 외부에 위치)

### audit 호출 통합 (best-effort 정책)

`audit::record` 는 pool 미초기화 시 silent fail — startup 흐름을 차단하지 않는다.
호출 추가 위치:

| 모듈 | 호출 지점 | 이벤트 | 비고 |
|------|----------|-------|------|
| `auth::set_password` | 성공 직후 | PasswordChange | pool 미초기화 시점이라 자연 skip |
| `recovery::generate_recovery_code` | 해시 저장 직후 | RecoveryCodeIssued | unlock 후 호출되므로 정상 기록 |
| `recovery::reset_password_with_code` | keyring 갱신 직후 | PasswordChange | 마찬가지 |
| `lock::acquire_lock(force=true)` | 강제 점유 성공 시 | LockForced | startup 전 호출 가능, silent fail |
| `backup::create_backup_sync` | 성공 직후 | BackupCreated | layer 를 subject 에 |
| `integrity::restore_from_path_sync` | 성공 직후 | BackupRestored | 경로를 subject 에 |
| `integrity::run_pragma_check` | Failed 반환 직전 | IntegrityCheckFailed | detail 첫 줄을 details 에 |

민감 데이터(비밀번호, 복구 코드 평문/해시, hex key, salt) 절대 미포함.

### 백그라운드 task

- **heartbeat**: 60초 interval, `lock::acquire_lock_atomic(false)` 재호출하여 mtime 갱신.
  - startup 후 spawn, AbortHandle 을 lib.rs 의 `OnceLock<TaskHandles>` 에 보관.
- **hourly 백업**: 60분 interval, `backup::create_backup(Hourly)` 호출.
  - cipher off 빌드에서는 즉시 stub 에러 → 첫 시도 시 silent skip (반복 로그 노이즈 방지).
- **exit 백업**: app 종료 직전 동기 호출 — Tauri `RunEvent::ExitRequested` 또는
  `on_window_event` 의 `CloseRequested` 에서 `block_on(create_backup(Exit))`.

### 새 의존성

- 없음 — tokio `full` feature 에 spawn / interval / Instant 포함.

### Feature 게이트

- cipher off 빌드: backup·integrity·DB pool 의 SQLCipher 경로가 모두 안내 메시지 반환.
  startup 시퀀스는 cipher off 에서도 시연 가능하도록 무결성 검증 실패를 fail-soft 처리:
  pool 초기화는 평문 SQLite 로 진행 (PRAGMA key 적용 단계 skip).
- cipher on 빌드: 전체 흐름 정상 동작.

### 프론트엔드

- `src/types/index.ts`: `StartupResult` 타입 (elapsed_ms / integrity / lock_acquired).
- `src/lib/tauri/index.ts`: `appStartupSequence(password, forceLock?)` 래퍼.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/startup.rs | [0회] | 신규 — app_startup_sequence IPC + 측정 + 백그라운드 spawn |
| src-tauri/src/lib.rs | [0회] | startup 모듈 추가 + invoke_handler 등록 + RunEvent ExitRequested hook |
| src-tauri/src/commands/audit.rs | [0회] | `try_record` best-effort 헬퍼 추가 (record wrapper) |
| src-tauri/src/commands/auth.rs | [0회] | set_password / 내부 verify 함수에 audit 호출 |
| src-tauri/src/commands/recovery.rs | [0회] | generate / reset 에 audit 호출 |
| src-tauri/src/commands/lock.rs | [0회] | acquire force=true 성공 시 audit 호출, lock_path pub(crate) 노출 |
| src-tauri/src/commands/backup.rs | [0회] | create_backup_sync 성공 시 audit 호출 |
| src-tauri/src/commands/integrity.rs | [0회] | restore_from_path_sync 성공 시 audit, check Failed 시 audit |
| src-tauri/src/commands/db.rs | [0회] | initialize 시 PRAGMA WAL/cache_size/foreign_keys 적용 |
| src/lib/tauri/index.ts | [0회] | appStartupSequence 래퍼 추가 |
| src/types/index.ts | [0회] | StartupResult 타입 추가 |
| docs/sprint/sprint1/scope.md | [0회] | 본 파일 — Session #10 갱신 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/`, `.claude/` — 정책·에이전트
- ⬜ `PRD.md`, `ROADMAP.md`, `docs/phase/`, `docs/sprint/sprint1.md` — 계획·사양 SSOT
- ⬜ `.env` — 개발 환경 한정 파일
- ⬜ `src-tauri/migrations/` — T9 에서 확정, T10 변경 없음

## 이번 세션의 완료 기준 (T10)

- ⬜ `src-tauri/src/startup.rs` 신규 — `app_startup_sequence` IPC 작성
- ⬜ 락 + 무결성 `tokio::join!` 병렬 실행
- ⬜ db::initialize 가 PRAGMA `journal_mode=WAL`, `cache_size=-8000`, `foreign_keys=ON` 적용
- ⬜ audit 호출 통합 7곳 (silent fail wrapper 적용)
- ⬜ heartbeat 백그라운드 task spawn (60초 interval)
- ⬜ hourly 백업 백그라운드 task spawn (60분 interval, cipher off 시 skip)
- ⬜ exit 백업 종료 hook (RunEvent 또는 CloseRequested)
- ⬜ `audit::cleanup_older_than(365)` startup 호출
- ⬜ `lib.rs` invoke_handler 에 `app_startup_sequence` 등록
- ⬜ 프론트엔드 `appStartupSequence` 래퍼 + `StartupResult` 타입
- ⬜ `cargo test` 통과 — startup 단위 테스트 (인메모리 pool + 측정 결과)
- ⬜ `cargo clippy -- -D warnings` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` 통과

## 보안·성능 원칙 (T1~T9 연장)

- `app_startup_sequence` 는 사용자 비밀번호 인자 받자마자 `Zeroizing<String>` 으로 감싸 폐기 보장
- 백그라운드 task 는 pool 초기화 후 spawn — 미초기화 race 방지
- audit best-effort: 호출 실패가 startup 흐름을 차단하지 않음 (silent log to stderr 만)
- cipher off 빌드에서 backup/integrity 가 거부되어도 startup 자체는 성공 — 개발 빌드 호환

## simplify 보류 항목 (T10 또는 T11)

본 세션에서는 audit 통합 + 시작 시퀀스에 집중. 아래는 T11 까지 보류:

- `commands/paths.rs` 분리 (data_root / db_path / pragma_key_sql)
- `commands/runtime.rs` 분리 (run_blocking)
- `*_err` 매크로 통합 (backup/integrity/lock 3중)
- keyring 키 조회 + hex 변환 헬퍼 (4중 중복)
- backend.md V{NNN} 표기 sqlx 정합 정리

## 추후 세션 계획 (참고)

- **Session #11**: T11 단위 테스트 보강 + CI 양 OS 빌드 매트릭스 + cipher feature on 검증
  - simplify 보류 항목 일괄 처리
  - `.github/workflows/` 수정 시 사용자 명시 허가 필수 (Forbidden Area)

본 scope.md는 각 세션 시작 시 갱신한다.
