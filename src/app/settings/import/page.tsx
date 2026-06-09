'use client'

/**
 * 원생 CSV 가져오기 (Sprint 16 T2, PRD §4.13.1).
 *
 * 파일 선택(Tauri Dialog) → 미리보기(파싱·검증·중복판정, INSERT 없음) → 가져오기(백업 1회 후
 * 유효·비중복 행 INSERT) → 결과 요약. 대상은 students 한 테이블이며 학교·스케줄은 제외한다.
 * window.confirm 이 Tauri WebView 에서 차단되므로 가져오기 확인은 자체 모달을 쓴다.
 */

import { useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SettingsHomeLink } from '@/components/settings/SettingsHomeLink'
import { importStudentsCsv, previewStudentsCsv, selectCsvFile } from '@/lib/tauri'
import type { ImportPreviewResult, ImportResult, ImportRowStatus } from '@/types/import'

const STATUS_META: Record<ImportRowStatus, { label: string; cls: string }> = {
  ok: { label: '정상', cls: 'bg-green-50 text-green-700' },
  warning: { label: '확인', cls: 'bg-amber-50 text-amber-800' },
  duplicate: { label: '중복 · 제외', cls: 'bg-gray-100 text-gray-600' },
  error: { label: '오류 · 제외', cls: 'bg-red-50 text-[var(--danger)]' },
}

