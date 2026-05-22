'use client'

/**
 * 학사 스케줄 관리 페이지 — Sprint 6 T9 + T10 + T11 (PRD §4.4).
 *
 * Mode 분기 (state lift up):
 * - 'view'         : 셀 클릭 무동작 (배지 클릭만 활성 — 삭제 모드)
 * - 'study-period' : T10 교습기간 설정 (StudyPeriodEditor 가 셀 클릭 → selection)
 * - 'event-place'  : T11 일정 배치 (EventPlacer + 선택 코드 → 셀 클릭 = INSERT 또는 selection)
 *
 * ScheduleCodePanel 의 코드 선택 = mode 자동 활성. 코드 해제 = mode='view' 복귀.
 */

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { checkAuthStatus, deleteScheduleEvent } from '@/lib/tauri'
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
import { ScheduleCodePanel } from '@/components/academic/ScheduleCodePanel'
import {
  EventPlacer,
  useEventPlaceCellHandler,
} from '@/components/academic/EventPlacer'
import type { ScheduleCode, ScheduleEventListItem } from '@/types/academic'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

export default function AcademicPage() {
  const router = useRouter()
  const queryClient = useQueryClient()
  const [ready, setReady] = useState(false)
  const unlocked = useSessionStore((s) => s.unlocked)

  // T10 교습기간 모드 (study-period)
  const [studyPeriodMode, setStudyPeriodMode] = useState<EditorMode>('view')
  const [studyPeriodSelection, setStudyPeriodSelection] = useState<SelectionRange>({
    start: null,
    end: null,
  })

  // T11 일정 배치 모드 (event-place) — 선택된 코드 존재 = 모드 활성
  const [selectedCode, setSelectedCode] = useState<ScheduleCode | null>(null)
  const [eventSelectionStart, setEventSelectionStart] = useState<string | null>(null)
  const [eventSelectionEnd, setEventSelectionEnd] = useState<string | null>(null)
  const [centerYearMonth, setCenterYearMonth] = useState<string>('')

  // T11 일정 삭제 다이얼로그
  const [eventToDelete, setEventToDelete] = useState<ScheduleEventListItem | null>(null)
  const [eventErrorMessage, setEventErrorMessage] = useState<string | null>(null)

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

  // 일정 삭제 mutation
  const deleteMutation = useMutation({
    mutationFn: deleteScheduleEvent,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['schedule-events'] })
      setEventToDelete(null)
    },
    onError: (err) => {
      setEventToDelete(null)
      setEventErrorMessage(err instanceof Error ? err.message : String(err))
    },
  })

  // T11 일정 배치 셀 클릭 핸들러 — 선택된 코드가 있을 때만 동작.
  const handleEventPlaceCellClick = useEventPlaceCellHandler({
    selectedCode,
    selectionStart: eventSelectionStart,
    setSelectionStart: setEventSelectionStart,
    setSelectionEnd: setEventSelectionEnd,
    onError: setEventErrorMessage,
  })

  // 통합 셀 클릭 핸들러 — mode 분기
  function handleCellClick(date: string) {
    if (studyPeriodMode === 'editing') {
      // T10 교습기간 모드 — selection 갱신
      if (!studyPeriodSelection.start || studyPeriodSelection.end) {
        setStudyPeriodSelection({ start: date, end: null })
      } else {
        setStudyPeriodSelection({ start: studyPeriodSelection.start, end: date })
      }
      return
    }
    if (selectedCode !== null) {
      // T11 일정 배치 모드
      handleEventPlaceCellClick(date)
    }
  }

  function handleSelectCode(code: ScheduleCode | null) {
    setSelectedCode(code)
    setEventSelectionStart(null)
    setEventSelectionEnd(null)
    // 교습기간 모드와 충돌 방지: 코드 선택 시 교습기간 모드 종료.
    if (code) {
      setStudyPeriodMode('view')
      setStudyPeriodSelection({ start: null, end: null })
    }
  }

  function handleCloseEventMode() {
    setSelectedCode(null)
    setEventSelectionStart(null)
    setEventSelectionEnd(null)
  }

  // 교습기간 모드 진입 시 일정 배치 모드 종료 (StudyPeriodEditor 내부 setMode 호출 시 동기화 위해 effect)
  useEffect(() => {
    if (studyPeriodMode === 'editing' && selectedCode !== null) {
      setSelectedCode(null)
      setEventSelectionStart(null)
      setEventSelectionEnd(null)
    }
  }, [studyPeriodMode, selectedCode])

  // 캘린더에 전달할 selection — 활성 모드에 따라 분기.
  const calendarSelection: SelectionRange | null = (() => {
    if (studyPeriodMode === 'editing') return studyPeriodSelection
    if (selectedCode?.is_period_type) {
      return { start: eventSelectionStart, end: eventSelectionEnd }
    }
    return null
  })()

  // onCellClick 콜백 — 활성 모드 없으면 undefined (셀 클릭 무동작).
  const calendarCellHandler =
    studyPeriodMode === 'editing' || selectedCode !== null ? handleCellClick : undefined

  // 일정 배지 클릭 = 삭제 (view 모드에서만, 다른 모드에서는 셀 클릭 우선).
  const calendarEventClick =
    studyPeriodMode === 'editing' || selectedCode !== null
      ? undefined
      : (event: ScheduleEventListItem) => setEventToDelete(event)

  if (!ready) return <SplashScreen message="학사 스케줄 화면 준비 중..." />

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <main className="flex flex-col gap-4 p-4">
        <header>
          <h1 className="text-2xl font-bold text-[var(--foreground)]">학사 스케줄</h1>
          <p className="text-sm text-gray-600">
            교습기간 설정 / 학사 일정 코드 관리 / 일정 배치 — 좌·중·우 3개월을 한 화면에서.
          </p>
        </header>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-[2fr_1fr]">
          <div className="flex flex-col gap-4">
            <StudyPeriodEditor
              mode={studyPeriodMode}
              setMode={setStudyPeriodMode}
              selection={studyPeriodSelection}
              setSelection={setStudyPeriodSelection}
            />
            {selectedCode !== null && (
              <EventPlacer
                selectedCode={selectedCode}
                selectionStart={eventSelectionStart}
                selectionEnd={eventSelectionEnd}
                setSelectionStart={setEventSelectionStart}
                setSelectionEnd={setEventSelectionEnd}
                centerYearMonth={centerYearMonth}
                onClose={handleCloseEventMode}
              />
            )}
          </div>
          <ScheduleCodePanel
            selectedCodeId={selectedCode?.id ?? null}
            onSelect={handleSelectCode}
          />
        </div>

        <ThreeMonthCalendar
          selection={calendarSelection}
          onCellClick={calendarCellHandler}
          onEventClick={calendarEventClick}
          onCenterChange={setCenterYearMonth}
        />

        {/* 일정 삭제 확인 다이얼로그 */}
        <AlertDialog
          open={eventToDelete !== null}
          onOpenChange={(open) => {
            if (!open) setEventToDelete(null)
          }}
        >
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>학사 일정 삭제</AlertDialogTitle>
              <AlertDialogDescription>
                {eventToDelete && (
                  <>
                    <strong>{eventToDelete.event_date}</strong>{' '}
                    <strong>
                      {eventToDelete.display_name ?? eventToDelete.code_name}
                    </strong>{' '}
                    일정을 삭제합니다.
                    <br />
                    지난 달 일정은 백엔드에서 차단됩니다.
                  </>
                )}
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel disabled={deleteMutation.isPending}>취소</AlertDialogCancel>
              <AlertDialogAction
                onClick={(e) => {
                  e.preventDefault()
                  if (eventToDelete) deleteMutation.mutate(eventToDelete.id)
                }}
                disabled={deleteMutation.isPending}
              >
                {deleteMutation.isPending ? '삭제 중...' : '삭제'}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>

        {/* 백엔드 에러 (지난 달 삭제 시도 등) */}
        <AlertDialog
          open={eventErrorMessage !== null}
          onOpenChange={(open) => {
            if (!open) setEventErrorMessage(null)
          }}
        >
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>일정 처리 실패</AlertDialogTitle>
              <AlertDialogDescription>{eventErrorMessage}</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogAction onClick={() => setEventErrorMessage(null)}>확인</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </main>
    </AppShell>
  )
}
