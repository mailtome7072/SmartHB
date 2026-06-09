/**
 * 공지문 이미지 생성 엔진 — Sprint 12 T7 (PRD §4.10.2, AC-4.10-1).
 *
 * 배경서식(data URL)을 Canvas 2D 에 그리고 그 위에 레이아웃 텍스트박스를 직접 렌더링한다.
 * 원생별로 텍스트만 교체하여 순차 생성·저장한다. (macOS WKWebView 의 html-to-image
 * foreignObject+img 빈 이미지 결함 회피 — Canvas 직접 드로잉)
 * 청구액은 천단위 콤마(`Intl.NumberFormat('ko-KR')`)로 표기한다 (AC-4.10-1).
 */

import { saveNoticeImage } from '@/lib/tauri'
import type { NoticeLayout, NoticeFieldType, NoticeImageKind, TextboxConfig } from '@/types/notice'

/** 캔버스에 그릴 이미지 요소의 실제 dataUrl 맵 (종류별). 레이아웃 배치 + 이 맵으로 렌더. */
export type NoticeImageUrls = Partial<Record<NoticeImageKind, string | null>>

/** 사용자 추가 이미지 1개의 그리기 정보 — dataUrl + 비율 배치. */
export interface NoticeCustomImageDraw {
  dataUrl: string
  xRatio: number
  yRatio: number
  wRatio: number
  hRatio: number
}

/** 한 원생의 공지문 데이터 소스. */
export interface NoticeStudentData {
  studentName: string
  /** 청구년월 'YYYY-MM'. */
  billYearMonth: string
  /** 청구액(원). */
  billAmount: number
  /** 교습기간 표기 텍스트 (월 공통). */
  teachingPeriodText?: string | null
  /** 보강데이 표기 텍스트 (월 공통). */
  makeupDayText?: string | null
}

export interface GenerateOptions {
  /** 공지문(템플릿) 이름 — 저장 폴더/파일명에 사용. */
  noticeName: string
  yearMonth: string
  /** 배경서식 이미지 data URL. */
  backgroundDataUrl: string
  /** 렌더 캔버스 크기(px) — 배경 이미지 자연 크기 기준. */
  width: number
  height: number
  layout: NoticeLayout
  /** 이미지 요소(로고/2D바코드) 실제 dataUrl — 레이아웃 배치(layout.images)와 함께 그려진다. */
  imageUrls?: NoticeImageUrls
  /** 사용자 추가 이미지 — 배경 바로 위(다른 컨트롤 아래)에 그려진다. */
  customImages?: NoticeCustomImageDraw[]
  students: NoticeStudentData[]
  /** 진행률 콜백 (완료 건수, 총 건수). */
  onProgress?: (done: number, total: number) => void
}

const wonFormatter = new Intl.NumberFormat('ko-KR')

/** 'YYYY-MM' → 'M월' (청구월 컨트롤은 월만 표시). */
function formatBillMonth(yearMonth: string): string {
  const [y, m] = yearMonth.split('-')
  if (!y || !m) return yearMonth
  return `${Number(m)}월`
}

/** 텍스트박스의 표시 텍스트. custom 은 사용자 입력 text, 데이터 필드는 원생 데이터. */
export function noticeFieldText(
  tb: { fieldType: NoticeFieldType; text?: string | null },
  data: NoticeStudentData,
): string {
  switch (tb.fieldType) {
    case 'bill_month':
      return formatBillMonth(data.billYearMonth)
    case 'student_name':
      return data.studentName
    case 'bill_amount':
      return `${wonFormatter.format(data.billAmount)}원` // AC-4.10-1 천단위 콤마
    case 'teaching_period':
      return data.teachingPeriodText ?? ''
    case 'makeup_day':
      return data.makeupDayText ?? ''
    case 'custom':
      return tb.text ?? ''
    default:
      return ''
  }
}

/**
 * 글자별 색 정보를 연속 동일색 런으로 묶는다. 각 글자색은 `charColors[i] ?? defaultColor`.
 * 인덱스는 UTF-16 코드유닛 기준(textarea selectionStart/End 와 정합) — 한글/숫자/영문은 1:1.
 * 미리보기·생성·편집 오버레이가 공통 사용한다.
 */
export function buildColorRuns(
  text: string,
  charColors: (string | null)[] | null | undefined,
  defaultColor: string,
): { text: string; color: string }[] {
  const runs: { text: string; color: string }[] = []
  for (let i = 0; i < text.length; i++) {
    const color = (charColors?.[i] ?? null) || defaultColor
    const last = runs[runs.length - 1]
    if (last && last.color === color) last.text += text[i]
    else runs.push({ text: text[i], color })
  }
  return runs
}