export default function ImportStudentsPage() {
  const [filePath, setFilePath] = useState<string | null>(null)
  const [preview, setPreview] = useState<ImportPreviewResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [importing, setImporting] = useState(false)
  const [result, setResult] = useState<ImportResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [confirmImport, setConfirmImport] = useState(false)

  const handlePick = async () => {
    setError(null)
    setResult(null)
    setPreview(null)
    const path = await selectCsvFile()
    if (path === null) return
    setFilePath(path)
    setLoading(true)
    try {
      setPreview(await previewStudentsCsv(path))
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : '미리보기를 불러오지 못했습니다.')
    } finally {
      setLoading(false)
    }
  }

  const handleImport = async () => {
    if (filePath === null) return
    setConfirmImport(false)
    setImporting(true)
    setError(null)
    try {
      const r = await importStudentsCsv(filePath)
      setResult(r)
      setPreview(null)
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : '가져오기에 실패했습니다.')
    } finally {
      setImporting(false)
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-4xl">
        <SettingsHomeLink />
        <div className="mb-6">
          <h1 className="text-2xl font-bold">원생 CSV 가져오기</h1>
          <p className="mt-1 text-sm text-gray-600">
            CSV 파일로 원생 명단을 일괄 등록합니다. 미리보기로 확인한 뒤 가져오며, 중복은 건너뜁니다.
          </p>
        </div>

        {/* CSV 형식 안내 */}
        <section className="mb-6 rounded-lg border border-[var(--border)] bg-white p-5 text-sm text-gray-700">
          <p className="mb-2 font-semibold text-[var(--foreground)]">CSV 형식 안내</p>
          <ul className="list-inside list-disc space-y-1">
            <li>첫 행에 컬럼 제목이 있어야 합니다.</li>
            <li>
              <b>필수</b>: 이름 · 학년 · 입교일 &nbsp;|&nbsp; <b>선택</b>: 성별 · 생년월일 · 연락처 ·
              일련번호
            </li>
            <li>학년은 <b>&ldquo;초3&rdquo; · &ldquo;중2&rdquo;</b> 형식, 날짜는 <b>2026-03-01</b> 형식으로 입력하세요.</li>
            <li>일련번호를 비우면 자동으로 채번하며, 학교·수업요일은 가져온 뒤 앱에서 입력합니다.</li>
          </ul>
        </section>

        {error !== null && (
          <p
            role="alert"
            className="mb-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        <div className="mb-6 flex flex-wrap items-center gap-3">
          <button
            type="button"
            onClick={() => void handlePick()}
            disabled={loading || importing}
            className="h-11 rounded-md border border-[var(--border)] px-5 text-base font-medium hover:bg-gray-50 disabled:opacity-50"
          >
            CSV 파일 선택
          </button>
          {filePath !== null && (
            <span className="text-sm text-gray-600 break-all">{filePath}</span>
          )}
          {loading && <span className="text-sm text-gray-600">미리보기를 분석하는 중...</span>}
        </div>

        {/* 가져오기 결과 요약 */}
        {result !== null && (
          <section className="mb-6 rounded-lg border-2 border-[var(--accent)] bg-white p-5">
            <h2 className="mb-3 text-lg font-bold">가져오기 완료</h2>
            <div className="flex flex-wrap gap-4 text-base">
              <span className="font-semibold text-green-700">등록 {result.inserted}건</span>
              <span className="text-gray-600">중복 건너뜀 {result.skipped}건</span>
              {result.errored > 0 && (
                <span className="text-[var(--danger)]">오류 {result.errored}건</span>
              )}
            </div>
            <p className="mt-3 text-sm text-gray-600">{result.backup_note}</p>
            {result.errors.length > 0 && (
              <ul className="mt-3 list-inside list-disc space-y-1 text-sm text-[var(--danger)]">
                {result.errors.map((msg) => (
                  <li key={msg}>{msg}</li>
                ))}
              </ul>
            )}
          </section>
        )}

        {/* 미리보기 테이블 */}
        {preview !== null && (
          <section className="rounded-lg border border-[var(--border)] bg-white p-5">
            <div className="mb-3 flex flex-wrap items-center justify-between gap-3">
              <div className="flex flex-wrap gap-4 text-base">
                <span className="font-semibold text-[var(--foreground)]">전체 {preview.total}건</span>
                <span className="text-green-700">가져올 수 있음 {preview.importable}건</span>
                {preview.duplicate > 0 && (
                  <span className="text-gray-600">중복 {preview.duplicate}건</span>
                )}
                {preview.error > 0 && (
                  <span className="text-[var(--danger)]">오류 {preview.error}건</span>
                )}
              </div>
              <button
                type="button"
                onClick={() => setConfirmImport(true)}
                disabled={preview.importable === 0 || importing}
                className="h-11 rounded-md bg-[var(--accent)] px-5 text-base font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
              >
                {importing ? '가져오는 중...' : `${preview.importable}건 가져오기`}
              </button>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full border-collapse text-sm">
                <thead>
                  <tr className="border-b border-[var(--border)] text-left text-gray-600">
                    <th className="p-2">#</th>
                    <th className="p-2">상태</th>
                    <th className="p-2">이름</th>
                    <th className="p-2">학년</th>
                    <th className="p-2">성별</th>
                    <th className="p-2">입교일</th>
                    <th className="p-2">일련번호</th>
                    <th className="p-2">비고</th>
                  </tr>
                </thead>
                <tbody>
                  {preview.rows.map((row) => {
                    const meta = STATUS_META[row.status]
                    return (
                      <tr key={row.row_number} className="border-b border-gray-100 align-top">
                        <td className="p-2 text-gray-500">{row.row_number}</td>
                        <td className="p-2">
                          <span className={`rounded px-2 py-0.5 text-xs font-medium ${meta.cls}`}>
                            {meta.label}
                          </span>
                        </td>
                        <td className="p-2">{row.name || '—'}</td>
                        <td className="p-2">{row.grade_label || '—'}</td>
                        <td className="p-2">{row.gender_label || '—'}</td>
                        <td className="p-2">{row.enroll_date || '—'}</td>
                        <td className="p-2">{row.serial_no ?? '자동'}</td>
                        <td className="p-2 text-gray-600">{row.messages.join(' / ')}</td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>
          </section>
        )}
      </div>

      {/* 가져오기 확인 모달 (window.confirm 차단 대체) */}
      {confirmImport && preview !== null && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
        >
          <div className="w-full max-w-md rounded-lg bg-white p-5 shadow-xl">
            <p className="mb-4 text-base text-gray-800">
              {preview.importable}건의 원생을 등록합니다.
              <br />
              가져오기 전 자동으로 백업이 생성됩니다. 진행하시겠습니까?
            </p>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => setConfirmImport(false)}
                className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50"
              >
                취소
              </button>
              <button
                type="button"
                onClick={() => void handleImport()}
                className="min-h-[44px] flex-1 rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90"
              >
                가져오기
              </button>
            </div>
          </div>
        </div>
      )}
    </AppShell>
  )
}
