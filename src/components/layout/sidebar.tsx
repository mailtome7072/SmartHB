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
 */

import Link from 'next/link'
import { useAppStore } from '@/stores/app-store'
import { MENU_ITEMS } from '@/lib/menu-config'

export function Sidebar() {
  const sidebarOpen = useAppStore((s) => s.sidebarOpen)

  if (!sidebarOpen) return null

  return (
    <nav
      aria-label="메인 메뉴"
      className="flex h-full w-56 flex-col border-r border-[var(--border)] bg-white"
    >
      <div className="border-b border-[var(--border)] px-4 py-4">
        <p className="text-base font-bold text-[var(--foreground)]">스마트해법수학</p>
        <p className="text-sm text-gray-600">서현효자점</p>
      </div>
      <ul className="flex-1 overflow-y-auto py-2">
        {MENU_ITEMS.map((item) => (
          <li key={item.href}>
            {item.disabledHint !== undefined ? (
              <span
                aria-disabled="true"
                title={item.disabledHint}
                className="flex min-h-[44px] cursor-not-allowed items-center justify-between px-4 py-3 text-gray-400"
              >
                <span>{item.label}</span>
                {item.shortcut !== undefined && (
                  <span className="text-sm">{item.shortcut}</span>
                )}
              </span>
            ) : (
              <Link
                href={item.href}
                className="flex min-h-[44px] items-center justify-between px-4 py-3 text-[var(--foreground)] hover:bg-[var(--background)]"
              >
                <span>{item.label}</span>
                {item.shortcut !== undefined && (
                  <span className="text-sm text-gray-500">{item.shortcut}</span>
                )}
              </Link>
            )}
          </li>
        ))}
      </ul>
    </nav>
  )
}
