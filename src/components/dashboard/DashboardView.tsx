'use client'

/**
 * 대시보드 (Sprint 14 T4, PRD §4.11).
 *
 * 위젯(교습소 현황 / 당일 수업 / 청구총액 추이 / 월 요약 / 메모) + 알림. TanStack Query 로
 * 위젯별 IPC 병렬 호출 + staleTime 캐싱(AC-4.11-1).
 * 차트는 recharts 를 `next/dynamic` ssr:false 로 로드 (static export 안전 + 번들 분리 R96).
 * (출결 입력 진행률 위젯은 Sprint 14 검증 중 제거 — 출결이 월 단위 'present' 일괄 생성
 *  모델이라 항상 100%가 되어 무의미.)
 */

import { useEffect, useRef, useState } from 'react'
import Link from 'next/link'
import dynamic from 'next/dynamic'
import { useQuery } from '@tanstack/react-query'
import {
  getAcademyOverview,
  getBillingTrend,
  getBirthdaysThisMonth,
  getDashboardAlerts,
  getDashboardMemos,
  getMonthlySummary,
  getTodaySchedule,
  saveDashboardMemo,
} from '@/lib/tauri'
import type { DashboardAlert, MemoNote } from '@/types/dashboard'

const OverviewCharts = dynamic(
  () => import('./charts').then((m) => m.OverviewCharts),
  { ssr: false, loading: () => <ChartSkeleton /> },
)
const BillingTrendChart = dynamic(
  () => import('./charts').then((m) => m.BillingTrendChart),
  { ssr: false, loading: () => <ChartSkeleton /> },
)

const WEEKDAY_LABEL = ['', '월', '화', '수', '목', '금', '토', '일']
const STALE = 60_000

/** 현재 연월 (YYYY-MM). */
function currentYearMonth(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
}

