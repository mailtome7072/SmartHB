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

export function AttendanceGrid({ grid }: Props) {
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
            <tr>
              <th className="sticky left-0 z-20 min-w-[140px] border-b border-r border-[var(--border)] bg-gray-100 px-3 py-2 text-left">
                원생
              </th>
              {days.map((d) => (
                <th
                  key={d}
                  className="min-w-[44px] border-b border-r border-[var(--border)] px-1 py-2 text-center text-sm"
                >
                  {d}
                </th>
              ))}
              <th className="min-w-[80px] border-b border-r border-[var(--border)] px-2 py-2 text-center text-sm">
                출석
              </th>
              <th className="min-w-[80px] border-b border-r border-[var(--border)] px-2 py-2 text-center text-sm">
                결석
              </th>
              <th className="min-w-[100px] border-b border-r border-[var(--border)] px-2 py-2 text-center text-sm">
                보강필요(분)
              </th>
              <th className="min-w-[100px] border-b border-[var(--border)] px-2 py-2 text-center text-sm">
                보강완료(분)
              </th>
            </tr>
          </thead>
          <tbody>
            {grid.students.map((student) => (
              <StudentRow
                key={student.studentId}
                student={student}
                days={days}
                onCellClick={handleCellClick}
                onCellContextMenu={(cell) => {
                  if (cell.status === 'absent') {
                    setMemoDialogCell(cell)
                  }
                }}
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
  onCellClick: (cell: AttendanceCell) => void
  onCellContextMenu: (cell: AttendanceCell) => void
}

const StudentRow = memo(function StudentRow({
  student,
  days,
  onCellClick,
  onCellContextMenu,
}: StudentRowProps) {
  const byDay = useMemo(
    () => buildAttendanceByDay(student.attendances),
    [student.attendances],
  )

  return (
    <tr className="hover:bg-gray-50">
      <th
        scope="row"
        className="sticky left-0 z-10 min-w-[140px] border-b border-r border-[var(--border)] bg-white px-3 py-2 text-left text-base font-medium"
      >
        <div>{student.name}</div>
        <div className="text-xs text-gray-500">#{student.serialNo}</div>
      </th>
      {days.map((day) => {
        const dayKey = String(day).padStart(2, '0')
        const cell = byDay.get(dayKey)
        return (
          <CellView
            key={day}
            cell={cell ?? null}
            onClick={onCellClick}
            onContextMenu={onCellContextMenu}
          />
        )
      })}
      <td className="border-b border-r border-[var(--border)] px-2 py-2 text-center">
        {student.summary.presentCount}
      </td>
      <td className="border-b border-r border-[var(--border)] px-2 py-2 text-center">
        {student.summary.absentCount}
      </td>
      <td className="border-b border-r border-[var(--border)] px-2 py-2 text-center">
        {student.summary.makeupNeededMinutes}
      </td>
      <td className="border-b border-[var(--border)] px-2 py-2 text-center">
        {student.summary.makeupCompletedMinutes}
      </td>
    </tr>
  )
})

interface CellViewProps {
  cell: AttendanceCell | null
  onClick: (cell: AttendanceCell) => void
  onContextMenu: (cell: AttendanceCell) => void
}

/** 출결 셀 — 클릭 토글 + 우클릭 메모. 비수업일은 회색 placeholder. */
function CellView({ cell, onClick, onContextMenu }: CellViewProps) {
  if (cell === null) {
    return (
      <td
        aria-label="수업일 아님"
        className="min-w-[44px] border-b border-r border-[var(--border)] bg-gray-50"
      />
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
