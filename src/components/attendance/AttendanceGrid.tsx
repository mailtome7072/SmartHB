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

import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react'
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
import type { SchoolLevel } from '@/types/student'
import { minutesToHoursText } from '@/lib/time'
import { isEditableTarget } from '@/components/layout/GlobalShortcuts'
import { compareKorean, useTableSort, withTiebreak } from '@/hooks/useTableSort'
import { AbsenceMemoDialog } from './AbsenceMemoDialog'

/**
 * 정렬 가능 컬럼 (Sprint 19 T2, 사용자 요청 2번) — 그리드에 실제로 표시되는 좌측 고정
 * 컬럼만 대상. 모든 comparator는 동일 값일 때 이름 가나다순으로 tie-break한다.
 *
 * `comparators`는 컴포넌트 밖 모듈 상수로 선언 — useTableSort의 useMemo가 매 렌더
 * 무효화되지 않도록 안정된 참조를 유지한다.
 */
type AttendanceSortKey = 'student' | 'present' | 'absent' | 'needed' | 'completed'

const SCHOOL_LEVEL_ORDER: Record<SchoolLevel, number> = {
  elementary: 0,
  middle: 1,
}

function compareStudentDefault(a: AttendanceGridStudent, b: AttendanceGridStudent): number {
  return (
    SCHOOL_LEVEL_ORDER[a.schoolLevel] - SCHOOL_LEVEL_ORDER[b.schoolLevel] ||
    a.grade - b.grade ||
    compareKorean(a.name, b.name)
  )
}

const ATTENDANCE_SORT_COMPARATORS: Record<
  AttendanceSortKey,
  (a: AttendanceGridStudent, b: AttendanceGridStudent) => number
> = {
  student: compareStudentDefault,
  present: withTiebreak(
    (a, b) => a.summary.presentCount - b.summary.presentCount,
    (a, b) => compareKorean(a.name, b.name),
  ),
  absent: withTiebreak(
    (a, b) => a.summary.absentCount - b.summary.absentCount,
    (a, b) => compareKorean(a.name, b.name),
  ),
  needed: withTiebreak(
    (a, b) => a.summary.makeupNeededMinutes - b.summary.makeupNeededMinutes,
    (a, b) => compareKorean(a.name, b.name),
  ),
  completed: withTiebreak(
    (a, b) => a.summary.makeupCompletedMinutes - b.summary.makeupCompletedMinutes,
    (a, b) => compareKorean(a.name, b.name),
  ),
}

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
  // 케이스 A — 평일 + 보강불가 코드 없음 (전체 ISO 날짜로 요일 산출)
  const [year, month, day] = eventDate.split('-').map(Number)
  const dow = new Date(year, month - 1, day).getDay() // 0=일, 6=토
  return dow >= 1 && dow <= 5
}

interface Props {
  grid: AttendanceGridType
  /** Sprint 21 T2 — 선택된 교습기간의 시작/종료 일자. 다월 교습기간이면 그리드 컬럼을
   *  이 범위로 생성한다. null 이면 달력월 폴백. 페이지의 listStudyPeriods 데이터에서 전달. */
  periodStartDate?: string | null
  periodEndDate?: string | null
  /** Sprint 9 T6 — 비수업일(보강 가능 후보) 셀 클릭 시 호출. */
  onNonClassDayClick?: (studentId: number, eventDate: string) => void
  /** Sprint 9 Session #10 J6 — 보강일 셀 클릭 시 호출 (보강 관리 다이얼로그 진입).
   *  기존 onMakeupCellClick (결석 셀 진입) 은 J6 정책으로 폐기 — 보강일 셀에서만 진입. */
  onMakeupDayCellClick?: (studentId: number, makeup: GridMakeupCell) => void
  /** Sprint 9 T8 — 학생명 클릭 시 호출 (결석 이력 다이얼로그 진입). */
  onStudentNameClick?: (studentId: number) => void
  /** Sprint 9 Session #12 K3 — 정규 수업 셀(makeup_done/makeup_expired) 우클릭 시
   *  보강 등록 진입. 결석(absent) 셀 우클릭은 기존 메모 동작 유지. */
  onClassDayMakeupRegister?: (studentId: number, eventDate: string) => void
  /** Sprint 16 T0 — present(출석) 셀 우클릭 시 [수업일 이동 / 보강 등록] 액션 선택. */
  onPresentCellAction?: (studentId: number, cell: AttendanceCell) => void
}

