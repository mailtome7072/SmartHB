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

import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  applyScheduleChange,
  changeScheduleDay,
  deleteSchedule,
  getOperatingHours,
  getSchedules,
  getWeeklyHours,
  matchFeeByHours,
  setSchedule,
  type DayHours,
} from '@/lib/tauri'
import { todayLocalISO } from '@/lib/format'
import type { StudentSchedule } from '@/types/schedule'

const DAY_LABELS = ['', '월', '화', '수', '목', '금', '토', '일']

interface DraftRow {
  day_of_week: number
  start_time: string
  duration_hours: string
  /** Sprint 16 T0 케이스2 — 변경 적용 시작일(effective_from). 과거(사후)·미래(사전) 모두 허용. */
  effective_from: string
}

const EMPTY_ROW: DraftRow = {
  day_of_week: 1,
  start_time: '16:00',
  duration_hours: '1',
  effective_from: '',
}
/** 1회 수업 시간 — 1시간 단위만 지원 (T11 사용자 요청). */
const DURATION_OPTIONS = ['1', '2', '3', '4']

export function ScheduleEditor({ studentId }: { studentId: number }) {
  const qc = useQueryClient()
  // P0-3: 로컬 기준 오늘 — UTC 기준이면 KST 오전 9시 전 "어제"가 스케줄 삭제 기준일이 됨
  const today = todayLocalISO()

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
  // T9 (이슈 #9): 운영시간 fetch — 요일별 open/close 기반으로 시작시간 콤보 옵션 생성
  const { data: operatingHours = [] } = useQuery<DayHours[]>({
    queryKey: ['operating-hours'],
    queryFn: () => getOperatingHours(),
  })

  const [draft, setDraft] = useState<DraftRow>({ ...EMPTY_ROW, effective_from: today })
  const [error, setError] = useState<string | null>(null)
  // Sprint 16 T0 케이스2 — 변경 적용 결과 안내 (재생성/보존/청구 변동).
  const [notice, setNotice] = useState<string | null>(null)
  // 수정 중인 원래 요일 (null = 추가 모드). 요일 변경 시 원래 요일 종료에 사용.
  const [editingDay, setEditingDay] = useState<number | null>(null)
  // 추가/변경/삭제 확인 다이얼로그 — 변경 내용 요약 후 사용자 확인 시 반영.
  const [confirm, setConfirm] = useState<
    | { kind: 'upsert'; row: DraftRow; summary: string }
    | { kind: 'remove'; dayOfWeek: number; summary: string }
    | null
  >(null)

  // 선택 가능 요일: 평일(월~금) 중 아직 등록되지 않은 요일 + (수정 중이면 자기 요일).
  // 정규수업은 평일만 — 토(6)/일(7) 제외. 단 기존 토/일 데이터를 수정 중이면 그 요일은 노출.
  const availableDays = useMemo(() => {
    const used = new Set(schedules.map((s) => s.day_of_week))
    return [1, 2, 3, 4, 5, 6, 7].filter((d) => {
      const weekdayOk = d <= 5 || d === editingDay
      const notUsed = !used.has(d) || d === editingDay
      return weekdayOk && notUsed
    })
  }, [schedules, editingDay])

  // 추가 모드에서 현재 draft 요일이 더 이상 선택 불가하면 첫 가용 요일로 보정.
  useEffect(() => {
    if (
      editingDay === null &&
      availableDays.length > 0 &&
      !availableDays.includes(draft.day_of_week)
    ) {
      setDraft((d) => ({ ...d, day_of_week: availableDays[0] }))
    }
  }, [availableDays, editingDay, draft.day_of_week])

  // 선택된 요일의 운영 시간 → 1시간 단위 시작 옵션 (close 1시간 전까지)
  const startTimeOptions = useMemo(() => {
    const day = operatingHours.find((h) => h.day_of_week === draft.day_of_week)
    if (!day || day.open_time === null || day.close_time === null) {
      // 미운영 요일은 옵션 없음 — UI 가 안내
      return [] as string[]
    }
    const openH = Number(day.open_time.slice(0, 2))
    const closeH = Number(day.close_time.slice(0, 2))
    const opts: string[] = []
    for (let h = openH; h < closeH; h += 1) {
      opts.push(`${h.toString().padStart(2, '0')}:00`)
    }
    return opts
  }, [operatingHours, draft.day_of_week])

  const upsert = useMutation({
    mutationFn: async (row: DraftRow) => {
      const [hh, mm] = row.start_time.split(':').map(Number)
      if (Number.isNaN(hh) || hh < 0 || hh > 23)
        throw new Error('시작 시간이 올바르지 않습니다.')
      if (Number.isNaN(mm) || mm < 0 || mm > 59)
        throw new Error('시작 시간이 올바르지 않습니다.')
      if (row.effective_from === '')
        throw new Error('적용 시작일을 선택해주세요.')
      const payload = {
        student_id: studentId,
        day_of_week: row.day_of_week,
        start_time: row.start_time.length === 5 ? `${row.start_time}:00` : row.start_time,
        duration_hours: Number(row.duration_hours),
        effective_from: row.effective_from,
      }
      // P0-7: 요일 변경은 단일 트랜잭션 커맨드 — 기존 delete→set 순차 호출은 중간 실패 시
      // "원래 요일만 종료된 반쪽 상태"가 남았음. 실패 시 전체 롤백으로 기존 스케줄 유지.
      if (editingDay !== null && editingDay !== row.day_of_week) {
        await changeScheduleDay(payload, editingDay)
      } else {
        await setSchedule(payload)
      }
      // Sprint 16 T0 케이스2: 변경일 이후 출결을 신 스케줄로 재생성 (미처리만, 처리행 보존).
      return applyScheduleChange(studentId, row.effective_from)
    },
    onSuccess: (result) => {
      setError(null)
      setEditingDay(null)
      setDraft({ ...EMPTY_ROW, effective_from: today })
      // 변경일 이후 출결 반영 결과 안내.
      const parts = [`변경일 이후 출결 ${result.regeneratedCount}건 재생성`]
      if (result.preservedCount > 0) {
        parts.push(`결석·보강 ${result.preservedCount}건 보존`)
      }
      if (result.weeklyMinutesBefore !== result.weeklyMinutesAfter) {
        parts.push('주당 수업시간이 바뀌어 이번 달 청구액 재확인이 필요합니다')
      }
      setNotice(parts.join(' · '))
      qc.invalidateQueries({ queryKey: ['schedules', studentId] })
      qc.invalidateQueries({ queryKey: ['weekly-hours', studentId] })
      qc.invalidateQueries({ queryKey: ['attendance-grid'] })
    },
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  const remove = useMutation({
    mutationFn: async (dayOfWeek: number) => {
      // 오늘부터 해당 요일 스케줄 종료 + 오늘 이후 출결 정리(미처리만 제거, 처리행 보존).
      await deleteSchedule(studentId, dayOfWeek, today)
      return applyScheduleChange(studentId, today)
    },
    onSuccess: (result) => {
      setError(null)
      const parts = ['스케줄 삭제 — 오늘 이후 해당 요일 출결이 정리되었습니다']
      if (result.preservedCount > 0) {
        parts.push(`결석·보강 ${result.preservedCount}건은 보존`)
      }
      setNotice(parts.join(' · '))
      qc.invalidateQueries({ queryKey: ['schedules', studentId] })
      qc.invalidateQueries({ queryKey: ['weekly-hours', studentId] })
      qc.invalidateQueries({ queryKey: ['attendance-grid'] })
    },
    onError: (e) => setError(e instanceof Error ? e.message : String(e)),
  })

  // T9 (이슈 #10): 기존 행 "수정" — 폼에 값 prefill, 사용자가 변경 후 "추가/변경" 클릭하면
  // set_schedule 의 upsert 패턴(같은 요일 자동 close+insert)으로 처리.
  const handleEdit = (s: StudentSchedule) => {
    setEditingDay(s.day_of_week)
    setDraft({
      day_of_week: s.day_of_week,
      start_time: s.start_time.slice(0, 5),
      duration_hours: String(s.duration_hours),
      // 변경 적용일은 사용자가 새로 정한다 (기존 effective_from 이 아님) — 기본 오늘.
      effective_from: today,
    })
  }

  const cancelEdit = () => {
    setEditingDay(null)
    setDraft({ ...EMPTY_ROW, day_of_week: availableDays[0] ?? 1, effective_from: today })
  }

  // 추가/변경 확인 다이얼로그용 요약 문구.
  const buildUpsertSummary = (row: DraftRow): string => {
    const time = `${row.start_time} ${row.duration_hours}시간`
    if (editingDay !== null && editingDay !== row.day_of_week) {
      return `${DAY_LABELS[editingDay]}요일 수업을 ${DAY_LABELS[row.day_of_week]}요일 ${time}(으)로 변경합니다.\n적용 시작일: ${row.effective_from}\n→ ${DAY_LABELS[editingDay]}요일은 종료되고, 변경일 이후 출결이 새 스케줄로 재생성됩니다 (결석·보강 보존).`
    }
    if (editingDay !== null) {
      return `${DAY_LABELS[row.day_of_week]}요일 수업을 ${time}(으)로 변경합니다.\n적용 시작일: ${row.effective_from}\n→ 변경일 이후 출결이 재생성됩니다 (결석·보강 보존).`
    }
    return `${DAY_LABELS[row.day_of_week]}요일 ${time} 수업을 추가합니다.\n적용 시작일: ${row.effective_from}`
  }

  const buildRemoveSummary = (s: StudentSchedule): string =>
    `${DAY_LABELS[s.day_of_week]}요일 ${s.start_time.slice(0, 5)} ${s.duration_hours}시간 수업을 삭제합니다.\n오늘(${today})부터 종료되며 이후 출결이 정리됩니다 (결석·보강 보존).`

  return (
    <section className="mt-8 border-t border-[var(--border)] pt-6">
      <header className="mb-3 flex items-baseline justify-between">
        <h2 className="text-xl font-bold">수업 스케줄</h2>
        <p className="text-sm text-gray-600">
          주 총 {weeklyHours} 시간
          {matchedFee && ` / 매칭 교습비: ${matchedFee.amount.toLocaleString()} 원`}
        </p>
      </header>

      {editingDay !== null && (
        <p className="mb-1 text-sm font-semibold text-amber-700">
          {DAY_LABELS[editingDay]}요일 수업 수정 중 — 요일/시작시간/시간/적용 시작일을 바꾼 뒤 &lsquo;변경&rsquo;을 누르세요. (요일을 바꾸면 원래 요일은 종료됩니다)
        </p>
      )}

      {/* T11: 폼이 그리드 위 — 사용자 시선 흐름 자연스럽게. */}
      <form
        onSubmit={(e) => {
          e.preventDefault()
          if (draft.effective_from === '') {
            setError('적용 시작일을 선택해주세요.')
            return
          }
          setConfirm({ kind: 'upsert', row: draft, summary: buildUpsertSummary(draft) })
        }}
        className="mb-4 flex flex-wrap items-end gap-2"
        aria-label="스케줄 추가/변경"
      >
        <label className="flex flex-col gap-1 text-sm">
          요일
          <select
            value={draft.day_of_week}
            onChange={(e) => setDraft({ ...draft, day_of_week: Number(e.target.value) })}
            className="h-11 rounded-md border border-[var(--border)] px-3"
          >
            {/* 이미 등록된 요일은 선택 불가 — 미등록 요일 + (수정 중) 자기 요일만 노출.
                수정 중 다른 요일 선택 시 upsert 가 원래 요일을 종료하고 새 요일로 변경한다. */}
            {availableDays.map((d) => (
              <option key={d} value={d}>
                {DAY_LABELS[d]}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-sm">
          시작
          {startTimeOptions.length === 0 ? (
            <span className="flex h-11 items-center rounded-md border border-[var(--border)] bg-gray-100 px-3 text-muted-foreground">
              미운영 요일
            </span>
          ) : (
            <select
              value={draft.start_time}
              onChange={(e) => setDraft({ ...draft, start_time: e.target.value })}
              className="h-11 rounded-md border border-[var(--border)] px-3"
            >
              {!startTimeOptions.includes(draft.start_time) && (
                <option value={draft.start_time}>{draft.start_time} (운영시간 외)</option>
              )}
              {startTimeOptions.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          )}
        </label>
        <label className="flex flex-col gap-1 text-sm">
          시간(시)
          <select
            value={draft.duration_hours}
            onChange={(e) => setDraft({ ...draft, duration_hours: e.target.value })}
            className="h-11 w-24 rounded-md border border-[var(--border)] px-3"
          >
            {DURATION_OPTIONS.map((d) => (
              <option key={d} value={d}>
                {d}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-sm">
          적용 시작일
          <input
            type="date"
            value={draft.effective_from}
            onChange={(e) => setDraft({ ...draft, effective_from: e.target.value })}
            className="h-11 rounded-md border border-[var(--border)] px-3"
            aria-label="변경 적용 시작일 (이 날짜부터 신 스케줄 반영)"
          />
        </label>
        <button
          type="submit"
          disabled={upsert.isPending || startTimeOptions.length === 0 || availableDays.length === 0}
          title={startTimeOptions.length === 0 ? '미운영 요일 — 운영시간 설정에서 활성화 후 추가 가능' : undefined}
          className="h-11 rounded-md bg-[var(--accent)] px-4 font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          {upsert.isPending ? '저장 중...' : editingDay !== null ? '변경' : '추가'}
        </button>
        {editingDay !== null && (
          <button
            type="button"
            onClick={cancelEdit}
            disabled={upsert.isPending}
            className="h-11 rounded-md border border-[var(--border)] px-4 font-semibold text-gray-700 hover:bg-gray-50 disabled:opacity-50"
          >
            취소
          </button>
        )}
      </form>

      {error !== null && (
        <p role="alert" className="mb-2 text-sm text-[var(--danger)]">
          {error}
        </p>
      )}

      {notice !== null && (
        <div
          role="status"
          className="mb-2 flex items-start justify-between gap-3 rounded-md border-2 border-amber-400 bg-amber-50 p-2 text-sm text-amber-900"
        >
          <span>{notice}</span>
          <button
            type="button"
            onClick={() => setNotice(null)}
            aria-label="안내 닫기"
            className="shrink-0 rounded px-1 text-amber-700 hover:bg-amber-100"
          >
            ×
          </button>
        </div>
      )}

      <table className="w-full overflow-hidden rounded-md border border-[var(--border)] bg-white">
        <thead className="bg-[var(--background)]">
          <tr className="text-left">
            <th className="px-3 py-2 text-sm font-bold">요일</th>
            <th className="px-3 py-2 text-sm font-bold">시작 시간</th>
            <th className="px-3 py-2 text-sm font-bold">수업 시간</th>
            <th className="px-3 py-2 text-sm font-bold">적용 시작</th>
            <th className="px-3 py-2 text-sm font-bold">동작</th>
          </tr>
        </thead>
        <tbody>
          {schedules.length === 0 && (
            <tr>
              <td colSpan={5} className="px-3 py-6 text-center text-sm text-muted-foreground">
                등록된 스케줄이 없습니다. 위 폼에서 추가해주세요.
              </td>
            </tr>
          )}
          {schedules.map((s) => (
            <tr key={s.id} className="border-t border-[var(--border)]">
              <td className="px-3 py-2">{DAY_LABELS[s.day_of_week]}</td>
              <td className="px-3 py-2">{s.start_time.slice(0, 5)}</td>
              <td className="px-3 py-2">{s.duration_hours} 시간</td>
              <td className="px-3 py-2 text-sm text-gray-600">{s.effective_from}</td>
              <td className="px-3 py-2">
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={() => handleEdit(s)}
                    className="h-9 rounded-md border border-[var(--border)] px-3 text-sm hover:bg-gray-50"
                    aria-label={`${DAY_LABELS[s.day_of_week]}요일 스케줄 수정`}
                  >
                    수정
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      if (remove.isPending) return
                      setConfirm({
                        kind: 'remove',
                        dayOfWeek: s.day_of_week,
                        summary: buildRemoveSummary(s),
                      })
                    }}
                    disabled={remove.isPending}
                    className="h-9 rounded-md border border-[var(--danger)] px-3 text-sm text-[var(--danger)] hover:bg-red-50 disabled:opacity-50"
                    aria-label={`${DAY_LABELS[s.day_of_week]}요일 스케줄 삭제`}
                  >
                    삭제
                  </button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {confirm !== null && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onClick={() => setConfirm(null)}
          role="presentation"
        >
          <div
            className="w-[440px] rounded-lg bg-white p-6 shadow-xl"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label="스케줄 변경 확인"
          >
            <h2 className="text-xl font-bold">
              {confirm.kind === 'remove' ? '수업 스케줄 삭제 확인' : '수업 스케줄 변경 확인'}
            </h2>
            {/* 사용자 요청 — 원생관리 전체 행간 1.25로 통일(기존 leading-relaxed 예외 제거). */}
            <p className="mt-3 whitespace-pre-line text-base leading-tight text-gray-800">
              {confirm.summary}
            </p>
            <div className="mt-5 flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setConfirm(null)}
                className="min-h-[44px] rounded-lg border-2 border-[var(--border)] px-4 text-base hover:bg-gray-50"
              >
                취소
              </button>
              <button
                type="button"
                onClick={() => {
                  if (confirm.kind === 'upsert') upsert.mutate(confirm.row)
                  else remove.mutate(confirm.dayOfWeek)
                  setConfirm(null)
                }}
                className={`min-h-[44px] rounded-lg px-4 text-base font-semibold text-white ${
                  confirm.kind === 'remove'
                    ? 'bg-[var(--danger)] hover:opacity-90'
                    : 'bg-[var(--accent)] hover:bg-[var(--accent-hover)]'
                }`}
              >
                {confirm.kind === 'remove' ? '삭제' : '확인'}
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  )
}
