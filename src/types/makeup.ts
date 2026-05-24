/**
 * 보강 도메인 타입 (Sprint 9 T5).
 *
 * 백엔드 `src-tauri/src/commands/makeup.rs` 의 serde struct 와 1:1 매핑.
 * camelCase 직렬화 — `serde(rename_all = "camelCase")` 적용된 응답.
 */

/** 미처리 결석 1건 — `get_pending_absences` 응답 요소. */
export interface PendingAbsence {
  id: number
  eventDate: string // YYYY-MM-DD
  yearMonth: string // YYYY-MM
  classMinutes: number
  /** 소멸기한 YYYY-MM. NULL 가능 (이전 월 데이터). */
  makeupDeadline: string | null
  /** 결석 사유 메모. */
  absenceMemo: string | null
}

/** 보강 가능 일자 1건 — `get_makeup_eligible_dates` 응답 요소. */
export interface EligibleDate {
  eventDate: string // YYYY-MM-DD
  scheduleCodeName: string
}

/** 보강 등록 페이로드 — `create_makeup_with_absences` 입력. */
export interface CreateMakeupPayload {
  studentId: number
  eventDate: string // YYYY-MM-DD
  classMinutes: number
  absenceIds: number[]
}

/** 보강 등록 결과 — `create_makeup_with_absences` / batch 요소. */
export interface MakeupResult {
  makeupId: number
  studentId: number
  eventDate: string
  matchedCount: number
}

/** 보강데이 일괄 — 학생 1명분 입력. */
export interface BatchMakeupEntry {
  studentId: number
  classMinutes: number
  absenceIds: number[]
}

/** 보강데이 일괄 페이로드. */
export interface BatchCreateMakeupsPayload {
  eventDate: string
  entries: BatchMakeupEntry[]
}

/** 일괄 등록 실패 1건. */
export interface BatchFailure {
  studentId: number
  /** 사용자 친화 한글 에러 메시지. */
  reason: string
}

/** 일괄 등록 결과 — 학생별 독립 트랜잭션의 부분 성공 처리. */
export interface BatchResult {
  succeeded: MakeupResult[]
  failed: BatchFailure[]
}
