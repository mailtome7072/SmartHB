---
Sprint: 1  |  Date: 2026-05-19  |  Session: #7 (T7 진입)
---

## 세션 진행 기록

- **Session #1** (T1 SQLCipher PoC + ADR-001): ✅ 완료. commit `10bf105`.
- **Session #2** (T2 에러 처리 기반): ✅ 완료. commit `9ba7f6a`.
- **Session #3** (T3 Keychain + PBKDF2 + ADR-004): ✅ 완료. commit `8e17324`.
- **Session #4** (T4 인증 IPC + 잠금 화면 UI): ✅ 완료. commit `c00fa7e`.
- **Session #5** (T5 PI-07 복구 코드 Argon2id): ✅ 완료. commit `80eb975`.
- **Session #6** (T6 app.lock + ADR-002): ✅ 완료. commit `eba6456`.
- **Session #7** (T7 4계층 백업 + ADR-003): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T7 — Day 7~8 첫 번째 작업)

**4계층 자동 백업 + ADR-003** · skill: brainstorming

### 백엔드 (src-tauri/src/commands/backup.rs 신규)

- 4계층 백업: `exit(10)` / `hourly(24)` / `daily(30)` / `weekly(4)` 순환 삭제
- 파일명 규칙: `app_YYYYMMDD_HHMMSS.db`
- 백업 위치: T7 임시로 `./SmartHB-data/backup/{exit,hourly,daily,weekly}/` (dev), T9 마법사 통합 시 클라우드 동기화 폴더 하위로 이전
- 백업 후 `PRAGMA quick_check` 검증 — 실패 시 파일 삭제 + 에러 반환
- 백업 파일은 SQLCipher 암호화 상태 그대로 보관 (복호화 금지)
- `hourly` 백업: tokio 백그라운드 task (UI 비블로킹) — T10 startup sequence에서 시작
- `exit` 백업: 동기 실행 (~50ms 예산, 종료 직전 보장)
- ADR 문서: `docs/arch/adr-003-backup-implementation.md` (brainstorming)

### IPC 3개

- `create_backup(layer: BackupLayer)` — 지정 계층에 백업 생성
- `list_backups(layer: Option<BackupLayer>)` — 백업 메타데이터 조회 (path, created_at, size)
- `restore_backup(path: String)` — 지정 백업으로 복원 (T8에서 확장)

### 새 의존성

- `rusqlite = { version = "0.32", features = ["bundled-sqlcipher-vendored-openssl"] }` — SQLite Online Backup API 전용 보조 의존성
  - 주의: sqlx 0.8 + libsqlite3-sys 0.30 과 SQLCipher 빌드 호환성 검증 필요 → ADR-003에서 다룬다
  - `cipher` feature off 시 동작 방식도 ADR에서 결정

### 보안·성능 원칙 (T1·T3·T6 연장)

- 백업 파일도 SQLCipher 키와 분리 보관 — 키 파일 미생성, 키는 Keychain만
- 백업 시작 전 락 acquire 확인 (T6 락 점유자만 백업 가능)
- `tokio::fs` 사용, `tokio::task::spawn_blocking` 으로 백업 I/O 처리
- 동시 백업 호출 방지: backup module 내부 `Arc<Mutex>` 또는 sqlx pool 단일 호출

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/Cargo.toml | [0회] | `rusqlite` crate 추가 (cipher feature 연동) |
| src-tauri/src/commands/backup.rs | [0회] | 신규 — BackupLayer enum + 4계층 순환 + IPC 3개 |
| src-tauri/src/commands/mod.rs | [0회] | `pub mod backup;` 추가 |
| src-tauri/src/lib.rs | [0회] | invoke_handler 에 IPC 3개 등록 |
| src-tauri/src/error.rs | [0회] | (필요 시 `Backup` 에러 variant 확장) |
| docs/arch/adr-003-backup-implementation.md | [0회] | 신규 — brainstorming (Weighted Matrix + SWOT) |
| src/lib/tauri/index.ts | [0회] | `createBackup`, `listBackups`, `restoreBackup` 래퍼 |
| src/types/index.ts | [0회] | `BackupLayer`, `BackupMetadata` 타입 |
| docs/sprint/sprint1/scope.md | [0회] | 본 파일 — Session #7 갱신 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/` — 정책 문서
- ⬜ `.claude/` — 에이전트/룰/스킬/훅
- ⬜ `PRD.md`, `ROADMAP.md`, `docs/phase/`, `docs/sprint/sprint1.md` — 계획·사양 SSOT
- ⬜ `CLAUDE.md` — 이전 턴 `/init`의 ESLint 섹션 정정이 unstaged 상태. T7과 무관하므로 별도 chore 커밋 또는 사용자 결정 후 처리 (T7 커밋에 포함 금지)

