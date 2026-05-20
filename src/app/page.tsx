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

export default function Home() {
  const router = useRouter()
  const [ready, setReady] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const unlocked = useSessionStore((s) => s.unlocked)
  const lastStartup = useSessionStore((s) => s.lastStartup)

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
    return (
      <main className="flex min-h-screen items-center justify-center">
        <p className="text-lg text-gray-600">불러오는 중...</p>
      </main>
    )
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="flex flex-col items-center justify-center pt-12">
        <h1 className="mb-4 text-4xl font-bold">스마트해법수학</h1>
        <p className="mb-8 text-lg text-gray-600">정쌤의 교습소 관리 시스템</p>
        {lastStartup !== null && (
          <p className="text-sm text-gray-500">
            시작 시간: {lastStartup.elapsed_ms} ms
            {lastStartup.elapsed_ms > 3000 && ' (PRD §5.6 < 3000 ms 초과 — 환경 점검 권장)'}
          </p>
        )}
        <p className="mt-8 text-sm text-gray-500">대시보드는 Phase 6 에서 구축됩니다. 사이드바의 ‘원생 관리’ 또는 ‘설정’으로 이동.</p>
      </div>
    </AppShell>
  )
}