interface LastToggle {
  attendanceId: number
  previousStatus: AttendanceStatus
}

/** 교습기간 start_date~end_date 의 모든 날짜(YYYY-MM-DD) 배열. 다월 교습기간이면
 *  달력월 밖 날짜(7/30, 9/1 등)도 포함한다 (Sprint 21 T2). */
function periodDates(startDate: string, endDate: string): string[] {
  const [sy, sm, sd] = startDate.split('-').map(Number)
  const cur = new Date(sy, sm - 1, sd)
  const out: string[] = []
  // 안전 상한(교습기간은 최대 ~6주) — 무한 루프 방지.
  for (let i = 0; i < 400; i++) {
    const iso = `${cur.getFullYear()}-${String(cur.getMonth() + 1).padStart(2, '0')}-${String(
      cur.getDate(),
    ).padStart(2, '0')}`
    if (iso > endDate) break
    out.push(iso)
    cur.setDate(cur.getDate() + 1)
  }
  return out
}

/** 폴백 — 교습기간 범위를 알 수 없을 때 달력월 1~말일을 ISO 날짜로 생성. */
function calendarMonthDates(yearMonth: string): string[] {
  const [year, month] = yearMonth.split('-').map(Number)
  const lastDay = new Date(year, month, 0).getDate()
  return Array.from(
    { length: lastDay },
    (_, i) => `${yearMonth}-${String(i + 1).padStart(2, '0')}`,
  )
}

const WEEKDAY_LABEL = ['일', '월', '화', '수', '목', '금', '토'] as const

/** ISO 날짜(YYYY-MM-DD) → 한글 요일 (일~토). */
function weekdayLabel(iso: string): string {
  const [year, month, day] = iso.split('-').map(Number)
  return WEEKDAY_LABEL[new Date(year, month - 1, day).getDay()]
}

/** 컬럼 헤더 표시 — 주 월(primaryYm) 날짜는 일(day)만, 이웃 달 날짜는 월/일 표기. */
function headerDayLabel(iso: string, primaryYm: string): string {
  const day = Number(iso.slice(8, 10))
  if (iso.slice(0, 7) === primaryYm) return String(day)
  return `${Number(iso.slice(5, 7))}/${day}`
}

/** 날짜(전체 ISO)에 해당하는 출결 셀 검색 (Map). 일(DD) 대신 전체 날짜 키 — 다월 충돌 해소. */
function buildAttendanceByDate(
  attendances: AttendanceCell[],
): Map<string, AttendanceCell> {
  const map = new Map<string, AttendanceCell>()
  for (const a of attendances) map.set(a.eventDate, a)
  return map
}

