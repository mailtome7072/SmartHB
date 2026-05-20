/**
 * 수업 스케줄 타입 — Sprint 2 T10.
 *
 * `src-tauri/src/commands/schedules.rs` 와 정합. data-model §1.2.
 * `day_of_week`: 1=월, 7=일 (V101 CHECK). `effective_to=null` = 현행.
 */

export interface StudentSchedule {
  id: number
  student_id: number
  day_of_week: number
  start_time: string
  duration_hours: number
  effective_from: string
  effective_to: string | null
  created_at: string
  updated_at: string
}

/** 스케줄 설정/변경 payload. */
export interface ScheduleSet {
  student_id: number
  day_of_week: number
  start_time: string
  duration_hours: number
  effective_from: string
}
