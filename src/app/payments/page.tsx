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
  // P0-4: 수납 입력 중(미저장 draft 존재) 탭/월 변경 가드 — PaymentsView 가 건수를 통지.
  const [dirtyCount, setDirtyCount] = useState(0)
  const [pendingAction, setPendingAction] = useState<
    | { kind: 'tab'; value: Tab }
    | { kind: 'month'; value: string }
    | null
  >(null)

  const guarded = (action: { kind: 'tab'; value: Tab } | { kind: 'month'; value: string }) => {
    if (dirtyCount > 0) {
      setPendingAction(action)
      return false
    }
    return true
  }
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
  // monthOptions 는 최신순 정렬(sort b>a) — index 0 이 최신월, 마지막이 최과거월.
  const monthIdx = monthOptions.indexOf(effectiveYearMonth)

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
              onClick={() => {
                if (guarded({ kind: 'tab', value: key })) setTab(key)
              }}
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
            {/* 일정 관리 메뉴의 교습년월 선택 UI(◀ 이전 / 년월 / 다음 ▶)와 통일 — monthOptions
                (교습기간 등록 월) 범위 내에서만 이동. */}
            <div className="flex items-center gap-2">
              <button
                type="button"
                aria-label="이전 달"
                disabled={monthIdx >= monthOptions.length - 1}
                onClick={() => {
                  const prevYm = monthOptions[monthIdx + 1]
                  if (prevYm !== undefined && guarded({ kind: 'month', value: prevYm })) {
                    setYearMonth(prevYm)
                  }
                }}
                className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
              >
                ← 이전
              </button>
              <span className="min-w-[7rem] text-center text-lg font-bold text-[var(--foreground)]">
                {effectiveYearMonth.slice(0, 4)}년 {Number(effectiveYearMonth.slice(5, 7))}월
              </span>
              <button
                type="button"
                aria-label="다음 달"
                disabled={monthIdx <= 0}
                onClick={() => {
                  const nextYm = monthOptions[monthIdx - 1]
                  if (nextYm !== undefined && guarded({ kind: 'month', value: nextYm })) {
                    setYearMonth(nextYm)
                  }
                }}
                className="min-h-[44px] min-w-[44px] rounded border border-[var(--border)] bg-white px-3 py-2 text-base hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
              >
                다음 →
              </button>
            </div>

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
            onDirtyChange={setDirtyCount}
          />
        )}

        {tab === 'summary' && <BillingSummaryView defaultYearMonth={effectiveYearMonth} />}
      </div>

      {/* P0-4: 미저장 수납 입력 보호 — 탭/월 변경 확인 다이얼로그 (Tauri confirm 차단 → 커스텀 모달) */}
      {pendingAction !== null && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
          <div
            className="w-full max-w-md rounded-lg border border-[var(--border)] bg-white p-6 shadow-xl"
            role="alertdialog"
            aria-modal="true"
            aria-label="미저장 입력 확인"
          >
            <h3 className="mb-2 text-lg font-bold text-[var(--danger)]">
              저장하지 않은 수납 입력이 있습니다
            </h3>
            <p className="mb-4 text-base text-gray-700">
              입력 중인 수납 정보 <strong>{dirtyCount}건</strong>이 저장되지 않았습니다.
              {pendingAction.kind === 'tab' ? ' 탭을 이동하면' : ' 청구년월을 바꾸면'} 입력한
              내용이 사라집니다.
            </p>
            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setPendingAction(null)}
                className="h-11 rounded-md bg-[var(--accent)] px-5 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
              >
                계속 입력
              </button>
              <button
                type="button"
                onClick={() => {
                  if (pendingAction.kind === 'tab') setTab(pendingAction.value)
                  else setYearMonth(pendingAction.value)
                  setDirtyCount(0)
                  setPendingAction(null)
                }}
                className="h-11 rounded-md border-2 border-[var(--danger)] px-5 text-base font-bold text-[var(--danger)] hover:bg-red-50"
              >
                입력 버리고 이동
              </button>
            </div>
          </div>
        </div>
      )}

      <ErrorDialog
        open={error !== null && error !== ''}
        message={error ?? ''}
        onClose={() => setError(null)}
      />
    </AppShell>
  )
}
