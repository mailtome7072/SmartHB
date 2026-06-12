'use client'

/**
 * 보강 관리 뷰 — Sprint 10 T11 (PRD §4.6.3) + 1차 시각 검증 반영.
 *
 * - 보강 필요 원생 목록 (백엔드에서 소멸기한 임박 순 정렬)
 * - 소멸 임박(isImminent) 행 강조 (amber 배경 + 경고 아이콘) — AC-4.6-2
 * - 상단 원생 검색 + 재원중 체크 필터 (출결관리와 동일 UX) — 부모(page)에서 값 주입
 * - "출결관리 이동" 버튼 → 해당 원생 이름으로 출결관리 필터 이동 (PI-04: 일괄 등록 없음)
 */

import { useMemo } from 'react'
import { useRouter } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import { getMakeupManagementData } from '@/lib/tauri'
import { minutesToHoursText } from '@/lib/time'
import { useAppStore } from '@/stores/app-store'

interface Props {
  yearMonth: string
  /** 원생 이름 부분 일치 필터 (소문자). */
  search: string
  /** 재원중(withdrawDate === null)만 표시. */
  enrolledOnly: boolean
}

export function MakeupManagementView({ yearMonth, search, enrolledOnly }: Props) {
  const router = useRouter()
  const setAttendanceSearchPreset = useAppStore((s) => s.setAttendanceSearchPreset)

  const query = useQuery({
    queryKey: ['makeup-management', yearMonth],
    queryFn: () => getMakeupManagementData(yearMonth),
  })

  const students = useMemo(() => {
    const list = query.data ?? []
    const q = search.trim().toLowerCase()
    return list.filter((s) => {
      if (enrolledOnly && s.withdrawDate !== null) return false
      if (q !== '' && !s.studentName.toLowerCase().includes(q)) return false
      return true
    })
  }, [query.data, search, enrolledOnly])

  function goToAttendance(studentName: string) {
    setAttendanceSearchPreset(studentName)
    router.push('/attendance')
  }

  return (
    <section className="py-4">
      <p className="mb-3 text-base text-gray-700">
        보강이 필요한 원생을 소멸기한이 임박한 순으로 표시합니다. 실제 보강 등록은
        &ldquo;출결관리 이동&rdquo; 버튼으로 이동해 진행하세요.
      </p>

      {query.isLoading && <p className="text-base text-muted-foreground">불러오는 중...</p>}

      {!query.isLoading && students.length === 0 && (
        <div className="rounded-lg border border-[var(--border)] bg-white p-8 text-center">
          <p className="text-lg text-gray-700">보강이 필요한 원생이 없습니다.</p>
        </div>
      )}

      {students.length > 0 && (
        <div className="overflow-x-auto rounded-lg border border-[var(--border)]">
          <table className="w-full text-left text-base">
            <thead className="bg-gray-50 text-sm text-gray-600">
              <tr>
                <th className="px-4 py-3">원생</th>
                <th className="px-4 py-3">일련번호</th>
                <th className="px-4 py-3">잔여 보강필요시간</th>
                <th className="px-4 py-3">소멸기한</th>
                <th className="px-4 py-3">상태</th>
                <th className="px-4 py-3">관리</th>
              </tr>
            </thead>
            <tbody>
              {students.map((s) => (
                <tr
                  key={s.studentId}
                  className={
                    s.isImminent
                      ? 'border-t border-amber-200 bg-amber-50'
                      : 'border-t border-gray-100'
                  }
                >
                  <td className="px-4 py-3 font-medium">
                    {s.studentName}
                    {s.withdrawDate !== null && (
                      <span className="ml-1 text-xs text-muted-foreground">(퇴교)</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-gray-600">{s.serialNo}</td>
                  <td className="px-4 py-3">{minutesToHoursText(s.remainingMinutes)}시간</td>
                  <td className="px-4 py-3 text-gray-700">{s.earliestDeadline ?? '미확정'}</td>
                  <td className="px-4 py-3">
                    {s.isImminent ? (
                      <span className="inline-flex items-center gap-1 rounded-full bg-amber-200 px-2 py-1 text-sm font-semibold text-amber-900">
                        ⚠ 소멸 임박
                      </span>
                    ) : (
                      <span className="text-sm text-muted-foreground">—</span>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <button
                      type="button"
                      onClick={() => goToAttendance(s.studentName)}
                      className="min-h-[44px] rounded-md border-2 border-[var(--accent)] px-3 text-base font-semibold text-[var(--accent)] hover:bg-blue-50"
                    >
                      출결관리 이동
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
