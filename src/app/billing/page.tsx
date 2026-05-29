'use client'

/**
 * 청구 관리 페이지 (Sprint 11 T7, PRD §4.9).
 *
 * 흐름:
 *  1. 월 선택 (디폴트: `getDefaultBillingYearMonth` — 가장 최근 교습기간 월)
 *  2. 청구 데이터 없으면 "청구 데이터 생성" 버튼 표시 → `generateBills`
 *  3. 청구 목록(`BillingGrid`) — draft → confirmed → closed 정렬, mid_month 우선
 *  4. 개별 행 인라인 금액 편집 + 상태별 다이얼로그 (T5 연동)
 *  5. 일괄 확정 / 당월 청구 마감 (모든 confirmed 시에만 활성)
 *
 * 캐싱: TanStack Query `['bills', yearMonth]` + `['billing-summary', yearMonth]`.
 */

import { Suspense, useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { BillingGrid } from '@/components/billing/BillingGrid'
import { CloseMonthDialog } from '@/components/billing/CloseMonthDialog'
import { PaymentsView } from '@/components/billing/PaymentsView'
import { ErrorDialog } from '@/components/ui/error-dialog'
import {
  closeBillingMonth,
  confirmAllBills,
  generateBills,
  getBillingSummary,
  listBills,
} from '@/lib/tauri'

function currentYearMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

function previousYearMonths(count: number, from: string): string[] {
  const [y, m] = from.split('-').map(Number)
  const out: string[] = []
  for (let i = 0; i < count; i++) {
    const month = m - i
    const year = y + Math.floor((month - 1) / 12)
    const monthNorm = ((month - 1) % 12 + 12) % 12 + 1
    out.push(`${year}-${String(monthNorm).padStart(2, '0')}`)
  }
  return out
}

export default function BillingPage() {
  return (
    <Suspense fallback={<SplashScreen message="청구 관리 화면을 여는 중입니다..." />}>
      <BillingContent />
    </Suspense>
  )
}

type Tab = 'bills' | 'payments'

function BillingContent() {
  const qc = useQueryClient()
  const [yearMonth, setYearMonth] = useState<string | null>(null)
  const [error, setErrorRaw] = useState<string | null>(null)
  const [closeMonthOpen, setCloseMonthOpen] = useState(false)
  const [tab, setTab] = useState<Tab>('bills')

  // 임시 디버그 — setError 호출 추적
  const [dbgSetErrCalls, setDbgSetErrCalls] = useState(0)
  const [dbgSetErrLog, setDbgSetErrLog] = useState<string[]>([])
  const setError = (val: string | null) => {
    // stack 의 1~4번째 줄에서 호출자 식별 (dev mode 함수 이름 노출)
    const stack = (new Error().stack ?? '')
      .split('\n')
      .slice(2, 6)
      .map((l) => l.trim().replace(/.*\//, ''))
      .join(' < ')
    setDbgSetErrCalls((c) => c + 1)
    setDbgSetErrLog((arr) => [
      ...arr.slice(-4),
      `${JSON.stringify(val)} :: ${stack}`,
    ])
    setErrorRaw(val)
  }

  // 디폴트 청구년월 — 오늘 날짜 기준 년월 (hotfix post-Sprint 11 정책).
  // 청구년월 'YYYY-MM' = 해당 연·월 수업 원생의 교습비 청구서.
  const effectiveYearMonth = yearMonth ?? currentYearMonth()

  const billsQuery = useQuery({
    queryKey: ['bills', effectiveYearMonth],
    queryFn: () => listBills(effectiveYearMonth),
  })
  const summaryQuery = useQuery({
    queryKey: ['billing-summary', effectiveYearMonth],
    queryFn: () => getBillingSummary(effectiveYearMonth),
  })

  const monthOptions = useMemo(
    () => previousYearMonths(12, currentYearMonth()),
    [],
  )

  const generateMutation = useMutation({
    mutationFn: () => generateBills(effectiveYearMonth),
    onMutate: () => setError(null),
    onSuccess: () => {
      setError(null)
      qc.invalidateQueries({ queryKey: ['bills', effectiveYearMonth] })
      qc.invalidateQueries({ queryKey: ['billing-summary', effectiveYearMonth] })
    },
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  const confirmAllMutation = useMutation({
    mutationFn: () => confirmAllBills(effectiveYearMonth),
    onMutate: () => setError(null),
    onSuccess: () => {
      setError(null)
      qc.invalidateQueries({ queryKey: ['bills', effectiveYearMonth] })
      qc.invalidateQueries({ queryKey: ['billing-summary', effectiveYearMonth] })
    },
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  const closeMonthMutation = useMutation({
    mutationFn: () => closeBillingMonth(effectiveYearMonth),
    onMutate: () => setError(null),
    onSuccess: () => {
      setError(null)
      setCloseMonthOpen(false)
      qc.invalidateQueries({ queryKey: ['bills', effectiveYearMonth] })
      qc.invalidateQueries({ queryKey: ['billing-summary', effectiveYearMonth] })
    },
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  const bills = billsQuery.data ?? []
  const summary = summaryQuery.data
  const draftCount = bills.filter((b) => b.status === 'draft').length
  const confirmedCount = bills.filter((b) => b.status === 'confirmed').length
  const allClosed = bills.length > 0 && bills.every((b) => b.status === 'closed')
  const showCloseButton = bills.length > 0 && draftCount === 0 && confirmedCount > 0
  // 청구 생성 버튼 표시/라벨 조건 (hotfix post-Sprint 11):
  // - bills 0건 → "청구 데이터 생성"
  // - bills>0 & totalBillableStudents>billCount → "추가 청구 데이터 생성"
  // - bills>0 & 모두 청구 → 버튼 숨김
  const ungeneratedCount = summary
    ? Math.max(0, summary.totalBillableStudents - summary.billCount)
    : 0
  const showGenerateButton = bills.length === 0 || ungeneratedCount > 0
  const generateButtonLabel =
    bills.length === 0 ? '청구 데이터 생성' : `추가 청구 데이터 생성 (${ungeneratedCount}명)`

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-6xl">
        <h1 className="mb-4 text-2xl font-bold">청구 관리</h1>

        {/* 탭 */}
        <div className="mb-3 flex gap-1 border-b border-[var(--border)]">
          {([
            ['bills', '청구 목록'],
            ['payments', '수납 관리'],
          ] as const).map(([key, label]) => (
            <button
              key={key}
              type="button"
              onClick={() => setTab(key)}
              aria-pressed={tab === key}
              className={`-mb-px min-h-[44px] border-b-2 px-4 text-base font-semibold ${
                tab === key
                  ? 'border-[var(--accent)] text-[var(--accent)]'
                  : 'border-transparent text-gray-600 hover:text-[var(--foreground)]'
              }`}
            >
              {label}
            </button>
          ))}
        </div>

        {/* 툴바 — 월 선택 + 액션 버튼 */}
        <div className="mb-4 flex flex-wrap items-center gap-3">
          <label className="text-base font-medium">
            청구년월
            <select
              value={effectiveYearMonth}
              onChange={(e) => setYearMonth(e.target.value)}
              className="ml-2 h-11 rounded-md border border-[var(--border)] px-3 text-base"
            >
              {monthOptions.includes(effectiveYearMonth) ? null : (
                <option value={effectiveYearMonth}>{effectiveYearMonth}</option>
              )}
              {monthOptions.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </label>

          {tab === 'bills' && showGenerateButton && (
            <button
              type="button"
              onClick={() => generateMutation.mutate()}
              disabled={generateMutation.isPending}
              className="h-11 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
            >
              {generateMutation.isPending ? '생성 중...' : generateButtonLabel}
            </button>
          )}

          {tab === 'bills' && draftCount > 0 && (
            <button
              type="button"
              onClick={() => confirmAllMutation.mutate()}
              disabled={confirmAllMutation.isPending}
              className="h-11 rounded-md border border-[var(--accent)] px-4 text-base text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
            >
              {confirmAllMutation.isPending ? '확정 중...' : `미확정 ${draftCount}건 일괄 확정`}
            </button>
          )}

          {tab === 'bills' && showCloseButton && (
            <button
              type="button"
              onClick={() => setCloseMonthOpen(true)}
              className="h-11 rounded-md border-2 border-[var(--danger)] px-4 text-base font-semibold text-[var(--danger)] hover:bg-red-50"
            >
              당월 청구 마감
            </button>
          )}

          {tab === 'bills' && allClosed && (
            <span className="rounded-md border border-gray-300 bg-gray-100 px-3 py-2 text-sm text-gray-600">
              ✓ 마감 완료
            </span>
          )}
        </div>

        {/* 미확정 청구 배너 (AC-4.9-5) — 청구 탭에서만 */}
        {tab === 'bills' && draftCount > 0 && (
          <div
            role="status"
            className="mb-3 rounded-md border-2 border-amber-400 bg-amber-50 p-3 text-sm text-amber-900"
          >
            미확정 청구가 <strong>{draftCount}건</strong> 있습니다. 검토 후 확정해 주세요.
          </div>
        )}

        {/* 요약 */}
        {summary && (summary.totalBillableStudents > 0 || summary.billCount > 0) && (
          <div className="mb-3 grid grid-cols-2 gap-3 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm md:grid-cols-3 lg:grid-cols-5">
            <div>
              총수업원생: <strong>{summary.totalBillableStudents}명</strong>
            </div>
            <div>
              청구 건수:{' '}
              <strong>
                {summary.billCount}건
                {ungeneratedCount > 0 && (
                  <span className="ml-1 text-amber-700">/ 미생성 {ungeneratedCount}명</span>
                )}
              </strong>
            </div>
            <div>
              청구 총액: <strong>{summary.totalBilled.toLocaleString()}원</strong>
            </div>
            <div>
              입금 완료: <strong>{summary.totalPaid.toLocaleString()}원</strong>
            </div>
            <div>
              미납: <strong className="text-[var(--danger)]">{summary.totalUnpaid.toLocaleString()}원</strong>
            </div>
          </div>
        )}

        {billsQuery.isLoading && <p>불러오는 중...</p>}

        {tab === 'bills' && !billsQuery.isLoading && bills.length === 0 && !showGenerateButton && (
          <p className="text-gray-600">청구 데이터가 없습니다.</p>
        )}

        {tab === 'bills' && bills.length > 0 && (
          <BillingGrid
            bills={bills}
            yearMonth={effectiveYearMonth}
            onError={(msg) => setError(msg)}
          />
        )}

        {tab === 'payments' && (
          <PaymentsView yearMonth={effectiveYearMonth} onError={(msg) => setError(msg)} />
        )}
      </div>

      {summary && (
        <CloseMonthDialog
          open={closeMonthOpen}
          yearMonth={effectiveYearMonth}
          confirmedCount={confirmedCount}
          totalBilled={summary.totalBilled}
          onConfirm={async () => {
            await closeMonthMutation.mutateAsync()
          }}
          onCancel={() => setCloseMonthOpen(false)}
        />
      )}

      <ErrorDialog
        open={error !== null && error !== ''}
        message={error ?? ''}
        onClose={() => setError(null)}
      />

      {/* 임시 디버그 — 다음 commit 에서 제거 */}
      <div
        style={{
          position: 'fixed',
          top: 8,
          right: 8,
          background: 'yellow',
          color: 'black',
          padding: 8,
          fontSize: 12,
          fontFamily: 'monospace',
          zIndex: 2147483645,
          maxWidth: 560,
          border: '2px solid red',
          whiteSpace: 'pre-wrap',
        }}
      >
        PAGE error = {JSON.stringify(error)}
        {'\n'}ErrorDialog open = {String(error !== null && error !== '')}
        {'\n'}setError 호출: {dbgSetErrCalls}
        {'\n'}최근 호출: {dbgSetErrLog.join(' → ')}
      </div>
    </AppShell>
  )
}
