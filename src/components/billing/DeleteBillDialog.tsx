'use client'

/**
 * 청구 삭제 확인 다이얼로그 — Sprint 20 T4 (ADR-010 B안).
 *
 * 위험 동작이므로 명시적 확인을 요구한다 (frontend.md 실수 복구 규칙).
 * - 확정(confirmed) 청구면 확정 상태 경고를 덧붙인다.
 * - 수납완료(isPaid) 청구는 BillingGrid 에서 삭제 버튼 자체가 비활성 (백엔드도 거부).
 */

import { useState } from 'react'
import type { Bill } from '@/types/billing'

interface Props {
  bill: Bill
  onConfirm: () => Promise<void>
  onCancel: () => void
}

export function DeleteBillDialog({ bill, onConfirm, onCancel }: Props) {
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleConfirm = async () => {
    setSubmitting(true)
    setError(null)
    try {
      await onConfirm()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="청구 삭제 확인"
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
        <h2 className="mb-2 text-xl font-bold text-[var(--danger)]">청구 삭제</h2>
        <p className="mb-4 text-base text-gray-700">
          <strong>{bill.studentName}</strong> 원생의 <strong>{bill.billYearMonth}</strong> 청구를
          삭제합니다. 이 작업은 되돌릴 수 없습니다.
        </p>

        <div className="mb-4 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm">
          금액: <strong>{bill.adjustedAmount.toLocaleString()}원</strong>
          <br />
          상태: <strong>{bill.status === 'draft' ? '미확정' : '확정'}</strong>
        </div>

        {bill.status === 'confirmed' && (
          <div
            role="alert"
            className="mb-3 rounded-md border-2 border-amber-400 bg-amber-50 p-3 text-sm text-amber-900"
          >
            이미 <strong>확정</strong>된 청구입니다. 삭제 전에 다시 한 번 확인하세요.
          </div>
        )}

        {error !== null && (
          <div
            role="alert"
            className="mb-3 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-sm text-[var(--danger)]"
          >
            {error}
          </div>
        )}

        <div className="flex gap-2">
          <button
            type="button"
            onClick={onCancel}
            disabled={submitting}
            className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            취소
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            disabled={submitting}
            className="min-h-[44px] flex-1 rounded-md bg-[var(--danger)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? '삭제 중...' : '삭제'}
          </button>
        </div>
      </div>
    </div>
  )
}
