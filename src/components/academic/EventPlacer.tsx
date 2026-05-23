'use client'

/**
 * 학사 일정 배치 에디터 — Sprint 6 T11 (PRD §4.4.6, §4.4.7).
 *
 * 흐름:
 *   1) ScheduleCodeSelector 에서 코드 선택 → mode='event-place' + selectedCode 설정
 *   2) 캘린더 셀 클릭:
 *      - 단일 일자 코드: 1회 클릭 → createScheduleEvent({ code_id, event_date, period_end_date: null })
 *      - 기간성 코드: 시작/종료일 두 셀 클릭 → createScheduleEvent({ ..., period_end_date })
 *   3) 단원평가 코드 선택 시 "자동 배치" 버튼 노출 → autoPlaceAssessmentDates(중앙 month)
 *   4) 중복불가 / 지난 달 / 기타 백엔드 에러 → AlertDialog 한국어 메시지
 *
 * State 끌어올림: page 가 selectedCode + selection 보유. EventPlacer 는 mutation + UI 만.
 *
 * 본 컴포넌트는 선택된 코드가 있을 때만 의미가 있음 — 부모에서 mode 분기로 마운트 제어 권장.
 */

import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import {
  autoPlaceAssessmentDates,
  createScheduleEvent,
} from '@/lib/tauri'
import type { ScheduleCode } from '@/types/academic'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

interface EventPlacerProps {
  selectedCode: ScheduleCode
  /** 기간성 코드 선택 시 시작/종료 선택 상태 — 부모 관리. */
  selectionStart: string | null
  selectionEnd: string | null
  setSelectionStart: (s: string | null) => void
  setSelectionEnd: (s: string | null) => void
  /** 자동 배치 기준 month (`YYYY-MM`) — 캘린더 중앙 월. */
  centerYearMonth: string
  /** 모드 종료 콜백 (코드 선택 해제 + selection 초기화). */
  onClose: () => void
}

