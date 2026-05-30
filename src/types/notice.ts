/**
 * 공지문(교습비 안내 이미지) 도메인 타입 (Sprint 12, PRD §4.10).
 *
 * 백엔드 `src-tauri/src/commands/notice.rs` 의 serde camelCase 직렬화와 1:1 정합.
 * 이미지 바이너리는 IPC에서 `number[]`(Rust `Vec<u8>`)로 주고받는다 (base64 미사용).
 */

/** 배경서식 파일 메타데이터. */
export interface NoticeAsset {
  name: string
  size: number
  /** 수정 시각 (epoch millis). */
  modifiedMs: number
}

export type NoticeFieldType = 'bill_month' | 'student_name' | 'bill_amount'

/** 텍스트박스 1종 설정 — 배경 이미지 픽셀 기준. */
export interface TextboxConfig {
  fieldType: NoticeFieldType
  x: number
  y: number
  width: number
  height: number
  fontSize: number
  fontWeight: 'normal' | 'bold'
  fontColor: string
  textAlign: 'left' | 'center' | 'right'
}

/** 공지문 레이아웃 — 배경서식 + 텍스트박스 3종 (AC-4.10-3 보존 대상). */
export interface NoticeLayout {
  backgroundAsset: string | null
  textboxes: TextboxConfig[]
}

/** 일괄 저장 입력 1건. image 는 PNG 바이트 배열. */
export interface NoticeImageItem {
  studentName: string
  image: number[]
}
