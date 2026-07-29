'use client'

/**
 * 수업일 이동 다이얼로그 — Sprint 16 T0 케이스1 (PI-26/27).
 *
 * present 셀 우클릭 → 액션 선택 → "수업일 이동" 진입. 흐름:
 * 1. **교습기간 범위 달력**(grid-cols-7)에서 도착일 선택 (텍스트 입력 아닌 시각적 달력)
 * 2. 선택 불가일 비활성: 교습기간 밖 / 출발일 자신 / 이미 출결 있는 날 / 휴일·정규수업 OFF 일자 / 주말
 * 3. 날짜 클릭 → `moveAttendance(studentId, fromDate, toDate)` → 성공 시 onSuccess
 *
 * Sprint 24: 백엔드가 "같은 교습기간 안에서만" 이동을 허용하도록 바뀌어(B1), 다월 교습기간
 * (예: 8월 = 7/30~9/2)의 이동 대상이 두 달에 걸칠 수 있다. 달력을 출발일 달력월이 아니라
 * **교습기간 범위(periodStart~periodEnd)** 로 그려 9/1·9/2 같은 이웃 달 대상도 고를 수 있게 한다.
 * 교습기간 밖(예: 9/30)은 다른 교습기간이므로 이동이 아닌 보강으로 등록한다.
 */

import { useMemo, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { moveAttendance } from '@/lib/tauri'
import type { AttendanceGridStudent, DaySchedule } from '@/types/attendance'

const WEEKDAY = ['일', '월', '화', '수', '목', '금', '토'] as const

interface Props {
  student: AttendanceGridStudent
  invalidationYm: string // YYYY-MM — 이동 반영 후 무효화할 출결 그리드 년월 (A126 명확화)
  fromDate: string // YYYY-MM-DD
  /** 교습기간 시작/종료 (YYYY-MM-DD). 없으면 출발일 달력월로 폴백. */
  periodStart: string | null
  periodEnd: string | null
  daySchedules: DaySchedule[]
  onClose: () => void
  onSuccess: () => void
}

function parseISO(s: string): Date {
  const [y, m, d] = s.split('-').map(Number)
  return new Date(y, m - 1, d)
}

function toISO(dt: Date): string {
  return `${dt.getFullYear()}-${String(dt.getMonth() + 1).padStart(2, '0')}-${String(
    dt.getDate(),
  ).padStart(2, '0')}`
}

export function MoveAttendanceDialog({
  student,
  invalidationYm,
  fromDate,
  periodStart,
  periodEnd,
  daySchedules,
  onClose,
  onSuccess,
}: Props) {
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  // PI-28/29: 도착일의 수업 시작시간 — 시(時) 단위만 선택(분 없음). 기본 16시.
  const [startHour, setStartHour] = useState(16)
  const queryClient = useQueryClient()

  // 이동 가능 범위 = 교습기간(periodStart~periodEnd). 미제공 시 출발일 달력월로 폴백.
  const rangeStart = periodStart ?? `${fromDate.slice(0, 7)}-01`
  const rangeEnd = useMemo(() => {
    if (periodEnd) return periodEnd
    const y = Number(fromDate.slice(0, 4))
    const m = Number(fromDate.slice(5, 7))
    const last = new Date(y, m, 0).getDate()
    return `${fromDate.slice(0, 7)}-${String(last).padStart(2, '0')}`
  }, [periodEnd, fromDate])

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
  const regularBlocked = useMemo(() => {
    const s = new Set<string>()
    for (const d of daySchedules) if (d.regularBlocked) s.add(d.eventDate)
    return s
  }, [daySchedules])

  // 교습기간 범위 시작주(일요일)~종료주(토요일) 의 모든 날짜 셀.
  const cells = useMemo(() => {
    const start = parseISO(rangeStart)
    const end = parseISO(rangeEnd)
    const gridStart = new Date(start)
    gridStart.setDate(gridStart.getDate() - gridStart.getDay()) // 그 주 일요일
    const gridEnd = new Date(end)
    gridEnd.setDate(gridEnd.getDate() + (6 - gridEnd.getDay())) // 그 주 토요일
    const out: { iso: string; inRange: boolean }[] = []
    const cur = new Date(gridStart)
    while (cur <= gridEnd) {
      const iso = toISO(cur)
      out.push({ iso, inRange: iso >= rangeStart && iso <= rangeEnd })
      cur.setDate(cur.getDate() + 1)
    }
    return out
  }, [rangeStart, rangeEnd])

  function reason(iso: string, inRange: boolean): string | null {
    if (!inRange) return '교습기간 밖 (다른 교습기간은 보강으로 등록)'
    if (iso === fromDate) return '현재 수업일'
    if (occupied.has(iso)) return '이미 수업이 있는 날 (추가 수업은 보강으로 등록)'
    const dow = parseISO(iso).getDay()
    if (dow === 0 || dow === 6) return '주말 (정규수업 불가)'
    if (blocked.has(iso)) return '휴일 (정규수업 불가)'
    if (regularBlocked.has(iso)) return '정규수업 불가일'
    return null
  }

  async function handleSelect(iso: string) {
    const startTime = `${String(startHour).padStart(2, '0')}:00`
    setSubmitting(true)
    setError(null)
    try {
      await moveAttendance(student.studentId, fromDate, iso, startTime)
      void queryClient.invalidateQueries({ queryKey: ['attendance-grid', invalidationYm] })
      onSuccess()
    } catch (e) {
      setError(typeof e === 'string' ? e : (e as Error).message)
      setSubmitting(false)
    }
  }

  const rangeLabel = `${rangeStart.slice(5).replace('-', '/')}~${rangeEnd.slice(5).replace('-', '/')}`

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
        <p className="mt-1 text-sm text-muted-foreground">
          같은 교습기간({rangeLabel}) 안에서만 이동할 수 있습니다. 다른 교습기간·휴일·이미 수업이
          있는 날은 선택할 수 없습니다. (다른 교습기간으로 옮기려면 보강을 이용하세요.)
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
          <span className="text-sm text-muted-foreground">날짜를 클릭하면 이 시간으로 이동</span>
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
          {cells.map(({ iso, inRange }) => {
            const d = parseISO(iso)
            const day = d.getDate()
            // 월 경계 인식: 매월 1일은 "M/1" 로 표기해 달이 바뀌는 지점을 보여준다.
            const label = day === 1 ? `${d.getMonth() + 1}/1` : String(day)
            const blockReason = reason(iso, inRange)
            const isFrom = iso === fromDate
            const disabled = blockReason !== null || submitting
            return (
              <button
                key={iso}
                type="button"
                disabled={disabled}
                onClick={() => handleSelect(iso)}
                title={blockReason ?? ''}
                className={`min-h-[40px] rounded text-base ${
                  isFrom
                    ? 'bg-amber-100 font-bold text-amber-800'
                    : !inRange
                      ? 'cursor-not-allowed text-gray-200'
                      : blockReason === null
                        ? 'hover:bg-[var(--accent)] hover:text-white'
                        : 'cursor-not-allowed text-gray-300'
                }`}
              >
                {label}
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
