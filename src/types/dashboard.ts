/**
 * 대시보드 도메인 타입 (Sprint 14 T4, PRD §4.11).
 *
 * `src-tauri/src/commands/dashboard.rs` 의 구조체와 정합.
 */

export interface LabelCount {
  label: string
  count: number
}

export interface QuarterEnrollment {
  label: string
  enrolled: number
  withdrawn: number
}

/** 4.11.1 교습소 현황. */
export interface AcademyOverview {
  total_active: number
  by_gender: LabelCount[]
  by_grade: LabelCount[]
  by_school: LabelCount[]
  quarterly: QuarterEnrollment[]
}

/** 4.11.2 당일 수업 — 시간대별 명단. */
export interface TodaySlot {
  start_time: string
  students: string[]
}

export interface TodaySchedule {
  /** ISO weekday 1=월~7=일. */
  weekday: number
  slots: TodaySlot[]
}

/** 4.11.3 월 핵심 요약. */
export interface MonthlySummary {
  year_month: string
  bill_total: number
  paid_total: number
  unpaid_total: number
  bill_count: number
  paid_count: number
  enrolled_this_month: number
  withdrawn_this_month: number
  attendance_recorded_days: number
}

/** 4.11.4 알림. */
export interface DashboardAlert {
  kind: string
  /** 'red' | 'orange' | 'blue'. */
  severity: string
  message: string
  count: number
}

/** 4.11.5 출결 입력 진행률. */
export interface AttendanceProgress {
  year_month: string
  expected_days: number
  recorded_days: number
  missing_dates: string[]
}
