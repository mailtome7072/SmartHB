'use client'

/**
 * 청구 목록 그리드 — Sprint 11 T7 (PRD §4.9, AC-4.9-2/3/4).
 *
 * 정렬: 백엔드 `list_bills` 가 이미 (status, isMidMonth, name) 으로 정렬해 반환.
 *
 * 행 동작:
 * - draft: 조정액 인라인 편집 + "확정" 버튼
 * - confirmed: 조정액 수정 시 [[ConfirmBillUpdateDialog]] 통한 확인
 * - 수납완료(isPaid) 청구: 금액 편집 불가 (이미 수금 완료)
 *
 * 월중입퇴교 시각 구분 (AC-4.9-2): 행 배경 + 라벨.
 */

import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { confirmBill, updateBill } from '@/lib/tauri'
import { ConfirmBillUpdateDialog } from './ConfirmBillUpdateDialog'
import type { Bill } from '@/types/billing'
import type { SchoolLevel } from '@/types/student'
import { compareKorean, useTableSort, withTiebreak } from '@/hooks/useTableSort'

interface Props {
  bills: Bill[]
  yearMonth: string
  onError: (msg: string) => void
}

const STATUS_LABEL: Record<Bill['status'], string> = {
  draft: '미확정',
  confirmed: '확정',
}

const MID_MONTH_LABEL: Record<NonNullable<Bill['midMonthType']>, string> = {
  enrolled: '월중입교',
  withdrawn: '월중퇴교',
}

/**
 * 정렬 가능 컬럼 (Sprint 19 T3, 사용자 요청 2번).
 *
 * 기본('default')은 백엔드가 반환한 순서(미확정→확정, 월중입퇴교 우선, 그 안에서
 * 학년별+이름) 그대로 유지 — 청구 확정 워크플로우 그룹핑은 업무상 우선순위라
 * 컬럼 클릭 전에는 건드리지 않는다. 컬럼 클릭 시에는 그 기준으로 전체 재정렬.
 */
type BillingSortKey = 'default' | 'name' | 'grade' | 'billAmount' | 'adjustedAmount' | 'status'

const SCHOOL_LEVEL_ORDER: Record<SchoolLevel, number> = {
  elementary: 0,
  middle: 1,
}

const nameTiebreak = (a: Bill, b: Bill) => compareKorean(a.studentName, b.studentName)

const BILLING_SORT_COMPARATORS: Record<BillingSortKey, (a: Bill, b: Bill) => number> = {
  default: () => 0, // Array.sort는 stable — 백엔드 원본 순서 그대로 유지
  name: (a, b) => compareKorean(a.studentName, b.studentName),
  grade: withTiebreak(
    (a, b) =>
      SCHOOL_LEVEL_ORDER[a.studentSchoolLevel] - SCHOOL_LEVEL_ORDER[b.studentSchoolLevel] ||
      a.studentGrade - b.studentGrade,
    nameTiebreak,
  ),
  billAmount: withTiebreak((a, b) => a.billAmount - b.billAmount, nameTiebreak),
  adjustedAmount: withTiebreak((a, b) => a.adjustedAmount - b.adjustedAmount, nameTiebreak),
  status: withTiebreak((a, b) => a.status.localeCompare(b.status), nameTiebreak),
}

export function BillingGrid({ bills, yearMonth, onError }: Props) {
  const qc = useQueryClient()
  const { sorted: sortedBills, toggleSort, indicator } = useTableSort<Bill, BillingSortKey>(
    bills,
    BILLING_SORT_COMPARATORS,
    { key: 'default', direction: 'asc' },
  )
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
    mutationFn: ({ id, amount }: { id: number; amount: number }) => updateBill(id, amount),
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
      updateMutation.mutate({ id: bill.id, amount: parsed })
    } else {
      // confirmed — 확인 다이얼로그 경유
      setPendingUpdate({ bill, newAmount: parsed })
    }
  }

  return (
    // 사용자 요청 — 청구/수납 그리드 모두 좌우+상하 스크롤 가능하도록(출결관리와 동일 패턴).
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-0 flex-1 overflow-auto rounded-md border border-[var(--border)]">
        <table className="w-full text-base">
          <thead className="sticky top-0 z-10 bg-gray-100 text-left">
            <tr>
              <th className="px-3 py-2">번호</th>
              <th className="px-3 py-2">
                <button
                  type="button"
                  onClick={() => toggleSort('name')}
                  className="hover:text-[var(--accent)]"
                  aria-label="원생명 정렬 토글"
                >
                  원생명{indicator('name')}
                </button>
              </th>
              <th className="px-3 py-2">
                <button
                  type="button"
                  onClick={() => toggleSort('grade')}
                  className="hover:text-[var(--accent)]"
                  aria-label="학년 정렬 토글"
                >
                  학년{indicator('grade')}
                </button>
              </th>
              <th className="px-3 py-2 text-right">주 시간</th>
              <th className="px-3 py-2 text-right">
                <button
                  type="button"
                  onClick={() => toggleSort('billAmount')}
                  className="hover:text-[var(--accent)]"
                  aria-label="표준 금액 정렬 토글"
                >
                  표준{indicator('billAmount')}
                </button>
              </th>
              <th className="px-3 py-2 text-right">
                <button
                  type="button"
                  onClick={() => toggleSort('adjustedAmount')}
                  className="hover:text-[var(--accent)]"
                  aria-label="조정 금액 정렬 토글"
                >
                  조정{indicator('adjustedAmount')}
                </button>
              </th>
              <th className="px-3 py-2">
                <button
                  type="button"
                  onClick={() => toggleSort('status')}
                  className="hover:text-[var(--accent)]"
                  aria-label="상태 정렬 토글"
                >
                  상태{indicator('status')}
                </button>
              </th>
              <th className="px-3 py-2">구분</th>
              <th className="px-3 py-2 text-right">작업</th>
            </tr>
          </thead>
          <tbody>
            {sortedBills.map((b) => {
              const isEditing = editingId === b.id
              const rowBg = b.isMidMonth ? 'bg-amber-50' : ''
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
                    ) : b.isPaid ? (
                      // 수납완료된 청구는 수정 불가 (금액 편집 비활성).
                      <span
                        className="font-semibold text-muted-foreground"
                        title="수납완료된 청구는 수정할 수 없습니다."
                      >
                        {b.adjustedAmount.toLocaleString()}
                      </span>
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
                          : 'bg-blue-100 text-blue-900'
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
            })
          }}
          onCancel={() => setPendingUpdate(null)}
        />
      )}
    </div>
  )
}
