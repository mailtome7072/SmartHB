'use client'

/**
 * 교습일정 인쇄용 컴포넌트 — T9/이슈10 (Sprint 18).
 *
 * window.print() 호출 시 @media print 스타일로 A4 세로 레이아웃으로 출력된다.
 * 요일 순서: 일월화수목금토 (T5에서 변경된 일요일 시작).
 */

import type { StudyPeriod, ScheduleEventListItem } from '@/types/academic'
import { codeColor } from '@/lib/schedule-code-colors'

interface Props {
  period: StudyPeriod
  events: ScheduleEventListItem[]
}

const DOW_LABELS = ['일', '월', '화', '수', '목', '금', '토']

function buildCalendarGrid(
  year: number,
  month: number,
): { day: number; outsideMonth: boolean; dow: number }[] {
  const firstDow = new Date(year, month - 1, 1).getDay() // 0=일
  const daysInMonth = new Date(year, month, 0).getDate()
  const prevDays = new Date(year, month - 1, 0).getDate()
  const cells: { day: number; outsideMonth: boolean; dow: number }[] = []

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

export function AcademicSchedulePrint({ period, events }: Props) {
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

  const title = `${sm}월 교습일정 (${period.start_date.slice(5).replace('-', '.')}~${period.end_date.slice(5).replace('-', '.')})`

  return (
    <div className="academic-print-root">
      <h2 className="print-title">{title}</h2>
      {months.map(({ year, month }) => {
        const grid = buildCalendarGrid(year, month)
        return (
          <div key={`${year}-${month}`} className="print-month">
            <h3 className="print-month-heading">{year}년 {month}월</h3>
            <table className="print-cal-table">
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
                {Array.from({ length: grid.length / 7 }, (_, row) => (
                  <tr key={row}>
                    {grid.slice(row * 7, row * 7 + 7).map((cell, col) => {
                      const ds = cell.outsideMonth
                        ? ''
                        : dateStr(year, month, cell.day)
                      const inPeriod =
                        !cell.outsideMonth && ds >= period.start_date && ds <= period.end_date
                      const dayEvents = ds ? (eventByDate.get(ds) ?? []) : []
                      return (
                        <td
                          key={col}
                          className={`print-cell ${cell.outsideMonth ? 'print-outside' : ''} ${inPeriod ? 'print-in-period' : ''}`}
                        >
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
