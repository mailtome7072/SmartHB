'use client'

/**
 * 출결 그리드 — Sprint 8 T4 (PRD §4.5.3).
 *
 * 행 = 원생, 열 = 일자 (1~31). 50명×31일 = 1,550셀 native React 렌더링.
 * 좌측 원생 컬럼은 sticky. 셀 클릭으로 present↔absent 토글 (낙관적 업데이트).
 * 결석 셀은 우클릭 또는 메모 아이콘으로 사유 메모 입력.
 *
 * Undo: 마지막 토글 1건만 메모리 보관. Ctrl+Z (또는 Cmd+Z) 로 역토글 호출.
 *
 * 상태별 셀 색상:
 * - present: 흰색 + 회색 테두리
 * - absent: 빨간색 (`bg-red-100`)
 * - makeup_done: 빨강 + "보강" 표기
 * - makeup_expired: 회색 + "소멸"
 * - 비수업일 (셀 없음): `bg-gray-50` placeholder
 */

import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toggleAttendance, updateAbsenceMemo } from '@/lib/tauri'
import type {
  AttendanceCell,
  AttendanceGrid as AttendanceGridType,
  AttendanceStatus,
} from '@/types/attendance'
import { AbsenceMemoDialog } from './AbsenceMemoDialog'

interface Props {
  grid: AttendanceGridType
  /** Sprint 9 T6 — 비수업일(보강 가능 후보) 셀 클릭 시 호출. */
  onNonClassDayClick?: (studentId: number, eventDate: string) => void
  /** Sprint 9 T7 — makeup_done 셀 클릭 시 호출 (보강 관리 다이얼로그 진입). */
  onMakeupCellClick?: (studentId: number, cell: AttendanceCell) => void
  /** Sprint 9 T8 — 학생명 클릭 시 호출 (결석 이력 다이얼로그 진입). */
  onStudentNameClick?: (studentId: number) => void
}

interface LastToggle {
  attendanceId: number
  previousStatus: AttendanceStatus
}

/** 해당 월의 모든 일자 (1~말일) — yearMonth 기준. */
function daysOfMonth(yearMonth: string): number[] {
  const [year, month] = yearMonth.split('-').map(Number)
  // month 는 1-indexed (frontend), Date 는 0-indexed
  const lastDay = new Date(year, month, 0).getDate()
  return Array.from({ length: lastDay }, (_, i) => i + 1)
}

const WEEKDAY_LABEL = ['일', '월', '화', '수', '목', '금', '토'] as const

/** yearMonth(YYYY-MM) + 일자 → 한글 요일 (일~토). */
function weekdayLabel(yearMonth: string, day: number): string {
  const [year, month] = yearMonth.split('-').map(Number)
  return WEEKDAY_LABEL[new Date(year, month - 1, day).getDay()]
}

/** 분 → 시간 (소수점 1자리, 정수면 정수로). 0 → '0'. */
function minutesToHours(minutes: number): string {
  if (minutes === 0) return '0'
  const hours = minutes / 60
  return Number.isInteger(hours) ? String(hours) : hours.toFixed(1)
}

/** 일자에 해당하는 출결 셀 검색 (Map). */
function buildAttendanceByDay(
  attendances: AttendanceCell[],
): Map<string, AttendanceCell> {
  const map = new Map<string, AttendanceCell>()
  for (const a of attendances) {
    // event_date = YYYY-MM-DD → 일자(DD)만 추출
    const day = a.eventDate.slice(8, 10)
    map.set(day, a)
  }
  return map
}

