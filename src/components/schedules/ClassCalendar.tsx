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

/** "HH:MM[:SS]" → "HH:MM:00" (초 포함 입력도 안전하게 정규화). 비정상/빈 값은 "00:00:00". */
function toIsoTime(t: string | null | undefined): string {
  const [h = '', m = ''] = (t ?? '').split(':')
  return `${(h || '00').padStart(2, '0')}:${(m || '00').padStart(2, '0')}:00`
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

/** 원생별 수업 블록 색상 팔레트 — 같은 시간대 다른 원생을 시각적으로 구분 (주/일 뷰). */
const STUDENT_PALETTE: Array<{ bg: string; border: string; text: string }> = [
  { bg: '#dbeafe', border: '#3b82f6', text: '#1e3a8a' }, // blue
  { bg: '#dcfce7', border: '#22c55e', text: '#14532d' }, // green
  { bg: '#fef9c3', border: '#ca8a04', text: '#713f12' }, // yellow
  { bg: '#fee2e2', border: '#ef4444', text: '#7f1d1d' }, // red
  { bg: '#f3e8ff', border: '#a855f7', text: '#581c87' }, // purple
  { bg: '#ffedd5', border: '#f97316', text: '#7c2d12' }, // orange
  { bg: '#cffafe', border: '#06b6d4', text: '#164e63' }, // cyan
  { bg: '#fce7f3', border: '#db2777', text: '#831843' }, // pink
]

/** 원생 ID → 안정적 색상 매핑 (같은 원생은 항상 같은 색). */
function colorForStudent(studentId: number): { bg: string; border: string; text: string } {
  return STUDENT_PALETTE[studentId % STUDENT_PALETTE.length]
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
        // 이동된 출결 등 시작시간 미상(null/빈값)인 정규 수업은 '시간미정' 그룹으로.
        const key = s.startTime || '시간미정'
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

  // 원생 칩 hover 시 그 원생 수업 시간 범위(시작~종료)를 시간 그리드에 테두리(검정)로 강조.
  const [hovered, setHovered] = useState<{
    date: string
    startTime: string
    classMinutes: number
  } | null>(null)

  // 주/일 보기 이벤트 — 시간대별로 원생을 하나의 블록에 묶고, 내부를 2열 grid(2×N)로 표시.
  // (원생별 색칩으로 구분. 학사일정은 dayHeaderContent 안에 표기.)
  const events = useMemo<EventInput[]>(() => {
    if (!isTimeGrid) return []
    const result: EventInput[] = []
    for (const day of data.days) {
      const bySlot = new Map<
        string,
        {
          students: { studentId: number; studentName: string; classMinutes: number }[]
          maxMin: number
        }
      >()
      for (const s of day.regularSessions) {
        // 시작시간 미상(null/빈값/형식이상)인 정규 수업은 시간 슬롯에 배치 불가 → 주/일 뷰에서 생략.
        // (이동된 출결처럼 스케줄 없는 요일의 수업. 월 뷰에서는 '시간미정'으로 표시됨.)
        if (!s.startTime || !s.startTime.includes(':')) continue
        const cur = bySlot.get(s.startTime) ?? { students: [], maxMin: 0 }
        cur.students.push({
          studentId: s.studentId,
          studentName: s.studentName,
          classMinutes: s.classMinutes,
        })
        cur.maxMin = Math.max(cur.maxMin, s.classMinutes)
        bySlot.set(s.startTime, cur)
      }
      for (const [startTime, { students, maxMin }] of bySlot) {
        result.push({
          start: `${day.eventDate}T${toIsoTime(startTime)}`,
          end: `${day.eventDate}T${addMinutes(startTime, maxMin)}`,
          backgroundColor: 'transparent',
          borderColor: 'transparent',
          editable: false,
          extendedProps: { kind: 'class', students },
        })
      }
    }
    return result
  }, [data, isTimeGrid])

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
      const badge = document.createElement('div')
      badge.className = 'shb-count-badge'
      // title 은 전역 GlobalTooltip(AppShell)이 20px 커스텀 팝업으로 표시한다.
      badge.title = info.tooltip
      badge.textContent = `${info.count}명`
      badge.style.cssText =
        'position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);' +
        'font-size:28px;font-weight:400;color:#111;cursor:pointer;' +
        'z-index:5;pointer-events:auto;white-space:nowrap;'
      frame.appendChild(badge)
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
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-xl leading-none hover:bg-gray-50"
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
            className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-xl leading-none hover:bg-gray-50"
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
          initialView="dayGridMonth"
          initialDate={`${data.yearMonth}-01`}
          locale={koLocale}
          firstDay={1}
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
            const students =
              (arg.event.extendedProps.students as {
                studentId: number
                studentName: string
                classMinutes: number
              }[]) ?? []
            const isDay = viewType === 'timeGridDay'
            const hoursLabel = (min: number): string => {
              const h = min / 60
              return Number.isInteger(h) ? `${h}시간` : `${h.toFixed(1)}시간`
            }
            // 한 시간대 원생을 2열 grid(2×N)로 배치. 각 원생은 색칩 + 수업 시간으로 구분.
            return (
              <div
                className={`grid h-full grid-cols-2 content-start gap-0.5 overflow-hidden p-0.5 ${
                  isDay ? 'text-sm' : 'text-xs'
                }`}
              >
                {students.map((st, i) => {
                  const c = colorForStudent(st.studentId)
                  return (
                    <span
                      key={`${st.studentId}-${i}`}
                      role="button"
                      tabIndex={0}
                      onClick={(ev) => {
                        ev.stopPropagation()
                        onStudentNameClick(st.studentName)
                      }}
                      onMouseEnter={() =>
                        setHovered({
                          date: arg.event.startStr.slice(0, 10),
                          startTime: arg.event.startStr.slice(11, 16),
                          classMinutes: st.classMinutes,
                        })
                      }
                      onMouseLeave={() => setHovered(null)}
                      className="flex cursor-pointer items-center justify-center gap-0.5 truncate rounded px-1 py-0.5 text-center font-semibold hover:underline"
                      style={{ backgroundColor: c.bg, color: c.text, border: `1px solid ${c.border}` }}
                      title={`${st.studentName} ${hoursLabel(st.classMinutes)}`}
                    >
                      <span className="truncate">{st.studentName}</span>
                      <span className="shrink-0 font-normal opacity-80">
                        {hoursLabel(st.classMinutes)}
                      </span>
                    </span>
                  )
                })}
              </div>
            )
          }}
        />
      </div>
    </div>
  )
}
