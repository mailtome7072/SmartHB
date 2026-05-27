'use client'

/**
 * 수업 관리 캘린더 (FullCalendar 래퍼) — Sprint 10 T11 (PRD §4.6.1, ADR-006).
 *
 * - 일/주/월 뷰 전환 (timeGridDay / timeGridWeek / dayGridMonth)
 * - 정규 수업 = 시간 이벤트 (start_time ~ +class_minutes), 보강 = allDay 이벤트
 * - 이벤트 클릭 → 부모 onEventClick (원생 상세 팝업)
 * - 뷰 이동(prev/next/today) 시 보이는 월 변경 → onMonthChange 로 부모 refetch
 *
 * static export(R67): 본 컴포넌트는 페이지에서 `dynamic(..., { ssr: false })` 로 로드.
 */

import { useMemo } from 'react'
import FullCalendar from '@fullcalendar/react'
import dayGridPlugin from '@fullcalendar/daygrid'
import timeGridPlugin from '@fullcalendar/timegrid'
import interactionPlugin from '@fullcalendar/interaction'
import koLocale from '@fullcalendar/core/locales/ko'
import type { EventClickArg, DatesSetArg, EventInput } from '@fullcalendar/core'
import type { CalendarMonth } from '@/types/calendar'
import type { StudentDetailTarget } from './StudentDetailPopup'

interface Props {
  data: CalendarMonth
  /** 보이는 기간이 다른 월로 바뀌면 호출 — 부모가 yearMonth state 갱신 → refetch. */
  onMonthChange: (yearMonth: string) => void
  onEventClick: (target: StudentDetailTarget) => void
}

/** "HH:MM" + 분 → "HH:MM:SS" (24시 넘어가면 그대로 다음날 계산은 FullCalendar 가 처리). */
function addMinutes(startTime: string, addMin: number): string {
  const [h, m] = startTime.split(':').map(Number)
  const total = h * 60 + m + addMin
  const eh = Math.floor(total / 60)
  const em = total % 60
  return `${String(eh).padStart(2, '0')}:${String(em).padStart(2, '0')}:00`
}

/** FullCalendar 의 현재 표시 기준일에서 "YYYY-MM" 추출. */
function yearMonthFromDatesSet(arg: DatesSetArg): string {
  // view.currentStart 는 표시 중인 기간의 시작 — 월 뷰는 1일, 주/일 뷰는 해당 날짜.
  // 월 경계를 안정적으로 잡기 위해 start~end 중간 지점을 사용.
  const mid = new Date((arg.start.getTime() + arg.end.getTime()) / 2)
  return `${mid.getFullYear()}-${String(mid.getMonth() + 1).padStart(2, '0')}`
}

export default function ClassCalendar({
  data,
  onMonthChange,
  onEventClick,
}: Props) {
  const events = useMemo<EventInput[]>(() => {
    const result: EventInput[] = []
    for (const day of data.days) {
      for (const s of day.regularSessions) {
        const base: EventInput = {
          title: s.studentName,
          backgroundColor: '#3b82f6',
          borderColor: '#2563eb',
          extendedProps: {
            studentId: s.studentId,
            studentName: s.studentName,
            sessionType: 'regular',
            startTime: s.startTime,
            classMinutes: s.classMinutes,
            eventDate: day.eventDate,
          },
        }
        if (s.startTime !== null) {
          base.start = `${day.eventDate}T${s.startTime}:00`
          base.end = `${day.eventDate}T${addMinutes(s.startTime, s.classMinutes)}`
        } else {
          base.start = day.eventDate
          base.allDay = true
        }
        result.push(base)
      }
      for (const s of day.makeupSessions) {
        result.push({
          title: `[보강] ${s.studentName}`,
          start: day.eventDate,
          allDay: true,
          backgroundColor: '#10b981',
          borderColor: '#059669',
          extendedProps: {
            studentId: s.studentId,
            studentName: s.studentName,
            sessionType: 'makeup',
            startTime: null,
            classMinutes: s.classMinutes,
            eventDate: day.eventDate,
          },
        })
      }
    }
    return result
  }, [data])

  function handleEventClick(arg: EventClickArg) {
    const p = arg.event.extendedProps
    onEventClick({
      studentId: p.studentId as number,
      studentName: p.studentName as string,
      sessionType: p.sessionType as 'regular' | 'makeup',
      startTime: p.startTime as string | null,
      classMinutes: p.classMinutes as number,
      eventDate: p.eventDate as string,
    })
  }

  return (
    <FullCalendar
      plugins={[dayGridPlugin, timeGridPlugin, interactionPlugin]}
      initialView="dayGridMonth"
      initialDate={`${data.yearMonth}-01`}
      locale={koLocale}
      headerToolbar={{
        left: 'prev,next today',
        center: 'title',
        right: 'dayGridMonth,timeGridWeek,timeGridDay',
      }}
      buttonText={{
        today: '오늘',
        month: '월',
        week: '주',
        day: '일',
      }}
      events={events}
      eventClick={handleEventClick}
      datesSet={(arg) => onMonthChange(yearMonthFromDatesSet(arg))}
      slotMinTime="12:00:00"
      slotMaxTime="23:00:00"
      allDaySlot
      allDayText="보강"
      height="auto"
      dayMaxEvents={4}
      nowIndicator
    />
  )
}
