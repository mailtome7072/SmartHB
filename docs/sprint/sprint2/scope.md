---
Sprint: 2  |  Date: 2026-05-20  |  Session: #3 (T9 원생 CRUD IPC)
---

## 세션 진행 기록

- **Session #1** (T1/T3/T4 — Sprint 1 잔여 해소): ✅ 완료. commits `a466541`, `84b18e5`, `052bb81`. T2 는 Sprint 3 마법사 통합으로 이연 (`c010423`).
- **Session #2** (T5~T7 — V101~V103 마이그레이션): ✅ 완료. commit `16edbd1`. T8 는 매크로 도입 시점으로 이연.
- **Session #3** (T9 — 원생 CRUD IPC + PI-05 자동 채번): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T5~T8 — Day 3~4)

**원생/스케줄/학사/표준교습비 DB 스키마 + sqlx 오프라인 캐시 갱신**

### T5: V101 students + student_schedules

- 파일: `src-tauri/migrations/101__create_students_and_schedules.sql`
- **SSOT 정합**: data-model.md §1.1, §1.2 우선 (sprint2.md 의 `INTEGER serial_no` / `REAL duration_hours` 표기는 미스 — data-model.md `TEXT serial_no` / `INTEGER duration_hours` 따름)
- `students`:
  - `serial_no TEXT NOT NULL UNIQUE` (PI-05 자동 채번은 숫자 문자열 생성)
  - `gender CHECK IN ('male', 'female')`
  - `school_level CHECK IN ('elementary', 'middle')` (PRD §4.1.1 초·중 한정)
  - `grade CHECK BETWEEN 1 AND 9` (초1~6 + 중1~3)
  - `school_id` FK → `schools(id)`
  - `withdraw_date >= enroll_date` CHECK (AC-4.1.1-4)
- `student_schedules`:
  - `day_of_week CHECK BETWEEN 1 AND 7` (1=월, 7=일)
  - `duration_hours INTEGER CHECK > 0` (시간 단위)
  - `effective_to NULL ALLOWED` (현행 스케줄)
  - 부분 인덱스: `UNIQUE(student_id, day_of_week) WHERE effective_to IS NULL` (R11 SQLite 3.8.0+ 의존)

### T6: V102 study_periods + schedule_codes

- 파일: `src-tauri/migrations/102__create_study_periods_and_schedule_codes.sql`
- `study_periods`: `year_month UNIQUE "YYYY-MM"`, `start_date`, `end_date`, `is_confirmed`, `is_closed`
- `schedule_codes`: 3속성 모델 (`allows_regular_class`, `allows_makeup_class`, `is_duplicate_blocked`) + 시스템 예약 5종 시드 (보강데이/공휴수업일/방학/단원평가 응시일/휴원일)

### T7: V103 schedule_events

- 파일: `src-tauri/migrations/103__create_schedule_events.sql`
- `schedule_events`: `code_id FK`, `event_date`, `period_end_date`, `display_name`

### T8: .sqlx 오프라인 캐시 갱신

- `cargo install sqlx-cli` (이미 설치 가정) → `sqlx migrate run` 으로 dev DB 에 적용
- `cargo sqlx prepare --manifest-path src-tauri/Cargo.toml` → `.sqlx/` 갱신
- `SQLX_OFFLINE=true cargo build --manifest-path src-tauri/Cargo.toml` 성공 확인

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/101__create_students_and_schedules.sql | [0회] | 신규 — T5 |
| src-tauri/migrations/102__create_study_periods_and_schedule_codes.sql | [0회] | 신규 — T6 |
| src-tauri/migrations/103__create_schedule_events.sql | [0회] | 신규 — T7 |
| src-tauri/.sqlx/ | [0회] | sqlx prepare 결과물 — T8 |
| docs/sprint/sprint2/scope.md | [0회] | 본 파일 — Session #2 갱신 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인
- ⬜ `SETUP.sh` — 초기화 스크립트
- ⬜ `docs/harness-engineering/`, `.claude/agents/`
- ⬜ `PRD.md`, `docs/phase/`, `docs/sprint/sprint2.md` (sprint-planner 결과물)
- ⬜ `src-tauri/migrations/001__*`, `008__*` (Sprint 1 산출물)
- ⬜ `src-tauri/src/commands/*.rs` (IPC 는 T9 이후)

## 이번 세션의 완료 기준 (T5~T8)

