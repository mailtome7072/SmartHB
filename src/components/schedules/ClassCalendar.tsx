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

import { useMemo, useRef, useState } from 'react'
import FullCalendar from '@fullcalendar/react'
import dayGridPlugin from '@fullcalendar/daygrid'
import timeGridPlugin from '@fullcalendar/timegrid'
import koLocale from '@fullcalendar/core/locales/ko'
import type { DatesSetArg, EventInput } from '@fullcalendar/core'
import type { CalendarMonth } from '@/types/calendar'
import type { ScheduleEventListItem } from '@/types/academic'

interface Props {
  data: CalendarMonth
  academicEvents: ScheduleEventListItem[]
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

const VIEWS: Array<[string, string]> = [
  ['dayGridMonth', '월'],
  ['timeGridWeek', '주'],
  ['timeGridDay', '일'],
]

export default function ClassCalendar({
  data,
  academicEvents,
  onMonthChange,
  onStudentNameClick,
}: Props) {
  const calendarRef = useRef<FullCalendar>(null)
  const dateInputRef = useRef<HTMLInputElement>(null)
  const [viewType, setViewType] = useState('dayGridMonth')
  const [title, setTitle] = useState('')

  const isTimeGrid = viewType.startsWith('timeGrid')

  const holidayDates = useMemo(
    () => new Set(academicEvents.filter((e) => e.code_name === '공휴일').map((e) => e.event_date)),
    [academicEvents],
  )

  // 학사일정: 일자별 코드 명칭 텍스트(색).
  const academicByDate = useMemo(() => {
    const map = new Map<string, Array<{ name: string; color: string }>>()
    for (const e of academicEvents) {
      const color = e.is_system_reserved
        ? (EVENT_TEXT_COLOR[e.code_name] ?? USER_EVENT_TEXT_COLOR)
        : USER_EVENT_TEXT_COLOR
      const arr = map.get(e.event_date) ?? []
      arr.push({ name: e.display_name ?? e.code_name, color })
      map.set(e.event_date, arr)
    }
    return map
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

  // 수업 이벤트 — 주/일(timeGrid) 보기에서만 시간대별 블록(같은 시작시간 학생 묶음).
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
      for (const [startTime, { names, maxMin }] of bySlot) {
        result.push({
          start: `${day.eventDate}T${toIsoTime(startTime)}`,
          end: `${day.eventDate}T${addMinutes(startTime, maxMin)}`,
          backgroundColor: '#dbeafe',
          borderColor: '#3b82f6',
          textColor: '#1e3a8a',
          editable: false,
          extendedProps: { names },
        })
      }
    }
    return result
  }, [data, isTimeGrid])

  function api() {
    return calendarRef.current?.getApi()
  }

  return (
    <div className="flex h-full flex-col">
      {/* 커스텀 툴바 — ◀ 년월 ▶ … 월·주·일 (오늘 버튼 없음) */}
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <button
            type="button"
            aria-label="이전"
            onClick={() => api()?.prev()}
            className="flex h-10 w-10 items-center justify-center rounded-md bg-slate-700 text-lg text-white hover:bg-slate-800"
          >
            ‹
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
            className="flex h-10 w-10 items-center justify-center rounded-md bg-slate-700 text-lg text-white hover:bg-slate-800"
          >
            ›
          </button>
        </div>
        <div className="flex gap-1">
          {VIEWS.map(([v, label]) => (
            <button
              key={v}
              type="button"
              onClick={() => api()?.changeView(v)}
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
          // 셀 배경(수업 유무) + 주말/공휴일 날짜색.
          dayCellClassNames={(arg) => {
            const ds = dateStr(arg.date)
            const dow = arg.date.getDay()
            const cls: string[] = []
            if (holidayDates.has(ds) || dow === 0) cls.push('shb-sun')
            else if (dow === 6) cls.push('shb-sat')
            cls.push(dayInfo.has(ds) ? 'shb-has-class' : 'shb-no-class')
            return cls
          }}
          // 월 보기 셀: 좌측 상단 학사일정 텍스트 / 우측 날짜 + 그 아래 인원수.
          dayCellContent={(arg) => {
            const ds = dateStr(arg.date)
            const acts = academicByDate.get(ds) ?? []
            const info = dayInfo.get(ds)
            return (
              <div className="flex w-full justify-between gap-1">
                <div className="flex min-w-0 flex-col items-start gap-0.5">
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
                <div className="flex flex-col items-end leading-tight">
                  <span>{arg.dayNumberText}</span>
                  {info !== undefined && (
                    <span
                      title={info.tooltip}
                      className="cursor-pointer text-base font-bold text-blue-700"
                    >
                      {info.count}명
                    </span>
                  )}
                </div>
              </div>
            )
          }}
          // 주/일 보기 수업 블록: 원생 이름 줄바꿈 + 클릭 시 출결관리 이동.
          eventContent={(arg) => {
            const names = (arg.event.extendedProps.names as string[]) ?? []
            return (
              <div className="whitespace-normal break-words px-1 py-0.5 text-xs leading-snug">
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