/** YYYY-MM 에 delta 개월을 더한 연월 (연도 경계 자동 처리). */
function shiftMonth(ym: string, delta: number): string {
  const [y, m] = ym.split('-').map(Number)
  const d = new Date(y, m - 1 + delta, 1)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

function won(n: number): string {
  return `${n.toLocaleString('ko-KR')}원`
}

/** 수업 시작시간 "16:00:00" → "pm.4시" (분이 있으면 "pm.4시30분"). */
function formatSlotTime(t: string): string {
  const [hh, mm] = t.split(':')
  const h = parseInt(hh, 10)
  const minute = parseInt(mm ?? '0', 10)
  if (Number.isNaN(h)) return t
  const ampm = h < 12 ? 'am' : 'pm'
  const h12 = h % 12 === 0 ? 12 : h % 12
  return minute === 0 ? `${ampm}.${h12}시` : `${ampm}.${h12}시${minute}분`
}

/** 메모 포스트잇 기본 높이(px) — 백엔드 MEMO_DEFAULT_HEIGHT 와 일치. */
const MEMO_DEFAULT_HEIGHT = 140
/** 포스트잇 3장 배경색 (노랑/민트/핑크). */
const MEMO_COLORS = ['#fff8b8', '#d8f5d0', '#ffd9e6']

export function DashboardView() {
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
  const alerts = useQuery({
    queryKey: ['dashboard', 'alerts'],
    queryFn: getDashboardAlerts,
    staleTime: STALE,
  })
  const trend = useQuery({
    queryKey: ['dashboard', 'billing-trend'],
    queryFn: getBillingTrend,
    staleTime: STALE,
  })
  const birthdays = useQuery({
    queryKey: ['dashboard', 'birthdays'],
    queryFn: getBirthdaysThisMonth,
    staleTime: STALE,
  })

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <h1 className="text-2xl font-bold">대시보드</h1>

      <MemoWidget />

      <MonthlySummaryWidget />

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

        {/* 오른쪽 열: 당일 수업 + 월별 청구총액 추이 스택.
            grid stretch 로 컬럼 높이 = 교습소 현황 높이, 두 위젯이 lg:flex-1 로 균등 분할 →
            당일 수업이 비어도 (당일수업 + 추이) 합이 교습소 현황 높이와 같게 고정된다. */}
        <div className="flex flex-col gap-6 lg:h-full">
          <Widget
            title={`당일 수업 (${today.data ? WEEKDAY_LABEL[today.data.weekday] : ''}요일)`}
            className="lg:flex-1 lg:min-h-0"
          >
            {today.isLoading || today.data === undefined ? (
              <Loading />
            ) : today.data.slots.length === 0 ? (
              <Empty>오늘은 예정된 수업이 없습니다.</Empty>
            ) : (
              <ul className="h-full space-y-3 overflow-y-auto">
                {today.data.slots.map((slot) => (
                  <li key={slot.start_time} className="text-gray-700">
                    <span className="font-bold text-[var(--accent)]">
                      {formatSlotTime(slot.start_time)}
                    </span>{' '}
                    <span className="text-black">({slot.students.length}명)</span>
                    {' - '}
                    {slot.students.join(', ')}
                  </li>
                ))}
              </ul>
            )}
          </Widget>

          <Widget title="월별 청구총액 추이 (최근 12개월)" className="lg:flex-1 lg:min-h-0">
            {trend.isLoading || trend.data === undefined ? (
              <Loading />
            ) : trend.data.length === 0 ? (
              <Empty>청구 데이터가 없습니다.</Empty>
            ) : (
              <BillingTrendChart data={trend.data} />
            )}
          </Widget>
        </div>
      </div>

      <Widget title="이달의 생일">
        {birthdays.isLoading || birthdays.data === undefined ? (
          <Loading />
        ) : birthdays.data.length === 0 ? (
          <Empty>이달 생일인 원생이 없습니다.</Empty>
        ) : (
          <div className="flex flex-wrap gap-x-4 gap-y-2 text-base text-[var(--foreground)]">
            {birthdays.data.map((b, i) => (
              <span key={`${b.name}-${b.day}-${i}`}>
                {b.name}
                <span className="font-medium text-[var(--accent)]">({b.day}일)</span>
              </span>
            ))}
          </div>
        )}
      </Widget>
    </div>
  )
}

// ── 월 요약 (이전/다음 월 전환) ──

function MonthlySummaryWidget() {
  const [month, setMonth] = useState(currentYearMonth())
  const monthly = useQuery({
    queryKey: ['dashboard', 'monthly', month],
    queryFn: () => getMonthlySummary(month),
    staleTime: STALE,
  })

  const navBtn =
    'inline-flex h-10 w-10 items-center justify-center rounded-md border border-[var(--border)] text-xl text-gray-600 hover:bg-[var(--background)]'
  const action = (
    <div className="flex items-center gap-1">
      <button type="button" aria-label="이전 달" onClick={() => setMonth((m) => shiftMonth(m, -1))} className={navBtn}>
        ‹
      </button>
      <button
        type="button"
        onClick={() => setMonth(currentYearMonth())}
        className="inline-flex h-10 items-center justify-center rounded-md border border-[var(--border)] px-3 text-sm text-gray-600 hover:bg-[var(--background)]"
      >
        이번 달
      </button>
      <button type="button" aria-label="다음 달" onClick={() => setMonth((m) => shiftMonth(m, 1))} className={navBtn}>
        ›
      </button>
    </div>
  )

  return (
    <Widget title={`${month} 월 요약`} action={action}>
      {monthly.isLoading || monthly.data === undefined ? (
        <Loading />
      ) : (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
          <Stat label="청구 총액" value={won(monthly.data.bill_total)} />
          <Stat label="입금" value={won(monthly.data.paid_total)} />
          <Stat label="미납" value={won(monthly.data.unpaid_total)} danger={monthly.data.unpaid_total > 0} />
          <Stat label="청구 건수" value={`${monthly.data.paid_count}/${monthly.data.bill_count} 수납`} />
          <Stat label="당월 입교" value={`${monthly.data.enrolled_this_month}명`} />
          <Stat label="당월 퇴교" value={`${monthly.data.withdrawn_this_month}명`} />
        </div>
      )}
    </Widget>
  )
}

