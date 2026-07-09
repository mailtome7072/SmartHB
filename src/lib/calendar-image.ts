/**
 * 공지문용 교습일정 달력 이미지 렌더러 — Sprint 16 Session #5 (PRD §4.10 확장).
 *
 * 청구년월의 한 달 학사일정을 **달력 형식 PNG**(dataURL)로 캔버스에 직접 그린다.
 * 공지문 캔버스에 이미지 요소(`NoticeImageKind = 'calendar'`)로 합성된다.
 *
 * 시각 요소 (예시 이미지 기준):
 * - 일요일 시작 6×7 그리드 (공지문 = 대중 공개용 → 일요일 시작; 앱 학사캘린더는 월요일 시작과 별개)
 * - 교습기간 **빨간 외곽선**: `교습기간 ∩ 수업일` 셀 영역의 경계 (휴원/공휴일 제외, 공휴수업일 포함)
 * - 특이일 라벨(공휴일·보강데이 등) + 기간 하이라이트(단원평가 주간 등 period_end_date 보유 코드)
 * - 전월/익월 날짜 회색, 일=빨강·토=파랑 요일색
 *
 * 데이터는 기존 IPC 그대로 사용 (신규 IPC/마이그레이션 없음):
 *   getStudyPeriod / listScheduleEvents / getOperatingHours
 *
 * 수업일 판정(`hasClassOnDate`)은 `ThreeMonthCalendar` 와 동일 규칙을 포팅한다.
 */

import { codeColor } from '@/lib/schedule-code-colors'
import { isoDayOfWeek, isWeekday, nextIsoDate, prevIsoDate } from '@/lib/time'
import type { ScheduleEventListItem, StudyPeriod } from '@/types/academic'
import type { DayHours } from '@/lib/tauri'

export interface CalendarImageParams {
  /** 'YYYY-MM' — 그릴 달. */
  yearMonth: string
  /** 해당 월 교습기간 (없으면 빨간 외곽선 미표시). */
  studyPeriod: StudyPeriod | null
  /** 그리드 범위(전월~익월)를 덮는 학사일정 목록. */
  events: ScheduleEventListItem[]
  /** 운영 시간 — 셀별 수업일 판정에 사용. */
  operatingHours: DayHours[]
}

// ─── 날짜 유틸 (라이브러리 무의존, ThreeMonthCalendar 패턴) ──────────────────

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

function ymd(year: number, month: number, day: number): string {
  return `${year}-${pad2(month)}-${pad2(day)}`
}

function daysInMonth(year: number, month: number): number {
  return new Date(year, month, 0).getDate()
}

function shiftMonth(year: number, month: number, delta: number): { year: number; month: number } {
  const d = new Date(year, month - 1 + delta, 1)
  return { year: d.getFullYear(), month: d.getMonth() + 1 }
}

interface GridCell {
  date: string // "YYYY-MM-DD"
  dayOfMonth: number
  isOutsideMonth: boolean
  isSunday: boolean
  isSaturday: boolean
  row: number
  col: number // 0=일 ~ 6=토
}

/** 일요일 시작 6×7=42 셀 그리드. 그리드 밖 일자도 채워 outside flag 부여. */
function buildSundayGrid(year: number, month: number): GridCell[] {
  const cells: GridCell[] = []
  const lead = new Date(year, month - 1, 1).getDay() // 0=일 → 일요일 시작 leading 칸 수
  const days = daysInMonth(year, month)
  const prev = shiftMonth(year, month, -1)
  const prevDays = daysInMonth(prev.year, prev.month)
  const next = shiftMonth(year, month, 1)

  const push = (y: number, m: number, day: number, outside: boolean) => {
    const idx = cells.length
    const jsDay = new Date(y, m - 1, day).getDay()
    cells.push({
      date: ymd(y, m, day),
      dayOfMonth: day,
      isOutsideMonth: outside,
      isSunday: jsDay === 0,
      isSaturday: jsDay === 6,
      row: Math.floor(idx / 7),
      col: idx % 7,
    })
  }

  for (let i = 0; i < lead; i++) push(prev.year, prev.month, prevDays - lead + 1 + i, true)
  for (let day = 1; day <= days; day++) push(year, month, day, false)
  let trailingDay = 1
  while (cells.length < 42) push(next.year, next.month, trailingDay++, true)
  return cells
}

/**
 * event_date → events 매핑 (기간성 코드는 시작~종료 전체 일자에 매핑).
 * ThreeMonthCalendar 의 eventsByDate 로직과 동일 — 수업일 판정·라벨 표시 공용.
 */
