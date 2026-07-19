/**
 * 교습일정 인쇄 — 독립 팝업창용 HTML 문서 생성 (Sprint 19 후속수정).
 *
 * 기존 방식(createPortal + @media print + 같은 창에서 window.print())은 App Router의
 * body 직속 자식 숨김 트릭에 의존해 레이스 컨디션·CSS 캐스케이드 간섭에 취약했다.
 * 팝업창에 완전히 독립된 문서(별도 <style>, 별도 DOM)를 작성해 이런 간섭 요인을
 * 원천 차단한다 — 팝업 자신의 인라인 스크립트가 로드 완료 후 스스로 print()를 호출한다.
 *
 * 캘린더 판정 로직(수업일/외곽선/밴드)은 `calendar-image.ts`(공지문)와 동일 규칙을 포팅했다.
 */

import { codeColor } from '@/lib/schedule-code-colors'
import { isoDayOfWeek, isWeekday, nextIsoDate, prevIsoDate } from '@/lib/time'
import type { ScheduleEventListItem, StudyPeriod } from '@/types/academic'
import type { DayHours } from '@/lib/tauri'

interface BuildParams {
  period: StudyPeriod
  events: ScheduleEventListItem[]
  operatingHours: DayHours[]
}

const DOW_LABELS = ['일', '월', '화', '수', '목', '금', '토']

interface GridCell {
  day: number
  outsideMonth: boolean
  dow: number
}

