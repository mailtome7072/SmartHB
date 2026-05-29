'use client'

/**
 * 청구 목록 그리드 — Sprint 11 T7 (PRD §4.9, AC-4.9-2/3/4).
 *
 * 정렬: 백엔드 `list_bills` 가 이미 (status, isMidMonth, name) 으로 정렬해 반환.
 *
 * 행 동작:
 * - draft: 조정액 인라인 편집 + "확정" 버튼
 * - confirmed: 조정액 수정 시 [[ConfirmBillUpdateDialog]] 통한 확인
 * - closed: 조정액 수정 시 [[CloseReasonDialog]] 통한 사유 입력 (AC-4.9-8)
 *
 * 월중입퇴교 시각 구분 (AC-4.9-2): 행 배경 + 라벨.
 */

import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { confirmBill, updateBill } from '@/lib/tauri'
import { CloseReasonDialog } from './CloseReasonDialog'
import { ConfirmBillUpdateDialog } from './ConfirmBillUpdateDialog'
import type { Bill } from '@/types/billing'

interface Props {
  bills: Bill[]
  yearMonth: string
  onError: (msg: string) => void
}

const STATUS_LABEL: Record<Bill['status'], string> = {
  draft: '미확정',
  confirmed: '확정',
  closed: '마감',
}

const MID_MONTH_LABEL: Record<NonNullable<Bill['midMonthType']>, string> = {
  enrolled: '월중입교',
  withdrawn: '월중퇴교',
}

