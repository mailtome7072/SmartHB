/**
 * 학사 스케줄 도메인 타입 — Sprint 6 T8.
 *
 * `src-tauri/src/commands/academic.rs` 와 1:1 정합. PRD §4.4 / §6.2.
 * - Rust `Option<T>` → TS `T | null` (Tauri serde 기본 직렬화)
 * - Rust `i64` → TS `number`, Rust `bool` → TS `boolean`
 */

// ─── study_periods (T5) ──────────────────────────────────────────────

/** 교습기간 (월 단위). V102 study_periods. PRD §4.4.2. */
export interface StudyPeriod {
  id: number
  year_month: string         // "YYYY-MM"
  start_date: string         // "YYYY-MM-DD"
  end_date: string           // "YYYY-MM-DD"
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

// ─── schedule_codes (T6) ─────────────────────────────────────────────

/** 학사 일정 코드 (3속성 모델). V102 schedule_codes. PRD §4.4.3~4.4.5. */
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

// ─── schedule_events (T7) ────────────────────────────────────────────

/** 학사 일정 (캘린더 배치). V103 schedule_events. PRD §4.4.6. */
export interface ScheduleEvent {
  id: number
  code_id: number
  event_date: string                   // "YYYY-MM-DD"
  period_end_date: string | null       // 기간성 코드만 값 있음
  display_name: string | null
  created_at: string
  updated_at: string
}

/** 캘린더 렌더링용 평탄 응답 — schedule_codes JOIN 결과. */
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
