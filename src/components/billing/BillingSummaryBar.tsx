'use client'

/**
 * 월 요약 5-스탯 바 (Sprint 16 — 청구/수납 페이지 공유).
 * 총수업원생 / 청구건수(+미생성) / 청구총액 / 입금완료 / 미납.
 */

import type { BillingSummary } from '@/types/billing'

export function BillingSummaryBar({ summary }: { summary: BillingSummary }) {
  if (summary.totalBillableStudents === 0 && summary.billCount === 0) return null
  const ungeneratedCount = Math.max(0, summary.totalBillableStudents - summary.billCount)
  return (
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
  )
}
