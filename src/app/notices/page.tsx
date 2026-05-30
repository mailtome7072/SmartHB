'use client'

/**
 * 공지문 편집/생성 화면 — Sprint 12 T6 (PRD §4.10.1).
 *
 * 좌측: 청구 대상 원생(confirmed) 리스트 + 전체/개별 선택 + 년월 선택.
 * 우측: 배경서식 미리보기 + 텍스트박스 3종(react-rnd 드래그/리사이즈) + 폰트 툴바.
 * 하단: "발송용 공지문 생성" — 선택 원생 일괄 PNG 생성·저장 (AC-4.10-1/2).
 *
 * 레이아웃(배경/텍스트박스)은 변경 시 debounce 저장(saveNoticeLayout), 진입 시 로드(getNoticeLayout) — AC-4.10-3.
 */

import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Rnd } from 'react-rnd'
import { useQuery } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { ErrorDialog } from '@/components/ui/error-dialog'
import {
  checkNoticeOutputExists,
  deleteNoticeAsset,
  getNoticeLayout,
  listBilledMonths,
  listBills,
  listNoticeAssets,
  readNoticeAsset,
  saveNoticeAsset,
  saveNoticeLayout,
} from '@/lib/tauri'
import {
  bytesToDataUrl,
  generateAndSaveNotices,
  noticeFieldText,
  type NoticeStudentData,
} from '@/lib/notice-generator'
import type { NoticeLayout, NoticeFieldType, TextboxConfig } from '@/types/notice'
import type { Bill } from '@/types/billing'

const FIELD_LABEL: Record<NoticeFieldType, string> = {
  bill_month: '청구월',
  student_name: '원생명',
  bill_amount: '청구액',
}

function currentYearMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

export default function NoticesPage() {
  return (
    <Suspense fallback={<SplashScreen message="공지문 화면을 여는 중입니다..." />}>
      <NoticesContent />
    </Suspense>
  )
}

