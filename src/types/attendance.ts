/**
 * 출결 도메인 TypeScript 타입 — Sprint 8 T4.
 *
 * 백엔드 `src-tauri/src/commands/attendance.rs` 의 응답 구조체와 정확히 매칭.
 * camelCase serde rename 적용된 형태.
 */

/** 정규 출결 상태. */
export type AttendanceStatus =
  | 'present'
  | 'absent'
  | 'makeup_done'
  | 'makeup_expired'

/** 보강 출결 상태. */
export type MakeupStatus = 'makeup_attended' | 'makeup_absent'

/** 그리드 한 칸 (출결 셀). */
export interface AttendanceCell {
  id: number
  eventDate: string // YYYY-MM-DD
  status: AttendanceStatus
  classMinutes: number
  absenceMemo: string | null
  makeupDeadline: string | null // YYYY-MM
  makeupAttendanceId: number | null
}

/** 원생 월간 요약. */
export interface AttendanceSummary {
  studentId: number
  yearMonth: string
  presentCount: number
  absentCount: number
  makeupNeededMinutes: number
  makeupCompletedMinutes: number
}

/** 그리드 한 원생 행. */
export interface AttendanceGridStudent {
  studentId: number
  name: string
  serialNo: string
  scheduleDays: number[] // ISO 요일 1=월~7=일
  enrollDate: string // YYYY-MM-DD — Sprint 9 Session #10 I8
  withdrawDate: string | null // Sprint 9 Session #10 I8
  attendances: AttendanceCell[]
  summary: AttendanceSummary
}

/** 일자별 학사일정 매핑 — Sprint 9 Session #10 I7/I8. */
export interface DaySchedule {
  eventDate: string // YYYY-MM-DD
  allowsMakeup: boolean // 보강데이/단원평가/공휴수업일
  isBlock: boolean // 공휴일/방학/휴원일
  label: string // 표시용 코드명
}

/** 그리드 응답 전체. */
export interface AttendanceGrid {
  yearMonth: string
  students: AttendanceGridStudent[]
  daySchedules: DaySchedule[]
}

/** 토글 결과 — 낙관적 업데이트의 권위적 응답. */
export interface ToggleResult {
  attendanceId: number
  newStatus: AttendanceStatus
  newMakeupDeadline: string | null
  updatedSummary: AttendanceSummary
}

/** 출결 일괄 생성 결과. */
export interface GenerateResult {
  yearMonth: string
  studentCount: number
  attendanceCount: number
}
