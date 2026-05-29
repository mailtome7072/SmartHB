'use client'

/**
 * 출결 관리 메인 페이지 — Sprint 8 T4 (PRD §4.5).
 *
 * 흐름:
 * 1. 월 선택 (year_month, 기본=현재 월)
 * 2. 해당 월 출결 존재 여부 조회 → 없으면 "출결 생성" 버튼, 있으면 그리드
 * 3. AttendanceGrid 에서 셀 클릭 토글 / 결석 사유 메모 다이얼로그
 *
 * TanStack Query:
 * - 'attendance-exists', ym — 출결 존재 여부 (출결 생성 버튼 활성 조건)
 * - 'attendance-grid', ym — 그리드 데이터
 * - mutation 성공 시 두 쿼리 모두 invalidate
 */

import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  checkAttendanceExists,
  generateAttendances,
  getAttendanceGrid,
  countUngeneratedAttendanceStudents,
} from '@/lib/tauri'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { useAppStore } from '@/stores/app-store'
import { AbsenceHistoryDialog } from '@/components/attendance/AbsenceHistoryDialog'
import { AttendanceGrid } from '@/components/attendance/AttendanceGrid'
import { MakeupManageDialog } from '@/components/attendance/MakeupManageDialog'
import { MakeupRegisterDialog } from '@/components/attendance/MakeupRegisterDialog'

function currentYearMonth(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
}

function previousYearMonths(count: number, from: string): string[] {
  const [y, m] = from.split('-').map(Number)
  const result: string[] = []
  for (let i = 0; i < count; i++) {
    const date = new Date(y, m - 1 - i, 1)
    result.push(
      `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`,
    )
  }
  return result
}

interface MakeupDialogTarget {
  studentId: number
  studentName: string
  studentSerialNo: string
  eventDate: string
}

interface MakeupManageTarget {
  studentId: number
  studentName: string
  studentSerialNo: string
  makeupId: number
  eventDate: string
  classMinutes: number
}

