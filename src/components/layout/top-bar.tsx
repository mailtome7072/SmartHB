'use client'

/**
 * 상단바 (Sprint 3 T5, PRD §5.1·§5.3).
 *
 * 좌측: 사이드바 토글 버튼.
 * 우측: 점유 디바이스 (app-store.lockStatus), 마지막 동기화/백업 시각.
 * 글로벌 검색바는 T6 에서 본 상단바 중앙에 삽입된다.
 *
 * 락 상태는 백엔드 IPC(`checkLockStatus`)를 호출한 컴포넌트가 `useAppStore.setLockStatus` 로
 * 갱신한 값을 그대로 표시한다. 본 컴포넌트가 IPC 를 호출하지 않는다 — store 가 SSOT.
 */

import { useAppStore } from '@/stores/app-store'
import type { LockStatus } from '@/types'

function formatLockStatus(status: LockStatus | null): string {
  if (status === null) return '확인 중...'
  switch (status.kind) {
    case 'free':
      return '점유 없음'
    case 'owned-by-self':
      return `본 디바이스 점유 중 (${status.last_heartbeat_seconds_ago}s 전)`
    case 'owned-by-other':
      return status.stale
        ? '다른 디바이스 점유 (응답 없음 — 강제 점유 가능)'
        : '다른 디바이스에서 사용 중'
  }
}

export function TopBar({ children }: { children?: React.ReactNode }) {
  const toggleSidebar = useAppStore((s) => s.toggleSidebar)
  const lockStatus = useAppStore((s) => s.lockStatus)

  return (
    <header className="flex h-16 items-center justify-between border-b border-[var(--border)] bg-white px-4">
      <button
        type="button"
        onClick={toggleSidebar}
        aria-label="사이드바 토글"
        className="flex h-11 w-11 items-center justify-center rounded-md hover:bg-[var(--background)]"
      >
        <span aria-hidden="true" className="text-xl">≡</span>
      </button>

      {/* 글로벌 검색바 슬롯 — T6 에서 채워진다 */}
      <div className="flex-1 px-4">{children}</div>

      <div className="flex items-center gap-3 text-sm text-gray-600">
        <span>{formatLockStatus(lockStatus)}</span>
      </div>
    </header>
  )
}
