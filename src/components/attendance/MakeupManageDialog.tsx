'use client'

/**
 * 보강 관리 다이얼로그 — Sprint 9 T7 (PRD §4.5.6) + Session #10 J5/J6.
 *
 * 보강일 셀(J4 emerald) 클릭 시 진입. 보강 정보 표시 + "보강 삭제" 단일 액션.
 * - 삭제: `cancelMakeup` → 매칭된 결석을 absent 환원 + makeup_attendances DELETE
 *
 * Session #10 J5 (2026-05-25): 보강 미등원 개념 삭제 — 보강은 결과 기록.
 * Session #10 J6 (2026-05-25): 삭제 진입점을 결석일 셀 → 보강일 셀로 이동.
 *
 * 위험 동작 (삭제) 명시적 확인 다이얼로그 (PRD §5.7).
 */

import { useEffect, useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { cancelMakeup } from '@/lib/tauri'
import { minutesToHoursText } from '@/lib/time'

interface Props {
  makeupId: number
  studentName: string
  studentSerialNo: string
  /** 보강일자 (YYYY-MM-DD). */
  eventDate: string
  /** 보강 수업 시간 (분). UI 에서 시간 단위 변환. */
  classMinutes: number
  onClose: () => void
  onSuccess: () => void
}

type Mode = 'menu' | 'confirm-cancel'

export function MakeupManageDialog({
  makeupId,
  studentName,
  studentSerialNo,
  eventDate,
  classMinutes,
  onClose,
  onSuccess,
}: Props) {
  const [mode, setMode] = useState<Mode>('menu')
  const [error, setError] = useState<string | null>(null)

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
    mutationFn: () => cancelMakeup(makeupId),
    onSuccess: () => {
      setError(null)
      onSuccess()
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
    },
  })

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
          <span className="ml-1 text-muted-foreground">#{studentSerialNo}</span>
          <span className="mx-2">·</span>
          <span>
            {eventDate} 보강 ({minutesToHoursText(classMinutes)}시간)
          </span>
        </p>

        {mode === 'menu' && (
          <div className="mt-6 space-y-3">
            <p className="text-base text-gray-700">처리 방법을 선택하세요:</p>
            <button
              type="button"
              onClick={() => setMode('confirm-cancel')}
              className="block w-full min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 py-3 text-left text-base text-gray-700 hover:bg-gray-50"
            >
              <span className="font-semibold">보강 삭제</span>
              <span className="ml-2 text-sm text-muted-foreground">
                — 보강 기록 삭제 + 결석으로 환원
              </span>
            </button>
          </div>
        )}

        {mode === 'confirm-cancel' && (
          <ConfirmPanel
            title="보강 기록을 삭제하시겠습니까?"
            description="보강 기록이 삭제되고, 매칭된 결석은 다시 '미처리 결석' 상태로 환원됩니다."
            confirmLabel={cancelMutation.isPending ? '삭제 중...' : '보강 삭제'}
            onCancel={() => setMode('menu')}
            onConfirm={() => cancelMutation.mutate()}
            disabled={cancelMutation.isPending}
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
            disabled={cancelMutation.isPending}
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
  onCancel: () => void
  onConfirm: () => void
  disabled: boolean
}

function ConfirmPanel({
  title,
  description,
  confirmLabel,
  onCancel,
  onConfirm,
  disabled,
}: ConfirmPanelProps) {
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
          className="min-h-[44px] rounded-md bg-[var(--danger)] px-4 text-base font-semibold text-white hover:bg-red-700 disabled:opacity-50"
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  )
}