// ── 알림 ──

const ALERT_ROUTE: Record<string, string> = {
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

// ── 메모 ──

function MemoWidget() {
  const initial = useQuery({
    queryKey: ['dashboard', 'memos'],
    queryFn: getDashboardMemos,
    staleTime: STALE,
  })
  // 로드 전엔 null. 로드 후 항상 3장으로 보정.
  const [notes, setNotes] = useState<MemoNote[] | null>(null)

  useEffect(() => {
    if (notes === null && initial.data !== undefined) {
      const loaded = initial.data
      setNotes(
        Array.from({ length: 3 }, (_, i) => loaded[i] ?? { content: '', height: MEMO_DEFAULT_HEIGHT }),
      )
    }
  }, [initial.data, notes])

  // 박스(타이틀·배경·테두리) 없이 포스트잇만 노출. 한 행에 3장 — 너비 flex 가변,
  // items-start 로 행 높이 = 가장 큰 포스트잇.
  if (notes === null) return null
  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-start">
      {notes.map((note, i) => (
        <MemoNoteCard key={i} index={i} note={note} color={MEMO_COLORS[i % MEMO_COLORS.length]} />
      ))}
    </div>
  )
}

/** 포스트잇 1장 — 내용은 1초 디바운스, 높이(드래그)는 0.6초 디바운스로 각각 자동 저장. */
function MemoNoteCard({ index, note, color }: { index: number; note: MemoNote; color: string }) {
  const [content, setContent] = useState(note.content)
  const [saved, setSaved] = useState(false)
  const taRef = useRef<HTMLTextAreaElement>(null)
  const contentRef = useRef(note.content)
  const heightRef = useRef(note.height)
  const contentTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const heightTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  const handleChange = (v: string) => {
    setContent(v)
    contentRef.current = v
    setSaved(false)
    if (contentTimer.current) clearTimeout(contentTimer.current)
    contentTimer.current = setTimeout(() => {
      void saveDashboardMemo(index, v, heightRef.current).then(() => setSaved(true))
    }, 1000)
  }

  // 높이 드래그 조정 감지 → 디바운스 저장 (reload 시 복원).
  useEffect(() => {
    const el = taRef.current
    if (el === null || typeof ResizeObserver === 'undefined') return
    const ro = new ResizeObserver(() => {
      const h = el.offsetHeight
      if (Math.abs(h - heightRef.current) < 3) return // 초기/너비변경 무시
      heightRef.current = h
      if (heightTimer.current) clearTimeout(heightTimer.current)
      heightTimer.current = setTimeout(() => {
        void saveDashboardMemo(index, contentRef.current, h)
      }, 600)
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [index])

  return (
    <div className="flex flex-1 flex-col">
      <textarea
        ref={taRef}
        value={content}
        onChange={(e) => handleChange(e.target.value)}
        placeholder="메모..."
        style={{ height: `${note.height}px`, backgroundColor: color }}
        className="min-h-[80px] w-full resize-y rounded-md border border-black/10 p-3 text-base text-[var(--foreground)] shadow-sm focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
      />
      <p className="mt-1 h-4 text-xs text-gray-500">{saved ? '저장됨' : ''}</p>
    </div>
  )
}

// ── 공통 ──

function Widget({
  title,
  action,
  children,
  className,
}: {
  title: React.ReactNode
  action?: React.ReactNode
  children: React.ReactNode
  className?: string
}) {
  return (
    <section
      className={`flex flex-col rounded-lg border border-[var(--border)] bg-white p-5 ${className ?? ''}`}
    >
      <div className="mb-4 flex items-center gap-2">
        {action}
        <h2 className="text-lg font-bold text-[var(--foreground)]">{title}</h2>
      </div>
      <div className="min-h-0 flex-1">{children}</div>
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
