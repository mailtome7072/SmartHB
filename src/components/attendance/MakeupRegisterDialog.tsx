'use client'

/**
 * 보강 등록 다이얼로그 — Sprint 9 T6 (PRD §4.5.4) + Session #10 (I1/I4/I5/I6).
 *
 * 출결표의 비수업일 셀 클릭 → 진입. 흐름:
 * 1. `getMakeupEligibleDates(studentId, yearMonth)` → eventDate 매칭 검증
 * 2. eligible 시: `getPendingAbsences(studentId)` → 결석 다중 선택
 *    (Session #10 I4: 선택 일자 이전 + 소멸기한 미도래 결석만 표시)
 * 3. 보강 시간(시간 단위, decimal) 입력 — Session #10 I1
 *    (체크 시 자동 합산 / 해제 시 min(absenceHours, currentHours) 차감 — I5+I6)
 * 4. "확정" → `createMakeupWithAbsences` → 성공 시 onSuccess
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
import {
  formatHours,
  hoursToMinutes,
  minutesToHours,
  minutesToHoursText,
} from '@/lib/time'
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
  // 보강 수업 시간 (시간 단위, decimal). 백엔드 전송 시 hoursToMinutes 변환.
  // 초기 0 — 결석 체크 시 자동 합산 (사용자 시각 검증 J3 정책: 선택만으로 시간 결정).
  const [classHours, setClassHours] = useState(0)
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

  // I4: 선택 일자 이전 + 소멸기한 미도래 결석만 필터.
  // 소멸기한 미도래: deadline === null OR deadline >= target year_month
  const filteredPending = useMemo(() => {
    if (pendingQuery.data === undefined) return []
    const targetYearMonth = eventDate.slice(0, 7)
    return pendingQuery.data.filter(
      (a) =>
        a.eventDate < eventDate &&
        (a.makeupDeadline === null || a.makeupDeadline >= targetYearMonth),
    )
  }, [pendingQuery.data, eventDate])

  // I5+I6: 결석 체크 토글 + 시간 자동 합산/차감
  // React Strict Mode 에서 setState 콜백 안의 또 다른 setState 호출은 중복 실행됨
  // (J3-2 시각 검증 발견 — 1시간 결석 체크 시 3시간 표시 버그). 두 setState 를 분리.
  function toggleAbsence(absence: PendingAbsence) {
    const absenceHours = minutesToHours(absence.classMinutes)
    const isCurrentlySelected = selected.has(absence.id)

    setSelected((prev) => {
      const next = new Set(prev)
      if (isCurrentlySelected) next.delete(absence.id)
      else next.add(absence.id)
      return next
    })

    if (isCurrentlySelected) {
      // 해제: min(absenceHours, currentHours) 만큼 차감 — 음수 방지
      setClassHours((h) => Math.max(0, h - Math.min(absenceHours, h)))
    } else {
      setClassHours((h) => h + absenceHours)
    }
  }

  // 3. 등록 mutation
  const mutation = useMutation({
    mutationFn: () =>
      createMakeupWithAbsences({
        studentId,
        eventDate,
        classMinutes: hoursToMinutes(classHours),
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

  const canSubmit =
    isEligible && selected.size > 0 && classHours > 0 && !mutation.isPending

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
            {eventDate} 은 보강 가능 일자가 아닙니다. 공휴일/방학/휴원일 또는
            주말+보강데이 미설정 일자입니다.
          </p>
        )}

        {/* eligible 시: 결석 목록 + 시간 입력 */}
        {isEligible && (
          <>
            <div className="mt-4">
              <h3 className="text-base font-semibold">충당할 결석 선택</h3>
              <p className="text-sm text-gray-600">
                보강 일자 이전 + 소멸기한 미도래 결석. 체크 시 보강 시간이 자동 합산됩니다.
              </p>
              {pendingQuery.isLoading && (
                <p className="mt-2 text-base text-gray-600">결석 조회 중...</p>
              )}
              {pendingQuery.isSuccess && filteredPending.length === 0 && (
                <p className="mt-2 text-base text-gray-700">
                  충당 가능한 결석이 없습니다 (보강 일자 이전 + 소멸기한 미도래 조건 미충족).
                </p>
              )}
              {filteredPending.length > 0 && (
                <ul className="mt-2 max-h-64 overflow-y-auto rounded-md border border-[var(--border)]">
                  {filteredPending.map((absence) => (
                    <AbsenceRow
                      key={absence.id}
                      absence={absence}
                      checked={selected.has(absence.id)}
                      onToggle={() => toggleAbsence(absence)}
                    />
                  ))}
                </ul>
              )}
            </div>

            <div className="mt-4 flex items-center gap-2">
              <label htmlFor="makeup-class-hours" className="text-base text-gray-700">
                보강 수업 시간 (시간):
              </label>
              <input
                id="makeup-class-hours"
                type="number"
                min={0}
                step={1}
                max={10}
                value={classHours}
                onChange={(e) => setClassHours(Number(e.target.value) || 0)}
                className="min-h-[44px] w-28 rounded-md border-2 border-[var(--border)] px-3 text-base"
              />
              <span className="text-sm text-gray-500">
                현재 {formatHours(classHours)}시간 ({hoursToMinutes(classHours)}분)
              </span>
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
        <span className="ml-2 text-sm text-gray-600">
          {minutesToHoursText(absence.classMinutes)}시간
        </span>
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
