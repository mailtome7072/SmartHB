/**
 * 데이터 내보내기 도메인 타입 (Sprint 14 T6, PRD §4.13.2).
 *
 * `src-tauri/src/commands/export.rs` 의 ExportResult 와 정합.
 */

/** 내보내기 1회 결과 — 저장 경로 / 데이터 행 수 / 파일 바이트 크기(BOM 포함). */
export interface ExportResult {
  file_path: string
  row_count: number
  byte_size: number
}

/** 내보내기 대상 도메인. */
export type ExportTarget = 'students' | 'attendances' | 'billing'
