'use client'

/**
 * 수업 관리 캘린더 (FullCalendar 래퍼) — Sprint 10 T11 + 1차 시각 검증 반영 (PRD §4.6.1, ADR-006).
 *
 * 시각 검증 반영:
 * - 주 시작 월요일 / 창 높이에 맞춘 높이 / 오늘 버튼·년월·뷰 버튼 재배치
 * - 토·일·공휴일 색 + 학사일정(단원평가·보강데이 등) 배지 표시
 * - 월 보기: 일자별 수업 인원수 + hover 시 시간대별 명단 툴팁(줄바꿈)
 * - 주/일 보기: 1시간 단위, 수업시간에 원생 이름 나열(줄바꿈) + 이름 클릭 → 출결관리 이동
 * - 년월 클릭 → 날짜 선택(date picker) → 해당 일자의 월/주/일 보기
 *
 * static export(R67): 페이지에서 `dynamic(..., { ssr: false })` 로 로드.
 */

import { useEffect, useMemo, useRef, useState } from 'react'
import FullCalendar from '@fullcalendar/react'
import dayGridPlugin from '@fullcalendar/daygrid'
import timeGridPlugin from '@fullcalendar/timegrid'
import interactionPlugin from '@fullcalendar/interaction'
import koLocale from '@fullcalendar/core/locales/ko'
import type { DatesSetArg, EventInput } from '@fullcalendar/core'
import type { CalendarMonth } from '@/types/calendar'
import type { ScheduleEventListItem } from '@/types/academic'

interface Props {
  data: CalendarMonth
  academicEvents: ScheduleEventListItem[]
  /** 보이는 기간이 다른 월로 바뀌면 호출 — 부모가 yearMonth state 갱신 → refetch. */
  onMonthChange: (yearMonth: string) => void
  /** 원생 이름 클릭(주/일 보기) → 출결관리 이동 + 필터. */
  onStudentNameClick: (studentName: string) => void
}

/** 학사일정 코드명 → 배지 색 (academic CalendarCell 과 동일 팔레트). */
const EVENT_COLORS: Record<string, { bg: string; border: string }> = {
  공휴일: { bg: '#fecaca', border: '#ef4444' },
  보강데이: { bg: '#99f6e4', border: '#14b8a6' },
  공휴수업일: { bg: '#fbcfe8', border: '#ec4899' },
  방학: { bg: '#e9d5ff', border: '#a855f7' },
  휴원일: { bg: '#e5e7eb', border: '#9ca3af' },
  '단원평가 응시일': { bg: '#bfdbfe', border: '#3b82f6' },
}
const USER_EVENT_COLOR = { bg: '#fde68a', border: '#f59e0b' }

function addMinutes(startTime: string, addMin: number): string {
  const [h, m] = startTime.split(':').map(Number)
  const total = h * 60 + m + addMin
  return `${String(Math.floor(total / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}:00`
}

/** "HH:MM[:SS]" → "오전/오후 N시[ M분]" (한국어, 시·분만). 비시각 라벨(보강 등)은 그대로. */
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

