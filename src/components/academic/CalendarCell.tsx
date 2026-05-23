'use client'

/**
 * 학사 캘린더 1일 셀 — Sprint 6 T9 + T10 + T11 (PRD §4.4.1·§5.7).
 *
 * 시각 요소 (위에서 아래):
 * - 단원평가 응시일: 일반 배지 텍스트로 표시 (V14 — 이전 셀 상단 띠 표시 폐기)
 * - 날짜 숫자 + 공휴일/일요일 빨강, 토요일 파랑
 * - 일정 배지 EventBadge — 클릭 시 삭제 + 드래그 가능(단일 일자)
 *
 * 배경:
 * - 교습기간 안: bg-amber-50
 * - 선택 모드 활성 + 범위 안: bg-blue-100 (우선)
 * - 드래그 호버: bg-green-100
 * - 시작/종료일: ring-2 ring-blue-500
 * - 지난 달 (isPastMonth): opacity-60 + 클릭 차단 (AC-T9-5)
 * - 그리드 외 일자 (isOutsideMonth): opacity-40
 *
 * T11 변경:
 * - 외부 button → div (HTML button 중첩 금지) + role="button" + tabIndex
 * - 일정 배지를 EventBadge 컴포넌트로 분리, useDraggable hook 적용
 * - onEventClick prop — 삭제 흐름
 * - droppableProps — 부모(DroppableCell wrapper)가 useDroppable hook 결과 전달
 */

import { useDraggable } from '@dnd-kit/core'
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
  /** V23 — 교습기간 내 셀의 수업 가능 여부 (운영시간 + 공휴일/휴원일/공휴수업일 종합). */
  hasClass?: boolean
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

/** @dnd-kit draggable ID 변환 — 숫자 추출은 부모 onDragEnd 에서 동일 패턴으로. */
export function eventDraggableId(eventId: number): string {
  return `event-${eventId}`
}

export function cellDroppableId(date: string): string {
  return `cell-${date}`
}

/** 드래그 가능한 일정 배지 — useDraggable hook 호출은 각 컴포넌트 인스턴스에서 1회. */
interface EventBadgeProps {
  event: ScheduleEventListItem
  /** 본 배지가 표시되는 셀의 일자 — V13 기간성 코드 시작/종료 라벨 분기. */
  cellDate: string
  draggable: boolean
  disabled: boolean        // 지난 달/그리드 외 — 클릭+드래그 모두 차단
  onClick?: (event: ScheduleEventListItem) => void
}

function EventBadge({ event, cellDate, draggable, disabled, onClick }: EventBadgeProps) {
  // Sprint 7 T9 (Issue 7) + V21 (post-review): 시드 공휴일만 클릭 차단. 사용자 추가 공휴일
  // (is_seeded=false)은 일반 클릭 흐름 허용 — 백엔드 delete_schedule_event 가드와 동일 조건.
  const isSeededHoliday =
    event.is_system_reserved && event.code_name === '공휴일' && event.is_seeded
  // V13 (Sprint 7 post-review): 기간성 코드 + 시작일 ≠ 종료일 → 시작 셀 "S" / 종료 셀 "E" 마커.
  const periodMarker = event.is_period_type && event.period_end_date && event.period_end_date !== event.event_date
    ? cellDate === event.event_date
      ? 'S'
      : cellDate === event.period_end_date
        ? 'E'
        : ''
    : ''
  const label = (event.display_name ?? event.code_name) + periodMarker
  const clickable = !disabled && !isSeededHoliday && onClick !== undefined
  const { setNodeRef, listeners, attributes, transform, isDragging } = useDraggable({
    id: eventDraggableId(event.id),
    disabled: disabled || !draggable,
    data: { eventId: event.id, codeName: event.code_name },
  })
  const style = transform
    ? { transform: `translate(${transform.x}px, ${transform.y}px)`, zIndex: 50 }
    : undefined
  return (
    <button
      ref={setNodeRef}
      type="button"
      data-event-id={event.id}
      onClick={(ev) => {
        ev.stopPropagation()
        if (clickable) onClick?.(event)
      }}
      disabled={!clickable}
      title={
        disabled
          ? '지난 달 일정은 수정할 수 없습니다'
          : isSeededHoliday
            ? '시드된 공휴일은 삭제할 수 없습니다'
            : label + (draggable ? ' · 드래그로 이동' : '')
      }
      style={style}
      {...(draggable && !disabled ? listeners : {})}
      {...(draggable && !disabled ? attributes : {})}
      className={[
        'truncate rounded px-1 text-left text-xs',
        codeBadgeClass(event.code_name, event.is_system_reserved),
        clickable ? 'hover:opacity-80 cursor-pointer' : 'cursor-default',
        isDragging ? 'opacity-50' : '',
      ].join(' ')}
    >
      {label}
    </button>
  )
}

