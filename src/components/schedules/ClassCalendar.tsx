'use client'

/**
 * 수업 관리 캘린더 (FullCalendar 래퍼) — Sprint 10 T11 + 1·2·3차 시각 검증 반영 (PRD §4.6.1, ADR-006).
 *
 * 3차 검증 반영:
 * - 오늘 버튼 제거 / 커스텀 툴바(◀ 년월 ▶ … 월·주·일) — 년월 클릭 시 그 자리에 날짜 선택기
 * - 셀 배경: 수업 있는 날 amber-100 / 없는 날 gray-100 (학사 캘린더와 통일)
 * - 학사일정 코드 텍스트는 셀 좌측 상단, 인원수는 날짜 아래(우측)
 * - 주/일 보기 수업시간에 원생 이름 표시 (시작시간 초 포함 "HH:MM:SS" 정규화 버그 수정)
 *
 * static export(R67): 페이지에서 `dynamic(..., { ssr: false })` 로 로드.
 */

import { useEffect, useMemo, useRef, useState } from 'react'
import FullCalendar from '@fullcalendar/react'
import dayGridPlugin from '@fullcalendar/daygrid'
import timeGridPlugin from '@fullcalendar/timegrid'
import koLocale from '@fullcalendar/core/locales/ko'
import type { DatesSetArg, EventInput } from '@fullcalendar/core'
import { codeColor } from '@/lib/schedule-code-colors'
import type { CalendarMonth } from '@/types/calendar'
import type { ScheduleEventListItem, StudyPeriod } from '@/types/academic'

interface Props {
  data: CalendarMonth
  academicEvents: ScheduleEventListItem[]
  /** 교습기간 목록 — 셀 배경(교습기간 내 amber) 판정용. */
  studyPeriods: StudyPeriod[]
  onMonthChange: (yearMonth: string) => void
  onStudentNameClick: (studentName: string) => void
}

/** "HH:MM[:SS]" → "HH:MM:00" (초 포함 입력도 안전하게 정규화). 비정상/빈 값은 "00:00:00". */
function toIsoTime(t: string | null | undefined): string {
  const [h = '', m = ''] = (t ?? '').split(':')
  return `${(h || '00').padStart(2, '0')}:${(m || '00').padStart(2, '0')}:00`
}

/** "HH:MM[:SS]" + 분 → "HH:MM:00". */
function addMinutes(startTime: string | null | undefined, addMin: number): string {
  const [h, m] = (startTime ?? '').split(':').map(Number)
  const total = (h || 0) * 60 + (m || 0) + addMin
  return `${String(Math.floor(total / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}:00`
}