/** 바이트 배열(number[]) → data URL. 저장된 배경서식 미리보기/생성 src 용. 대용량 대비 chunk btoa. */
export function bytesToDataUrl(bytes: number[], mime = 'image/png'): string {
  let binary = ''
  const CHUNK = 8192
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode(...bytes.slice(i, i + CHUNK))
  }
  return `data:${mime};base64,${btoa(binary)}`
}

/** data URL ("data:image/png;base64,....") → 바이트 배열(number[]). IPC Vec<u8> 전달용. */
export function dataUrlToBytes(dataUrl: string): number[] {
  const comma = dataUrl.indexOf(',')
  const base64 = comma >= 0 ? dataUrl.slice(comma + 1) : dataUrl
  const binary = atob(base64)
  const bytes = new Array<number>(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return bytes
}

/**
 * 텍스트박스 1종을 캔버스에 그린다 — 박스 영역 내 세로 중앙, 좌/중/우 정렬, 글자별 색 반영.
 * 폰트 크기 = 박스높이 × fontRatio. 줄바꿈('\n')은 줄 단위로 그리며 자동 줄바꿈(소프트 랩)은 미지원.
 */
function drawTextbox(
  ctx: CanvasRenderingContext2D,
  tb: TextboxConfig,
  data: NoticeStudentData,
  bgW: number,
  bgH: number,
): void {
  const boxX = tb.xRatio * bgW
  const boxY = tb.yRatio * bgH
  const boxW = tb.wRatio * bgW
  const boxH = tb.hRatio * bgH
  // 배경색(지정 시) — 텍스트 유무와 무관하게 박스 영역을 채운다(미리보기 박스와 일치).
  if (tb.backgroundColor) {
    ctx.fillStyle = tb.backgroundColor
    ctx.fillRect(boxX, boxY, boxW, boxH)
  }
  const text = noticeFieldText(tb, data)
  if (text === '') return
  const fontSize = tb.fontRatio * boxH
  const weight = tb.fontWeight === 'bold' ? '700' : '400'
  ctx.font = `${weight} ${fontSize}px Pretendard, sans-serif`
  ctx.textBaseline = 'middle'
  ctx.textAlign = 'left'
  const lineHeight = fontSize * 1.3
  const lines = text.split('\n')
  // 여러 줄이면 블록 전체를 박스 세로 중앙에 배치
  let cy = boxY + boxH / 2 - (lineHeight * (lines.length - 1)) / 2
  let charOffset = 0
  for (const line of lines) {
    const lineColors = tb.charColors ? tb.charColors.slice(charOffset, charOffset + line.length) : null
    const runs = buildColorRuns(line, lineColors, tb.fontColor)
    const totalW = runs.reduce((acc, r) => acc + ctx.measureText(r.text).width, 0)
    let x =
      tb.textAlign === 'center'
        ? boxX + (boxW - totalW) / 2
        : tb.textAlign === 'right'
          ? boxX + boxW - totalW
          : boxX
    for (const run of runs) {
      ctx.fillStyle = run.color
      ctx.fillText(run.text, x, cy)
      x += ctx.measureText(run.text).width
    }
    charOffset += line.length + 1 // '\n' 1글자 포함
    cy += lineHeight
  }
}

function waitImageLoaded(img: HTMLImageElement): Promise<void> {
  return new Promise((resolve, reject) => {
    if (img.complete && img.naturalWidth > 0) return resolve()
    img.onload = () => resolve()
    img.onerror = () => reject(new Error('배경서식 이미지를 불러올 수 없습니다.'))
  })
}

/**
 * 단일 원생 공지문을 Canvas 2D 로 렌더 → PNG 바이트 배열 반환.
 *
 * html-to-image(SVG foreignObject) 는 macOS WKWebView 에서 data URL `<img>` 를 빈 이미지로
 * 출력하는 결함이 있어, 캔버스에 배경+텍스트를 직접 그린다 (Windows/macOS 공통 안정).
 */
/** 미리보기용 렌더 파라미터 (저장 불필요 — 단일 이미지 미리보기/내보내기). */
export interface RenderParams {
  backgroundDataUrl: string
  width: number
  height: number
  layout: NoticeLayout
  imageUrls?: NoticeImageUrls
  customImages?: NoticeCustomImageDraw[]
}

/**
 * 배경 + 텍스트박스를 캔버스에 그려 PNG data URL 을 반환한다 (저장 없음).
 * 생성(`renderNoticePng`)·미리보기 공통 코어.
 */
export async function renderNoticeDataUrl(params: RenderParams, data: NoticeStudentData): Promise<string> {
  if (typeof document === 'undefined') {
    throw new Error('이미지 생성은 앱 화면(클라이언트)에서만 가능합니다.')
  }
  // 폰트 로드 보장 — 미로드 시 캔버스가 fallback 폰트로 그려 글자 폭/모양이 달라짐
  if (document.fonts?.ready) {
    try {
      await document.fonts.ready
    } catch {
      /* 폰트 상태 조회 실패는 무시 */
    }
  }

  const canvas = document.createElement('canvas')

  // 배경서식 — 캔버스를 배경 "원본 해상도"로 잡아 글씨 깨짐을 막는다. params.width/height 는
  // 미리보기용 표시 추정치라 원본보다 작을 수 있어, 그대로 쓰면 축소 생성되어 글씨가 깨진다.
  const bg = new Image()
  bg.src = params.backgroundDataUrl
  await waitImageLoaded(bg)
  const w = bg.naturalWidth || params.width
  const h = bg.naturalHeight || params.height
  canvas.width = w
  canvas.height = h
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('캔버스를 초기화할 수 없습니다.')
  ctx.imageSmoothingEnabled = true
  ctx.imageSmoothingQuality = 'high'
  ctx.drawImage(bg, 0, 0, w, h)

  // 사용자 추가 이미지 — 배경 바로 위(로고/바코드·텍스트보다 아래). 개별 로드 실패는 건너뛴다.
  for (const ci of params.customImages ?? []) {
    try {
      const elem = new Image()
      elem.src = ci.dataUrl
      await waitImageLoaded(elem)
      ctx.drawImage(elem, ci.xRatio * w, ci.yRatio * h, ci.wRatio * w, ci.hRatio * h)
    } catch {
      /* 추가 이미지 로드 실패 — 해당 이미지만 생략 */
    }
  }

  // 이미지 요소(로고/2D바코드) — 텍스트보다 먼저 그려 텍스트가 이미지 위에 오도록 한다
  // (미리보기 DOM 순서와 일치). 개별 이미지 로드 실패는 건너뛴다(전체 생성 중단 방지).
  const imageUrls = params.imageUrls
  if (imageUrls) {
    for (const im of params.layout.images ?? []) {
      if (!im.enabled) continue
      const url = imageUrls[im.kind]
      if (!url) continue
      try {
        const elem = new Image()
        elem.src = url
        await waitImageLoaded(elem)
        ctx.drawImage(elem, im.xRatio * w, im.yRatio * h, im.wRatio * w, im.hRatio * h)
      } catch {
        /* 이미지 요소 로드 실패 — 해당 이미지만 생략 */
      }
    }
  }

  // 텍스트박스 (체크 해제 항목 제외) — 이미지 위에 그린다.
  for (const tb of params.layout.textboxes) {
    if (tb.enabled === false) continue
    drawTextbox(ctx, tb, data, w, h)
  }

  return canvas.toDataURL('image/png') // 1x
}

async function renderNoticePng(opts: GenerateOptions, data: NoticeStudentData): Promise<number[]> {
  const dataUrl = await renderNoticeDataUrl(
    {
      backgroundDataUrl: opts.backgroundDataUrl,
      width: opts.width,
      height: opts.height,
      layout: opts.layout,
      imageUrls: opts.imageUrls,
      customImages: opts.customImages,
    },
    data,
  )
  return dataUrlToBytes(dataUrl)
}

/**
 * 선택된 원생들의 공지문을 순차 생성·저장한다. 저장 경로 목록 반환.
 * 순차 처리 + setTimeout(0) 인터리브로 UI 블로킹을 방지한다.
 */
export async function generateAndSaveNotices(
  opts: GenerateOptions,
): Promise<{ saved: number; paths: string[] }> {
  const paths: string[] = []
  const total = opts.students.length
  for (let i = 0; i < total; i++) {
    const bytes = await renderNoticePng(opts, opts.students[i])
    const path = await saveNoticeImage(
      opts.noticeName,
      opts.yearMonth,
      opts.students[i].studentName,
      bytes,
    )
    paths.push(path)
    opts.onProgress?.(i + 1, total)
    await new Promise((r) => setTimeout(r, 0)) // UI 양보
  }
  return { saved: paths.length, paths }
}
