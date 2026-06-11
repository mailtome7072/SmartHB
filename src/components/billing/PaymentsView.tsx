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
import { batchUpdatePayments, listCodes, listPaymentView } from '@/lib/tauri'
import { useUnsavedChanges } from '@/lib/use-unsaved-changes'
import type {
  BillingSearchResult,
  PaymentInput,
  PaymentViewRow,
} from '@/types/billing'
import type { CodeEntry } from '@/types/code'

interface Props {
  yearMonth: string
  onError: (msg: string) => void
  /** 통합 검색 매칭 학생 ID 집합. null = 검색 미적용. */
  matchedStudentIds: Set<number> | null
  /** 검색 결과 — 자동 채움(입금일=오늘 + 최근 결제수단/카드사/입금자) 에 사용. */
  searchResults: BillingSearchResult[]
  /** 수납 상태 필터 — 'all' / 'paid' / 'unpaid'. */
  paymentFilter: 'all' | 'paid' | 'unpaid'
  /** P0-4: 미저장 변경 건수 통지 — 부모가 탭/월 변경 가드에 사용. */
  onDirtyChange?: (count: number) => void
}

interface RowDraft {
  isPaid: boolean
  paidDate: string
  payerName: string
  paymentMethodId: number | null
  cardCompanyId: number | null
  /** 수납완료 행의 수납취소 예정 표시 — 저장 시 is_paid=false 로 되돌린다. */
  cancel: boolean
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
    cancel: false,
  }
}

