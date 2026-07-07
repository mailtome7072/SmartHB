'use client'

/**
 * 교습일정 인쇄용 컴포넌트 — T9/이슈10 (Sprint 18), Sprint 19 T4 개선.
 *
 * window.print() 호출 시 @media print 스타일로 A4 세로 레이아웃으로 출력된다.
 * 요일 순서: 일월화수목금토 (T5에서 변경된 일요일 시작).
 *
 * Sprint 19 T4(사용자 요청 4번): 공지문 생성 캘린더(`src/lib/calendar-image.ts`)와 동일한
 * 규칙으로 수업 가능일 red 외곽선 + 기간성 학사일정 밴드 오버레이를 추가한다. 두 컴포넌트는
 * 렌더링 기술(canvas vs HTML table)이 달라 로직을 공유하지 못하지만, "수업 가능일 판정"과
 * "외곽선 트림 규칙"은 의도적으로 동일하게 포팅했다 — 정책이 바뀌면 두 곳 모두 확인 필요.
 */

import { useQuery } from '@tanstack/react-query'
import type { StudyPeriod, ScheduleEventListItem } from '@/types/academic'
import { codeColor } from '@/lib/schedule-code-colors'
import { getOperatingHours } from '@/lib/tauri'
import { isWeekday, isoDayOfWeek, nextIsoDate, prevIsoDate } from '@/lib/time'

interface Props {
  period: StudyPeriod
  events: ScheduleEventListItem[]
}

const DOW_LABELS = ['일', '월', '화', '수', '목', '금', '토']

interface GridCell {
  day: number
  outsideMonth: boolean
  dow: number
}

function buildCalendarGrid(year: number, month: number): GridCell[] {
  const firstDow = new Date(year, month - 1, 1).getDay() // 0=일
  const daysInMonth = new Date(year, month, 0).getDate()
  const prevDays = new Date(year, month - 1, 0).getDate()
  const cells: GridCell[] = []

  for (let i = 0; i < firstDow; i++) {
    const d = prevDays - firstDow + 1 + i
    cells.push({ day: d, outsideMonth: true, dow: i })
  }
  for (let d = 1; d <= daysInMonth; d++) {
    cells.push({ day: d, outsideMonth: false, dow: (firstDow + d - 1) % 7 })
  }
  // A116: 필요한 주(week) 수만큼만 채운다 — 5주로 끝나는 달에서 빈 6번째 행 제거
  const totalCells = Math.ceil(cells.length / 7) * 7
  const remaining = totalCells - cells.length
  for (let i = 1; i <= remaining; i++) {
    cells.push({ day: i, outsideMonth: true, dow: (firstDow + daysInMonth + i - 1) % 7 })
  }
  return cells
}

function dateStr(year: number, month: number, day: number): string {
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
}

/** 두 날짜의 중간 날짜 — 밴드 라벨을 표시할 셀 위치 판단(사용자 확정: 밴드 중앙 1회). */
function midIsoDate(start: string, end: string): string {
  const s = new Date(`${start}T00:00:00Z`).getTime()
  const e = new Date(`${end}T00:00:00Z`).getTime()
  return new Date((s + e) / 2).toISOString().slice(0, 10)
}

/** 여러 날에 걸친 기간성 학사일정(방학 등) — 개별 셀 반복 라벨 대신 밴드로 표현. */
function isBandEvent(e: ScheduleEventListItem): boolean {
  return e.is_period_type && e.period_end_date !== null && e.period_end_date !== e.event_date
}

