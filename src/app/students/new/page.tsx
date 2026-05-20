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
      <div className="mx-auto max-w-3xl">
        <h1 className="mb-6 text-2xl font-bold">신규 원생 등록</h1>
        <StudentForm
          draftKey="new"
          submitLabel="등록"
          onCancel={() => router.push('/students')}
          onSubmit={async (payload) => {
            const created = await createStudent(payload)
            qc.invalidateQueries({ queryKey: ['students'] })
            router.push(`/students/edit?id=${created.id}`)
          }}
        />
      </div>
    </AppShell>
  )
}