export default function ClassCalendar({
  data,
  academicEvents,
  onMonthChange,
  onStudentNameClick,
}: Props) {
  const calendarRef = useRef<FullCalendar>(null)
  const dateInputRef = useRef<HTMLInputElement>(null)
  const [viewType, setViewType] = useState('dayGridMonth')

  const isTimeGrid = viewType.startsWith('timeGrid')

  // 공휴일 일자 집합 — 주말/공휴일 셀 색상용.
  const holidayDates = useMemo(
    () => new Set(academicEvents.filter((e) => e.code_name === '공휴일').map((e) => e.event_date)),
    [academicEvents],
  )

  // 월 보기용: 일자별 인원수 + 시간대별 명단 툴팁.
  const dayInfo = useMemo(() => {
    const map = new Map<string, { count: number; tooltip: string }>()
    for (const day of data.days) {
      const ids = new Set<number>()
      const bySlot = new Map<string, string[]>()
      for (const s of day.regularSessions) {
        ids.add(s.studentId)
        const key = s.startTime ?? '시간미정'
        const arr = bySlot.get(key) ?? []
        arr.push(s.studentName)
        bySlot.set(key, arr)
      }
      for (const s of day.makeupSessions) {
        ids.add(s.studentId)
        const arr = bySlot.get('보강') ?? []
        arr.push(s.studentName)
        bySlot.set('보강', arr)
      }
      const tooltip = [...bySlot.entries()]
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([slot, names]) => `${formatKoreanTime(slot)}: ${names.join(', ')}`)
        .join('\n')
      if (ids.size > 0) map.set(day.eventDate, { count: ids.size, tooltip })
    }
    return map
  }, [data])

  // 이벤트 빌드 — 학사일정(항상) + 수업(주/일 보기에서만 시간 블록).
  const events = useMemo<EventInput[]>(() => {
    const result: EventInput[] = []

    // 학사일정 — 바(블록)가 아니라 코드 명칭 텍스트로 표기 (복수 시 줄바꿈으로 자동 누적).
    for (const e of academicEvents) {
      const color = e.is_system_reserved
        ? (EVENT_COLORS[e.code_name] ?? USER_EVENT_COLOR)
        : USER_EVENT_COLOR
      result.push({
        title: e.display_name ?? e.code_name,
        start: e.event_date,
        allDay: true,
        backgroundColor: 'transparent',
        borderColor: 'transparent',
        textColor: color.border,
        editable: false,
        extendedProps: { kind: 'academic' },
      })
    }

    // 수업 — 주/일(timeGrid) 보기에서만 시간대별 블록(같은 시작시간 학생 묶음).
    if (isTimeGrid) {
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
            start: `${day.eventDate}T${startTime}:00`,
            end: `${day.eventDate}T${addMinutes(startTime, maxMin)}`,
            backgroundColor: '#dbeafe',
            borderColor: '#3b82f6',
            textColor: '#1e3a8a',
            editable: false,
            extendedProps: { kind: 'class', names },
          })
        }
      }
    }
    return result
  }, [data, academicEvents, isTimeGrid])

  // 년월(타이틀) 클릭 → 숨김 date input picker 열기.
  useEffect(() => {
    const titleEl = document.querySelector<HTMLElement>('.fc-toolbar-title')
    if (!titleEl) return
    titleEl.style.cursor = 'pointer'
    titleEl.title = '클릭하여 날짜 선택'
    const handler = () => {
      const input = dateInputRef.current
      if (!input) return
      if (typeof input.showPicker === 'function') input.showPicker()
      else input.focus()
    }
    titleEl.addEventListener('click', handler)
    return () => titleEl.removeEventListener('click', handler)
  }, [viewType, data.yearMonth])

  return (
    <div className="flex h-full flex-col">
      <input
        ref={dateInputRef}
        type="date"
        aria-label="날짜로 이동"
        className="sr-only"
        onChange={(e) => {
          const v = e.target.value
          if (v) calendarRef.current?.getApi().gotoDate(v)
        }}
      />
      <FullCalendar
        ref={calendarRef}
        plugins={[dayGridPlugin, timeGridPlugin, interactionPlugin]}
        initialView="dayGridMonth"
        initialDate={`${data.yearMonth}-01`}
        locale={koLocale}
        firstDay={1}
        headerToolbar={{
          left: 'today',
          center: 'prev,title,next',
          right: 'dayGridMonth,timeGridWeek,timeGridDay',
        }}
        buttonText={{ today: '오늘', month: '월', week: '주', day: '일' }}
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
          onMonthChange(ymFromDatesSet(arg))
        }}
        // 토·일·공휴일 셀 색상.
        dayCellClassNames={(arg) => {
          const ds = `${arg.date.getFullYear()}-${String(arg.date.getMonth() + 1).padStart(2, '0')}-${String(arg.date.getDate()).padStart(2, '0')}`
          const dow = arg.date.getDay()
          if (holidayDates.has(ds) || dow === 0) return ['shb-day-holiday']
          if (dow === 6) return ['shb-day-saturday']
          return []
        }}
        // 월 보기: 날짜 숫자 아래 수업 인원수(날짜 크기)를 표기 + 시간대별 명단 툴팁(손가락 커서).
        dayCellContent={(arg) => {
          const ds = `${arg.date.getFullYear()}-${String(arg.date.getMonth() + 1).padStart(2, '0')}-${String(arg.date.getDate()).padStart(2, '0')}`
          const info = dayInfo.get(ds)
          return (
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
          )
        }}
        // 학사일정: 코드 명칭 텍스트(색만). 수업 블록(주/일): 원생 이름 줄바꿈 + 클릭 시 출결관리 이동.
        eventContent={(arg) => {
          const props = arg.event.extendedProps
          if (props.kind === 'academic') {
            return (
              <div
                className="whitespace-normal break-words px-1 text-xs font-semibold leading-snug"
                style={{ color: arg.event.textColor }}
              >
                {arg.event.title}
              </div>
            )
          }
          if (props.kind !== 'class') return undefined
          const names = (props.names as string[]) ?? []
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
  )
}