export function PaymentsView({
  yearMonth,
  onError,
  matchedStudentIds,
  searchResults,
  paymentFilter,
  onDirtyChange,
}: Props) {
  const qc = useQueryClient()
  const viewQuery = useQuery({
    queryKey: ['payment-view', yearMonth],
    queryFn: () => listPaymentView(yearMonth),
  })
  const paymentMethodsQuery = useQuery({
    queryKey: ['codes', 'payment-methods'],
    queryFn: () => listCodes('payment-methods', 100, 0),
  })
  const cardCompaniesQuery = useQuery({
    queryKey: ['codes', 'card-companies'],
    queryFn: () => listCodes('card-companies', 100, 0),
  })

  // billId → 임시 입력 상태 (미수납 행만 편집 가능)
  const [drafts, setDrafts] = useState<Record<number, RowDraft>>({})

  // P0-4 (2026-06 코드리뷰): 데이터 갱신 시 drafts 를 전체 초기화하지 않는다 — 기존
  // `setDrafts({})` 는 창 포커스 복귀 등 백그라운드 refetch 만으로 입력 중인 수납 정보를
  // 통째로 소실시켰다. 현재 데이터와 어긋난 stale draft(사라진 청구·상태가 바뀐 행)만 정리.
  useEffect(() => {
    const data = viewQuery.data
    if (!data) return
    setDrafts((prev) => {
      const rowById = new Map(data.map((r) => [r.billId, r]))
      const next: Record<number, RowDraft> = {}
      let changed = false
      for (const [key, d] of Object.entries(prev)) {
        const row = rowById.get(Number(key))
        // 사라진 청구(월 변경 등) / 입력 종류와 행 상태가 어긋난 draft 는 폐기:
        // 미수납용 입력(d.cancel=false)은 미수납 행에만, 수납취소 예정은 수납완료 행에만 유효.
        if (row === undefined || row.isPaid !== d.cancel) {
          changed = true
          continue
        }
        next[Number(key)] = d
      }
      return changed ? next : prev
    })
  }, [viewQuery.data])

  const allRows: PaymentViewRow[] = viewQuery.data ?? []
  // 검색 + 수납 상태 필터 동시 적용.
  const rows: PaymentViewRow[] = allRows.filter((r) => {
    if (matchedStudentIds !== null && !matchedStudentIds.has(r.studentId)) return false
    if (paymentFilter === 'paid' && !r.isPaid) return false
    if (paymentFilter === 'unpaid' && r.isPaid) return false
    return true
  })
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

  // 검색 결과로 좁혀진 미납 행에 자동 채움 (입금일=오늘 + 최근 결제수단/카드사/입금자).
  // 사용자는 완료 체크 + 저장만 수행. 수납완료 행은 건드리지 않음.
  useEffect(() => {
    if (matchedStudentIds === null || searchResults.length === 0) return
    if (rows.length === 0) return
    const today = todayStr()
    const byStudent = new Map(searchResults.map((r) => [r.studentId, r]))
    setDrafts((prev) => {
      const next = { ...prev }
      let touched = false
      for (const r of rows) {
        if (r.isPaid) continue // 수납완료는 read-only
        if (prev[r.billId] !== undefined) continue
        const info = byStudent.get(r.studentId)
        if (!info) continue
        next[r.billId] = {
          isPaid: false,
          paidDate: today,
          payerName: info.latestPayerName ?? '',
          paymentMethodId: info.latestPaymentMethodId,
          cardCompanyId: info.latestCardCompanyId,
          cancel: false,
        }
        touched = true
      }
      return touched ? next : prev
    })
  }, [matchedStudentIds, searchResults, rows])

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

  // 수납완료 행의 수납취소 토글 (저장 전까지는 예정 상태).
  const toggleCancel = (billId: number, cancel: boolean) => {
    updateDraft(billId, { cancel })
  }

  // 변경 대상: 입금 체크 / 결제수단 선택 / 수납취소 예정 중 하나라도 있으면 dirty.
  // A70: 입금자명만 입력한 행도 변경 대상에 포함 (payerName-only 소실 방지).
  const dirtyEntries = useMemo(
    () =>
      Object.entries(drafts).filter(
        ([, d]) =>
          d.isPaid || d.paymentMethodId !== null || d.cancel || d.payerName.trim() !== '',
      ),
    [drafts],
  )

  // P0-4: 미저장 입력 보호 — 창 닫기·메뉴 이동 경고(공통 훅) + Ctrl+S 일괄 저장.
  // 탭/월 변경은 부모(payments/page)가 onDirtyChange 로 받아 가드한다.
  useUnsavedChanges(dirtyEntries.length > 0, () => handleSave())
  useEffect(() => {
    onDirtyChange?.(dirtyEntries.length)
    // unmount 시 dirty 해제 통지 — 부모 가드 잔존 방지.
    return () => onDirtyChange?.(0)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dirtyEntries.length])

  const batchMutation = useMutation({
    mutationFn: (items: PaymentInput[]) => batchUpdatePayments(items),
    onMutate: () => onError(''),
    onSuccess: () => {
      onError('')
      setDrafts({})
      qc.invalidateQueries({ queryKey: ['payment-view', yearMonth] })
      qc.invalidateQueries({ queryKey: ['billing-summary', yearMonth] })
    },
    onError: (e) => onError(e instanceof Error ? e.message : String(e)),
  })

  const handleSave = () => {
    if (dirtyEntries.length === 0 || batchMutation.isPending) return
    // #6: 입금 완료(취소 예정 아님) 인데 결제수단 미선택이면 저장 차단.
    const missingMethod = dirtyEntries.some(
      ([, d]) => !d.cancel && d.isPaid && d.paymentMethodId === null,
    )
    if (missingMethod) {
      onError('입금 완료 항목은 결제수단을 반드시 선택해 주세요.')
      return
    }
    const items: PaymentInput[] = dirtyEntries.map(([billId, d]) => {
      // 수납취소 예정 — is_paid=false 로 되돌리고 입금 정보 초기화.
      if (d.cancel) {
        return {
          billId: Number(billId),
          isPaid: false,
          paidDate: null,
          payerName: null,
          paymentMethodId: null,
          cardCompanyId: null,
        }
      }
      return {
        billId: Number(billId),
        isPaid: d.isPaid,
        paidDate: d.isPaid ? d.paidDate : null,
        payerName: d.payerName.trim() === '' ? null : d.payerName.trim(),
        paymentMethodId: d.paymentMethodId,
        cardCompanyId: d.cardCompanyId,
      }
    })
    batchMutation.mutate(items)
  }

  if (viewQuery.isLoading) return <p>불러오는 중...</p>
  if (rows.length === 0) {
    return (
      <div className="rounded-md border border-[var(--border)] bg-gray-50 p-6 text-center text-gray-600">
        해당 조건의 청구가 없습니다.
      </div>
    )
  }

  const paidCount = rows.filter((r) => r.isPaid).length
  const unpaidCount = rows.length - paidCount

  return (
    <>
      <div className="mb-3 flex items-center justify-between">
        <p className="text-base">
          청구 <strong>{rows.length}건</strong> · 수납완료 {paidCount} · 미수납 {unpaidCount}
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
            {rows.map((b) => {
              const d = getDraft(b.billId)
              const isCard = cardMethodId !== null && d.paymentMethodId === cardMethodId
              const rowBg = b.isPaid
                ? 'bg-emerald-50'
                : b.isMidMonth
                  ? 'bg-amber-50'
                  : ''
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
                    {b.isPaid && (
                      <span className="ml-1 rounded-full bg-emerald-200 px-1.5 py-0.5 text-xs text-emerald-900">
                        수납완료
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-2 text-right font-semibold">
                    {b.adjustedAmount.toLocaleString()}
                  </td>
                  <td className="px-3 py-2">
                    {b.isPaid ? (
                      d.cancel ? (
                        <div className="flex items-center gap-1">
                          <span className="text-sm font-medium text-[var(--danger)]">취소 예정</span>
                          <button
                            type="button"
                            onClick={() => toggleCancel(b.billId, false)}
                            className="h-8 rounded border border-[var(--border)] px-2 text-xs text-gray-700 hover:bg-gray-50"
                          >
                            되돌리기
                          </button>
                        </div>
                      ) : (
                        <div className="flex items-center gap-1">
                          <span className="text-emerald-700">✓</span>
                          <button
                            type="button"
                            onClick={() => toggleCancel(b.billId, true)}
                            className="h-8 rounded border border-[var(--danger)] px-2 text-xs text-[var(--danger)] hover:bg-red-50"
                          >
                            수납취소
                          </button>
                        </div>
                      )
                    ) : (
                      <input
                        type="checkbox"
                        checked={d.isPaid}
                        onChange={(e) => togglePaid(b.billId, e.target.checked)}
                        className="h-5 w-5"
                        aria-label="입금 완료"
                      />
                    )}
                  </td>
                  <td className="px-3 py-2">
                    {b.isPaid ? (
                      <span className="text-sm text-gray-700">{b.paidDate ?? '—'}</span>
                    ) : (
                      <input
                        type="date"
                        value={d.paidDate}
                        onChange={(e) => {
                          updateDraft(b.billId, { paidDate: e.target.value })
                          // 날짜 선택 시 달력을 닫고 다음 입력(입금자)으로 포커스 이동.
                          if (typeof document !== 'undefined') {
                            document.getElementById(`payer-${b.billId}`)?.focus()
                          }
                        }}
                        disabled={!d.isPaid}
                        className="h-9 w-36 rounded border border-[var(--border)] px-2 disabled:bg-gray-100"
                      />
                    )}
                  </td>
                  <td className="px-3 py-2">
                    {b.isPaid ? (
                      <span className="text-sm text-gray-700">{b.payerName ?? '—'}</span>
                    ) : (
                      <input
                        id={`payer-${b.billId}`}
                        type="text"
                        value={d.payerName}
                        onChange={(e) => updateDraft(b.billId, { payerName: e.target.value })}
                        placeholder="이름"
                        className="h-9 w-28 rounded border border-[var(--border)] px-2"
                      />
                    )}
                  </td>
                  <td className="px-3 py-2">
                    {b.isPaid ? (
                      <span className="text-sm text-gray-700">{b.paymentMethodLabel ?? '—'}</span>
                    ) : (
                      <select
                        value={d.paymentMethodId ?? ''}
                        onChange={(e) =>
                          updateDraft(b.billId, {
                            paymentMethodId:
                              e.target.value === '' ? null : Number(e.target.value),
                            cardCompanyId:
                              e.target.value === '' ||
                              Number(e.target.value) !== cardMethodId
                                ? null
                                : d.cardCompanyId,
                          })
                        }
                        className={`h-9 w-28 rounded border px-2 ${
                          d.isPaid && d.paymentMethodId === null
                            ? 'border-[var(--danger)]'
                            : 'border-[var(--border)]'
                        }`}
                        aria-invalid={d.isPaid && d.paymentMethodId === null ? 'true' : undefined}
                      >
                        <option value="">{d.isPaid ? '선택 (필수)' : '선택'}</option>
                        {paymentMethods.map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.label}
                          </option>
                        ))}
                      </select>
                    )}
                  </td>
                  <td className="px-3 py-2">
                    {b.isPaid ? (
                      <span className="text-sm text-gray-700">{b.cardCompanyLabel ?? '—'}</span>
                    ) : (
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
                    )}
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
