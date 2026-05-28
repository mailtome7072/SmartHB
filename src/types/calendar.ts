/**
 * 수업 관리 캘린더 도메인 타입 — Sprint 10 T8/T11 (PRD §4.6).
 *
 * `src-tauri/src/commands/calendar.rs` 와 1:1 정합. camelCase serde rename 적용됨.
 */

/** 캘린더 한 달 데이터 — 일자별 수업 목록. */
export interface CalendarMonth {
  yearMonth: string
  days: CalendarDay[]
}

/** 캘린더 한 일자 — 정규 수업 + 보강 수업 분리. */
export interface CalendarDay {
  eventDate: string // "YYYY-MM-DD"
  regularSessions: CalendarSession[]
  makeupSessions: CalendarSession[]
}

/** 캘린더 수업 1건 — 정규는 `startTime` 있음, 보강은 null. */
export interface CalendarSession {
  studentId: number
  studentName: string
  startTime: string | null // "HH:MM" (정규만)
  classMinutes: number
}

/** 보강 관리 뷰 한 원생 — PRD §4.6.3. */
export interface MakeupManagementStudent {
  studentId: number
  studentName: string
  serialNo: string
  /** 잔여 보강필요시간 (분 단위). */
  remainingMinutes: number
  /** 가장 임박한 makeup_deadline (YYYY-MM) — 없으면 null. */
  earliestDeadline: string | null
  /** 소멸 임박 플래그 — deadline 월 교습기간 종료일 -7일 이내 도래. */
  isImminent: boolean
  /** 퇴교일 (YYYY-MM-DD) — 재원중 필터용. null 이면 재원중. */
  withdrawDate: string | null
}
