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
  AttendanceGridStudent,
  AttendanceStatus,
  DaySchedule,
  GridMakeupCell,
} from '@/types/attendance'
import { minutesToHoursText } from '@/lib/time'
import { AbsenceMemoDialog } from './AbsenceMemoDialog'

/**
 * 비수업일 셀의 "+" 표시 여부 — Sprint 9 Session #10 I8.
 *
 * 사용자 룰 (T10 백엔드 get_makeup_eligible_dates 와 일치):
 * - 학생 입퇴교 범위 내
 * - 케이스 A: 평일(월~금) + 보강불가 코드(is_block) 없음
 * - 케이스 B: allowsMakeup 코드 명시 (요일 무관)
 */
function isMakeupEligibleForCell(
  student: AttendanceGridStudent,
  eventDate: string,
  yearMonth: string,
  daySchedule: DaySchedule | undefined,
): boolean {
  // 학생 입퇴교 범위 외 → 차단
  if (eventDate < student.enrollDate) return false
  if (student.withdrawDate !== null && eventDate > student.withdrawDate) {
    return false
  }
  // 케이스 B 우선 — allowsMakeup 코드 명시
  if (daySchedule !== undefined && daySchedule.allowsMakeup) return true
  // 보강불가 코드 차단
  if (daySchedule !== undefined && daySchedule.isBlock) return false
  // 케이스 A — 평일 + 보강불가 코드 없음
  const [year, month] = yearMonth.split('-').map(Number)
  const day = Number(eventDate.slice(8, 10))
  const dow = new Date(year, month - 1, day).getDay() // 0=일, 6=토
  return dow >= 1 && dow <= 5
}

interface Props {
  grid: AttendanceGridType
  /** Sprint 9 T6 — 비수업일(보강 가능 후보) 셀 클릭 시 호출. */
  onNonClassDayClick?: (studentId: number, eventDate: string) => void
  /** Sprint 9 Session #10 J6 — 보강일 셀 클릭 시 호출 (보강 관리 다이얼로그 진입).
   *  기존 onMakeupCellClick (결석 셀 진입) 은 J6 정책으로 폐기 — 보강일 셀에서만 진입. */
  onMakeupDayCellClick?: (studentId: number, makeup: GridMakeupCell) => void
  /** Sprint 9 T8 — 학생명 클릭 시 호출 (결석 이력 다이얼로그 진입). */
  onStudentNameClick?: (studentId: number) => void
  /** Sprint 9 Session #12 K3 — 정규 수업 셀(present/makeup_done/makeup_expired) 우클릭 시
   *  보강 등록 진입. 결석(absent) 셀 우클릭은 기존 메모 동작 유지. */
  onClassDayMakeupRegister?: (studentId: number, eventDate: string) => void
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
  onMakeupDayCellClick,
  onStudentNameClick,
  onClassDayMakeupRegister,
}: Props) {
  const queryClient = useQueryClient()
  const [lastToggle, setLastToggle] = useState<LastToggle | null>(null)
  const [memoDialogCell, setMemoDialogCell] = useState<AttendanceCell | null>(null)
  const [error, setError] = useState<string | null>(null)

  const days = useMemo(() => daysOfMonth(grid.yearMonth), [grid.yearMonth])

  // Session #10 I7/I8 — 일자별 학사일정 매핑 (event_date → DaySchedule).
  const dayScheduleMap = useMemo(() => {
    const map = new Map<string, DaySchedule>()
    for (const d of grid.daySchedules) map.set(d.eventDate, d)
    return map
  }, [grid.daySchedules])

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
                const eventDate = `${grid.yearMonth}-${String(d).padStart(2, '0')}`
                const sched = dayScheduleMap.get(eventDate)
                // Session #12 K4: 단원평가 응시일은 배경색 제거(일반 평일과 동일 표기).
                // 보강데이/공휴수업일 등 그 외 allowsMakeup=true 일자만 sky 배경 유지.
                const isAssessment = sched?.label === '단원평가 응시일'
                const showSkyBg = sched?.allowsMakeup === true && !isAssessment
                return (
                  <th
                    key={`wd-${d}`}
                    title={sched?.label}
                    className={`min-w-[44px] border-b border-r border-[var(--border)] px-1 py-1 text-center text-xs ${
                      showSkyBg
                        ? 'bg-sky-100 text-sky-800 font-semibold'
                        : isWeekend
                          ? 'text-red-600'
                          : 'text-gray-600'
                    }`}
                  >
                    {wd}
                  </th>
                )
              })}
            </tr>
            <tr>
              {days.map((d) => {
                const eventDate = `${grid.yearMonth}-${String(d).padStart(2, '0')}`
                const sched = dayScheduleMap.get(eventDate)
                const isAssessment = sched?.label === '단원평가 응시일'
                const showSkyBg = sched?.allowsMakeup === true && !isAssessment
                // K4: 보강데이는 날짜 밑에 작은 폰트 라벨 — 셀 너비 변경 없도록 absolute 또는 leading-none.
                const isMakeupDayLabel = sched?.label === '보강데이'
                return (
                  <th
                    key={`d-${d}`}
                    title={sched?.label}
                    className={`min-w-[44px] border-b border-r border-[var(--border)] px-1 py-2 text-center text-sm leading-tight ${
                      showSkyBg ? 'bg-sky-100 text-sky-800 font-semibold' : ''
                    }`}
                  >
                    {d}
                    {isMakeupDayLabel && (
                      <div className="text-[10px] font-normal leading-none text-sky-700">
                        보강데이
                      </div>
                    )}
                  </th>
                )
              })}
            </tr>
          </thead>
          <tbody>
            {grid.students.map((student) => (
              <StudentRow
                key={student.studentId}
                student={student}
                days={days}
                yearMonth={grid.yearMonth}
                dayScheduleMap={dayScheduleMap}
                onCellClick={handleCellClick}
                onCellContextMenu={(cell, studentId) => {
                  // Session #12 K3: 결석 셀 = 메모, 그 외(present/makeup_done/makeup_expired) = 보강 등록.
                  if (cell.status === 'absent') {
                    setMemoDialogCell(cell)
                  } else if (onClassDayMakeupRegister !== undefined) {
                    onClassDayMakeupRegister(studentId, cell.eventDate)
                  }
                }}
                onNonClassDayClick={onNonClassDayClick}
                onMakeupDayCellClick={onMakeupDayCellClick}
                onStudentNameClick={onStudentNameClick}
              />
            ))}
          </tbody>
        </table>
      </div>

      <p className="mt-3 text-sm text-gray-600">
        셀 클릭 = 출석↔결석 토글 · 결석 셀 우클릭 = 사유 메모 · 출석/보강완료 셀 우클릭 = 보강 등록 · Ctrl+Z (또는 Cmd+Z) = 마지막 토글 취소
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
  /** Sprint 9 Session #10 I7/I8 — 일자별 학사일정 매핑. */
  dayScheduleMap: Map<string, DaySchedule>
  onCellClick: (cell: AttendanceCell) => void
  /** Session #12 K3 — studentId 동봉 시그니처. */
  onCellContextMenu: (cell: AttendanceCell, studentId: number) => void
  onNonClassDayClick?: (studentId: number, eventDate: string) => void
  /** Sprint 9 Session #10 J6 — 보강일 셀 클릭 시 보강 관리 다이얼로그 호출. */
  onMakeupDayCellClick?: (studentId: number, makeup: GridMakeupCell) => void
  /** Sprint 9 T8 — 학생명 클릭 시 결석 이력 다이얼로그 호출. */
  onStudentNameClick?: (studentId: number) => void
}

