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

/** 결석 이력 1건 — `get_absence_history` 응답 요소 (T8). */
export interface AbsenceHistoryItem {
  id: number
  eventDate: string // YYYY-MM-DD
  classMinutes: number
  /** 'absent' | 'makeup_done' | 'makeup_expired' */
  status: 'absent' | 'makeup_done' | 'makeup_expired'
  makeupDeadline: string | null
  absenceMemo: string | null
  /** makeup_done 인 경우 매칭된 보강 일자. */
  makeupEventDate: string | null
  /** 매칭된 보강의 수업 시간 (분). */
  makeupClassMinutes: number | null
}
