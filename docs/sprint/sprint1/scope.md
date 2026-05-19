---
Sprint: 1  |  Date: 2026-05-19  |  Session: #9 (T9 진입)
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
- **Session #9** (T9 동기화 + 감사 로그 + 코드 테이블): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T9 — Day 9 첫 번째 작업)

**동기화 대기 + 감사 로그 + 코드 테이블 인프라**

### DB 마이그레이션 (src-tauri/migrations/ 신규)

- `V001__create_code_tables.sql` — 코드 테이블 4종 + 시드 데이터
  - `schools` (학교명·학년 매핑)
  - `payment_methods` (결제 수단)
  - `card_companies` (카드사)
  - `standard_fees` (표준 교습비)
- `V008__create_app_settings_and_audit_logs.sql`
  - `app_settings` (key/value — 마법사 결과, 단축키 커스텀)
  - `audit_logs` (id / created_at / event_type / event_subject / details)
  - INDEX: `audit_logs(created_at DESC)` 시간순 조회 최적화

> V001~V008 사이 V002~V007 갭은 후속 sprint(원생 / 수업 스케줄 / 출결 / 청구 등 도메인 테이블) 예약.

### 백엔드 모듈 (3개 신규)

- `src-tauri/src/commands/db.rs` — sqlx pool 초기화 + `PRAGMA key` 적용 + 마이그레이션 실행
  - `OnceCell<SqlitePool>` 패턴으로 unlock 후 lazy 초기화
  - cipher feature on/off 양쪽 호환 (off 는 평문 SQLite, on 은 SQLCipher)
- `src-tauri/src/commands/sync.rs` — 동기화 대기 (PRD §5.3)
  - DB / 락 파일 mtime 확인으로 최신 동기화 판단
  - 30초 타임아웃 + 사용자 새로고침 옵션 (UI 가 IPC 재호출)
  - IPC: `check_sync_status`
- `src-tauri/src/commands/audit.rs` — 감사 로그
  - `record(event_type, event_subject?, details?)` 내부 헬퍼 — 다른 모듈이 호출
  - IPC: `get_audit_logs(since?, limit?)` — 시간 역순 페이지네이션
  - 민감 데이터 마스킹은 호출자 책임 (audit::record 호출 시점에 필터링)
  - 1년 롤링 정리는 T10 시작 시퀀스에서 호출 (본 sprint 에서는 헬퍼만 노출)

### 새 의존성

- 없음 — sqlx 0.8 이미 매크로/migrate feature 포함

### Feature 게이트

- cipher off 빌드: 평문 SQLite 로 마이그레이션 + 모든 IPC 정상 동작 (개발 빌드 호환)
- cipher on 빌드: 첫 connection에서 `PRAGMA key` 적용 후 마이그레이션
- 본 sprint 의 T1~T8 에서 cipher off 빌드도 검증해왔으므로 일관성 유지

### 프론트엔드

- `src/types/index.ts`: `SyncStatus`, `AuditEventType`, `AuditLogEntry` 타입
- `src/lib/tauri/index.ts`: `checkSyncStatus`, `getAuditLogs` 래퍼

### audit 로깅 호출 통합 (보류)

