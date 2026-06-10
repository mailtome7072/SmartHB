'use client'

/**
 * 청구 관리 페이지 (Sprint 11 T7, PRD §4.9 / Sprint 16 메뉴 분리).
 *
 * '청구/수납 관리' 분리 후 본 페이지는 **청구 목록**만 담당한다 (수납·월별집계는 /payments).
 * 흐름:
 *  1. 청구년월 선택 (디폴트: 현재 년월, 교습기간 등록 월로 자동 보정)
 *  2. 청구 데이터 없으면 "청구 데이터 생성" → generateBills
 *  3. 청구 목록(BillingGrid) — draft→confirmed 정렬 + 인라인 금액 편집/확정
 *  4. 미확정 일괄 확정
 */

import { Suspense, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { BillingGrid } from '@/components/billing/BillingGrid'
import { BillingSearchBar } from '@/components/billing/BillingSearchBar'
import { BillingSummaryBar } from '@/components/billing/BillingSummaryBar'
import { useBillingShared } from '@/components/billing/use-billing-shared'
import { ErrorDialog } from '@/components/ui/error-dialog'
import { confirmAllBills, generateBills, listBills } from '@/lib/tauri'

export default function BillingPage() {
  return (
    <Suspense fallback={<SplashScreen message="청구 관리 화면을 여는 중입니다..." />}>
      <BillingContent />
    </Suspense>
  )
}

type BillFilter = 'all' | 'confirmed' | 'draft'

function BillingContent() {
  const qc = useQueryClient()
  const [error, setError] = useState<string | null>(null)
  const [billFilter, setBillFilter] = useState<BillFilter>('all')
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

  const billsQuery = useQuery({
    queryKey: ['bills', effectiveYearMonth],
    queryFn: () => listBills(effectiveYearMonth),
  })

  const invalidate = () => {
    qc.invalidateQueries({ queryKey: ['bills', effectiveYearMonth] })
    qc.invalidateQueries({ queryKey: ['billing-summary', effectiveYearMonth] })
  }
  const generateMutation = useMutation({
    mutationFn: () => generateBills(effectiveYearMonth),
    onMutate: () => setError(null),
    onSuccess: invalidate,
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })
  const confirmAllMutation = useMutation({
    mutationFn: () => confirmAllBills(effectiveYearMonth),
    onMutate: () => setError(null),
    onSuccess: invalidate,
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  const bills = billsQuery.data ?? []
  const draftCount = bills.filter((b) => b.status === 'draft').length
  const confirmedCount = bills.filter((b) => b.status === 'confirmed').length
  // 청구 생성 버튼 표시/라벨 (hotfix post-Sprint 11):
  // bills 0건 → "청구 데이터 생성" / 미생성 원생 있으면 "추가 청구 데이터 생성" / 모두 청구 시 숨김.
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

        {/* 툴바 — 월 선택 + 검색 + 상태 필터 + 액션 버튼 */}
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

          <div className="flex items-center gap-3 text-base" role="radiogroup" aria-label="청구 상태 필터">
            {(
              [
                ['all', '전체', bills.length],
                ['confirmed', '확정', confirmedCount],
                ['draft', '미확정', draftCount],
              ] as const
            ).map(([key, label, count]) => (
              <label
                key={key}
                className="flex min-h-[44px] cursor-pointer items-center gap-1 text-gray-700"
              >
                <input
                  type="radio"
                  name="bill-filter"
                  value={key}
                  checked={billFilter === key}
                  onChange={() => setBillFilter(key)}
                  className="h-4 w-4 cursor-pointer accent-[var(--accent)]"
                />
                {label}({count})
              </label>
            ))}
          </div>

          {showGenerateButton && (
            <button
              type="button"
              onClick={() => generateMutation.mutate()}
              disabled={generateMutation.isPending}
              className="h-11 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
            >
              {generateMutation.isPending ? '생성 중...' : generateButtonLabel}
            </button>
          )}

          {draftCount > 0 && (
            <button
              type="button"
              onClick={() => confirmAllMutation.mutate()}
              disabled={confirmAllMutation.isPending}
              className="h-11 rounded-md border border-[var(--accent)] px-4 text-base text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
            >
              {confirmAllMutation.isPending ? '확정 중...' : `미확정 ${draftCount}건 일괄 확정`}
            </button>
          )}
        </div>

        {/* 미확정 청구 배너 (AC-4.9-5) */}
        {draftCount > 0 && (
          <div
            role="status"
            className="mb-3 rounded-md border-2 border-amber-400 bg-amber-50 p-3 text-sm text-amber-900"
          >
            미확정 청구가 <strong>{draftCount}건</strong> 있습니다. 검토 후 확정해 주세요.
          </div>
        )}

        {summary && <BillingSummaryBar summary={summary} />}

        {billsQuery.isLoading && <p>불러오는 중...</p>}

        {!billsQuery.isLoading && bills.length === 0 && !showGenerateButton && (
          <p className="text-gray-600">청구 데이터가 없습니다.</p>
        )}

        {bills.length > 0 && (
          <BillingGrid
            bills={bills.filter((b) => {
              if (matchedStudentIds !== null && !matchedStudentIds.has(b.studentId)) return false
              if (billFilter === 'confirmed' && b.status !== 'confirmed') return false
              if (billFilter === 'draft' && b.status !== 'draft') return false
              return true
            })}
            yearMonth={effectiveYearMonth}
            onError={(msg) => setError(msg)}
          />
        )}
      </div>

      <ErrorDialog
        open={error !== null && error !== ''}
        message={error ?? ''}
        onClose={() => setError(null)}
      />
    </AppShell>
  )
}
