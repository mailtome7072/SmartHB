'use client'

/**
 * 보강 관리 다이얼로그 — Sprint 9 T7 (PRD §4.5.6).
 *
 * `makeup_done` 셀 클릭 시 진입. 보강 정보 표시 + "취소" / "미등원" 2가지 액션:
 * - 취소: `cancelMakeup` → 매칭된 결석을 absent 환원 + makeup_attendances DELETE
 * - 미등원: `markMakeupAbsent` → 보강 상태 'makeup_absent' 마킹 + 결석 absent 환원 (재매칭 가능)
 *
 * 각 액션은 명시적 확인 다이얼로그 (CLAUDE.md: 위험 동작 확인 필수, PRD §5.7).
 */

import { useEffect, useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { cancelMakeup, markMakeupAbsent } from '@/lib/tauri'
import type { AttendanceCell } from '@/types/attendance'

interface Props {
  cell: AttendanceCell
  studentName: string
  studentSerialNo: string
  onClose: () => void
  onSuccess: () => void
}

type Mode = 'menu' | 'confirm-cancel' | 'confirm-absent'

export function MakeupManageDialog({
  cell,
  studentName,
  studentSerialNo,
  onClose,
  onSuccess,
}: Props) {
  const [mode, setMode] = useState<Mode>('menu')
  const [error, setError] = useState<string | null>(null)

  const makeupId = cell.makeupAttendanceId

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

  const cancelMutation = useMutation({
    mutationFn: () => {
      if (makeupId === null) {
        throw new Error('보강 정보가 없습니다.')
      }
      return cancelMakeup(makeupId)
    },
    onSuccess: () => {
      setError(null)
      onSuccess()
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
    },
  })

  const absentMutation = useMutation({
    mutationFn: () => {
      if (makeupId === null) {
        throw new Error('보강 정보가 없습니다.')
      }
      return markMakeupAbsent(makeupId)
    },
    onSuccess: () => {
      setError(null)
      onSuccess()
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
    },
  })

  const isPending = cancelMutation.isPending || absentMutation.isPending

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="makeup-manage-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="makeup-manage-title" className="text-xl font-bold">
          보강 관리
        </h2>
        <p className="mt-1 text-sm text-gray-600">
          <span className="font-semibold">{studentName}</span>
          <span className="ml-1 text-gray-500">#{studentSerialNo}</span>
          <span className="mx-2">·</span>
          <span>{cell.eventDate} 결석에 매칭된 보강</span>
        </p>

        {makeupId === null && (
          <p
            role="alert"
            className="mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            보강 정보가 없습니다 (출결 데이터 새로고침이 필요할 수 있습니다).
          </p>
        )}

        {makeupId !== null && mode === 'menu' && (
          <div className="mt-6 space-y-3">
            <p className="text-base text-gray-700">처리 방법을 선택하세요:</p>
            <button
              type="button"
              onClick={() => setMode('confirm-cancel')}
              className="block w-full min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 py-3 text-left text-base text-gray-700 hover:bg-gray-50"
            >
              <span className="font-semibold">보강 약속 취소</span>
              <span className="ml-2 text-sm text-gray-500">
                — 보강 기록 삭제 + 결석으로 환원
              </span>
            </button>
            <button
              type="button"
              onClick={() => setMode('confirm-absent')}
              className="block w-full min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 py-3 text-left text-base text-gray-700 hover:bg-gray-50"
            >
              <span className="font-semibold">보강 미등원 (보강결석)</span>
              <span className="ml-2 text-sm text-gray-500">
                — 보강 기록 유지(미등원 마킹) + 결석 재매칭 가능
              </span>
            </button>
          </div>
        )}

        {mode === 'confirm-cancel' && makeupId !== null && (
          <ConfirmPanel
            title="보강 약속을 취소하시겠습니까?"
            description={`보강 기록이 삭제되고, 매칭된 결석은 다시 '미처리 결석' 상태로 환원됩니다.`}
            confirmLabel={cancelMutation.isPending ? '취소 중...' : '보강 취소'}
            isDanger
            onCancel={() => setMode('menu')}
            onConfirm={() => cancelMutation.mutate()}
            disabled={isPending}
          />
        )}

        {mode === 'confirm-absent' && makeupId !== null && (
          <ConfirmPanel
            title="이 보강을 미등원으로 처리하시겠습니까?"
            description="보강 기록은 '미등원(보강결석)' 으로 마킹되어 보존되며, 결석은 다시 '미처리 결석' 상태로 환원됩니다 (다음 보강 매칭 가능)."
            confirmLabel={absentMutation.isPending ? '처리 중...' : '미등원 처리'}
            onCancel={() => setMode('menu')}
            onConfirm={() => absentMutation.mutate()}
            disabled={isPending}
          />
        )}

        {error !== null && (
          <p
            role="alert"
            className="mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        <div className="mt-6 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            disabled={isPending}
            className="min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            닫기
          </button>
        </div>
      </div>
    </div>
  )
}

interface ConfirmPanelProps {
  title: string
  description: string
  confirmLabel: string
  isDanger?: boolean
  onCancel: () => void
  onConfirm: () => void
  disabled: boolean
}

function ConfirmPanel({
  title,
  description,
  confirmLabel,
  isDanger,
  onCancel,
  onConfirm,
  disabled,
}: ConfirmPanelProps) {
  const confirmClass = isDanger === true
    ? 'bg-[var(--danger)] hover:bg-red-700'
    : 'bg-[var(--accent)] hover:bg-[var(--accent-hover)]'
  return (
    <div className="mt-6 rounded-md border-2 border-amber-300 bg-amber-50 p-4">
      <p className="text-base font-semibold text-gray-800">{title}</p>
      <p className="mt-2 text-sm text-gray-700">{description}</p>
      <div className="mt-4 flex justify-end gap-2">
        <button
          type="button"
          onClick={onCancel}
          disabled={disabled}
          className="min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
        >
          뒤로
        </button>
        <button
          type="button"
          onClick={onConfirm}
          disabled={disabled}
          className={`min-h-[44px] rounded-md px-4 text-base font-semibold text-white disabled:opacity-50 ${confirmClass}`}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  )
}
