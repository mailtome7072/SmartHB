'use client'

/**
 * 일정 관리 페이지 — Sprint 6 T9 + T10 + T11 (PRD §4.4).
 *
 * Sprint 7 T5: 코드 CRUD 는 `/settings/schedule-codes` 로 이동, 본 페이지는 Selector 만 사용
 * Sprint 7 T6 (Issue 5): 교습기간 토글 모드 제거 — 셀 클릭 분기를 데이터 상태로 자동 결정.
 *
 * 셀 클릭 분기 (Sprint 7 T6 이후):
 * - `selectedCode !== null` → 일정 배치 모드 (EventPlacer 가 mutation 담당)
 * - `selectedCode === null` + 중앙 월 교습기간 미확정 → 교습기간 selection 모드 자동 활성
 * - 위 둘 다 아님 (확정 월 + 코드 미선택) → 셀 클릭 무동작 (배지 클릭만 활성 — 삭제)
 */

import { useEffect, useMemo, useState } from 'react'
import { createPortal } from 'react-dom'
import { useRouter } from 'next/navigation'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  checkAuthStatus,
  deleteScheduleEvent,
  getStudyPeriod,
  listScheduleEvents,
  listStudyPeriods,
  updateScheduleEvent,
} from '@/lib/tauri'
import { useSessionStore } from '@/stores/session-store'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { ThreeMonthCalendar } from '@/components/academic/ThreeMonthCalendar'
import {
  StudyPeriodEditor,
  type SelectionRange,
} from '@/components/academic/StudyPeriodEditor'
import { ScheduleCodeSelector } from '@/components/academic/ScheduleCodeSelector'
import {
  EventPlacer,
  useEventPlaceCellHandler,
} from '@/components/academic/EventPlacer'
import { AcademicSchedulePrint } from '@/components/academic/AcademicSchedulePrint'
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

  // T10 교습기간 selection (Sprint 7 T6: mode state 제거 — 중앙 월 확정 여부로 자동 분기)
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

  // T9: 교습일정 인쇄
  const [printMode, setPrintMode] = useState(false)

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

  // 일정 드래그 이동 mutation — 단일 일자만 (period_end_date=null 보존)
  const dragMoveMutation = useMutation({
    mutationFn: ({ id, newDate, displayName }: { id: number; newDate: string; displayName: string | null }) =>
      updateScheduleEvent(id, {
        event_date: newDate,
        period_end_date: null,
        display_name: displayName,
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['schedule-events'] })
    },
    onError: (err) => {
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

  // 중앙 월 교습기간 확정 여부 조회 — null 이면 미확정 (자동 selection 모드 활성).
  const centerPeriodQuery = useQuery({
    queryKey: ['study-period', centerYearMonth],
    queryFn: () => getStudyPeriod(centerYearMonth),
    staleTime: 30_000,
    enabled: centerYearMonth.length > 0,
  })
  const isCenterUnconfirmed =
    centerYearMonth.length > 0 &&
    !centerPeriodQuery.isLoading &&
    centerPeriodQuery.data === null
  const studyPeriodMode = isCenterUnconfirmed && selectedCode === null
  // T9: 확정 교습기간 이벤트 — 인쇄용 (printMode=true 일 때만 fetch)
  const confirmedPeriod = centerPeriodQuery.data ?? null
  const printEventsQuery = useQuery({
    queryKey: ['schedule-events-print', confirmedPeriod?.start_date, confirmedPeriod?.end_date],
    queryFn: () =>
      listScheduleEvents(confirmedPeriod!.start_date, confirmedPeriod!.end_date),
    enabled: printMode && confirmedPeriod !== null,
    staleTime: 30_000,
  })
  // T9: 쿼리 완료 후 인쇄 다이얼로그 — setTimeout 대신 쿼리 상태 감지로 race condition 방지
  useEffect(() => {
    if (printMode && printEventsQuery.isSuccess) {
      window.print()
      setPrintMode(false)
    }
  }, [printMode, printEventsQuery.isSuccess])

  // V12 (Sprint 7 post-review): 인접 3개월 교습기간 조회 — selection 단계에서 다른 교습기간
  // 일자 포함 차단. ThreeMonthCalendar 와 동일 쿼리 키로 캐시 공유.
  const adjacentMonths = useMemo(() => {
    if (!/^\d{4}-\d{2}$/.test(centerYearMonth)) return { from: '', to: '' }
    const [y, m] = centerYearMonth.split('-').map(Number)
    const prev = m === 1 ? `${y - 1}-12` : `${y}-${String(m - 1).padStart(2, '0')}`
    const next = m === 12 ? `${y + 1}-01` : `${y}-${String(m + 1).padStart(2, '0')}`
    return { from: prev, to: next }
  }, [centerYearMonth])
  const adjacentPeriodsQuery = useQuery({
    queryKey: ['study-periods', adjacentMonths.from, adjacentMonths.to],
    queryFn: () => listStudyPeriods(adjacentMonths.from, adjacentMonths.to),
    staleTime: 30_000,
    enabled: adjacentMonths.from.length > 0,
  })

  /** V12: 주어진 일자가 다른 교습월의 교습기간에 이미 속하는지. */
  function isDateInOtherPeriod(date: string): boolean {
    return (adjacentPeriodsQuery.data ?? []).some(
      (p) =>
        p.year_month !== centerYearMonth &&
        date >= p.start_date &&
        date <= p.end_date,
    )
  }

  /** V12: 선택 range [start, end] 가 다른 교습기간과 겹치는지. */
  function rangeOverlapsOtherPeriod(start: string, end: string): boolean {
    const lo = start <= end ? start : end
    const hi = start <= end ? end : start
    return (adjacentPeriodsQuery.data ?? []).some(
      (p) =>
        p.year_month !== centerYearMonth &&
        p.start_date <= hi &&
        p.end_date >= lo,
    )
  }

  // 통합 셀 클릭 핸들러 — 활성 모드 자동 분기 (Sprint 7 T6).
  function handleCellClick(date: string) {
    if (selectedCode !== null) {
      handleEventPlaceCellClick(date)
      return
    }
    if (studyPeriodMode) {
      // V12: 다른 교습기간 일자에 포함되면 차단
      if (isDateInOtherPeriod(date)) {
        setEventErrorMessage(
          '이미 다른 교습월의 교습기간에 포함된 날짜입니다. 다른 일자를 선택해 주세요.',
        )
        return
      }
      if (!studyPeriodSelection.start || studyPeriodSelection.end) {
        setStudyPeriodSelection({ start: date, end: null })
      } else {
        // V12: 시작 ~ 종료 범위가 다른 교습기간과 겹치는지 검사.
        if (rangeOverlapsOtherPeriod(studyPeriodSelection.start, date)) {
          setEventErrorMessage(
            '선택 범위가 다른 교습월의 교습기간과 겹칩니다. 종료일을 다시 선택해 주세요.',
          )
          return
        }
        setStudyPeriodSelection({ start: studyPeriodSelection.start, end: date })
      }
    }
  }

  function handleSelectCode(code: ScheduleCode | null) {
    setSelectedCode(code)
    setEventSelectionStart(null)
    setEventSelectionEnd(null)
    // 코드 선택 시 교습기간 selection 초기화 — 모드 충돌 회피.
    if (code) {
      setStudyPeriodSelection({ start: null, end: null })
    }
  }

  function handleCloseEventMode() {
    setSelectedCode(null)
    setEventSelectionStart(null)
    setEventSelectionEnd(null)
  }

  // 캘린더에 전달할 selection — 활성 모드에 따라 분기.
  const calendarSelection: SelectionRange | null = (() => {
    if (studyPeriodMode) return studyPeriodSelection
    if (selectedCode?.is_period_type) {
      return { start: eventSelectionStart, end: eventSelectionEnd }
    }
    return null
  })()

  // onCellClick 콜백 — 활성 모드 없으면 undefined (셀 클릭 무동작).
  const calendarCellHandler =
    studyPeriodMode || selectedCode !== null ? handleCellClick : undefined

  // Sprint 8 T8 (R51 / A37): selection 모드(studyPeriodMode)에서는 배지 클릭 무시.
  // 사용자가 교습기간 일자 범위를 선택하는 동안 배지 클릭으로 의도치 않은 삭제 다이얼로그가
  // 뜨는 것을 차단. 코드 배치 모드(selectedCode !== null)는 stopPropagation 으로 셀 클릭과
  // 분리되므로 그대로 허용 (V27 의 의도 유지).
  const calendarEventClick = (event: ScheduleEventListItem) => {
    if (studyPeriodMode) return
    setEventToDelete(event)
  }

  if (!ready) return <SplashScreen message="일정 관리 화면 준비 중..." />

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <main className="flex flex-col gap-4 p-4">
        <header className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-[var(--foreground)]">일정 관리</h1>
            <p className="text-sm text-gray-600">
              교습기간 설정 / 학사 일정 코드 관리 / 일정 배치 — 좌·중·우 3개월을 한 화면에서.
            </p>
          </div>
          <button
            type="button"
            disabled={confirmedPeriod === null}
            title={confirmedPeriod === null ? '이 달의 교습기간을 먼저 등록해주세요' : undefined}
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-4 py-2 text-sm font-medium hover:bg-gray-50 disabled:cursor-not-allowed disabled:bg-gray-100 disabled:text-gray-400 disabled:hover:bg-gray-100 print:hidden"
            onClick={() => setPrintMode(true)}
          >
            교습일정 인쇄
          </button>
        </header>

        {/* V11 (Sprint 7 post-review): 교습기간 + 일정 배치 코드를 단일 컨트롤 바로 통합.
            높이 최소화하여 캘린더 영역이 상단으로 더 노출되도록. 좌(1) 교습기간 / 우(2) 코드 선택.
            EventPlacer 는 코드 선택 시에만 컨트롤 바 아래 별도 박스로 노출. */}
        <section
          aria-label="학사 컨트롤"
          className="rounded-lg border border-[var(--border)] bg-white p-2"
        >
          <div className="grid gap-2 lg:grid-cols-3 lg:items-start">
            <StudyPeriodEditor
              centerYearMonth={centerYearMonth}
              eventPlaceMode={selectedCode !== null}
              selection={studyPeriodSelection}
              setSelection={setStudyPeriodSelection}
            />
            <div className="lg:col-span-2 lg:border-l lg:border-[var(--border)] lg:pl-3">
              <ScheduleCodeSelector
                selectedCodeId={selectedCode?.id ?? null}
                onSelect={handleSelectCode}
              />
            </div>
          </div>
        </section>
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

        <ThreeMonthCalendar
          selection={calendarSelection}
          onCellClick={calendarCellHandler}
          onEventClick={calendarEventClick}
          onCenterChange={setCenterYearMonth}
          onEventDrop={(event, newDate) =>
            dragMoveMutation.mutate({
              id: event.id,
              newDate,
              displayName: event.display_name,
            })
          }
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
              {/* P1-6: 위험 동작(삭제) 확정 버튼은 빨강 — 삭제류 일관 규칙 */}
              <AlertDialogAction
                onClick={(e) => {
                  e.preventDefault()
                  if (eventToDelete) deleteMutation.mutate(eventToDelete.id)
                }}
                disabled={deleteMutation.isPending}
                className="bg-[var(--danger)] text-white hover:bg-[var(--danger)] hover:opacity-90"
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
      {/* T9: 인쇄 전용 컴포넌트 — @media print 에서만 표시.
          document.body 에 직접 portal — App Router 에는 Pages Router 의 #__next 가 없어
          AppShell 내부에 두면 인쇄 시 조상(App 루트)이 display:none 처리되어 후손인
          이 wrapper 가 !important 를 걸어도 함께 가려지는(빈 인쇄) 문제가 있었다. */}
      {printMode &&
        confirmedPeriod !== null &&
        typeof window !== 'undefined' &&
        createPortal(
          <div className="academic-print-wrapper" style={{ display: 'none' }}>
            <AcademicSchedulePrint
              period={confirmedPeriod}
              events={printEventsQuery.data ?? []}
            />
          </div>,
          document.body,
        )}
    </AppShell>
  )
}
