'use client'

/**
 * 보강데이 일괄 등록 다이얼로그 — Sprint 9 T7 (PRD §4.5.5).
 *
 * `/attendance` 헤더 "보강데이 일괄" 버튼으로 진입. 흐름:
 * 1. 이벤트 일자 입력 (date)
 * 2. 그리드에서 미처리 결석이 있는 학생 자동 추출 → 체크박스 다중 선택
 * 3. 각 학생의 모든 미처리 결석을 자동 충당 + 첫 결석의 class_minutes 적용
 * 4. `batchCreateMakeups` → 부분 성공 결과 표시 (succeeded/failed)
 *
 * 디자인 단순화:
 * - 학생별 결석 세부 편집은 본 다이얼로그 범위 외 — MakeupRegisterDialog 가 담당
 * - 본 다이얼로그는 "보강데이" 시나리오 (여러 원생 + 단일 일자) 에 특화
 */

import { useEffect, useMemo, useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { batchCreateMakeups } from '@/lib/tauri'
import { minutesToHoursText } from '@/lib/time'
import type {
  AttendanceCell,
  AttendanceGrid as AttendanceGridType,
} from '@/types/attendance'
import type { BatchFailure, BatchMakeupEntry, MakeupResult } from '@/types/makeup'

interface Props {
  grid: AttendanceGridType
  onClose: () => void
  onSuccess: () => void
}

/** 학생 1명의 미처리 결석 추출 — grid 데이터에서 client-side 필터. */
function extractPendingAbsences(attendances: AttendanceCell[]): AttendanceCell[] {
  return attendances.filter(
    (a) => a.status === 'absent' && a.makeupAttendanceId === null,
  )
}

export function BatchMakeupDialog({ grid, onClose, onSuccess }: Props) {
  const [eventDate, setEventDate] = useState('')
  // 일괄 대상 학생 — 미처리 결석이 있는 학생 ID 집합. 초기 전원 선택.
  const candidateStudents = useMemo(
    () =>
      grid.students
        .map((s) => ({
          studentId: s.studentId,
          name: s.name,
          serialNo: s.serialNo,
          pending: extractPendingAbsences(s.attendances),
        }))
        .filter((s) => s.pending.length > 0),
    [grid.students],
  )
  const [selected, setSelected] = useState<Set<number>>(
    () => new Set(candidateStudents.map((s) => s.studentId)),
  )
  const [result, setResult] = useState<{
    succeeded: MakeupResult[]
    failed: BatchFailure[]
  } | null>(null)
  const [error, setError] = useState<string | null>(null)

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

  function toggleStudent(studentId: number) {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(studentId)) next.delete(studentId)
      else next.add(studentId)
      return next
    })
  }

  function selectAll() {
    setSelected(new Set(candidateStudents.map((s) => s.studentId)))
  }
  function selectNone() {
    setSelected(new Set())
  }

  const mutation = useMutation({
    mutationFn: () => {
      const entries: BatchMakeupEntry[] = candidateStudents
        .filter((s) => selected.has(s.studentId))
        .map((s) => ({
          studentId: s.studentId,
          // 단순화: 첫 결석의 class_minutes 적용. 결석들이 다른 시간일 경우 사용자는 개별
          // MakeupRegisterDialog 로 fallback (그리드에서 각 결석 셀 다시 검토).
          classMinutes: s.pending[0].classMinutes,
          absenceIds: s.pending.map((a) => a.id),
        }))
      return batchCreateMakeups({ eventDate, entries })
    },
    onSuccess: (data) => {
      setError(null)
      setResult(data)
      // 부분 성공이라도 onSuccess 트리거 — 그리드 invalidate.
      if (data.succeeded.length > 0) {
        onSuccess()
      }
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
    },
  })

  const canSubmit =
    eventDate.length === 10 &&
    selected.size > 0 &&
    !mutation.isPending &&
    result === null

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="batch-makeup-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="batch-makeup-title" className="text-xl font-bold">
          보강데이 일괄 등록
        </h2>
        <p className="mt-1 text-sm text-gray-600">
          한 일자에 미처리 결석이 있는 원생들을 일괄 보강 등록합니다.
          학생별로 모든 미처리 결석이 자동 충당됩니다.
        </p>

        {/* 이벤트 일자 입력 */}
        <div className="mt-4 flex items-center gap-2">
          <label htmlFor="batch-event-date" className="text-base text-gray-700">
            보강 일자:
          </label>
          <input
            id="batch-event-date"
            type="date"
            value={eventDate}
            onChange={(e) => setEventDate(e.target.value)}
            disabled={result !== null}
            className="min-h-[44px] rounded-md border-2 border-[var(--border)] px-3 text-base disabled:bg-gray-50"
          />
          <span className="text-sm text-gray-500">
            ※ 학사일정에서 &ldquo;보강 진행 가능&rdquo; 코드가 활성된 일자만 가능
          </span>
        </div>

        {/* 후보 학생 리스트 */}
        {candidateStudents.length === 0 ? (
          <p className="mt-4 rounded-md border border-[var(--border)] bg-gray-50 p-4 text-center text-base text-gray-700">
            미처리 결석이 있는 원생이 없습니다.
          </p>
        ) : (
          <>
            <div className="mt-4 flex items-center justify-between">
              <h3 className="text-base font-semibold">
                대상 원생 ({selected.size} / {candidateStudents.length}명)
              </h3>
              <div className="flex gap-2 text-sm">
                <button
                  type="button"
                  onClick={selectAll}
                  disabled={result !== null}
                  className="text-blue-600 hover:underline disabled:text-gray-400"
                >
                  전체 선택
                </button>
                <span className="text-gray-300">|</span>
                <button
                  type="button"
                  onClick={selectNone}
                  disabled={result !== null}
                  className="text-blue-600 hover:underline disabled:text-gray-400"
                >
                  전체 해제
                </button>
              </div>
            </div>
            <ul className="mt-2 max-h-64 overflow-y-auto rounded-md border border-[var(--border)]">
              {candidateStudents.map((s) => (
                <li
                  key={s.studentId}
                  className="flex items-center gap-3 border-b border-[var(--border)] px-3 py-2 last:border-b-0 hover:bg-gray-50"
                >
                  <input
                    type="checkbox"
                    checked={selected.has(s.studentId)}
                    onChange={() => toggleStudent(s.studentId)}
                    disabled={result !== null}
                    aria-label={`${s.name} 선택`}
                    className="h-5 w-5 cursor-pointer"
                  />
                  <button
                    type="button"
                    onClick={() => toggleStudent(s.studentId)}
                    disabled={result !== null}
                    className="flex-1 text-left text-base disabled:cursor-default"
                  >
                    <span className="font-semibold">{s.name}</span>
                    <span className="ml-1 text-sm text-gray-500">#{s.serialNo}</span>
                    <span className="ml-2 text-sm text-gray-600">
                      미처리 {s.pending.length}건
                    </span>
                    <span className="ml-2 text-sm text-gray-500">
                      ({minutesToHoursText(s.pending[0].classMinutes)}시간/회)
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        )}

        {/* 결과 표시 */}
        {result !== null && (
          <div className="mt-4 rounded-md border-2 border-amber-300 bg-amber-50 p-4">
            <p className="text-base font-semibold text-gray-800">
              일괄 등록 결과 — 성공 {result.succeeded.length}명 / 실패 {result.failed.length}명
            </p>
            {result.failed.length > 0 && (
              <ul className="mt-2 space-y-1 text-sm text-[var(--danger)]">
                {result.failed.map((f) => {
                  const student = candidateStudents.find(
                    (s) => s.studentId === f.studentId,
                  )
                  return (
                    <li key={f.studentId}>
                      <span className="font-semibold">{student?.name ?? f.studentId}</span>:{' '}
                      {f.reason}
                    </li>
                  )
                })}
              </ul>
            )}
          </div>
        )}

        {/* 에러 */}
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
            {result === null ? '취소' : '닫기'}
          </button>
          {result === null && candidateStudents.length > 0 && (
            <button
              type="button"
              onClick={() => mutation.mutate()}
              disabled={!canSubmit}
              className="min-h-[44px] rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
            >
              {mutation.isPending ? '등록 중...' : `${selected.size}명 일괄 등록`}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
