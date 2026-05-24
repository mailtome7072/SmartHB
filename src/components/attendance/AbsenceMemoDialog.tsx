'use client'

/**
 * 결석 사유 메모 다이얼로그 — Sprint 8 T4 (PRD §4.5.3).
 *
 * 결석 셀에서 우클릭 진입. 멀티라인 textarea 1개. 빈 문자열 저장 시 NULL 로 환원.
 * ESC 또는 취소 버튼으로 닫기.
 */

import { useEffect, useState } from 'react'
import type { AttendanceCell } from '@/types/attendance'

interface Props {
  cell: AttendanceCell
  onSave: (memo: string | null) => void
  onClose: () => void
}

export function AbsenceMemoDialog({ cell, onSave, onClose }: Props) {
  const [memo, setMemo] = useState(cell.absenceMemo ?? '')

  // ESC 키로 닫기
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault()
        onClose()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="absence-memo-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="absence-memo-title" className="text-xl font-bold">
          결석 사유 메모
        </h2>
        <p className="mt-1 text-sm text-gray-600">
          {cell.eventDate} 결석에 대한 사유를 입력하세요. 비워두면 메모가 삭제됩니다.
        </p>

        <textarea
          value={memo}
          onChange={(e) => setMemo(e.target.value)}
          rows={5}
          autoFocus
          className="mt-4 w-full rounded-md border-2 border-[var(--border)] p-3 text-base focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          placeholder="예: 가족 행사로 인한 결석"
        />

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 text-base text-gray-700 hover:bg-gray-50"
          >
            취소
          </button>
          <button
            type="button"
            onClick={() => onSave(memo.trim().length === 0 ? null : memo.trim())}
            className="min-h-[44px] rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:bg-[var(--accent-hover)]"
          >
            저장
          </button>
        </div>
      </div>
    </div>
  )
}
