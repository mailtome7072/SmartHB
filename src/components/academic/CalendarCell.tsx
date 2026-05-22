'use client'

/**
 * 학사 캘린더 1일 셀 — Sprint 6 T9 (PRD §4.4.1·§5.7).
 *
 * 시각 요소 (위에서 아래):
 * - 단원평가 응시일: 셀 상단 띠 배지 (PRD §4.4.7)
 * - 날짜 숫자 + 공휴일이면 빨간색 (한국 캘린더 관습)
 * - 공휴일 배지 (display_name)
 * - 학사 일정 배지 (코드별 색상)
 *
 * 배경:
 * - 교습기간 안: 파스텔 amber-50
 * - 지난 달 (isPastMonth): opacity-60 + cursor-not-allowed — 클릭 차단 (AC-T9-5)
 * - 그리드 외 일자 (isOutsideMonth): 옅은 텍스트
 *
 * 접근성: 44×44px 최소, Pretendard 18pt, WCAG AA.
 */

import type { ScheduleEventListItem } from '@/types/academic'

interface CalendarCellProps {
  date: string                              // "YYYY-MM-DD"
  dayOfMonth: number                        // 1~31
  isToday: boolean
  isPastMonth: boolean                      // 셀이 속한 캘린더 표시 월이 지난 달인지 (전체 차단)
  isOutsideMonth: boolean                   // 그리드 leading/trailing 일자 (이전/다음 달 표시)
  isSunday: boolean
  isSaturday: boolean
  inStudyPeriod: boolean
  events: ScheduleEventListItem[]           // 해당 일자의 schedule_events (공휴일 포함)
  onClick?: (date: string) => void
}

/** 코드명 → 배지 색상 매핑. 시스템 5종 + "공휴일" + 사용자 코드(기본). */
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
  onClick,
}: CalendarCellProps) {
  const hasHoliday = events.some((e) => e.code_name === '공휴일')
  const hasAssessment = events.some((e) => e.code_name === '단원평가 응시일')
  const nonAssessmentEvents = events.filter((e) => e.code_name !== '단원평가 응시일')

  const clickable = !isPastMonth && !isOutsideMonth && onClick !== undefined
  const dayColor =
    hasHoliday || isSunday ? 'text-red-700' : isSaturday ? 'text-blue-700' : 'text-[var(--foreground)]'

  return (
    <button
      type="button"
      onClick={clickable ? () => onClick(date) : undefined}
      disabled={!clickable}
      aria-label={`${date}${hasHoliday ? ' 공휴일' : ''}${hasAssessment ? ' 단원평가' : ''}`}
      className={[
        'relative flex min-h-[72px] min-w-[44px] flex-col items-stretch border border-[var(--border)] p-1 text-left',
        'focus:outline-none focus:ring-2 focus:ring-blue-400',
        inStudyPeriod ? 'bg-amber-50' : 'bg-white',
        isPastMonth ? 'cursor-not-allowed opacity-60' : '',
        isOutsideMonth ? 'opacity-40' : '',
        clickable ? 'hover:bg-amber-100' : '',
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
        {nonAssessmentEvents.slice(0, 3).map((e) => (
          <span
            key={e.id}
            title={e.display_name ?? e.code_name}
            className={['truncate rounded px-1 text-xs', codeBadgeClass(e.code_name)].join(' ')}
          >
            {e.display_name ?? e.code_name}
          </span>
        ))}
        {nonAssessmentEvents.length > 3 && (
          <span className="text-xs text-gray-500">+{nonAssessmentEvents.length - 3}</span>
        )}
      </span>
    </button>
  )
}
