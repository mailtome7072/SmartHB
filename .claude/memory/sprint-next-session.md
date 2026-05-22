---
name: sprint-next-session
description: "Sprint 6 Session #3 완료(T7 schedule_events IPC, 6/12=50%, 2026-05-22). 다음: /sprint-dev 6 재진입 → Session #4 (T8 프론트엔드 IPC 래퍼·타입)"
metadata: 
  node_type: memory
  type: project
  originSessionId: ec4dbd04-fb9a-48c2-82eb-4dab55bbfa1a
---

Sprint 6 Session #3 종료. **브랜치 `sprint6` 가 origin 동기화 완료** (https://github.com/mailtome7072/SmartHB/tree/sprint6). 새 대화에서 `/sprint-dev 6` 재입력하면 깨끗한 컨텍스트로 Session #4 진입.

## Sprint 6 진행률: 6/12 (50%) — 백엔드 IPC 14개 모두 완성

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 (A20 lock 재시도) | `2c5b8a1` |
| #1 | T3 (A21 paths.rs OnceLock 분리) | `c2be584` |
| #1 | T4 (A22 DnD 방법 B) | `83f19d1` |
| #2 | T5+T6 (academic.rs 신규: study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | **T7** (academic.rs 확장: schedule_events 5) | `a4c380e` |

검증 현황: `cargo test` **141 passed**, `cargo clippy -- -D warnings` clean.

## Session #4 진입 시 우선 액션

