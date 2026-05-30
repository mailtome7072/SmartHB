'use client'

/**
 * 공지문 편집/생성 화면 — Sprint 12 T6 (PRD §4.10.1).
 *
 * 레이아웃 모델: 텍스트박스는 **배경 원본 해상도 대비 비율(0..1)** 로 저장한다.
 * 미리보기는 가용 영역에 맞춰 scale 로 축소 표시(react-rnd `scale` 로 드래그 좌표 보정),
 * 생성은 배경 원본 해상도로 렌더 → 텍스트도 비례 확대. 폰트는 박스 높이×fontRatio 로 자동 연동.
 *
 * - 좌/우 패널 사이 드래그 스플리터 (너비 localStorage 저장)
 * - 패널 높이는 창 높이에 연동 (h-full + 내부 스크롤)
 * - 배경서식 파일명 hover 미리보기
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

const LEFT_WIDTH_KEY = 'smarthb.notice.leftWidth'
const LEFT_MIN = 220
const LEFT_MAX = 560

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

  // 청구년월 — 청구 생성된 월만
  const monthsQuery = useQuery({ queryKey: ['billed-months'], queryFn: listBilledMonths })
  const monthOptions = useMemo(() => monthsQuery.data ?? [], [monthsQuery.data])
  const [yearMonth, setYearMonth] = useState<string>(currentYearMonth())
  useEffect(() => {
    if (monthOptions.length > 0 && !monthOptions.includes(yearMonth)) setYearMonth(monthOptions[0])
  }, [monthOptions, yearMonth])

  // 청구 원생 (confirmed)
  const billsQuery = useQuery({ queryKey: ['bills', yearMonth], queryFn: () => listBills(yearMonth) })
  const bills = useMemo(
    () => (billsQuery.data ?? []).filter((b) => b.status === 'confirmed'),
    [billsQuery.data],
  )
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set())
  useEffect(() => setSelectedIds(new Set()), [yearMonth])
  const allSelected = bills.length > 0 && selectedIds.size === bills.length
  const toggleAll = () => setSelectedIds(allSelected ? new Set() : new Set(bills.map((b) => b.id)))
  const toggleOne = (id: number) =>
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })

  // 배경서식 + 레이아웃
  const assetsQuery = useQuery({ queryKey: ['notice-assets'], queryFn: listNoticeAssets })
  const assets = assetsQuery.data ?? []
  const layoutQuery = useQuery({ queryKey: ['notice-layout'], queryFn: getNoticeLayout })
  const [layout, setLayout] = useState<NoticeLayout | null>(null)
  useEffect(() => {
    if (layoutQuery.data && layout === null) setLayout(layoutQuery.data)
  }, [layoutQuery.data, layout])

  const [bgDataUrl, setBgDataUrl] = useState<string | null>(null)
  const [bgDims, setBgDims] = useState<{ w: number; h: number }>({ w: 800, h: 800 })
  const [selectedBoxIdx, setSelectedBoxIdx] = useState(0)

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
    else setBgDataUrl(null)
  }, [layout?.backgroundAsset, loadBackground])

  // 파일명 hover 미리보기
  const previewCache = useRef<Map<string, string>>(new Map())
  const [hoverPreview, setHoverPreview] = useState<{ name: string; url: string } | null>(null)
  // 미리보기는 마우스 포인터 우측 하단을 따라다닌다.
  const [mousePos, setMousePos] = useState<{ x: number; y: number }>({ x: 0, y: 0 })
  const showAssetPreview = useCallback(async (name: string) => {
    const cached = previewCache.current.get(name)
    if (cached) {
      setHoverPreview({ name, url: cached })
      return
    }
    try {
      const bytes = await readNoticeAsset(name)
      if (bytes.length === 0) return
      const mime = name.toLowerCase().endsWith('.png') ? 'image/png' : 'image/jpeg'
      const url = bytesToDataUrl(bytes, mime)
      previewCache.current.set(name, url)
      setHoverPreview((prev) => (prev === null || prev.name === name ? { name, url } : prev))
    } catch {
      /* 미리보기 실패 무시 */
    }
  }, [])

  // 레이아웃 debounce 저장 (AC-4.10-3)
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const updateLayout = useCallback((next: NoticeLayout) => {
    setLayout(next)
    if (saveTimer.current) clearTimeout(saveTimer.current)
    saveTimer.current = setTimeout(() => void saveNoticeLayout(next).catch(() => {}), 500)
  }, [])
  const updateBox = (idx: number, patch: Partial<TextboxConfig>) => {
    if (!layout) return
    updateLayout({
      ...layout,
      textboxes: layout.textboxes.map((tb, i) => (i === idx ? { ...tb, ...patch } : tb)),
    })
  }

  // 미리보기 원생 데이터
  const previewBill: Bill | undefined = bills.find((b) => selectedIds.has(b.id)) ?? bills[0]
  const previewData: NoticeStudentData = previewBill
    ? { studentName: previewBill.studentName, billYearMonth: yearMonth, billAmount: previewBill.adjustedAmount }
    : { studentName: '원생 이름', billYearMonth: yearMonth, billAmount: 0 }

  // 업로드/삭제
  const fileInputRef = useRef<HTMLInputElement>(null)
  const handleUpload = async (file: File) => {
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()))
      const saved = await saveNoticeAsset(file.name, bytes)
      previewCache.current.delete(saved)
      await assetsQuery.refetch()
      if (layout) updateLayout({ ...layout, backgroundAsset: saved })
    } catch (e) {
      setError(e instanceof Error ? e.message : '배경서식 업로드 실패')
    }
  }
  const handleDeleteAsset = async (name: string) => {
    try {
      await deleteNoticeAsset(name)
      previewCache.current.delete(name)
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
    const exists = await checkNoticeOutputExists(yearMonth)
    if (exists && typeof window !== 'undefined') {
      if (!window.confirm(`${yearMonth} 폴더에 기존 공지문이 있습니다. 덮어쓰시겠습니까?`)) return
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

  // ── 스플리터 (좌 패널 너비, localStorage 저장) ──
  const [leftWidth, setLeftWidth] = useState<number>(320)
  useEffect(() => {
    if (typeof window === 'undefined') return
    const saved = Number(window.localStorage.getItem(LEFT_WIDTH_KEY))
    if (saved >= LEFT_MIN && saved <= LEFT_MAX) setLeftWidth(saved)
  }, [])
  const rowRef = useRef<HTMLDivElement>(null)
  const draggingSplit = useRef(false)
  const onSplitDown = () => {
    draggingSplit.current = true
    if (typeof document !== 'undefined') document.body.style.userSelect = 'none'
  }
  useEffect(() => {
    if (typeof window === 'undefined') return
    const onMove = (e: MouseEvent) => {
      if (!draggingSplit.current || !rowRef.current) return
      const rect = rowRef.current.getBoundingClientRect()
      const w = Math.min(LEFT_MAX, Math.max(LEFT_MIN, e.clientX - rect.left))
      setLeftWidth(w)
    }
    const onUp = () => {
      if (!draggingSplit.current) return
      draggingSplit.current = false
      document.body.style.userSelect = ''
      window.localStorage.setItem(LEFT_WIDTH_KEY, String(Math.round(leftWidth)))
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [leftWidth])

  // ── 미리보기 가용 영역 측정 → scale (창 크기 연동) ──
  const previewWrapRef = useRef<HTMLDivElement>(null)
  const [avail, setAvail] = useState<{ w: number; h: number }>({ w: 600, h: 600 })
  useEffect(() => {
    if (typeof window === 'undefined' || !previewWrapRef.current) return
    const el = previewWrapRef.current
    const ro = new ResizeObserver((entries) => {
      const cr = entries[0]?.contentRect
      if (cr) setAvail({ w: cr.width, h: cr.height })
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [bgDataUrl])
  const scale = Math.min(avail.w / bgDims.w, avail.h / bgDims.h) || 0.1

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="flex h-full flex-col">
        <h1 className="mb-3 text-2xl font-bold">공지문 생성</h1>

        <div ref={rowRef} className="flex min-h-0 flex-1">
          {/* 좌측: 원생 리스트 */}
          <section
            className="flex flex-col overflow-hidden rounded-md border border-[var(--border)] p-3"
            style={{ width: leftWidth, flex: '0 0 auto' }}
          >
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
                <label className="mb-2 flex min-h-[40px] cursor-pointer items-center gap-2 border-b border-[var(--border)] text-base font-medium">
                  <input type="checkbox" checked={allSelected} onChange={toggleAll} className="h-5 w-5" />
                  전체 선택 ({selectedIds.size}/{bills.length})
                </label>
                <ul className="min-h-0 flex-1 overflow-y-auto">
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

          {/* 스플리터 */}
          <div
            role="separator"
            aria-orientation="vertical"
            onMouseDown={onSplitDown}
            className="mx-1 w-1.5 shrink-0 cursor-col-resize rounded bg-gray-200 hover:bg-[var(--accent)]"
            title="드래그하여 패널 너비 조절"
          />

          {/* 우측: 편집 캔버스 */}
          <section className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-md border border-[var(--border)] p-3">
            {/* 배경서식 관리 */}
            <div className="mb-2">
              <div className="mb-2 flex items-center gap-2">
                <span className="text-base font-medium">배경서식</span>
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
                  className="h-9 rounded-md border border-[var(--accent)] px-3 text-sm text-[var(--accent)] hover:bg-blue-50"
                >
                  업로드
                </button>
                <span className="text-xs text-gray-500">파일명에 마우스를 올리면 미리보기</span>
              </div>
              {assets.length === 0 ? (
                <p className="rounded-md bg-gray-50 px-3 py-2 text-sm text-gray-600">
                  업로드된 배경서식이 없습니다.
                </p>
              ) : (
                <div className="relative">
                  <ul className="max-h-28 divide-y divide-[var(--border)] overflow-y-auto rounded-md border border-[var(--border)]">
                    {assets.map((a) => {
                      const selected = layout?.backgroundAsset === a.name
                      return (
                        <li
                          key={a.name}
                          className={`flex items-center gap-2 px-3 py-1.5 ${selected ? 'bg-blue-50' : 'hover:bg-gray-50'}`}
                          onMouseEnter={(e) => {
                            setMousePos({ x: e.clientX, y: e.clientY })
                            void showAssetPreview(a.name)
                          }}
                          onMouseMove={(e) => setMousePos({ x: e.clientX, y: e.clientY })}
                          onMouseLeave={() => setHoverPreview(null)}
                        >
                          <button
                            type="button"
                            onClick={() => layout && updateLayout({ ...layout, backgroundAsset: a.name })}
                            className={`flex-1 truncate text-left text-sm ${selected ? 'font-semibold text-[var(--accent)]' : 'text-gray-800'}`}
                            title={a.name}
                          >
                            {selected ? '✓ ' : ''}{a.name}
                          </button>
                          <button
                            type="button"
                            onClick={() => handleDeleteAsset(a.name)}
                            aria-label={`${a.name} 삭제`}
                            className="rounded px-1.5 text-sm text-gray-400 hover:bg-red-50 hover:text-[var(--danger)]"
                          >
                            ✕
                          </button>
                        </li>
                      )
                    })}
                  </ul>
                  {hoverPreview && (
                    <div
                      className="pointer-events-none fixed z-50 rounded-md border border-[var(--border)] bg-white p-1 shadow-lg"
                      style={{
                        left:
                          typeof window !== 'undefined'
                            ? Math.min(mousePos.x + 14, window.innerWidth - 244)
                            : mousePos.x + 14,
                        top:
                          typeof window !== 'undefined'
                            ? Math.min(mousePos.y + 14, window.innerHeight - 224)
                            : mousePos.y + 14,
                      }}
                    >
                      {/* eslint-disable-next-line @next/next/no-img-element */}
                      <img src={hoverPreview.url} alt={`${hoverPreview.name} 미리보기`} className="max-h-48 max-w-[220px] object-contain" />
                      <p className="mt-1 max-w-[220px] truncate text-center text-xs text-gray-600">{hoverPreview.name}</p>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* 폰트 툴바 */}
            {layout && layout.textboxes[selectedBoxIdx] && (
              <div className="mb-2 flex flex-wrap items-center gap-2 rounded-md bg-gray-50 p-2 text-sm">
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
                  글자비율
                  <input
                    type="range"
                    min={10}
                    max={100}
                    value={Math.round(layout.textboxes[selectedBoxIdx].fontRatio * 100)}
                    onChange={(e) => updateBox(selectedBoxIdx, { fontRatio: Number(e.target.value) / 100 })}
                  />
                  <span className="w-8 text-right">
                    {Math.round(layout.textboxes[selectedBoxIdx].fontRatio * 100)}%
                  </span>
                </label>
                <button
                  type="button"
                  onClick={() =>
                    updateBox(selectedBoxIdx, {
                      fontWeight: layout.textboxes[selectedBoxIdx].fontWeight === 'bold' ? 'normal' : 'bold',
                    })
                  }
                  className={`h-9 rounded border px-2 ${layout.textboxes[selectedBoxIdx].fontWeight === 'bold' ? 'border-[var(--accent)] bg-blue-50 font-bold text-[var(--accent)]' : 'border-[var(--border)]'}`}
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
                    className={`h-9 rounded border px-2 ${layout.textboxes[selectedBoxIdx].textAlign === al ? 'border-[var(--accent)] bg-blue-50 text-[var(--accent)]' : 'border-[var(--border)]'}`}
                  >
                    {al === 'left' ? '좌' : al === 'center' ? '중' : '우'}
                  </button>
                ))}
              </div>
            )}

            {/* 미리보기 캔버스 (가용 영역 채움) */}
            <div ref={previewWrapRef} className="flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-gray-100">
              {bgDataUrl && layout ? (
                <div
                  className="relative border border-dashed border-gray-300"
                  style={{ width: bgDims.w * scale, height: bgDims.h * scale }}
                >
                  <div style={{ width: bgDims.w, height: bgDims.h, transform: `scale(${scale})`, transformOrigin: 'top left', position: 'relative' }}>
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
                    {layout.textboxes.map((tb, i) => {
                      const boxH = tb.hRatio * bgDims.h
                      return (
                        <Rnd
                          key={tb.fieldType}
                          scale={scale}
                          bounds="parent"
                          position={{ x: tb.xRatio * bgDims.w, y: tb.yRatio * bgDims.h }}
                          size={{ width: tb.wRatio * bgDims.w, height: boxH }}
                          onDragStop={(_e, d) =>
                            updateBox(i, { xRatio: d.x / bgDims.w, yRatio: d.y / bgDims.h })
                          }
                          onResizeStop={(_e, _dir, ref, _delta, pos) =>
                            updateBox(i, {
                              wRatio: parseFloat(ref.style.width) / bgDims.w,
                              hRatio: parseFloat(ref.style.height) / bgDims.h,
                              xRatio: pos.x / bgDims.w,
                              yRatio: pos.y / bgDims.h,
                            })
                          }
                          onMouseDown={() => setSelectedBoxIdx(i)}
                          style={{ outline: i === selectedBoxIdx ? '2px solid var(--accent)' : '1px dashed #999' }}
                        >
                          <div
                            style={{
                              width: '100%',
                              height: '100%',
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: tb.textAlign === 'center' ? 'center' : tb.textAlign === 'right' ? 'flex-end' : 'flex-start',
                              fontSize: tb.fontRatio * boxH,
                              fontWeight: tb.fontWeight,
                              color: tb.fontColor,
                              textAlign: tb.textAlign,
                              lineHeight: 1.2,
                              overflow: 'hidden',
                              cursor: 'move',
                            }}
                          >
                            {noticeFieldText(tb.fieldType, previewData)}
                          </div>
                        </Rnd>
                      )
                    })}
                  </div>
                </div>
              ) : (
                <p className="text-center text-gray-600">
                  배경서식을 업로드하거나 선택하면 편집 미리보기가 표시됩니다.
                </p>
              )}
            </div>

            {/* 생성 */}
            <div className="mt-2 flex items-center gap-3">
              <button
                type="button"
                onClick={handleGenerate}
                disabled={generating || !layout?.backgroundAsset || selectedIds.size === 0}
                className="h-11 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
              >
                {generating ? `생성 중... ${progress ? `(${progress.done}/${progress.total})` : ''}` : `발송용 공지문 생성 (${selectedIds.size}명)`}
              </button>
              <span className="text-sm text-gray-500">저장 위치: output/{yearMonth.replace('-', '')}/</span>
            </div>
          </section>
        </div>
      </div>

      <ErrorDialog open={error !== null && error !== ''} message={error ?? ''} onClose={() => setError(null)} />
    </AppShell>
  )
}
