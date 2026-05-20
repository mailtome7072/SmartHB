'use client'

/**
 * 수업 스케줄 편집 UI (Sprint 3 T13, PRD §4.2).
 *
 * 원생 상세 화면 안에서 사용. 요일별로 시작 시간 + 1회 수업 시간을 입력하면 setSchedule
 * IPC 를 호출하여 신규 또는 갱신(effective_to NULL 행 close + 새 행 INSERT) 수행.
 *
 * - 주 총 수업시간 실시간 계산 + matchFeeByHours 로 매칭 교습비 표시
 * - 운영시간 가드(AC-4.1.1-2, AC-4.1.1-5)는 Phase 2 운영시간 도입 시점에 추가
 *   — 현 단계는 09:00 ~ 22:00 하드코딩 가드만 적용
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  getSchedules,
  getWeeklyHours,
  matchFeeByHours,
  setSchedule,
} from '@/lib/tauri'
import type { StudentSchedule } from '@/types/schedule'

const DAY_LABELS = ['', '월', '화', '수', '목', '금', '토', '일']

interface DraftRow {
  day_of_week: number
  start_time: string
  duration_hours: string
}

const EMPTY_ROW: DraftRow = { day_of_week: 1, start_time: '16:00', duration_hours: '1' }

export function ScheduleEditor({ studentId }: { studentId: number }) {
  const qc = useQueryClient()
  const today = new Date().toISOString().slice(0, 10)

  const { data: schedules = [] } = useQuery<StudentSchedule[]>({
    queryKey: ['schedules', studentId],
    queryFn: () => getSchedules(studentId),
    enabled: Number.isFinite(studentId),
  })
  const { data: weeklyHours = 0 } = useQuery<number>({
    queryKey: ['weekly-hours', studentId],
    queryFn: () => getWeeklyHours(studentId),
    enabled: Number.isFinite(studentId),
  })
  const { data: matchedFee } = useQuery({
    queryKey: ['fee-match', weeklyHours],
    queryFn: () => matchFeeByHours(weeklyHours),
    enabled: weeklyHours > 0,
  })

  const [draft, setDraft] = useState<DraftRow>(EMPTY_ROW)
  const [error, setError] = useState<string | null>(null)

  const upsert = useMutation({
    mutationFn: async (row: DraftRow) => {
      const [hh, mm] = row.start_time.split(':').map(Number)
      if (hh < 9 || hh > 22) throw new Error('운영시간 09:00 ~ 22:00 내로 입력해주세요.')
      if (Number.isNaN(mm) || mm < 0 || mm > 59) throw new Error('시작 시간이 올바르지 않습니다.')
      await setSchedule({
        student_id: studentId,
        day_of_week: row.day_of_week,
        start_time: row.start_time.length === 5 ? `${row.start_time}:00` : row.start_time,
        duration_hours: Number(row.duration_hours),
        effective_from: today,
      })
    },
    onSuccess: () => {
      setError(null)
      setDraft(EMPTY_ROW)
      qc.invalidateQueries({ queryKey: ['schedules', studentId] })
      qc.invalidateQueries({ queryKey: ['weekly-hours', studentId] })
    },
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  return (
    <section className="mt-8 border-t border-[var(--border)] pt-6">
      <header className="mb-3 flex items-baseline justify-between">
        <h2 className="text-xl font-bold">수업 스케줄</h2>
        <p className="text-sm text-gray-600">
          주 총 {weeklyHours} 시간
          {matchedFee && ` / 매칭 교습비: ${matchedFee.amount.toLocaleString()} 원`}
        </p>
      </header>

      <table className="mb-4 w-full overflow-hidden rounded-md border border-[var(--border)] bg-white">
        <thead className="bg-[var(--background)]">
          <tr className="text-left">
            <th className="px-3 py-2 text-sm font-bold">요일</th>
            <th className="px-3 py-2 text-sm font-bold">시작 시간</th>
            <th className="px-3 py-2 text-sm font-bold">수업 시간</th>
            <th className="px-3 py-2 text-sm font-bold">적용 시작</th>
          </tr>
        </thead>
        <tbody>
          {schedules.length === 0 && (
            <tr>
              <td colSpan={4} className="px-3 py-6 text-center text-sm text-gray-500">
                등록된 스케줄이 없습니다. 아래에서 추가해주세요.
              </td>
            </tr>
          )}
          {schedules.map((s) => (
            <tr key={s.id} className="border-t border-[var(--border)]">
              <td className="px-3 py-2">{DAY_LABELS[s.day_of_week]}</td>
              <td className="px-3 py-2">{s.start_time.slice(0, 5)}</td>
              <td className="px-3 py-2">{s.duration_hours} 시간</td>
              <td className="px-3 py-2 text-sm text-gray-600">{s.effective_from}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <form
        onSubmit={(e) => {
          e.preventDefault()
          upsert.mutate(draft)
        }}
        className="flex flex-wrap items-end gap-2"
        aria-label="스케줄 추가/변경"
      >
        <label className="flex flex-col gap-1 text-sm">
          요일
          <select
            value={draft.day_of_week}
            onChange={(e) => setDraft({ ...draft, day_of_week: Number(e.target.value) })}
            className="h-11 rounded-md border border-[var(--border)] px-3"
          >
            {[1, 2, 3, 4, 5, 6, 7].map((d) => (
              <option key={d} value={d}>
                {DAY_LABELS[d]}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-sm">
          시작
          <input
            type="time"
            value={draft.start_time}
            onChange={(e) => setDraft({ ...draft, start_time: e.target.value })}
            className="h-11 rounded-md border border-[var(--border)] px-3"
          />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          시간(시)
          <input
            type="number"
            value={draft.duration_hours}
            onChange={(e) => setDraft({ ...draft, duration_hours: e.target.value })}
            step="0.5"
            min="0.5"
            max="4"
            className="h-11 w-24 rounded-md border border-[var(--border)] px-3"
          />
        </label>
        <button
          type="submit"
          disabled={upsert.isPending}
          className="h-11 rounded-md bg-[var(--accent)] px-4 font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          {upsert.isPending ? '저장 중...' : '추가/변경'}
        </button>
      </form>

      {error !== null && (
        <p role="alert" className="mt-2 text-sm text-[var(--danger)]">
          {error}
        </p>
      )}
    </section>
  )
}
