'use client'

/**
 * 루트 페이지 (Sprint 2 T1) — 인증 게이트.
 *
 * `checkAuthStatus()` IPC 호출로 분기:
 * - `not-initialized` → `/lock?mode=setup` redirect (최초 비밀번호 설정)
 * - `locked` → `/lock` redirect (잠금 해제)
 * - 본 세션에서 이미 잠금 해제됨 (`useSessionStore.unlocked`) → 메인 화면 표시
 *
 * PRD §5.6 인수 기준 "최초 실행 시 비밀번호 설정 화면 자동 진입" 충족.
 *
 * 메인 화면 자체는 후속 sprint 에서 대시보드 등으로 채워진다.
 */

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { checkAuthStatus, getSetupStatus } from '@/lib/tauri'
import { useSessionStore } from '@/stores/session-store'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { DashboardView } from '@/components/dashboard/DashboardView'

export default function Home() {
  const router = useRouter()
  const [ready, setReady] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const unlocked = useSessionStore((s) => s.unlocked)
  // Sprint 10 T9 (PI-09): 앱 시작 시 소멸 자동 전이 결과 토스트.
  const lastStartup = useSessionStore((s) => s.lastStartup)
  const expirationNoticeDismissed = useSessionStore(
    (s) => s.expirationNoticeDismissed,
  )
  const dismissExpirationNotice = useSessionStore(
    (s) => s.dismissExpirationNotice,
  )
  const expirationCount =
    lastStartup?.expiration_report.transitionedCount ?? 0
  const showExpirationNotice =
    ready && expirationCount > 0 && !expirationNoticeDismissed
  // Sprint 16 — 시작 시 DB 손상 자동복원 고지.
  const restoreNoticeDismissed = useSessionStore((s) => s.restoreNoticeDismissed)
  const dismissRestoreNotice = useSessionStore((s) => s.dismissRestoreNotice)
  const autoRestored = lastStartup?.auto_restored ?? null
  const showRestoreNotice = ready && autoRestored !== null && !restoreNoticeDismissed

  useEffect(() => {
    if (unlocked) {
      setReady(true)
      return
    }
    let cancelled = false
    // 마법사 우선 분기: setup_completed=false 면 /setup 으로. 완료 후 잠금 화면 또는
    // (잠금 해제 완료 시) 본 페이지.
    Promise.all([getSetupStatus(), checkAuthStatus()])
      .then(([setupStatus, authStatus]) => {
        if (cancelled) return
        if (!setupStatus.setup_completed) {
          router.replace('/setup')
          return
        }
        const target = authStatus === 'not-initialized' ? '/lock?mode=setup' : '/lock'
        router.replace(target)
      })
      .catch((e: unknown) => {
        if (cancelled) return
        setError(typeof e === 'string' ? e : '인증 상태를 확인할 수 없습니다.')
      })
    return () => {
      cancelled = true
    }
  }, [router, unlocked])

  if (error !== null) {
    return (
      <main className="flex min-h-screen items-center justify-center p-8">
        <div
          role="alert"
          className="max-w-md rounded-md border-2 border-[var(--danger)] bg-red-50 p-4 text-base text-[var(--danger)]"
        >
          {error}
        </div>
      </main>
    )
  }

  if (!ready) {
    return <SplashScreen message="시작 상태를 확인하는 중입니다..." />
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      {showRestoreNotice && (
        <div
          role="alert"
          className="mx-6 mt-4 flex items-start justify-between gap-3 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
        >
          <span>
            데이터 파일 손상이 감지되어 <b>최근 정상 백업으로 자동 복원</b>했습니다. 복원 직후
            일부 최근 입력이 누락됐을 수 있으니 원생·출결·청구 데이터를 확인해 주세요. (손상된
            파일은 보존되어 있습니다)
          </span>
          <button
            type="button"
            onClick={dismissRestoreNotice}
            aria-label="알림 닫기"
            className="ml-1 min-h-[32px] min-w-[32px] shrink-0 rounded text-[var(--danger)] hover:bg-red-100"
          >
            ×
          </button>
        </div>
      )}
      {showExpirationNotice && (
        <div
          role="status"
          className="mx-6 mt-4 flex items-center justify-between rounded-md border-2 border-amber-400 bg-amber-50 p-3 text-base text-amber-900"
        >
          <span>
            앱 시작과 함께 소멸기한 도래 결석 {expirationCount}건이 자동
            처리되었습니다.
          </span>
          <button
            type="button"
            onClick={dismissExpirationNotice}
            aria-label="알림 닫기"
            className="ml-3 min-h-[32px] min-w-[32px] rounded text-amber-700 hover:bg-amber-100"
          >
            ×
          </button>
        </div>
      )}
      <DashboardView />
    </AppShell>
  )
}
