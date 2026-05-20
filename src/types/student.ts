/**
 * 원생 도메인 타입 — Sprint 2 T9.
 *
 * `src-tauri/src/commands/students.rs` 와 serde rename_all="kebab-case" 정합.
 */

export type Gender = 'male' | 'female'
export type SchoolLevel = 'elementary' | 'middle'
export type StudentSort = 'name-asc' | 'enroll-date-desc' | 'grade-asc'

export interface Student {
  id: number
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
