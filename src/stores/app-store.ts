/**
 * 앱 전역 UI store (Sprint 3 T4) — Zustand.
 *
 * 화면 간 공유되어야 하는 UI 상태를 보관한다. IPC 응답 데이터(원생·코드 등)는 TanStack
 * Query 가 캐싱하므로 본 store 에 두지 않는다.
 *
 * ## 보관 항목
 *
 * - `lockStatus`: 최근 조회된 app.lock 점유 상태 (상단바 표시용)
 * - `sidebarOpen`: 사이드바 펼침 여부 (T5 앱 셸에서 사용)
 * - `selectedPeriodMonth`: 선택된 교습기간월 (YYYY-MM, 출결·청구 화면 공유, Phase 2+ 활용)
 *
 * ## persist 정책
 *
 * `selectedPeriodMonth` 와 `sidebarOpen` 은 사용자 편의를 위해 localStorage 에 보관할 수
 * 있으나 Tauri WebView 의 storage 정책은 후속 sprint 에서 검토. 현재는 메모리만 사용.
 */

import { create } from 'zustand'
import type { LockStatus } from '@/types'

interface AppState {
  lockStatus: LockStatus | null
  sidebarOpen: boolean
  selectedPeriodMonth: string | null
  /**
   * 수업 관리 캘린더 → 출결 관리 이동 시 전달할 원생 이름 프리셋 (Sprint 10 T11 follow-up).
   * 출결 관리 페이지가 mount 시 검색어로 소비한 뒤 null 로 비운다 (1회성).
   */
  attendanceSearchPreset: string | null

  /**
   * 미저장 변경 네비게이션 가드 (Sprint 12 — 공지문 편집 중 메뉴 이동 시 저장 확인).
   * 화면이 등록하며, 이동 직전 호출되어 `true` 면 즉시 이동 허용, `false` 면 차단(화면이 확인
   * 다이얼로그 표시 후 직접 이동 처리). 등록 화면 unmount 시 null 로 해제한다.
   */
  unsavedGuard: ((href: string) => boolean) | null

  /**
   * 공용 미저장 이동 확인 다이얼로그의 이동 대상 경로 (Sprint 16 T1).
   * `useUnsavedChanges` 훅이 가드 차단 시 여기에 href 를 세팅하면, AppShell 의
   * `UnsavedNavDialog` 가 확인 다이얼로그를 띄운다. Tauri WebView 가 `window.confirm`/
   * `alert` 을 차단(dialog 권한)하므로 자체 다이얼로그로 대체.
   */
  unsavedNavTarget: string | null

  setLockStatus: (status: LockStatus | null) => void
  setSidebarOpen: (open: boolean) => void
  toggleSidebar: () => void
  setSelectedPeriodMonth: (month: string | null) => void
  setAttendanceSearchPreset: (name: string | null) => void
  setUnsavedGuard: (guard: ((href: string) => boolean) | null) => void
  setUnsavedNavTarget: (href: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  lockStatus: null,
  sidebarOpen: true,
  selectedPeriodMonth: null,
  attendanceSearchPreset: null,
  unsavedGuard: null,
  unsavedNavTarget: null,

  setLockStatus: (lockStatus) => set({ lockStatus }),
  setSidebarOpen: (sidebarOpen) => set({ sidebarOpen }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setSelectedPeriodMonth: (selectedPeriodMonth) => set({ selectedPeriodMonth }),
  setAttendanceSearchPreset: (attendanceSearchPreset) => set({ attendanceSearchPreset }),
  setUnsavedGuard: (unsavedGuard) => set({ unsavedGuard }),
  setUnsavedNavTarget: (unsavedNavTarget) => set({ unsavedNavTarget }),
}))

/**
 * 미저장 가드를 거쳐 네비게이션한다. 가드가 차단하면 `navigate` 를 호출하지 않는다.
 * (가드 등록 화면이 확인 다이얼로그를 띄우고, 확인 시 자체적으로 이동을 수행)
 */
export function guardedNavigate(href: string, navigate: () => void): void {
  const guard = useAppStore.getState().unsavedGuard
  if (guard && !guard(href)) return
  navigate()
}
