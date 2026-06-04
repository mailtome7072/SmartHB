'use client'

/**
 * 대시보드 (Sprint 14 T4, PRD §4.11).
 *
 * 6개 위젯 + 5종 알림. TanStack Query 로 위젯별 IPC 병렬 호출 + staleTime 캐싱(AC-4.11-1).
 * 차트는 recharts 를 `next/dynamic` ssr:false 로 로드 (static export 안전 + 번들 분리 R96).
 */

import { useEffect, useRef, useState } from 'react'
import Link from 'next/link'
import dynamic from 'next/dynamic'
import { useQuery } from '@tanstack/react-query'
import {
  getAcademyOverview,
  getAttendanceProgress,
  getDashboardAlerts,
  getDashboardMemo,
  getMonthlySummary,
  getTodaySchedule,
  saveDashboardMemo,
} from '@/lib/tauri'
import type { DashboardAlert } from '@/types/dashboard'

const OverviewCharts = dynamic(
  () => import('./charts').then((m) => m.OverviewCharts),
  { ssr: false, loading: () => <ChartSkeleton /> },
)

const WEEKDAY_LABEL = ['', '월', '화', '수', '목', '금', '토', '일']
const STALE = 60_000

/** 현재 연월 (YYYY-MM). */
function currentYearMonth(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
}

function won(n: number): string {
  return `${n.toLocaleString('ko-KR')}원`
}

export function DashboardView() {
  const ym = currentYearMonth()

  const overview = useQuery({
    queryKey: ['dashboard', 'overview'],
    queryFn: getAcademyOverview,
    staleTime: STALE,
  })
  const today = useQuery({
    queryKey: ['dashboard', 'today'],
    queryFn: getTodaySchedule,
    staleTime: STALE,
  })
  const monthly = useQuery({
    queryKey: ['dashboard', 'monthly', ym],
    queryFn: () => getMonthlySummary(ym),
    staleTime: STALE,
  })
  const progress = useQuery({
    queryKey: ['dashboard', 'progress', ym],
    queryFn: () => getAttendanceProgress(ym),
    staleTime: STALE,
  })
  const alerts = useQuery({
    queryKey: ['dashboard', 'alerts'],
    queryFn: getDashboardAlerts,
    staleTime: STALE,
  })

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <h1 className="text-2xl font-bold">대시보드</h1>

      <MemoWidget />

      <AlertsPanel alerts={alerts.data ?? []} loading={alerts.isLoading} />

      <div className="grid gap-6 lg:grid-cols-2">
        <Widget title="교습소 현황">
          {overview.isLoading || overview.data === undefined ? (
            <Loading />
          ) : (
            <>
              <p className="mb-4 text-base text-gray-700">
                재원 총원 <span className="text-3xl font-bold text-[var(--foreground)]">{overview.data.total_active}</span> 명
              </p>
              <OverviewCharts overview={overview.data} />
            </>
          )}
        </Widget>

        <Widget title={`당일 수업 (${today.data ? WEEKDAY_LABEL[today.data.weekday] : ''}요일)`}>
          {today.isLoading || today.data === undefined ? (
            <Loading />
          ) : today.data.slots.length === 0 ? (
            <Empty>오늘은 예정된 수업이 없습니다.</Empty>
          ) : (
            <ul className="space-y-3">
              {today.data.slots.map((slot) => (
                <li key={slot.start_time} className="flex gap-3">
                  <span className="w-16 shrink-0 font-bold text-[var(--accent)]">{slot.start_time}</span>
                  <span className="text-gray-700">{slot.students.join(', ')} ({slot.students.length}명)</span>
                </li>
              ))}
            </ul>
          )}
        </Widget>

        <Widget title={`${ym} 월 요약`}>
          {monthly.isLoading || monthly.data === undefined ? (
            <Loading />
          ) : (
            <div className="grid grid-cols-2 gap-4">
              <Stat label="청구 총액" value={won(monthly.data.bill_total)} />
              <Stat label="입금" value={won(monthly.data.paid_total)} />
              <Stat label="미납" value={won(monthly.data.unpaid_total)} danger={monthly.data.unpaid_total > 0} />
              <Stat label="청구 건수" value={`${monthly.data.paid_count}/${monthly.data.bill_count} 수납`} />
              <Stat label="당월 입교" value={`${monthly.data.enrolled_this_month}명`} />
              <Stat label="당월 퇴교" value={`${monthly.data.withdrawn_this_month}명`} />
            </div>
          )}
        </Widget>

        <Widget title="출결 입력 진행률">
          {progress.isLoading || progress.data === undefined ? (
            <Loading />
          ) : (
            <ProgressBody
              expected={progress.data.expected_days}
              recorded={progress.data.recorded_days}
              missing={progress.data.missing_dates}
            />
          )}
        </Widget>
      </div>
    </div>
  )
}

// ── 알림 ──

const ALERT_ROUTE: Record<string, string> = {
  attendance_missing: '/attendance',
  makeup_expiring: '/attendance',
  draft_bills: '/billing',
  academic_not_set: '/academic',
  diagnosis_issues: '/settings/diagnosis',
}

