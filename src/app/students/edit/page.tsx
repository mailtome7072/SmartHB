'use client'

/**
 * 원생 상세/수정 화면 (Sprint 3 T11, PRD §4.1).
 *
 * 수정 + 퇴교 처리. 퇴교 확인 다이얼로그는 shadcn/ui AlertDialog 사용 (Sprint 4 T1, A11).
 * Tauri 2.x 가 보안 정책으로 `window.confirm()` 을 차단 (`dialog.confirm not allowed.
 * Command not found`) 하므로 native 다이얼로그 대신 컴포넌트 기반 모달로 교체.
 */

import { Suspense, useState } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { StudentForm } from '@/components/students/student-form'
import { ScheduleEditor } from '@/components/students/schedule-editor'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import {
  getPendingMakeupForWithdrawal,
  getStudent,
  reinstateStudent,
  updateStudent,
  withdrawStudent,
} from '@/lib/tauri'
import type { NewStudent, Student } from '@/types/student'
import type { WithdrawalPendingMakeup } from '@/types/withdrawal'
import { WithdrawalMakeupDialog } from '@/components/students/WithdrawalMakeupDialog'

export default function StudentDetailPage() {
  return (
    <Suspense fallback={<SplashScreen message="원생 정보 페이지를 여는 중입니다..." />}>
      <StudentDetailContent />
    </Suspense>
  )
}

