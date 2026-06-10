'use client'

/**
 * 백업 관리 화면 — `/settings/backup` (Sprint 14 T7, PRD §5.4).
 *
 * - 4계층 백업 파일 목록 (시점 / 계층 / 크기)
 * - 선택한 백업의 "복원 리허설" 실행 — 격리된 사본으로 무결성 + 주요 테이블 행 수 검증
 * - 결과 표시: 성공/실패, 검증 건수, 손상 사유
 *
 * 리허설은 운영 DB 에 영향을 주지 않는다 (백업 사본만 열람 후 폐기). cipher off 개발 빌드는
 * 평문 백업만 리허설 대상이다 (R98).
 */

import Link from 'next/link'
import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { listBackups, relaunchApp, restoreBackup, runBackupRehearsal } from '@/lib/tauri'
import type { BackupLayer, BackupMetadata, RehearsalResult } from '@/types'

// 개발 빌드는 화면을 localhost dev 서버에서 로드하므로 relaunch() 시 화면을 못 띄운다.
// 개발 모드에서는 자동 재시작 대신 수동 안내. (T3 DB 폴더 변경과 동일 가드)
const IS_DEV = process.env.NODE_ENV !== 'production'

const LAYER_LABEL: Record<BackupLayer, string> = {
  exit: '종료 시',
  hourly: '시간별',
  daily: '일별',
  weekly: '주별',
}

