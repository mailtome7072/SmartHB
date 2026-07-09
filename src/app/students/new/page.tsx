'use client'

/**
 * 신규 원생 등록 화면 (Sprint 3 T11).
 */

import { useRouter } from 'next/navigation'
import { useQueryClient } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { StudentForm } from '@/components/students/student-form'
import { createStudent } from '@/lib/tauri'

export default function NewStudentPage() {
  const router = useRouter()
  const qc = useQueryClient()

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      {/* 사용자 요청 — 원생관리 전체 행간 1.25(leading-tight)로 통일. */}
      <div className="mx-auto max-w-3xl leading-tight">
        <h1 className="mb-6 text-2xl font-bold">신규 원생 등록</h1>
        <StudentForm
          draftKey="new"
          submitLabel="등록"
          onCancel={() => router.push('/students')}
          onSubmit={async (payload) => {
            const created = await createStudent(payload)
            qc.invalidateQueries({ queryKey: ['students'] })
            // T7 (사용자 이슈 #6): 등록 직후 상세화면으로 자동 진입 + 안내 배너 트리거
            router.push(`/students/edit?id=${created.id}&just_created=1`)
          }}
        />
      </div>
    </AppShell>
  )
}
