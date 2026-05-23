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

import { useCallback, useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  DndContext,
  PointerSensor,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  getOperatingHours,
  listScheduleEvents,
  listStudyPeriods,
  type DayHours,
} from '@/lib/tauri'
import type { ScheduleEventListItem, StudyPeriod } from '@/types/academic'
import { CalendarCell, cellDroppableId } from './CalendarCell'

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

/** "YYYY-MM-DD" → 다음 날짜 "YYYY-MM-DD" (V20). UTC 명시로 timezone 영향 회피. */
function nextIsoDate(date: string): string {
  const [y, m, d] = date.split('-').map(Number)
  const dt = new Date(Date.UTC(y, m - 1, d))
  dt.setUTCDate(dt.getUTCDate() + 1)
  return dt.toISOString().slice(0, 10)
}

/** "YYYY-MM-DD" → ISO 요일 (1=월 ~ 7=일). V23 — 운영 시간 매칭에 사용. */
function isoDayOfWeek(date: string): number {
  const [y, m, d] = date.split('-').map(Number)
  const jsDay = new Date(Date.UTC(y, m - 1, d)).getUTCDay() // 0=일~6=토
  return jsDay === 0 ? 7 : jsDay
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

interface SelectionRange {
  start: string | null
  end: string | null
}

// ─── DroppableCell — useDroppable hook 을 CalendarCell 외부 wrapper 에서 호출 ────

interface DroppableCellProps extends React.ComponentProps<typeof CalendarCell> {
  isDropDisabled: boolean
}

function DroppableCell({ isDropDisabled, ...cellProps }: DroppableCellProps) {
  const { setNodeRef, isOver } = useDroppable({
    id: cellDroppableId(cellProps.date),
    disabled: isDropDisabled,
  })
  return (
    <CalendarCell
      {...cellProps}
      droppableProps={{ setNodeRef, isOver }}
    />
  )
}

interface MonthGridProps {
  year: number
  month: number
  isPastMonth: boolean
  eventsByDate: Map<string, ScheduleEventListItem[]>
  /** 현재 month 와 year_month 가 일치하는 교습기간 — 헤더에 표시. */
  studyPeriod: StudyPeriod | null
  /** 전체 교습기간 리스트 — 셀의 `inStudyPeriod` 색 판정에 사용 (cross-month 포함, V7). */
  allStudyPeriods: StudyPeriod[]
  /** V23 — 셀이 수업 가능 일자인지 판정 콜백 (운영시간 + 공휴일/휴원일/공휴수업일 종합). */
  hasClassOnDate: (date: string) => boolean
  today: string
  selection?: SelectionRange
  /** 드래그 가능한 일정 id 집합 (단일 일자 + 시스템 코드 제외 등 부모가 계산). */
  draggableEventIds?: Set<number>
  onCellClick?: (date: string) => void
  onEventClick?: (event: ScheduleEventListItem) => void
  /** V15 — 확대 보기 버튼 핸들러. 헤더에 ⛶ 버튼 노출. 본 그리드가 이미 확대 모드면 undefined. */
  onExpand?: () => void
}

function MonthGrid({
  year,
  month,
  isPastMonth,
  eventsByDate,
  studyPeriod,
  allStudyPeriods,
  hasClassOnDate,
  today,
  selection,
  draggableEventIds,
  onCellClick,
  onEventClick,
  onExpand,
}: MonthGridProps) {
  const cells = useMemo(() => buildMonthGrid(year, month, today), [year, month, today])

  // V7 (Sprint 7 post-review): 교습기간이 month 경계를 넘어가는 경우(예: 6월 교습기간 = 5/29~6/30)
  // 5월 그리드에도 5/29~5/31 부분이 인-스터디 셀로 표시되어야 함. 단일 studyPeriod 대신 전체
  // 리스트를 순회하여 date 포함 여부 판정.
  function inStudyPeriod(date: string): boolean {
    return allStudyPeriods.some((p) => date >= p.start_date && date <= p.end_date)
  }

  /** 선택 범위 내 — start <= date <= end (둘 다 있을 때) 또는 start === date (start 만 있을 때). */
  function inSelectionRange(date: string): boolean {
    if (!selection?.start) return false
    if (!selection.end) return date === selection.start
    const lo = selection.start <= selection.end ? selection.start : selection.end
    const hi = selection.start <= selection.end ? selection.end : selection.start
    return date >= lo && date <= hi
  }

  return (
    <section
      aria-label={`${year}년 ${month}월`}
      className="flex flex-col rounded-lg border border-[var(--border)] bg-white p-3"
    >
      <header className="mb-2 flex items-center justify-between gap-2">
        <h3 className="text-lg font-bold text-[var(--foreground)]">
          {year}년 {month}월
        </h3>
        <div className="flex items-center gap-2">
          {studyPeriod !== null && (
            <span className="text-xs text-amber-700">
              교습기간 {studyPeriod.start_date.slice(5)} ~ {studyPeriod.end_date.slice(5)}
              {studyPeriod.is_confirmed ? ' · 확정' : ''}
              {isPastMonth ? ' · 🔒 수정 불가' : ''}
            </span>
          )}
          {/* V15 — 크게 보기 버튼. */}
          {onExpand !== undefined && (
            <button
              type="button"
              onClick={onExpand}
              aria-label={`${year}년 ${month}월 크게 보기`}
              title="크게 보기"
              className="min-h-[32px] min-w-[32px] rounded border border-[var(--border)] bg-white px-2 text-base hover:bg-gray-50"
            >
              ⛶
            </button>
          )}
        </div>
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
          <DroppableCell
            key={c.date}
            date={c.date}
            dayOfMonth={c.dayOfMonth}
            isToday={c.isToday}
            isPastMonth={isPastMonth}
            isOutsideMonth={c.isOutsideMonth}
            isSunday={c.isSunday}
            isSaturday={c.isSaturday}
            inStudyPeriod={!c.isOutsideMonth && inStudyPeriod(c.date)}
            hasClass={!c.isOutsideMonth && hasClassOnDate(c.date)}
            events={eventsByDate.get(c.date) ?? []}
            isInSelection={!c.isOutsideMonth && inSelectionRange(c.date)}
            isSelectionStart={selection?.start === c.date}
            isSelectionEnd={selection?.end === c.date}
            draggableEventIds={draggableEventIds}
            isDropDisabled={isPastMonth || c.isOutsideMonth}
            onClick={onCellClick}
            onEventClick={onEventClick}
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
  /** 선택 모드에서 시작/종료일 프리뷰 (Sprint 6 T10). null = 선택 모드 비활성. */
  selection?: SelectionRange | null
  /** 일정 배지 클릭 (Sprint 6 T11) — 삭제 핸들러. */
  onEventClick?: (event: ScheduleEventListItem) => void
  /** 중앙 month 변경 콜백 — 부모가 자동 배치 등 month 기반 액션에 사용. */
  onCenterChange?: (yearMonth: string) => void
  /** 일정 드래그 이동 완료 (Sprint 6 T11) — 부모가 updateScheduleEvent 호출. */
  onEventDrop?: (event: ScheduleEventListItem, newDate: string) => void
}

export function ThreeMonthCalendar({
  onCellClick,
  selection,
  onEventClick,
  onCenterChange,
  onEventDrop,
}: ThreeMonthCalendarProps) {
  // 중앙 = 기본 다음 달 (PRD §4.4.1).
  // V15 — 확대 보기 모드 (단일 월 큰 그리드). null 이면 3개월 그리드.
  const [expandedMonth, setExpandedMonth] = useState<{ year: number; month: number } | null>(null)
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

  // V23 (Sprint 7 post-review): 운영 시간 조회 — 셀별 수업 가능/불가 판정에 사용.
  const operatingHoursQuery = useQuery({
    queryKey: ['operating-hours'],
    queryFn: getOperatingHours,
    staleTime: 5 * 60_000,
  })

  // event_date → events 매핑 (성능: O(n) → O(1) 셀 렌더 조회).
  // V13: 기간성 코드는 시작/종료 셀에 "S"/"E" 마커 분기.
  // V20 (Sprint 7 post-review): 기간성 코드의 시작~종료 **사이 모든 일자** 에도 매핑하여
  // 셀 전체 구간에 이벤트가 보이도록. EventBadge 는 cellDate 비교로 시작/종료/사이 분기.
  const eventsByDate = useMemo(() => {
    const map = new Map<string, ScheduleEventListItem[]>()
    for (const e of eventsQuery.data ?? []) {
      const startArr = map.get(e.event_date) ?? []
      startArr.push(e)
      map.set(e.event_date, startArr)
      if (
        e.is_period_type &&
        e.period_end_date &&
        e.period_end_date !== e.event_date
      ) {
        // 시작 다음 날부터 종료일까지 모든 일자에 동일 event 추가.
        let cursor = nextIsoDate(e.event_date)
        while (cursor <= e.period_end_date) {
          const arr = map.get(cursor) ?? []
          arr.push(e)
          map.set(cursor, arr)
          cursor = nextIsoDate(cursor)
        }
      }
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

  // 마운트 시 + center 변경 시 부모에 알림 (자동 배치 등 month 기반 액션에 사용).
  useEffect(() => {
    onCenterChange?.(`${center.year}-${pad2(center.month)}`)
  }, [center, onCenterChange])

  // 드래그 가능 일정 id 집합 — 단일 일자(`is_period_type=0`) + 시스템 예약 아닌 코드.
  // 공휴일·단원평가 등 시스템 코드 이동은 의도 외 동작 위험 → UI 차단.
  // Sprint 7 T4 (R33): 시스템 코드명 한국어 리터럴 6종 Set → 백엔드 `is_system_reserved`
  // 플래그 기반으로 전환. 향후 시스템 코드 추가/변경 시 프론트 수정 불필요.
  const draggableEventIds = useMemo(() => {
    const ids = new Set<number>()
    for (const e of eventsQuery.data ?? []) {
      if (!e.is_period_type && !e.is_system_reserved) ids.add(e.id)
    }
    return ids
  }, [eventsQuery.data])

  // V23 + V25 + V28 (Sprint 7 post-review): 셀별 수업 가능 여부 판정.
  // - 운영 시간 (해당 요일 open_time 있어야 함)
  // - 이벤트가 있을 때: 어떤 이벤트라도 `allows_regular_class || allows_makeup_class` 면 수업 가능
  //   - V25: 정규수업 허용 (공휴수업일, 단원평가)
  //   - V28: 보강수업 허용 (보강데이, 공휴수업일)
  //   - 방학·휴원일·공휴일 = 둘 다 0 → 수업 불가
  // - 이벤트 없을 때 + 운영일 = 수업 가능
  const hasClassOnDate = useCallback(
    (date: string): boolean => {
      const dow = isoDayOfWeek(date)
      const dayHours = (operatingHoursQuery.data ?? []).find(
        (h: DayHours) => h.day_of_week === dow,
      )
      const isOperatingDay =
        dayHours !== undefined &&
        dayHours.open_time !== null &&
        dayHours.close_time !== null
      if (!isOperatingDay) return false

      const cellEvents = eventsByDate.get(date) ?? []
      if (cellEvents.length === 0) return true
      return cellEvents.some(
        (e) => e.allows_regular_class || e.allows_makeup_class,
      )
    },
    [eventsByDate, operatingHoursQuery.data],
  )

  // 드래그 센서 — pointer 8px 이동 후 활성화 (배지 클릭과 구분).
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 8 } }))

  function handleDragEnd(e: DragEndEvent) {
    if (!onEventDrop) return
    const overId = e.over?.id
    if (typeof overId !== 'string' || !overId.startsWith('cell-')) return
    const newDate = overId.slice(5)
    const eventId = Number((e.active.data.current as { eventId?: number })?.eventId)
    if (!Number.isFinite(eventId)) return
    const event = (eventsQuery.data ?? []).find((x) => x.id === eventId)
    if (!event) return
    if (event.event_date === newDate) return
    onEventDrop(event, newDate)
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
    <div className="flex flex-col gap-3">
      {/* V31 (Sprint 7 post-review): 확대 보기 모드에서는 prev/next 비활성화 + 확대된 월을
          타이틀로 표시. 3개월 보기 복귀 후에는 다시 활성화 + center 월 표시. */}
      <nav className="flex items-center justify-center gap-4">
        <button
          type="button"
          onClick={() => shift(-1)}
          aria-label="이전 달"
          disabled={expandedMonth !== null}
          className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
        >
          ← 이전
        </button>
        <span className="min-w-[8rem] text-center text-lg font-bold text-[var(--foreground)]">
          {expandedMonth
            ? `${expandedMonth.year}년 ${expandedMonth.month}월`
            : `${center.year}년 ${center.month}월`}
        </span>
        <button
          type="button"
          onClick={() => shift(1)}
          aria-label="다음 달"
          disabled={expandedMonth !== null}
          className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
        >
          다음 →
        </button>
      </nav>

      {(eventsQuery.isError || periodsQuery.isError) && (
        <div role="alert" className="rounded border border-red-300 bg-red-50 p-2 text-sm text-red-800">
          캘린더 데이터를 불러오지 못했습니다. 새로고침을 시도하세요.
        </div>
      )}

      {/* V15 — 확대 모드: 단일 월 크게. 일반 모드: 3개월 동시. */}
      {expandedMonth ? (
        <div className="flex flex-col gap-2">
          <button
            type="button"
            onClick={() => setExpandedMonth(null)}
            className="self-start min-h-[44px] rounded-md border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50"
          >
            ← 3개월 보기로 돌아가기
          </button>
          <div className="mx-auto w-full max-w-5xl">
            <MonthGrid
              year={expandedMonth.year}
              month={expandedMonth.month}
              isPastMonth={isMonthPast(expandedMonth.year, expandedMonth.month)}
              eventsByDate={eventsByDate}
              studyPeriod={
                periodByYm.get(`${expandedMonth.year}-${pad2(expandedMonth.month)}`) ?? null
              }
              allStudyPeriods={periodsQuery.data ?? []}
              hasClassOnDate={hasClassOnDate}
              today={today}
              selection={selection ?? undefined}
              draggableEventIds={draggableEventIds}
              onCellClick={onCellClick}
              onEventClick={onEventClick}
            />
          </div>
        </div>
      ) : (
      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        {[prev, center, next].map((m) => (
          <MonthGrid
            key={`${m.year}-${m.month}`}
            year={m.year}
            month={m.month}
            isPastMonth={isMonthPast(m.year, m.month)}
            eventsByDate={eventsByDate}
            studyPeriod={periodByYm.get(`${m.year}-${pad2(m.month)}`) ?? null}
            allStudyPeriods={periodsQuery.data ?? []}
            hasClassOnDate={hasClassOnDate}
            today={today}
            selection={selection ?? undefined}
            draggableEventIds={draggableEventIds}
            onCellClick={onCellClick}
            onEventClick={onEventClick}
            onExpand={() => setExpandedMonth({ year: m.year, month: m.month })}
          />
        ))}
      </div>
      )}
    </div>
    </DndContext>
  )
}