const StudentRow = memo(function StudentRow({
  student,
  days,
  yearMonth,
  dayScheduleMap,
  onCellClick,
  onCellContextMenu,
  onNonClassDayClick,
  onMakeupDayCellClick,
  onStudentNameClick,
}: StudentRowProps) {
  // Session #10 J6: 결석 셀(makeup_done 포함) 은 일반 토글 / 보강일 셀 클릭은 별도.
  // handleCellClick 은 makeup_done 분기 없음 — 백엔드가 차단 메시지 반환.
  function handleCellClick(cell: AttendanceCell) {
    onCellClick(cell)
  }
  const byDay = useMemo(
    () => buildAttendanceByDay(student.attendances),
    [student.attendances],
  )
  // J4: 학생별 보강 출결 매핑 (event_date → GridMakeupCell). 비수업일 셀에 표시.
  const makeupsByDay = useMemo(() => {
    const map = new Map<string, GridMakeupCell>()
    for (const m of student.makeups) {
      map.set(m.eventDate.slice(8, 10), m)
    }
    return map
  }, [student.makeups])
  // J8: 보강 ID → 매칭된 결석 일자 목록 (보강 셀 hover 시 결석일자 표시용).
  const absenceDatesByMakeupId = useMemo(() => {
    const map = new Map<number, string[]>()
    for (const a of student.attendances) {
      if (a.makeupAttendanceId === null) continue
      const list = map.get(a.makeupAttendanceId) ?? []
      list.push(a.eventDate)
      map.set(a.makeupAttendanceId, list)
    }
    return map
  }, [student.attendances])
  // J8: 결석 셀(makeup_done) → 보강 일자 lookup (id → eventDate).
  const makeupEventDateById = useMemo(() => {
    const map = new Map<number, string>()
    for (const m of student.makeups) map.set(m.id, m.eventDate)
    return map
  }, [student.makeups])

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
      {(() => {
        const pending = pendingTooltip(student)
        return (
          <td
            title={pending}
            className={`sticky left-[264px] z-10 w-[84px] min-w-[84px] border-b border-r border-[var(--border)] bg-amber-50 px-2 py-2 text-center ${
              pending !== undefined ? 'cursor-help' : ''
            }`}
          >
            {minutesToHoursText(student.summary.makeupNeededMinutes)}
          </td>
        )
      })()}
      {(() => {
        const completed = completedTooltip(student)
        return (
          <td
            title={completed}
            className={`sticky left-[348px] z-10 w-[84px] min-w-[84px] border-b border-r-2 border-r-[var(--border)] border-[var(--border)] bg-amber-50 px-2 py-2 text-center ${
              completed !== undefined ? 'cursor-help' : ''
            }`}
          >
            {minutesToHoursText(student.summary.makeupCompletedMinutes)}
          </td>
        )
      })()}
      {days.map((day) => {
        const dayKey = String(day).padStart(2, '0')
        const cell = byDay.get(dayKey)
        const eventDate = `${yearMonth}-${dayKey}`
        // J4: 비수업일 셀에 보강 진행 정보 (해당 일자 makeup 등록 시).
        const makeupOnThisDay = cell === undefined ? makeupsByDay.get(dayKey) : undefined
        // I8: 비수업일 셀 사전 판단 — 보강 불가 일자는 "+" 자체 비표시.
        // J4 보강이 있는 비수업일에는 "+" 대신 보강 표기.
        // K1' (Session #12): 만기 미도래 미보강 결석이 이 일자 이전에 있을 때만 "+" 표시.
        const earliestPending = student.earliestPendingAbsenceDate
        const hasPriorPendingAbsence =
          earliestPending !== null && earliestPending < eventDate
        const isEligible =
          onNonClassDayClick !== undefined &&
          cell === undefined &&
          makeupOnThisDay === undefined &&
          hasPriorPendingAbsence &&
          isMakeupEligibleForCell(
            student,
            eventDate,
            yearMonth,
            dayScheduleMap.get(eventDate),
          )
        // J8: hint — 결석 셀에는 매칭된 보강일, 보강 셀에는 충당 결석일자 목록.
        const cellMakeupHintDate =
          cell?.status === 'makeup_done' && cell.makeupAttendanceId !== null
            ? makeupEventDateById.get(cell.makeupAttendanceId)
            : undefined
        const makeupAbsenceHintDates =
          makeupOnThisDay !== undefined
            ? absenceDatesByMakeupId.get(makeupOnThisDay.id)
            : undefined
        return (
          <CellView
            key={day}
            cell={cell ?? null}
            makeup={makeupOnThisDay}
            cellMakeupHintDate={cellMakeupHintDate}
            makeupAbsenceHintDates={makeupAbsenceHintDates}
            onClick={handleCellClick}
            onContextMenu={(c) => onCellContextMenu(c, student.studentId)}
            onEmptyCellClick={
              isEligible
                ? () => onNonClassDayClick!(student.studentId, eventDate)
                : undefined
            }
            onMakeupClick={
              makeupOnThisDay !== undefined && onMakeupDayCellClick !== undefined
                ? () => onMakeupDayCellClick(student.studentId, makeupOnThisDay)
                : undefined
            }
          />
        )
      })}
    </tr>
  )
})

