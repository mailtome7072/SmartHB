'use client'

/**
 * 청구 관리 / 수납 관리 페이지 공유 컨트롤 훅 (Sprint 16 — 메뉴 분리).
 *
 * '청구/수납 관리' 단일 페이지를 '청구 관리'(/billing)·'수납 관리'(/payments) 둘로 분리하면서,
 * 두 페이지가 공통으로 쓰는 상태/쿼리를 한 곳에 모은다:
 *  - 청구년월 선택 + 옵션(교습기간 등록 월) + 자동 보정
 *  - 통합 검색(원생 이름/연락처/입금자) — 매칭 원생 id 집합
 *  - 월별 요약(getBillingSummary)
 *
 * 두 페이지에서 동일 로직이 어긋나지 않도록 SSOT 로 유지한다.
 */

import { useEffect, useMemo, useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { getBillingSummary, listStudyPeriods, searchStudentsForBilling } from '@/lib/tauri'
import type { BillingSearchResult, BillingSummary } from '@/types/billing'

/** 일정 관리(교습기간) 조회 범위 — study_periods 테이블은 작아 비용 무시 가능. */
const STUDY_PERIOD_FROM = '2000-01'
const STUDY_PERIOD_TO = '2099-12'

function currentYearMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

export interface BillingShared {
  effectiveYearMonth: string
  setYearMonth: (ym: string) => void
  monthOptions: string[]
  searchInput: string
  setSearchInput: (s: string) => void
  appliedSearch: string
  applySearch: () => void
  clearSearch: () => void
  /** 검색 미적용 시 null, 적용 시 매칭 원생 id 집합. */
  matchedStudentIds: Set<number> | null
  searchResults: BillingSearchResult[]
  summary: BillingSummary | undefined
}

export function useBillingShared(): BillingShared {
  const qc = useQueryClient()
  const [yearMonth, setYearMonth] = useState<string | null>(null)
  const effectiveYearMonth = yearMonth ?? currentYearMonth()

  // 통합 검색 — 원생 이름 / 연락처(- 제거 후 완전 일치) / 입금자 이름.
  const [searchInput, setSearchInput] = useState('')
  const [appliedSearch, setAppliedSearch] = useState('')
  const searchQuery = useQuery({
    queryKey: ['billing-search', appliedSearch],
    queryFn: () => searchStudentsForBilling(appliedSearch),
    enabled: appliedSearch.trim().length > 0,
  })
  const matchedStudentIds: Set<number> | null =
    appliedSearch.trim() === ''
      ? null
      : new Set((searchQuery.data ?? []).map((r) => r.studentId))
  const searchResults: BillingSearchResult[] = searchQuery.data ?? []
  const applySearch = () => setAppliedSearch(searchInput)
  const clearSearch = () => {
    setSearchInput('')
    setAppliedSearch('')
  }

  const summaryQuery = useQuery({
    queryKey: ['billing-summary', effectiveYearMonth],
    queryFn: () => getBillingSummary(effectiveYearMonth),
  })

  // 청구년월 드롭다운 옵션 — 일정 관리(교습기간) 등록된 년월만. 미등록 시 현재 년월 fallback.
  // 메뉴 진입(=mount)마다 매번 갱신 (출결관리와 동일 정책).
  const studyPeriodsQuery = useQuery({
    queryKey: ['study-periods', STUDY_PERIOD_FROM, STUDY_PERIOD_TO],
    queryFn: () => listStudyPeriods(STUDY_PERIOD_FROM, STUDY_PERIOD_TO),
    staleTime: 0,
    refetchOnMount: 'always',
  })
  useEffect(() => {
    void qc.invalidateQueries({ queryKey: ['study-periods'] })
  }, [qc])

  const monthOptions = useMemo(() => {
    const periods = studyPeriodsQuery.data
    if (periods === undefined || periods.length === 0) {
      return [currentYearMonth()]
    }
    return [...new Set(periods.map((p) => p.year_month))].sort((a, b) => b.localeCompare(a))
  }, [studyPeriodsQuery.data])

  // 교습기간 로드 후 현재 effectiveYearMonth 가 옵션에 없으면 첫 옵션(최신)으로 이동.
  useEffect(() => {
    if (studyPeriodsQuery.data === undefined) return
    if (!monthOptions.includes(effectiveYearMonth)) {
      setYearMonth(monthOptions[0])
    }
  }, [studyPeriodsQuery.data, monthOptions, effectiveYearMonth])

  return {
    effectiveYearMonth,
    setYearMonth,
    monthOptions,
    searchInput,
    setSearchInput,
    appliedSearch,
    applySearch,
    clearSearch,
    matchedStudentIds,
    searchResults,
    summary: summaryQuery.data,
  }
}
