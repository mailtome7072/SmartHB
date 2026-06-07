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
import { useRouter } from 'next/navigation'
import { Rnd } from 'react-rnd'
import { useAppStore } from '@/stores/app-store'
import { useQuery } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SplashScreen } from '@/components/splash-screen'
import { ErrorDialog } from '@/components/ui/error-dialog'
import {
  checkNoticeOutputExists,
  deleteNoticeAsset,
  deleteNoticeLayoutNamed,
  getNoticeLayoutNamed,
  getNoticeMonthInfo,
  listBilledMonths,
  listBills,
  listNoticeAssets,
  listNoticeLayouts,
  noticePreviewDefaultPath,
  openNoticeOutputDir,
  openNoticePreviewDir,
  readNoticeAsset,
  saveNoticeAsset,
  saveNoticeLayout,
  saveNoticeLayoutNamed,
  saveNoticePreview,
  showSaveDialog,
} from '@/lib/tauri'
import {
  buildColorRuns,
  bytesToDataUrl,
  dataUrlToBytes,
  generateAndSaveNotices,
  noticeFieldText,
  renderNoticeDataUrl,
  type NoticeStudentData,
} from '@/lib/notice-generator'
import type { NoticeLayout, NoticeFieldType, TextboxConfig } from '@/types/notice'
import type { Bill } from '@/types/billing'

const FIELD_LABEL: Record<NoticeFieldType, string> = {
  bill_month: '청구월',
  teaching_period: '교습기간',
  makeup_day: '보강데이',
  student_name: '원생명',
  bill_amount: '청구액',
  custom: '텍스트',
}

/** 데이터 필드 체크박스 표시 순서 (청구월 아래 교습기간/보강데이). */
const DATA_FIELD_ORDER: NoticeFieldType[] = [
  'bill_month',
  'teaching_period',
  'makeup_day',
  'student_name',
  'bill_amount',
]

/** 체크박스/편집 라벨 — custom 은 입력 텍스트(없으면 '텍스트'). */
function boxLabel(tb: TextboxConfig): string {
  if (tb.fieldType === 'custom') return tb.text?.trim() || '텍스트'
  return FIELD_LABEL[tb.fieldType]
}

/** charColors 를 길이 len 의 배열로 정규화 (없는 인덱스는 null). */
function normalizeCharColors(charColors: (string | null)[] | null | undefined, len: number): (string | null)[] {
  const out = new Array<string | null>(len).fill(null)
  if (charColors) for (let i = 0; i < len && i < charColors.length; i++) out[i] = charColors[i] ?? null
  return out
}

/** 전부 미지정이면 null 로 압축 (저장 정리). */
function compactCharColors(colors: (string | null)[]): (string | null)[] | null {
  return colors.some((c) => c != null) ? colors : null
}

/**
 * 텍스트 편집 시 글자별 색을 재정렬한다 — 공통 접두/접미는 보존, 새로 입력된 중간 글자는 기본색(null).
 * (insert/delete 위치를 prefix/suffix 일치로 추정)
 */
function realignCharColors(
  oldText: string,
  oldColors: (string | null)[],
  newText: string,
): (string | null)[] {
  if (oldText === newText) return oldColors.slice(0, newText.length)
  let p = 0
  const maxP = Math.min(oldText.length, newText.length)
  while (p < maxP && oldText[p] === newText[p]) p++
  let s = 0
  const maxS = Math.min(oldText.length - p, newText.length - p)
  while (s < maxS && oldText[oldText.length - 1 - s] === newText[newText.length - 1 - s]) s++
  const result = new Array<string | null>(newText.length).fill(null)
  for (let i = 0; i < p; i++) result[i] = oldColors[i] ?? null
  for (let i = 0; i < s; i++) result[newText.length - 1 - i] = oldColors[oldText.length - 1 - i] ?? null
  return result
}

/** 빈/기본 레이아웃 — 초기화용. (백엔드 default_textboxes 와 동일 배치) */
function makeDefaultLayout(): NoticeLayout {
  const mk = (f: NoticeFieldType, y: number, enabled: boolean): TextboxConfig => ({
    id: f,
    fieldType: f,
    text: null,
    enabled,
    xRatio: 0.1,
    yRatio: y,
    wRatio: 0.8,
    hRatio: 0.12,
    fontRatio: 0.5,
    fontWeight: 'bold',
    fontColor: '#1A1A1A',
    textAlign: 'center',
  })
  return {
    backgroundAsset: null,
    textboxes: [
      mk('bill_month', 0.05, true),
      mk('teaching_period', 0.2, false),
      mk('makeup_day', 0.35, false),
      mk('student_name', 0.55, true),
      mk('bill_amount', 0.75, true),
    ],
  }
}

/** 구버전 레이아웃에 누락된 데이터 필드(교습기간/보강데이 등)를 비활성으로 보강. */
function normalizeLayout(l: NoticeLayout): NoticeLayout {
  const existing = new Set(l.textboxes.map((t) => t.fieldType))
  const missing = DATA_FIELD_ORDER.filter((f) => !existing.has(f))
  if (missing.length === 0) return l
  const added: TextboxConfig[] = missing.map((f, idx) => ({
    id: f,
    fieldType: f,
    text: null,
    enabled: false,
    xRatio: 0.1,
    yRatio: 0.2 + idx * 0.15,
    wRatio: 0.8,
    hRatio: 0.12,
    fontRatio: 0.5,
    fontWeight: 'bold',
    fontColor: '#1A1A1A',
    textAlign: 'center',
  }))
  return { ...l, textboxes: [...l.textboxes, ...added] }
}

