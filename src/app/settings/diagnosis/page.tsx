'use client'

/**
 * 데이터 자가 진단 화면 — `/settings/diagnosis` (Sprint 14 T2, PRD §6.6).
 *
 * - 수동 진단 실행 버튼 (AC-6.6-2)
 * - 최근 12개월 진단 이력 목록 (날짜 / 유형 / 발견 건수)
 * - 선택한 진단 결과 상세 — 이상 항목별 설명 + 해결 가이드 + 해당 화면 이동 링크 (AC-6.6-3)
 *
 * 자동 진단(매월 1일, AC-6.6-1)은 AppShell 에서 백그라운드 트리거. 본 화면은 결과 표시/수동 실행.
 */

import Link from 'next/link'
import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { Trash2 } from 'lucide-react'
import {
  clearDiagnosisHistory,
  deleteDiagnosisHistory,
  getDiagnosisHistory,
  runDiagnosis,
} from '@/lib/tauri'
import type { DiagnosisHistoryRow, DiagnosisIssue, DiagnosisResult } from '@/types/diagnosis'

/**
 * 실행 결과를 이력 행 형태로 변환 — 완전 0건 정책상 이상이 없으면 이력이 남지 않으므로,
 * 방금 실행 결과(이상 없음)를 상세 패널에 직접 표시하기 위한 합성 행 (id=0, 목록에는 없음).
 */
function resultToRow(r: DiagnosisResult): DiagnosisHistoryRow {
  return {
    id: 0,
    run_date: r.run_date,
    run_type: r.run_type,
    total_checks: r.total_checks,
    issues_found: r.issues_found,
    issues: r.issues,
    created_at: '',
  }
}

/** 검사별 해결 가이드 (check_id 기준, AC-6.6-3). */
const GUIDE: Record<string, string> = {
  negative_makeup_minutes: '출결/보강 기록을 확인해 중복 보강이 없는지 점검하세요.',
  missing_attendance: '출결 관리에서 해당 월 출결을 생성하세요. (월초라면 정상일 수 있습니다)',
  missing_billing: '청구/수납 관리에서 해당 월 청구를 생성하세요.',
  schedule_attendance_mismatch:
    '수업 스케줄과 출결 요일을 대조해 잘못 입력된 출결을 정정하세요. (학기 중 시간표 변경 시 발생할 수 있습니다)',
  absent_without_deadline: '출결 관리에서 해당 결석의 보강 소멸기한을 설정하세요.',
  orphan_makeup: '보강 기록을 해당 결석과 연결하거나 불필요한 보강을 정리하세요.',
  payment_integrity: '청구/수납 관리에서 결제수단(카드 결제 시 카드사)을 입력하세요.',
}

/** 관련 테이블 → 이동 대상 화면. */
function targetLink(table: string | null): { href: string; label: string } | null {
  switch (table) {
    case 'students':
      return { href: '/students', label: '원생 관리로 이동' }
    case 'regular_attendances':
      return { href: '/attendance', label: '출결 관리로 이동' }
    case 'makeup_attendances':
      return { href: '/schedules', label: '수업/보강 관리로 이동' }
    case 'bills':
    case 'payments':
      return { href: '/billing', label: '청구/수납 관리로 이동' }
    default:
      return null
  }
}

function runTypeLabel(t: string): string {
  return t === 'auto' ? '자동' : '수동'
}