export function AttendanceGrid({
  grid,
  onNonClassDayClick,
  onMakeupCellClick,
  onStudentNameClick,
}: Props) {
  const queryClient = useQueryClient()
  const [lastToggle, setLastToggle] = useState<LastToggle | null>(null)
  const [memoDialogCell, setMemoDialogCell] = useState<AttendanceCell | null>(null)
  const [error, setError] = useState<string | null>(null)

  const days = useMemo(() => daysOfMonth(grid.yearMonth), [grid.yearMonth])

  const toggleMutation = useMutation({
    mutationFn: async ({
      attendanceId,
      newStatus,
    }: {
      attendanceId: number
      newStatus: 'present' | 'absent'
    }) => toggleAttendance(attendanceId, newStatus),
    onSuccess: () => {
      setError(null)
      void queryClient.invalidateQueries({
        queryKey: ['attendance-grid', grid.yearMonth],
      })
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
      setLastToggle(null) // 실패 시 undo 무의미
    },
  })

  const handleCellClick = useCallback(
    (cell: AttendanceCell) => {
      if (cell.status !== 'present' && cell.status !== 'absent') {
        // makeup_done / makeup_expired — 토글 차단 (백엔드도 차단하지만 UX 차원에서도)
        setError('보강 매칭 또는 소멸된 출결은 직접 변경할 수 없습니다.')
        return
      }
      const newStatus: 'present' | 'absent' =
        cell.status === 'present' ? 'absent' : 'present'
      setLastToggle({ attendanceId: cell.id, previousStatus: cell.status })
      toggleMutation.mutate({ attendanceId: cell.id, newStatus })
    },
    [toggleMutation],
  )

  // Ctrl+Z / Cmd+Z — 마지막 토글 역실행
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const isUndo = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z'
      if (!isUndo || lastToggle === null) return
      e.preventDefault()
      toggleMutation.mutate({
        attendanceId: lastToggle.attendanceId,
        newStatus: lastToggle.previousStatus as 'present' | 'absent',
      })
      setLastToggle(null)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [lastToggle, toggleMutation])

  if (grid.students.length === 0) {
    return (
      <p className="text-gray-600">
        해당 월에 등록된 원생 출결이 없습니다. 교습기간 확정 + 원생 등록 상태를 확인하세요.
      </p>
    )
  }

  return (
    <>
      {error !== null && (
        <div
          role="alert"
          className="mb-3 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
        >
          {error}
        </div>
      )}

      <div className="overflow-auto rounded-lg border border-[var(--border)]">
        <table className="border-collapse text-base">
          <thead className="sticky top-0 z-10 bg-gray-100">
            {/* Sprint 8 T9 follow-up: 원생 + 요약 4컬럼 모두 sticky left 누적.
                좌측 가로 스크롤 시 일자 영역만 이동하고 원생/요약 컬럼은 시야 유지.
                너비는 헤더 텍스트 기준 ~120% — 출석/결석(2글자) 88px, 보강필요/완료(4글자) 120px.
                offset 누적: 0 → 140 → 228 → 316 → 436 (총 sticky 너비 556px). */}
            <tr>
              <th
                rowSpan={2}
                className="sticky left-0 z-20 w-[140px] min-w-[140px] border-b border-r border-[var(--border)] bg-amber-100 px-3 py-2 text-left"
              >
                원생
              </th>
              <th
                rowSpan={2}
                className="sticky left-[140px] z-20 w-[62px] min-w-[62px] border-b border-r border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
              >
                출석
                <div className="text-xs text-gray-600">(일)</div>
              </th>
              {/* Sprint 9 T7 (A41 흡수): absent_count 는 status='absent' AND makeup_attendance_id IS NULL
                  만 카운트 — 보강완료/소멸 제외. "미처리 결석" 으로 의미 명확화. */}
              <th
                rowSpan={2}
                className="sticky left-[202px] z-20 w-[62px] min-w-[62px] border-b border-r border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
                title="status='absent' AND makeup_attendance_id IS NULL — 보강완료·소멸 결석은 제외"
              >
                미처리
                <div>결석</div>
                <div className="text-xs text-gray-600">(일)</div>
              </th>
              <th
                rowSpan={2}
                className="sticky left-[264px] z-20 w-[84px] min-w-[84px] border-b border-r border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
              >
                보강필요
                <div className="text-xs text-gray-600">(시간)</div>
              </th>
              <th
                rowSpan={2}
                className="sticky left-[348px] z-20 w-[84px] min-w-[84px] border-b border-r-2 border-r-[var(--border)] border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
              >
                보강완료
                <div className="text-xs text-gray-600">(시간)</div>
              </th>
              {days.map((d) => {
                const wd = weekdayLabel(grid.yearMonth, d)
                const isWeekend = wd === '토' || wd === '일'
                return (
                  <th
                    key={`wd-${d}`}
                    className={`min-w-[44px] border-b border-r border-[var(--border)] px-1 py-1 text-center text-xs ${
                      isWeekend ? 'text-red-600' : 'text-gray-600'
                    }`}
                  >
                    {wd}
                  </th>
                )
              })}
            </tr>
            <tr>
              {days.map((d) => (
                <th
                  key={`d-${d}`}
                  className="min-w-[44px] border-b border-r border-[var(--border)] px-1 py-2 text-center text-sm"
                >
                  {d}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {grid.students.map((student) => (
              <StudentRow
                key={student.studentId}
                student={student}
                days={days}
                yearMonth={grid.yearMonth}
                onCellClick={handleCellClick}
                onCellContextMenu={(cell) => {
                  if (cell.status === 'absent') {
                    setMemoDialogCell(cell)
                  }
                }}
                onNonClassDayClick={onNonClassDayClick}
                onMakeupCellClick={onMakeupCellClick}
                onStudentNameClick={onStudentNameClick}
              />
            ))}
          </tbody>
        </table>
      </div>

      <p className="mt-3 text-sm text-gray-600">
        셀 클릭 = 출석↔결석 토글 · 결석 셀 우클릭 = 사유 메모 · Ctrl+Z (또는 Cmd+Z) = 마지막 토글 취소
      </p>

      {memoDialogCell !== null && (
        <AbsenceMemoDialog
          cell={memoDialogCell}
          onSave={async (memo) => {
            try {
              await updateAbsenceMemo(memoDialogCell.id, memo)
              void queryClient.invalidateQueries({
                queryKey: ['attendance-grid', grid.yearMonth],
              })
              setMemoDialogCell(null)
            } catch (e) {
              setError(typeof e === 'string' ? e : (e as Error).message)
            }
          }}
          onClose={() => setMemoDialogCell(null)}
        />
      )}
    </>
  )
}

interface StudentRowProps {
  student: AttendanceGridType['students'][number]
  days: number[]
  yearMonth: string
  onCellClick: (cell: AttendanceCell) => void
  onCellContextMenu: (cell: AttendanceCell) => void
  onNonClassDayClick?: (studentId: number, eventDate: string) => void
  /** Sprint 9 T7 — makeup_done 셀 클릭 시 보강 관리 다이얼로그 호출. */
  onMakeupCellClick?: (studentId: number, cell: AttendanceCell) => void
  /** Sprint 9 T8 — 학생명 클릭 시 결석 이력 다이얼로그 호출. */
  onStudentNameClick?: (studentId: number) => void
}

const StudentRow = memo(function StudentRow({
  student,
  days,
  yearMonth,
  onCellClick,
  onCellContextMenu,
  onNonClassDayClick,
  onMakeupCellClick,
  onStudentNameClick,
}: StudentRowProps) {
  // makeup_done 셀 클릭 시 보강 관리 다이얼로그로 분기, 그 외엔 일반 토글.
  function handleCellClick(cell: AttendanceCell) {
    if (cell.status === 'makeup_done' && onMakeupCellClick !== undefined) {
      onMakeupCellClick(student.studentId, cell)
      return
    }
    onCellClick(cell)
  }
  const byDay = useMemo(
    () => buildAttendanceByDay(student.attendances),
    [student.attendances],
  )

  return (
    <tr className="hover:bg-gray-50">
      {/* Sprint 8 T9 follow-up: 데이터 셀도 헤더와 동일한 sticky left offset 적용.
          z-10 (헤더 z-20 보다 낮음) — 가로 스크롤 시 일자 셀 위로 덮이고, 헤더 행 아래로 숨음. */}
      <th
        scope="row"
        className="sticky left-0 z-10 w-[140px] min-w-[140px] border-b border-r border-[var(--border)] bg-amber-50 px-3 py-2 text-left text-base font-medium"
      >
        {onStudentNameClick === undefined ? (
          <>
            <div>{student.name}</div>
            <div className="text-xs text-gray-500">#{student.serialNo}</div>
          </>
        ) : (
          <button
            type="button"
            onClick={() => onStudentNameClick(student.studentId)}
            className="block w-full text-left hover:text-[var(--accent)] hover:underline"
            title="결석 이력 보기"
          >
            <div>{student.name}</div>
            <div className="text-xs text-gray-500">#{student.serialNo}</div>
          </button>
        )}
      </th>
      <td className="sticky left-[140px] z-10 w-[62px] min-w-[62px] border-b border-r border-[var(--border)] bg-amber-50 px-2 py-2 text-center">
        {student.summary.presentCount}
      </td>
      <td className="sticky left-[202px] z-10 w-[62px] min-w-[62px] border-b border-r border-[var(--border)] bg-amber-50 px-2 py-2 text-center">
        {student.summary.absentCount}
      </td>
      <td className="sticky left-[264px] z-10 w-[84px] min-w-[84px] border-b border-r border-[var(--border)] bg-amber-50 px-2 py-2 text-center">
        {minutesToHours(student.summary.makeupNeededMinutes)}
      </td>
      <td className="sticky left-[348px] z-10 w-[84px] min-w-[84px] border-b border-r-2 border-r-[var(--border)] border-[var(--border)] bg-amber-50 px-2 py-2 text-center">
        {minutesToHours(student.summary.makeupCompletedMinutes)}
      </td>
      {days.map((day) => {
        const dayKey = String(day).padStart(2, '0')
        const cell = byDay.get(dayKey)
        const eventDate = `${yearMonth}-${dayKey}`
        return (
          <CellView
            key={day}
            cell={cell ?? null}
            onClick={handleCellClick}
            onContextMenu={onCellContextMenu}
            onEmptyCellClick={
              onNonClassDayClick === undefined
                ? undefined
                : () => onNonClassDayClick(student.studentId, eventDate)
            }
          />
        )
      })}
    </tr>
  )
})

interface CellViewProps {
  cell: AttendanceCell | null
  onClick: (cell: AttendanceCell) => void
  onContextMenu: (cell: AttendanceCell) => void
  /** Sprint 9 T6 — 비수업일(=cell null) 클릭 시 호출 (보강 등록 진입). */
  onEmptyCellClick?: () => void
}

/** 출결 셀 — 클릭 토글 + 우클릭 메모. 비수업일은 회색 placeholder.
 *  Sprint 9 T6: 비수업일 셀에 클릭 핸들러가 주어지면 보강 등록 진입점이 된다. */
function CellView({ cell, onClick, onContextMenu, onEmptyCellClick }: CellViewProps) {
  if (cell === null) {
    if (onEmptyCellClick === undefined) {
      return (
        <td
          aria-label="수업일 아님"
          className="min-w-[44px] border-b border-r border-[var(--border)] bg-gray-50"
        />
      )
    }
    return (
      <td
        className="min-w-[44px] cursor-pointer border-b border-r border-[var(--border)] bg-gray-50 p-0 text-center align-middle hover:bg-amber-50"
        onClick={onEmptyCellClick}
        title="비수업일 — 클릭하여 보강 등록"
      >
        <button
          type="button"
          aria-label="보강 등록"
          className="block h-[44px] w-full min-w-[44px] text-base text-gray-400 hover:text-amber-700"
        >
          +
        </button>
      </td>
    )
  }

  const cls = statusCellClass(cell.status)
  return (
    <td
      className={`min-w-[44px] cursor-pointer border-b border-r border-[var(--border)] p-0 text-center align-middle ${cls.cell}`}
      onClick={() => onClick(cell)}
      onContextMenu={(e) => {
        e.preventDefault()
        onContextMenu(cell)
      }}
      title={cellTooltip(cell)}
    >
      <button
        type="button"
        aria-label={`${cell.eventDate} ${cls.label}`}
        className="block h-[44px] w-full min-w-[44px] text-base"
      >
        {cls.label}
        {cell.absenceMemo !== null && cell.status === 'absent' && (
          <span className="ml-0.5 text-xs">*</span>
        )}
      </button>
    </td>
  )
}

function statusCellClass(status: AttendanceStatus): {
  cell: string
  label: string
} {
  switch (status) {
    case 'present':
      return { cell: 'bg-white hover:bg-gray-100', label: '○' }
    case 'absent':
      return { cell: 'bg-red-100 text-red-900 font-bold hover:bg-red-200', label: '×' }
    case 'makeup_done':
      return { cell: 'bg-red-50 text-red-700 font-bold', label: '보강' }
    case 'makeup_expired':
      return { cell: 'bg-gray-200 text-gray-600', label: '소멸' }
  }
}

function cellTooltip(cell: AttendanceCell): string {
  const parts = [cell.eventDate, statusCellClass(cell.status).label]
  if (cell.absenceMemo !== null) parts.push(`메모: ${cell.absenceMemo}`)
  if (cell.makeupDeadline !== null) parts.push(`소멸기한: ${cell.makeupDeadline}`)
  return parts.join(' · ')
}
