/**
 * 원생 도메인 타입 — Sprint 2 T9.
 *
 * `src-tauri/src/commands/students.rs` 와 serde rename_all="kebab-case" 정합.
 */

export type Gender = 'male' | 'female'
export type SchoolLevel = 'elementary' | 'middle'
export type StudentSort =
  | 'serial-asc'
  | 'serial-desc'
  | 'name-asc'
  | 'name-desc'
  | 'grade-asc'
  | 'grade-desc'
  | 'enroll-date-asc'
  | 'enroll-date-desc'

export interface Student {
  id: number
  serial_no: string
  name: string
  /** T11 이슈 #4: 원생 목록에 표시. list_students 만 제공, 다른 IPC 응답에는 null. */
  weekly_hours?: number | null
  /** 현행 스케줄 요일 콤마 구분 (예: "1,3,5" = 월/수/금). null = 미등록. */
  schedule_days_csv?: string | null
  gender: Gender
  school_level: SchoolLevel
  grade: number
  school_id: number | null
  phone_student: string | null
  phone_mother: string | null
  phone_father: string | null
  enroll_date: string
  withdraw_date: string | null
  created_at: string
  updated_at: string
}

/** 신규 원생 등록 payload — `serial_no` 미지정 시 PI-05 자동 채번. */
export interface NewStudent {
  serial_no?: string | null
  name: string
  gender: Gender
  school_level: SchoolLevel
  grade: number
  school_id?: number | null
  phone_student?: string | null
  phone_mother?: string | null
  phone_father?: string | null
  enroll_date: string
}

/** 원생 수정 payload — PUT-like 전체 필드. */
export interface StudentUpdate {
  serial_no: string
  name: string
  gender: Gender
  school_level: SchoolLevel
  grade: number
  school_id: number | null
  phone_student: string | null
  phone_mother: string | null
  phone_father: string | null
  enroll_date: string
  withdraw_date: string | null
}

/**
 * 목록 조회 필터 — 모든 필드 Optional.
 *
 * R14 페이지네이션: `limit` 미지정 시 백엔드 기본값 100 (상한 1000), `offset` 기본 0.
 * `countStudents` 는 동일 필터(`limit`/`offset` 제외)로 총 건수를 반환.
 */
export interface StudentFilter {
  active_only?: boolean
  name_query?: string
  school_level?: SchoolLevel
  grade?: number
  school_id?: number
  gender?: Gender
  day_of_week?: number
  sort?: StudentSort
  limit?: number
  offset?: number
}
