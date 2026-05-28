/**
 * 퇴교 시 미사용 보강 처리 도메인 타입 — Sprint 10 T6/T10 (PRD §4.5.9).
 *
 * `src-tauri/src/commands/expiration.rs` 의 응답 구조체와 1:1 정합. camelCase.
 */

/** 퇴교 시 미보강 결석 1건 — 다이얼로그 리스트 표시. */
export interface PendingAbsenceForWithdrawal {
  id: number
  eventDate: string          // "YYYY-MM-DD"
  classMinutes: number
  makeupDeadline: string | null  // "YYYY-MM"
}

/** 퇴교 시 미사용 보강 조회 응답. */
export interface WithdrawalPendingMakeup {
  studentId: number
  remainingMinutes: number
  absences: PendingAbsenceForWithdrawal[]
}

/**
 * 퇴교 처리 선택지 — PRD §4.5.9.
 *
 * Rust `WithdrawalChoice` enum 의 `#[serde(tag = "type", rename_all = "snake_case")]` 직렬화 형식.
 * `defer_withdrawal` 은 UI 에서 다이얼로그 닫기로 처리 — IPC 호출 없음 (PI-08 결정).
 */
export type WithdrawalChoice =
  | { type: 'immediate_expire' }
  | { type: 'external_expire'; memo: string }
