'use client'

/**
 * 보강 등록 다이얼로그 — Sprint 9 T6 (PRD §4.5.4).
 *
 * 출결표의 비수업일 셀 클릭 → 진입. 흐름:
 * 1. `getMakeupEligibleDates(studentId, yearMonth)` → eventDate 매칭 검증
 * 2. eligible 시: `getPendingAbsences(studentId)` → 결석 다중 선택
 * 3. class_minutes 입력 (default 60)
 * 4. "확정" → `createMakeupWithAbsences` → 성공 시 onSuccess 호출
 *
 * eligibility 미충족 시 안내 메시지 + 닫기만 가능.
 */

import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import {
  createMakeupWithAbsences,
  getMakeupEligibleDates,
  getPendingAbsences,
} from '@/lib/tauri'
import type { PendingAbsence } from '@/types/makeup'

interface Props {
  studentId: number
  studentName: string
  studentSerialNo: string
  eventDate: string // YYYY-MM-DD
  yearMonth: string // YYYY-MM
  onClose: () => void
  onSuccess: () => void
}

export function MakeupRegisterDialog({
  studentId,
  studentName,
  studentSerialNo,
  eventDate,
  yearMonth,
  onClose,
  onSuccess,
}: Props) {
  const [selected, setSelected] = useState<Set<number>>(new Set())
  const [classMinutes, setClassMinutes] = useState(60)
  const [error, setError] = useState<string | null>(null)

  // ESC 키 닫기
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault()
        onClose()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  // 1. eligibility 검증 — eventDate 가 보강 가능 일자 목록에 포함되는지
  const eligibilityQuery = useQuery({
    queryKey: ['makeup-eligibility', studentId, yearMonth],
    queryFn: () => getMakeupEligibleDates(studentId, yearMonth),
  })

  const eligibleEntry = useMemo(
    () => eligibilityQuery.data?.find((d) => d.eventDate === eventDate),
    [eligibilityQuery.data, eventDate],
  )
  const isEligible = eligibleEntry !== undefined

  // 2. 미처리 결석 목록 — eligible 일 때만 활성
  const pendingQuery = useQuery({
    queryKey: ['pending-absences', studentId],
    queryFn: () => getPendingAbsences(studentId),
    enabled: isEligible,
  })

  // 3. 등록 mutation
  const mutation = useMutation({
    mutationFn: () =>
      createMakeupWithAbsences({
        studentId,
        eventDate,
        classMinutes,
        absenceIds: Array.from(selected),
      }),
    onSuccess: () => {
      setError(null)
      onSuccess()
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
    },
  })

  function toggleAbsence(id: number) {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const canSubmit =
    isEligible && selected.size > 0 && classMinutes > 0 && !mutation.isPending

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="makeup-register-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="makeup-register-title" className="text-xl font-bold">
          보강 등록
        </h2>
        <p className="mt-1 text-sm text-gray-600">
          <span className="font-semibold">{studentName}</span>
          <span className="ml-1 text-gray-500">#{studentSerialNo}</span>
          <span className="mx-2">·</span>
          <span>{eventDate}</span>
          {eligibleEntry !== undefined && (
            <span className="ml-1 text-gray-500">({eligibleEntry.scheduleCodeName})</span>
          )}
        </p>

        {/* eligibility 로딩 / 실패 */}
        {eligibilityQuery.isLoading && (
          <p className="mt-4 text-base text-gray-600">보강 가능 여부 확인 중...</p>
        )}
        {eligibilityQuery.isError && (
          <p
            role="alert"
            className="mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            보강 가능 일자 확인 실패: {(eligibilityQuery.error as Error).message}
          </p>
        )}
        {eligibilityQuery.isSuccess && !isEligible && (
          <p
            role="alert"
            className="mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {eventDate} 은 보강 가능 일자가 아닙니다. 학사 일정에서 &ldquo;보강 진행 가능&rdquo;
            코드가 활성된 일자에만 등록할 수 있습니다.
          </p>
        )}

        {/* eligible 시: 결석 목록 + 시간 입력 */}
        {isEligible && (
          <>
            <div className="mt-4">
              <h3 className="text-base font-semibold">충당할 결석 선택</h3>
              <p className="text-sm text-gray-600">
                소멸기한이 임박한 결석부터 정렬됩니다. 1건 이상 선택해 주세요.
              </p>
              {pendingQuery.isLoading && (
                <p className="mt-2 text-base text-gray-600">결석 조회 중...</p>
              )}
              {pendingQuery.isSuccess && pendingQuery.data.length === 0 && (
                <p className="mt-2 text-base text-gray-700">
                  이 원생의 미처리 결석이 없습니다.
                </p>
              )}
              {pendingQuery.isSuccess && pendingQuery.data.length > 0 && (
                <ul className="mt-2 max-h-64 overflow-y-auto rounded-md border border-[var(--border)]">
                  {pendingQuery.data.map((absence) => (
                    <AbsenceRow
                      key={absence.id}
                      absence={absence}
                      checked={selected.has(absence.id)}
                      onToggle={() => toggleAbsence(absence.id)}
                    />
                  ))}
                </ul>
              )}
            </div>

            <div className="mt-4 flex items-center gap-2">
              <label htmlFor="makeup-class-minutes" className="text-base text-gray-700">
                보강 수업 시간 (분):
              </label>
              <input
                id="makeup-class-minutes"
                type="number"
                min={1}
                max={300}
                value={classMinutes}
                onChange={(e) => setClassMinutes(Number(e.target.value) || 0)}
                className="min-h-[44px] w-28 rounded-md border-2 border-[var(--border)] px-3 text-base"
              />
            </div>
          </>
        )}

        {/* mutation 에러 */}
        {error !== null && (
          <p
            role="alert"
            className="mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 text-base text-gray-700 hover:bg-gray-50"
          >
            취소
          </button>
          {isEligible && (
            <button
              type="button"
              onClick={() => mutation.mutate()}
              disabled={!canSubmit}
              className="min-h-[44px] rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
            >
              {mutation.isPending ? '등록 중...' : `${selected.size}건 보강 등록`}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

interface AbsenceRowProps {
  absence: PendingAbsence
  checked: boolean
  onToggle: () => void
}

function AbsenceRow({ absence, checked, onToggle }: AbsenceRowProps) {
  return (
    <li className="flex items-center gap-3 border-b border-[var(--border)] px-3 py-2 last:border-b-0 hover:bg-gray-50">
      <input
        type="checkbox"
        checked={checked}
        onChange={onToggle}
        aria-label={`${absence.eventDate} 결석 선택`}
        className="h-5 w-5 cursor-pointer"
      />
      <button
        type="button"
        onClick={onToggle}
        className="flex-1 text-left text-base"
      >
        <span className="font-semibold">{absence.eventDate}</span>
        <span className="ml-2 text-sm text-gray-600">{absence.classMinutes}분</span>
        {absence.makeupDeadline !== null && (
          <span className="ml-2 text-sm text-amber-700">
            소멸기한 {absence.makeupDeadline}
          </span>
        )}
        {absence.absenceMemo !== null && (
          <div className="text-sm text-gray-500">메모: {absence.absenceMemo}</div>
        )}
      </button>
    </li>
  )
}