/** 리허설이 검증하는 주요 테이블 → 한글 라벨. */
const TABLE_LABEL: Record<string, string> = {
  students: '원생',
  student_schedules: '수업 스케줄',
  regular_attendances: '정규 출결',
  makeup_attendances: '보강 출결',
  bills: '청구',
  payments: '수납',
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

/** ISO8601 UTC → 사용자 로컬 타임존 표시. */
function formatDateTime(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleString('ko-KR', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export default function BackupPage() {
  const [backups, setBackups] = useState<BackupMetadata[]>([])
  const [selected, setSelected] = useState<BackupMetadata | null>(null)
  const [loading, setLoading] = useState(true)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<RehearsalResult | null>(null)
  // 수동 복원: 확인 모달 / 진행 중 / 완료(재시작 안내).
  const [confirmRestore, setConfirmRestore] = useState(false)
  const [restoring, setRestoring] = useState(false)
  const [restored, setRestored] = useState(false)

  const load = async () => {
    setLoading(true)
    try {
      const rows = await listBackups()
      setBackups(rows)
      setSelected((prev) => prev ?? rows[0] ?? null)
    } catch (e) {
      setError(typeof e === 'string' ? e : '백업 목록을 불러올 수 없습니다.')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void load()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const handleRehearsal = async () => {
    if (running || selected === null) return
    setRunning(true)
    setError(null)
    setResult(null)
    try {
      const r = await runBackupRehearsal(selected.path)
      setResult(r)
    } catch (e) {
      setError(typeof e === 'string' ? e : '복원 리허설을 실행할 수 없습니다.')
    } finally {
      setRunning(false)
    }
  }

  // 수동 복원 — 선택 백업으로 현재 DB 교체(현재 DB 는 rollback 보존). 완료 후 재시작 필요.
  const handleRestore = async () => {
    if (restoring || selected === null) return
    setConfirmRestore(false)
    setRestoring(true)
    setError(null)
    try {
      await restoreBackup(selected.path)
      setRestored(true)
    } catch (e) {
      setError(typeof e === 'string' ? e : '백업 복원에 실패했습니다.')
    } finally {
      setRestoring(false)
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
        <div className="mb-2">
          <h1 className="text-2xl font-bold">백업 관리</h1>
          <p className="mt-1 text-base text-gray-600">
            자동 백업 파일이 실제로 복원 가능한지 “복원 리허설”로 미리 검증합니다. 리허설은 백업
            사본만 열어 점검하므로 현재 운영 데이터에는 전혀 영향을 주지 않습니다.
          </p>
        </div>

        {error !== null && (
          <p
            role="alert"
            className="mb-4 rounded-md border border-[var(--danger)] bg-red-50 p-3 text-sm text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        <div className="grid gap-6 md:grid-cols-[300px_1fr]">
          {/* 백업 목록 */}
          <section
            aria-label="백업 목록"
            className="rounded-lg border border-[var(--border)] bg-white p-4"
          >
            <h2 className="mb-3 text-lg font-bold">백업 파일</h2>
            {loading ? (
              <p className="text-sm text-gray-500">불러오는 중...</p>
            ) : backups.length === 0 ? (
              <p className="text-sm text-gray-500">
                아직 백업 파일이 없습니다. (개발 빌드에서는 암호화 백업이 생성되지 않습니다)
              </p>
            ) : (
              <ul className="space-y-1">
                {backups.map((b) => {
                  const active = selected?.path === b.path
                  return (
                    <li key={b.path}>
                      <button
                        type="button"
                        onClick={() => {
                          setSelected(b)
                          setResult(null)
                        }}
                        className={`flex min-h-[44px] w-full flex-col items-start rounded-md px-3 py-2 text-left transition-colors ${
                          active
                            ? 'bg-[var(--background)] ring-1 ring-[var(--accent)]'
                            : 'hover:bg-[var(--background)]'
                        }`}
                      >
                        <span className="text-sm font-medium text-[var(--foreground)]">
                          {formatDateTime(b.created_at)}
                        </span>
                        <span className="text-xs text-gray-500">
                          {LAYER_LABEL[b.layer]} · {formatBytes(b.size_bytes)}
                        </span>
                      </button>
                    </li>
                  )
                })}
              </ul>
            )}
          </section>

          {/* 리허설 실행/결과 */}
          <section
            aria-label="복원 리허설"
            className="rounded-lg border border-[var(--border)] bg-white p-5"
          >
            {selected === null ? (
              <p className="text-base text-gray-500">
                왼쪽에서 검증할 백업 파일을 선택하세요.
              </p>
            ) : (
              <div>
                <div className="mb-4 flex items-center justify-between gap-4">
                  <div className="min-w-0">
                    <h2 className="text-lg font-bold">복원 리허설</h2>
                    <p className="mt-1 truncate text-sm text-gray-500" title={selected.path}>
                      {formatDateTime(selected.created_at)} · {LAYER_LABEL[selected.layer]}
                    </p>
                  </div>
                  <div className="flex shrink-0 gap-2">
                    <button
                      type="button"
                      onClick={handleRehearsal}
                      disabled={running || restoring}
                      className="h-12 rounded-md bg-[var(--accent)] px-5 text-base font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
                    >
                      {running ? '검증 중...' : '복원 리허설 실행'}
                    </button>
                    <button
                      type="button"
                      onClick={() => setConfirmRestore(true)}
                      disabled={running || restoring}
                      className="h-12 rounded-md border-2 border-[var(--danger)] px-5 text-base font-bold text-[var(--danger)] hover:bg-red-50 disabled:opacity-50"
                    >
                      {restoring ? '복원 중...' : '이 백업으로 복원'}
                    </button>
                  </div>
                </div>

                {result === null ? (
                  <p className="text-base text-gray-500">
                    “복원 리허설 실행”을 누르면 이 백업의 사본을 만들어 무결성과 데이터 건수를
                    점검합니다.
                  </p>
                ) : (
                  <RehearsalReport result={result} />
                )}
              </div>
            )}
          </section>
        </div>

        {/* 복원 확인 모달 */}
        {confirmRestore && selected !== null && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
            <div className="w-full max-w-md rounded-lg border border-[var(--border)] bg-white p-6 shadow-xl">
              <h3 className="mb-2 text-lg font-bold text-[var(--danger)]">이 백업으로 복원할까요?</h3>
              <p className="mb-4 text-sm text-gray-700">
                <b>{formatDateTime(selected.created_at)}</b> ({LAYER_LABEL[selected.layer]}) 시점으로
                되돌립니다. 그 이후 입력한 데이터는 사라집니다.
                <br />
                현재 데이터는 <b>복원 직전 상태로 보존</b>되며(되돌리기 가능), 복원 후 앱을
                재시작해야 적용됩니다.
              </p>
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setConfirmRestore(false)}
                  className="h-11 rounded-md border border-[var(--border)] px-5 text-base hover:bg-gray-50"
                >
                  취소
                </button>
                <button
                  type="button"
                  onClick={handleRestore}
                  className="h-11 rounded-md bg-[var(--danger)] px-5 text-base font-bold text-white hover:opacity-90"
                >
                  복원 실행
                </button>
              </div>
            </div>
          </div>
        )}

        {/* 복원 완료 모달 — 재시작 안내 */}
        {restored && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
            <div className="w-full max-w-md rounded-lg border border-[var(--border)] bg-white p-6 shadow-xl">
              <h3 className="mb-2 text-lg font-bold">복원 완료</h3>
              {IS_DEV ? (
                <>
                  <p className="mb-4 text-sm text-gray-700">
                    백업으로 복원했습니다. <b>개발 모드</b>에서는 자동 재시작이 동작하지 않습니다.
                    앱을 직접 종료한 뒤 다시 실행해 주세요.
                  </p>
                  <button
                    type="button"
                    onClick={() => setRestored(false)}
                    className="h-12 w-full rounded-md bg-[var(--accent)] px-6 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
                  >
                    확인
                  </button>
                </>
              ) : (
                <>
                  <p className="mb-4 text-sm text-gray-700">
                    백업으로 복원했습니다. 변경을 적용하려면 앱을 재시작해야 합니다.
                  </p>
                  <button
                    type="button"
                    onClick={() => void relaunchApp()}
                    className="h-12 w-full rounded-md bg-[var(--accent)] px-6 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
                  >
                    지금 재시작
                  </button>
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </AppShell>
  )
}

function RehearsalReport({ result }: { result: RehearsalResult }) {
  if (!result.success) {
    return (
      <div className="rounded-md border border-red-300 bg-red-50 p-4">
        <p className="text-base font-bold text-[var(--danger)]">
          ⚠️ 이 백업은 복원에 사용할 수 없습니다.
        </p>
        {result.integrity_detail !== null && (
          <pre className="mt-2 max-h-48 overflow-auto whitespace-pre-wrap break-words text-sm text-red-700">
            {result.integrity_detail}
          </pre>
        )}
      </div>
    )
  }

  return (
    <div>
      <p className="rounded-md border border-green-300 bg-green-50 p-4 text-base text-green-700">
        ✅ 복원 가능한 정상 백업입니다. 무결성 검사를 통과했으며 주요 데이터{' '}
        {result.total_rows.toLocaleString('ko-KR')}건을 확인했습니다.
      </p>
      <table className="mt-4 w-full text-sm">
        <thead>
          <tr className="border-b border-[var(--border)] text-left text-gray-500">
            <th className="py-2 font-medium">항목</th>
            <th className="py-2 text-right font-medium">건수</th>
          </tr>
        </thead>
        <tbody>
          {result.table_counts.map((tc) => (
            <tr key={tc.table} className="border-b border-[var(--border)]/50">
              <td className="py-2 text-[var(--foreground)]">
                {TABLE_LABEL[tc.table] ?? tc.table}
              </td>
              <td className="py-2 text-right tabular-nums text-[var(--foreground)]">
                {tc.count.toLocaleString('ko-KR')}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
