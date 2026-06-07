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

// 월 보기 인원수 배지 hover 시 시간대별 명단 팝업.
// native `title` 속성 툴팁은 브라우저가 그려 폰트 크기를 키울 수 없으므로, document.body 에
// 커스텀 div 를 띄워 가독성 있는 큰 폰트(24px ≈ native 의 2배)로 표시한다. 단일 인스턴스만 유지.
const CLASS_TOOLTIP_ID = 'shb-class-tooltip'

function hideClassTooltip() {
  document.getElementById(CLASS_TOOLTIP_ID)?.remove()
}

function showClassTooltip(text: string, anchor: HTMLElement) {
  hideClassTooltip()
  if (text === '') return
  const tip = document.createElement('div')
  tip.id = CLASS_TOOLTIP_ID
  tip.textContent = text
  tip.style.cssText =
    'position:fixed;z-index:9999;pointer-events:none;background:#111;color:#fff;' +
    'padding:10px 14px;border-radius:8px;font-size:24px;line-height:1.5;' +
    'white-space:pre-line;max-width:520px;box-shadow:0 4px 12px rgba(0,0,0,0.3);'
  document.body.appendChild(tip)
  // 배지 중앙 위쪽에 배치하되 화면 경계를 넘지 않도록 보정. 위 공간이 부족하면 아래로.
  const a = anchor.getBoundingClientRect()
  const t = tip.getBoundingClientRect()
  const left = Math.max(8, Math.min(a.left + a.width / 2 - t.width / 2, window.innerWidth - t.width - 8))
  const top = a.top - t.height - 8
  tip.style.left = `${left}px`
  tip.style.top = `${top < 8 ? a.bottom + 8 : top}px`
}

/** 학사일정 코드명 → 텍스트 색 (academic CalendarCell 팔레트). */
const EVENT_TEXT_COLOR: Record<string, string> = {
  공휴일: '#dc2626',
  보강데이: '#0d9488',
  공휴수업일: '#db2777',
  방학: '#9333ea',
  휴원일: '#6b7280',
  '단원평가 응시일': '#2563eb',
}
const USER_EVENT_TEXT_COLOR = '#d97706'

/** "HH:MM[:SS]" → "HH:MM:00" (초 포함 입력도 안전하게 정규화). */
function toIsoTime(t: string): string {
  const [h, m] = t.split(':')
  return `${h.padStart(2, '0')}:${m.padStart(2, '0')}:00`
}

