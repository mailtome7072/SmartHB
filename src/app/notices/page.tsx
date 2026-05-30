'use client'

/**
 * 공지문 편집/생성 화면 — Sprint 12 T6 (PRD §4.10.1).
 *
 * 레이아웃 모델: 텍스트박스는 **배경 원본 해상도 대비 비율(0..1)** 로 저장한다.
 * 미리보기는 가용 영역에 맞춰 scale 로 축소 표시(react-rnd `scale` 로 드래그 좌표 보정),
 * 생성은 배경 원본 해상도로 렌더 → 텍스트도 비례 확대. 폰트는 박스 높이×fontRatio 로 자동 연동.
 *
 * - 좌측 원생 패널은 고정 너비, 우측 편집 캔버스가 나머지 공간 차지
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
  deleteNoticeLayoutNamed,
  getNoticeLayout,
  getNoticeLayoutNamed,
  listBilledMonths,
  listBills,
  listNoticeAssets,
  listNoticeLayouts,
  readNoticeAsset,
  saveNoticeAsset,
  saveNoticeLayout,
  saveNoticeLayoutNamed,
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
  custom: '텍스트',
}

/** 체크박스/편집 라벨 — custom 은 입력 텍스트(없으면 '텍스트'). */
function boxLabel(tb: TextboxConfig): string {
  if (tb.fieldType === 'custom') return tb.text?.trim() || '텍스트'
  return FIELD_LABEL[tb.fieldType]
}

const LEFT_PANEL_WIDTH = 240 // 좌측 원생 패널 고정 너비(최소)
const RIGHT_PANEL_WIDTH = 220 // 우측 저장 패널 고정 너비

function currentYearMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

