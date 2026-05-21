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
  getStudent,
  updateStudent,
  withdrawStudent,
} from '@/lib/tauri'
import type { NewStudent, Student } from '@/types/student'

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
  const router = useRouter()
  const qc = useQueryClient()
  const [withdrawing, setWithdrawing] = useState(false)

  const { data: student, isLoading, error } = useQuery<Student>({
    queryKey: ['students', 'detail', studentId],
    queryFn: () => getStudent(studentId),
    enabled: Number.isFinite(studentId),
  })

  const handleUpdate = async (payload: NewStudent) => {
    if (!student) return
    await updateStudent(student.id, {
      ...payload,
      serial_no: payload.serial_no ?? student.serial_no,
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
    setWithdrawing(true)
    try {
      const today = new Date().toISOString().slice(0, 10)
      await withdrawStudent(student.id, today)
      qc.invalidateQueries({ queryKey: ['students'] })
      router.push('/students')
    } finally {
      setWithdrawing(false)
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-3xl">
        <h1 className="mb-6 text-2xl font-bold">원생 상세 / 수정</h1>

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
                  <AlertDialog>
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
                          <strong>{student.name}</strong> 원생을 퇴교 처리하시겠습니까?
                          <br />
                          취소 시 보강 잔여 처리는 Phase 3 에서 별도 제공됩니다.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
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
                  <span className="text-sm text-gray-600">퇴교일: {student.withdraw_date}</span>
                )
              }
            />
            <ScheduleEditor studentId={student.id} />
          </>
        )}
      </div>
    </AppShell>
  )
}
