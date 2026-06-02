/**
 * 사이드바·글로벌 검색이 공유하는 메뉴 구성 (Sprint 3 T5+T6).
 *
 * 단일 SSOT — 메뉴 추가·라벨 변경 시 본 파일만 수정한다.
 */

export interface MenuItem {
  label: string
  href: string
  /** Phase 2+ 예정 항목 — 비활성 표시 + 클릭 시 안내 */
  disabledHint?: string
}

// V19 (Sprint 7 post-review): 모든 메뉴 단축키 제거 — 50대 사용자 친화 UX 단순화.
// Sprint 13: Phase 5(단원 평가/학습 보고서) 전면 취소 — 두 메뉴 항목 제거 (원장 지시).
export const MENU_ITEMS: MenuItem[] = [
  { label: '대시보드', href: '/', disabledHint: 'Phase 5 에서 제공' },
  { label: '원생 관리', href: '/students' },
  { label: '일정 관리', href: '/academic' },
  { label: '출결 관리', href: '/attendance' },
  { label: '수업 관리', href: '/schedules' },
  // 보강 관리는 Sprint 10 T11 에서 `/schedules` 페이지 내 탭으로 통합 — 별도 메뉴 항목 제거 (Sprint 11 F7).
  { label: '청구/수납 관리', href: '/billing' },
  { label: '공지문', href: '/notices' },
  { label: '설정', href: '/settings' },
]

/** 글로벌 검색이 결과로 노출 가능한 활성 메뉴 (disabled 제외). */
export const ACTIVE_MENU_ITEMS: MenuItem[] = MENU_ITEMS.filter(
  (m) => m.disabledHint === undefined,
)