function AlertsPanel({ alerts, loading }: { alerts: DashboardAlert[]; loading: boolean }) {
  if (loading) return null
  if (alerts.length === 0) {
    return (
      <div className="rounded-lg border border-green-300 bg-green-50 p-4 text-base text-green-700">
        ✅ 현재 처리할 알림이 없습니다.
      </div>
    )
  }
  return (
    <div className="flex flex-wrap gap-3">
      {alerts.map((a) => {
        const href = ALERT_ROUTE[a.kind] ?? '/'
        const color =
          a.severity === 'red'
            ? 'border-red-300 bg-red-50 text-[var(--danger)]'
            : a.severity === 'orange'
              ? 'border-amber-300 bg-amber-50 text-amber-800'
              : 'border-blue-300 bg-blue-50 text-blue-800'
        return (
          <Link
            key={a.kind}
            href={href}
            className={`flex min-h-[44px] min-w-[200px] flex-1 items-center rounded-lg border px-4 py-2 text-sm font-medium transition-colors hover:brightness-95 ${color}`}
          >
            {a.message}
          </Link>
        )
      })}
    </div>
  )
}

// ── 출결 진행률 ──

function ProgressBody({
  expected,
  recorded,
  missing,
}: {
  expected: number
  recorded: number
  missing: string[]
}) {
  const pct = expected === 0 ? 100 : Math.round((recorded / expected) * 100)
  return (
    <div>
      <div className="mb-2 flex items-baseline justify-between">
        <span className="text-2xl font-bold text-[var(--foreground)]">{pct}%</span>
        <span className="text-sm text-gray-600">
          {recorded}/{expected} 수업일 입력
        </span>
      </div>
      <div className="mb-4 h-3 w-full overflow-hidden rounded-full bg-gray-200">
        <div className="h-full rounded-full bg-[var(--accent)]" style={{ width: `${pct}%` }} />
      </div>
      {missing.length === 0 ? (
        <Empty>미입력 일자가 없습니다.</Empty>
      ) : (
        <div>
          <p className="mb-2 text-sm text-gray-600">미입력 일자 (클릭 시 출결 화면 이동)</p>
          <div className="flex flex-wrap gap-2">
            {missing.map((d) => {
              const isFriday = new Date(d).getDay() === 5 // AC-4.11-5 금요일 강조
              return (
                <Link
                  key={d}
                  href={`/attendance?date=${d}`}
                  className={`min-h-[44px] rounded-md border px-3 py-2 text-sm transition-colors hover:bg-[var(--background)] ${
                    isFriday
                      ? 'border-[var(--accent)] font-bold text-[var(--accent)]'
                      : 'border-[var(--border)] text-gray-700'
                  }`}
                >
                  {d.slice(5)}
                </Link>
              )
            })}
          </div>
        </div>
      )}
    </div>
  )
}

// ── 메모 ──

function MemoWidget() {
  const initial = useQuery({
    queryKey: ['dashboard', 'memo'],
    queryFn: getDashboardMemo,
    staleTime: STALE,
  })
  const [text, setText] = useState<string | null>(null)
  const [saved, setSaved] = useState(false)
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

  // 최초 로드 값 반영 (사용자 입력 시작 전).
  useEffect(() => {
    if (text === null && initial.data !== undefined) {
      setText(initial.data ?? '')
    }
  }, [initial.data, text])

  const handleChange = (v: string) => {
    setText(v)
    setSaved(false)
    if (timer.current) clearTimeout(timer.current)
    timer.current = setTimeout(() => {
      void saveDashboardMemo(v).then(() => setSaved(true))
    }, 1000)
  }

  return (
    <Widget title="메모">
      <textarea
        value={text ?? ''}
        onChange={(e) => handleChange(e.target.value)}
        placeholder="자유 메모 — 입력 1초 후 자동 저장됩니다."
        rows={4}
        className="w-full resize-y rounded-md border border-[var(--border)] bg-[#fffdf5] p-3 text-base text-[var(--foreground)] focus:border-[var(--accent)] focus:outline-none"
      />
      <p className="mt-1 h-4 text-xs text-gray-500">{saved ? '저장됨' : ''}</p>
    </Widget>
  )
}

// ── 공통 ──

function Widget({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="rounded-lg border border-[var(--border)] bg-white p-5">
      <h2 className="mb-4 text-lg font-bold text-[var(--foreground)]">{title}</h2>
      {children}
    </section>
  )
}

function Stat({ label, value, danger }: { label: string; value: string; danger?: boolean }) {
  return (
    <div className="rounded-md bg-[var(--background)] p-3">
      <p className="text-xs text-gray-500">{label}</p>
      <p className={`mt-1 text-lg font-bold ${danger ? 'text-[var(--danger)]' : 'text-[var(--foreground)]'}`}>
        {value}
      </p>
    </div>
  )
}

function Loading() {
  return <p className="text-sm text-gray-400">불러오는 중...</p>
}

function ChartSkeleton() {
  return <div className="h-44 animate-pulse rounded-md bg-gray-100" />
}

function Empty({ children }: { children: React.ReactNode }) {
  return <p className="text-sm text-gray-500">{children}</p>
}