export function AttendanceGrid({
  grid,
  periodStartDate,
  periodEndDate,
  onNonClassDayClick,
  onMakeupDayCellClick,
  onStudentNameClick,
  onClassDayMakeupRegister,
  onPresentCellAction,
}: Props) {
  const queryClient = useQueryClient()
  const [lastToggle, setLastToggle] = useState<LastToggle | null>(null)
  const [memoDialogCell, setMemoDialogCell] = useState<AttendanceCell | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Sprint 21 T2: 컬럼을 교습기간 날짜 범위로 생성(다월 걸침 포함). 범위 미확보 시 달력월 폴백.
  const dates = useMemo(
    () =>
      periodStartDate != null && periodEndDate != null
        ? periodDates(periodStartDate, periodEndDate)
        : calendarMonthDates(grid.yearMonth),
    [periodStartDate, periodEndDate, grid.yearMonth],
  )

  // Session #10 I7/I8 — 일자별 학사일정 매핑 (event_date → DaySchedule).
  const dayScheduleMap = useMemo(() => {
    const map = new Map<string, DaySchedule>()
    for (const d of grid.daySchedules) map.set(d.eventDate, d)
    return map
  }, [grid.daySchedules])

  // Sprint 19 T2(사용자 요청 1·2번) — 기본 정렬 학년별+이름, 헤더 클릭으로 재정렬.
  const { sorted: sortedStudents, toggleSort, indicator } = useTableSort<
    AttendanceGridStudent,
    AttendanceSortKey
  >(grid.students, ATTENDANCE_SORT_COMPARATORS, { key: 'student', direction: 'asc' })

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
      // P0-6: 텍스트 입력 중(결석 메모 textarea 등)에는 출결 undo 가 아니라 입력 undo 가
      // 기대 동작 — 가로채면 사용자가 인지 못 한 채 출결이 역토글된다.
      if (isEditableTarget(e.target)) return
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

  // 진입/월 변경 시 오늘 날짜 열을 가로 스크롤로 노출 — 오늘 출결을 바로 입력하도록.
  // 조회월이 현재월일 때만 해당 일(day), 아니면 null.
  const scrollRef = useRef<HTMLDivElement>(null)
  const todayRef = useRef<HTMLTableCellElement>(null)
  const todayIso = useMemo(() => {
    const now = new Date()
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(
      now.getDate(),
    ).padStart(2, '0')}`
  }, [])

  useEffect(() => {
    if (!dates.includes(todayIso)) return
    const raf = requestAnimationFrame(() => {
      const cell = todayRef.current
      const container = scrollRef.current
      if (cell === null || container === null) return
      // 좌측 고정 컬럼 너머에 오늘 열이 보이도록 컨테이너 중앙 정렬.
      const target = cell.offsetLeft - container.clientWidth / 2 + cell.clientWidth / 2
      container.scrollTo({ left: Math.max(0, target), behavior: 'smooth' })
    })
    return () => cancelAnimationFrame(raf)
  }, [dates, todayIso])

  if (grid.students.length === 0) {
    return (
      <p className="text-gray-600">
        해당 월에 등록된 원생 출결이 없습니다. 교습기간 확정 + 원생 등록 상태를 확인하세요.
      </p>
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      {error !== null && (
        <div
          role="alert"
          className="mb-3 shrink-0 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
        >
          {error}
        </div>
      )}

      {/* Sprint 19 T2: 이 div가 유일한 스크롤 컨테이너(상하+좌우) — 부모(attendance/page.tsx
          section)는 overflow-hidden으로 바뀌어 더 이상 독립적으로 스크롤하지 않는다.
          그 결과 가로 스크롤바가 항상 이 박스 하단(뷰포트 내 고정 위치)에서 즉시 접근 가능하다. */}
      <div
        ref={scrollRef}
        className="min-h-0 flex-1 overflow-auto rounded-lg border border-[var(--border)]"
      >
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
                <button
                  type="button"
                  onClick={() => toggleSort('student')}
                  className="hover:text-[var(--accent)]"
                  aria-label="원생(학년+이름) 정렬 토글"
                >
                  원생{indicator('student')}
                </button>
              </th>
              <th
                rowSpan={2}
                className="sticky left-[140px] z-20 w-[62px] min-w-[62px] border-b border-r border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
              >
                <button
                  type="button"
                  onClick={() => toggleSort('present')}
                  className="hover:text-[var(--accent)]"
                  aria-label="출석 일수 정렬 토글"
                >
                  출석{indicator('present')}
                </button>
                <div className="text-sm text-gray-600">(일)</div>
              </th>
              {/* Sprint 9 T7 (A41 흡수): absent_count 는 status='absent' AND makeup_attendance_id IS NULL
                  만 카운트 — 보강완료/소멸 제외. "미처리 결석" 으로 의미 명확화. */}
              <th
                rowSpan={2}
                className="sticky left-[202px] z-20 w-[62px] min-w-[62px] border-b border-r border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
                title="status='absent' AND makeup_attendance_id IS NULL — 보강완료·소멸 결석은 제외"
              >
                <button
                  type="button"
                  onClick={() => toggleSort('absent')}
                  className="hover:text-[var(--accent)]"
                  aria-label="미처리 결석 정렬 토글"
                >
                  미처리{indicator('absent')}
                </button>
                <div>결석</div>
                <div className="text-sm text-gray-600">(일)</div>
              </th>
              <th
                rowSpan={2}
                className="sticky left-[264px] z-20 w-[84px] min-w-[84px] border-b border-r border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
              >
                <button
                  type="button"
                  onClick={() => toggleSort('needed')}
                  className="hover:text-[var(--accent)]"
                  aria-label="보강필요 시간 정렬 토글"
                >
                  보강필요{indicator('needed')}
                </button>
                <div className="text-sm text-gray-600">(시간)</div>
              </th>
              <th
                rowSpan={2}
                className="sticky left-[348px] z-20 w-[84px] min-w-[84px] border-b border-r-2 border-r-[var(--border)] border-[var(--border)] bg-amber-100 px-2 py-2 text-center text-sm leading-tight"
              >
                <button
                  type="button"
                  onClick={() => toggleSort('completed')}
                  className="hover:text-[var(--accent)]"
                  aria-label="보강완료 시간 정렬 토글"
                >
                  보강완료{indicator('completed')}
                </button>
                <div className="text-sm text-gray-600">(시간)</div>
              </th>
              {dates.map((eventDate) => {
                const wd = weekdayLabel(eventDate)
                const isWeekend = wd === '토' || wd === '일'
                const sched = dayScheduleMap.get(eventDate)
                // Session #12 K4: 단원평가 응시일은 배경색 제거(일반 평일과 동일 표기).
                // 보강데이/공휴수업일 등 그 외 allowsMakeup=true 일자만 sky 배경 유지.
                const isAssessment = sched?.label === '단원평가 응시일'
                const showSkyBg = sched?.allowsMakeup === true && !isAssessment
                const isToday = eventDate === todayIso
                return (
                  <th
                    key={`wd-${eventDate}`}
                    title={isToday ? '오늘' : sched?.label}
                    className={`min-w-[44px] border-b border-r border-[var(--border)] px-1 py-1 text-center text-sm ${
                      isToday
                        ? 'bg-[var(--accent)] font-bold text-white'
                        : showSkyBg
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
              {dates.map((eventDate) => {
                const sched = dayScheduleMap.get(eventDate)
                const isAssessment = sched?.label === '단원평가 응시일'
                const showSkyBg = sched?.allowsMakeup === true && !isAssessment
                // K4: 보강데이는 날짜 밑에 작은 폰트 라벨 — 셀 너비 변경 없도록 absolute 또는 leading-none.
                const isMakeupDayLabel = sched?.label === '보강데이'
                const isToday = eventDate === todayIso
                return (
                  <th
                    key={`d-${eventDate}`}
                    ref={isToday ? todayRef : undefined}
                    title={isToday ? '오늘' : sched?.label}
                    className={`min-w-[44px] border-b border-r border-[var(--border)] px-1 py-2 text-center text-sm leading-tight ${
                      isToday
                        ? 'bg-[var(--accent)] font-bold text-white'
                        : showSkyBg
                          ? 'bg-sky-100 text-sky-800 font-semibold'
                          : ''
                    }`}
                  >
                    {headerDayLabel(eventDate, grid.yearMonth)}
                    {isMakeupDayLabel && (
                      <div className="text-xs font-semibold leading-none text-sky-700">
                        보강데이
                      </div>
                    )}
                  </th>
                )
              })}
            </tr>
          </thead>
          <tbody>
            {sortedStudents.map((student) => (
              <StudentRow
                key={student.studentId}
                student={student}
                dates={dates}
                dayScheduleMap={dayScheduleMap}
                onCellClick={handleCellClick}
                onCellContextMenu={(cell, studentId) => {
                  // 결석 셀 = 메모(K3).
                  // present 셀 = [수업일 이동 / 보강 등록] 액션 선택(Sprint 16 T0).
                  // makeup_done/makeup_expired = 보강 등록(K3).
                  if (cell.status === 'absent') {
                    setMemoDialogCell(cell)
                  } else if (cell.status === 'present' && onPresentCellAction !== undefined) {
                    onPresentCellAction(studentId, cell)
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

      <p className="mt-3 shrink-0 text-sm text-gray-600">
        셀 클릭 = 출석↔결석 토글 · 결석 셀 우클릭 = 사유 메모 · 출석 셀 우클릭 = 수업일 이동/보강 등록 · 보강완료 셀 우클릭 = 보강 등록 · Ctrl+Z (또는 Cmd+Z) = 마지막 토글 취소
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
    </div>
  )
}

interface StudentRowProps {
  student: AttendanceGridType['students'][number]
  /** Sprint 21 T2 — 교습기간 날짜 범위(전체 ISO 날짜 배열). */
  dates: string[]
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
  dates,
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
  const byDate = useMemo(
    () => buildAttendanceByDate(student.attendances),
    [student.attendances],
  )
  // J4: 학생별 보강 출결 매핑 (event_date → GridMakeupCell). 비수업일 셀에 표시.
  // Sprint 21 T2: 일(DD) 대신 전체 ISO 날짜 키 — 다월 충돌 해소.
  const makeupsByDate = useMemo(() => {
    const map = new Map<string, GridMakeupCell>()
    for (const m of student.makeups) {
      map.set(m.eventDate, m)
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
            <div className="text-sm text-muted-foreground">#{student.serialNo}</div>
          </>
        ) : (
          <button
            type="button"
            onClick={() => onStudentNameClick(student.studentId)}
            className="block w-full text-left hover:text-[var(--accent)] hover:underline"
            title="결석 이력 보기"
          >
            <div>{student.name}</div>
            <div className="text-sm text-muted-foreground">#{student.serialNo}</div>
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
      {dates.map((eventDate) => {
        const cell = byDate.get(eventDate)
        // J4: 비수업일 셀에 보강 진행 정보 (해당 일자 makeup 등록 시).
        const makeupOnThisDay = cell === undefined ? makeupsByDate.get(eventDate) : undefined
        // 사용자 요청 — 정규수업일에 같은 날 보강도 진행된 경우(예: 결석 발생일과 다른
        // 정규수업일에 보강) 다른 정규수업 완료 셀과 구분되도록 "+보강" 배지 표시.
        const sameDayMakeup = cell !== undefined ? makeupsByDate.get(eventDate) : undefined
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
          isMakeupEligibleForCell(student, eventDate, dayScheduleMap.get(eventDate))
        // J8: hint — 결석 셀에는 매칭된 보강일, 보강 셀에는 충당 결석일자 목록.
        const cellMakeupHintDate =
          cell?.status === 'makeup_done' && cell.makeupAttendanceId !== null
            ? makeupEventDateById.get(cell.makeupAttendanceId)
            : undefined
        const makeupAbsenceHintDates =
          makeupOnThisDay !== undefined
            ? absenceDatesByMakeupId.get(makeupOnThisDay.id)
            : undefined
        const sameDayMakeupAbsenceHintDates =
          sameDayMakeup !== undefined ? absenceDatesByMakeupId.get(sameDayMakeup.id) : undefined
        return (
          <CellView
            key={eventDate}
            cell={cell ?? null}
            makeup={makeupOnThisDay}
            cellMakeupHintDate={cellMakeupHintDate}
            makeupAbsenceHintDates={makeupAbsenceHintDates}
            sameDayMakeup={sameDayMakeup}
            sameDayMakeupAbsenceHintDates={sameDayMakeupAbsenceHintDates}
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
  /** 사용자 요청 — 정규수업일에 같은 날 보강도 진행된 경우의 보강 정보("+보강" 배지). */
  sameDayMakeup?: GridMakeupCell
  /** 같은 날 보강 배지 hover 시 충당 결석 일자 목록. */
  sameDayMakeupAbsenceHintDates?: string[]
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
  sameDayMakeup,
  sameDayMakeupAbsenceHintDates,
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
          className="block h-[44px] w-full min-w-[44px] text-base text-gray-600 hover:text-amber-700"
        >
          +
        </button>
      </td>
    )
  }

  const cls = statusCellClass(cell.status)
  // 사용자 요청 — 같은 날 보강도 진행된 정규수업 셀은 보강완료(결석)일과 동일한
  // emerald 배경으로 통일해 다른 정규수업 완료 셀과 구분되게 한다.
  const cellClass = sameDayMakeup !== undefined ? 'bg-emerald-100 text-emerald-800 font-bold' : cls.cell
  return (
    <td
      className={`min-w-[44px] cursor-pointer border-b border-r border-[var(--border)] p-0 text-center align-middle ${cellClass}`}
      onClick={() => onClick(cell)}
      onContextMenu={(e) => {
        e.preventDefault()
        onContextMenu(cell)
      }}
      title={cellTooltip(cell, cellMakeupHintDate, sameDayMakeup, sameDayMakeupAbsenceHintDates)}
    >
      <button
        type="button"
        aria-label={`${cell.eventDate} ${cls.label}${sameDayMakeup !== undefined ? ' + 보강 진행' : ''}`}
        className="block h-[44px] w-full min-w-[44px] leading-tight text-base"
      >
        {cls.label}
        {cell.absenceMemo !== null && cell.status === 'absent' && (
          <span className="ml-0.5 text-xs">*</span>
        )}
        {/* 사용자 요청 — 정규수업일에 같은 날 보강도 진행된 경우 다른 정규수업 완료
            셀과 구분되도록 작은 배지 표시. */}
        {sameDayMakeup !== undefined && (
          <span className="block text-[10px] font-semibold leading-tight text-emerald-700">
            +보강
          </span>
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

function cellTooltip(
  cell: AttendanceCell,
  makeupHintDate?: string,
  sameDayMakeup?: GridMakeupCell,
  sameDayMakeupAbsenceHintDates?: string[],
): string {
  // Session #10 J5/J8 — makeup_done 셀은 "결석" 표시 + hover 시 매칭 보강일자 노출.
  // Session #10 J9 — 줄바꿈으로 가독성 향상.
  const parts: string[] = [cell.eventDate]
  if (cell.status === 'makeup_done') {
    parts.push(
      makeupHintDate !== undefined
        ? `결석 (보강일: ${makeupHintDate})`
        : '결석 (보강 매칭됨)',
    )
  } else if (cell.status === 'present') {
    // 출석 셀 hover 시 기호('○') 대신 의미를 명확히 표기.
    parts.push('(출석)')
  } else {
    parts.push(statusCellClass(cell.status).label)
  }
  // Sprint 16 T0 케이스1 — 1회성 수업일 이동 메모 (예: "6/8(월)→6/10(수) 이동").
  if (cell.note !== null) parts.push(cell.note)
  if (cell.absenceMemo !== null) parts.push(`메모: ${cell.absenceMemo}`)
  if (cell.makeupDeadline !== null) parts.push(`소멸기한: ${cell.makeupDeadline}`)
  // 사용자 요청 — 같은 날 정규수업 + 보강이 함께 진행된 경우 "+보강" 배지의 상세 설명.
  if (sameDayMakeup !== undefined) {
    parts.push(`보강 진행 (${minutesToHoursText(sameDayMakeup.classMinutes)}시간)`)
    if (sameDayMakeupAbsenceHintDates !== undefined && sameDayMakeupAbsenceHintDates.length > 0) {
      if (sameDayMakeupAbsenceHintDates.length === 1) {
        parts.push(`충당 결석: ${sameDayMakeupAbsenceHintDates[0]}`)
      } else {
        parts.push('충당 결석:')
        for (const d of sameDayMakeupAbsenceHintDates) parts.push(`  ${d}`)
      }
    }
  }
  return parts.join('\n')
}