function buildEventsByDate(events: ScheduleEventListItem[]): Map<string, ScheduleEventListItem[]> {
  const map = new Map<string, ScheduleEventListItem[]>()
  const add = (date: string, e: ScheduleEventListItem) => {
    const arr = map.get(date) ?? []
    arr.push(e)
    map.set(date, arr)
  }
  for (const e of events) {
    add(e.event_date, e)
    if (e.is_period_type && e.period_end_date && e.period_end_date !== e.event_date) {
      let cursor = nextIsoDate(e.event_date)
      while (cursor <= e.period_end_date) {
        add(cursor, e)
        cursor = nextIsoDate(cursor)
      }
    }
  }
  return map
}

// ─── 색상 (캔버스 직접 드로잉 — Tailwind 클래스 대신 hex) ─────────────────────

const RED = '#E03131' // 일요일/공휴일 날짜, 교습기간 외곽선
const BLUE = '#1971C2' // 토요일 날짜
const DAY_DEFAULT = '#1A1A1A'
const OUTSIDE_GRAY = '#ADB5BD'
const GRID_LINE = '#000000' // 셀 기본 테두리(검정)
const PERIOD_FILL = 'rgba(173, 214, 240, 0.40)' // 기간 하이라이트(연한 파랑)
const PERIOD_LABEL = '#1864AB'

/** 코드명 → 라벨 텍스트 색 (캔버스). P2-13: schedule-code-colors.ts SSOT 사용 — 앱 화면과 동일 색.
 *  (보강데이=teal·공휴수업일=pink 로 통일, 기존 공지문 전용 crimson/orange 팔레트 폐기). */
function labelColor(e: ScheduleEventListItem): string {
  return codeColor(e.code_name, e.is_system_reserved).hex
}

// ─── 렌더 치수 (고해상도 — 공지문에서 축소 배치돼도 선명) ─────────────────────

const CELL_W = 200
const CELL_H = 132
const HEADER_H = 64
const PAD = 2 // 캔버스 가장자리 여백(외곽선 잘림 방지)
const WEEKDAY_LABELS = ['일', '월', '화', '수', '목', '금', '토']

// 강조 라벨(단원평가 주간 밴드·보강데이) 공통 — 둘의 top 위치를 정확히 맞추기 위해 상수 공유.
const BAND_LABEL_FONT_PX = 30
const BAND_LABEL_CENTER_Y = CELL_H / 2 + 14 // 단원평가 주간 밴드 라벨 세로 중앙(middle baseline)
const EMPHASIS_LABEL_TOP_Y = BAND_LABEL_CENTER_Y - BAND_LABEL_FONT_PX / 2 // 보강데이 top 정렬용(top baseline)

/**
 * 한 달 교습일정을 달력 PNG dataURL 로 렌더한다 (저장 없음).
 * 클라이언트(앱 화면)에서만 호출 가능.
 */