export default function DiagnosisPage() {
  const [history, setHistory] = useState<DiagnosisHistoryRow[]>([])
  const [selected, setSelected] = useState<DiagnosisHistoryRow | null>(null)
  const [loading, setLoading] = useState(true)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [confirm, setConfirm] = useState<{ kind: 'one'; id: number } | { kind: 'all' } | null>(null)

  const load = async () => {
    setLoading(true)
    try {
      const rows = await getDiagnosisHistory(12)
      setHistory(rows)
      setSelected((prev) => prev ?? rows[0] ?? null)
    } catch (e) {
      setError(typeof e === 'string' ? e : '진단 이력을 불러올 수 없습니다.')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void load()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const handleRun = async () => {
    if (running) return
    setRunning(true)
    setError(null)
    try {
      const result = await runDiagnosis('manual')
      const rows = await getDiagnosisHistory(12)
      setHistory(rows)
      // 완전 0건이면 이력이 비므로 방금 실행 결과를 직접 표시("이상 없음").
      setSelected(rows[0] ?? resultToRow(result))
    } catch (e) {
      setError(typeof e === 'string' ? e : '진단을 실행할 수 없습니다.')
    } finally {
      setRunning(false)
    }
  }

  const handleDeleteOne = async (id: number) => {
    setConfirm(null)
    try {
      await deleteDiagnosisHistory(id)
      const rows = await getDiagnosisHistory(12)
      setHistory(rows)
      // 삭제된 항목이 선택 중이었으면 첫 행으로, 아니면 선택 유지.
      setSelected((prev) => (prev !== null && rows.some((r) => r.id === prev.id) ? prev : (rows[0] ?? null)))
    } catch (e) {
      setError(typeof e === 'string' ? e : '이력 삭제에 실패했습니다.')
    }
  }

  const handleClearAll = async () => {
    setConfirm(null)
    try {
      await clearDiagnosisHistory()
      setHistory([])
      setSelected(null)
    } catch (e) {
      setError(typeof e === 'string' ? e : '이력 삭제에 실패했습니다.')
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-4xl">
        <div className="mb-4">
          <Link href="/settings" className="text-sm text-gray-500 hover:text-[var(--accent)]">
            ← 설정
          </Link>
        </div>
        <div className="mb-6 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold">데이터 자가 진단</h1>
            <p className="mt-1 text-base text-gray-600">
              원생·출결·청구 데이터의 정합성을 7개 항목으로 점검합니다. 매월 1일 자동 점검되며,
              아래 버튼으로 직접 실행할 수도 있습니다.
            </p>
          </div>
          <button
            type="button"
            onClick={handleRun}
            disabled={running}
            className="h-12 shrink-0 rounded-md bg-[var(--accent)] px-5 text-base font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            {running ? '점검 중...' : '자가 진단 실행'}
          </button>
        </div>

        {error !== null && (
          <p role="alert" className="mb-4 rounded-md border border-[var(--danger)] bg-red-50 p-3 text-sm text-[var(--danger)]">
            {error}
          </p>
        )}

        <div className="grid gap-6 md:grid-cols-[260px_1fr]">
          {/* 이력 목록 */}
          <section aria-label="진단 이력" className="rounded-lg border border-[var(--border)] bg-white p-4">
            <div className="mb-3 flex items-center justify-between gap-2">
              <h2 className="text-lg font-bold">최근 이력</h2>
              {history.length > 0 && (
                <button
                  type="button"
                  onClick={() => setConfirm({ kind: 'all' })}
                  className="rounded px-2 py-1 text-sm text-[var(--danger)] hover:bg-red-50"
                >
                  이력 비우기
                </button>
              )}
            </div>
            {loading ? (
              <p className="text-sm text-gray-500">불러오는 중...</p>
            ) : history.length === 0 ? (
              <p className="text-sm text-gray-500">아직 진단 기록이 없습니다.</p>
            ) : (
              <ul className="space-y-1">
                {history.map((row) => {
                  const active = selected?.id === row.id
                  return (
                    <li key={row.id} className="flex items-stretch gap-1">
                      <button
                        type="button"
                        onClick={() => setSelected(row)}
                        className={`flex min-h-[44px] flex-1 flex-col items-start rounded-md px-3 py-2 text-left transition-colors ${
                          active ? 'bg-[var(--background)] ring-1 ring-[var(--accent)]' : 'hover:bg-[var(--background)]'
                        }`}
                      >
                        <span className="text-sm font-medium text-[var(--foreground)]">
                          {row.run_date} · {runTypeLabel(row.run_type)}
                        </span>
                        <span className={`text-xs ${row.issues_found > 0 ? 'text-[var(--danger)]' : 'text-green-600'}`}>
                          {row.issues_found > 0 ? `이상 ${row.issues_found}건` : '이상 없음'}
                        </span>
                      </button>
                      <button
                        type="button"
                        onClick={() => setConfirm({ kind: 'one', id: row.id })}
                        aria-label="이 진단 이력 삭제"
                        title="이 진단 이력 삭제"
                        className="flex min-h-[44px] w-11 shrink-0 items-center justify-center rounded-md text-gray-600 hover:bg-red-50 hover:text-[var(--danger)]"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </li>
                  )
                })}
              </ul>
            )}
          </section>

          {/* 선택 결과 상세 */}
          <section aria-label="진단 결과" className="rounded-lg border border-[var(--border)] bg-white p-5">
            {selected === null ? (
              <p className="text-base text-gray-500">
                진단 기록을 선택하거나 “자가 진단 실행”을 눌러 점검을 시작하세요.
              </p>
            ) : (
              <DiagnosisDetail row={selected} />
            )}
          </section>
        </div>
      </div>

      {confirm !== null && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="mx-4 w-full max-w-sm rounded-lg bg-white p-6 shadow-xl">
            <p className="text-base text-[var(--foreground)]">
              {confirm.kind === 'all'
                ? '모든 진단 이력을 삭제하시겠습니까? 이 작업은 되돌릴 수 없습니다.'
                : '이 진단 이력을 삭제하시겠습니까?'}
            </p>
            <div className="mt-5 flex justify-end gap-3">
              <button
                type="button"
                onClick={() => setConfirm(null)}
                className="h-11 rounded-md border border-[var(--border)] px-4 hover:bg-gray-50"
              >
                취소
              </button>
              <button
                type="button"
                onClick={() => {
                  if (confirm.kind === 'all') void handleClearAll()
                  else void handleDeleteOne(confirm.id)
                }}
                className="h-11 rounded-md bg-[var(--danger)] px-4 font-semibold text-white hover:opacity-90"
              >
                삭제
              </button>
            </div>
          </div>
        </div>
      )}
    </AppShell>
  )
}

function DiagnosisDetail({ row }: { row: DiagnosisHistoryRow }) {
  return (
    <div>
      <div className="mb-4 flex items-baseline justify-between gap-3">
        <h2 className="text-lg font-bold">
          {row.run_date} 진단 결과 ({runTypeLabel(row.run_type)})
        </h2>
        <span className="text-sm text-gray-500">검사 {row.total_checks}항목</span>
      </div>

      {row.issues_found === 0 ? (
        <p className="rounded-md border border-green-300 bg-green-50 p-4 text-base text-green-700">
          ✅ 이상이 발견되지 않았습니다. 데이터가 정상입니다.
        </p>
      ) : (
        <ul className="space-y-3">
          {row.issues.map((issue, idx) => (
            <IssueCard key={`${issue.check_id}-${issue.target_id ?? idx}`} issue={issue} />
          ))}
        </ul>
      )}
    </div>
  )
}

function IssueCard({ issue }: { issue: DiagnosisIssue }) {
  const isError = issue.severity === 'error'
  const link = targetLink(issue.target_table)
  const guide = GUIDE[issue.check_id]
  return (
    <li
      className={`rounded-md border p-4 ${
        isError ? 'border-red-300 bg-red-50' : 'border-amber-300 bg-amber-50'
      }`}
    >
      <div className="flex items-start gap-2">
        <span
          className={`mt-0.5 shrink-0 rounded px-2 py-0.5 text-xs font-bold ${
            isError ? 'bg-[var(--danger)] text-white' : 'bg-amber-500 text-white'
          }`}
        >
          {isError ? '오류' : '확인'}
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-base text-[var(--foreground)]">{issue.message}</p>
          {guide !== undefined && <p className="mt-1 text-sm text-gray-600">{guide}</p>}
          {link !== null && (
            <Link
              href={link.href}
              className="mt-2 inline-flex min-h-[44px] items-center text-sm font-medium text-[var(--accent)] hover:underline"
            >
              {link.label} →
            </Link>
          )}
        </div>
      </div>
    </li>
  )
}