function buildCalendarGrid(year: number, month: number): GridCell[] {
  const firstDow = new Date(year, month - 1, 1).getDay()
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

function midIsoDate(start: string, end: string): string {
  const s = new Date(`${start}T00:00:00Z`).getTime()
  const e = new Date(`${end}T00:00:00Z`).getTime()
  return new Date((s + e) / 2).toISOString().slice(0, 10)
}

function isBandEvent(e: ScheduleEventListItem): boolean {
  return e.is_period_type && e.period_end_date !== null && e.period_end_date !== e.event_date
}

/** 사용자 입력 텍스트(코드명/표시명)를 HTML로 안전하게 이스케이프. */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

const STYLE = `
  /* 인쇄 시 배경색이 기본적으로 생략되는 Chromium 동작(인쇄 대화상자 "배경 그래픽"
     옵션이 꺼져 있으면 배경색/이미지가 통째로 빠짐) 을 강제로 우회 — 이 옵션 때문에
     기간 밴드(단원평가 응시일 등) 배경색이 사용자 설정과 무관하게 항상 인쇄되도록 한다. */
  * {
    box-sizing: border-box;
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
    color-adjust: exact;
  }
  html, body { height: 100%; }
  body {
    margin: 0;
    font-family: Pretendard, -apple-system, sans-serif;
    background: #e5e7eb;
  }
  .print-root { width: 100%; }
  /* 한 페이지 = A4 가로 1장. 페이지 안에서 1~2개월 표가 flex 로 세로 공간을 나눠 갖는다
     (행 수만큼 축소). 교습기간이 3개월 이상 걸치면 아래 페이지 분할(page-break)로 여러 장에
     나눠, 각 월 달력이 항상 읽을 수 있는 크기를 유지한다(2개월/페이지). */
  .print-page {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100vh;
    padding: 8mm;
  }
  .print-title { flex: 0 0 auto; font-size: 20pt; font-weight: bold; text-align: center; margin-bottom: 8pt; }
  .print-month {
    flex: 1 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: #fff;
    margin-bottom: 8pt;
  }
  .print-month:last-child { margin-bottom: 0; }
  .print-month-heading { flex: 0 0 auto; font-size: 15pt; font-weight: 600; margin-bottom: 4pt; }
  .print-cal-table {
    flex: 1 1 auto;
    width: 100%;
    height: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    --print-rows: 7;
  }
  .print-cal-table tr { height: calc(100% / var(--print-rows)); }
  .print-cal-table th, .print-cal-table td {
    border: 0.5pt solid #aaa;
    padding: 3pt 5pt;
    vertical-align: top;
    position: relative;
    overflow: hidden;
  }
  .print-dow { text-align: center; font-size: 13pt; font-weight: 600; background: #f0f0f0; }
  .print-outside { color: #bbb; background: #fafafa; }
  .print-in-period { background: #fff8e1; }
  /* 기간성 학사일정 밴드 — 셀 위에 패널을 덧대면 셀 사이 테두리에 틈이 보여, 대신 해당
     기간에 속한 셀들의 배경색 자체를 칠해 인접 셀 사이 이질감 없이 이어져 보이게 한다. */
  .print-band { background: rgba(173, 214, 240, 0.45); }
  .print-day-num { display: block; position: relative; z-index: 1; font-size: 15pt; font-weight: 700; margin-bottom: 2pt; }
  .print-sun { color: #dc2626; }
  .print-sat { color: #2563eb; }
  .print-event-label {
    display: block; position: relative; z-index: 1; font-size: 13pt; font-weight: 700;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .print-teach-t { border-top: 2.5pt solid #E03131 !important; }
  .print-teach-r { border-right: 2.5pt solid #E03131 !important; }
  .print-teach-b { border-bottom: 2.5pt solid #E03131 !important; }
  .print-teach-l { border-left: 2.5pt solid #E03131 !important; }
  .print-band-label {
    display: flex; position: absolute; inset: 0; z-index: 0;
    align-items: center; justify-content: center;
    font-size: 13pt; font-weight: 700; color: #1e3a8a;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  @media print {
    body { background: #fff; }
    @page { size: A4 landscape; margin: 8mm; }
    .print-month { page-break-inside: avoid; }
    /* 3개월 이상 걸침 → 페이지당 2개월씩 분할 인쇄. 마지막 페이지 뒤 빈 장 방지. */
    .print-page:not(:last-child) { page-break-after: always; }
  }
`

/** 교습일정 인쇄 팝업창에 쓸 완결된 HTML 문서를 생성한다. */
export function buildAcademicPrintHtml({ period, events, operatingHours }: BuildParams): string {
  const [sy, sm] = period.start_date.split('-').map(Number)
  const [ey, em] = period.end_date.split('-').map(Number)

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

  const eventByDate = new Map<string, ScheduleEventListItem[]>()
  for (const ev of events) {
    const end = ev.period_end_date ?? ev.event_date
    let cur = ev.event_date
    while (cur <= end) {
      const arr = eventByDate.get(cur) ?? []
      arr.push(ev)
      eventByDate.set(cur, arr)
      cur = nextIsoDate(cur)
    }
  }

  function hasClassOnDate(date: string): boolean {
    const dow = isoDayOfWeek(date)
    const dh = operatingHours.find((h) => h.day_of_week === dow)
    const operating = dh !== undefined && dh.open_time !== null && dh.close_time !== null
    if (!operating) return false
    const cellEvents = eventByDate.get(date) ?? []
    if (cellEvents.length === 0) return true
    return cellEvents.some((e) => e.allows_regular_class || e.allows_makeup_class)
  }

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

  const bandEvents = events.filter(isBandEvent)
  const bandLabelDate = new Map<number, string>()
  for (const e of bandEvents) {
    bandLabelDate.set(e.id, midIsoDate(e.event_date, e.period_end_date as string))
  }

  const title = `${sm}월 교습일정 (${period.start_date.slice(5).replace('-', '.')}~${period.end_date.slice(5).replace('-', '.')})`

  const monthHtmlList = months
    .map(({ year, month }) => {
      const grid = buildCalendarGrid(year, month)
      const rowCount = grid.length / 7

      function neighborIsTeaching(row: number, col: number): boolean {
        if (row < 0 || row >= rowCount || col < 0 || col > 6) return false
        const n = grid[row * 7 + col]
        if (n.outsideMonth) return false
        return isTeaching(dateStr(year, month, n.day))
      }

      const rowsHtml = Array.from({ length: rowCount }, (_, row) => {
        const cellsHtml = grid
          .slice(row * 7, row * 7 + 7)
          .map((cell, col) => {
            const ds = cell.outsideMonth ? '' : dateStr(year, month, cell.day)
            const inPeriod = !cell.outsideMonth && ds >= period.start_date && ds <= period.end_date
            const dayEvents = (ds ? (eventByDate.get(ds) ?? []) : []).filter((e) => !isBandEvent(e))
            const cellBands = ds
              ? bandEvents.filter((e) => ds >= e.event_date && ds <= (e.period_end_date as string))
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

            const dowClass = cell.dow === 0 ? 'print-sun' : cell.dow === 6 ? 'print-sat' : ''
            const bandsHtml = cellBands
              .filter((band) => bandLabelDate.get(band.id) === ds)
              .map((band) => `<span class="print-band-label">${escapeHtml(band.display_name ?? band.code_name)}</span>`)
              .join('')
            const eventsHtml = dayEvents
              .map((ev) => {
                const color = codeColor(ev.code_name, ev.is_system_reserved).hex
                const label = escapeHtml(ev.display_name ?? ev.code_name)
                return `<span class="print-event-label" style="color:${color}">${label}</span>`
              })
              .join('')

            const bandClass = cellBands.length > 0 ? 'print-band' : ''
            return `<td class="print-cell ${cell.outsideMonth ? 'print-outside' : ''} ${inPeriod ? 'print-in-period' : ''} ${bandClass} ${outlineClasses}">${bandsHtml}<span class="print-day-num ${dowClass}">${cell.day}</span>${eventsHtml}</td>`
          })
          .join('')
        return `<tr>${cellsHtml}</tr>`
      }).join('')

      const headerHtml = DOW_LABELS.map(
        (d, i) => `<th class="print-dow ${i === 0 ? 'print-sun' : i === 6 ? 'print-sat' : ''}">${d}</th>`,
      ).join('')

      return `
        <div class="print-month">
          <h3 class="print-month-heading">${year}년 ${month}월</h3>
          <table class="print-cal-table" style="--print-rows:${rowCount + 1}">
            <thead><tr>${headerHtml}</tr></thead>
            <tbody>${rowsHtml}</tbody>
          </table>
        </div>
      `
    })

  // 페이지당 최대 2개월씩 묶어 페이지로 분할한다(1~2개월은 자연히 1페이지 → 기존 레이아웃 유지).
  // 3개월 이상 걸치면 여러 페이지로 나뉘어 각 월 달력이 읽을 수 있는 크기를 유지한다.
  const MONTHS_PER_PAGE = 2
  const pagesHtml: string[] = []
  for (let i = 0; i < monthHtmlList.length; i += MONTHS_PER_PAGE) {
    const pageMonths = monthHtmlList.slice(i, i + MONTHS_PER_PAGE).join('')
    pagesHtml.push(
      `<div class="print-page"><h2 class="print-title">${escapeHtml(title)}</h2>${pageMonths}</div>`,
    )
  }

  return `<!doctype html>
<html lang="ko">
<head>
<meta charset="utf-8" />
<title>${escapeHtml(title)}</title>
<style>${STYLE}</style>
</head>
<body>
  <div class="print-root">
    ${pagesHtml.join('')}
  </div>
  <script>
    // 인쇄/취소와 무관하게 대화상자가 닫히면(afterprint) 이 창은 용도를 다했으므로 스스로 닫는다.
    // 일반 window.close()는 window.open()으로 연 창에만 허용되는 브라우저 정책 때문에 이 창
    // (Tauri WebviewWindow로 생성)에는 통하지 않아, tauri.conf.json의 withGlobalTauri로 노출된
    // window.__TAURI__ 를 통해 Tauri 창 자체를 직접 닫는다.
    function closeThisWindow() {
      try {
        if (window.__TAURI__ && window.__TAURI__.window) {
          window.__TAURI__.window.getCurrentWindow().close()
          return
        }
      } catch (e) {}
      window.close()
    }
    window.addEventListener('afterprint', closeThisWindow)
    window.addEventListener('load', function () {
      if (document.fonts && document.fonts.ready) {
        document.fonts.ready.then(function () { window.print() })
      } else {
        window.print()
      }
    })
  </script>
</body>
</html>`
}
