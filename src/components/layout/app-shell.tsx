'use client'

/**
 * AppShell — 인증 완료 후 모든 메인 화면을 감싸는 레이아웃.
 *
 * Sprint 3 T5. `/`, `/students`, `/settings/*` 등에서 사용. `/lock`, `/setup` 은 본 셸을
 * 적용하지 않는다 (잠금/마법사 흐름은 단독 화면).
 */

import { Sidebar } from './sidebar'
import { TopBar } from './top-bar'
import { useKeyboardShortcuts } from '@/hooks/use-keyboard-shortcuts'

export function AppShell({
  children,
  topBarSlot,
}: {
  children: React.ReactNode
  /** 글로벌 검색바 등 상단바 중앙 슬롯 — T6 이후 채워진다 */
  topBarSlot?: React.ReactNode
}) {
  useKeyboardShortcuts()

  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex flex-1 flex-col">
        <TopBar>{topBarSlot}</TopBar>
        <main className="flex-1 overflow-y-auto bg-[var(--background)] p-6">{children}</main>
      </div>
    </div>
  )
}
