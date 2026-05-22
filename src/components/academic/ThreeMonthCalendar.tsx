'use client'

/**
 * 3개월 캘린더 — Sprint 6 T9 (PRD §4.4.1).
 *
 * 레이아웃:
 *   ┌──────────────────────────────────────────────────────────┐
 *   │ [← 이전월]   {중앙 YYYY-MM}   [익월 →]                    │
 *   ├──────────────┬──────────────┬───────────────────────────┤
 *   │  이전월       │  중앙월       │  익월                       │
 *   │  (지난 달이면 │  (기본 다음달) │                            │
 *   │   읽기전용)   │              │                            │
 *   └──────────────┴──────────────┴───────────────────────────┘
 *
 * - 화살표 클릭 시 중앙월 ±1 → 좌/우도 연동 이동 (AC-T9-2)
 * - 좁은 화면(`md:` 미만)에서는 세로 스택 (AC-T9-1)
 * - TanStack Query 로 IPC 응답 캐싱 — 월 이동 시 자동 refetch
 * - 셀 onClick prop 는 T10/T11 에서 모드별 핸들러로 확장
 */

import { useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { listScheduleEvents, listStudyPeriods } from '@/lib/tauri'
import type { ScheduleEventListItem, StudyPeriod } from '@/types/academic'
import { CalendarCell } from './CalendarCell'

// ─── 날짜 유틸 (라이브러리 무의존) ─────────────────────────────────────────

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

function ymd(year: number, month: number, day: number): string {
  return `${year}-${pad2(month)}-${pad2(day)}`
}

function daysInMonth(year: number, month: number): number {
  return new Date(year, month, 0).getDate()
}

/** JS getDay(): 0=일, 1=월…6=토 → 월요일 시작 그리드 leading 칸 수 (0~6). */
function leadingBlankCount(year: number, month: number): number {
  const jsDay = new Date(year, month - 1, 1).getDay()
  return (jsDay + 6) % 7
}

function shiftMonth(year: number, month: number, delta: number): { year: number; month: number } {
  const d = new Date(year, month - 1 + delta, 1)
  return { year: d.getFullYear(), month: d.getMonth() + 1 }
}

function todayYmd(): string {
  const d = new Date()
  return ymd(d.getFullYear(), d.getMonth() + 1, d.getDate())
}

function currentYearMonth(): { year: number; month: number } {
  const d = new Date()
  return { year: d.getFullYear(), month: d.getMonth() + 1 }
}

const WEEKDAY_LABELS = ['월', '화', '수', '목', '금', '토', '일']

interface MonthCell {
  date: string             // "YYYY-MM-DD"
  dayOfMonth: number
  isOutsideMonth: boolean
  isSunday: boolean
  isSaturday: boolean
  isToday: boolean
}

/** 월요일 시작 6×7=42 셀 그리드. 그리드 밖 일자도 채워서 outside flag 부여. */
function buildMonthGrid(year: number, month: number, today: string): MonthCell[] {
  const cells: MonthCell[] = []
  const lead = leadingBlankCount(year, month)
  const days = daysInMonth(year, month)
  const prev = shiftMonth(year, month, -1)
  const prevDays = daysInMonth(prev.year, prev.month)

  // Leading (이전월 꼬리)
  for (let i = 0; i < lead; i++) {
    const day = prevDays - lead + 1 + i
    const date = ymd(prev.year, prev.month, day)
    const jsDay = new Date(prev.year, prev.month - 1, day).getDay()
    cells.push({
      date,
      dayOfMonth: day,
      isOutsideMonth: true,
      isSunday: jsDay === 0,
      isSaturday: jsDay === 6,
      isToday: date === today,
    })
  }

  // Current month
  for (let day = 1; day <= days; day++) {
    const date = ymd(year, month, day)
    const jsDay = new Date(year, month - 1, day).getDay()
    cells.push({
      date,
      dayOfMonth: day,
      isOutsideMonth: false,
      isSunday: jsDay === 0,
      isSaturday: jsDay === 6,
      isToday: date === today,
    })
  }

  // Trailing (다음월 머리) — 6주 = 42셀까지 채움
  const next = shiftMonth(year, month, 1)
  let trailingDay = 1
  while (cells.length < 42) {
    const date = ymd(next.year, next.month, trailingDay)
    const jsDay = new Date(next.year, next.month - 1, trailingDay).getDay()
    cells.push({
      date,
      dayOfMonth: trailingDay,
      isOutsideMonth: true,
      isSunday: jsDay === 0,
      isSaturday: jsDay === 6,
      isToday: date === today,
    })
    trailingDay++
  }

  return cells
}

// ─── 단일 월 그리드 컴포넌트 ───────────────────────────────────────────────

interface MonthGridProps {
  year: number
  month: number
  isPastMonth: boolean
  eventsByDate: Map<string, ScheduleEventListItem[]>
  studyPeriod: StudyPeriod | null
  today: string
  onCellClick?: (date: string) => void
}

function MonthGrid({
  year,
  month,
  isPastMonth,
  eventsByDate,
  studyPeriod,
  today,
  onCellClick,
}: MonthGridProps) {
  const cells = useMemo(() => buildMonthGrid(year, month, today), [year, month, today])

  function inStudyPeriod(date: string): boolean {
    if (!studyPeriod) return false
    return date >= studyPeriod.start_date && date <= studyPeriod.end_date
  }

  return (
    <section
      aria-label={`${year}년 ${month}월`}
      className="flex flex-col rounded-lg border border-[var(--border)] bg-white p-3"
    >
      <header className="mb-2 flex items-center justify-between">
        <h3 className="text-lg font-bold text-[var(--foreground)]">
          {year}년 {month}월
        </h3>
        {studyPeriod !== null && (
          <span className="text-xs text-amber-700">
            교습기간 {studyPeriod.start_date.slice(5)} ~ {studyPeriod.end_date.slice(5)}
            {studyPeriod.is_confirmed ? ' · 확정' : ''}
          </span>
        )}
      </header>
      <div className="grid grid-cols-7 gap-px text-center text-sm font-semibold text-gray-600">
        {WEEKDAY_LABELS.map((d, i) => (
          <div
            key={d}
            className={['py-1', i === 5 ? 'text-blue-700' : i === 6 ? 'text-red-700' : ''].join(' ')}
          >
            {d}
          </div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-px">
        {cells.map((c) => (
          <CalendarCell
            key={c.date}
            date={c.date}
            dayOfMonth={c.dayOfMonth}
            isToday={c.isToday}
            isPastMonth={isPastMonth}
            isOutsideMonth={c.isOutsideMonth}
            isSunday={c.isSunday}
            isSaturday={c.isSaturday}
            inStudyPeriod={!c.isOutsideMonth && inStudyPeriod(c.date)}
            events={eventsByDate.get(c.date) ?? []}
            onClick={onCellClick}
          />
        ))}
      </div>
    </section>
  )
}

// ─── 3개월 컨테이너 ────────────────────────────────────────────────────────

interface ThreeMonthCalendarProps {
  /** 셀 클릭 핸들러 — T10/T11 에서 모드별 분기 (교습기간 설정 / 일정 배치). */
  onCellClick?: (date: string) => void
}

export function ThreeMonthCalendar({ onCellClick }: ThreeMonthCalendarProps) {
  // 중앙 = 기본 다음 달 (PRD §4.4.1).
  const [center, setCenter] = useState<{ year: number; month: number }>(() => {
    const cur = currentYearMonth()
    return shiftMonth(cur.year, cur.month, 1)
  })
  const prev = useMemo(() => shiftMonth(center.year, center.month, -1), [center])
  const next = useMemo(() => shiftMonth(center.year, center.month, 1), [center])
  const today = useMemo(() => todayYmd(), [])
  const currentYm = useMemo(() => currentYearMonth(), [])

  // IPC 조회 범위: 이전월 1일 ~ 익월 말일.
  const rangeFrom = ymd(prev.year, prev.month, 1)
  const rangeTo = ymd(next.year, next.month, daysInMonth(next.year, next.month))
  const studyFromMonth = `${prev.year}-${pad2(prev.month)}`
  const studyToMonth = `${next.year}-${pad2(next.month)}`

  const eventsQuery = useQuery({
    queryKey: ['schedule-events', rangeFrom, rangeTo],
    queryFn: () => listScheduleEvents(rangeFrom, rangeTo),
    staleTime: 30_000,
  })

  const periodsQuery = useQuery({
    queryKey: ['study-periods', studyFromMonth, studyToMonth],
    queryFn: () => listStudyPeriods(studyFromMonth, studyToMonth),
    staleTime: 30_000,
  })

  // event_date → events 매핑 (성능: O(n) → O(1) 셀 렌더 조회).
  const eventsByDate = useMemo(() => {
    const map = new Map<string, ScheduleEventListItem[]>()
    for (const e of eventsQuery.data ?? []) {
      const arr = map.get(e.event_date) ?? []
      arr.push(e)
      map.set(e.event_date, arr)
    }
    return map
  }, [eventsQuery.data])

  // year_month → StudyPeriod 매핑.
  const periodByYm = useMemo(() => {
    const map = new Map<string, StudyPeriod>()
    for (const p of periodsQuery.data ?? []) {
      map.set(p.year_month, p)
    }
    return map
  }, [periodsQuery.data])

  function isMonthPast(year: number, month: number): boolean {
    return year < currentYm.year || (year === currentYm.year && month < currentYm.month)
  }

  function shift(delta: number) {
    setCenter((c) => shiftMonth(c.year, c.month, delta))
  }

  return (
    <div className="flex flex-col gap-3">
      <nav className="flex items-center justify-center gap-4">
        <button
          type="button"
          onClick={() => shift(-1)}
          aria-label="이전 달"
          className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50"
        >
          ← 이전
        </button>
        <span className="min-w-[8rem] text-center text-lg font-bold text-[var(--foreground)]">
          {center.year}년 {center.month}월
        </span>
        <button
          type="button"
          onClick={() => shift(1)}
          aria-label="다음 달"
          className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50"
        >
          다음 →
        </button>
      </nav>

      {(eventsQuery.isError || periodsQuery.isError) && (
        <div role="alert" className="rounded border border-red-300 bg-red-50 p-2 text-sm text-red-800">
          캘린더 데이터를 불러오지 못했습니다. 새로고침을 시도하세요.
        </div>
      )}

      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        {[prev, center, next].map((m) => (
          <MonthGrid
            key={`${m.year}-${m.month}`}
            year={m.year}
            month={m.month}
            isPastMonth={isMonthPast(m.year, m.month)}
            eventsByDate={eventsByDate}
            studyPeriod={periodByYm.get(`${m.year}-${pad2(m.month)}`) ?? null}
            today={today}
            onCellClick={onCellClick}
          />
        ))}
      </div>
    </div>
  )
}
