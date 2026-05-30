'use client'

/**
 * 마감 후 청구 수정 사유 모달 — Sprint 11 T5 (PRD §4.9.7, AC-4.9-8, PI-10 모달 확정).
 *
 * 마감(`closed`) 상태 청구를 수정할 때 자동 팝업.
 * 사유 입력(최소 10자) 후 확정 시 부모가 받은 `onConfirm(reason)` 으로 IPC 호출.
 *
 * 다이얼로그 패턴은 Sprint 10 `WithdrawalMakeupDialog` 와 동일 (`fixed inset-0 z-[60]`).
 */

import { useState } from 'react'

// 백엔드 update_bill 요건과 정합: 공백 제외 1자 이상이면 충분 (AC-4.9-8 "사유 입력 필수").
// 과거 10자 게이트는 백엔드보다 과하게 엄격해 확정 버튼이 비활성으로 남는 문제가 있었다.
const MIN_REASON_LENGTH = 1

interface Props {
  open: boolean
  studentName: string
  currentAmount: number
  newAmount: number
  onConfirm: (reason: string) => Promise<void>
  onCancel: () => void
}

export function CloseReasonDialog({
  open,
  studentName,
  currentAmount,
  newAmount,
  onConfirm,
  onCancel,
}: Props) {
  const [reason, setReason] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  if (!open) return null

  const trimmed = reason.trim()
  const reasonValid = trimmed.length >= MIN_REASON_LENGTH

  const handleConfirm = async () => {
    if (!reasonValid) {
      setError('수정 사유를 입력해 주세요.')
      return
    }
    setSubmitting(true)
    setError(null)
    try {
      await onConfirm(trimmed)
      setReason('')
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSubmitting(false)
    }
  }

  const handleCancel = () => {
    setReason('')
    setError(null)
    onCancel()
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="마감 후 청구 수정 사유"
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
        <h2 className="mb-2 text-xl font-bold">마감 후 청구 수정</h2>
        <p className="mb-4 text-base text-gray-700">
          <strong>{studentName}</strong> 원생의 마감된 청구를 수정합니다. 수정 사유는 감사 로그에
          기록됩니다 (PRD §4.9.7).
        </p>

        <div className="mb-3 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm">
          현재 금액: <strong>{currentAmount.toLocaleString()}원</strong>
          <br />
          변경 금액: <strong>{newAmount.toLocaleString()}원</strong>
        </div>

        <label className="mb-3 block text-sm font-medium text-gray-700">
          수정 사유 (필수)
          <textarea
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            rows={3}
            placeholder="예: 부분 환불 합의에 따른 차액 조정"
            className="mt-1 w-full rounded-md border-2 border-[var(--border)] px-3 py-2 text-base"
            disabled={submitting}
          />
        </label>

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
            onClick={handleCancel}
            disabled={submitting}
            className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            취소
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            disabled={!reasonValid || submitting}
            className="min-h-[44px] flex-1 rounded-md bg-[var(--danger)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? '저장 중...' : '확정'}
          </button>
        </div>
      </div>
    </div>
  )
}
