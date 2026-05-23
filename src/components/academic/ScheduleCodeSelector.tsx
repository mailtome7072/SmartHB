'use client'

/**
 * 학사 일정 배치 코드 selector — Sprint 7 T5 (Issue 3) + V6/V10/V11 (post-review).
 *
 * - V6: 활성화된 시스템 예약 코드도 노출 (사용자가 settings 에서 활성 토글한 시스템 코드 포함)
 * - V10: 코드명만 한 줄 chip 으로 표기 — 🔒 / 중복불가 / 기간성 배지 제거하여 시각 노이즈 최소화
 * - V11: 외곽 box 제거 — 부모(`/academic` page) 의 통합 컨트롤 바 박스 안에 inline 렌더
 *
 * CRUD 가 필요하면 우측 "설정에서 관리" 링크로 이동 (ScheduleCodePanel).
 */

import Link from 'next/link'
import { useQuery } from '@tanstack/react-query'
import { listScheduleCodes } from '@/lib/tauri'
import type { ScheduleCode } from '@/types/academic'

interface ScheduleCodeSelectorProps {
  selectedCodeId: number | null
  onSelect: (code: ScheduleCode | null) => void
}

export function ScheduleCodeSelector({
  selectedCodeId,
  onSelect,
}: ScheduleCodeSelectorProps) {
  const codesQuery = useQuery({
    queryKey: ['schedule-codes'],
    queryFn: listScheduleCodes,
    staleTime: 60_000,
  })

  const activeCodes = (codesQuery.data ?? []).filter((c) => c.is_active)

  return (
    <div aria-label="학사 일정 배치 코드" className="flex flex-col gap-1">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-base font-bold text-[var(--foreground)]">일정 배치 코드</h2>
        <Link
          href="/settings/schedule-codes"
          className="text-sm text-blue-700 underline hover:text-blue-900"
        >
          설정에서 관리 →
        </Link>
      </div>

      {codesQuery.isLoading && (
        <p className="text-xs text-gray-500">코드 목록 불러오는 중...</p>
      )}
      {codesQuery.isError && (
        <p role="alert" className="text-xs text-red-700">
          코드 목록을 불러오지 못했습니다.
        </p>
      )}

      {!codesQuery.isLoading && activeCodes.length === 0 && (
        <p className="text-xs text-gray-500">
          활성 코드가 없습니다.{' '}
          <Link href="/settings/schedule-codes" className="text-blue-700 underline">
            설정에서 활성화하세요
          </Link>
          .
        </p>
      )}

      {activeCodes.length > 0 && (
        <div
          className="flex flex-wrap gap-1.5"
          role="radiogroup"
          aria-label="배치할 코드 선택"
        >
          {activeCodes.map((code) => {
            const isSelected = code.id === selectedCodeId
            return (
              <button
                key={code.id}
                type="button"
                role="radio"
                aria-checked={isSelected}
                onClick={() => onSelect(isSelected ? null : code)}
                className={[
                  'min-h-[44px] rounded-md border px-3 py-1 text-base',
                  isSelected
                    ? 'border-blue-500 bg-blue-50 text-blue-900'
                    : 'border-[var(--border)] bg-white text-[var(--foreground)] hover:bg-gray-50',
                ].join(' ')}
              >
                {code.code_name}
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}
