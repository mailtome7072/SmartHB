'use client'

/**
 * 원생 상세 팝업 — Sprint 10 T11 (PRD §4.6.2).
 *
 * 캘린더 이벤트(정규/보강 수업) 클릭 시 표시.
 * - 표시: 이름, 정규/보강 구분, 수업 시간, 해당 월 출결 요약
 * - "출결/보강관리 이동" 버튼 → `/attendance` 라우팅
 */

import { useRouter } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import { getAttendanceSummary } from '@/lib/tauri'
import { minutesToHoursText } from '@/lib/time'

export interface StudentDetailTarget {
  studentId: number
  studentName: string
  sessionType: 'regular' | 'makeup'
  startTime: string | null
  classMinutes: number
  eventDate: string
}

interface Props {
  target: StudentDetailTarget
  yearMonth: string
  onClose: () => void
}

export function StudentDetailPopup({ target, yearMonth, onClose }: Props) {
  const router = useRouter()

  const summaryQuery = useQuery({
    queryKey: ['attendance-summary', target.studentId, yearMonth],
    queryFn: () => getAttendanceSummary(target.studentId, yearMonth),
  })

  const summary = summaryQuery.data

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="원생 수업 상세"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-start justify-between">
          <div>
            <h2 className="text-xl font-bold">{target.studentName}</h2>
            <p className="mt-1 text-base text-gray-600">{target.eventDate}</p>
          </div>
          <span
            className={
              target.sessionType === 'makeup'
                ? 'rounded-full bg-emerald-100 px-3 py-1 text-sm font-semibold text-emerald-800'
                : 'rounded-full bg-blue-100 px-3 py-1 text-sm font-semibold text-blue-800'
            }
          >
            {target.sessionType === 'makeup' ? '보강 수업' : '정규 수업'}
          </span>
        </div>

        <dl className="mb-4 space-y-2 rounded-md border border-[var(--border)] bg-gray-50 p-3 text-base">
          <div className="flex justify-between">
            <dt className="text-gray-600">수업 시간</dt>
            <dd className="font-medium">
              {target.startTime !== null ? `${target.startTime} · ` : ''}
              {minutesToHoursText(target.classMinutes)}시간
            </dd>
          </div>
        </dl>

        <h3 className="mb-2 text-base font-semibold">
          {yearMonth.replace('-', '년 ') + '월'} 출결 요약
        </h3>
        {summaryQuery.isLoading && (
          <p className="text-base text-gray-500">불러오는 중...</p>
        )}
        {summary !== undefined && (
          <dl className="mb-5 grid grid-cols-2 gap-2 text-base">
            <div className="flex justify-between rounded-md bg-gray-50 px-3 py-2">
              <dt className="text-gray-600">출석</dt>
              <dd className="font-medium">{summary.presentCount}회</dd>
            </div>
            <div className="flex justify-between rounded-md bg-gray-50 px-3 py-2">
              <dt className="text-gray-600">결석</dt>
              <dd className="font-medium">{summary.absentCount}회</dd>
            </div>
            <div className="flex justify-between rounded-md bg-gray-50 px-3 py-2">
              <dt className="text-gray-600">보강 필요</dt>
              <dd className="font-medium">
                {minutesToHoursText(summary.makeupNeededMinutes)}시간
              </dd>
            </div>
            <div className="flex justify-between rounded-md bg-gray-50 px-3 py-2">
              <dt className="text-gray-600">보강 완료</dt>
              <dd className="font-medium">
                {minutesToHoursText(summary.makeupCompletedMinutes)}시간
              </dd>
            </div>
          </dl>
        )}

        <div className="flex gap-2">
          <button
            type="button"
            onClick={onClose}
            className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50"
          >
            닫기
          </button>
          <button
            type="button"
            onClick={() => router.push('/attendance')}
            className="min-h-[44px] flex-1 rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:bg-[var(--accent-hover)]"
          >
            출결/보강관리 이동
          </button>
        </div>
      </div>
    </div>
  )
}
