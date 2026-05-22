'use client'

/**
 * 학사 일정 코드 관리 — Sprint 7 T5 (Issue 3, sprint7.md §T5).
 *
 * Sprint 6 까지는 `/academic` 페이지 우측에 [[ScheduleCodePanel]] 이 마운트되어 있었다.
 * 설정성 작업(코드 CRUD)과 운영성 작업(일정 배치)의 동선 분리를 위해 본 페이지로 이전.
 *
 * `/academic` 페이지는 [[ScheduleCodeSelector]] 로 활성 사용자 코드 선택만 담당하며,
 * 코드 추가/수정/토글이 필요하면 본 페이지로 이동한다.
 */

import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { ScheduleCodePanel } from '@/components/academic/ScheduleCodePanel'

export default function ScheduleCodesSettingsPage() {
  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-4xl">
        <h1 className="mb-2 text-2xl font-bold">학사 일정 코드 관리</h1>
        <p className="mb-6 text-base text-gray-600">
          공휴일·보강데이 등 시스템 예약 코드(🔒)는 활성/비활성 토글만 가능합니다.
          사용자 추가 코드는 자유롭게 추가·수정할 수 있으며, 활성 상태인 코드만
          학사 캘린더에서 배치할 수 있습니다.
        </p>
        <ScheduleCodePanel />
      </div>
    </AppShell>
  )
}
