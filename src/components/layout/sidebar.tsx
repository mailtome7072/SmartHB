'use client'

/**
 * 사이드바 (Sprint 3 T5, PRD §5.1 + §5.7).
 *
 * Phase 1 에서 활성: 원생 관리 / 설정.
 * Phase 2+ 메뉴(대시보드/수업/출결/청구/단원평가/학습보고서/공지문)는 disabled 처리하여
 * 사용자에게 "다음 업데이트 예정" 안내. 메뉴 항목에 단축키 표기 병기 (PRD §5.7).
 *
 * - 클릭 영역 44×44px 이상 (Tailwind py-3 = 12px × 2 + 본문 18px line-height ≈ 51px)
 * - WCAG AA 명도 대비 보장 (foreground/border 토큰 사용)
 *
 * Sprint 7 보강: 사이드바 하단 "종료" 항목.
 * `quitApp()` IPC → `AppHandle::exit(0)` → `RunEvent::ExitRequested` →
 * `startup::exit_hook` 의 release_lock + exit 백업 (R15 보장).
 * 기존 `getCurrentWindow().close()` 는 capabilities `core:window:allow-close` 권한 부재로
 * 거부됐었음. 백엔드 IPC 경유로 권한 + macOS 닥 잔존 이슈 동시 회피.
 */

import Link from 'next/link'
import { useAppStore } from '@/stores/app-store'
import { MENU_ITEMS } from '@/lib/menu-config'
import { quitApp } from '@/lib/tauri'

export function Sidebar() {
  const sidebarOpen = useAppStore((s) => s.sidebarOpen)

  if (!sidebarOpen) return null

  return (
    <nav
      aria-label="메인 메뉴"
      className="flex h-full w-56 flex-col border-r border-[var(--border)] bg-white"
    >
      <div className="flex h-16 flex-col justify-center border-b border-[var(--border)] px-4">
        <p className="text-base font-bold leading-tight text-[var(--foreground)]">스마트해법수학</p>
        <p className="text-sm leading-tight text-gray-600">서현효자점</p>
      </div>
      <ul className="flex-1 overflow-y-auto py-2">
        {MENU_ITEMS.map((item) => (
          <li key={item.href}>
            {item.disabledHint !== undefined ? (
              <span
                aria-disabled="true"
                title={item.disabledHint}
                className="flex min-h-[44px] cursor-not-allowed items-center px-4 py-3 text-gray-400"
              >
                {item.label}
              </span>
            ) : (
              <Link
                href={item.href}
                className="flex min-h-[44px] items-center px-4 py-3 text-[var(--foreground)] hover:bg-[var(--background)]"
              >
                {item.label}
              </Link>
            )}
          </li>
        ))}
        <li>
          <button
            type="button"
            onClick={() => void quitApp()}
            className="flex min-h-[44px] w-full items-center justify-between px-4 py-3 text-left text-[var(--foreground)] hover:bg-[var(--background)]"
          >
            <span>종료</span>
          </button>
        </li>
      </ul>
    </nav>
  )
}