backup/lock/auth/recovery 등 기존 모듈에서 `audit::record` 호출은 **T10 시작 시퀀스 통합** 단계로 미룬다 — DB pool 초기화 lifecycle 결정이 T10 의 startup 시퀀스 설계와 묶이기 때문. 본 sprint 에서는 audit 인프라 자체만 제공.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/V001__create_code_tables.sql | [0회] | 신규 — schools/payment_methods/card_companies/standard_fees + 시드 |
| src-tauri/migrations/V008__create_app_settings_and_audit_logs.sql | [0회] | 신규 — app_settings + audit_logs + index |
| src-tauri/src/commands/db.rs | [0회] | 신규 — SqlitePool OnceCell + PRAGMA key + migrate |
| src-tauri/src/commands/sync.rs | [0회] | 신규 — check_sync_status IPC (mtime 기반) |
| src-tauri/src/commands/audit.rs | [0회] | 신규 — record/get_audit_logs/cleanup 헬퍼 + IPC |
| src-tauri/src/commands/mod.rs | [0회] | `pub mod db; pub mod sync; pub mod audit;` 추가 |
| src-tauri/src/lib.rs | [0회] | invoke_handler 에 IPC 2개 등록 |
| src-tauri/Cargo.toml | [0회] | (만약 sqlx feature 누락 시) `migrate` feature 확인 — 이미 포함 예상 |
| src/lib/tauri/index.ts | [0회] | checkSyncStatus / getAuditLogs 래퍼 |
| src/types/index.ts | [0회] | SyncStatus / AuditEventType / AuditLogEntry 타입 |
| docs/sprint/sprint1/scope.md | [0회] | 본 파일 — Session #9 갱신 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/`, `.claude/` — 정책·에이전트
- ⬜ `PRD.md`, `ROADMAP.md`, `docs/phase/`, `docs/sprint/sprint1.md` — 계획·사양 SSOT
- ⬜ `.env` — 개발 환경 한정 파일, 사용자가 직접 관리 (`.env.example` 참고)
- ⬜ 기존 모듈 (`auth.rs`, `lock.rs`, `backup.rs`, `integrity.rs`, `recovery.rs`) — audit 로깅 호출 통합은 T10 으로 미룸

## 이번 세션의 완료 기준 (T9)

- ⬜ 마이그레이션 2개 SQL 파일 작성 — UNIQUE/CHECK 제약 PRD §6.2 정합
- ⬜ `db::pool()` 또는 동등 헬퍼 — `OnceCell` 또는 `tokio::sync::OnceCell` 패턴
- ⬜ `db::initialize(password)` 또는 동등 — keyring 키 조회 → PRAGMA key → migrate 실행
- ⬜ `check_sync_status` IPC — DB/락 파일 mtime 비교 + 30초 타임아웃 명세
- ⬜ `audit::record(event_type, subject?, details?)` 내부 헬퍼
- ⬜ `get_audit_logs(since?, limit?)` IPC — 시간 역순 페이지네이션, 기본 limit 100
- ⬜ `cargo test` 통과 — 마이그레이션 인메모리 DB 로 적용 검증 + 모듈 단위 테스트
- ⬜ `cargo clippy -- -D warnings` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` 통과

## 보안·성능 원칙 (T1~T8 연장)

- DB pool 은 unlock 후에만 초기화 (사용자 비밀번호 검증 통과 시점) — pre-auth 코드 실행 차단
- audit 로깅 시 민감 데이터(비밀번호, 복구 코드 해시, hex key) 절대 미기록 — 호출자가 사전 마스킹
- mtime 기반 동기화 판단은 advisory — 클라우드 동기화 지연 시 false positive 가능, 사용자 새로고침 보완
- 마이그레이션은 SQLCipher 적용 후 실행 — pool 초기화 함수가 PRAGMA key → migrate 순서 보장

## sqlx 도입 결정

- **macro vs query()**: 본 T9 에서는 동적 `sqlx::query()` + `bind()` 사용 — `query!()` 매크로 도입은 `.sqlx` 오프라인 캐시 + `.env` 환경 정리 작업이 별도 필요. backend.md 정책의 핵심("raw concat 금지, bind 사용")은 동적 함수로도 충족.
- T10 또는 T11 에서 `query!()` 매크로 전환 검토 — 후속 sprint 의 도메인 테이블(원생·스케줄) 추가 시 컴파일 타임 schema 검증 가치 커짐.

## 추후 세션 계획 (참고)

- **Session #10**: T10 앱 시작 시퀀스 통합 + 성능 검증 (< 3초)
  - audit 로깅 호출 통합 (backup/lock/auth/recovery)
  - hourly 백업 백그라운드 task 시작
  - audit_logs 1년 롤링 정리
- **Session #11**: T11 단위 테스트 + CI 양 OS 빌드 검증 + cipher feature on 매트릭스 추가

본 scope.md는 각 세션 시작 시 갱신한다.