/** ISO local datetime("YYYY-MM-DDTHH:mm:ss") + 분 → 동일 형식 문자열. 겹침 재배치용. */
function shiftIso(iso: string, minutes: number): string {
  const d = new Date(iso)
  d.setMinutes(d.getMinutes() + minutes)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

/** "HH:MM[:SS]" → "오전/오후 N시[ M분]". 비시각 라벨은 그대로. */
function formatKoreanTime(slot: string): string {
  const m = /^(\d{1,2}):(\d{2})/.exec(slot)
  if (!m) return slot
  const hour = Number(m[1])
  const min = Number(m[2])
  const period = hour < 12 ? '오전' : '오후'
  const h12 = hour % 12 === 0 ? 12 : hour % 12
  return `${period} ${h12}시${min > 0 ? ` ${min}분` : ''}`
}

function ymFromDatesSet(arg: DatesSetArg): string {
  const mid = new Date((arg.start.getTime() + arg.end.getTime()) / 2)
  return `${mid.getFullYear()}-${String(mid.getMonth() + 1).padStart(2, '0')}`
}

function dateStr(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

/** start ~ end (둘 다 "YYYY-MM-DD") 일자 배열 (양 끝 포함). end null/동일이면 [start]. */
function expandDates(start: string, end: string | null): string[] {
  if (!end || end === start) return [start]
  const out: string[] = []
  const cur = new Date(start)
  const last = new Date(end)
  while (cur <= last) {
    out.push(dateStr(cur))
    cur.setDate(cur.getDate() + 1)
  }
  return out
}

const VIEWS: Array<[string, string]> = [
  ['dayGridMonth', '월'],
  ['timeGridWeek', '주'],
  ['timeGridDay', '일'],
]

/** 수업 시간(분) 기준 4색 팔레트 — 1h/2h/3h/4h를 색상으로 시각적 구분. */
const DURATION_COLORS: Record<number, { bg: string; border: string; text: string }> = {
  60:  { bg: '#dbeafe', border: '#3b82f6', text: '#1e3a8a' }, // 1h — blue
  120: { bg: '#dcfce7', border: '#22c55e', text: '#14532d' }, // 2h — green
  180: { bg: '#ede9fe', border: '#7c3aed', text: '#4c1d95' }, // 3h — violet (교습일 셀배경 amber와 명확히 구분)
  240: { bg: '#fee2e2', border: '#ef4444', text: '#7f1d1d' }, // 4h — red
}

/** 수업 시간(분) → 색상. 미등록 시간은 blue 기본. */
function colorForDuration(classMinutes: number): { bg: string; border: string; text: string } {
  return DURATION_COLORS[classMinutes] ?? DURATION_COLORS[60]
}

/**
 * 하루 기준 겹치는 시간대 열 배정 — Google Calendar 식 greedy interval packing.
 * 같은 열은 서로 시간이 겹치지 않도록 보장 → 한 원생의 여러 시간대 조각(다중 슬롯 칩)이
 * 항상 동일한 열에 배치되어 주/일 보기에서 세로로 이어지는 시각적 일관성을 유지한다.
 * overlapTotal: 이 항목과 직접 겹치는 항목들 중 사용된 최대 열 수(+1) — 2 초과 시 2열×N행 재배치 필요.
 */
function assignColumns(items: { startMs: number; endMs: number }[]): Array<{
  column: number
  overlapTotal: number
}> {
  const order = items.map((_, i) => i).sort((a, b) => items[a].startMs - items[b].startMs || a - b)
  const columnEndMs: number[] = []
  const column: number[] = new Array(items.length).fill(0)
  for (const i of order) {
    const it = items[i]
    let col = 0
    while (columnEndMs[col] !== undefined && columnEndMs[col] > it.startMs) col++
    column[i] = col
    columnEndMs[col] = it.endMs
  }
  return items.map((it, i) => {
    let maxCol = column[i]
    for (let j = 0; j < items.length; j++) {
      if (j === i) continue
      const o = items[j]
      if (it.startMs < o.endMs && o.startMs < it.endMs) maxCol = Math.max(maxCol, column[j])
    }
    return { column: column[i], overlapTotal: maxCol + 1 }
  })
}

export default function ClassCalendar({
  data,
  academicEvents,
  studyPeriods,
  onMonthChange,
  onStudentNameClick,
}: Props) {
  const calendarRef = useRef<FullCalendar>(null)
  const dateInputRef = useRef<HTMLInputElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [viewType, setViewType] = useState('timeGridWeek')
  const [title, setTitle] = useState('')

  const isTimeGrid = viewType.startsWith('timeGrid')

  const holidayDates = useMemo(
    () => new Set(academicEvents.filter((e) => e.code_name === '공휴일').map((e) => e.event_date)),
    [academicEvents],
  )

  // 학사일정: 일자별 코드 명칭 텍스트(색) + 정규/보강 가능 플래그.
  // 기간성 코드(단원평가 응시일 등)는 event_date~period_end_date 전 일자에 확장 표기.
  const { academicByDate, academicFlags } = useMemo(() => {
    const byDate = new Map<string, Array<{ name: string; color: string }>>()
    const flags = new Map<string, { hasMakeupOn: boolean; hasRegularOff: boolean }>()
    for (const e of academicEvents) {
      // P2-13: 색은 schedule-code-colors.ts SSOT (학사 캘린더·공지문 달력과 일치).
      const color = codeColor(e.code_name, e.is_system_reserved).hex
      const name = e.display_name ?? e.code_name
      const dates = expandDates(e.event_date, e.period_end_date)
      for (const d of dates) {
        const arr = byDate.get(d) ?? []
        arr.push({ name, color })
        byDate.set(d, arr)
        const f = flags.get(d) ?? { hasMakeupOn: false, hasRegularOff: false }
        if (e.allows_makeup_class) f.hasMakeupOn = true
        if (!e.allows_regular_class) f.hasRegularOff = true
        flags.set(d, f)
      }
    }
    return { academicByDate: byDate, academicFlags: flags }
  }, [academicEvents])

  // 월 보기: 일자별 원생 목록 + 툴팁.
  const dayInfo = useMemo(() => {
    const map = new Map<
      string,
      {
        count: number
        tooltip: string
        students: { name: string; classMinutes: number; isMakeup: boolean }[]
      }
    >()
    for (const day of data.days) {
      const ids = new Set<number>()
      const bySlot = new Map<string, string[]>()
      const students: { name: string; classMinutes: number; isMakeup: boolean }[] = []
      for (const s of day.regularSessions) {
        ids.add(s.studentId)
        const key = s.startTime || '시간미정'
        bySlot.set(key, [...(bySlot.get(key) ?? []), s.studentName])
        students.push({ name: s.studentName, classMinutes: s.classMinutes, isMakeup: false })
      }
      for (const s of day.makeupSessions) {
        ids.add(s.studentId)
        bySlot.set('보강', [...(bySlot.get('보강') ?? []), s.studentName])
        students.push({ name: s.studentName, classMinutes: s.classMinutes ?? 60, isMakeup: true })
      }
      const tooltip = [...bySlot.entries()]
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([slot, names]) => `${formatKoreanTime(slot)}: ${names.join(', ')}`)
        .join('\n')
      if (ids.size > 0) map.set(day.eventDate, { count: ids.size, tooltip, students })
    }
    return map
  }, [data])

  // 원생 칩 hover 시 그 원생 수업 시간 범위(시작~종료)를 시간 그리드에 테두리(검정)로 강조.
  const [hovered, setHovered] = useState<{
    date: string
    startTime: string
    classMinutes: number
  } | null>(null)

  // 주/일 보기 공통: 원생별 1시간 슬롯 이벤트 — 2h+ 수업은 슬롯마다 칩 생성(동일 색상, 이슈 4).
  //   하루 단위로 겹치는 시간대의 열을 미리 배정(assignColumns)해 다중 슬롯 원생이 항상
  //   동일한 열에 표시되도록 보장한다. 겹침 2명까지는 FullCalendar 자동 균등 폭 배분(이슈 3)에
  //   맡기고, 3명 이상이면 30분 단위로 나눠 2열×N행으로 재배치한다.
  const events = useMemo<EventInput[]>(() => {
    if (!isTimeGrid) return []
    // 일 보기는 하루 전체 폭을 쓸 수 있어 2열×N행 재배치가 불필요 — 겹쳐도 한 행에 모두 표시.
    const isDay = viewType === 'timeGridDay'
    const result: EventInput[] = []
    for (const day of data.days) {
      // 시작시간 미상(null/빈값/형식이상)은 시간 슬롯 배치 불가 → 주/일 뷰 생략(월 뷰 '시간미정').
      const valid = day.regularSessions.filter((s) => s.startTime && s.startTime.includes(':'))
      if (valid.length === 0) continue
      const items = valid.map((s) => {
        const startMs = new Date(`${day.eventDate}T${toIsoTime(s.startTime)}`).getTime()
        return { s, startMs, endMs: startMs + s.classMinutes * 60000 }
      })
      const layout = assignColumns(items)
      // 열 배정 순서 고정(열 → 원생ID) — FullCalendar가 매 시간대마다 항상 동일한 좌우 순서로 렌더링.
      const ordered = items
        .map((it, i) => ({ ...it, ...layout[i] }))
        .sort((a, b) => a.column - b.column || a.s.studentId - b.s.studentId)
      for (const { s, column, overlapTotal } of ordered) {
        const c = colorForDuration(s.classMinutes)
        const totalSlots = Math.max(1, Math.ceil(s.classMinutes / 60))
        const needSplit = !isDay && overlapTotal > 2
        const rowGroup = needSplit ? Math.floor(column / 2) : 0
        for (let h = 0; h < totalSlots; h++) {
          const slotStart = addMinutes(s.startTime!, h * 60)
          const slotEnd = addMinutes(s.startTime!, (h + 1) * 60)
          let startIso = `${day.eventDate}T${toIsoTime(slotStart)}`
          let endIso = `${day.eventDate}T${toIsoTime(slotEnd)}`
          if (needSplit) {
            endIso = shiftIso(startIso, rowGroup * 30 + 30)
            startIso = shiftIso(startIso, rowGroup * 30)
          }
          result.push({
            start: startIso,
            end: endIso,
            backgroundColor: c.bg,
            borderColor: c.border,
            textColor: c.text,
            editable: false,
            extendedProps: {
              kind: 'class',
              studentId: s.studentId,
              studentName: s.studentName,
              classMinutes: s.classMinutes,
              slotIndex: h,
              totalSlots,
              classStartTime: s.startTime!,
            },
          })
        }
      }
    }
    return result
  }, [data, isTimeGrid, viewType])

  // hover 강조용 background 이벤트를 합쳐서 전달.
  const allEvents = useMemo<EventInput[]>(() => {
    if (hovered === null) return events
    return [
      ...events,
      {
        start: `${hovered.date}T${toIsoTime(hovered.startTime)}`,
        end: `${hovered.date}T${addMinutes(hovered.startTime, hovered.classMinutes)}`,
        display: 'background',
      },
    ]
  }, [events, hovered])

  function api() {
    return calendarRef.current?.getApi()
  }

  // 월 보기 인원수 배지(absolute) 를 day-frame 에 주입한다.
  // dayCellDidMount 의 1회성 한계를 우회 — dayInfo / viewType 이 바뀔 때마다 모든 day-frame
  // 을 다시 훑어 배지를 새로 그린다. 비월 보기 진입 시 잔존 배지는 자동 청소된다.
  useEffect(() => {
    const root = containerRef.current
    if (!root) return
    const cells = root.querySelectorAll<HTMLElement>('.fc-daygrid-day')
    cells.forEach((cell: HTMLElement) => {
      const frame = cell.querySelector('.fc-daygrid-day-frame') as HTMLElement | null
      if (!frame) return
      const existing = frame.querySelector('.shb-count-badge')
      if (existing) existing.remove()
      if (viewType !== 'dayGridMonth') return
      const ds = cell.getAttribute('data-date')
      if (ds === null) return
      const info = dayInfo.get(ds)
      if (info === undefined) return
      frame.style.position = 'relative'
      // 원생 이름 칩 컨테이너 — 2열 그리드로 이름 표기 (이슈 5/T7)
      const container = document.createElement('div')
      container.className = 'shb-count-badge'
      container.style.cssText =
        'display:grid;grid-template-columns:1fr 1fr 1fr;gap:2px;padding:2px 4px 4px;' +
        'width:100%;box-sizing:border-box;z-index:5;pointer-events:auto;'
      for (const st of info.students) {
        const chip = document.createElement('span')
        // 주/일 보기 칩과 동일한 색상 기준(수업 시간별 배경+글자색, colorForDuration SSOT).
        const color = colorForDuration(st.classMinutes)
        const label = st.isMakeup ? `${st.name}(보강)` : st.name
        chip.textContent = label
        chip.title = `${label} ${st.classMinutes / 60}시간`
        chip.style.cssText =
          `display:block;font-size:12px;font-weight:600;color:${color.text};background-color:${color.bg};` +
          'white-space:nowrap;overflow:hidden;text-overflow:ellipsis;cursor:pointer;' +
          'border-radius:3px;padding:0 2px;'
        container.appendChild(chip)
      }
      frame.appendChild(container)
    })
  }, [dayInfo, viewType])

  // 뷰 전환 — 주/일은 오늘 날짜가 포함되도록 이동, 월은 현재 위치 유지.
  // Sprint 11 F5: setViewType 을 클릭 시점에 명시적으로 호출하여 한 프레임 동안의
  // 버튼 highlight / events memo 불일치를 제거. datesSet 콜백의 setViewType 은 동일 값이라 no-op.
  function changeView(v: string) {
    setViewType(v)
    const a = api()
    if (!a) return
    if (v === 'dayGridMonth') a.changeView(v)
    else a.changeView(v, dateStr(new Date()))
  }

  return (
    <div ref={containerRef} className="flex h-full flex-col">
      {/* 커스텀 툴바 — [중앙] ◀ 년월 ▶ / [우] 월·주·일 (오늘 버튼 없음) */}
      <div className="mb-2 grid grid-cols-3 items-center gap-2">
        <div />
        <div className="flex items-center justify-center gap-2">
          <button
            type="button"
            aria-label="이전"
            onClick={() => api()?.prev()}
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-sm leading-none hover:bg-gray-50"
          >
            ◀
          </button>
          <div className="relative inline-flex items-center justify-center">
            <span className="px-2 text-[18px] font-bold text-[var(--foreground)]">{title}</span>
            <input
              ref={dateInputRef}
              type="date"
              aria-label="날짜로 이동"
              className="absolute inset-0 cursor-pointer opacity-0"
              onChange={(e) => {
                const v = e.target.value
                if (v) api()?.gotoDate(v)
                e.target.value = ''
              }}
            />
          </div>
          <button
            type="button"
            aria-label="다음"
            onClick={() => api()?.next()}
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-sm leading-none hover:bg-gray-50"
          >
            ▶
          </button>
        </div>
        <div className="flex justify-end gap-1">
          {VIEWS.map(([v, label]) => (
            <button
              key={v}
              type="button"
              onClick={() => changeView(v)}
              aria-pressed={viewType === v}
              className={[
                'min-h-[40px] min-w-[3.5rem] rounded-md px-6 text-base font-semibold',
                viewType === v
                  ? 'bg-[var(--accent)] text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300',
              ].join(' ')}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      <div className="min-h-0 flex-1">
        <FullCalendar
          ref={calendarRef}
          plugins={[dayGridPlugin, timeGridPlugin]}
          initialView="timeGridWeek"
          initialDate={`${data.yearMonth}-01`}
          locale={koLocale}
          firstDay={0}
          headerToolbar={false}
          events={allEvents}
          // hover 강조 background 이벤트는 채움(음영) 대신 테두리만 표시한다.
          eventDidMount={(arg) => {
            if (arg.event.display === 'background') {
              arg.el.style.backgroundColor = 'transparent'
              // 월 보기 셀 hover 테두리와 동일한 스타일 (outline 2px #334155, offset -2px).
              arg.el.style.border = 'none'
              arg.el.style.outline = '2px solid #334155'
              arg.el.style.outlineOffset = '-2px'
              arg.el.style.borderRadius = '0'
              arg.el.style.boxSizing = 'border-box'
            }
          }}
          height="100%"
          expandRows
          // 일 보기 개별 블록을 겹치지 않고 나란히 배치(같은 시간대 여러 원생).
          slotEventOverlap={false}
          slotDuration="01:00:00"
          slotLabelInterval="01:00:00"
          slotLabelContent={(arg) => {
            const h = arg.date.getHours()
            const m = arg.date.getMinutes()
            const meridiem = h < 12 ? 'am.' : 'pm.'
            const h12 = h % 12 === 0 ? 12 : h % 12
            return `${h12}${m > 0 ? `:${String(m).padStart(2, '0')}` : ''}${meridiem}`
          }}
          slotMinTime="14:00:00"
          slotMaxTime="23:00:00"
          allDaySlot={false}
          nowIndicator
          dayMaxEvents={4}
          datesSet={(arg) => {
            setViewType(arg.view.type)
            setTitle(arg.view.title)
            onMonthChange(ymFromDatesSet(arg))
          }}
          // 셀 배경 — 수업 가능 여부 정밀 판정 (PRD §4.4 동일):
          //   교습기간 밖              → gray
          //   보강 가능 코드(allows_makeup_class=true, 예: 보강데이) → amber  ← 최우선
          //   정규 OFF 코드(allows_regular_class=false, 공휴일/대체공휴일/휴원일/방학 등) → gray
          //   토요일·일요일 (보강데이 없음)                              → gray
          //   그 외 평일                                                  → amber
          // 별도로 주말·공휴일 날짜 숫자 색만 적용 (배경과 독립).
          // 일(day) 보기는 사용자 지정으로 배경색 없음.
          dayCellClassNames={(arg) => {
            const ds = dateStr(arg.date)
            const dow = arg.date.getDay()
            const cls: string[] = []
            if (holidayDates.has(ds) || dow === 0) cls.push('shb-sun')
            else if (dow === 6) cls.push('shb-sat')
            if (arg.view.type === 'timeGridDay') return cls
            const inPeriod = studyPeriods.some((p) => ds >= p.start_date && ds <= p.end_date)
            const f = academicFlags.get(ds)
            let amber: boolean
            if (!inPeriod) amber = false
            else if (f?.hasMakeupOn) amber = true
            else if (f?.hasRegularOff) amber = false
            else if (dow === 0 || dow === 6) amber = false
            else amber = true
            cls.push(amber ? 'shb-has-class' : 'shb-no-class')
            return cls
          }}
          // 주/일 보기 날짜 헤더 — 날짜 / 학사일정 코드(중앙) / 총 N명 수업.
          dayHeaderContent={(arg) => {
            // 월 보기: 요일(일~토)만 표기. 주말은 색 구분.
            if (arg.view.type === 'dayGridMonth') {
              const dow = arg.date.getDay()
              const color = dow === 0 ? '#dc2626' : dow === 6 ? '#2563eb' : 'inherit'
              const label = ['일', '월', '화', '수', '목', '금', '토'][dow]
              return (
                <span className="text-sm font-semibold" style={{ color }}>
                  {label}
                </span>
              )
            }
            if (!arg.view.type.startsWith('timeGrid')) return undefined
            const ds = dateStr(arg.date)
            const acts = academicByDate.get(ds) ?? []
            const info = dayInfo.get(ds)
            return (
              <div className="flex flex-col items-center gap-0.5 py-1">
                <span className="text-sm font-semibold">{arg.text}</span>
                {acts.map((a, i) => (
                  <span
                    key={`${a.name}-${i}`}
                    className="text-sm font-semibold"
                    style={{ color: a.color }}
                  >
                    {a.name}
                  </span>
                ))}
                {info !== undefined && (
                  <span className="text-sm text-gray-700">총 {info.count}명 수업</span>
                )}
              </div>
            )
          }}
          // 월 보기 전용 셀 상단: 좌 학사코드 / 우 "N일" 날짜.
          dayCellContent={(arg) => {
            if (arg.view.type !== 'dayGridMonth') return undefined
            const ds = dateStr(arg.date)
            const acts = academicByDate.get(ds) ?? []
            return (
              <div className="flex w-full items-start">
                <div className="flex min-w-0 flex-1 flex-col items-start gap-0.5 pr-4 pt-1">
                  {acts.map((a, i) => (
                    <span
                      key={i}
                      className="max-w-full truncate text-sm font-semibold"
                      style={{ color: a.color }}
                    >
                      {a.name}
                    </span>
                  ))}
                </div>
                <span>{arg.date.getDate()}일</span>
              </div>
            )
          }}
          // 월 보기 인원수 배지는 dayCellDidMount 가 아니라 아래 useEffect 에서 주입.
          // 이유: dayCellDidMount 는 셀 DOM 마운트 시점 1회만 호출되어 클로저로 캡쳐한 dayInfo 가
          // 그 시점에 빈 상태(데이터 로딩 중)이면 배지가 빠진다. 데이터가 늦게 도착해 dayInfo 가
          // 갱신돼도 셀은 unmount 되지 않으므로 훅이 재발화 안 됨 — 주/일 보기 갔다 돌아올 때만
          // 셀이 재마운트되며 badge 가 늦게 나타나는 증상의 원인이었다.
          // 주/일 수업 블록: 원생 이름 + 수업시간 표기. 클릭 시 출결관리 이동.
          // 주 보기: 1시간 슬롯마다 칩. 2h+ 수업은 슬롯별로 연속 표시(→/←).
          // 일 보기: 전체 기간 블록, 폰트 확대.
          eventContent={(arg) => {
            // hover 강조용 background 이벤트는 extendedProps가 없어 콘텐츠 렌더링 대상이 아님.
            if (arg.event.display === 'background') return null
            const { studentName, classMinutes, slotIndex, totalSlots, classStartTime } =
              arg.event.extendedProps as {
                studentName: string
                classMinutes: number
                slotIndex: number
                totalSlots: number
                classStartTime: string
              }
            const isDay = viewType === 'timeGridDay'
            const hoursLabel = (min: number): string => {
              const h = min / 60
              return Number.isInteger(h) ? `${h}시간` : `${h.toFixed(1)}시간`
            }
            const isFirst = slotIndex === 0
            const isLast = slotIndex === totalSlots - 1
            const multiSlot = totalSlots > 1
            return (
              <div
                role="button"
                tabIndex={0}
                className={`flex h-full cursor-pointer items-center overflow-hidden px-1 py-0.5 font-semibold hover:underline ${isDay ? 'text-base' : 'text-sm'}`}
                onClick={(ev) => {
                  ev.stopPropagation()
                  onStudentNameClick(studentName)
                }}
                onMouseEnter={() =>
                  setHovered({
                    date: arg.event.startStr.slice(0, 10),
                    startTime: classStartTime,
                    classMinutes,
                  })
                }
                onMouseLeave={() => setHovered(null)}
                title={`${studentName} ${hoursLabel(classMinutes)}`}
              >
                <span className="truncate">{studentName}</span>
                {multiSlot && !isLast && (
                  <span className="ml-0.5 shrink-0 text-xs opacity-60">↓</span>
                )}
                {multiSlot && !isFirst && (
                  <span className="ml-0.5 shrink-0 text-xs opacity-60">↑</span>
                )}
                {isFirst && (
                  <span className="ml-0.5 shrink-0 text-xs font-normal opacity-70">
                    {hoursLabel(classMinutes)}
                  </span>
                )}
              </div>
            )
          }}
        />
      </div>
    </div>
  )
}