/** 텍스트 정렬 아이콘 — 정렬 방향에 맞춰 길이가 다른 가로선 3줄. */
function AlignIcon({ align }: { align: 'left' | 'center' | 'right' }) {
  // 각 줄을 정렬 방향 기준으로 배치 (full / short / mid).
  const lines =
    align === 'left'
      ? [[1, 13], [1, 8], [1, 11]]
      : align === 'right'
        ? [[1, 13], [6, 13], [3, 13]]
        : [[1, 13], [3.5, 10.5], [2.5, 11.5]]
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" aria-hidden="true">
      {lines.map(([x1, x2], i) => (
        <line key={i} x1={x1} x2={x2} y1={3 + i * 4} y2={3 + i * 4} />
      ))}
    </svg>
  )
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
  // 배경서식 드롭다운(콤보박스) 열림 상태.
  const [assetMenuOpen, setAssetMenuOpen] = useState(false)
  // 드롭다운이 닫히면 미리보기도 정리 (선택 시 항목 언마운트로 onMouseLeave 누락 방지).
  useEffect(() => {
    if (!assetMenuOpen) setHoverPreview(null)
  }, [assetMenuOpen])
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

  // 사용자 정의(custom) 텍스트박스 추가 — 중앙 부근 기본 배치.
  const addTextbox = () => {
    if (!layout) return
    const newBox: TextboxConfig = {
      id: `custom-${Date.now()}`,
      fieldType: 'custom',
      text: '텍스트',
      enabled: true,
      xRatio: 0.3,
      yRatio: 0.45,
      wRatio: 0.4,
      hRatio: 0.1,
      fontRatio: 0.5,
      fontWeight: 'normal',
      fontColor: '#1A1A1A',
      textAlign: 'center',
    }
    updateLayout({ ...layout, textboxes: [...layout.textboxes, newBox] })
    setSelectedBoxIdx(layout.textboxes.length) // 새 박스 선택
  }

  // custom 텍스트박스 삭제 (데이터 필드 3종은 삭제 불가).
  const removeTextbox = (idx: number) => {
    if (!layout) return
    updateLayout({ ...layout, textboxes: layout.textboxes.filter((_, i) => i !== idx) })
    setSelectedBoxIdx(0)
  }

  // 선택된 텍스트박스가 체크 해제(비활성)면 폰트 컨트롤 비활성.
  const selDisabled =
    !layout?.textboxes[selectedBoxIdx] || layout.textboxes[selectedBoxIdx].enabled === false

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
      setHoverPreview(null)
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

  // ── 저장 템플릿 ──
  const templatesQuery = useQuery({ queryKey: ['notice-layouts'], queryFn: listNoticeLayouts })
  const templates = templatesQuery.data ?? []

  const handleSaveNotice = async () => {
    if (!layout) return
    try {
      await saveNoticeLayout(layout)
      setError('✅ 공지문 레이아웃이 저장되었습니다.')
    } catch (e) {
      setError(e instanceof Error ? e.message : '저장 실패')
    }
  }
  const handleSaveAs = async () => {
    if (!layout || typeof window === 'undefined') return
    const name = window.prompt('다른 이름으로 저장 — 템플릿 이름을 입력하세요.')
    if (!name || name.trim() === '') return
    try {
      await saveNoticeLayoutNamed(name.trim(), layout)
      await templatesQuery.refetch()
      setError(`✅ '${name.trim()}' 템플릿으로 저장되었습니다.`)
    } catch (e) {
      setError(e instanceof Error ? e.message : '다른 이름으로 저장 실패')
    }
  }
  const handleLoadTemplate = async (name: string) => {
    try {
      const loaded = await getNoticeLayoutNamed(name)
      updateLayout(loaded)
      setSelectedBoxIdx(0)
      setError(`'${name}' 템플릿을 불러왔습니다.`)
    } catch (e) {
      setError(e instanceof Error ? e.message : '템플릿 불러오기 실패')
    }
  }
  const handleDeleteTemplate = async (name: string) => {
    try {
      await deleteNoticeLayoutNamed(name)
      await templatesQuery.refetch()
    } catch (e) {
      setError(e instanceof Error ? e.message : '템플릿 삭제 실패')
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="flex h-full flex-col">
        <h1 className="mb-3 text-2xl font-bold">공지문 생성</h1>

        <div className="flex min-h-0 flex-1 gap-3">
          {/* 좌측: 원생 리스트 (고정 너비) */}
          <section
            className="flex flex-col overflow-hidden rounded-md border border-[var(--border)] p-3"
            style={{ width: LEFT_PANEL_WIDTH, flex: '0 0 auto' }}
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

          {/* 우측: 편집 캔버스 */}
          <section className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-md border border-[var(--border)] p-3">
            {/* 배경서식 관리 — 한 줄: 콤보박스 + 업로드, 안내 텍스트 우측 */}
            <div className="mb-2 flex items-center gap-2">
              <span className="text-base font-medium">배경서식</span>

              {assets.length === 0 ? (
                <span className="text-sm text-gray-600">업로드된 배경서식이 없습니다.</span>
              ) : (
                <div className="relative inline-block">
                  {/* 콤보박스 버튼 */}
                  <button
                    type="button"
                    onClick={() => setAssetMenuOpen((o) => !o)}
                    className="flex h-9 min-w-[220px] items-center justify-between gap-2 rounded-md border border-[var(--border)] px-2 text-sm hover:bg-gray-50"
                  >
                    <span className={`truncate ${layout?.backgroundAsset ? 'font-medium' : 'text-gray-500'}`}>
                      {layout?.backgroundAsset ?? '배경서식 선택'}
                    </span>
                    <span className="text-gray-500">▾</span>
                  </button>

                  {assetMenuOpen && (
                    <>
                      {/* 바깥 클릭 닫기 */}
                      <div className="fixed inset-0 z-30" onClick={() => setAssetMenuOpen(false)} />
                      <ul className="absolute left-0 top-full z-40 mt-1 max-h-60 w-[280px] overflow-y-auto rounded-md border border-[var(--border)] bg-white shadow-lg">
                        <li>
                          <button
                            type="button"
                            onClick={() => {
                              if (layout) updateLayout({ ...layout, backgroundAsset: null })
                              setAssetMenuOpen(false)
                            }}
                            className="w-full px-3 py-1.5 text-left text-sm text-gray-500 hover:bg-gray-50"
                          >
                            선택 안 함
                          </button>
                        </li>
                        {assets.map((a) => {
                          const selected = layout?.backgroundAsset === a.name
                          return (
                            <li
                              key={a.name}
                              className={`flex items-center gap-2 border-t border-[var(--border)] px-3 py-1.5 ${selected ? 'bg-blue-50' : 'hover:bg-gray-50'}`}
                              onMouseEnter={(e) => {
                                setMousePos({ x: e.clientX, y: e.clientY })
                                void showAssetPreview(a.name)
                              }}
                              onMouseMove={(e) => setMousePos({ x: e.clientX, y: e.clientY })}
                              onMouseLeave={() => setHoverPreview(null)}
                            >
                              <button
                                type="button"
                                onClick={() => {
                                  if (layout) updateLayout({ ...layout, backgroundAsset: a.name })
                                  setAssetMenuOpen(false)
                                }}
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
                    </>
                  )}
                </div>
              )}

              {/* 업로드 */}
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

            </div>

            {/* 마우스 포인터 우측 하단 추종 미리보기 (1.5배) */}
            {hoverPreview && (
              <div
                className="pointer-events-none fixed z-50 rounded-md border border-[var(--border)] bg-white p-1 shadow-lg"
                style={{
                  left:
                    typeof window !== 'undefined'
                      ? Math.min(mousePos.x + 14, window.innerWidth - 350)
                      : mousePos.x + 14,
                  top:
                    typeof window !== 'undefined'
                      ? Math.min(mousePos.y + 14, window.innerHeight - 340)
                      : mousePos.y + 14,
                }}
              >
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img src={hoverPreview.url} alt={`${hoverPreview.name} 미리보기`} className="max-h-72 max-w-[330px] object-contain" />
                <p className="mt-1 max-w-[330px] truncate text-center text-xs text-gray-600">{hoverPreview.name}</p>
              </div>
            )}

            {/* 표시 필드 체크박스(좌) + 미리보기 캔버스 + 저장 패널(우) */}
            <div className="flex min-h-0 flex-1 gap-2">
              {/* 좌측: 표시 필드 토글 + 선택 박스 폰트 컨트롤 */}
              <div className="flex w-44 shrink-0 flex-col gap-2 pt-1">
                {/* 선택된 텍스트박스 폰트 컨트롤 (위) — 캔버스에서 박스 클릭 시 대상 변경 */}
                {layout && layout.textboxes[selectedBoxIdx] && (
                  <div className={`flex flex-col gap-2 text-sm ${selDisabled ? 'opacity-50' : ''}`}>
                    <span className="text-xs text-gray-500">
                      편집: {boxLabel(layout.textboxes[selectedBoxIdx])}
                      {selDisabled && ' (체크 해제됨)'}
                    </span>
                    {layout.textboxes[selectedBoxIdx].fieldType === 'custom' && (
                      <input
                        type="text"
                        disabled={selDisabled}
                        value={layout.textboxes[selectedBoxIdx].text ?? ''}
                        onChange={(e) => updateBox(selectedBoxIdx, { text: e.target.value })}
                        placeholder="표시할 텍스트"
                        className="h-9 w-full rounded border border-[var(--border)] px-2 disabled:cursor-not-allowed"
                      />
                    )}
                    <label className="flex items-center gap-1">
                      Size
                      <input
                        type="range"
                        min={10}
                        max={100}
                        disabled={selDisabled}
                        value={Math.round(layout.textboxes[selectedBoxIdx].fontRatio * 100)}
                        onChange={(e) => updateBox(selectedBoxIdx, { fontRatio: Number(e.target.value) / 100 })}
                        className="w-[60%] disabled:cursor-not-allowed"
                      />
                      <span className="w-8 text-right text-xs">
                        {Math.round(layout.textboxes[selectedBoxIdx].fontRatio * 100)}%
                      </span>
                    </label>
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        title="굵게"
                        aria-label="굵게"
                        disabled={selDisabled}
                        onClick={() =>
                          updateBox(selectedBoxIdx, {
                            fontWeight: layout.textboxes[selectedBoxIdx].fontWeight === 'bold' ? 'normal' : 'bold',
                          })
                        }
                        className={`flex h-9 w-9 items-center justify-center rounded border text-xs disabled:cursor-not-allowed ${layout.textboxes[selectedBoxIdx].fontWeight === 'bold' ? 'border-[var(--accent)] bg-blue-50' : 'border-[var(--border)]'}`}
                      >
                        🅱️
                      </button>
                      <input
                        type="color"
                        disabled={selDisabled}
                        value={layout.textboxes[selectedBoxIdx].fontColor}
                        onChange={(e) => updateBox(selectedBoxIdx, { fontColor: e.target.value })}
                        className="h-9 w-9 cursor-pointer rounded border border-[var(--border)] disabled:cursor-not-allowed"
                        title="글자 색"
                        aria-label="글자 색"
                      />
                      {([
                        ['left', '왼쪽 정렬'],
                        ['center', '가운데 정렬'],
                        ['right', '오른쪽 정렬'],
                      ] as const).map(([al, label]) => (
                        <button
                          key={al}
                          type="button"
                          title={label}
                          aria-label={label}
                          disabled={selDisabled}
                          onClick={() => updateBox(selectedBoxIdx, { textAlign: al })}
                          className={`flex h-9 w-9 items-center justify-center rounded border disabled:cursor-not-allowed ${layout.textboxes[selectedBoxIdx].textAlign === al ? 'border-[var(--accent)] bg-blue-50 text-[var(--accent)]' : 'border-[var(--border)] text-gray-700'}`}
                        >
                          <AlignIcon align={al} />
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                {/* 표시 필드 체크박스 (아래) */}
                <div className="flex flex-col gap-2 border-t border-[var(--border)] pt-2">
                  {(layout?.textboxes ?? []).map((tb, i) => (
                    <div key={tb.id || tb.fieldType} className="flex items-center gap-1">
                      <label className="flex flex-1 cursor-pointer items-center gap-1 truncate text-sm text-gray-700">
                        <input
                          type="checkbox"
                          checked={tb.enabled !== false}
                          onChange={(e) => updateBox(i, { enabled: e.target.checked })}
                          className="h-4 w-4 shrink-0"
                        />
                        <span className="truncate" title={boxLabel(tb)}>{boxLabel(tb)}</span>
                      </label>
                      {tb.fieldType === 'custom' && (
                        <button
                          type="button"
                          onClick={() => removeTextbox(i)}
                          aria-label="텍스트박스 삭제"
                          className="rounded px-1 text-xs text-gray-400 hover:bg-red-50 hover:text-[var(--danger)]"
                        >
                          ✕
                        </button>
                      )}
                    </div>
                  ))}
                  <button
                    type="button"
                    onClick={addTextbox}
                    disabled={!layout}
                    className="mt-1 h-9 rounded-md border border-dashed border-[var(--accent)] text-sm text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
                  >
                    + 텍스트박스 추가
                  </button>
                </div>
              </div>

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
                      if (tb.enabled === false) return null // 체크 해제 항목은 미표시
                      const boxH = tb.hRatio * bgDims.h
                      return (
                        <Rnd
                          key={tb.id || tb.fieldType}
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
                            {noticeFieldText(tb, previewData)}
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

              {/* 우측: 저장 패널 (고정 너비) */}
              <div
                className="flex shrink-0 flex-col gap-2 overflow-y-auto rounded-md border border-[var(--border)] p-2"
                style={{ width: RIGHT_PANEL_WIDTH }}
              >
                <button
                  type="button"
                  onClick={handleSaveNotice}
                  disabled={!layout}
                  className="h-9 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50"
                >
                  공지문 저장
                </button>
                <button
                  type="button"
                  onClick={handleSaveAs}
                  disabled={!layout}
                  className="h-9 rounded-md border border-[var(--accent)] text-sm text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
                >
                  다른 이름으로 저장
                </button>

                <div className="mt-1 border-t border-[var(--border)] pt-2 text-xs text-gray-500">
                  저장된 템플릿
                </div>
                {templates.length === 0 ? (
                  <p className="text-xs text-gray-400">저장된 템플릿이 없습니다.</p>
                ) : (
                  <ul className="flex flex-col gap-1">
                    {templates.map((name) => (
                      <li key={name} className="flex items-center gap-1">
                        <button
                          type="button"
                          onClick={() => handleLoadTemplate(name)}
                          className="flex-1 truncate rounded px-2 py-1 text-left text-sm text-gray-800 hover:bg-gray-50"
                          title={`${name} 불러오기`}
                        >
                          {name}
                        </button>
                        <button
                          type="button"
                          onClick={() => handleDeleteTemplate(name)}
                          aria-label={`${name} 삭제`}
                          className="rounded px-1 text-xs text-gray-400 hover:bg-red-50 hover:text-[var(--danger)]"
                        >
                          ✕
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
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
