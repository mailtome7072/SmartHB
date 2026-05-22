'use client'

/**
 * 학사 일정 배치 모드의 코드 선택 컴팩트 selector — Sprint 7 T5 (Issue 3).
 *
 * 분리 원칙:
 * - 본 컴포넌트: `/academic` 페이지 운영성 작업 (활성 사용자 코드 선택만)
 * - [[ScheduleCodePanel]]: `/settings/schedule-codes` 설정성 작업 (CRUD)
 *
 * 시스템 예약 코드(`is_system_reserved=1`) 는 자동 배치 (공휴일 시드·단원평가 자동 배치) 되거나
 * 별도 워크플로우로 처리되므로 본 selector 는 **활성 사용자 코드만** 노출한다.
 * 코드 추가/관리가 필요하면 우측 "설정에서 관리" 링크로 이동.
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

  const activeUserCodes = (codesQuery.data ?? []).filter(
    (c) => c.is_active && !c.is_system_reserved,
  )

  return (
    <section
      aria-label="학사 일정 배치 코드"
      className="flex flex-col gap-2 rounded-lg border border-[var(--border)] bg-white p-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-lg font-bold text-[var(--foreground)]">일정 배치 코드</h2>
        <Link
          href="/settings/schedule-codes"
          className="text-sm text-blue-700 underline hover:text-blue-900"
        >
          설정에서 관리 →
        </Link>
      </div>

      {codesQuery.isLoading && (
        <p className="text-sm text-gray-500">코드 목록 불러오는 중...</p>
      )}
      {codesQuery.isError && (
        <p role="alert" className="text-sm text-red-700">
          코드 목록을 불러오지 못했습니다.
        </p>
      )}

      {!codesQuery.isLoading && activeUserCodes.length === 0 && (
        <p className="text-sm text-gray-500">
          활성 사용자 코드가 없습니다.{' '}
          <Link href="/settings/schedule-codes" className="text-blue-700 underline">
            설정에서 추가하세요
          </Link>
          .
        </p>
      )}

      {activeUserCodes.length > 0 && (
        <ul className="flex flex-col gap-1" role="radiogroup" aria-label="배치할 코드 선택">
          {activeUserCodes.map((code) => {
            const isSelected = code.id === selectedCodeId
            return (
              <li key={code.id}>
                <button
                  type="button"
                  role="radio"
                  aria-checked={isSelected}
                  onClick={() => onSelect(isSelected ? null : code)}
                  className={[
                    'flex min-h-[44px] w-full items-center gap-2 rounded border px-3 py-2 text-left text-base',
                    isSelected
                      ? 'border-blue-500 bg-blue-50 text-blue-900'
                      : 'border-[var(--border)] bg-white hover:bg-gray-50',
                  ].join(' ')}
                >
                  <span className="font-semibold">{code.code_name}</span>
                  {code.is_period_type && (
                    <span className="rounded bg-purple-100 px-1.5 py-0.5 text-xs text-purple-700">
                      기간성
                    </span>
                  )}
                  {code.is_duplicate_blocked && (
                    <span className="rounded bg-orange-100 px-1.5 py-0.5 text-xs text-orange-700">
                      중복불가
                    </span>
                  )}
                </button>
              </li>
            )
          })}
        </ul>
      )}
    </section>
  )
}