- ⬜ V101 마이그레이션 작성 — students + student_schedules + 부분 인덱스
- ⬜ V102 마이그레이션 작성 — study_periods + schedule_codes + 시스템 예약 5종 시드
- ⬜ V103 마이그레이션 작성 — schedule_events
- ⬜ `.sqlx/` 오프라인 캐시 갱신 + 커밋
- ⬜ `cargo test` 통과 (db::tests::in_memory_pool_runs_migrations 가 새 테이블도 검증)
- ⬜ `cargo clippy --all-targets -- -D warnings` 통과

## 다음 세션 예정 (참고)

## 이번 세션의 목표 (T1 — Day 1)

**루트 라우팅 + 인증 게이트 미들웨어** (Sprint 1 잔여)

PRD §5.6 인수 기준 "최초 실행 시 비밀번호 설정 화면 자동 진입" 충족.

### 흐름

```
앱 시작 → src/app/page.tsx (root)
    ├── checkAuthStatus() 호출
    ├── not-initialized → /lock?mode=setup redirect
    ├── locked        → /lock redirect
    └── unlocked (메모리 상태) → 메인 화면 진입

/lock 페이지
    ├── 비밀번호 입력
    ├── 성공 시 app_startup_sequence(password) 호출
    └── startup 성공 시 / 으로 redirect
```

### 구현 포인트

- **`src/app/page.tsx`**: 클라이언트 컴포넌트로 변경. `useEffect` + `checkAuthStatus()` IPC 호출 → 분기 redirect. 데모 `greet` 코드 제거.
- **`src/app/lock/page.tsx`**: 인증 성공 콜백에서 `app_startup_sequence(password, force_lock=false)` 호출 추가. 결과 `StartupResult` 의 `elapsed_ms` 검토 (3초 초과 경고).
- **`src/components/LockScreen.tsx`**: 기존 컴포넌트 그대로 — onSuccess 콜백 시그니처만 확장 (필요 시).
- **인증 후 메모리 상태**: 단순 module-scope 변수 또는 Zustand. Sprint 2 에서 Zustand 도입은 보류 (T1 에서 Zustand 도입까지 하면 범위 초과). React state + sessionStorage 또는 단순 module 변수로 처리.
- **에러 핸들링**: 모든 IPC 호출 try/catch + 사용자 친화 한국어 메시지 표시 (LockScreen 의 errorMessage state 활용).

### Next.js static export 제약

- `src/middleware.ts` 는 static export 에서 동작하지 않음 — root layout 또는 page.tsx 의 클라이언트 가드 패턴 사용
- `next/navigation` 의 `useRouter().replace('/lock')` 활용
- `'use client'` 지시어 필수

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint2/scope.md | [0회] | 본 파일 — Session #1 생성 |
| ROADMAP.md | [0회] | sprint-planner 결과물 (sprint2 ✅) — 첫 커밋에 포함 |
| docs/sprint/sprint2.md | [0회] | sprint-planner 결과물 — 첫 커밋에 포함 |
| docs/risk-register/2026-05-20.md | [0회] | sprint-planner 결과물 — 첫 커밋에 포함 |
| .claude/agents/agent-memory/sprint-planner/MEMORY.md | [0회] | sprint-planner 결과물 — 첫 커밋에 포함 |
| src/app/page.tsx | [0회] | T1 — 데모 greet 제거 + checkAuthStatus 가드 |
| src/app/lock/page.tsx | [0회] | T1 — 인증 성공 시 app_startup_sequence 호출 |
| src/components/LockScreen.tsx | [0회] | T1 — onSuccess 시그니처 확장 (필요 시) |
| src/lib/auth-state.ts | [0회] | (신규 가능) — 모듈 스코프 인증 상태 헬퍼 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (사용자 허가 후만 변경 가능)
- ⬜ `SETUP.sh` — 초기화 스크립트
- ⬜ `docs/harness-engineering/`, `.claude/agents/` (sprint-planner 메모리 외) — 정책·에이전트
- ⬜ `PRD.md`, `docs/phase/`, `docs/sprint/sprint2.md` (sprint-planner 결과물 외 변경 금지)
- ⬜ `.env`, `src-tauri/migrations/` (T5~T7 전까지 변경 없음)
- ⬜ `src-tauri/src/commands/*.rs` (T1 은 frontend 한정)

## 이번 세션의 완료 기준 (T1)

