'use client'

/**
 * 수업 관리 페이지 — Sprint 10 T11 (PRD §4.6) + 1차 시각 검증 반영.
 *
 * 탭 2개:
 * - 캘린더: 일/주/월 뷰 (FullCalendar) + 학사일정 + 원생명 클릭 → 출결관리 이동
 * - 보강 관리: 소멸 임박 순 목록 (상단 원생검색 + 재원중 체크 필터)
 *
 * FullCalendar 는 static export(R67) 대응으로 `dynamic(..., { ssr: false })` 로드.
 */

import { useMemo, useState } from 'react'
import dynamic from 'next/dynamic'
import { useRouter } from 'next/navigation'
import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { getCalendarData, listScheduleEvents } from '@/lib/tauri'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { MakeupManagementView } from '@/components/schedules/MakeupManagementView'
import { useAppStore } from '@/stores/app-store'

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

/** 해당 월 ± 7일 범위(주 뷰가 인접 월을 걸치는 경우 대비) 의 from/to 일자. */
function monthRange(yearMonth: string): { from: string; to: string } {
  const [y, m] = yearMonth.split('-').map(Number)
  const start = new Date(y, m - 1, 1)
  start.setDate(start.getDate() - 7)
  const end = new Date(y, m, 0)
  end.setDate(end.getDate() + 7)
  const fmt = (d: Date) =>
    `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
  return { from: fmt(start), to: fmt(end) }
}

type Tab = 'calendar' | 'makeup'

export default function SchedulesPage() {
  const router = useRouter()
  const setAttendanceSearchPreset = useAppStore((s) => s.setAttendanceSearchPreset)
  const [tab, setTab] = useState<Tab>('calendar')
  const [yearMonth, setYearMonth] = useState(currentYearMonth)
  // 보강 관리 탭 필터 — 재원중 체크 (원생 검색은 2차 검증에서 제거).
  const [makeupEnrolledOnly, setMakeupEnrolledOnly] = useState(true)

  // placeholderData=keepPreviousData: 달 이동 refetch 중에도 직전 데이터 유지 →
  // 캘린더가 언마운트/재마운트되며 초기 날짜로 튕기는 현상 방지 (오늘/이전/다음 동작 안정화).
  const calendarQuery = useQuery({
    queryKey: ['calendar-data', yearMonth],
    queryFn: () => getCalendarData(yearMonth),
    enabled: tab === 'calendar',
    placeholderData: keepPreviousData,
  })

  const range = useMemo(() => monthRange(yearMonth), [yearMonth])
  const academicQuery = useQuery({
    queryKey: ['calendar-academic', range.from, range.to],
    queryFn: () => listScheduleEvents(range.from, range.to),
    enabled: tab === 'calendar',
    placeholderData: keepPreviousData,
  })

  function goToAttendance(studentName: string) {
    setAttendanceSearchPreset(studentName)
    router.push('/attendance')
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <main className="flex h-full flex-col">
        <header className="flex flex-wrap items-center gap-4 border-b border-[var(--border)] py-4">
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

          {/* 보강 관리 탭: 재원중 체크 (원생 검색은 2차 검증 피드백으로 제거) */}
          {tab === 'makeup' && (
            <label className="flex min-h-[44px] cursor-pointer items-center gap-2 text-base text-gray-700">
              <input
                type="checkbox"
                checked={makeupEnrolledOnly}
                onChange={(e) => setMakeupEnrolledOnly(e.target.checked)}
                aria-label="재원중 원생만 보기"
                className="h-5 w-5 cursor-pointer accent-[var(--accent)]"
              />
              재원중
            </label>
          )}
        </header>

        <section className="min-h-0 flex-1">
          {tab === 'calendar' && (
            <div className="h-full pt-4">
              {calendarQuery.data !== undefined && (
                <ClassCalendar
                  data={calendarQuery.data}
                  academicEvents={academicQuery.data ?? []}
                  onMonthChange={(ym) => setYearMonth(ym)}
                  onStudentNameClick={goToAttendance}
                />
              )}
            </div>
          )}

          {tab === 'makeup' && (
            <div className="h-full overflow-auto">
              <MakeupManagementView
                yearMonth={yearMonth}
                search=""
                enrolledOnly={makeupEnrolledOnly}
              />
            </div>
          )}
        </section>
      </main>
    </AppShell>
  )
}
