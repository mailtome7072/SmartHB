'use client'

/**
 * 수업 관리 페이지 — Sprint 10 T11 (PRD §4.6).
 *
 * 탭 2개:
 * - 캘린더: 일/주/월 뷰 (FullCalendar) + 원생 상세 팝업
 * - 보강 관리: 소멸 임박 순 보강 필요 원생 목록
 *
 * FullCalendar 는 static export(R67) 대응으로 `dynamic(..., { ssr: false })` 로드.
 */

import { useState } from 'react'
import dynamic from 'next/dynamic'
import { useQuery } from '@tanstack/react-query'
import { getCalendarData } from '@/lib/tauri'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { MakeupManagementView } from '@/components/schedules/MakeupManagementView'
import {
  StudentDetailPopup,
  type StudentDetailTarget,
} from '@/components/schedules/StudentDetailPopup'

const ClassCalendar = dynamic(
  () => import('@/components/schedules/ClassCalendar'),
  {
    ssr: false,
    loading: () => <p className="text-base text-gray-500">캘린더 불러오는 중...</p>,
  },
)

function currentYearMonth(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
}

type Tab = 'calendar' | 'makeup'

export default function SchedulesPage() {
  const [tab, setTab] = useState<Tab>('calendar')
  const [yearMonth, setYearMonth] = useState(currentYearMonth)
  const [detailTarget, setDetailTarget] = useState<StudentDetailTarget | null>(
    null,
  )

  const calendarQuery = useQuery({
    queryKey: ['calendar-data', yearMonth],
    queryFn: () => getCalendarData(yearMonth),
    enabled: tab === 'calendar',
  })

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <main className="flex h-full flex-col">
        <header className="flex items-center gap-4 border-b border-[var(--border)] py-4">
          <h1 className="text-2xl font-bold">수업 관리</h1>
          <div className="flex gap-1 rounded-lg bg-gray-100 p-1" role="tablist">
            <button
              type="button"
              role="tab"
              aria-selected={tab === 'calendar'}
              onClick={() => setTab('calendar')}
              className={
                tab === 'calendar'
                  ? 'min-h-[40px] rounded-md bg-white px-4 text-base font-semibold text-[var(--accent)] shadow-sm'
                  : 'min-h-[40px] rounded-md px-4 text-base text-gray-600 hover:text-gray-900'
              }
            >
              캘린더
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={tab === 'makeup'}
              onClick={() => setTab('makeup')}
              className={
                tab === 'makeup'
                  ? 'min-h-[40px] rounded-md bg-white px-4 text-base font-semibold text-[var(--accent)] shadow-sm'
                  : 'min-h-[40px] rounded-md px-4 text-base text-gray-600 hover:text-gray-900'
              }
            >
              보강 관리
            </button>
          </div>
        </header>

        <section className="flex-1 overflow-auto">
          {tab === 'calendar' && (
            <div className="py-4">
              {calendarQuery.data !== undefined && (
                <ClassCalendar
                  data={calendarQuery.data}
                  onMonthChange={(ym) => setYearMonth(ym)}
                  onEventClick={(target) => setDetailTarget(target)}
                />
              )}
            </div>
          )}

          {tab === 'makeup' && <MakeupManagementView yearMonth={yearMonth} />}
        </section>

        {detailTarget !== null && (
          <StudentDetailPopup
            target={detailTarget}
            yearMonth={yearMonth}
            onClose={() => setDetailTarget(null)}
          />
        )}
      </main>
    </AppShell>
  )
}