export async function renderCalendarImageDataUrl(params: CalendarImageParams): Promise<string> {
  if (typeof document === 'undefined') {
    throw new Error('달력 이미지 생성은 앱 화면(클라이언트)에서만 가능합니다.')
  }
  if (typeof document !== 'undefined' && document.fonts?.ready) {
    try {
      await document.fonts.ready
    } catch {
      /* 폰트 상태 조회 실패는 무시 (fallback 폰트로 렌더) */
    }
  }

  const [y, m] = params.yearMonth.split('-').map(Number)
  if (!y || !m) throw new Error('청구년월 형식이 올바르지 않습니다.')

  const cells = buildSundayGrid(y, m)
  const eventsByDate = buildEventsByDate(params.events)
  const sp = params.studyPeriod
  const opHours = params.operatingHours

  // 수업일 판정 — 운영요일 + 이벤트(정규/보강 허용 여부) 종합 (ThreeMonthCalendar 동일 규칙).
  const hasClassOnDate = (date: string): boolean => {
    const dow = isoDayOfWeek(date)
    const dh = opHours.find((h) => h.day_of_week === dow)
    const operating = dh !== undefined && dh.open_time !== null && dh.close_time !== null
    if (!operating) return false
    const cellEvents = eventsByDate.get(date) ?? []
    if (cellEvents.length === 0) return true
    return cellEvents.some((e) => e.allows_regular_class || e.allows_makeup_class)
  }

  // ── 빨간 외곽선 영역 산출 (사용자 확정 규칙) ──
  // - 경계(교습기간 시작/종료일)가 수업불가일(공휴일 등)이면 영역에서 제외(안쪽으로 트림)
  // - 시작~종료 사이(interior)의 평일 수업불가일(공휴일 등)은 영역에 포함(구멍 없음)
  // - **토·일요일은 항상 제외** (수업일 여부와 무관)
  // → "첫 평일 수업일 ~ 마지막 평일 수업일" 구간을 감싸되, 주말 열은 비워둔다.
  let regionStart: string | null = null
  let regionEnd: string | null = null
  if (sp !== null) {
    for (let d = sp.start_date; d <= sp.end_date; d = nextIsoDate(d)) {
      if (hasClassOnDate(d) && isWeekday(d)) {
        regionStart = d
        break
      }
    }
    for (let d = sp.end_date; regionStart !== null && d >= sp.start_date; d = prevIsoDate(d)) {
      if (hasClassOnDate(d) && isWeekday(d)) {
        regionEnd = d
        break
      }
    }
  }
  const isTeaching = (date: string): boolean =>
    regionStart !== null &&
    regionEnd !== null &&
    date >= regionStart &&
    date <= regionEnd &&
    isWeekday(date)
  const teachingByIdx = cells.map((c) => isTeaching(c.date))

  const gridW = 7 * CELL_W
  const gridH = HEADER_H + 6 * CELL_H
  const canvas = document.createElement('canvas')
  canvas.width = gridW + PAD * 2
  canvas.height = gridH + PAD * 2
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('캔버스를 초기화할 수 없습니다.')
  ctx.translate(PAD, PAD)
  ctx.textBaseline = 'alphabetic'

  // 배경 흰색
  ctx.fillStyle = '#FFFFFF'
  ctx.fillRect(0, 0, gridW, gridH)

  const cellX = (col: number) => col * CELL_W
  const cellY = (row: number) => HEADER_H + row * CELL_H

  // ── 1. 요일 헤더 ──
  ctx.fillStyle = '#F1F3F5'
  ctx.fillRect(0, 0, gridW, HEADER_H)
  ctx.font = '700 30px Pretendard, sans-serif'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  for (let col = 0; col < 7; col++) {
    ctx.fillStyle = col === 0 ? RED : col === 6 ? BLUE : DAY_DEFAULT
    ctx.fillText(WEEKDAY_LABELS[col], cellX(col) + CELL_W / 2, HEADER_H / 2)
  }

  // ── 2. 기간 하이라이트(연한 파랑) — period_end_date 보유 코드 ──
  // 셀 단위로 칠한 뒤, 이벤트별·행별 중앙에 라벨 1회.
  const bandEvents = params.events.filter(
    (e) => e.is_period_type && e.period_end_date && e.period_end_date !== e.event_date,
  )
  const inBand = (e: ScheduleEventListItem, date: string): boolean =>
    !!e.period_end_date && date >= e.event_date && date <= e.period_end_date
  ctx.fillStyle = PERIOD_FILL
  for (const c of cells) {
    if (bandEvents.some((e) => inBand(e, c.date))) {
      ctx.fillRect(cellX(c.col), cellY(c.row), CELL_W, CELL_H)
    }
  }

  // ── 3. 셀 그리드선 + 전월/익월 회색 처리 ──
  ctx.strokeStyle = GRID_LINE
  ctx.lineWidth = 1
  for (const c of cells) {
    const x = cellX(c.col)
    const yy = cellY(c.row)
    ctx.strokeRect(x + 0.5, yy + 0.5, CELL_W, CELL_H)
  }

  // ── 4. 날짜 숫자 ──
  ctx.textAlign = 'left'
  ctx.textBaseline = 'top'
  ctx.font = '700 32px Pretendard, sans-serif'
  for (const c of cells) {
    const hasHoliday = (eventsByDate.get(c.date) ?? []).some((e) => e.code_name === '공휴일')
    let color: string
    if (c.isOutsideMonth) color = OUTSIDE_GRAY
    else if (hasHoliday || c.isSunday) color = RED
    else if (c.isSaturday) color = BLUE
    else color = DAY_DEFAULT
    ctx.fillStyle = color
    // 익월 첫 주는 'M/D' 로 월 구분 (예: 7/1) — 그 외는 일자만.
    const label =
      c.isOutsideMonth && c.dayOfMonth <= 7 && c.row >= 4
        ? `${shiftMonth(y, m, 1).month}/${c.dayOfMonth}`
        : String(c.dayOfMonth)
    ctx.fillText(label, cellX(c.col) + 10, cellY(c.row) + 8)
  }

  // ── 5. 특이일 라벨(단일 일자 이벤트) — 기간성 밴드 라벨은 별도(아래) ──
  // 보강데이는 강조: 볼드 + 150% 크기(20→30px) + top 위치를 단원평가 주간 밴드 라벨과 동일하게.
  const LABEL_SIZE = 20
  ctx.textBaseline = 'top'
  for (const c of cells) {
    if (c.isOutsideMonth) continue
    const dayEvents = (eventsByDate.get(c.date) ?? []).filter(
      (e) => !(e.is_period_type && e.period_end_date && e.period_end_date !== e.event_date),
    )
    let ly = cellY(c.row) + 48
    for (const e of dayEvents.slice(0, 2)) {
      const emphasize = e.code_name === '보강데이'
      const size = emphasize ? BAND_LABEL_FONT_PX : LABEL_SIZE
      ctx.font = `${emphasize ? '700' : '600'} ${size}px Pretendard, sans-serif`
      ctx.fillStyle = labelColor(e)
      const text = e.display_name ?? e.code_name
      // 보강데이는 단원평가 주간 밴드 라벨과 top 일치, 그 외는 날짜 아래로 순차 배치.
      const top = emphasize ? cellY(c.row) + EMPHASIS_LABEL_TOP_Y : ly
      // 셀 폭 초과 시 말줄임
      ctx.fillText(truncateToWidth(ctx, text, CELL_W - 16), cellX(c.col) + 10, top)
      if (!emphasize) ly += size + 6 // 폰트 크기에 맞춘 줄 간격
    }
  }

  // ── 6. 기간 하이라이트 라벨 (행별 중앙) ──
  ctx.font = `700 ${BAND_LABEL_FONT_PX}px Pretendard, sans-serif`
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillStyle = PERIOD_LABEL
  for (const e of bandEvents) {
    const label = e.display_name ?? e.code_name
    // 행별로 이 이벤트가 덮는 연속 칸 범위를 찾아 중앙에 라벨 1회.
    for (let row = 0; row < 6; row++) {
      const rowCells = cells.filter((c) => c.row === row && inBand(e, c.date))
      if (rowCells.length === 0) continue
      const cols = rowCells.map((c) => c.col)
      const c0 = Math.min(...cols)
      const c1 = Math.max(...cols)
      const cx = (cellX(c0) + cellX(c1) + CELL_W) / 2
      const cy = cellY(row) + BAND_LABEL_CENTER_Y
      ctx.fillText(label, cx, cy)
    }
  }

  // ── 7. 교습기간 빨간 외곽선 — teaching 영역 경계 (그리드 이웃 기준) ──
  drawTeachingOutline(ctx, cells, teachingByIdx, cellX, cellY)

  return canvas.toDataURL('image/png')
}

