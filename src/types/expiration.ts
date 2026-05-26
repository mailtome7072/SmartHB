/**
 * 보강 소멸 자동 전이 도메인 타입 — Sprint 10 T3/T4 (PRD §4.5.7).
 *
 * `src-tauri/src/commands/expiration.rs` 와 1:1 정합. camelCase serde rename 적용됨.
 */

/** 자동 전이 결과 — IPC 응답. `transitioned_count > 0` 시 토스트 표시 (PI-09). */
export interface ExpirationReport {
  transitionedCount: number
  details: ExpiredAbsenceDetail[]
}

/** 소멸 전이된 결석 1건 — 토스트/audit 로그 메타데이터. */
export interface ExpiredAbsenceDetail {
  studentId: number
  studentName: string
  eventDate: string       // "YYYY-MM-DD"
  makeupDeadline: string  // "YYYY-MM"
}
