'use client'

/**
 * 확정(`confirmed`) 상태 청구 수정 확인 다이얼로그 — Sprint 11 T5 (AC-4.9-3).
 *
 * 확정된 청구는 사유 없이 수정 가능하지만 실수 방지를 위해 확인 다이얼로그를 요구한다.
 * 마감(`closed`) 상태는 본 다이얼로그가 아닌 [[CloseReasonDialog]] 가 담당 (사유 필수).
 */

import { useState } from 'react'

interface Props {
  open: boolean
  studentName: string
  currentAmount: number
  newAmount: number
  onConfirm: () => Promise<void>
  onCancel: () => void
}

export function ConfirmBillUpdateDialog({
  open,
  studentName,
  currentAmount,
  newAmount,
  onConfirm,
  onCancel,
}: Props) {
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  if (!open) return null

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
      aria-label="확정 청구 수정 확인"
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
        <h2 className="mb-2 text-xl font-bold">확정 청구 수정</h2>
        <p className="mb-4 text-base text-gray-700">
          <strong>{studentName}</strong> 원생의 확정된 청구 금액을 변경합니다.
        </p>

        <div className="mb-4 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm">
          현재 금액: <strong>{currentAmount.toLocaleString()}원</strong>
          <br />
          변경 금액: <strong>{newAmount.toLocaleString()}원</strong>
        </div>

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
            className="min-h-[44px] flex-1 rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? '저장 중...' : '변경'}
          </button>
        </div>
      </div>
    </div>
  )
}
