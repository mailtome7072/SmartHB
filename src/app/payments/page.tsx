'use client'

/**
 * 수납 관리 페이지 (Sprint 16 — '청구/수납 관리'에서 분리, PRD §4.9).
 *
 * 두 탭으로 구성:
 *  - 수납 관리: 청구 건별 수납 상태/입금 처리 (PaymentsView)
 *  - 월별 집계: 월별 청구·수납 통계 (BillingSummaryView, 자체 기간 선택)
 *
 * 청구 목록(생성/확정)은 '청구 관리'(/billing) 로 분리됨.
 */

import { Suspense, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { BillingSearchBar } from '@/components/billing/BillingSearchBar'
import { BillingSummaryBar } from '@/components/billing/BillingSummaryBar'
import { BillingSummaryView } from '@/components/billing/BillingSummaryView'
import { PaymentsView } from '@/components/billing/PaymentsView'
import { useBillingShared } from '@/components/billing/use-billing-shared'
import { ErrorDialog } from '@/components/ui/error-dialog'

export default function PaymentsPage() {
  return (
    <Suspense fallback={<SplashScreen message="수납 관리 화면을 여는 중입니다..." />}>
      <PaymentsContent />
    </Suspense>
  )
}

type Tab = 'payments' | 'summary'
type PaymentFilter = 'all' | 'paid' | 'unpaid'

function PaymentsContent() {
  const [error, setError] = useState<string | null>(null)
  const [tab, setTab] = useState<Tab>('payments')
  const [paymentFilter, setPaymentFilter] = useState<PaymentFilter>('all')
  const {
    effectiveYearMonth,
    setYearMonth,
    monthOptions,
    searchInput,
    setSearchInput,
    appliedSearch,
    applySearch,
    clearSearch,
    matchedStudentIds,
    searchResults,
    summary,
  } = useBillingShared()

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-6xl">
        <h1 className="mb-4 text-2xl font-bold">수납 관리</h1>

        {/* 탭 */}
        <div className="mb-3 flex gap-1 border-b border-[var(--border)]">
          {([
            ['payments', '수납 관리'],
            ['summary', '월별 집계'],
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

        {/* 툴바 — 수납 탭만 (월별 집계는 자체 기간 선택 사용) */}
        {tab === 'payments' && (
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

            <BillingSearchBar
              searchInput={searchInput}
              setSearchInput={setSearchInput}
              appliedSearch={appliedSearch}
              applySearch={applySearch}
              clearSearch={clearSearch}
              resultCount={searchResults.length}
            />

            <div className="flex items-center gap-3 text-base" role="radiogroup" aria-label="수납 상태 필터">
              {(
                [
                  ['all', '전체', summary?.billCount ?? 0],
                  ['paid', '수납완료', summary?.paidCount ?? 0],
                  ['unpaid', '미수납', summary?.unpaidCount ?? 0],
                ] as const
              ).map(([key, label, count]) => (
                <label
                  key={key}
                  className="flex min-h-[44px] cursor-pointer items-center gap-1 text-gray-700"
                >
                  <input
                    type="radio"
                    name="payment-filter"
                    value={key}
                    checked={paymentFilter === key}
                    onChange={() => setPaymentFilter(key)}
                    className="h-4 w-4 cursor-pointer accent-[var(--accent)]"
                  />
                  {label}({count})
                </label>
              ))}
            </div>
          </div>
        )}

        {tab === 'payments' && summary && <BillingSummaryBar summary={summary} />}

        {tab === 'payments' && (
          <PaymentsView
            yearMonth={effectiveYearMonth}
            onError={(msg) => setError(msg)}
            matchedStudentIds={matchedStudentIds}
            searchResults={searchResults}
            paymentFilter={paymentFilter}
          />
        )}

        {tab === 'summary' && <BillingSummaryView defaultYearMonth={effectiveYearMonth} />}
      </div>

      <ErrorDialog
        open={error !== null && error !== ''}
        message={error ?? ''}
        onClose={() => setError(null)}
      />
    </AppShell>
  )
}
