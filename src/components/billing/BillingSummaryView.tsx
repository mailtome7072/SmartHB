'use client'

/**
 * 월별 집계 뷰 — 청구/수납 현황 요약 (PRD §4.11.3 대시보드 위젯 선행).
 *
 * 기간 선택:
 *  - 년/월 체크박스(상호 배타) — '년' 선택 시 연도 단위 집계, '월' 선택 시 년월 단위 집계
 *  - period = 'YYYY'(연도) 또는 'YYYY-MM'(월) → `getBillingPeriodStats`
 *
 * 표시:
 *  - 상단 요약 박스: 청구 건수 / 청구 총액 / 수납 건수 / 총 수납금액 / 미납
 *  - 결제수단별 수납 총액: 결제수단을 열(가로)로 배치 (is_paid=1 한정)
 *
 * 캐싱: TanStack Query `['billing-period-stats', period]`.
 */

import { useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { getBillingPeriodStats, listBilledMonths } from '@/lib/tauri'

interface Props {
  /** 상단 청구년월(YYYY-MM) — 기본 선택값 시드용. */
  defaultYearMonth: string
}

type Mode = 'month' | 'year'

function won(n: number): string {
  return `${n.toLocaleString()}원`
}

export function BillingSummaryView({ defaultYearMonth }: Props) {
  const [mode, setMode] = useState<Mode>('month')
  const [selectedMonth, setSelectedMonth] = useState(defaultYearMonth)
  const [selectedYear, setSelectedYear] = useState(defaultYearMonth.slice(0, 4))

  // 기간 선택 옵션 — 실제 청구가 생성된 년월만 제시.
  const monthsQuery = useQuery({
    queryKey: ['billed-months'],
    queryFn: listBilledMonths,
  })
  const monthOptions = useMemo(() => monthsQuery.data ?? [], [monthsQuery.data])
  const yearOptions = useMemo(
    () => [...new Set(monthOptions.map((m) => m.slice(0, 4)))],
    [monthOptions],
  )

  // 목록 로드 후 현재 선택이 목록에 없으면 첫(최신) 항목으로 보정.
  useEffect(() => {
    if (monthOptions.length > 0 && !monthOptions.includes(selectedMonth)) {
      setSelectedMonth(monthOptions[0])
    }
  }, [monthOptions, selectedMonth])
  useEffect(() => {
    if (yearOptions.length > 0 && !yearOptions.includes(selectedYear)) {
      setSelectedYear(yearOptions[0])
    }
  }, [yearOptions, selectedYear])

  const period = mode === 'year' ? selectedYear : selectedMonth
  const hasPeriods = monthOptions.length > 0

  const statsQuery = useQuery({
    queryKey: ['billing-period-stats', period],
    queryFn: () => getBillingPeriodStats(period),
    enabled: hasPeriods && monthOptions.includes(selectedMonth),
  })
  const stats = statsQuery.data

  if (monthsQuery.isLoading) return <p>불러오는 중...</p>
  if (!hasPeriods) {
    return (
      <div className="rounded-md border border-[var(--border)] bg-gray-50 p-6 text-center text-gray-600">
        청구 데이터가 생성된 월이 없습니다. 청구 목록 탭에서 먼저 청구를 생성해 주세요.
      </div>
    )
  }

  return (
    <div className="space-y-5">
      {/* 기간 선택 — 년/월 토글 + 선택 */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="flex items-center gap-3" role="group" aria-label="집계 단위">
          {(
            [
              ['month', '월'],
              ['year', '년'],
            ] as const
          ).map(([key, label]) => (
            <label
              key={key}
              className="flex min-h-[44px] cursor-pointer items-center gap-1 text-base text-gray-700"
            >
              <input
                type="checkbox"
                checked={mode === key}
                onChange={() => setMode(key)}
                className="h-4 w-4 cursor-pointer accent-[var(--accent)]"
              />
              {label}
            </label>
          ))}
        </div>

        {mode === 'month' ? (
          <label className="text-base font-medium">
            년월
            <select
              value={selectedMonth}
              onChange={(e) => setSelectedMonth(e.target.value)}
              className="ml-2 h-11 rounded-md border border-[var(--border)] px-3 text-base"
            >
              {monthOptions.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </label>
        ) : (
          <label className="text-base font-medium">
            연도
            <select
              value={selectedYear}
              onChange={(e) => setSelectedYear(e.target.value)}
              className="ml-2 h-11 rounded-md border border-[var(--border)] px-3 text-base"
            >
              {yearOptions.map((y) => (
                <option key={y} value={y}>
                  {y}년
                </option>
              ))}
            </select>
          </label>
        )}
      </div>

      {statsQuery.isLoading || !stats ? (
        <p>불러오는 중...</p>
      ) : stats.billCount === 0 ? (
        <div className="rounded-md border border-[var(--border)] bg-gray-50 p-6 text-center text-gray-600">
          {period} 청구 데이터가 없습니다.
        </div>
      ) : (
        <>
          {/* 상단 요약 박스 */}
          <div className="grid grid-cols-2 gap-3 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-sm md:grid-cols-3 lg:grid-cols-5">
            <div>
              청구 건수: <strong>{stats.billCount}건</strong>
            </div>
            <div>
              청구 총액: <strong>{won(stats.totalBilled)}</strong>
            </div>
            <div>
              수납 건수: <strong>{stats.paidCount}건</strong>
            </div>
            <div>
              총 수납금액: <strong>{won(stats.totalPaid)}</strong>
            </div>
            <div>
              미납:{' '}
              <strong className="text-[var(--danger)]">
                {won(stats.totalUnpaid)} ({stats.unpaidCount}건)
              </strong>
            </div>
          </div>

          {/* 결제수단별 수납 총액 — 결제수단을 열로 배치 */}
          <section>
            <h2 className="mb-2 text-lg font-bold">결제수단별 수납 총액</h2>
            {stats.byMethod.length === 0 ? (
              <div className="rounded-md border border-[var(--border)] bg-gray-50 p-4 text-center text-gray-600">
                수납 완료된 청구가 없습니다.
              </div>
            ) : (
              <div className="overflow-x-auto rounded-md border border-[var(--border)]">
                <table className="w-full text-base">
                  <thead className="bg-gray-100 text-center">
                    <tr>
                      <th className="px-3 py-2 text-left">구분</th>
                      {stats.byMethod.map((m) => (
                        <th key={m.paymentMethodId ?? 'none'} className="px-3 py-2">
                          {m.paymentMethodLabel}
                        </th>
                      ))}
                      <th className="px-3 py-2 bg-gray-200">합계</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr className="border-t border-[var(--border)] text-center">
                      <td className="px-3 py-2 text-left font-medium">수납 건수</td>
                      {stats.byMethod.map((m) => (
                        <td key={m.paymentMethodId ?? 'none'} className="px-3 py-2">
                          {m.paidCount}건
                        </td>
                      ))}
                      <td className="px-3 py-2 bg-gray-50 font-semibold">{stats.paidCount}건</td>
                    </tr>
                    <tr className="border-t border-[var(--border)] text-center">
                      <td className="px-3 py-2 text-left font-medium">수납 총액</td>
                      {stats.byMethod.map((m) => (
                        <td key={m.paymentMethodId ?? 'none'} className="px-3 py-2 font-semibold">
                          {won(m.totalPaid)}
                        </td>
                      ))}
                      <td className="px-3 py-2 bg-gray-50 font-bold">{won(stats.totalPaid)}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            )}
          </section>
        </>
      )}
    </div>
  )
}