export function AcademicSchedulePrint({ period, events }: Props) {
  const { data: operatingHours = [] } = useQuery({
    queryKey: ['operating-hours'],
    queryFn: getOperatingHours,
  })

  const [sy, sm] = period.start_date.split('-').map(Number)
  const [ey, em] = period.end_date.split('-').map(Number)

  // 교습기간 내 달 목록 (보통 1~2개월)
  const months: { year: number; month: number }[] = []
  let cy = sy
  let cm = sm
  while (cy < ey || (cy === ey && cm <= em)) {
    months.push({ year: cy, month: cm })
    if (cm === 12) {
      cy++
      cm = 1
    } else {
      cm++
    }
  }

  // 날짜별 이벤트 맵 — 기간성 코드는 start~end 전 일자에 전개
  const eventByDate = new Map<string, ScheduleEventListItem[]>()
  for (const ev of events) {
    const end = ev.period_end_date ?? ev.event_date
    const s = new Date(ev.event_date)
    const e = new Date(end)
    const cur = new Date(s)
    while (cur <= e) {
      const ds = cur.toISOString().slice(0, 10)
      const arr = eventByDate.get(ds) ?? []
      arr.push(ev)
      eventByDate.set(ds, arr)
      cur.setDate(cur.getDate() + 1)
    }
  }

  // Sprint 19 T4(사용자 요청 4번) — 수업 가능일 판정. calendar-image.ts의
  // hasClassOnDate 와 동일 규칙(운영요일 + 이벤트 allows_regular_class/allows_makeup_class).
  function hasClassOnDate(date: string): boolean {
    const dow = isoDayOfWeek(date)
    const dh = operatingHours.find((h) => h.day_of_week === dow)
    const operating = dh !== undefined && dh.open_time !== null && dh.close_time !== null
    if (!operating) return false
    const cellEvents = eventByDate.get(date) ?? []
    if (cellEvents.length === 0) return true
    return cellEvents.some((e) => e.allows_regular_class || e.allows_makeup_class)
  }
  // 외곽선 영역 — "첫 평일 수업일 ~ 마지막 평일 수업일" (calendar-image.ts 동일 트림 규칙).
  let regionStart: string | null = null
  let regionEnd: string | null = null
  for (let d = period.start_date; d <= period.end_date; d = nextIsoDate(d)) {
    if (hasClassOnDate(d) && isWeekday(d)) {
      regionStart = d
      break
    }
  }
  for (let d = period.end_date; regionStart !== null && d >= period.start_date; d = prevIsoDate(d)) {
    if (hasClassOnDate(d) && isWeekday(d)) {
      regionEnd = d
      break
    }
  }
  function isTeaching(date: string): boolean {
    return (
      regionStart !== null && regionEnd !== null && date >= regionStart && date <= regionEnd && isWeekday(date)
    )
  }

  // 밴드(기간성 학사일정) — 각 이벤트의 중간 날짜에만 라벨 표시.
  const bandEvents = events.filter(isBandEvent)
  const bandLabelDate = new Map<number, string>()
  for (const e of bandEvents) {
    bandLabelDate.set(e.id, midIsoDate(e.event_date, e.period_end_date as string))
  }

  const title = `${sm}월 교습일정 (${period.start_date.slice(5).replace('-', '.')}~${period.end_date.slice(5).replace('-', '.')})`

  return (
    <div className="academic-print-root">
      <h2 className="print-title">{title}</h2>
      {months.map(({ year, month }) => {
        const grid = buildCalendarGrid(year, month)
        const rowCount = grid.length / 7

        /**
         * (row, col) 인접 셀이 같은 수업가능 영역에 속하는지 — 외곽선을 그릴 변(邊) 판정용.
         * row/col 좌표로 범위를 판단해 flat index 산술(±1, ±7)이 열 경계에서 옆 행으로
         * 잘못 넘어가는(wrap-around) 실수를 방지한다. 그리드는 이 달(month)에서만 유효 —
         * 교습기간이 두 달에 걸치면 각 월 테이블 가장자리에서 외곽선이 독립적으로 닫힌다
         * (다음 달 테이블과 시각적으로 이어지지 않음 — 별도 표이므로 의도된 동작).
         */
        function neighborIsTeaching(row: number, col: number): boolean {
          if (row < 0 || row >= rowCount || col < 0 || col > 6) return false
          const n = grid[row * 7 + col]
          if (n.outsideMonth) return false
          return isTeaching(dateStr(year, month, n.day))
        }

        return (
          <div key={`${year}-${month}`} className="print-month">
            <h3 className="print-month-heading">{year}년 {month}월</h3>
            <table
              className="print-cal-table"
              style={{ '--print-rows': rowCount + 1 } as React.CSSProperties}
            >
              <thead>
                <tr>
                  {DOW_LABELS.map((d, i) => (
                    <th
                      key={d}
                      className={`print-dow ${i === 0 ? 'print-sun' : i === 6 ? 'print-sat' : ''}`}
                    >
                      {d}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {Array.from({ length: rowCount }, (_, row) => (
                  <tr key={row}>
                    {grid.slice(row * 7, row * 7 + 7).map((cell, col) => {
                      const ds = cell.outsideMonth ? '' : dateStr(year, month, cell.day)
                      const inPeriod =
                        !cell.outsideMonth && ds >= period.start_date && ds <= period.end_date
                      const dayEvents = (ds ? (eventByDate.get(ds) ?? []) : []).filter(
                        (e) => !isBandEvent(e),
                      )
                      const cellBands = ds
                        ? bandEvents.filter(
                            (e) => ds >= e.event_date && ds <= (e.period_end_date as string),
                          )
                        : []

                      const teaching = !cell.outsideMonth && ds !== '' && isTeaching(ds)
                      const outlineClasses = teaching
                        ? [
                            !neighborIsTeaching(row - 1, col) ? 'print-teach-t' : '',
                            !neighborIsTeaching(row, col + 1) ? 'print-teach-r' : '',
                            !neighborIsTeaching(row + 1, col) ? 'print-teach-b' : '',
                            !neighborIsTeaching(row, col - 1) ? 'print-teach-l' : '',
                          ]
                            .filter(Boolean)
                            .join(' ')
                        : ''

                      return (
                        <td
                          key={col}
                          className={`print-cell ${cell.outsideMonth ? 'print-outside' : ''} ${inPeriod ? 'print-in-period' : ''} ${outlineClasses}`}
                        >
                          {cellBands.map((band) => (
                            <div key={band.id} className="print-band-fill">
                              {bandLabelDate.get(band.id) === ds && (
                                <span className="print-band-label">
                                  {band.display_name ?? band.code_name}
                                </span>
                              )}
                            </div>
                          ))}
                          <span
                            className={`print-day-num ${cell.dow === 0 ? 'print-sun' : cell.dow === 6 ? 'print-sat' : ''}`}
                          >
                            {cell.day}
                          </span>
                          {dayEvents.map((ev, i) => {
                            const color = codeColor(ev.code_name, ev.is_system_reserved).hex
                            return (
                              <span key={i} className="print-event-label" style={{ color }}>
                                {ev.display_name ?? ev.code_name}
                              </span>
                            )
                          })}
                        </td>
                      )
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )
      })}
    </div>
  )
}