interface CellViewProps {
  cell: AttendanceCell | null
  /** Sprint 9 Session #10 J4 — 비수업일에 보강이 등록된 경우 표시할 보강 정보. */
  makeup?: GridMakeupCell
  /** Sprint 9 Session #10 J8 — 결석(makeup_done) 셀 hover 시 매칭된 보강일자. */
  cellMakeupHintDate?: string
  /** Sprint 9 Session #10 J8 — 보강 셀 hover 시 충당 결석 일자 목록. */
  makeupAbsenceHintDates?: string[]
  onClick: (cell: AttendanceCell) => void
  onContextMenu: (cell: AttendanceCell) => void
  /** Sprint 9 T6 — 비수업일(=cell null) 클릭 시 호출 (보강 등록 진입). */
  onEmptyCellClick?: () => void
  /** Sprint 9 Session #10 J6 — 보강 셀 클릭 시 보강 관리 다이얼로그 진입. */
  onMakeupClick?: () => void
}

/** 출결 셀 — 클릭 토글 + 우클릭 메모. 비수업일은 회색 placeholder.
 *  Sprint 9 T6: 비수업일 셀에 클릭 핸들러가 주어지면 보강 등록 진입점이 된다.
 *  Sprint 9 Session #10 J4: 비수업일에 보강이 등록된 경우 보강 정보 표시.
 *  Sprint 9 Session #10 J6: 보강 셀 클릭 시 보강 관리(삭제) 다이얼로그 진입. */