/** 텍스트가 maxWidth 초과 시 '…' 말줄임. */
function truncateToWidth(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
  if (ctx.measureText(text).width <= maxWidth) return text
  let lo = 0
  let hi = text.length
  while (lo < hi) {
    const mid = Math.ceil((lo + hi) / 2)
    if (ctx.measureText(text.slice(0, mid) + '…').width <= maxWidth) lo = mid
    else hi = mid - 1
  }
  return text.slice(0, lo) + '…'
}

/**
 * 교습기간 영역(첫 수업일~마지막 수업일 연속 구간)의 외곽선을 빨간 선으로 그린다.
 * 각 영역 셀에서 그리드 이웃(상/하/좌/우)이 영역 밖이거나 그리드 밖이면 그 변에 선.
 * 영역은 연속 구간이므로 중간 비수업일(공휴일·일요일)도 포함되어 구멍 없는 윤곽이 된다.
 */
function drawTeachingOutline(
  ctx: CanvasRenderingContext2D,
  cells: GridCell[],
  teaching: boolean[],
  cellX: (col: number) => number,
  cellY: (row: number) => number,
): void {
  // (row,col) → teaching 조회용 맵.
  const at = (row: number, col: number): boolean => {
    if (row < 0 || row > 5 || col < 0 || col > 6) return false
    const idx = row * 7 + col
    return teaching[idx] ?? false
  }
  ctx.strokeStyle = RED
  ctx.lineWidth = 5
  ctx.lineCap = 'square'
  ctx.beginPath()
  for (const c of cells) {
    const idx = c.row * 7 + c.col
    if (!teaching[idx]) continue
    const x = cellX(c.col)
    const yy = cellY(c.row)
    // 상
    if (!at(c.row - 1, c.col)) {
      ctx.moveTo(x, yy)
      ctx.lineTo(x + CELL_W, yy)
    }
    // 하
    if (!at(c.row + 1, c.col)) {
      ctx.moveTo(x, yy + CELL_H)
      ctx.lineTo(x + CELL_W, yy + CELL_H)
    }
    // 좌
    if (!at(c.row, c.col - 1)) {
      ctx.moveTo(x, yy)
      ctx.lineTo(x, yy + CELL_H)
    }
    // 우
    if (!at(c.row, c.col + 1)) {
      ctx.moveTo(x + CELL_W, yy)
      ctx.lineTo(x + CELL_W, yy + CELL_H)
    }
  }
  ctx.stroke()
}
