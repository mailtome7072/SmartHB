'use client'

/**
 * AppShell — 인증 완료 후 모든 메인 화면을 감싸는 레이아웃.
 *
 * Sprint 3 T5 + Sprint 4 T3 (상태바 IPC 통합).
 * - `/`, `/students`, `/settings/*` 등에서 사용
 * - `/lock`, `/setup` 은 본 셸 미적용
 * - mount 시 락/백업/동기화 IPC 호출 + 60초 polling — 결과를 TopBar 에 전달
 *
 * **사용자 이슈 #1 해소**: 이전엔 `setLockStatus` 호출자가 없어 TopBar 가 영원히
 * "확인 중...". AppShell 이 단일 책임으로 IPC 호출 + store 갱신.
 */

import { useEffect, useState } from 'react'
import { Sidebar } from './sidebar'
import { TopBar } from './top-bar'
import { useAppStore } from '@/stores/app-store'
import { useSessionStore } from '@/stores/session-store'
import { checkLockStatus, checkSyncStatus, listBackups } from '@/lib/tauri'
import type { SyncStatus } from '@/types'

const POLLING_INTERVAL_MS = 60_000

export function AppShell({
  children,
  topBarSlot,
}: {
  children: React.ReactNode
  /** 글로벌 검색바 등 상단바 중앙 슬롯 — T6 이후 채워진다 */
  topBarSlot?: React.ReactNode
}) {
  const unlocked = useSessionStore((s) => s.unlocked)
  const setLockStatus = useAppStore((s) => s.setLockStatus)
  const [latestBackupAt, setLatestBackupAt] = useState<string | null>(null)
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null)

  useEffect(() => {
    // IPC 호출은 unlock 후 DB pool 초기화 완료 상태에서만 의미. 마법사 등 unlock 전
    // 화면이 본 셸을 사용하지 않으므로 가드는 fail-safe 차원.
    if (!unlocked) return

    let cancelled = false

    const refresh = async () => {
      try {
        const lock = await checkLockStatus()
        if (!cancelled) setLockStatus(lock)
      } catch {
        /* IPC 실패는 store 갱신 생략 — 다음 polling 에서 재시도 */
      }
      try {
        const sync = await checkSyncStatus()
        if (!cancelled) setSyncStatus(sync)
      } catch {
        /* noop */
      }
      try {
        const backups = await listBackups()
        if (!cancelled) {
          // 가장 최근 백업 = created_at 기준 첫 항목 (백엔드가 역순 반환)
          setLatestBackupAt(backups[0]?.created_at ?? null)
        }
      } catch {
        /* noop */
      }
    }

    refresh()
    const id = setInterval(refresh, POLLING_INTERVAL_MS)
    return () => {
      cancelled = true
      clearInterval(id)
    }
  }, [unlocked, setLockStatus])

  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex flex-1 flex-col">
        <TopBar latestBackupAt={latestBackupAt} syncStatus={syncStatus}>
          {topBarSlot}
        </TopBar>
        <main className="flex-1 overflow-y-auto bg-[var(--background)] p-6">{children}</main>
      </div>
    </div>
  )
}