## 이번 세션의 완료 기준 (T7)

- ⬜ `rusqlite` 0.32 + `bundled-sqlcipher-vendored-openssl` 의존성 추가 + `cipher` feature 연동
- ⬜ `cargo build --features cipher --manifest-path src-tauri/Cargo.toml` 성공 (Windows Strawberry Perl 환경)
- ⬜ `BackupLayer` enum 4종 (Exit, Hourly, Daily, Weekly) + 계층별 최대 보관 개수 상수
- ⬜ `create_backup` IPC 동작 — Online Backup API 사용, `quick_check` 검증 후 파일 확정
- ⬜ `list_backups` IPC 동작 — 4계층 디렉토리 스캔, 메타데이터 반환
- ⬜ `restore_backup` IPC 스텁 (실복원 흐름은 T8과 통합)
- ⬜ 4계층 순환 삭제 로직 — 계층별 최대 개수 초과 시 가장 오래된 파일 삭제
- ⬜ `cargo test` 통과 — backup 모듈 단위 테스트 (인메모리 + 임시 디렉토리)
- ⬜ `cargo clippy -- -D warnings` 통과
- ⬜ ADR-003 작성 — Weighted Matrix + SWOT 적용 (rusqlite vs sqlx raw connection 등 후보 비교)

## brainstorming 스킬 적용 — ADR-003 작성 방법

T7은 sprint1.md에서 `skill: brainstorming` 으로 명시되었다. ADR-003 후보 옵션:

1. **(A) `rusqlite` + `bundled-sqlcipher-vendored-openssl`** — Online Backup API 직접 사용, sqlx와 분리된 connection
2. **(B) `sqlx` raw connection에서 SQLite C API 호출** — 별도 crate 없이 sqlx pool 재사용, FFI 직접 호출
3. **(C) `sqlx` query로 `VACUUM INTO` 사용** — Online Backup API 미사용, 트랜잭션 일관성 검토 필요

### Weighted Matrix 평가축

- 빌드 단순성 (Windows Perl 등 추가 의존 회피)
- SQLCipher 호환성 (암호화 상태 유지 여부)
- 성능 (~50ms exit 백업 목표)
- 트랜잭션 일관성 (백업 중 쓰기 충돌)
- 라이브러리 신뢰성·유지보수

### SWOT 분석 (각 옵션별 S/W/O/T)

ADR 본문에 두 분석 결과 + 권장안 + 결정 + Consequence 명시.

## 추후 세션 계획 (참고)

- **Session #8**: T8 무결성 검증 + 자동 복원 (`PRAGMA quick_check/integrity_check` + 손상 시 자동 복원)
- **Session #9**: T9 동기화 대기 + 감사 로그 + 코드 테이블 (DB 마이그레이션 V001/V008)
- **Session #10**: T10 앱 시작 시퀀스 통합 + 성능 검증 (< 3초 목표)
- **Session #11**: T11 단위 테스트 + CI 양 OS 빌드 검증

본 scope.md는 각 세션 시작 시 갱신한다.
