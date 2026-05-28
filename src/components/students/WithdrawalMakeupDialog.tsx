'use client'

/**
 * 퇴교 시 미사용 보강 처리 다이얼로그 — Sprint 10 T10 (PRD §4.5.9).
 *
 * 진입: `/students/edit` 의 퇴교 처리 흐름에서 잔여 보강이 있을 때 부모가 mount.
 *
 * 3가지 선택지:
 * - 즉시 소멸: `ImmediateExpire` IPC → 모든 미보강 결석 → makeup_expired + withdraw_date 설정
 * - 보강 후 퇴교: 다이얼로그 닫기 (IPC 호출 없음, PI-08 결정)
 * - 외부 처리 후 소멸: memo textarea + `ExternalExpire { memo }` IPC → 동일하게 처리
 *
 * 빈 리스트(absences === 0) 인 경우 부모는 본 다이얼로그를 mount 하지 않고 직접 `withdrawStudent` 호출.
 */

import { useMemo, useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { processWithdrawalMakeup } from '@/lib/tauri'
import { minutesToHoursText } from '@/lib/time'
import type { WithdrawalChoice, WithdrawalPendingMakeup } from '@/types/withdrawal'

interface Props {
  studentName: string
  withdrawDate: string
  pending: WithdrawalPendingMakeup
  onCompleted: () => void
  onCancel: () => void
}

type Mode = 'menu' | 'external'

export function WithdrawalMakeupDialog({
  studentName,
  withdrawDate,
  pending,
  onCompleted,
  onCancel,
}: Props) {
  const [mode, setMode] = useState<Mode>('menu')
  const [memo, setMemo] = useState('')
  const [error, setError] = useState<string | null>(null)

  const remainingHours = useMemo(
    () => minutesToHoursText(pending.remainingMinutes),
    [pending.remainingMinutes],
  )

  const mutation = useMutation({
    mutationFn: async (choice: WithdrawalChoice) => {
      await processWithdrawalMakeup(pending.studentId, choice, withdrawDate)
    },
    onSuccess: onCompleted,
    onError: (e) =>
      setError(e instanceof Error ? e.message : String(e)),
  })

  function handleImmediate() {
    setError(null)
    mutation.mutate({ type: 'immediate_expire' })
  }

  function handleExternal() {
    setError(null)
    const trimmed = memo.trim()
    if (trimmed.length === 0) {
      setError('외부 처리 사유 메모를 입력해 주세요.')
      return
    }
    mutation.mutate({ type: 'external_expire', memo: trimmed })
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="퇴교 시 미사용 보강 처리"
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-2xl rounded-lg bg-white p-6 shadow-xl">
        <h2 className="mb-2 text-xl font-bold">퇴교 시 미사용 보강 처리</h2>
        <p className="mb-4 text-base text-gray-700">
          <strong>{studentName}</strong> 원생에게 미보강 결석이{' '}
          <strong>{pending.absences.length}건</strong> ({remainingHours}시간) 남아
          있습니다. 처리 방식을 선택해 주세요.
        </p>

        <div className="mb-4 max-h-48 overflow-y-auto rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm">
          <table className="w-full text-left">
            <thead className="text-xs text-gray-600">
              <tr>
                <th className="pb-1">결석일</th>
                <th className="pb-1">수업 시간</th>
                <th className="pb-1">소멸기한</th>
              </tr>
            </thead>
            <tbody>
              {pending.absences.map((a) => (
                <tr key={a.id} className="border-t border-gray-200">
                  <td className="py-1">{a.eventDate}</td>
                  <td className="py-1">{minutesToHoursText(a.classMinutes)}시간</td>
                  <td className="py-1 text-gray-600">
                    {a.makeupDeadline ?? '미확정'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {error !== null && (
          <div
            role="alert"
            className="mb-3 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-sm text-[var(--danger)]"
          >
            {error}
          </div>
        )}

        {mode === 'menu' && (
          <div className="flex flex-col gap-2">
            <button
              type="button"
              onClick={handleImmediate}
              disabled={mutation.isPending}
              className="min-h-[44px] rounded-md bg-[var(--danger)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
            >
              즉시 소멸 (전체 보강소멸 + 퇴교)
            </button>
            <button
              type="button"
              onClick={onCancel}
              disabled={mutation.isPending}
              className="min-h-[44px] rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            >
              보강 후 퇴교 (지금 취소)
            </button>
            <button
              type="button"
              onClick={() => setMode('external')}
              disabled={mutation.isPending}
              className="min-h-[44px] rounded-md border-2 border-amber-400 px-4 text-base text-amber-900 hover:bg-amber-50 disabled:opacity-50"
            >
              외부 처리 후 소멸 (사유 메모 입력)
            </button>
          </div>
        )}

        {mode === 'external' && (
          <div className="flex flex-col gap-3">
            <label className="block text-sm font-medium text-gray-700">
              사유 메모 (필수)
              <textarea
                value={memo}
                onChange={(e) => setMemo(e.target.value)}
                rows={3}
                placeholder="예: 환불 처리 완료 (외부 정산)"
                className="mt-1 w-full rounded-md border-2 border-[var(--border)] px-3 py-2 text-base"
              />
            </label>
            <p className="text-xs text-gray-500">
              본 메모는 모든 미보강 결석 사유로 일괄 저장됩니다. 결석 이력 메뉴에서 확인 가능합니다.
            </p>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => setMode('menu')}
                disabled={mutation.isPending}
                className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
              >
                뒤로
              </button>
              <button
                type="button"
                onClick={handleExternal}
                disabled={mutation.isPending}
                className="min-h-[44px] flex-1 rounded-md bg-[var(--danger)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
              >
                {mutation.isPending ? '처리 중...' : '외부 처리 확정'}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
