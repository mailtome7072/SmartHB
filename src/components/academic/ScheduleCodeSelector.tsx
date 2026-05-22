'use client'

/**
 * 학사 일정 배치 모드의 코드 선택 컴팩트 selector — Sprint 7 T5 (Issue 3).
 *
 * 분리 원칙:
 * - 본 컴포넌트: `/academic` 페이지 운영성 작업 (활성 코드 선택)
 * - [[ScheduleCodePanel]]: `/settings/schedule-codes` 설정성 작업 (CRUD)
 *
 * V6 fix (Sprint 7 post-review): 활성화된 시스템 예약 코드도 노출 — 사용자가
 * `/settings/schedule-codes` 에서 활성 토글한 시스템 코드(예: 보강데이, 방학)는 교습기간
 * 일정 배치에 사용 가능해야 함. is_active 만 필터하고 is_system_reserved 는 노출 여부에
 * 영향 주지 않는다. 시스템 코드 row 는 시각적 마커(🔒)로 사용자에 안내.
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

      {!codesQuery.isLoading && activeCodes.length === 0 && (
        <p className="text-sm text-gray-500">
          활성 코드가 없습니다.{' '}
          <Link href="/settings/schedule-codes" className="text-blue-700 underline">
            설정에서 활성화하세요
          </Link>
          .
        </p>
      )}

      {activeCodes.length > 0 && (
        <ul className="flex flex-col gap-1" role="radiogroup" aria-label="배치할 코드 선택">
          {activeCodes.map((code) => {
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
                  <span className="font-semibold">
                    {code.is_system_reserved && <span title="시스템 예약 코드">🔒 </span>}
                    {code.code_name}
                  </span>
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