/** "HH:MM[:SS]" + 분 → "HH:MM:00". */
function addMinutes(startTime: string, addMin: number): string {
  const [h, m] = startTime.split(':').map(Number)
  const total = h * 60 + m + addMin
  return `${String(Math.floor(total / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}:00`
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
  const [viewType, setViewType] = useState('dayGridMonth')
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
      const color = e.is_system_reserved
        ? (EVENT_TEXT_COLOR[e.code_name] ?? USER_EVENT_TEXT_COLOR)
        : USER_EVENT_TEXT_COLOR
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

  // 월 보기: 일자별 인원수 + 시간대별 명단 툴팁.
  const dayInfo = useMemo(() => {
    const map = new Map<string, { count: number; tooltip: string }>()
    for (const day of data.days) {
      const ids = new Set<number>()
      const bySlot = new Map<string, string[]>()
      for (const s of day.regularSessions) {
        ids.add(s.studentId)
        const key = s.startTime ?? '시간미정'
        bySlot.set(key, [...(bySlot.get(key) ?? []), s.studentName])
      }
      for (const s of day.makeupSessions) {
        ids.add(s.studentId)
        bySlot.set('보강', [...(bySlot.get('보강') ?? []), s.studentName])
      }
      const tooltip = [...bySlot.entries()]
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([slot, names]) => `${formatKoreanTime(slot)}: ${names.join(', ')}`)
        .join('\n')
      if (ids.size > 0) map.set(day.eventDate, { count: ids.size, tooltip })
    }
    return map
  }, [data])

  // 주/일 보기 이벤트 — 시간대별 수업 블록만. 학사일정은 dayHeaderContent 안에 표기.
  const events = useMemo<EventInput[]>(() => {
    if (!isTimeGrid) return []
    const result: EventInput[] = []
    for (const day of data.days) {
      const bySlot = new Map<string, { names: string[]; maxMin: number }>()
      for (const s of day.regularSessions) {
        if (s.startTime === null) continue
        const cur = bySlot.get(s.startTime) ?? { names: [], maxMin: 0 }
        cur.names.push(s.studentName)
        cur.maxMin = Math.max(cur.maxMin, s.classMinutes)
        bySlot.set(s.startTime, cur)
      }
      const isDay = viewType === 'timeGridDay'
      for (const [startTime, { names, maxMin }] of bySlot) {
        result.push({
          start: `${day.eventDate}T${toIsoTime(startTime)}`,
          end: `${day.eventDate}T${addMinutes(startTime, maxMin)}`,
          // 일 보기는 배경/테두리 없음 (사용자 지정). 주 보기는 옅은 블루 유지.
          backgroundColor: isDay ? 'transparent' : '#dbeafe',
          borderColor: isDay ? 'transparent' : '#3b82f6',
          textColor: '#1e3a8a',
          editable: false,
          extendedProps: { kind: 'class', names },
        })
      }
    }
    return result
  }, [data, isTimeGrid, viewType])

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
      const badge = document.createElement('div')
      badge.className = 'shb-count-badge'
      badge.textContent = `${info.count}명`
      badge.style.cssText =
        'position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);' +
        'font-size:28px;font-weight:400;color:#111;cursor:pointer;' +
        'z-index:5;pointer-events:auto;white-space:nowrap;'
      // native `title` 대신 커스텀 큰 폰트 팝업 — 시간대별 명단을 가독성 있게 표시.
      const tip = info.tooltip
      badge.addEventListener('mouseenter', () => showClassTooltip(tip, badge))
      badge.addEventListener('mouseleave', hideClassTooltip)
      frame.appendChild(badge)
    })
    return () => hideClassTooltip()
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
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50"
          >
            ← 이전
          </button>
          <div className="relative inline-flex items-center justify-center">
            <span className="px-2 text-2xl font-bold text-[var(--foreground)]">{title}</span>
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
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50"
          >
            다음 →
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
          initialView="dayGridMonth"
          initialDate={`${data.yearMonth}-01`}
          locale={koLocale}
          firstDay={1}
          headerToolbar={false}
          events={events}
          height="100%"
          expandRows
          slotDuration="01:00:00"
          slotLabelInterval="01:00:00"
          slotLabelContent={(arg) => {
            const h = arg.date.getHours()
            const m = arg.date.getMinutes()
            const meridiem = h < 12 ? 'am.' : 'pm.'
            const h12 = h % 12 === 0 ? 12 : h % 12
            return `${h12}${m > 0 ? `:${String(m).padStart(2, '0')}` : ''}${meridiem}`
          }}
          slotMinTime="12:00:00"
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
                    className="text-xs font-semibold"
                    style={{ color: a.color }}
                  >
                    {a.name}
                  </span>
                ))}
                {info !== undefined && (
                  <span className="text-xs text-gray-700">총 {info.count}명 수업</span>
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
                      className="max-w-full truncate text-xs font-semibold"
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
          // 주/일 수업 블록: 원생 이름 줄바꿈 + 클릭 시 출결관리 이동.
          // 일 보기는 폰트 2단계 확대 + 파랑 볼드 (text-xs → text-base text-blue-700 font-bold).
          eventContent={(arg) => {
            const names = (arg.event.extendedProps.names as string[]) ?? []
            const cls =
              viewType === 'timeGridDay'
                ? 'text-base font-bold text-blue-700 text-center'
                : 'text-xs'
            return (
              <div className={`whitespace-normal break-words px-1 py-0.5 leading-snug ${cls}`}>
                {names.map((n, i) => (
                  <span key={`${n}-${i}`}>
                    <span
                      role="button"
                      tabIndex={0}
                      className="cursor-pointer hover:underline"
                      onClick={(ev) => {
                        ev.stopPropagation()
                        onStudentNameClick(n)
                      }}
                    >
                      {n}
                    </span>
                    {i < names.length - 1 ? ', ' : ''}
                  </span>
                ))}
              </div>
            )
          }}
        />
      </div>
    </div>
  )
}
