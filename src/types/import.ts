/**
 * CSV 원생 가져오기 타입 (Sprint 16 T2, PRD §4.13.1).
 *
 * `src-tauri/src/commands/import.rs` 의 Serialize 타입과 정합 (snake_case).
 */

export type ImportRowStatus = 'ok' | 'warning' | 'duplicate' | 'error'

export interface ImportPreviewRow {
  /** CSV 데이터 행 번호 (헤더 제외, 1부터). */
  row_number: number
  name: string
  /** 원본 학년 텍스트 (예: "초3"). */
  grade_label: string
  /** 표시용 성별 ("남"/"여"/""). */
  gender_label: string
  enroll_date: string
  serial_no: string | null
  status: ImportRowStatus
  messages: string[]
}

export interface ImportPreviewResult {
  rows: ImportPreviewRow[]
  total: number
  /** 가져올 수 있는 행 (ok + warning). */
  importable: number
  duplicate: number
  error: number
}

export interface ImportResult {
  inserted: number
  skipped: number
  errored: number
  errors: string[]
  backup_note: string
}
