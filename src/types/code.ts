/**
 * 코드 테이블 타입 — Sprint 2 T12.
 *
 * `src-tauri/src/commands/codes.rs` 와 정합.
 */

export type CodeTable = 'schools' | 'payment-methods' | 'card-companies'

/** schools.school_type — DB CHECK 제약과 정합 (Sprint 19 T9). */
export type SchoolType = 'elementary' | 'middle' | 'high' | 'etc'

/** 코드 항목 — 세 테이블 공통 정규화 응답. */
export interface CodeEntry {
  id: number
  code: string
  label: string
  /** Sprint 19 T9: schools 는 school_type. 그 외 테이블은 항상 null. */
  extra: string | null
  sort_order: number
  is_active: boolean
}

/** 신규 등록 payload — 테이블별로 필드 의미가 다름.
 *
 * - schools: `code` = 학교명, `extra` = school_type ('elementary' / 'middle' / 'high' / 'etc')
 * - payment_methods / card_companies: `code` + `label` 모두 사용
 */
export interface NewCode {
  code: string
  label?: string | null
  extra?: string | null
  sort_order?: number
}

export interface CodeUpdate {
  label: string
  /** Sprint 19 T9: schools 는 school_type 변경값. 미지정(null) 이면 기존 값 유지. 다른 테이블은 무시됨. */
  extra?: string | null
  sort_order: number
  is_active: boolean
}
