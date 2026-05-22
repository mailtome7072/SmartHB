'use client'

/**
 * 학사 캘린더 1일 셀 — Sprint 6 T9 + T10 + T11 (PRD §4.4.1·§5.7).
 *
 * 시각 요소 (위에서 아래):
 * - 단원평가 응시일: 셀 상단 띠 배지 (PRD §4.4.7)
 * - 날짜 숫자 + 공휴일/일요일 빨강, 토요일 파랑
 * - 일정 배지 (button 으로 분리 — 클릭 시 삭제 또는 드래그)
 *
 * 배경:
 * - 교습기간 안: bg-amber-50
 * - 선택 모드 활성 + 범위 안: bg-blue-100 (우선)
 * - 시작/종료일: ring-2 ring-blue-500
 * - 지난 달 (isPastMonth): opacity-60 + 클릭 차단 (AC-T9-5)
 * - 그리드 외 일자 (isOutsideMonth): opacity-40
 *
 * T11 변경:
 * - 외부 button → div (HTML button 중첩 금지) + role="button" + tabIndex
 * - 일정 배지를 <button> 으로 분리, e.stopPropagation 으로 셀 onClick 차단
 * - onEventClick prop 추가 — T11 일정 삭제 흐름
 */

import type { ScheduleEventListItem } from '@/types/academic'

interface CalendarCellProps {
  date: string                              // "YYYY-MM-DD"
  dayOfMonth: number
  isToday: boolean
  isPastMonth: boolean
  isOutsideMonth: boolean
  isSunday: boolean
  isSaturday: boolean
  inStudyPeriod: boolean
  events: ScheduleEventListItem[]
  isInSelection?: boolean
  isSelectionStart?: boolean
  isSelectionEnd?: boolean
  onClick?: (date: string) => void
  onEventClick?: (event: ScheduleEventListItem) => void
  /** 드래그 가능한 일정 id 집합 — useDraggable hook 은 ThreeMonthCalendar 에서 적용. 본 컴포넌트는 cursor 힌트만. */
  draggableEventIds?: Set<number>
  /** 드롭 가능한 droppable hook ref / props 를 셀에 적용하기 위한 외부 wrapper — T11 드래그. */
  droppableProps?: {
    setNodeRef: (node: HTMLElement | null) => void
    isOver?: boolean
  }
}

/** 코드명 → 배지 색상 매핑. 시스템 6종 + 사용자 코드(기본). */
function codeBadgeClass(codeName: string): string {
  switch (codeName) {
    case '공휴일':
      return 'bg-red-100 text-red-800'
    case '보강데이':
      return 'bg-teal-100 text-teal-800'
    case '공휴수업일':
      return 'bg-pink-100 text-pink-800'
    case '방학':
      return 'bg-purple-100 text-purple-800'
    case '휴원일':
      return 'bg-gray-200 text-gray-700'
    case '단원평가 응시일':
      return 'bg-blue-100 text-blue-800'
    default:
      return 'bg-amber-100 text-amber-800'
  }
}

export function CalendarCell({
  date,
  dayOfMonth,
  isToday,
  isPastMonth,
  isOutsideMonth,
  isSunday,
  isSaturday,
  inStudyPeriod,
  events,
  isInSelection = false,
  isSelectionStart = false,
  isSelectionEnd = false,
  onClick,
  onEventClick,
  draggableEventIds,
  droppableProps,
}: CalendarCellProps) {
  const hasHoliday = events.some((e) => e.code_name === '공휴일')
  const hasAssessment = events.some((e) => e.code_name === '단원평가 응시일')
  const nonAssessmentEvents = events.filter((e) => e.code_name !== '단원평가 응시일')

  const clickable = !isPastMonth && !isOutsideMonth && onClick !== undefined
  const dayColor =
    hasHoliday || isSunday ? 'text-red-700' : isSaturday ? 'text-blue-700' : 'text-[var(--foreground)]'

  const background = isInSelection
    ? 'bg-blue-100'
    : droppableProps?.isOver
      ? 'bg-green-100'
      : inStudyPeriod
        ? 'bg-amber-50'
        : 'bg-white'
  const ring = isSelectionStart || isSelectionEnd ? 'ring-2 ring-blue-500 z-10' : ''

  function handleCellClick() {
    if (clickable) onClick?.(date)
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (!clickable) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      onClick?.(date)
    }
  }

  function handleEventBadgeClick(e: React.MouseEvent, event: ScheduleEventListItem) {
    e.stopPropagation()
    onEventClick?.(event)
  }

  return (
    <div
      ref={droppableProps?.setNodeRef}
      role={clickable ? 'button' : undefined}
      tabIndex={clickable ? 0 : undefined}
      onClick={clickable ? handleCellClick : undefined}
      onKeyDown={clickable ? handleKeyDown : undefined}
      aria-label={`${date}${hasHoliday ? ' 공휴일' : ''}${hasAssessment ? ' 단원평가' : ''}`}
      aria-pressed={isSelectionStart || isSelectionEnd}
      className={[
        'relative flex min-h-[72px] min-w-[44px] flex-col items-stretch border border-[var(--border)] p-1 text-left',
        clickable ? 'focus:outline-none focus:ring-2 focus:ring-blue-400' : '',
        background,
        ring,
        isPastMonth ? 'cursor-not-allowed opacity-60' : '',
        isOutsideMonth ? 'opacity-40' : '',
        clickable && !isInSelection ? 'hover:bg-amber-100 cursor-pointer' : '',
        clickable && isInSelection ? 'hover:bg-blue-200 cursor-pointer' : '',
      ].join(' ')}
    >
      {hasAssessment && (
        <span
          aria-hidden="true"
          className="absolute top-0 left-0 right-0 h-1.5 rounded-t bg-blue-400"
        />
      )}
      <span
        className={[
          'mt-0.5 text-base font-semibold',
          dayColor,
          isToday ? 'rounded bg-blue-100 px-1' : '',
        ].join(' ')}
      >
        {dayOfMonth}
      </span>
      <span className="mt-0.5 flex flex-col gap-0.5">
        {nonAssessmentEvents.slice(0, 3).map((e) => {
          const isDraggable = draggableEventIds?.has(e.id) ?? false
          return (
            <button
              key={e.id}
              type="button"
              data-event-id={e.id}
              data-draggable={isDraggable}
              onClick={(ev) => handleEventBadgeClick(ev, e)}
              disabled={isPastMonth || isOutsideMonth || onEventClick === undefined}
              title={
                isPastMonth
                  ? '지난 달 일정은 수정할 수 없습니다'
                  : (e.display_name ?? e.code_name) +
                    (isDraggable ? ' · 드래그로 이동' : '')
              }
              className={[
                'truncate rounded px-1 text-left text-xs',
                codeBadgeClass(e.code_name),
                onEventClick !== undefined && !isPastMonth && !isOutsideMonth
                  ? 'hover:opacity-80 cursor-pointer'
                  : 'cursor-default',
              ].join(' ')}
            >
              {e.display_name ?? e.code_name}
            </button>
          )
        })}
        {nonAssessmentEvents.length > 3 && (
          <span className="text-xs text-gray-500">+{nonAssessmentEvents.length - 3}</span>
        )}
      </span>
    </div>
  )
}