/** 시스템 예약 코드의 코드명 → 배지 색상 매핑 (데이터 정의).
 *
 * V102 시드된 시스템 6종. 분기 로직이 아닌 lookup 테이블 — 신규 시스템 코드 추가 시
 * 본 객체에 한 줄만 추가하면 됨 (분기 코드 변경 없음). 매핑 누락 시 USER_BADGE_CLASS 로 폴백.
 */
const SYSTEM_BADGE_CLASS: Record<string, string> = {
  '공휴일': 'bg-red-100 text-red-800',
  '보강데이': 'bg-teal-100 text-teal-800',
  '공휴수업일': 'bg-pink-100 text-pink-800',
  '방학': 'bg-purple-100 text-purple-800',
  '휴원일': 'bg-gray-200 text-gray-700',
  '단원평가 응시일': 'bg-blue-100 text-blue-800',
}

const USER_BADGE_CLASS = 'bg-amber-100 text-amber-800'

/** 코드명 → 배지 색상 매핑.
 *
 * Sprint 7 T4 (R33): `isSystemReserved` 플래그 기반 분기 — 사용자 코드는 amber 고정.
 * 시스템 코드는 [[SYSTEM_BADGE_CLASS]] lookup, 누락 시 사용자 코드 색상으로 폴백.
 */
function codeBadgeClass(codeName: string, isSystemReserved: boolean): string {
  if (!isSystemReserved) return USER_BADGE_CLASS
  return SYSTEM_BADGE_CLASS[codeName] ?? USER_BADGE_CLASS
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
  hasClass = false,
  events,
  isInSelection = false,
  isSelectionStart = false,
  isSelectionEnd = false,
  onClick,
  onEventClick,
  draggableEventIds,
  droppableProps,
}: CalendarCellProps) {
  // V14 (Sprint 7 post-review): 단원평가 셀 상단 색 라인 제거 — 일반 배지로 통일.
  const hasHoliday = events.some((e) => e.code_name === '공휴일')

  const clickable = !isPastMonth && !isOutsideMonth && onClick !== undefined
  const dayColor =
    hasHoliday || isSunday ? 'text-red-700' : isSaturday ? 'text-blue-700' : 'text-[var(--foreground)]'

  // V22 + V23 (Sprint 7 post-review): 교습기간 셀 배경 강화 + 수업 가능/불가 구분.
  // - 교습기간 + 수업 가능 (운영 요일·공휴 없음·휴원 없음 or 공휴수업일): bg-amber-100 (진함)
  // - 교습기간 + 수업 불가 (미운영/공휴/휴원): bg-gray-100 (회색 — 시각 구분)
  // selection / 드롭 hover 우선순위 유지.
  const studyBg = inStudyPeriod ? (hasClass ? 'bg-amber-100' : 'bg-gray-100') : 'bg-white'
  const background = isInSelection
    ? 'bg-blue-100'
    : droppableProps?.isOver
      ? 'bg-green-100'
      : studyBg
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

  return (
    <div
      ref={droppableProps?.setNodeRef}
      role={clickable ? 'button' : undefined}
      tabIndex={clickable ? 0 : undefined}
      onClick={clickable ? handleCellClick : undefined}
      onKeyDown={clickable ? handleKeyDown : undefined}
      aria-label={`${date}${hasHoliday ? ' 공휴일' : ''}`}
      aria-pressed={isSelectionStart || isSelectionEnd}
      className={[
        'relative flex min-h-[72px] min-w-[44px] flex-col items-stretch border border-[var(--border)] p-1 text-left',
        clickable ? 'focus:outline-none focus:ring-2 focus:ring-blue-400' : '',
        background,
        ring,
        isPastMonth ? 'cursor-not-allowed opacity-60' : '',
        isOutsideMonth ? 'opacity-40' : '',
        clickable && !isInSelection ? 'hover:bg-amber-200 cursor-pointer' : '',
        clickable && isInSelection ? 'hover:bg-blue-200 cursor-pointer' : '',
      ].join(' ')}
    >
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
        {events.slice(0, 3).map((e) => (
          <EventBadge
            key={`${e.id}-${date}`}
            event={e}
            cellDate={date}
            draggable={draggableEventIds?.has(e.id) ?? false}
            disabled={isPastMonth || isOutsideMonth}
            onClick={onEventClick}
          />
        ))}
        {events.length > 3 && (
          <span className="text-xs text-gray-500">+{events.length - 3}</span>
        )}
      </span>
    </div>
  )
}
