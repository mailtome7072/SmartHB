'use client'

/**
 * 데이터 내보내기 화면 — `/settings/data` (Sprint 14 T6, PRD §4.13.2).
 *
 * - 내보내기 대상 선택: 원생 명단 / 출결 데이터 / 청구·수납
 * - 기간 선택: 전체 / 특정 월 (원생 명단은 기간 무관 — 전체)
 * - "내보내기" → Tauri save 다이얼로그로 저장 경로 지정 → IPC 호출 → 결과 표시 (AC-4.13-3)
 *
 * 엑셀(.xlsx) 로 저장 — 금전 천단위 콤마·우측정렬, 컬럼 너비 자동, 수업시간 '시간' 통일.
 * 비밀번호 보호 옵션은 Sprint 15 로 이연.
 */

import Link from 'next/link'
import { useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { exportAttendances, exportBilling, exportStudents, showXlsxSaveDialog } from '@/lib/tauri'
import type { ExportResult, ExportTarget } from '@/types/export'

interface TargetMeta {
  value: ExportTarget
  label: string
  desc: string
  /** 기간 선택(전체/특정 월) 사용 여부. 원생 명단은 기간 무관. */
  usesPeriod: boolean
}

const TARGETS: TargetMeta[] = [
  { value: 'students', label: '원생 명단', desc: '재원·퇴원 포함 전체 원생 정보', usesPeriod: false },
  { value: 'attendances', label: '출결 데이터', desc: '정규 수업 + 보강 출결 기록', usesPeriod: true },
  { value: 'billing', label: '청구·수납', desc: '월별 청구액·할인·수납 내역', usesPeriod: true },
]

/** 대상별 기본 파일명 접두. */
const FILE_BASE: Record<ExportTarget, string> = {
  students: '원생명단',
  attendances: '출결',
  billing: '청구수납',
}

/** 현재 연월(YYYY-MM). */
function currentMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

/** 기본 파일명 — `{대상}_{기간}.xlsx` (예: 원생명단_전체.xlsx, 출결_2026-06.xlsx). */
function buildFileName(target: ExportTarget, period: string | null): string {
  const suffix = period ?? '전체'
  return `${FILE_BASE[target]}_${suffix}.xlsx`
}

export default function DataExportPage() {
  const [target, setTarget] = useState<ExportTarget>('students')
  const [periodMode, setPeriodMode] = useState<'all' | 'month'>('all')
  const [month, setMonth] = useState<string>(currentMonth())
  const [running, setRunning] = useState(false)
  const [result, setResult] = useState<ExportResult | null>(null)
  const [error, setError] = useState<string | null>(null)

  const meta = TARGETS.find((t) => t.value === target) ?? TARGETS[0]

  const handleExport = async () => {
    if (running) return
    // 출결/청구만 기간 적용 — 원생 명단은 항상 전체.
    const period = meta.usesPeriod && periodMode === 'month' ? month : null
    setError(null)
    setResult(null)

    const path = await showXlsxSaveDialog(buildFileName(target, period))
    if (path === null) return // 사용자가 취소

    setRunning(true)
    try {
      let res: ExportResult
      if (target === 'students') {
        res = await exportStudents(path)
      } else if (target === 'attendances') {
        res = await exportAttendances(period, path)
      } else {
        res = await exportBilling(period, path)
      }
      setResult(res)
    } catch (e) {
      setError(typeof e === 'string' ? e : '내보내기에 실패했습니다. 폴더 접근 권한을 확인해주세요.')
    } finally {
      setRunning(false)
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-2xl">
        <div className="mb-4">
          <Link href="/settings" className="text-sm text-muted-foreground hover:text-[var(--accent)]">
            ← 설정
          </Link>
        </div>

        <h1 className="text-2xl font-bold">데이터 내보내기</h1>
        <p className="mt-1 mb-6 text-base text-gray-600">
          원생·출결·청구 데이터를 엑셀(.xlsx) 파일로 저장합니다. 금액은 천단위로 표시되고 열 너비가 자동 맞춤됩니다.
        </p>

        {/* 대상 선택 */}
        <fieldset className="mb-6">
          <legend className="mb-2 text-lg font-bold">무엇을 내보낼까요?</legend>
          <div className="grid gap-3 sm:grid-cols-3">
            {TARGETS.map((t) => {
              const active = t.value === target
              return (
                <button
                  key={t.value}
                  type="button"
                  aria-pressed={active}
                  onClick={() => setTarget(t.value)}
                  className={`min-h-[44px] rounded-lg border p-4 text-left transition-colors ${
                    active
                      ? 'border-[var(--accent)] bg-[var(--background)] ring-1 ring-[var(--accent)]'
                      : 'border-[var(--border)] bg-white hover:border-[var(--accent)]'
                  }`}
                >
                  <span className="block text-base font-bold text-[var(--foreground)]">{t.label}</span>
                  <span className="mt-1 block text-sm text-gray-600">{t.desc}</span>
                </button>
              )
            })}
          </div>
        </fieldset>

        {/* 기간 선택 — 출결/청구만 */}
        {meta.usesPeriod && (
          <fieldset className="mb-6 rounded-lg border border-[var(--border)] bg-white p-5">
            <legend className="px-1 text-base font-bold">기간</legend>
            <div className="flex flex-wrap items-center gap-4">
              <label className="flex min-h-[44px] items-center gap-2">
                <input
                  type="radio"
                  name="periodMode"
                  checked={periodMode === 'all'}
                  onChange={() => setPeriodMode('all')}
                  className="h-5 w-5"
                />
                <span className="text-base">전체 기간</span>
              </label>
              <label className="flex min-h-[44px] items-center gap-2">
                <input
                  type="radio"
                  name="periodMode"
                  checked={periodMode === 'month'}
                  onChange={() => setPeriodMode('month')}
                  className="h-5 w-5"
                />
                <span className="text-base">특정 월</span>
              </label>
              <input
                type="month"
                value={month}
                onChange={(e) => setMonth(e.target.value)}
                disabled={periodMode !== 'month'}
                aria-label="내보낼 월"
                className="h-11 rounded-md border border-[var(--border)] px-3 text-base disabled:bg-gray-100 disabled:opacity-60"
              />
            </div>
          </fieldset>
        )}

        <button
          type="button"
          onClick={handleExport}
          disabled={running}
          className="h-12 rounded-md bg-[var(--accent)] px-6 text-base font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          {running ? '내보내는 중...' : '엑셀로 내보내기'}
        </button>

        {error !== null && (
          <p
            role="alert"
            className="mt-4 rounded-md border border-[var(--danger)] bg-red-50 p-3 text-sm text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        {result !== null && (
          <p className="mt-4 rounded-md border border-green-300 bg-green-50 p-4 text-base text-green-700">
            ✅ {result.row_count.toLocaleString()}건을 저장했습니다.
            <span className="mt-1 block break-all text-sm text-green-600">{result.file_path}</span>
          </p>
        )}
      </div>
    </AppShell>
  )
}