export function BillingGrid({ bills, yearMonth, onError }: Props) {
  const qc = useQueryClient()
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editValue, setEditValue] = useState('')
  const [pendingUpdate, setPendingUpdate] = useState<{
    bill: Bill
    newAmount: number
  } | null>(null)

  const invalidate = () => {
    qc.invalidateQueries({ queryKey: ['bills', yearMonth] })
    qc.invalidateQueries({ queryKey: ['billing-summary', yearMonth] })
  }

  const confirmMutation = useMutation({
    mutationFn: (id: number) => confirmBill(id),
    onMutate: () => onError(''),
    onSuccess: () => {
      invalidate()
      onError('')
    },
    onError: (e) => onError(e instanceof Error ? e.message : String(e)),
  })

  const updateMutation = useMutation({
    mutationFn: ({
      id,
      amount,
      reason,
    }: {
      id: number
      amount: number
      reason: string | null
    }) => updateBill(id, amount, reason),
    onMutate: () => onError(''),
    onSuccess: () => {
      invalidate()
      setEditingId(null)
      setEditValue('')
      setPendingUpdate(null)
      onError('')
    },
    onError: (e) => onError(e instanceof Error ? e.message : String(e)),
  })

  const startEdit = (bill: Bill) => {
    setEditingId(bill.id)
    setEditValue(String(bill.adjustedAmount))
    onError('')
  }

  const cancelEdit = () => {
    setEditingId(null)
    setEditValue('')
  }

  const tryCommit = (bill: Bill) => {
    const parsed = Number(editValue.replace(/,/g, ''))
    if (!Number.isFinite(parsed) || parsed < 0) {
      onError('조정 금액은 0 이상의 숫자여야 합니다.')
      return
    }
    if (parsed === bill.adjustedAmount) {
      cancelEdit()
      return
    }
    if (bill.status === 'draft') {
      updateMutation.mutate({ id: bill.id, amount: parsed, reason: null })
    } else {
      // confirmed / closed — 다이얼로그 경유
      setPendingUpdate({ bill, newAmount: parsed })
    }
  }

  return (
    <>
      <div className="overflow-x-auto rounded-md border border-[var(--border)]">
        <table className="w-full text-base">
          <thead className="bg-gray-100 text-left">
            <tr>
              <th className="px-3 py-2">번호</th>
              <th className="px-3 py-2">원생명</th>
              <th className="px-3 py-2">학년</th>
              <th className="px-3 py-2 text-right">주 시간</th>
              <th className="px-3 py-2 text-right">표준</th>
              <th className="px-3 py-2 text-right">조정</th>
              <th className="px-3 py-2">상태</th>
              <th className="px-3 py-2">구분</th>
              <th className="px-3 py-2 text-right">작업</th>
            </tr>
          </thead>
          <tbody>
            {bills.map((b) => {
              const isEditing = editingId === b.id
              const rowBg = b.isMidMonth
                ? 'bg-amber-50'
                : b.status === 'closed'
                  ? 'bg-gray-50'
                  : ''
              return (
                <tr key={b.id} className={`border-t border-[var(--border)] ${rowBg}`}>
                  <td className="px-3 py-2">{b.studentSerialNo}</td>
                  <td className="px-3 py-2 font-medium">{b.studentName}</td>
                  <td className="px-3 py-2">
                    {b.studentSchoolLevel === 'elementary' ? '초' : '중'}
                    {b.studentGrade}
                  </td>
                  <td className="px-3 py-2 text-right">{b.weeklyHours}</td>
                  <td className="px-3 py-2 text-right text-gray-600">
                    {b.billAmount.toLocaleString()}
                  </td>
                  <td className="px-3 py-2 text-right">
                    {isEditing ? (
                      <input
                        type="text"
                        inputMode="numeric"
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.nativeEvent.isComposing) return
                          if (
                            e.key === 'Enter' ||
                            e.code === 'Enter' ||
                            e.code === 'NumpadEnter'
                          ) {
                            tryCommit(b)
                          } else if (e.key === 'Escape') {
                            cancelEdit()
                          }
                        }}
                        autoFocus
                        className="h-9 w-28 rounded-md border-2 border-[var(--accent)] px-2 text-right"
                      />
                    ) : (
                      <button
                        type="button"
                        onClick={() => startEdit(b)}
                        className="font-semibold hover:underline disabled:opacity-50"
                      >
                        {b.adjustedAmount.toLocaleString()}
                      </button>
                    )}
                  </td>
                  <td className="px-3 py-2">
                    <span
                      className={`rounded-full px-2 py-0.5 text-sm ${
                        b.status === 'draft'
                          ? 'bg-amber-100 text-amber-900'
                          : b.status === 'confirmed'
                            ? 'bg-blue-100 text-blue-900'
                            : 'bg-gray-200 text-gray-700'
                      }`}
                    >
                      {STATUS_LABEL[b.status]}
                    </span>
                    {b.isPaid && (
                      <span className="ml-1 rounded-full bg-emerald-100 px-2 py-0.5 text-sm text-emerald-900">
                        수납완료
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-2 text-sm text-amber-900">
                    {b.midMonthType !== null ? MID_MONTH_LABEL[b.midMonthType] : ''}
                  </td>
                  <td className="px-3 py-2 text-right">
                    {isEditing ? (
                      <div className="flex justify-end gap-1">
                        <button
                          type="button"
                          onClick={() => tryCommit(b)}
                          className="h-9 rounded border border-[var(--accent)] bg-[var(--accent)] px-3 text-sm text-white hover:opacity-90"
                        >
                          저장
                        </button>
                        <button
                          type="button"
                          onClick={cancelEdit}
                          className="h-9 rounded border border-[var(--border)] px-3 text-sm text-gray-700 hover:bg-gray-50"
                        >
                          취소
                        </button>
                      </div>
                    ) : b.status === 'draft' ? (
                      <button
                        type="button"
                        onClick={() => confirmMutation.mutate(b.id)}
                        disabled={confirmMutation.isPending}
                        className="h-9 rounded border border-[var(--accent)] px-3 text-sm text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
                      >
                        확정
                      </button>
                    ) : null}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {pendingUpdate && pendingUpdate.bill.status === 'confirmed' && (
        <ConfirmBillUpdateDialog
          open
          studentName={pendingUpdate.bill.studentName}
          currentAmount={pendingUpdate.bill.adjustedAmount}
          newAmount={pendingUpdate.newAmount}
          onConfirm={async () => {
            await updateMutation.mutateAsync({
              id: pendingUpdate.bill.id,
              amount: pendingUpdate.newAmount,
              reason: null,
            })
          }}
          onCancel={() => setPendingUpdate(null)}
        />
      )}

      {pendingUpdate && pendingUpdate.bill.status === 'closed' && (
        <CloseReasonDialog
          open
          studentName={pendingUpdate.bill.studentName}
          currentAmount={pendingUpdate.bill.adjustedAmount}
          newAmount={pendingUpdate.newAmount}
          onConfirm={async (reason) => {
            await updateMutation.mutateAsync({
              id: pendingUpdate.bill.id,
              amount: pendingUpdate.newAmount,
              reason,
            })
          }}
          onCancel={() => setPendingUpdate(null)}
        />
      )}
    </>
  )
}
