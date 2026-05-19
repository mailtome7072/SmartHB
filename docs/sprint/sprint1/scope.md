---
Sprint: 1  |  Date: 2026-05-19  |  Session: #8 (T8 진입)
---

## 세션 진행 기록

- **Session #1** (T1 SQLCipher PoC + ADR-001): ✅ 완료. commit `10bf105`.
- **Session #2** (T2 에러 처리 기반): ✅ 완료. commit `9ba7f6a`.
- **Session #3** (T3 Keychain + PBKDF2 + ADR-004): ✅ 완료. commit `8e17324`.
- **Session #4** (T4 인증 IPC + 잠금 화면 UI): ✅ 완료. commit `c00fa7e`.
- **Session #5** (T5 PI-07 복구 코드 Argon2id): ✅ 완료. commit `80eb975`.
- **Session #6** (T6 app.lock + ADR-002): ✅ 완료. commit `eba6456`.
- **Session #7** (T7 4계층 백업 + ADR-003): ✅ 완료. commit `67be493`.
- **Session #8** (T8 무결성 검증 + 자동 복원): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T8 — Day 7~8 두 번째 작업)

**무결성 검증 + 자동 복원**

### 백엔드 (src-tauri/src/commands/integrity.rs 신규)

- **검증 모드**: `quick`(PRAGMA quick_check, ~50ms, 앱 시작 시) / `full`(PRAGMA integrity_check, 일일 백업 시점)
- **자동 복원 흐름**:
  1. `backup/exit/` 디렉토리에서 가장 최신 백업 선택
  2. 후보 백업을 임시로 열어 `PRAGMA quick_check` 통과 확인 — 실패 시 다음 후보로 폴백
  3. 현재 DB 를 `restore_rollback/app_YYYYMMDD_HHMMSS.db` 로 이동 (복원 실패 시 되돌릴 수 있도록)
  4. 선택된 백업을 `app.db` 로 파일 복사 (SQLCipher 암호화 상태 그대로)
- **손상본 격리**: T8 에서는 `restore_rollback/` 1곳으로 단순화 — 손상본 보존 + 복원 전 rollback 보존 의미를 통합. PRD `corrupted/` 분리는 T10 startup 자동 감지 통합 시점에 분기 처리
- **IPC 2개**: `check_integrity(mode)`, `auto_restore()`
- **backup::restore_backup 실구현**: T7 스텁을 integrity 모듈의 헬퍼 호출로 교체 — 사용자가 specific path 지정 복원 시 동일 안전망(quick_check + rollback 보존) 적용

### Feature 게이트

- `cipher` feature on: rusqlite Online Backup API 기반 검증 + 복원 정식 동작
- `cipher` feature off: integrity 모듈도 stub — "암호화 빌드에서만 무결성 검증 가능" 안내 메시지

### 새 의존성

- 없음 (T7 에서 추가한 rusqlite + chrono + 기존 sqlx 재사용)

### 프론트엔드

- `src/types/index.ts`: `IntegrityMode`, `IntegrityCheckResult`, `RestoreResult` 타입
- `src/lib/tauri/index.ts`: `checkIntegrity`, `autoRestore` IPC 래퍼
- UI 컴포넌트는 T10 시작 시퀀스 통합 시점에 (현재는 IPC + 타입만)

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/integrity.rs | [0회] | 신규 — IntegrityMode + IntegrityCheckResult + RestoreResult + IPC 2개 |
| src-tauri/src/commands/backup.rs | [0회] | restore_backup 실구현 (integrity 헬퍼 호출) + `find_latest_backup`, `restore_from_path` 등 헬퍼 노출 |
| src-tauri/src/commands/mod.rs | [0회] | `pub mod integrity;` 추가 |
| src-tauri/src/lib.rs | [0회] | invoke_handler 에 IPC 2개 등록 |
| src/lib/tauri/index.ts | [0회] | checkIntegrity / autoRestore 래퍼 |
| src/types/index.ts | [0회] | IntegrityMode / IntegrityCheckResult / RestoreResult 타입 |
| docs/sprint/sprint1/scope.md | [0회] | 본 파일 — Session #8 갱신 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단). cipher feature on 빌드 매트릭스 변경은 T11 단계에서 사용자 허가 후 진행
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/` — 정책 문서
- ⬜ `.claude/` — 에이전트/룰/스킬/훅
- ⬜ `PRD.md`, `ROADMAP.md`, `docs/phase/`, `docs/sprint/sprint1.md` — 계획·사양 SSOT
- ⬜ `src-tauri/Cargo.toml` — T8 은 신규 의존성 없음. 추가 필요 시 사용자 확인

## 이번 세션의 완료 기준 (T8)

- ⬜ `IntegrityMode` enum (Quick / Full) + `IntegrityCheckResult` enum (Ok / Failed{detail})
- ⬜ `check_integrity(mode)` IPC — quick_check / integrity_check 결과 정확히 파싱 (다중 행 손상 메시지 포함)
- ⬜ `auto_restore()` IPC — exit 최신 백업 자동 선택 + quick_check 통과 후 복원 + rollback 보존
- ⬜ 복원 안전망: 후보 백업 손상 시 다음 후보로 폴백, rollback 디렉토리 자동 생성
- ⬜ `RestoreResult` — restored_from / rollback_path 반환 (UI 가 사용자에게 안내 가능)
- ⬜ `backup::restore_backup(path)` 실구현 — integrity 모듈 헬퍼 재사용
- ⬜ `cargo test` 통과 — 신규 모듈 단위 테스트 (모드 enum / 결과 파싱 / 디렉토리 헬퍼)
- ⬜ `cargo clippy -- -D warnings` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` 통과

## 보안·성능 원칙 (T7 연장)

- 복원 직전 현재 DB 는 반드시 `restore_rollback/` 에 보존 — 백업 자체가 손상된 경우 사용자가 수동 복원 가능
- 후보 백업 검증 실패 시 무결한 백업 찾을 때까지 폴백 — 시간순(최신→과거) 정렬
- exit 백업이 모두 손상되었으면 사용자에게 명확한 에러 메시지 (`AppError::Integrity`) — daily/weekly 자동 폴백은 T10 startup 통합 시점에 결정 (사용자 명시적 선택 필요)
- 키 메모리 최소 노출 — `retrieve_key_from_keyring()` 재사용, hex 즉시 `Zeroizing`

## 추후 세션 계획 (참고)

- **Session #9**: T9 동기화 대기 + 감사 로그 + 코드 테이블 (DB 마이그레이션 V001/V008)
- **Session #10**: T10 앱 시작 시퀀스 통합 + 성능 검증 (< 3초, 손상 자동 감지 + corrupted/ 격리 분리)
- **Session #11**: T11 단위 테스트 + CI 양 OS 빌드 검증 + cipher feature on 매트릭스 추가

본 scope.md는 각 세션 시작 시 갱신한다.