export function EventPlacer({
  selectedCode,
  selectionStart,
  selectionEnd,
  setSelectionStart,
  setSelectionEnd,
  centerYearMonth,
  onClose,
}: EventPlacerProps) {
  const queryClient = useQueryClient()
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [autoPlaceResult, setAutoPlaceResult] = useState<string | null>(null)

  function invalidate() {
    void queryClient.invalidateQueries({ queryKey: ['schedule-events'] })
  }

  const createMutation = useMutation({
    mutationFn: createScheduleEvent,
    onSuccess: () => {
      invalidate()
      setSelectionStart(null)
      setSelectionEnd(null)
    },
    onError: (err) => setErrorMessage(err instanceof Error ? err.message : String(err)),
  })

  const autoPlaceMutation = useMutation({
    mutationFn: (yearMonth: string) => autoPlaceAssessmentDates(yearMonth),
    onSuccess: (events, yearMonth) => {
      invalidate()
      if (events.length === 0) {
        setAutoPlaceResult(`${yearMonth} 에 이미 단원평가가 있어 추가 배치하지 않았습니다.`)
      } else {
        setAutoPlaceResult(`${yearMonth} 에 단원평가 ${events.length} 건 자동 배치되었습니다.`)
      }
    },
    onError: (err) => setErrorMessage(err instanceof Error ? err.message : String(err)),
  })

  const isPeriodType = selectedCode.is_period_type
  const isAssessment = selectedCode.code_name === '단원평가 응시일'

  function statusText(): string {
    if (!isPeriodType) return '캘린더 셀을 클릭하면 일정이 등록됩니다.'
    if (!selectionStart) return '기간성 코드 — 시작일을 클릭하세요.'
    if (!selectionEnd) return `시작일 ${selectionStart} — 종료일을 클릭하세요.`
    const lo = selectionStart <= selectionEnd ? selectionStart : selectionEnd
    const hi = selectionStart <= selectionEnd ? selectionEnd : selectionStart
    return `${lo} ~ ${hi} 선택됨 — 확정 버튼을 누르세요.`
  }

  function handleConfirmRange() {
    if (!selectionStart || !selectionEnd) return
    const lo = selectionStart <= selectionEnd ? selectionStart : selectionEnd
    const hi = selectionStart <= selectionEnd ? selectionEnd : selectionStart
    createMutation.mutate({
      code_id: selectedCode.id,
      event_date: lo,
      period_end_date: hi,
      display_name: null,
    })
  }

  return (
    <section
      aria-label="학사 일정 배치"
      className="flex flex-col gap-2 rounded-lg border border-blue-300 bg-blue-50 p-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="text-lg font-bold text-blue-900">
            {selectedCode.code_name} 배치 중
            <span className="ml-2 text-sm font-normal text-blue-700">
              ({isPeriodType ? '기간성' : '단일 일자'}
              {selectedCode.is_duplicate_blocked ? ' · 중복불가' : ''})
            </span>
          </h2>
          <p className="mt-1 text-sm text-blue-800">{statusText()}</p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {isAssessment && (
            <button
              type="button"
              onClick={() => autoPlaceMutation.mutate(centerYearMonth)}
              disabled={autoPlaceMutation.isPending}
              className="min-h-[44px] rounded-md border border-blue-500 bg-blue-500 px-3 py-2 text-sm font-semibold text-white hover:bg-blue-600 disabled:opacity-50"
            >
              {autoPlaceMutation.isPending ? '배치 중...' : `${centerYearMonth} 자동 배치`}
            </button>
          )}
          {isPeriodType && selectionStart && selectionEnd && (
            <button
              type="button"
              onClick={handleConfirmRange}
              disabled={createMutation.isPending}
              className="min-h-[44px] rounded-md border border-amber-500 bg-amber-500 px-3 py-2 text-sm font-semibold text-white hover:bg-amber-600 disabled:opacity-50"
            >
              확정
            </button>
          )}
          <button
            type="button"
            onClick={onClose}
            disabled={createMutation.isPending || autoPlaceMutation.isPending}
            className="min-h-[44px] rounded-md border border-gray-400 bg-white px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 disabled:opacity-50"
          >
            종료
          </button>
        </div>
      </div>

      {/* 에러 다이얼로그 — 백엔드 한국어 메시지 그대로 (중복불가/지난달 등) */}
      <AlertDialog
        open={errorMessage !== null}
        onOpenChange={(open) => {
          if (!open) setErrorMessage(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>일정 배치 실패</AlertDialogTitle>
            <AlertDialogDescription>{errorMessage}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setErrorMessage(null)}>확인</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 자동 배치 결과 */}
      <AlertDialog
        open={autoPlaceResult !== null}
        onOpenChange={(open) => {
          if (!open) setAutoPlaceResult(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>단원평가 자동 배치</AlertDialogTitle>
            <AlertDialogDescription>{autoPlaceResult}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setAutoPlaceResult(null)}>확인</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}

/**
 * 부모에서 사용할 셀 클릭 핸들러 생성 헬퍼 (mutation 호출).
 * EventPlacer 외부에서 useMutation 을 재사용하기 어렵기 때문에, 별도 hook 으로 분리.
 *
 * - 단일 일자 코드: 셀 클릭 즉시 createScheduleEvent.
 * - 기간성 코드: 셀 클릭 = selection 설정 → 확정 버튼이 mutation 트리거.
 */
export function useEventPlaceCellHandler(params: {
  selectedCode: ScheduleCode | null
  selectionStart: string | null
  setSelectionStart: (s: string | null) => void
  setSelectionEnd: (s: string | null) => void
  onError: (msg: string) => void
}) {
  const queryClient = useQueryClient()
  const createMutation = useMutation({
    mutationFn: createScheduleEvent,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['schedule-events'] })
    },
    onError: (err) => {
      params.onError(err instanceof Error ? err.message : String(err))
    },
  })

  return function handleCellClick(date: string) {
    const code = params.selectedCode
    if (!code) return
    if (code.is_period_type) {
      // 기간성: 첫 클릭=start, 두 번째=end. 이미 양쪽 채워졌으면 새 start.
      if (!params.selectionStart) {
        params.setSelectionStart(date)
      } else {
        params.setSelectionEnd(date)
      }
    } else {
      // 단일 일자: 즉시 INSERT
      createMutation.mutate({
        code_id: code.id,
        event_date: date,
        period_end_date: null,
        display_name: null,
      })
    }
  }
}
