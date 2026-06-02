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

export type NoticeFieldType =
  | 'bill_month'
  | 'teaching_period'
  | 'makeup_day'
  | 'student_name'
  | 'bill_amount'
  | 'custom'

/** 청구년월의 교습기간·보강데이 표기 텍스트. */
export interface NoticeMonthInfo {
  teachingPeriodText: string | null
  makeupDayText: string | null
}

/**
 * 텍스트박스 1종 설정 — 배경 원본 해상도 대비 비율(0..1)로 관리.
 * 미리보기 표시 배율/생성 원본 해상도와 무관하게 동일 레이아웃 유지.
 */
export interface TextboxConfig {
  /** 고유 키 (구버전 호환: 빈 값이면 fieldType 으로 대체). */
  id: string
  fieldType: NoticeFieldType
  /** 사용자 정의 박스(custom)의 표시 텍스트. 데이터 필드는 null. */
  text?: string | null
  /** 체크 시에만 배경 위에 표시·생성. */
  enabled: boolean
  xRatio: number // 배경 폭 대비 좌측 (0..1)
  yRatio: number // 배경 높이 대비 상단 (0..1)
  wRatio: number // 배경 폭 대비 너비 (0..1)
  hRatio: number // 배경 높이 대비 높이 (0..1)
  fontRatio: number // 박스 높이 대비 글자 크기 (0..1) — 박스 리사이즈 시 폰트 자동 연동
  fontWeight: 'normal' | 'bold'
  /** 박스 기본 글자색 — charColors 로 색이 지정되지 않은 글자에 적용. */
  fontColor: string
  textAlign: 'left' | 'center' | 'right'
  /**
   * 글자별 폰트색 — 글자 인덱스별 hex(null/누락은 fontColor 사용).
   * 데이터 필드는 원생마다 텍스트 길이가 달라 인덱스 기준으로 적용한다(초과분은 무시).
   */
  charColors?: (string | null)[] | null
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
