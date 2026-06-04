/**
 * 데이터 자가 진단 도메인 타입 (Sprint 14 T2, PRD §6.6).
 *
 * `src-tauri/src/commands/diagnosis.rs` 의 DiagnosisIssue/DiagnosisResult/DiagnosisHistoryRow 와 정합.
 */

export type DiagnosisSeverity = 'error' | 'warning'

/** 진단에서 발견된 개별 이상 항목. */
export interface DiagnosisIssue {
  /** 검사 식별자 (예: 'orphan_makeup'). 해결 가이드 매핑 키로 사용. */
  check_id: string
  /** 'error' | 'warning' (백엔드가 문자열로 직렬화). */
  severity: DiagnosisSeverity | string
  /** 50대 친화 한글 설명. */
  message: string
  /** 관련 테이블명 — 화면 이동 링크 구성용 (없으면 null). */
  target_table: string | null
  /** 관련 행 id (없으면 null). */
  target_id: number | null
}

/** 1회 진단 실행 결과 요약 + 발견 항목 전체. */
export interface DiagnosisResult {
  run_date: string
  run_type: string
  total_checks: number
  issues_found: number
  issues: DiagnosisIssue[]
}

/** 진단 이력 1건. */
export interface DiagnosisHistoryRow {
  id: number
  run_date: string
  run_type: string
  total_checks: number
  issues_found: number
  issues: DiagnosisIssue[]
  created_at: string
}