export default function AttendancePage() {
  const [yearMonth, setYearMonth] = useState(currentYearMonth)
  const [error, setError] = useState<string | null>(null)
  // Sprint 10 T4 (PI-09): 소멸 자동 전이 결과 알림. 별도 line 으로 표시.
  const [expirationNotice, setExpirationNotice] = useState<string | null>(null)
  // Sprint 8 T9 follow-up: 원생 이름 인플레이스 필터.
  // 글로벌 검색바(PRD §4.14)는 페이지 이동용 — 본 입력은 현 그리드 행 좁히기용.
  // 자모 부분 일치는 별도 라이브러리 필요로 추후 task 로 분리, 본 구현은 substring 만.
  const [searchInput, setSearchInput] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  // Sprint 9 Session #12 K2: 재원중만 필터 (withdrawDate === null). 디폴트 ON.
  const [enrolledOnly, setEnrolledOnly] = useState(true)
  // Sprint 9 Session #12 K6: 보강대상 필터 (earliestPendingAbsenceDate !== null). 디폴트 OFF.
  const [needsMakeupOnly, setNeedsMakeupOnly] = useState(false)
  // Sprint 9 T6: 비수업일 셀 클릭 → 보강 등록 다이얼로그.
  const [makeupTarget, setMakeupTarget] = useState<MakeupDialogTarget | null>(null)
  // Sprint 9 Session #10 J6: 보강일 셀 클릭 → 보강 관리(삭제) 다이얼로그.
  const [manageTarget, setManageTarget] = useState<MakeupManageTarget | null>(null)
  // Sprint 9 T8: 학생명 클릭 → 결석 이력 다이얼로그. 학생 ID + 표시용 이름/일련번호.
  const [historyTarget, setHistoryTarget] = useState<{
    studentId: number
    studentName: string
    studentSerialNo: string
  } | null>(null)
  const queryClient = useQueryClient()

  useEffect(() => {
    const handle = setTimeout(() => setDebouncedSearch(searchInput.trim().toLowerCase()), 200)
    return () => clearTimeout(handle)
  }, [searchInput])

  // Sprint 10 T11 follow-up: 수업 관리 캘린더에서 "출결관리 이동" 시 원생 이름 프리셋 소비.
  // 1회성 — 적용 후 즉시 store 를 비운다. 검색 대상이 퇴교생일 수도 있어 재원중 필터는 해제.
  const attendanceSearchPreset = useAppStore((s) => s.attendanceSearchPreset)
  const setAttendanceSearchPreset = useAppStore((s) => s.setAttendanceSearchPreset)
  useEffect(() => {
    if (attendanceSearchPreset !== null && attendanceSearchPreset !== '') {
      setSearchInput(attendanceSearchPreset)
      setDebouncedSearch(attendanceSearchPreset.trim().toLowerCase())
      setEnrolledOnly(false)
      setAttendanceSearchPreset(null)
    }
  }, [attendanceSearchPreset, setAttendanceSearchPreset])

  // 출결 존재 여부 (생성 버튼 활성 조건)
  const existsQuery = useQuery({
    queryKey: ['attendance-exists', yearMonth],
    queryFn: () => checkAttendanceExists(yearMonth),
  })

  // 그리드 데이터 — exists=true 일 때만 의미 있음
  const gridQuery = useQuery({
    queryKey: ['attendance-grid', yearMonth],
    queryFn: () => getAttendanceGrid(yearMonth),
    enabled: existsQuery.data === true,
  })

  // 미생성 원생 수 — hotfix post-Sprint 11. "추가 출결 데이터 생성" UX 트리거.
  const ungeneratedQuery = useQuery({
    queryKey: ['attendance-ungenerated', yearMonth],
    queryFn: () => countUngeneratedAttendanceStudents(yearMonth),
  })

  // 출결 일괄 생성
  const generateMutation = useMutation({
    mutationFn: () => generateAttendances(yearMonth),
    onSuccess: (result) => {
      setError(null)
      void queryClient.invalidateQueries({ queryKey: ['attendance-exists', yearMonth] })
      void queryClient.invalidateQueries({ queryKey: ['attendance-grid', yearMonth] })
      void queryClient.invalidateQueries({ queryKey: ['attendance-ungenerated', yearMonth] })
      // Sprint 10 T4 (PI-09): 출결 생성 직후 소멸 자동 전이 결과 알림.
      const count = result.expirationReport.transitionedCount
      setExpirationNotice(
        count > 0 ? `출결 생성과 함께 소멸기한 도래 결석 ${count}건이 자동 처리되었습니다.` : null,
      )
    },
    onError: (e) => {
      setError(typeof e === 'string' ? e : (e as Error).message)
    },
  })

  // 월 선택 옵션 — 현재 월 + 과거 11개월
  const monthOptions = previousYearMonths(12, currentYearMonth())

  // hotfix post-Sprint 11: 청구 패턴과 동일 — 출결 0건 → "생성", 추가 등록 원생 있으면 "추가 생성".
  const ungeneratedCount = ungeneratedQuery.data ?? 0
  const showGenerateButton =
    existsQuery.data === false || ungeneratedCount > 0
  const generateButtonLabel =
    existsQuery.data === false
      ? '출결 데이터 생성'
      : `추가 출결 데이터 생성 (${ungeneratedCount}명)`
  const showGrid = existsQuery.data === true && gridQuery.data !== undefined

  // 검색어 + 재원중 + 보강대상 필터 — 새 grid 객체를 만들어 AttendanceGrid 에 전달.
  // K2 (Session #12): enrolledOnly 체크 시 withdrawDate === null 만 통과.
  // K6 (Session #12): needsMakeupOnly 체크 시 earliestPendingAbsenceDate !== null 만 통과.
  const filteredGrid = useMemo(() => {
    if (gridQuery.data === undefined) return undefined
    if (debouncedSearch === '' && !enrolledOnly && !needsMakeupOnly) {
      return gridQuery.data
    }
    const q = debouncedSearch
    return {
      ...gridQuery.data,
      students: gridQuery.data.students.filter((s) => {
        if (enrolledOnly && s.withdrawDate !== null) return false
        if (needsMakeupOnly && s.earliestPendingAbsenceDate === null) return false
        if (q !== '' && !s.name.toLowerCase().includes(q)) return false
        return true
      }),
    }
  }, [gridQuery.data, debouncedSearch, enrolledOnly, needsMakeupOnly])

  const matchedCount = filteredGrid?.students.length ?? 0

  // K7 (Session #12): 라벨 옆 카운트 표기.
  // - 재원중(N명) = withdrawDate === null 인 원생 수 (전체 기준)
  // - 보강대상(N명) = 보강 필요 원생 수, 재원중 체크 ON 이면 재원중 한정 (연계)
  const enrolledCount = useMemo(
    () =>
      gridQuery.data?.students.filter((s) => s.withdrawDate === null).length ??
      0,
    [gridQuery.data],
  )
  const needsMakeupCount = useMemo(() => {
    if (gridQuery.data === undefined) return 0
    return gridQuery.data.students.filter((s) => {
      if (enrolledOnly && s.withdrawDate !== null) return false
      return s.earliestPendingAbsenceDate !== null
    }).length
  }, [gridQuery.data, enrolledOnly])

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <main className="flex h-full flex-col">
      <header className="flex items-center gap-4 border-b border-[var(--border)] px-0 py-4">
        <h1 className="text-2xl font-bold">출결 관리</h1>

        <div className="flex items-center gap-2">
          <label htmlFor="year-month" className="text-base text-gray-700">
            대상 월:
          </label>
          <select
            id="year-month"
            value={yearMonth}
            onChange={(e) => setYearMonth(e.target.value)}
            className="min-h-[44px] rounded-md border-2 border-[var(--border)] px-3 text-base"
          >
            {monthOptions.map((ym) => (
              <option key={ym} value={ym}>
                {ym.replace('-', '년 ') + '월'}
              </option>
            ))}
          </select>
        </div>

        {showGrid && (
          <div className="flex items-center gap-2">
            <label htmlFor="student-search" className="text-base text-gray-700">
              원생 검색:
            </label>
            <input
              id="student-search"
              type="search"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="이름 입력"
              aria-label="원생 이름 검색"
              className="min-h-[44px] w-48 rounded-md border-2 border-[var(--border)] px-3 text-base"
            />
            {showGenerateButton && (
              <button
                type="button"
                onClick={() => generateMutation.mutate()}
                disabled={generateMutation.isPending}
                className="min-h-[44px] rounded-lg bg-[var(--accent)] px-4 text-base font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
              >
                {generateMutation.isPending ? '생성 중...' : generateButtonLabel}
              </button>
            )}
            <label className="ml-2 flex min-h-[44px] cursor-pointer items-center gap-2 text-base text-gray-700">
              <input
                type="checkbox"
                checked={enrolledOnly}
                onChange={(e) => setEnrolledOnly(e.target.checked)}
                aria-label="재원중 원생만 보기"
                className="h-5 w-5 cursor-pointer accent-[var(--accent)]"
              />
              재원중({enrolledCount}명)
            </label>
            <label className="flex min-h-[44px] cursor-pointer items-center gap-2 text-base text-gray-700">
              <input
                type="checkbox"
                checked={needsMakeupOnly}
                onChange={(e) => setNeedsMakeupOnly(e.target.checked)}
                aria-label="보강대상 원생만 보기"
                className="h-5 w-5 cursor-pointer accent-[var(--accent)]"
              />
              보강대상({needsMakeupCount}명)
            </label>
          </div>
        )}

        {/* 출결 0건 — 그리드 없는 빈 상태에서는 헤더 우측에 표시 */}
        {showGenerateButton && !showGrid && (
          <button
            type="button"
            onClick={() => generateMutation.mutate()}
            disabled={generateMutation.isPending}
            className="ml-auto min-h-[44px] rounded-lg bg-[var(--accent)] px-4 text-base font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            {generateMutation.isPending ? '생성 중...' : generateButtonLabel}
          </button>
        )}
      </header>

      {error !== null && (
        <div
          role="alert"
          className="mx-6 mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
        >
          {error}
        </div>
      )}

      {expirationNotice !== null && (
        <div
          role="status"
          className="mx-6 mt-4 flex items-center justify-between rounded-md border-2 border-amber-400 bg-amber-50 p-3 text-base text-amber-900"
        >
          <span>{expirationNotice}</span>
          <button
            type="button"
            onClick={() => setExpirationNotice(null)}
            aria-label="알림 닫기"
            className="ml-3 min-h-[32px] min-w-[32px] rounded text-amber-700 hover:bg-amber-100"
          >
            ×
          </button>
        </div>
      )}

      <section className="flex-1 overflow-auto py-4">
        {existsQuery.isLoading && (
          <p className="text-gray-600">출결 상태 확인 중...</p>
        )}

        {existsQuery.data === false && (
          <div className="rounded-lg border border-[var(--border)] bg-white p-8 text-center">
            <p className="text-lg text-gray-700">
              {yearMonth.replace('-', '년 ') + '월'} 출결이 아직 생성되지 않았습니다.
            </p>
            <p className="mt-2 text-base text-gray-500">
              우측 상단의 &ldquo;출결 데이터 생성&rdquo; 버튼을 눌러 해당 월 재원 원생의 출결을 일괄 생성하세요.
            </p>
            <p className="mt-2 text-sm text-gray-500">
              ※ 교습기간이 먼저 확정되어 있어야 합니다 (학사 스케줄 메뉴).
            </p>
          </div>
        )}

        {showGrid && filteredGrid !== undefined && (
          <>
            <AttendanceGrid
              grid={filteredGrid}
              onNonClassDayClick={(studentId, eventDate) => {
                const student = filteredGrid.students.find(
                  (s) => s.studentId === studentId,
                )
                if (student === undefined) return
                setMakeupTarget({
                  studentId,
                  studentName: student.name,
                  studentSerialNo: student.serialNo,
                  eventDate,
                })
              }}
              onClassDayMakeupRegister={(studentId, eventDate) => {
                // Session #12 K3: 정규 수업 셀 우클릭 진입 — 비수업일 진입과 동일한 다이얼로그.
                const student = filteredGrid.students.find(
                  (s) => s.studentId === studentId,
                )
                if (student === undefined) return
                setMakeupTarget({
                  studentId,
                  studentName: student.name,
                  studentSerialNo: student.serialNo,
                  eventDate,
                })
              }}
              onMakeupDayCellClick={(studentId, makeup) => {
                const student = filteredGrid.students.find(
                  (s) => s.studentId === studentId,
                )
                if (student === undefined) return
                setManageTarget({
                  studentId,
                  studentName: student.name,
                  studentSerialNo: student.serialNo,
                  makeupId: makeup.id,
                  eventDate: makeup.eventDate,
                  classMinutes: makeup.classMinutes,
                })
              }}
              onStudentNameClick={(studentId) => {
                const student = filteredGrid.students.find(
                  (s) => s.studentId === studentId,
                )
                if (student === undefined) return
                setHistoryTarget({
                  studentId,
                  studentName: student.name,
                  studentSerialNo: student.serialNo,
                })
              }}
            />
            {debouncedSearch !== '' && matchedCount === 0 && (
              <p className="mt-4 text-center text-base text-gray-600">
                &ldquo;{searchInput}&rdquo; 검색 결과가 없습니다.
              </p>
            )}
          </>
        )}

        {existsQuery.data === true && gridQuery.isLoading && (
          <p className="text-gray-600">출결 데이터 불러오는 중...</p>
        )}
      </section>

      {makeupTarget !== null && (
        <MakeupRegisterDialog
          studentId={makeupTarget.studentId}
          studentName={makeupTarget.studentName}
          studentSerialNo={makeupTarget.studentSerialNo}
          eventDate={makeupTarget.eventDate}
          yearMonth={yearMonth}
          onClose={() => setMakeupTarget(null)}
          onSuccess={() => {
            setMakeupTarget(null)
            void queryClient.invalidateQueries({ queryKey: ['attendance-grid', yearMonth] })
            void queryClient.invalidateQueries({
              queryKey: ['pending-absences', makeupTarget.studentId],
            })
          }}
        />
      )}

      {manageTarget !== null && (
        <MakeupManageDialog
          makeupId={manageTarget.makeupId}
          studentName={manageTarget.studentName}
          studentSerialNo={manageTarget.studentSerialNo}
          eventDate={manageTarget.eventDate}
          classMinutes={manageTarget.classMinutes}
          onClose={() => setManageTarget(null)}
          onSuccess={() => {
            setManageTarget(null)
            void queryClient.invalidateQueries({ queryKey: ['attendance-grid', yearMonth] })
            void queryClient.invalidateQueries({
              queryKey: ['pending-absences', manageTarget.studentId],
            })
          }}
        />
      )}


      {historyTarget !== null && (
        <AbsenceHistoryDialog
          studentId={historyTarget.studentId}
          studentName={historyTarget.studentName}
          studentSerialNo={historyTarget.studentSerialNo}
          onClose={() => setHistoryTarget(null)}
        />
      )}
      </main>
    </AppShell>
  )
}
