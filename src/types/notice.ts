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

/**
 * 텍스트박스 1종 설정 — 배경 원본 해상도 대비 비율(0..1)로 관리.
 * 미리보기 표시 배율/생성 원본 해상도와 무관하게 동일 레이아웃 유지.
 */
export interface TextboxConfig {
  fieldType: NoticeFieldType
  /** 체크 시에만 배경 위에 표시·생성. */
  enabled: boolean
  xRatio: number // 배경 폭 대비 좌측 (0..1)
  yRatio: number // 배경 높이 대비 상단 (0..1)
  wRatio: number // 배경 폭 대비 너비 (0..1)
  hRatio: number // 배경 높이 대비 높이 (0..1)
  fontRatio: number // 박스 높이 대비 글자 크기 (0..1) — 박스 리사이즈 시 폰트 자동 연동
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