1. `/sprint-dev 6` 재진입 (사용자 직접 입력)
2. `docs/sprint/sprint6/scope.md` 읽고 **Session 번호 +1 (#4)**, 수정 횟수 [0회] 리셋
3. **T8 작업** — 프론트엔드 IPC 래퍼 + 도메인 타입 (2h, 짧음)

## T8 구체 작업 안내 (다음 세션 가이드)

### 신규/수정 파일
- 신규: `src/types/academic.ts` — 11개 타입 정의
- 확장: `src/lib/tauri/index.ts` — 14개 IPC 래퍼 추가

### TypeScript 타입 정의 (`src/types/academic.ts`)

```ts
// === study_periods ===
export interface StudyPeriod {
  id: number
  year_month: string         // "YYYY-MM"
  start_date: string         // "YYYY-MM-DD"
  end_date: string
  is_confirmed: boolean
  is_closed: boolean
  created_at: string
  updated_at: string
}
export interface CreateStudyPeriodPayload {
  year_month: string
  start_date: string
  end_date: string
}
export interface UpdateStudyPeriodPayload {
  start_date: string
  end_date: string
}

// === schedule_codes ===
export interface ScheduleCode {
  id: number
  code_name: string
  is_system_reserved: boolean
  allows_regular_class: boolean
  allows_makeup_class: boolean
  is_duplicate_blocked: boolean
  is_period_type: boolean
  is_active: boolean
  created_at: string
  updated_at: string
}
export interface CreateScheduleCodePayload {
  code_name: string
  allows_regular_class: boolean
  allows_makeup_class: boolean
  is_duplicate_blocked: boolean
  is_period_type: boolean
}
export interface UpdateScheduleCodePayload {
  allows_regular_class: boolean
  allows_makeup_class: boolean
  is_duplicate_blocked: boolean
  is_period_type: boolean
}

// === schedule_events ===
export interface ScheduleEvent {
  id: number
  code_id: number
  event_date: string
  period_end_date: string | null
  display_name: string | null
  created_at: string
  updated_at: string
}
export interface ScheduleEventListItem {
  id: number
  code_id: number
  code_name: string
  is_duplicate_blocked: boolean
  is_period_type: boolean
  event_date: string
  period_end_date: string | null
  display_name: string | null
}
export interface CreateScheduleEventPayload {
  code_id: number
  event_date: string
  period_end_date: string | null
  display_name: string | null
}
export interface UpdateScheduleEventPayload {
  event_date: string
  period_end_date: string | null
  display_name: string | null
}
```

> Rust `Option<T>` → TS `T | null` (Tauri serde 기본 직렬화), Rust `i64` → TS `number`, Rust `bool` → TS `boolean`.

### IPC 래퍼 (`src/lib/tauri/index.ts`)

Tauri invoke args 는 camelCase ↔ snake_case 자동 변환됨. Rust `from_month` → TS `fromMonth`. 기존 패턴 그대로 따라:

```ts
export async function createStudyPeriod(
  payload: CreateStudyPeriodPayload,
): Promise<StudyPeriod> {
  const inv = await getInvoke()
  if (!inv) return /* dev mode fallback — { id: 0, year_month: payload.year_month, ... } */
  return inv('create_study_period', { payload }) as Promise<StudyPeriod>
}
```

**14 래퍼 시그니처**:

| 래퍼 | Rust 커맨드 | Args |
|------|-----------|------|
| createStudyPeriod(payload) | create_study_period | { payload } |
| updateStudyPeriod(id, payload) | update_study_period | { id, payload } |
| listStudyPeriods(fromMonth, toMonth) | list_study_periods | { fromMonth, toMonth } |
| getStudyPeriod(yearMonth) | get_study_period | { yearMonth } |
| confirmStudyPeriod(id) | confirm_study_period | { id } |
| deleteStudyPeriod(id) | delete_study_period | { id } |
| listScheduleCodes() | list_schedule_codes | (no args) |
| createScheduleCode(payload) | create_schedule_code | { payload } |
| updateScheduleCode(id, payload) | update_schedule_code | { id, payload } |
| toggleScheduleCodeActive(id) | toggle_schedule_code_active | { id } |
| createScheduleEvent(payload) | create_schedule_event | { payload } |
| updateScheduleEvent(id, payload) | update_schedule_event | { id, payload } |
| deleteScheduleEvent(id) | delete_schedule_event | { id } |
| listScheduleEvents(fromDate, toDate) | list_schedule_events | { fromDate, toDate } |
| autoPlaceAssessmentDates(yearMonth) | auto_place_assessment_dates | { yearMonth } |

### Dev mode fallback (AC-T8-1)
- 빈 리스트 IPC: 빈 배열 `[]`
- 단일 객체 반환: 더미 객체 (예: `{ id: 0, year_month: '2026-05', start_date: '2026-05-01', end_date: '2026-05-31', is_confirmed: false, is_closed: false, created_at: '', updated_at: '' }`)
- void 반환: 그냥 `return`

### 검증 (AC-T8-2/3)
- `pnpm tsc --noEmit` — strict 모드 통과
- `pnpm lint` — clean
- 백엔드 (`src-tauri/src/commands/academic.rs`) 의 14 커맨드 시그니처와 1:1 대응 확인

### 마이그레이션 / 신규 의존성
- 둘 다 없음. 본 세션은 TypeScript 파일 2개만 작업.

## Sprint 6 남은 작업 (T8 이후)

| Task | 작업 | 의존성 | 권장 세션 묶음 |
|------|------|--------|---------------|
| T8 | IPC 래퍼 + 도메인 타입 (프론트) | T5+T6+T7 (충족) | **Session #4 단독 (다음)** |
| T2 | V301 시드 + 공휴일 + ADR | 없음 | Session #5 (brainstorming skill) |
| T9 | 3개월 캘린더 컴포넌트 | T2+T8 | Session #6 (frontend-design skill) |
| T10 | 교습기간 설정 UI | T9 | Session #7 |
| T11 | 일정 코드 + 배치 UI | T9+T10 | Session #8 (frontend-design skill) |
| T12 | 통합 검증 | 전부 | Session #9 (마지막) |

## Sprint 5 완료 산출물 (참고)

- `docs/sprint/sprint5.md` + `docs/sprint-retrospectives/sprint5-retrospective.md` + `docs/test-reports/sprint5-test-report.md`
- CHANGELOG.md `[0.2.1]`

## 정책 (재확인)

- **PR 단계 생략** ([[workflow-no-pr]]) — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **Forbidden Area**: `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/`
- **새 의존성 추가 시 사용자 허가 필수**

본 메모리는 Sprint 6 모든 Task 종료 후 sprint-close 시점에 다음 sprint-next-session으로 슬러그 갱신.
