'use client'

/**
 * 당월 청구 마감 확인 다이얼로그 — Sprint 11 T5 (PRD §4.9.7, AC-4.9-7, PI-11 reopen 불가).
 *
 * "당월 청구 마감" 버튼 클릭 시 강한 확인 다이얼로그.
 * - 마감 전제: 해당 월 모든 청구가 `confirmed` 상태 (백엔드 IPC 가 검증)
 * - PI-11 확정: 마감 해제 불가 — 본 다이얼로그에 명시 경고
 * - 마감 후 수정 시 사유 입력 필요 안내
 */

import { useState } from 'react'

interface Props {
  open: boolean
  yearMonth: string
  confirmedCount: number
  totalBilled: number
  onConfirm: () => Promise<void>
  onCancel: () => void
}

export function CloseMonthDialog({
  open,
  yearMonth,
  confirmedCount,
  totalBilled,
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
      aria-label="당월 청구 마감 확인"
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
        <h2 className="mb-2 text-xl font-bold">당월 청구 마감</h2>
        <p className="mb-4 text-base text-gray-700">
          <strong>{yearMonth}</strong> 월 청구를 마감합니다.
        </p>

        <div className="mb-4 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm">
          마감 대상 청구: <strong>{confirmedCount}건</strong>
          <br />
          청구 총액: <strong>{totalBilled.toLocaleString()}원</strong>
        </div>

        <div className="mb-4 rounded-md border-2 border-amber-400 bg-amber-50 p-3 text-sm text-amber-900">
          <p className="mb-1 font-semibold">⚠ 마감 후 주의사항</p>
          <ul className="list-disc pl-5 leading-relaxed">
            <li>마감된 청구는 수정 시 <strong>사유 입력이 필요</strong>합니다 (감사 로그 기록).</li>
            <li><strong>마감 해제는 불가</strong>합니다 (PI-11 확정 정책).</li>
            <li>미확정(draft) 청구가 남아 있으면 마감되지 않습니다.</li>
          </ul>
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
            className="min-h-[44px] flex-1 rounded-md bg-[var(--danger)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? '마감 중...' : '마감 확정'}
          </button>
        </div>
      </div>
    </div>
  )
}
