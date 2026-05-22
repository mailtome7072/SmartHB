'use client'

/**
 * 학사 스케줄 관리 페이지 — Sprint 6 T9 + T10 (PRD §4.4).
 *
 * 진입: 사이드바 "학사 스케줄" 메뉴 → `/academic`.
 * 잠금 해제 전이면 루트(`/`) 가드가 `/lock` 으로 redirect — 본 페이지는 unlocked 가정.
 *
 * 상태 끌어올리기 (T10):
 * - mode / selection 을 페이지가 보유하고 StudyPeriodEditor + ThreeMonthCalendar 가 공유.
 * - mode='editing' 일 때만 캘린더 셀 클릭이 selection 갱신 → 다른 모드는 셀 클릭 무동작.
 *
 * 후속 (T11): mode 에 'event-place' 추가, EventPlacer 와 통합.
 */

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { checkAuthStatus } from '@/lib/tauri'
import { useSessionStore } from '@/stores/session-store'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { ThreeMonthCalendar } from '@/components/academic/ThreeMonthCalendar'
import {
  StudyPeriodEditor,
  type EditorMode,
  type SelectionRange,
} from '@/components/academic/StudyPeriodEditor'

export default function AcademicPage() {
  const router = useRouter()
  const [ready, setReady] = useState(false)
  const unlocked = useSessionStore((s) => s.unlocked)

  const [mode, setMode] = useState<EditorMode>('view')
  const [selection, setSelection] = useState<SelectionRange>({ start: null, end: null })

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

  function handleCellClick(date: string) {
    if (mode !== 'editing') return
    // 첫 클릭 또는 이미 선택 완료(start+end) 상태 → 새 시작일.
    if (!selection.start || selection.end) {
      setSelection({ start: date, end: null })
    } else {
      // 두 번째 클릭 → 종료일 (start>end 처리는 StudyPeriodEditor 의 normalize 에서).
      setSelection({ start: selection.start, end: date })
    }
  }

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
        <StudyPeriodEditor
          mode={mode}
          setMode={setMode}
          selection={selection}
          setSelection={setSelection}
        />
        <ThreeMonthCalendar
          selection={mode === 'editing' ? selection : null}
          onCellClick={mode === 'editing' ? handleCellClick : undefined}
        />
      </main>
    </AppShell>
  )
}