/** 자주 쓰는 글자색 프리셋 — 클릭(값과 무관하게 항상 동작)으로 적용. 네이티브 피커의 동일색 무반응 회피. */
const COLOR_PRESETS: { hex: string; label: string }[] = [
  { hex: '#000000', label: '검정' },
  { hex: '#E03131', label: '빨강' },
  { hex: '#F08C00', label: '주황' },
  { hex: '#FFD43B', label: '노랑' },
  { hex: '#2F9E44', label: '초록' },
  { hex: '#1971C2', label: '파랑' },
  { hex: '#FFFFFF', label: '흰색' },
]

const STUDENT_PANEL_WIDTH = 240 // 원생 패널(청구년월 + 원생 리스트) 고정 너비
const TEMPLATE_PANEL_WIDTH = 220 // 저장 패널(공지문 이름 + 템플릿 목록) 고정 너비

function currentYearMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

/** 'YYYY-MM' → 'YYMM' (저장 폴더/파일명 형식과 일치, 예: 2026-06 → 2606). */
function yymm(yearMonth: string): string {
  const [y, m] = yearMonth.split('-')
  return y && m ? `${y.slice(-2)}${m}` : yearMonth
}

/** 폴더/파일명용 — 공백 제거 (백엔드 sanitize_path_part 표시 정합). */
function noSpace(s: string): string {
  return s.replace(/\s+/g, '')
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
  // 성공/안내 토스트 (오류 다이얼로그와 분리, 3초 자동 닫힘)
  const [toast, setToast] = useState<string | null>(null)
  useEffect(() => {
    if (!toast) return
    const id = setTimeout(() => setToast(null), 3000)
    return () => clearTimeout(id)
  }, [toast])
  // 확인 모달 (window.confirm 대체 — Tauri 웹뷰 호환)
  const [confirmDialog, setConfirmDialog] = useState<{ message: string; onConfirm: () => void } | null>(null)

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
  const [layout, setLayout] = useState<NoticeLayout | null>(null)
  useEffect(() => {
    // 프로그램 구동 후 페이지 진입 시 항상 초기화 상태(선택 공지문 없음, 빈 캔버스)로 시작.
    if (layout === null) {
      const base = makeDefaultLayout()
      setLayout({ ...base, textboxes: base.textboxes.map((tb) => ({ ...tb, enabled: false })) })
    }
  }, [layout])

  // 청구년월의 교습기간·보강데이 텍스트
  const monthInfoQuery = useQuery({
    queryKey: ['notice-month-info', yearMonth],
    queryFn: () => getNoticeMonthInfo(yearMonth),
  })
  const monthInfo = monthInfoQuery.data ?? { teachingPeriodText: null, makeupDayText: null }

  const [bgDataUrl, setBgDataUrl] = useState<string | null>(null)
  const [bgDims, setBgDims] = useState<{ w: number; h: number }>({ w: 800, h: 800 })
  const [selectedBoxIdx, setSelectedBoxIdx] = useState(0)
  // 다중 선택 인덱스 (Shift+클릭). primary = selectedBoxIdx(폰트 컨트롤 대상).
  const [selectedBoxIdxs, setSelectedBoxIdxs] = useState<Set<number>>(() => new Set())
  // 텍스트박스 인라인 편집 대상 id (더블클릭 시 진입). custom 은 텍스트 편집, 데이터 필드는 색칠만.
  const [editingId, setEditingId] = useState<string | null>(null)
  // 편집 중 textarea — 글자별 색 적용 시 선택 범위 참조용.
  const editTextareaRef = useRef<HTMLTextAreaElement>(null)
  // 마지막 선택 범위 — 색 버튼 클릭 시점엔 textarea 포커스가 빠져 selectionStart/End 가
  // 접힐 수 있으므로(WKWebView), 선택 순간(onSelect)에 범위를 저장해 둔다.
  const selRangeRef = useRef<{ start: number; end: number } | null>(null)

  // 단일 선택으로 리셋 (추가/삭제/템플릿 로드 시).
  const setSelectionSingle = useCallback((i: number) => {
    setSelectedBoxIdx(i)
    setSelectedBoxIdxs(new Set([i]))
  }, [])

  // 모든 선택 해제 + 편집 종료 (캔버스 빈 영역 클릭 시).
  const clearSelection = useCallback(() => {
    setSelectedBoxIdxs(new Set())
    setEditingId(null)
  }, [])

  // 박스 선택 — additive(Shift) 면 토글 누적, 아니면 단일 선택.
  // primary(selectedBoxIdx)는 마지막으로 클릭한 박스로 갱신.
  const selectBox = useCallback((i: number, additive: boolean) => {
    if (!additive) {
      setSelectedBoxIdx(i)
      setSelectedBoxIdxs(new Set([i]))
      return
    }
    setSelectedBoxIdxs((prev) => {
      const next = new Set(prev)
      if (next.has(i)) {
        next.delete(i)
        if (next.size === 0) next.add(i) // 최소 1개 유지
      } else {
        next.add(i)
      }
      return next
    })
    setSelectedBoxIdx(i)
  }, [])

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

  // 색상 선택 적용 — 편집 중 + 선택 영역(저장된 범위) 있으면 선택 글자만, 아니면 박스 기본색 변경.
  const applyColor = (color: string) => {
    const idx = selectedBoxIdx
    const tb = layout?.textboxes[idx]
    if (!tb) return
    const id = tb.id || tb.fieldType
    // 실시간 선택 우선, 없으면(포커스 이탈로 접힘) 마지막 저장 범위 사용.
    const ta = editTextareaRef.current
    const live =
      ta && ta.selectionStart !== ta.selectionEnd
        ? { start: ta.selectionStart, end: ta.selectionEnd }
        : null
    const sel = live ?? selRangeRef.current
    if (editingId === id && sel && sel.start !== sel.end) {
      const text = noticeFieldText(tb, previewData) // custom=tb.text, 데이터 필드=미리보기값
      const colors = normalizeCharColors(tb.charColors, text.length)
      for (let i = sel.start; i < sel.end && i < colors.length; i++) colors[i] = color
      updateBox(idx, { charColors: compactCharColors(colors) })
    } else {
      updateBox(idx, { fontColor: color })
    }
  }

  // 더블클릭으로 편집 진입 (custom: 텍스트+색, 데이터 필드: 색칠만).
  const enterEdit = (idx: number, tb: TextboxConfig) => {
    selRangeRef.current = null // 새 편집 박스 — 이전 선택 범위 폐기
    setSelectionSingle(idx)
    setEditingId(tb.id || tb.fieldType)
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
    setSelectionSingle(layout.textboxes.length) // 새 박스 선택
  }

  // custom 텍스트박스 삭제 (데이터 필드 3종은 삭제 불가).
  const removeTextbox = (idx: number) => {
    if (!layout) return
    updateLayout({ ...layout, textboxes: layout.textboxes.filter((_, i) => i !== idx) })
    setSelectionSingle(0)
  }

  // 체크박스 행 (데이터 필드/추가 박스 공통)
  const renderBoxRow = (tb: TextboxConfig, i: number) => (
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
          className="rounded px-1 text-xs text-gray-600 hover:bg-red-50 hover:text-[var(--danger)]"
        >
          ✕
        </button>
      )}
    </div>
  )

  // 선택된 텍스트박스가 체크 해제(비활성)면 폰트 컨트롤 비활성.
  const selDisabled =
    !layout?.textboxes[selectedBoxIdx] || layout.textboxes[selectedBoxIdx].enabled === false

  // 미리보기 원생 데이터
  const previewBill: Bill | undefined = bills.find((b) => selectedIds.has(b.id)) ?? bills[0]
  const previewData: NoticeStudentData = {
    studentName: previewBill?.studentName ?? '원생 이름',
    billYearMonth: yearMonth,
    billAmount: previewBill?.adjustedAmount ?? 0,
    teachingPeriodText: monthInfo.teachingPeriodText,
    makeupDayText: monthInfo.makeupDayText,
  }

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
  // 공지문 미리보기 팝업 — 렌더된 PNG data URL(열림 = non-null) + 렌더 중 표시.
  const [previewUrl, setPreviewUrl] = useState<string | null>(null)
  const [previewBusy, setPreviewBusy] = useState(false)
  const handleGenerate = async () => {
    if (!layout?.backgroundAsset || !bgDataUrl) {
      setError('배경서식을 먼저 선택해 주세요.')
      return
    }
    const noticeName = templateName.trim()
    if (noticeName === '') {
      setError('공지문 이름을 먼저 입력해 주세요. (저장 폴더·파일명에 사용됩니다)')
      return
    }
    const targets = bills.filter((b) => selectedIds.has(b.id))
    if (targets.length === 0) {
      setError('공지문을 생성할 원생을 선택해 주세요.')
      return
    }
    const exists = await checkNoticeOutputExists(noticeName, yearMonth)
    if (exists) {
      setConfirmDialog({
        message: `output/${noSpace(noticeName)}/${yymm(yearMonth)}/ 폴더에 기존 공지문이 있습니다. 덮어쓰시겠습니까?`,
        onConfirm: () => void runGenerate(noticeName, targets),
      })
      return
    }
    void runGenerate(noticeName, targets)
  }
  const runGenerate = async (noticeName: string, targets: Bill[]) => {
    if (!layout || !bgDataUrl) return
    setGenerating(true)
    setProgress({ done: 0, total: targets.length })
    try {
      const result = await generateAndSaveNotices({
        noticeName,
        yearMonth,
        backgroundDataUrl: bgDataUrl,
        width: bgDims.w,
        height: bgDims.h,
        layout,
        students: targets.map((b) => ({
          studentName: b.studentName,
          billYearMonth: yearMonth,
          billAmount: b.adjustedAmount,
          teachingPeriodText: monthInfo.teachingPeriodText,
          makeupDayText: monthInfo.makeupDayText,
        })),
        onProgress: (done, total) => setProgress({ done, total }),
      })
      setToast(`✅ ${result.saved}건 생성 완료. 저장 위치: output/${noSpace(noticeName)}/${yymm(yearMonth)}/`)
    } catch (e) {
      setError(e instanceof Error ? e.message : '공지문 생성 실패')
    } finally {
      setGenerating(false)
      setProgress(null)
    }
  }

  // 미리보기 — 현재 로드된 공지문을 렌더하여 팝업으로 표시 (저장 없음).
  const handlePreview = async () => {
    if (!layout || !bgDataUrl) return
    try {
      setPreviewBusy(true)
      const url = await renderNoticeDataUrl(
        { backgroundDataUrl: bgDataUrl, width: bgDims.w, height: bgDims.h, layout },
        previewData,
      )
      setPreviewUrl(url)
    } catch (e) {
      setError(e instanceof Error ? e.message : '미리보기 생성 실패')
    } finally {
      setPreviewBusy(false)
    }
  }
  // 미리보기 저장 — 파일 저장 다이얼로그(기본: output/공지문/{공지문이름}.png).
  const handleSavePreview = async () => {
    if (!previewUrl) return
    const name = templateName.trim() || '공지문'
    try {
      const defaultPath = await noticePreviewDefaultPath(name)
      const chosen = await showSaveDialog(defaultPath)
      if (!chosen) return // 취소
      const saved = await saveNoticePreview(chosen, dataUrlToBytes(previewUrl))
      setPreviewUrl(null)
      setToast(`✅ 미리보기 저장 완료: ${saved}`)
    } catch (e) {
      setError(e instanceof Error ? e.message : '미리보기 저장 실패')
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

  // ── 방향키 미세 이동 (선택된 박스 일괄) ──
  // 수식키 없이 방향키만으로 이동 (macOS Ctrl+방향키는 OS 데스크톱 전환과 충돌).
  // Shift+방향키는 10px 단위 이동. 한 번 누를 때마다 화면 기준 px → 원본 좌표 환산, 0..1 클램프.
  useEffect(() => {
    if (typeof window === 'undefined') return
    const onKeyDown = (e: KeyboardEvent) => {
      const arrows = ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown']
      if (!arrows.includes(e.key)) return
      if (e.ctrlKey || e.metaKey || e.altKey) return // 수식키 조합은 무시 (OS/브라우저 단축키)
      if (editingId !== null) return // 인라인 텍스트 편집 중엔 무시
      // 폼 입력(셀렉트/인풋/텍스트영역)에 포커스가 있으면 그쪽 동작 우선.
      const el = document.activeElement
      const tag = el?.tagName
      if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA' || (el as HTMLElement | null)?.isContentEditable) return
      if (!layout || !bgDataUrl || selectedBoxIdxs.size === 0) return
      e.preventDefault()
      const stepScreen = e.shiftKey ? 10 : 1 // 화면 px (Shift = 빠른 이동)
      const stepOrig = stepScreen / scale // 화면 px 에 해당하는 배경 원본 px
      const dxr = (e.key === 'ArrowLeft' ? -stepOrig : e.key === 'ArrowRight' ? stepOrig : 0) / bgDims.w
      const dyr = (e.key === 'ArrowUp' ? -stepOrig : e.key === 'ArrowDown' ? stepOrig : 0) / bgDims.h
      updateLayout({
        ...layout,
        textboxes: layout.textboxes.map((tb, i) => {
          if (!selectedBoxIdxs.has(i) || tb.enabled === false) return tb
          return {
            ...tb,
            xRatio: Math.min(Math.max(tb.xRatio + dxr, 0), 1 - tb.wRatio),
            yRatio: Math.min(Math.max(tb.yRatio + dyr, 0), 1 - tb.hRatio),
          }
        }),
      })
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [layout, bgDataUrl, selectedBoxIdxs, editingId, scale, bgDims, updateLayout])

  // ── 저장 템플릿 ──
  const templatesQuery = useQuery({ queryKey: ['notice-layouts'], queryFn: listNoticeLayouts })
  const templates = useMemo(() => templatesQuery.data ?? [], [templatesQuery.data])
  // 목록 표시는 이름 내림차순.
  const sortedTemplates = useMemo(() => [...templates].sort((a, b) => b.localeCompare(a)), [templates])

  // 작성/저장할 템플릿 이름. 디폴트 없음 — 빈 상태로 시작, 사용자가 직접 입력.
  const [templateName, setTemplateName] = useState('')
  // 현재 불러온/저장한 명명 템플릿의 레이아웃 스냅샷(JSON). 미저장 변경 감지 기준.
  // null = 불러온 템플릿 없음(새 작업) → 저장 확인 질의 대상 아님.
  const savedSnapshotRef = useRef<string | null>(null)
  // 미저장 변경 상태에서 다른 템플릿 불러오기/닫기 요청 시 대기 중인 동작.
  // 저장 확인 모달에서 네(저장 후 실행)/아니오(저장 없이 실행)/취소.
  const [pendingAction, setPendingAction] = useState<
    { kind: 'load'; name: string } | { kind: 'close' } | { kind: 'navigate'; href: string } | null
  >(null)
  // 현재 레이아웃이 마지막 저장/불러온 명명 템플릿과 달라졌는지(미저장 변경).
  const isTemplateDirty = useCallback(
    () =>
      templateName.trim() !== '' &&
      savedSnapshotRef.current !== null &&
      !!layout &&
      JSON.stringify(layout) !== savedSnapshotRef.current,
    [templateName, layout],
  )

  // 편집 중 다른 메뉴로 이동 시 미저장 확인 — 전역 네비게이션 가드 등록.
  const router = useRouter()
  const setUnsavedGuard = useAppStore((s) => s.setUnsavedGuard)
  useEffect(() => {
    const guard = (href: string) => {
      if (href === '/notices') return true // 같은 페이지 이동은 통과
      if (isTemplateDirty()) {
        setPendingAction({ kind: 'navigate', href })
        return false // 차단 — 확인 다이얼로그에서 결정
      }
      return true
    }
    setUnsavedGuard(guard)
    return () => setUnsavedGuard(null)
  }, [isTemplateDirty, setUnsavedGuard])

  // 배경서식이 로드되지 않은(선택되지 않은) 시점에 공지문 이름을 한 번 비운다.
  // - 의존성에 backgroundAsset 만 두어, 사용자가 타이핑 중인 값이 매 키 입력마다 지워지는
  //   상황을 피한다. (입력은 자유롭게 가능하되 저장 버튼이 별도로 비활성화됨)
  useEffect(() => {
    if (!layout?.backgroundAsset) {
      setTemplateName('')
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [layout?.backgroundAsset])

  // 배경서식이 로드되지 않은 시점에 데이터 필드 체크박스(청구월/원생명/청구액 등)를 모두 unchecked 로
  // 강제한다. 사용자가 체크 상태를 토글하는 동안 매번 되돌려지지 않도록 backgroundAsset 변경에만
  // 반응 — 배경을 다시 선택하면 그 시점부터는 자유롭게 체크 가능.
  useEffect(() => {
    if (layout?.backgroundAsset) return
    if (!layout) return
    if (layout.textboxes.every((tb) => tb.enabled === false)) return
    updateLayout({
      ...layout,
      textboxes: layout.textboxes.map((tb) => ({ ...tb, enabled: false })),
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [layout?.backgroundAsset])

  // 실제 저장 — 이름 확정 후 호출.
  // 성공 시 true, 실패 시 false 반환 — "저장 후 이동/닫기" 가 실패를 인지하고 중단할 수 있게 한다.
  const doSaveTemplate = async (name: string): Promise<boolean> => {
    if (!layout) return false
    try {
      await saveNoticeLayoutNamed(name, layout)
      await templatesQuery.refetch()
      setTemplateName(name)
      savedSnapshotRef.current = JSON.stringify(layout) // 저장 직후 = 미저장 변경 없음
      setToast(`✅ '${name}' 템플릿으로 저장되었습니다.`)
      return true
    } catch (e) {
      setError(e instanceof Error ? e.message : '저장 실패')
      return false
    }
  }
  // 공지문 저장 — 동명 덮어쓰기 확인 없이 바로 저장.
  const handleSaveNotice = () => {
    if (!layout) return
    const n = templateName.trim()
    if (n === '') {
      setError('공지문 이름을 입력해 주세요.')
      return
    }
    void doSaveTemplate(n)
  }
  const doLoadTemplate = async (name: string) => {
    try {
      const loaded = await getNoticeLayoutNamed(name)
      const normalized = normalizeLayout(loaded)
      updateLayout(normalized)
      savedSnapshotRef.current = JSON.stringify(normalized) // 불러온 직후 = 미저장 변경 없음
      setSelectionSingle(0)
      setEditingId(null)
      setTemplateName(name)
      setToast(`'${name}' 템플릿을 불러왔습니다.`)
    } catch (e) {
      setError(e instanceof Error ? e.message : '템플릿 불러오기 실패')
    }
  }
  // 공지문 닫기 — 캔버스 비우고 아무 공지문도 선택되지 않은 상태로.
  const doCloseNotice = () => {
    const base = makeDefaultLayout()
    updateLayout({ ...base, textboxes: base.textboxes.map((tb) => ({ ...tb, enabled: false })) })
    clearSelection()
    setTemplateName('')
    savedSnapshotRef.current = null
  }
  // 대기 동작 실행 (저장 확인 모달 이후).
  const runPendingAction = (
    action: { kind: 'load'; name: string } | { kind: 'close' } | { kind: 'navigate'; href: string },
  ) => {
    if (action.kind === 'load') void doLoadTemplate(action.name)
    else if (action.kind === 'navigate') router.push(action.href)
    else doCloseNotice()
  }
  // 다른 공지문 불러오기 — 작업 중인 공지문에 미저장 변경이 있으면 저장 여부 질의.
  const handleLoadTemplate = (name: string) => {
    if (name === templateName.trim()) {
      void doLoadTemplate(name) // 같은 템플릿 다시 불러오기는 질의 없이 새로고침
      return
    }
    if (isTemplateDirty()) {
      setPendingAction({ kind: 'load', name })
      return
    }
    void doLoadTemplate(name)
  }
  // 닫기 — 미저장 변경이 있으면 저장 여부 질의 후 비우기.
  const handleCloseNotice = () => {
    if (isTemplateDirty()) {
      setPendingAction({ kind: 'close' })
      return
    }
    doCloseNotice()
  }
  const handleDeleteTemplate = async (name: string) => {
    try {
      await deleteNoticeLayoutNamed(name)
      await templatesQuery.refetch()
      // 현재 편집 중(불러온) 템플릿이 삭제되면 초기화: 체크박스 모두 해제 + 편집 박스 + 이름 비움.
      if (templateName.trim() === name) {
        const base = makeDefaultLayout()
        updateLayout({ ...base, textboxes: base.textboxes.map((tb) => ({ ...tb, enabled: false })) })
        setSelectionSingle(0)
        setEditingId(null)
        setTemplateName('') // 비운 채 유지
        savedSnapshotRef.current = null // 불러온 템플릿 없음
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '템플릿 삭제 실패')
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="flex h-full flex-col">
        <h1 className="mb-3 text-2xl font-bold">공지문 생성</h1>

        <div className="flex min-h-0 flex-1 gap-3">
          {/* 우측: 원생 리스트 (고정 너비) — 청구년월 + 원생 체크리스트 */}
          <section
            className="order-3 flex flex-col overflow-hidden rounded-md border border-[var(--border)] p-3"
            style={{ width: STUDENT_PANEL_WIDTH, flex: '0 0 auto' }}
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

            {/* 생성 — 청구년월·원생 목록 하단 */}
            <div className="mt-2 flex flex-col gap-1 border-t border-[var(--border)] pt-2">
              <button
                type="button"
                onClick={handleGenerate}
                disabled={generating || !layout?.backgroundAsset || selectedIds.size === 0}
                className="h-11 w-full rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] px-3 text-base font-semibold text-white hover:opacity-90 disabled:opacity-50"
              >
                {generating ? `생성 중... ${progress ? `(${progress.done}/${progress.total})` : ''}` : `공지문 생성 (${selectedIds.size}명)`}
              </button>
              <button
                type="button"
                onClick={() => {
                  const name = templateName.trim()
                  if (name === '') {
                    setError('공지문 이름을 먼저 입력해 주세요.')
                    return
                  }
                  void openNoticeOutputDir(name, yearMonth).catch((e) =>
                    setError(e instanceof Error ? e.message : '폴더 열기 실패'),
                  )
                }}
                title="저장 폴더 열기 (없으면 생성)"
                className="break-all text-left text-xs text-gray-500 underline-offset-2 hover:text-[var(--accent)] hover:underline"
              >
                📂 저장 위치: output/{noSpace(templateName.trim()) || '{공지문이름}'}/{yymm(yearMonth)}/
              </button>
            </div>
          </section>

          {/* 중앙: 편집 캔버스 */}
          <section className="order-2 flex min-w-0 flex-1 flex-col overflow-hidden rounded-md border border-[var(--border)] p-3">
            {/* 배경서식 관리 — 한 줄: 콤보박스 + 업로드 */}
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
                                className="rounded px-1.5 text-sm text-gray-600 hover:bg-red-50 hover:text-[var(--danger)]"
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
                {layout && selectedBoxIdxs.size > 0 && layout.textboxes[selectedBoxIdx] && (
                  <div className={`flex flex-col gap-2 text-sm ${selDisabled ? 'opacity-50' : ''}`}>
                    <span className="text-xs text-gray-500">
                      편집: {boxLabel(layout.textboxes[selectedBoxIdx])}
                      {selDisabled && ' (체크 해제됨)'}
                    </span>
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
                        onChange={(e) => applyColor(e.target.value)}
                        className="h-9 w-9 cursor-pointer rounded border border-[var(--border)] disabled:cursor-not-allowed"
                        title="기타 색 직접 선택 (아래 프리셋으로 자주 쓰는 색 빠르게 적용)"
                        aria-label="기타 글자 색"
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
                    {/* 색 프리셋 — 편집 중 글자 선택 시 선택 부분만, 아니면 박스 전체 기본색 */}
                    {/* flex-1 로 한 줄에 균등 분배 — 프리셋 개수와 무관하게 항상 한 라인 */}
                    <div className="flex items-center gap-1">
                      {COLOR_PRESETS.map(({ hex, label }) => (
                        <button
                          key={hex}
                          type="button"
                          title={`${label} (${hex})`}
                          aria-label={`글자색 ${label}`}
                          disabled={selDisabled}
                          onClick={() => applyColor(hex)}
                          style={{ backgroundColor: hex }}
                          className="aspect-square min-w-0 flex-1 rounded border border-[var(--border)] disabled:cursor-not-allowed disabled:opacity-50"
                        />
                      ))}
                    </div>
                  </div>
                )}

                {/* 표시 필드 체크박스 (아래) */}
                <div className="flex flex-col gap-2 border-t border-[var(--border)] pt-2">
                  {/* 데이터 필드 (버튼 위) — 고정 순서: 청구월/교습기간/보강데이/원생명/청구액 */}
                  {DATA_FIELD_ORDER.map((ft) => {
                    const i = (layout?.textboxes ?? []).findIndex((t) => t.fieldType === ft)
                    return i >= 0 ? renderBoxRow(layout!.textboxes[i], i) : null
                  })}
                  <button
                    type="button"
                    onClick={addTextbox}
                    disabled={!layout}
                    className="mt-1 h-9 rounded-md border border-dashed border-[var(--accent)] text-sm text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
                  >
                    + 텍스트박스 추가
                  </button>
                  {/* 추가된 텍스트박스 (버튼 아래) */}
                  {(layout?.textboxes ?? []).map((tb, i) =>
                    tb.fieldType === 'custom' ? renderBoxRow(tb, i) : null,
                  )}
                </div>

                {/* 공지문 미리보기 — 공지문 로드 + 캔버스 컨트롤(활성 텍스트박스) 1개 이상일 때 활성 */}
                <div className="mt-auto flex shrink-0 flex-col gap-1">
                  <button
                    type="button"
                    onClick={handlePreview}
                    disabled={
                      previewBusy ||
                      !layout?.backgroundAsset ||
                      !bgDataUrl ||
                      !(layout?.textboxes.some((tb) => tb.enabled !== false) ?? false)
                    }
                    className="h-10 w-full rounded-md border-2 border-[var(--accent)] text-sm font-semibold text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
                  >
                    {previewBusy ? '미리보기 생성 중...' : '공지문 미리보기'}
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      void openNoticePreviewDir().catch((e) =>
                        setError(e instanceof Error ? e.message : '폴더 열기 실패'),
                      )
                    }
                    title="저장 폴더 열기 (없으면 생성)"
                    className="break-all text-left text-xs text-gray-500 underline-offset-2 hover:text-[var(--accent)] hover:underline"
                  >
                    📂 저장 위치: output/공지문/{noSpace(templateName.trim()) || '{공지문이름}'}.png
                  </button>
                </div>
              </div>

              {/* 미리보기 캔버스 (가용 영역 채움) */}
              <div
                ref={previewWrapRef}
                onMouseDown={clearSelection} // 빈 영역 클릭 → 선택 해제 (박스는 stopPropagation 으로 제외)
                className="flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-gray-100"
              >
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
                      const id = tb.id || tb.fieldType
                      const isEditing = editingId === id
                      const isCustom = tb.fieldType === 'custom'
                      // 표시 텍스트: custom 은 입력값, 데이터 필드는 미리보기 원생값.
                      const boxText = noticeFieldText(tb, previewData)
                      const runs = buildColorRuns(boxText, tb.charColors, tb.fontColor)
                      const justify =
                        tb.textAlign === 'center' ? 'center' : tb.textAlign === 'right' ? 'flex-end' : 'flex-start'
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
                          onMouseDown={(e) => {
                            e.stopPropagation() // 빈 영역 클릭 핸들러(선택 해제)로 버블링 방지
                            if (editingId && editingId !== id) {
                              selRangeRef.current = null
                              setEditingId(null) // 다른 박스 클릭 시 편집 종료
                            }
                            selectBox(i, e.shiftKey)
                          }}
                          disableDragging={isEditing}
                          style={{
                            // 선택됨: accent. primary(폰트 컨트롤 대상)는 실선, 그 외 다중 선택은 점선.
                            outline: selectedBoxIdxs.has(i)
                              ? i === selectedBoxIdx
                                ? '2px solid var(--accent)'
                                : '2px dashed var(--accent)'
                              : '1px dashed #999',
                          }}
                        >
                          {isEditing ? (
                            // 인라인 편집 — 색 오버레이(뒤) + 투명 글자 textarea(앞).
                            // textarea 가 입력/선택을 담당하고, 오버레이가 글자별 색을 보여준다.
                            <div style={{ position: 'relative', width: '100%', height: '100%', background: 'rgba(255,255,255,0.7)' }}>
                              <div
                                aria-hidden
                                style={{
                                  // textarea 와 동일한 박스/폰트/상단 정렬 — 선택 하이라이트와 글자 위치 일치.
                                  position: 'absolute',
                                  inset: 0,
                                  fontSize: tb.fontRatio * boxH,
                                  fontWeight: tb.fontWeight,
                                  fontFamily: 'inherit',
                                  textAlign: tb.textAlign,
                                  lineHeight: 1.2,
                                  whiteSpace: 'pre-wrap',
                                  wordBreak: 'break-word',
                                  overflow: 'hidden',
                                  boxSizing: 'border-box',
                                  padding: 0,
                                  pointerEvents: 'none',
                                }}
                              >
                                {runs.map((r, k) => (
                                  <span key={k} style={{ color: r.color }}>{r.text}</span>
                                ))}
                              </div>
                              <textarea
                                ref={editTextareaRef}
                                autoFocus
                                readOnly={!isCustom}
                                value={boxText}
                                onChange={
                                  isCustom
                                    ? (e) => {
                                        const oldText = tb.text ?? ''
                                        const oldColors = normalizeCharColors(tb.charColors, oldText.length)
                                        const next = realignCharColors(oldText, oldColors, e.target.value)
                                        updateBox(i, { text: e.target.value, charColors: compactCharColors(next) })
                                      }
                                    : undefined
                                }
                                onSelect={(e) => {
                                  // 비어있지 않은 선택만 저장(접힘 시 직전 범위 유지) — 색 버튼 클릭 시
                                  // 포커스 이탈로 선택이 접혀도 마지막 실제 선택에 색을 적용한다.
                                  const t = e.currentTarget
                                  if (t.selectionStart !== t.selectionEnd) {
                                    selRangeRef.current = { start: t.selectionStart, end: t.selectionEnd }
                                  }
                                }}
                                onKeyDown={(e) => {
                                  if (e.key === 'Escape') {
                                    e.preventDefault()
                                    selRangeRef.current = null
                                    setEditingId(null)
                                  }
                                }}
                                style={{
                                  position: 'absolute',
                                  inset: 0,
                                  width: '100%',
                                  height: '100%',
                                  resize: 'none',
                                  border: 'none',
                                  outline: 'none',
                                  background: 'transparent',
                                  color: 'transparent',
                                  caretColor: tb.fontColor,
                                  fontSize: tb.fontRatio * boxH,
                                  fontWeight: tb.fontWeight,
                                  fontFamily: 'inherit',
                                  textAlign: tb.textAlign,
                                  lineHeight: 1.2,
                                  whiteSpace: 'pre-wrap',
                                  wordBreak: 'break-word',
                                  boxSizing: 'border-box',
                                  padding: 0,
                                  cursor: isCustom ? 'text' : 'default',
                                }}
                              />
                            </div>
                          ) : (
                            <div
                              onDoubleClick={() => enterEdit(i, tb)}
                              title={isCustom ? '더블클릭: 텍스트·색 편집' : '더블클릭: 글자별 색 편집'}
                              style={{
                                width: '100%',
                                height: '100%',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: justify,
                                fontSize: tb.fontRatio * boxH,
                                fontWeight: tb.fontWeight,
                                textAlign: tb.textAlign,
                                lineHeight: 1.2,
                                whiteSpace: 'pre-wrap',
                                wordBreak: 'break-word',
                                overflow: 'hidden',
                                cursor: 'move',
                              }}
                            >
                              {runs.map((r, k) => (
                                <span key={k} style={{ color: r.color }}>{r.text}</span>
                              ))}
                            </div>
                          )}
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
            </div>
          </section>

          {/* 좌측: 저장 패널 (고정 너비) — 공지문 이름 + 저장 + 템플릿 목록 */}
          <section
            className="order-1 flex shrink-0 flex-col gap-2 overflow-y-auto rounded-md border border-[var(--border)] p-3"
            style={{ width: TEMPLATE_PANEL_WIDTH }}
          >
            <label className="text-xs text-gray-500">공지문 이름</label>
            <input
              type="text"
              value={templateName}
              onChange={(e) => setTemplateName(e.target.value)}
              placeholder="공지문 이름"
              className="h-9 rounded-md border border-[var(--border)] px-2 text-sm"
            />
            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleSaveNotice}
                disabled={!layout?.backgroundAsset || templateName.trim() === ''}
                className="h-9 flex-1 rounded-md border-2 border-[var(--accent)] bg-[var(--accent)] text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50"
              >
                공지문 저장
              </button>
              <button
                type="button"
                onClick={handleCloseNotice}
                disabled={!layout?.backgroundAsset && templateName.trim() === ''}
                title="현재 공지문 캔버스를 비웁니다"
                className="h-9 shrink-0 rounded-md border-2 border-[var(--border)] px-3 text-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50"
              >
                닫기
              </button>
            </div>

            <div className="mt-1 border-t border-[var(--border)] pt-2 text-xs text-gray-500">
              저장된 템플릿
            </div>
            {templates.length === 0 ? (
              <p className="text-xs text-gray-600">저장된 템플릿이 없습니다.</p>
            ) : (
              <ul className="flex flex-col gap-1">
                {sortedTemplates.map((name) => (
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
                      className="rounded px-1 text-xs text-gray-600 hover:bg-red-50 hover:text-[var(--danger)]"
                    >
                      ✕
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      </div>

      <ErrorDialog open={error !== null && error !== ''} message={error ?? ''} onClose={() => setError(null)} />

      {/* 성공/안내 토스트 */}
      {toast && (
        <div className="pointer-events-none fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-md bg-gray-900/90 px-4 py-2 text-sm text-white shadow-lg">
          {toast}
        </div>
      )}

      {/* 확인 모달 (덮어쓰기 등) */}
      {confirmDialog && (
        <div role="dialog" aria-modal="true" className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-sm rounded-lg bg-white p-5 shadow-xl">
            <p className="mb-4 text-base text-gray-800">{confirmDialog.message}</p>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => setConfirmDialog(null)}
                className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50"
              >
                취소
              </button>
              <button
                type="button"
                onClick={() => {
                  const fn = confirmDialog.onConfirm
                  setConfirmDialog(null)
                  fn()
                }}
                className="min-h-[44px] flex-1 rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90"
              >
                저장
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 저장 확인 모달 — 미저장 변경 중 다른 공지문 불러오기/닫기 시 */}
      {pendingAction !== null && (
        <div role="dialog" aria-modal="true" className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-md rounded-lg bg-white p-5 shadow-xl">
            <p className="mb-4 text-base text-gray-800">
              작업 중인 &lsquo;{templateName}&rsquo; 공지문에 저장하지 않은 변경이 있습니다.
              <br />
              작업 내용을 저장하시겠습니까?
            </p>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => setPendingAction(null)}
                className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50"
              >
                취소
              </button>
              <button
                type="button"
                onClick={() => {
                  const action = pendingAction
                  setPendingAction(null)
                  runPendingAction(action)
                }}
                className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50"
              >
                아니오
              </button>
              <button
                type="button"
                onClick={() => {
                  const action = pendingAction
                  if (action === null) return
                  const saveName = templateName.trim()
                  void (async () => {
                    const ok = await doSaveTemplate(saveName)
                    if (!ok) return // 저장 실패 — 다이얼로그 유지, 이동/닫기 중단
                    setPendingAction(null)
                    runPendingAction(action)
                  })()
                }}
                className="min-h-[44px] flex-1 rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90"
              >
                네
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 공지문 미리보기 팝업 — 중앙 미리보기 + 하단 저장/닫기 */}
      {previewUrl && (
        <div role="dialog" aria-modal="true" className="fixed inset-0 z-[70] flex items-center justify-center bg-black/60 p-6">
          <div className="flex max-h-full max-w-full flex-col rounded-lg bg-white p-4 shadow-xl">
            <p className="mb-2 text-base font-semibold text-gray-800">공지문 미리보기</p>
            <div className="flex min-h-0 flex-1 items-center justify-center overflow-auto">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={previewUrl}
                alt="공지문 미리보기"
                className="max-h-[72vh] max-w-[80vw] object-contain shadow"
              />
            </div>
            <div className="mt-3 flex justify-end gap-2">
              <button
                type="button"
                onClick={handleSavePreview}
                className="min-h-[44px] rounded-md bg-[var(--accent)] px-5 text-base font-semibold text-white hover:opacity-90"
              >
                저장
              </button>
              <button
                type="button"
                onClick={() => setPreviewUrl(null)}
                className="min-h-[44px] rounded-md border-2 border-[var(--border)] px-5 text-base text-gray-700 hover:bg-gray-50"
              >
                닫기
              </button>
            </div>
          </div>
        </div>
      )}

    </AppShell>
  )
}