function NoticesContent() {
  const [error, setError] = useState<string | null>(null)

  // 년월 선택 — 청구 생성된 년월만
  const monthsQuery = useQuery({ queryKey: ['billed-months'], queryFn: listBilledMonths })
  const monthOptions = useMemo(() => monthsQuery.data ?? [], [monthsQuery.data])
  const [yearMonth, setYearMonth] = useState<string>(currentYearMonth())
  useEffect(() => {
    if (monthOptions.length > 0 && !monthOptions.includes(yearMonth)) setYearMonth(monthOptions[0])
  }, [monthOptions, yearMonth])

  // 청구 원생 (confirmed 만 — 공지문 대상)
  const billsQuery = useQuery({
    queryKey: ['bills', yearMonth],
    queryFn: () => listBills(yearMonth),
  })
  const bills = useMemo(
    () => (billsQuery.data ?? []).filter((b) => b.status === 'confirmed'),
    [billsQuery.data],
  )
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set())
  // 년월 변경 시 선택 초기화
  useEffect(() => {
    setSelectedIds(new Set())
  }, [yearMonth])

  const allSelected = bills.length > 0 && selectedIds.size === bills.length
  const toggleAll = () =>
    setSelectedIds(allSelected ? new Set() : new Set(bills.map((b) => b.id)))
  const toggleOne = (id: number) =>
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })

  // 배경서식
  const assetsQuery = useQuery({ queryKey: ['notice-assets'], queryFn: listNoticeAssets })
  const assets = assetsQuery.data ?? []

  // 레이아웃
  const layoutQuery = useQuery({ queryKey: ['notice-layout'], queryFn: getNoticeLayout })
  const [layout, setLayout] = useState<NoticeLayout | null>(null)
  useEffect(() => {
    if (layoutQuery.data && layout === null) setLayout(layoutQuery.data)
  }, [layoutQuery.data, layout])

  // 배경 이미지 data URL + 자연 크기
  const [bgDataUrl, setBgDataUrl] = useState<string | null>(null)
  const [bgDims, setBgDims] = useState<{ w: number; h: number }>({ w: 800, h: 800 })
  const [selectedBoxIdx, setSelectedBoxIdx] = useState(0)

  // 선택된 배경서식 로드
  const loadBackground = useCallback(async (name: string | null) => {
    if (!name) {
      setBgDataUrl(null)
      return
    }
    try {
      const bytes = await readNoticeAsset(name)
      if (bytes.length === 0) {
        setBgDataUrl(null)
        return
      }
      const mime = name.toLowerCase().endsWith('.png') ? 'image/png' : 'image/jpeg'
      setBgDataUrl(bytesToDataUrl(bytes, mime))
    } catch (e) {
      setError(e instanceof Error ? e.message : '배경서식을 불러올 수 없습니다.')
    }
  }, [])

  useEffect(() => {
    if (layout?.backgroundAsset) void loadBackground(layout.backgroundAsset)
  }, [layout?.backgroundAsset, loadBackground])

  // 레이아웃 변경 시 debounce 저장 (AC-4.10-3)
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const updateLayout = useCallback((next: NoticeLayout) => {
    setLayout(next)
    if (saveTimer.current) clearTimeout(saveTimer.current)
    saveTimer.current = setTimeout(() => {
      void saveNoticeLayout(next).catch(() => {})
    }, 500)
  }, [])

  const updateBox = (idx: number, patch: Partial<TextboxConfig>) => {
    if (!layout) return
    const textboxes = layout.textboxes.map((tb, i) => (i === idx ? { ...tb, ...patch } : tb))
    updateLayout({ ...layout, textboxes })
  }

  // 미리보기용 원생 데이터 (선택 첫 원생 또는 첫 청구)
  const previewBill: Bill | undefined =
    bills.find((b) => selectedIds.has(b.id)) ?? bills[0]
  const previewData: NoticeStudentData = previewBill
    ? {
        studentName: previewBill.studentName,
        billYearMonth: yearMonth,
        billAmount: previewBill.adjustedAmount,
      }
    : { studentName: '원생 이름', billYearMonth: yearMonth, billAmount: 0 }

  // 업로드
  const fileInputRef = useRef<HTMLInputElement>(null)
  const handleUpload = async (file: File) => {
    try {
      const buf = await file.arrayBuffer()
      const bytes = Array.from(new Uint8Array(buf))
      const saved = await saveNoticeAsset(file.name, bytes)
      await assetsQuery.refetch()
      if (layout) updateLayout({ ...layout, backgroundAsset: saved })
    } catch (e) {
      setError(e instanceof Error ? e.message : '배경서식 업로드 실패')
    }
  }

  const handleDeleteAsset = async (name: string) => {
    try {
      await deleteNoticeAsset(name)
      await assetsQuery.refetch()
      if (layout?.backgroundAsset === name) updateLayout({ ...layout, backgroundAsset: null })
    } catch (e) {
      setError(e instanceof Error ? e.message : '배경서식 삭제 실패')
    }
  }

  // 일괄 생성
  const [generating, setGenerating] = useState(false)
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null)
  const handleGenerate = async () => {
    if (!layout?.backgroundAsset || !bgDataUrl) {
      setError('배경서식을 먼저 선택해 주세요.')
      return
    }
    const targets = bills.filter((b) => selectedIds.has(b.id))
    if (targets.length === 0) {
      setError('공지문을 생성할 원생을 선택해 주세요.')
      return
    }
    // AC-4.10-2: 기존 파일 존재 시 덮어쓰기 확인
    const exists = await checkNoticeOutputExists(yearMonth)
    if (exists && typeof window !== 'undefined') {
      const ok = window.confirm(
        `${yearMonth} 폴더에 기존 공지문이 있습니다. 덮어쓰시겠습니까?`,
      )
      if (!ok) return
    }
    setGenerating(true)
    setProgress({ done: 0, total: targets.length })
    try {
      const result = await generateAndSaveNotices({
        yearMonth,
        backgroundDataUrl: bgDataUrl,
        width: bgDims.w,
        height: bgDims.h,
        layout,
        students: targets.map((b) => ({
          studentName: b.studentName,
          billYearMonth: yearMonth,
          billAmount: b.adjustedAmount,
        })),
        onProgress: (done, total) => setProgress({ done, total }),
      })
      setError(`✅ ${result.saved}건 생성 완료. 저장 위치: output/${yearMonth.replace('-', '')}/`)
    } catch (e) {
      setError(e instanceof Error ? e.message : '공지문 생성 실패')
    } finally {
      setGenerating(false)
      setProgress(null)
    }
  }

  // 미리보기 표시 스케일 (자연 크기를 패널 폭에 맞춤)
  const MAX_PREVIEW_W = 460
  const scale = Math.min(1, MAX_PREVIEW_W / bgDims.w)

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-6xl">
        <h1 className="mb-4 text-2xl font-bold">공지문 생성</h1>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-[320px_1fr]">
          {/* 좌측: 원생 리스트 */}
          <section className="rounded-md border border-[var(--border)] p-3">
            <label className="mb-2 block text-base font-medium">
              청구년월
              <select
                value={yearMonth}
                onChange={(e) => setYearMonth(e.target.value)}
                className="ml-2 h-10 rounded-md border border-[var(--border)] px-2 text-base"
              >
                {(monthOptions.length > 0 ? monthOptions : [yearMonth]).map((m) => (
                  <option key={m} value={m}>{m}</option>
                ))}
              </select>
            </label>

            {bills.length === 0 ? (
              <p className="py-6 text-center text-sm text-gray-600">
                확정된 청구가 없습니다. 청구/수납 관리에서 확정 후 이용하세요.
              </p>
            ) : (
              <>
                <label className="mb-2 flex min-h-[44px] cursor-pointer items-center gap-2 border-b border-[var(--border)] text-base font-medium">
                  <input type="checkbox" checked={allSelected} onChange={toggleAll} className="h-5 w-5" />
                  전체 선택 ({selectedIds.size}/{bills.length})
                </label>
                <ul className="max-h-[460px] overflow-y-auto">
                  {bills.map((b) => (
                    <li key={b.id}>
                      <label className="flex min-h-[40px] cursor-pointer items-center gap-2 py-1 text-base">
                        <input
                          type="checkbox"
                          checked={selectedIds.has(b.id)}
                          onChange={() => toggleOne(b.id)}
                          className="h-5 w-5"
                        />
                        <span className="font-medium">{b.studentName}</span>
                        <span className="ml-auto text-sm text-gray-600">
                          {b.adjustedAmount.toLocaleString()}원
                        </span>
                      </label>
                    </li>
                  ))}
                </ul>
              </>
            )}
          </section>

          {/* 우측: 편집 캔버스 */}
          <section className="rounded-md border border-[var(--border)] p-3">
            {/* 배경서식 관리 */}
            <div className="mb-3 flex flex-wrap items-center gap-2">
              <label className="text-base font-medium">
                배경서식
                <select
                  value={layout?.backgroundAsset ?? ''}
                  onChange={(e) =>
                    layout && updateLayout({ ...layout, backgroundAsset: e.target.value || null })
                  }
                  className="ml-2 h-10 rounded-md border border-[var(--border)] px-2 text-base"
                >
                  <option value="">선택</option>
                  {assets.map((a) => (
                    <option key={a.name} value={a.name}>{a.name}</option>
                  ))}
                </select>
              </label>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/png,image/jpeg"
                className="hidden"
                onChange={(e) => {
                  const f = e.target.files?.[0]
                  if (f) void handleUpload(f)
                  e.target.value = ''
                }}
              />
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="h-10 rounded-md border border-[var(--accent)] px-3 text-base text-[var(--accent)] hover:bg-blue-50"
              >
                업로드
              </button>
              {layout?.backgroundAsset && (
                <button
                  type="button"
                  onClick={() => handleDeleteAsset(layout.backgroundAsset!)}
                  className="h-10 rounded-md border border-[var(--border)] px-3 text-base text-gray-700 hover:bg-gray-50"
                >
                  삭제
                </button>
              )}
            </div>

            {/* 폰트 툴바 (선택된 텍스트박스) */}
            {layout && layout.textboxes[selectedBoxIdx] && (
              <div className="mb-3 flex flex-wrap items-center gap-2 rounded-md bg-gray-50 p-2 text-sm">
                <select
                  value={selectedBoxIdx}
                  onChange={(e) => setSelectedBoxIdx(Number(e.target.value))}
                  className="h-9 rounded border border-[var(--border)] px-2"
                >
                  {layout.textboxes.map((tb, i) => (
                    <option key={tb.fieldType} value={i}>{FIELD_LABEL[tb.fieldType]}</option>
                  ))}
                </select>
                <label className="flex items-center gap-1">
                  크기
                  <input
                    type="number"
                    min={8}
                    max={120}
                    value={layout.textboxes[selectedBoxIdx].fontSize}
                    onChange={(e) => updateBox(selectedBoxIdx, { fontSize: Number(e.target.value) })}
                    className="h-9 w-16 rounded border border-[var(--border)] px-2"
                  />
                </label>
                <button
                  type="button"
                  onClick={() =>
                    updateBox(selectedBoxIdx, {
                      fontWeight:
                        layout.textboxes[selectedBoxIdx].fontWeight === 'bold' ? 'normal' : 'bold',
                    })
                  }
                  className={`h-9 rounded border px-2 ${
                    layout.textboxes[selectedBoxIdx].fontWeight === 'bold'
                      ? 'border-[var(--accent)] bg-blue-50 font-bold text-[var(--accent)]'
                      : 'border-[var(--border)]'
                  }`}
                >
                  굵게
                </button>
                <input
                  type="color"
                  value={layout.textboxes[selectedBoxIdx].fontColor}
                  onChange={(e) => updateBox(selectedBoxIdx, { fontColor: e.target.value })}
                  className="h-9 w-10 cursor-pointer rounded border border-[var(--border)]"
                  aria-label="글자 색"
                />
                {(['left', 'center', 'right'] as const).map((al) => (
                  <button
                    key={al}
                    type="button"
                    onClick={() => updateBox(selectedBoxIdx, { textAlign: al })}
                    className={`h-9 rounded border px-2 ${
                      layout.textboxes[selectedBoxIdx].textAlign === al
                        ? 'border-[var(--accent)] bg-blue-50 text-[var(--accent)]'
                        : 'border-[var(--border)]'
                    }`}
                  >
                    {al === 'left' ? '좌' : al === 'center' ? '중' : '우'}
                  </button>
                ))}
              </div>
            )}

            {/* 미리보기 캔버스 */}
            {bgDataUrl && layout ? (
              <div
                className="relative mx-auto overflow-hidden border border-dashed border-gray-300"
                style={{ width: bgDims.w * scale, height: bgDims.h * scale }}
              >
                <div
                  style={{
                    width: bgDims.w,
                    height: bgDims.h,
                    transform: `scale(${scale})`,
                    transformOrigin: 'top left',
                    position: 'relative',
                  }}
                >
                  {/* eslint-disable-next-line @next/next/no-img-element */}
                  <img
                    src={bgDataUrl}
                    alt="배경서식"
                    style={{ position: 'absolute', inset: 0, width: bgDims.w, height: bgDims.h }}
                    onLoad={(e) => {
                      const img = e.currentTarget
                      if (img.naturalWidth > 0) setBgDims({ w: img.naturalWidth, h: img.naturalHeight })
                    }}
                  />
                  {layout.textboxes.map((tb, i) => (
                    <Rnd
                      key={tb.fieldType}
                      scale={scale}
                      bounds="parent"
                      position={{ x: tb.x, y: tb.y }}
                      size={{ width: tb.width, height: tb.height }}
                      onDragStop={(_e, d) => updateBox(i, { x: d.x, y: d.y })}
                      onResizeStop={(_e, _dir, ref, _delta, pos) =>
                        updateBox(i, {
                          width: parseFloat(ref.style.width),
                          height: parseFloat(ref.style.height),
                          x: pos.x,
                          y: pos.y,
                        })
                      }
                      onMouseDown={() => setSelectedBoxIdx(i)}
                      style={{
                        outline: i === selectedBoxIdx ? '2px solid var(--accent)' : '1px dashed #999',
                      }}
                    >
                      <div
                        style={{
                          width: '100%',
                          height: '100%',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent:
                            tb.textAlign === 'center'
                              ? 'center'
                              : tb.textAlign === 'right'
                                ? 'flex-end'
                                : 'flex-start',
                          fontSize: tb.fontSize,
                          fontWeight: tb.fontWeight,
                          color: tb.fontColor,
                          textAlign: tb.textAlign,
                          lineHeight: 1.3,
                          overflow: 'hidden',
                          cursor: 'move',
                        }}
                      >
                        {noticeFieldText(tb.fieldType, previewData)}
                      </div>
                    </Rnd>
                  ))}
                </div>
              </div>
            ) : (
              <div className="flex h-64 items-center justify-center rounded-md bg-gray-50 text-center text-gray-600">
                배경서식을 업로드하거나 선택하면 편집 미리보기가 표시됩니다.
              </div>
            )}

            {/* 생성 버튼 */}
            <div className="mt-3 flex items-center gap-3">
              <button
                type="button"
                onClick={handleGenerate}
                disabled={generating || !layout?.backgroundAsset || selectedIds.size === 0}
                className="h-11 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
              >
                {generating
                  ? `생성 중... ${progress ? `(${progress.done}/${progress.total})` : ''}`
                  : `발송용 공지문 생성 (${selectedIds.size}명)`}
              </button>
              <span className="text-sm text-gray-500">
                저장 위치: output/{yearMonth.replace('-', '')}/
              </span>
            </div>
          </section>
        </div>
      </div>

      <ErrorDialog
        open={error !== null && error !== ''}
        message={error ?? ''}
        onClose={() => setError(null)}
      />
    </AppShell>
  )
}