- ⬜ `src/app/page.tsx` 데모 greet 코드 제거 + checkAuthStatus 분기 redirect
- ⬜ `src/app/lock/page.tsx` 인증 성공 시 `app_startup_sequence` 호출 + 메인 redirect
- ⬜ 모든 IPC 호출 try/catch + 한국어 에러 메시지
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- ⬜ 첫 진입 / 재시작 / 인증 성공 후 흐름 모두 로컬에서 시연 가능 (수동 검증은 사용자 일정 시)

## 발견된 이슈 — V001 standard_fees schema 불일치 (T11 진입 시 발견)

**이슈**: V001 의 `standard_fees` 는 `(grade_code, grade_label, monthly_fee)` 학년별 모델이고 시드 12건 등록. 그러나 data-model §5.1 SSOT 는 `(weekly_hours, amount, sort_order, is_active)` 주 수업시간별 모델. sprint2.md T11 `match_fee_by_hours` 도 주 수업시간 매칭 의도.

**조치**: V104 마이그레이션 추가 — `DROP TABLE standard_fees` 후 SSOT schema 로 재생성 + 새 시드. `db::tests::in_memory_pool_runs_migrations` 테스트의 standard_fees 검증 부분도 새 schema 에 맞춰 갱신.

**향후 검토**: Sprint 3 청구 도메인에서 학년별 교습비도 필요한 경우 별도 테이블 (`grade_fees`) 신설.

## 발견된 이슈 — T8 .sqlx prepare 불필요 (현 시점)

**관찰**:
- 현재 코드에 `sqlx::query!()` / `sqlx::query_as!()` 매크로 사용 0건 (Sprint 1 전체가 동적 `sqlx::query() + bind()` 패턴)
- `SQLX_OFFLINE=true cargo build` 가 매크로 없이도 이미 통과
- `.sqlx/` 디렉토리는 매크로 사용 시점에만 의미 있음

**결정**: T9 IPC 도 Sprint 1 패턴(동적 `sqlx::query()` + `bind()`) 유지. T8 sqlx prepare 는 매크로 도입 시점(후속 sprint 또는 backlog)으로 이연.

**근거**:
- `bind()` 패턴이 raw concat 금지 + SQL injection 안전 보장 (backend.md 핵심 정책 충족)
- 매크로 도입은 컴파일 타임 schema 검증 추가 이점이 있지만 sprint 시간 비용 큼 (`.env` + DATABASE_URL + sqlx-cli + prepare 워크플로 설정)
- Sprint 1 의 동적 query 패턴과 일관성 유지

**적용**: T8 task 를 "공식 보류 — 매크로 도입 시 함께 진행"으로 표시. cargo test 의 in_memory_pool_runs_migrations 가 마이그레이션 적용 검증을 대체.

## 발견된 이슈 — T2 명세 chicken-and-egg

**이슈**: sprint2.md T2 명세 "Keychain salt → `app_settings` 테이블 마이그레이션" 은 구조적으로 불가능.

**근거**:
- DB pool 초기화 = `PRAGMA key` 적용 + 마이그레이션 실행
- `PRAGMA key` 적용 = PBKDF2 `derive_key(password, salt)` 필요
- salt 가 DB 안에 있으면 DB 를 열 키가 없는 상태에서 salt 조회 불가
- 즉 salt 는 항상 **평문 (DB 외부)** 에 보관되어야 한다

**대안** (backend.md 가 이미 권장하는 방향):
- Keychain (현재 방식) → **클라우드 동기화 폴더 평문 파일** `{data_root}/salt.bin` 로 이전
- 양 PC 동기화 시 동일 salt 자동 공유 → 동일 키 유도 가능
- PBKDF2 의 salt 는 비밀이 아니므로 평문 보관 정당 (OWASP 권장사항)

**적용**: T2 의 마이그레이션 V100 추가는 생략. `auth.rs` 의 salt 이전 헬퍼는 Keychain → `{data_root}/salt.bin` 으로 변경. sprint2.md 와 risk-register 도 함께 갱신.

## 다음 세션 예정 (참고)

- **Session #2**: T2 R6 salt 이전 준비 + T3 R7 release_lock + T4 R8 startup 측정 (Day 2 묶음, 총 1.5일)
- **Session #3**: T5 V101 마이그레이션 (Day 3, 1일)
- **Session #4+**: T6~T8 마이그레이션 마무리 + T9 원생 CRUD IPC 시작

본 scope.md 는 각 세션 시작 시 갱신한다.
