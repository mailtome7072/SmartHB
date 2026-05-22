'use client'

/**
 * 학사 스케줄 관리 페이지 — Sprint 6 T9 (PRD §4.4).
 *
 * 진입: 사이드바 "학사 스케줄" 메뉴 → `/academic`.
 * 잠금 해제 전이면 루트(`/`) 가드가 `/lock` 으로 redirect — 본 페이지는 unlocked 가정.
 *
 * 본 세션 (Sprint 6 T9) 는 3개월 캘린더 표시까지. 교습기간 설정(T10) /
 * 학사 일정 배치(T11) 등 인터랙션은 후속 세션에서 onCellClick 핸들러로 통합.
 */

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { checkAuthStatus } from '@/lib/tauri'
import { useSessionStore } from '@/stores/session-store'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { ThreeMonthCalendar } from '@/components/academic/ThreeMonthCalendar'

export default function AcademicPage() {
  const router = useRouter()
  const [ready, setReady] = useState(false)
  const unlocked = useSessionStore((s) => s.unlocked)

  useEffect(() => {
    if (unlocked) {
      setReady(true)
      return
    }
    let cancelled = false
    void (async () => {
      const status = await checkAuthStatus()
      if (cancelled) return
      if (status === 'not-initialized') {
        router.replace('/lock?mode=setup')
      } else if (status === 'locked') {
        router.replace('/lock')
      } else {
        setReady(true)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [router, unlocked])

  if (!ready) return <SplashScreen message="학사 스케줄 화면 준비 중..." />

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <main className="flex flex-col gap-4 p-4">
        <header>
          <h1 className="text-2xl font-bold text-[var(--foreground)]">학사 스케줄</h1>
          <p className="text-sm text-gray-600">
            교습기간 설정과 학사 일정 배치 — 좌·중·우 3개월을 한 화면에서 확인합니다.
          </p>
        </header>
        <ThreeMonthCalendar />
      </main>
    </AppShell>
  )
}
