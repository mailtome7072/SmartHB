'use client'

/**
 * 수납 관리 뷰 — Sprint 11 T8 (PRD §4.9.5, §4.9.6, AC-4.9-4 / -6).
 *
 * 미납(payments 없음 또는 is_paid=0) 청구 목록을 한 화면(최소 20행)에 노출.
 * 각 행에서 입금 정보(완료 / 입금일 / 결제수단 / 카드사) 인라인 입력 후
 * "선택 일괄 저장" 으로 `batchUpdatePayments` 호출 (단일 트랜잭션).
 *
 * 카드 계열 결제수단(`code === 'card'`) 선택 시 카드사 select 활성.
 * 카드사 누락 시 백엔드 IPC 가 거부 (AC-4.9-4, 휴리스틱 안내는 클라이언트).
 */

import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { batchUpdatePayments, listCodes, listUnpaidBills } from '@/lib/tauri'
import type { PaymentInput, UnpaidBill } from '@/types/billing'
import type { CodeEntry } from '@/types/code'

interface Props {
  yearMonth: string
  onError: (msg: string) => void
}

interface RowDraft {
  isPaid: boolean
  paidDate: string
  payerName: string
  paymentMethodId: number | null
  cardCompanyId: number | null
}

const CARD_PAYMENT_CODE = 'card'

