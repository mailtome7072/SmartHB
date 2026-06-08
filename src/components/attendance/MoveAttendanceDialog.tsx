'use client'

/**
 * 수업일 이동 다이얼로그 — Sprint 16 T0 케이스1 (PI-26/27).
 *
 * present 셀 우클릭 → 액션 선택 → "수업일 이동" 진입. 흐름:
 * 1. 같은 달 달력(grid-cols-7)에서 도착일 선택 (텍스트 입력 아닌 시각적 달력)
 * 2. 선택 불가일 비활성: 출발일 자신 / 이미 출결 있는 날 / 휴일·정규수업 OFF 일자
 * 3. 날짜 클릭 → `moveAttendance(studentId, fromDate, toDate)` → 성공 시 onSuccess
 *
 * 동월 한정·OFF/충돌 차단은 백엔드가 최종 검증하며, 본 다이얼로그는 사전 비활성으로 실수를 줄인다.
 */

import { useMemo, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { moveAttendance } from '@/lib/tauri'
import type { AttendanceGridStudent, DaySchedule } from '@/types/attendance'

const WEEKDAY = ['일', '월', '화', '수', '목', '금', '토'] as const

interface Props {
  student: AttendanceGridStudent
  yearMonth: string // YYYY-MM
  fromDate: string // YYYY-MM-DD
  daySchedules: DaySchedule[]
  onClose: () => void
  onSuccess: () => void
}

export function MoveAttendanceDialog({
  student,
  yearMonth,
  fromDate,
  daySchedules,
  onClose,
  onSuccess,
}: Props) {
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  // PI-28/29: 도착일의 수업 시작시간 — 시(時) 단위만 선택(분 없음). 기본 16시.
  const [startHour, setStartHour] = useState(16)
  const queryClient = useQueryClient()

  const [year, month] = yearMonth.split('-').map(Number)
  const lastDay = new Date(year, month, 0).getDate()
  const firstDow = new Date(year, month - 1, 1).getDay() // 0=일

  // 이미 출결이 있는 일자 (충돌 차단)
  const occupied = useMemo(() => {
    const s = new Set<string>()
    for (const a of student.attendances) s.add(a.eventDate)
    return s
  }, [student.attendances])

  // 휴일/정규수업 OFF 일자 (이동 차단 — isBlock = 공휴일/방학/휴원)
  const blocked = useMemo(() => {
    const s = new Set<string>()
    for (const d of daySchedules) if (d.isBlock) s.add(d.eventDate)
    return s
  }, [daySchedules])

  // 정규수업 불가 코드일 (공휴일/방학/휴원/보강데이 — allows_regular_class=0) — 이동 차단 (PI-30)
  // 공휴수업일처럼 보강 가능하지만 정규도 가능한 날은 regularBlocked=false 라 차단되지 않는다.
  const regularBlocked = useMemo(() => {
    const s = new Set<string>()
    for (const d of daySchedules) if (d.regularBlocked) s.add(d.eventDate)
    return s
  }, [daySchedules])

  function dateStr(day: number): string {
    return `${yearMonth}-${String(day).padStart(2, '0')}`
  }

  function reason(day: number): string | null {
    const ds = dateStr(day)
    if (ds === fromDate) return '현재 수업일'
    if (occupied.has(ds)) return '이미 수업이 있는 날 (추가 수업은 보강으로 등록)'
    // 정규수업은 평일만 — 주말/공휴일/보강데이로는 이동 불가 (PI-30)
    const dow = new Date(year, month - 1, day).getDay()
    if (dow === 0 || dow === 6) return '주말 (정규수업 불가)'
    if (blocked.has(ds)) return '휴일 (정규수업 불가)'
    if (regularBlocked.has(ds)) return '정규수업 불가일'
    return null
  }

  async function handleSelect(day: number) {
    const to = dateStr(day)
    const startTime = `${String(startHour).padStart(2, '0')}:00`
    setSubmitting(true)
    setError(null)
    try {
      await moveAttendance(student.studentId, fromDate, to, startTime)
      void queryClient.invalidateQueries({ queryKey: ['attendance-grid', yearMonth] })
      onSuccess()
    } catch (e) {
      setError(typeof e === 'string' ? e : (e as Error).message)
      setSubmitting(false)
    }
  }

  // 달력 셀 — 1일 앞 빈 칸 + 1~말일
  const cells: (number | null)[] = []
  for (let i = 0; i < firstDow; i += 1) cells.push(null)
  for (let d = 1; d <= lastDay; d += 1) cells.push(d)

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="w-[440px] rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label="수업일 이동"
      >
        <h2 className="text-xl font-bold">수업일 이동</h2>
        <p className="mt-1 text-base text-gray-700">
          {student.name} · {fromDate.slice(5).replace('-', '/')} 수업을 옮길 날짜를 선택하세요.
        </p>
        <p className="mt-1 text-sm text-gray-500">
          같은 달 안에서만 이동할 수 있습니다. 휴일·이미 수업이 있는 날은 선택할 수 없습니다.
        </p>

        <div className="mt-3 flex items-center gap-2">
          <label htmlFor="move-start-hour" className="text-base text-gray-700">
            수업 시작시간:
          </label>
          <select
            id="move-start-hour"
            value={startHour}
            onChange={(e) => setStartHour(Number(e.target.value))}
            className="min-h-[40px] rounded-md border-2 border-[var(--border)] px-3 text-base"
            aria-label="도착일 수업 시작시간 (시 단위)"
          >
            {Array.from({ length: 14 }, (_, i) => i + 9).map((h) => (
              <option key={h} value={h}>
                {h}시
              </option>
            ))}
          </select>
          <span className="text-sm text-gray-500">날짜를 클릭하면 이 시간으로 이동</span>
        </div>

        {error !== null && (
          <div
            role="alert"
            className="mt-3 rounded-md border-2 border-[var(--danger)] bg-red-50 p-2 text-sm text-[var(--danger)]"
          >
            {error}
          </div>
        )}

        <div className="mt-4 grid grid-cols-7 gap-1 text-center">
          {WEEKDAY.map((w, i) => (
            <div
              key={w}
              className={`py-1 text-sm font-semibold ${
                i === 0 ? 'text-red-600' : i === 6 ? 'text-blue-600' : 'text-gray-600'
              }`}
            >
              {w}
            </div>
          ))}
          {cells.map((day, idx) => {
            if (day === null) return <div key={`empty-${idx}`} />
            const blockReason = reason(day)
            const isFrom = dateStr(day) === fromDate
            const disabled = blockReason !== null || submitting
            return (
              <button
                key={day}
                type="button"
                disabled={disabled}
                onClick={() => handleSelect(day)}
                title={blockReason ?? ''}
                className={`min-h-[40px] rounded text-base ${
                  isFrom
                    ? 'bg-amber-100 font-bold text-amber-800'
                    : blockReason === null
                      ? 'hover:bg-[var(--accent)] hover:text-white'
                      : 'cursor-not-allowed text-gray-300'
                }`}
              >
                {day}
              </button>
            )
          })}
        </div>

        <div className="mt-5 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="min-h-[44px] rounded-lg border-2 border-[var(--border)] px-4 text-base hover:bg-gray-50"
          >
            닫기
          </button>
        </div>
      </div>
    </div>
  )
}