function CellView({
  cell,
  makeup,
  cellMakeupHintDate,
  makeupAbsenceHintDates,
  onClick,
  onContextMenu,
  onEmptyCellClick,
  onMakeupClick,
}: CellViewProps) {
  if (cell === null) {
    // J4 — 보강 진행 표기 (보강 미등원 개념 삭제 — Session #10 J5).
    // J6 — 클릭 시 onMakeupClick 호출 (보강 관리 다이얼로그 진입).
    // J8 — hover 시 충당 결석 일자 표시.
    if (makeup !== undefined) {
      const clickable = onMakeupClick !== undefined
      // Session #10 J9 — 보강 셀 tooltip 줄바꿈 적용.
      const tooltipLines: string[] = [
        makeup.eventDate,
        `보강 (${minutesToHoursText(makeup.classMinutes)}시간)`,
      ]
      if (
        makeupAbsenceHintDates !== undefined &&
        makeupAbsenceHintDates.length > 0
      ) {
        // Session #10 J10 — 충당 결석 다건일 경우 줄바꿈.
        if (makeupAbsenceHintDates.length === 1) {
          tooltipLines.push(`충당 결석: ${makeupAbsenceHintDates[0]}`)
        } else {
          tooltipLines.push('충당 결석:')
          for (const d of makeupAbsenceHintDates) tooltipLines.push(`  ${d}`)
        }
      }
      if (clickable) tooltipLines.push('클릭하여 관리')
      return (
        <td
          aria-label="보강 진행"
          title={tooltipLines.join('\n')}
          onClick={onMakeupClick}
          className={`min-w-[44px] border-b border-r border-[var(--border)] bg-emerald-100 p-0 text-center align-middle text-base font-bold text-emerald-800 ${
            clickable ? 'cursor-pointer hover:bg-emerald-200' : ''
          }`}
        >
          보강
        </td>
      )
    }
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
      title={cellTooltip(cell, cellMakeupHintDate)}
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
      // Session #10 J7 — 라벨 '×' → '결석'.
      return { cell: 'bg-red-100 text-red-900 font-bold hover:bg-red-200', label: '결석' }
    case 'makeup_done':
      // Session #10 J5/J7 — 결석일 셀은 보강 후에도 "결석" 표기 + 배경은 보강 셀과 동일(emerald).
      return {
        cell: 'bg-emerald-100 text-emerald-800 font-bold',
        label: '결석',
      }
    case 'makeup_expired':
      return { cell: 'bg-gray-200 text-gray-600', label: '소멸' }
  }
}

/** 보강필요 셀 hover 내역 — 이월 누적 결석 목록 (결석/보강 셀 힌트와 동일한 title 방식). */
function pendingTooltip(student: AttendanceGridStudent): string | undefined {
  if (student.pendingAbsences.length === 0) return undefined
  const lines = [`보강필요 ${minutesToHoursText(student.summary.makeupNeededMinutes)}시간`]
  for (const p of student.pendingAbsences) {
    const dl = p.makeupDeadline !== null ? `, 소멸 ${p.makeupDeadline}` : ''
    lines.push(`${p.eventDate} (${minutesToHoursText(p.classMinutes)}시간${dl})`)
  }
  return lines.join('\n')
}

/** 보강완료 셀 hover 내역 — 이번 달 보강 출결 목록. */
function completedTooltip(student: AttendanceGridStudent): string | undefined {
  const done = student.makeups.filter((m) => m.status === 'makeup_attended')
  if (done.length === 0) return undefined
  const lines = [`보강완료 ${minutesToHoursText(student.summary.makeupCompletedMinutes)}시간`]
  for (const m of done) {
    lines.push(`${m.eventDate} (${minutesToHoursText(m.classMinutes)}시간)`)
  }
  return lines.join('\n')
}

function cellTooltip(cell: AttendanceCell, makeupHintDate?: string): string {
  // Session #10 J5/J8 — makeup_done 셀은 "결석" 표시 + hover 시 매칭 보강일자 노출.
  // Session #10 J9 — 줄바꿈으로 가독성 향상.
  const parts: string[] = [cell.eventDate]
  if (cell.status === 'makeup_done') {
    parts.push(
      makeupHintDate !== undefined
        ? `결석 (보강일: ${makeupHintDate})`
        : '결석 (보강 매칭됨)',
    )
  } else {
    parts.push(statusCellClass(cell.status).label)
  }
  if (cell.absenceMemo !== null) parts.push(`메모: ${cell.absenceMemo}`)
  if (cell.makeupDeadline !== null) parts.push(`소멸기한: ${cell.makeupDeadline}`)
  return parts.join('\n')
}