function todayStr(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

function emptyDraft(): RowDraft {
  return {
    isPaid: false,
    paidDate: '',
    payerName: '',
    paymentMethodId: null,
    cardCompanyId: null,
  }
}

export function PaymentsView({ yearMonth, onError }: Props) {
  const qc = useQueryClient()
  const unpaidQuery = useQuery({
    queryKey: ['unpaid-bills', yearMonth],
    queryFn: () => listUnpaidBills(yearMonth),
  })
  const paymentMethodsQuery = useQuery({
    queryKey: ['codes', 'payment-methods'],
    queryFn: () => listCodes('payment-methods', 100, 0),
  })
  const cardCompaniesQuery = useQuery({
    queryKey: ['codes', 'card-companies'],
    queryFn: () => listCodes('card-companies', 100, 0),
  })

  // billId → 임시 입력 상태
  const [drafts, setDrafts] = useState<Record<number, RowDraft>>({})

  // unpaid 목록 갱신 시 drafts 초기화
  useEffect(() => {
    if (unpaidQuery.data) {
      setDrafts({})
    }
  }, [unpaidQuery.data])

  const unpaid: UnpaidBill[] = unpaidQuery.data ?? []
  const paymentMethods: CodeEntry[] = (paymentMethodsQuery.data ?? []).filter(
    (c) => c.is_active,
  )
  const cardCompanies: CodeEntry[] = (cardCompaniesQuery.data ?? []).filter(
    (c) => c.is_active,
  )
  const cardMethodId = useMemo(
    () => paymentMethods.find((p) => p.code === CARD_PAYMENT_CODE)?.id ?? null,
    [paymentMethods],
  )

  const getDraft = (billId: number): RowDraft => drafts[billId] ?? emptyDraft()

  const updateDraft = (billId: number, patch: Partial<RowDraft>) => {
    setDrafts((prev) => ({
      ...prev,
      [billId]: { ...getDraft(billId), ...patch },
    }))
  }

  const togglePaid = (billId: number, checked: boolean) => {
    updateDraft(billId, {
      isPaid: checked,
      paidDate: checked ? getDraft(billId).paidDate || todayStr() : '',
    })
  }

  const dirtyEntries = useMemo(
    () => Object.entries(drafts).filter(([, d]) => d.isPaid || d.paymentMethodId !== null),
    [drafts],
  )

  const batchMutation = useMutation({
    mutationFn: (items: PaymentInput[]) => batchUpdatePayments(items),
    onSuccess: () => {
      onError('')
      setDrafts({})
      qc.invalidateQueries({ queryKey: ['unpaid-bills', yearMonth] })
      qc.invalidateQueries({ queryKey: ['billing-summary', yearMonth] })
    },
    onError: (e) => onError(e instanceof Error ? e.message : String(e)),
  })

  const handleSave = () => {
    const items: PaymentInput[] = dirtyEntries.map(([billId, d]) => ({
      billId: Number(billId),
      isPaid: d.isPaid,
      paidDate: d.isPaid ? d.paidDate : null,
      payerName: d.payerName.trim() === '' ? null : d.payerName.trim(),
      paymentMethodId: d.paymentMethodId,
      cardCompanyId: d.cardCompanyId,
    }))
    batchMutation.mutate(items)
  }

  if (unpaidQuery.isLoading) return <p>불러오는 중...</p>
  if (unpaid.length === 0) {
    return (
      <div className="rounded-md border border-[var(--border)] bg-gray-50 p-6 text-center text-gray-600">
        미납 청구가 없습니다.
      </div>
    )
  }

  return (
    <>
      <div className="mb-3 flex items-center justify-between">
        <p className="text-base">
          미납 청구 <strong>{unpaid.length}건</strong>
          {dirtyEntries.length > 0 && (
            <span className="ml-2 text-sm text-amber-700">
              · 변경 {dirtyEntries.length}건
            </span>
          )}
        </p>
        <button
          type="button"
          onClick={handleSave}
          disabled={dirtyEntries.length === 0 || batchMutation.isPending}
          className="h-11 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
        >
          {batchMutation.isPending ? '저장 중...' : `선택 일괄 저장 (${dirtyEntries.length})`}
        </button>
      </div>

      {/* AC-4.9-6: 한 화면 최소 20행 — max-h 로 스크롤 가능 (20행 분량 ≈ 800px) */}
      <div className="max-h-[800px] overflow-y-auto rounded-md border border-[var(--border)]">
        <table className="w-full text-base">
          <thead className="sticky top-0 bg-gray-100 text-left">
            <tr>
              <th className="px-3 py-2">번호</th>
              <th className="px-3 py-2">원생명</th>
              <th className="px-3 py-2 text-right">청구액</th>
              <th className="px-3 py-2">완료</th>
              <th className="px-3 py-2">입금일</th>
              <th className="px-3 py-2">입금자</th>
              <th className="px-3 py-2">결제수단</th>
              <th className="px-3 py-2">카드사</th>
            </tr>
          </thead>
          <tbody>
            {unpaid.map((b) => {
              const d = getDraft(b.billId)
              const isCard = cardMethodId !== null && d.paymentMethodId === cardMethodId
              const rowBg = b.isMidMonth ? 'bg-amber-50' : ''
              return (
                <tr key={b.billId} className={`border-t border-[var(--border)] ${rowBg}`}>
                  <td className="px-3 py-2">{b.studentSerialNo}</td>
                  <td className="px-3 py-2 font-medium">
                    {b.studentName}
                    {b.midMonthType !== null && (
                      <span className="ml-1 text-xs text-amber-900">
                        ({b.midMonthType === 'enrolled' ? '월중입교' : '월중퇴교'})
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-2 text-right font-semibold">
                    {b.adjustedAmount.toLocaleString()}
                  </td>
                  <td className="px-3 py-2">
                    <input
                      type="checkbox"
                      checked={d.isPaid}
                      onChange={(e) => togglePaid(b.billId, e.target.checked)}
                      className="h-5 w-5"
                      aria-label="입금 완료"
                    />
                  </td>
                  <td className="px-3 py-2">
                    <input
                      type="date"
                      value={d.paidDate}
                      onChange={(e) => updateDraft(b.billId, { paidDate: e.target.value })}
                      disabled={!d.isPaid}
                      className="h-9 w-36 rounded border border-[var(--border)] px-2 disabled:bg-gray-100"
                    />
                  </td>
                  <td className="px-3 py-2">
                    <input
                      type="text"
                      value={d.payerName}
                      onChange={(e) => updateDraft(b.billId, { payerName: e.target.value })}
                      placeholder="이름"
                      className="h-9 w-28 rounded border border-[var(--border)] px-2"
                    />
                  </td>
                  <td className="px-3 py-2">
                    <select
                      value={d.paymentMethodId ?? ''}
                      onChange={(e) =>
                        updateDraft(b.billId, {
                          paymentMethodId: e.target.value === '' ? null : Number(e.target.value),
                          cardCompanyId:
                            e.target.value === '' || Number(e.target.value) !== cardMethodId
                              ? null
                              : d.cardCompanyId,
                        })
                      }
                      className="h-9 w-28 rounded border border-[var(--border)] px-2"
                    >
                      <option value="">선택</option>
                      {paymentMethods.map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.label}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td className="px-3 py-2">
                    <select
                      value={d.cardCompanyId ?? ''}
                      onChange={(e) =>
                        updateDraft(b.billId, {
                          cardCompanyId: e.target.value === '' ? null : Number(e.target.value),
                        })
                      }
                      disabled={!isCard}
                      className={`h-9 w-32 rounded border px-2 disabled:bg-gray-100 ${
                        isCard && d.cardCompanyId === null
                          ? 'border-[var(--danger)]'
                          : 'border-[var(--border)]'
                      }`}
                      aria-invalid={isCard && d.cardCompanyId === null ? 'true' : undefined}
                    >
                      <option value="">{isCard ? '카드사 선택 (필수)' : '—'}</option>
                      {cardCompanies.map((c) => (
                        <option key={c.id} value={c.id}>
                          {c.label}
                        </option>
                      ))}
                    </select>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </>
  )
}
