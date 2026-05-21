/**
 * 사이드바·글로벌 검색이 공유하는 메뉴 구성 (Sprint 3 T5+T6).
 *
 * 단일 SSOT — 메뉴 추가·라벨 변경 시 본 파일만 수정한다.
 */

export interface MenuItem {
  label: string
  href: string
  shortcut?: string
  /** Phase 2+ 예정 항목 — 비활성 표시 + 클릭 시 안내 */
  disabledHint?: string
}

export const MENU_ITEMS: MenuItem[] = [
  { label: '대시보드', href: '/', disabledHint: 'Phase 6 에서 제공' },
  { label: '원생 관리', href: '/students', shortcut: 'Ctrl+N' },
  { label: '수업 관리', href: '/schedules', disabledHint: 'Phase 2 에서 제공' },
  { label: '출결 관리', href: '/attendance', disabledHint: 'Phase 2 에서 제공' },
  { label: '청구 관리', href: '/billing', disabledHint: 'Phase 4 에서 제공' },
  { label: '단원 평가', href: '/exams', disabledHint: 'Phase 5 에서 제공' },
  { label: '학습 보고서', href: '/reports', disabledHint: 'Phase 5 에서 제공' },
  { label: '공지문', href: '/notices', disabledHint: 'Phase 4 에서 제공' },
  { label: '설정', href: '/settings' },
]

/** 글로벌 검색이 결과로 노출 가능한 활성 메뉴 (disabled 제외). */
export const ACTIVE_MENU_ITEMS: MenuItem[] = MENU_ITEMS.filter(
  (m) => m.disabledHint === undefined,
)
