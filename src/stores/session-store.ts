/**
 * 세션 store (Sprint 3 T4) — Zustand.
 *
 * `lib/auth-state.ts` 의 module-scoped 변수를 대체한다. 본 세션의 잠금 해제 사실과
 * 마지막 `app_startup_sequence` 결과를 메모리에만 보관한다.
 *
 * ## 보안
 *
 * - 비밀번호·키·복구 코드는 절대 저장하지 않는다 (백엔드 Keychain·DB 책임).
 * - persist 미사용 — 페이지 새로고침 시 메모리 초기화되어 잠금 화면 재진입.
 * - `StartupResult` 만 보관 — `elapsed_ms` 등 측정 정보만 포함되어 민감하지 않다.
 */

import { create } from 'zustand'
import type { StartupResult } from '@/types'

interface SessionState {
  unlocked: boolean
  lastStartup: StartupResult | null
  /** Sprint 10 T9 — 사용자가 expiration_report 토스트를 닫은 후 재표시 차단 플래그. */
  expirationNoticeDismissed: boolean
  /** Sprint 16 — 사용자가 DB 자동복원 고지를 닫은 후 재표시 차단 플래그. */
  restoreNoticeDismissed: boolean
  markUnlocked: (result: StartupResult) => void
  dismissExpirationNotice: () => void
  dismissRestoreNotice: () => void
}

export const useSessionStore = create<SessionState>((set) => ({
  unlocked: false,
  lastStartup: null,
  expirationNoticeDismissed: false,
  restoreNoticeDismissed: false,
  markUnlocked: (result) => set({
    unlocked: true,
    lastStartup: result,
    expirationNoticeDismissed: false,
    restoreNoticeDismissed: false,
  }),
  dismissExpirationNotice: () => set({ expirationNoticeDismissed: true }),
  dismissRestoreNotice: () => set({ restoreNoticeDismissed: true }),
}))
