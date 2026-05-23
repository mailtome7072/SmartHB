'use client'

/**
 * 상단바 (Sprint 3 T5 + Sprint 4 T3 / 사용자 이슈 #1, #2).
 *
 * 우측 영역: 점유 디바이스 / 마지막 백업 / 동기화 상태 / 시작 속도 (정상속도/속도저하).
 * IPC 호출은 AppShell 이 담당하고 본 컴포넌트는 표시만 한다 (store + props SSOT).
 * 사이드바 토글 + 글로벌 검색바 슬롯도 유지.
 */

import { useAppStore } from '@/stores/app-store'
import { useSessionStore } from '@/stores/session-store'
import type { LockStatus, SyncStatus } from '@/types'

function formatLockStatus(status: LockStatus | null): string {
  if (status === null) return '점유: 확인 중...'
  switch (status.kind) {
    case 'free':
      return '점유: 없음'
    case 'owned-by-self':
      return `점유: 본 디바이스 (${status.last_heartbeat_seconds_ago}s 전)`
    case 'owned-by-other':
      return status.stale
        ? '점유: 다른 PC (응답 없음 — stale)'
        : '점유: 다른 PC 사용 중'
  }
}

function formatBackupAt(iso: string | null): string {
  if (iso === null) return '백업: 없음'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '백업: ?'
  return `백업: ${d.toLocaleString('ko-KR', { dateStyle: 'short', timeStyle: 'short' })}`
}

function formatSyncStatus(status: SyncStatus | null): string {
  if (status === null) return '동기화: 확인 중...'
  switch (status.kind) {
    case 'ready':
      return '동기화: 준비됨'
    case 'waiting':
      return `동기화: 대기 중 (${status.seconds_since_change}s)`
  }
}

export function TopBar({
  children,
  latestBackupAt,
  syncStatus,
}: {
  children?: React.ReactNode
  latestBackupAt?: string | null
  syncStatus?: SyncStatus | null
}) {
  const toggleSidebar = useAppStore((s) => s.toggleSidebar)
  const lockStatus = useAppStore((s) => s.lockStatus)
  const lastStartup = useSessionStore((s) => s.lastStartup)

  // V34 (Sprint 7 post-review): 시작 시간 ms 숫자 → "정상속도" / "속도저하" 라벨로 변경.
  // 사용자(50대) 친화 표현. 상세 ms 수치는 title 속성으로만 노출 (PRD §5.7).
  const startupWarn = lastStartup !== null && lastStartup.elapsed_ms > 3000
  const startupLabel =
    lastStartup === null ? null : startupWarn ? '속도저하' : '정상속도'

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

      {/* 글로벌 검색바 슬롯 */}
      <div className="flex-1 px-4">{children}</div>

      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-gray-600">
        <span title="현재 클라우드 폴더 락 점유 상태">{formatLockStatus(lockStatus)}</span>
        <span aria-hidden="true" className="text-gray-300">|</span>
        <span title="마지막 백업 생성 시각">{formatBackupAt(latestBackupAt ?? null)}</span>
        <span aria-hidden="true" className="text-gray-300">|</span>
        <span title="클라우드 동기화 폴더 mtime 안정 여부">
          {formatSyncStatus(syncStatus ?? null)}
        </span>
        {startupLabel !== null && (
          <>
            <span aria-hidden="true" className="text-gray-300">|</span>
            <span
              title={
                lastStartup !== null
                  ? startupWarn
                    ? `시작 ${lastStartup.elapsed_ms}ms — PRD §5.6 < 3000ms 초과, 환경 점검 권장`
                    : `시작 ${lastStartup.elapsed_ms}ms — 정상`
                  : ''
              }
              className={startupWarn ? 'font-semibold text-[var(--danger)]' : undefined}
            >
              {startupLabel}
            </span>
          </>
        )}
      </div>
    </header>
  )
}
