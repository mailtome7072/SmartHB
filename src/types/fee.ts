/**
 * 표준 교습비 타입 — Sprint 2 T11.
 *
 * `src-tauri/src/commands/fees.rs` 와 정합. data-model §5.1 (주 수업시간별).
 */

export interface StandardFee {
  id: number
  weekly_hours: number
  amount: number
  sort_order: number
  is_active: boolean
  created_at: string
  updated_at: string
}

/** 신규 등록 payload — `sort_order` 미지정 시 MAX+1 자동 부여. */
export interface NewFee {
  weekly_hours: number
  amount: number
  sort_order?: number
}

export interface FeeUpdate {
  weekly_hours: number
  amount: number
  sort_order: number
  is_active: boolean
}