function StudentDetailContent() {
  const searchParams = useSearchParams()
  const idParam = searchParams.get('id')
  const studentId = idParam !== null ? Number(idParam) : NaN
  // T7: students/new 가 등록 직후 ?just_created=1 로 redirect — 안내 배너 표시 토글
  const justCreated = searchParams.get('just_created') === '1'
  const router = useRouter()
  const qc = useQueryClient()
  const [withdrawing, setWithdrawing] = useState(false)
  const [reinstating, setReinstating] = useState(false)
  // hotfix (Sprint 10 post-merge): AlertDialog 를 controlled 로 관리해야 비동기 흐름 종료 시점에
  // 명시적으로 닫고 WithdrawalMakeupDialog 를 mount 할 수 있다.
  const [withdrawAlertOpen, setWithdrawAlertOpen] = useState(false)
  // T8: 퇴교일자는 사용자가 직접 지정 (기본값 오늘)
  const [withdrawDate, setWithdrawDate] = useState(() =>
    new Date().toISOString().slice(0, 10),
  )
  // Sprint 10 T10 (PRD §4.5.9): 잔여 보강 보유 시 처리 다이얼로그.
  const [pendingWithdrawalMakeup, setPendingWithdrawalMakeup] =
    useState<WithdrawalPendingMakeup | null>(null)

  const { data: student, isLoading, error } = useQuery<Student>({
    queryKey: ['students', 'detail', studentId],
    queryFn: () => getStudent(studentId),
    enabled: Number.isFinite(studentId),
  })

  const handleUpdate = async (payload: NewStudent) => {
    if (!student) return
    // T6: serial_no 는 백엔드가 변경 거부 — payload 값은 무시되지만 명시적으로 기존 값 전달.
    await updateStudent(student.id, {
      ...payload,
      serial_no: student.serial_no,
      school_id: payload.school_id ?? null,
      phone_student: payload.phone_student ?? null,
      phone_mother: payload.phone_mother ?? null,
      phone_father: payload.phone_father ?? null,
      withdraw_date: student.withdraw_date,
    })
    qc.invalidateQueries({ queryKey: ['students'] })
    router.push('/students')
  }

  const handleWithdrawConfirmed = async () => {
    if (!student) return
    if (student.withdraw_date !== null) return
    // hotfix: AlertDialog 를 명시적으로 먼저 닫아 backdrop 잔존으로 인한
    // 후속 WithdrawalMakeupDialog 클릭 차단을 방지.
    setWithdrawAlertOpen(false)
    setWithdrawing(true)
    try {
      // Sprint 10 T10: 잔여 보강 검증 — 있으면 처리 다이얼로그, 없으면 기존 흐름.
      const pending = await getPendingMakeupForWithdrawal(student.id)
      if (pending.absences.length === 0) {
        await withdrawStudent(student.id, withdrawDate)
        qc.invalidateQueries({ queryKey: ['students'] })
        router.push('/students')
        return
      }
      // 잔여 보강 있음 — WithdrawalMakeupDialog 가 mount 되며 IPC 호출 담당.
      setPendingWithdrawalMakeup(pending)
    } finally {
      setWithdrawing(false)
    }
  }

  const handleWithdrawalMakeupCompleted = () => {
    setPendingWithdrawalMakeup(null)
    qc.invalidateQueries({ queryKey: ['students'] })
    if (student !== undefined) {
      qc.invalidateQueries({
        queryKey: ['students', 'detail', student.id],
      })
      qc.invalidateQueries({ queryKey: ['attendance-grid'] })
    }
    router.push('/students')
  }

  const handleReinstateConfirmed = async () => {
    if (!student) return
    if (student.withdraw_date === null) return
    setReinstating(true)
    try {
      await reinstateStudent(student.id)
      qc.invalidateQueries({ queryKey: ['students'] })
      qc.invalidateQueries({ queryKey: ['students', 'detail', student.id] })
    } finally {
      setReinstating(false)
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-3xl">
        <h1 className="mb-6 text-2xl font-bold">원생 상세 / 수정</h1>

        {justCreated && student !== undefined && student.withdraw_date === null && (
          <p
            role="status"
            className="mb-4 rounded-md border border-[var(--accent)] bg-blue-50 p-3 text-base text-[var(--accent)]"
          >
            ✅ 원생이 등록되었습니다. 이어서 아래 <strong>수업 스케줄</strong>을 입력하세요.
          </p>
        )}

        {isLoading && <p>불러오는 중...</p>}
        {error !== null && error !== undefined && (
          <p role="alert" className="text-[var(--danger)]">
            원생을 불러올 수 없습니다.
          </p>
        )}

        {student && (
          <>
            <StudentForm
              draftKey={String(student.id)}
              initial={student}
              submitLabel="저장"
              onCancel={() => router.push('/students')}
              onSubmit={handleUpdate}
              extraActions={
                student.withdraw_date === null ? (
                  <AlertDialog open={withdrawAlertOpen} onOpenChange={setWithdrawAlertOpen}>
                    <AlertDialogTrigger
                      type="button"
                      disabled={withdrawing}
                      className="h-11 rounded-md border border-[var(--danger)] px-4 text-[var(--danger)] hover:bg-red-50 disabled:opacity-50"
                    >
                      퇴교 처리
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>원생 퇴교 처리</AlertDialogTitle>
                        <AlertDialogDescription>
                          <strong>{student.name}</strong> 원생의 퇴교일자를 지정해주세요.
                          <br />
                          기본값은 오늘이며 과거/미래 날짜도 선택 가능합니다.
                          <br />
                          잔여 보강이 있는 원생은 다음 단계에서 처리 방식을 선택할 수 있습니다 (PRD §4.5.9).
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <div className="px-1 py-2">
                        <label className="block text-sm font-medium text-gray-700">
                          퇴교일자
                          <input
                            type="date"
                            value={withdrawDate}
                            onChange={(e) => {
                              setWithdrawDate(e.target.value)
                              // hotfix: Tauri WebView 환경에서 native date picker 가 선택 후
                              // 자동 닫히지 않는 경우가 있어 blur 로 강제 종료.
                              e.target.blur()
                            }}
                            className="mt-1 h-11 w-full rounded-md border border-[var(--border)] px-3"
                          />
                        </label>
                      </div>
                      <AlertDialogFooter>
                        <AlertDialogCancel>취소</AlertDialogCancel>
                        <AlertDialogAction
                          onClick={handleWithdrawConfirmed}
                          disabled={withdrawing}
                        >
                          퇴교 처리
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                ) : (
                  <div className="flex items-center gap-3">
                    <span className="text-sm text-gray-600">
                      퇴교일: <strong>{student.withdraw_date}</strong>
                    </span>
                    <AlertDialog>
                      <AlertDialogTrigger
                        type="button"
                        disabled={reinstating}
                        className="h-11 rounded-md border border-[var(--accent)] px-4 text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
                      >
                        퇴교 번복
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>퇴교 번복</AlertDialogTitle>
                          <AlertDialogDescription>
                            <strong>{student.name}</strong> 원생의 퇴교 처리를 번복하여
                            재원 상태로 되돌립니다.
                            <br />
                            퇴교 시 강제 소멸된 결석(자연 만기 전) 은 자동으로 결석 상태로 환원됩니다.
                            <br />
                            자연 만기로 이미 소멸된 결석은 환원 대상이 아닙니다.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>취소</AlertDialogCancel>
                          <AlertDialogAction
                            onClick={handleReinstateConfirmed}
                            disabled={reinstating}
                          >
                            퇴교 번복
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                )
              }
            />
            {/* T8 (이슈 #8-1): 퇴교 상태에서는 스케줄 추가/변경 비활성 — 안내 메시지로 대체 */}
            {student.withdraw_date === null ? (
              <ScheduleEditor studentId={student.id} />
            ) : (
              <section className="mt-6 rounded-md border border-[var(--border)] bg-gray-50 p-4">
                <h2 className="mb-2 text-lg font-bold text-gray-600">수업 스케줄</h2>
                <p className="text-sm text-gray-600">
                  퇴교 처리된 원생은 수업 스케줄을 추가/변경할 수 없습니다. 변경이 필요하면
                  먼저 <strong>퇴교 번복</strong> 후 진행하세요.
                </p>
              </section>
            )}
          </>
        )}
      </div>
      {pendingWithdrawalMakeup !== null && student !== undefined && (
        <WithdrawalMakeupDialog
          studentName={student.name}
          withdrawDate={withdrawDate}
          pending={pendingWithdrawalMakeup}
          onCompleted={handleWithdrawalMakeupCompleted}
          onCancel={() => setPendingWithdrawalMakeup(null)}
        />
      )}
    </AppShell>
  )
}
