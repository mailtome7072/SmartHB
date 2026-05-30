/**
 * 공지문 이미지 생성 엔진 — Sprint 12 T7 (PRD §4.10.2, AC-4.10-1).
 *
 * 배경서식(data URL) 위에 레이아웃의 텍스트박스 3종을 배치한 오프스크린 DOM 노드를 만들고,
 * `html-to-image`의 `toPng`로 PNG를 렌더링한다. 원생별로 텍스트만 교체하여 순차 생성·저장한다.
 * 청구액은 천단위 콤마(`Intl.NumberFormat('ko-KR')`)로 표기한다 (AC-4.10-1).
 */

import { toPng } from 'html-to-image'
import { saveNoticeImage } from '@/lib/tauri'
import type { NoticeLayout, NoticeFieldType, TextboxConfig } from '@/types/notice'

/** 한 원생의 공지문 데이터 소스. */
export interface NoticeStudentData {
  studentName: string
  /** 청구년월 'YYYY-MM'. */
  billYearMonth: string
  /** 청구액(원). */
  billAmount: number
}

export interface GenerateOptions {
  yearMonth: string
  /** 배경서식 이미지 data URL. */
  backgroundDataUrl: string
  /** 렌더 캔버스 크기(px) — 배경 이미지 자연 크기 기준. */
  width: number
  height: number
  layout: NoticeLayout
  students: NoticeStudentData[]
  /** 진행률 콜백 (완료 건수, 총 건수). */
  onProgress?: (done: number, total: number) => void
}

const wonFormatter = new Intl.NumberFormat('ko-KR')

/** 'YYYY-MM' → 'YYYY년 M월'. */
function formatBillMonth(yearMonth: string): string {
  const [y, m] = yearMonth.split('-')
  if (!y || !m) return yearMonth
  return `${y}년 ${Number(m)}월`
}

/** 텍스트박스 field_type 에 해당하는 표시 텍스트. */
export function noticeFieldText(field: NoticeFieldType, data: NoticeStudentData): string {
  switch (field) {
    case 'bill_month':
      return formatBillMonth(data.billYearMonth)
    case 'student_name':
      return data.studentName
    case 'bill_amount':
      return `${wonFormatter.format(data.billAmount)}원` // AC-4.10-1 천단위 콤마
    default:
      return ''
  }
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

/** 비율 텍스트박스를 배경 해상도(bgW×bgH) 기준 절대 px 스타일로 변환. 폰트 = 박스높이 × fontRatio. */
function applyTextboxStyle(el: HTMLDivElement, tb: TextboxConfig, bgW: number, bgH: number): void {
  const boxH = tb.hRatio * bgH
  el.style.position = 'absolute'
  el.style.left = `${tb.xRatio * bgW}px`
  el.style.top = `${tb.yRatio * bgH}px`
  el.style.width = `${tb.wRatio * bgW}px`
  el.style.height = `${boxH}px`
  el.style.fontSize = `${tb.fontRatio * boxH}px`
  el.style.fontWeight = tb.fontWeight
  el.style.color = tb.fontColor
  el.style.textAlign = tb.textAlign
  el.style.display = 'flex'
  el.style.alignItems = 'center'
  el.style.justifyContent =
    tb.textAlign === 'center' ? 'center' : tb.textAlign === 'right' ? 'flex-end' : 'flex-start'
  el.style.fontFamily = 'Pretendard, sans-serif'
  el.style.lineHeight = '1.3'
  el.style.whiteSpace = 'pre-wrap'
  el.style.overflow = 'hidden'
}

function waitImageLoaded(img: HTMLImageElement): Promise<void> {
  return new Promise((resolve, reject) => {
    if (img.complete && img.naturalWidth > 0) return resolve()
    img.onload = () => resolve()
    img.onerror = () => reject(new Error('배경서식 이미지를 불러올 수 없습니다.'))
  })
}

/** 단일 원생 공지문을 오프스크린 렌더 → PNG 바이트 배열 반환. */
async function renderNoticePng(opts: GenerateOptions, data: NoticeStudentData): Promise<number[]> {
  if (typeof document === 'undefined') {
    throw new Error('이미지 생성은 앱 화면(클라이언트)에서만 가능합니다.')
  }
  const container = document.createElement('div')
  container.style.cssText = `position:fixed;left:-99999px;top:0;width:${opts.width}px;height:${opts.height}px;background:#fff;`

  const bg = document.createElement('img')
  bg.src = opts.backgroundDataUrl
  bg.style.cssText = `position:absolute;left:0;top:0;width:${opts.width}px;height:${opts.height}px;`
  container.appendChild(bg)

  for (const tb of opts.layout.textboxes) {
    const box = document.createElement('div')
    applyTextboxStyle(box, tb, opts.width, opts.height)
    box.textContent = noticeFieldText(tb.fieldType, data)
    container.appendChild(box)
  }

  document.body.appendChild(container)
  try {
    await waitImageLoaded(bg)
    const dataUrl = await toPng(container, {
      pixelRatio: 1, // 성능(50장<30초) — 배경 자연 크기 기준 1x (PRD §5.6)
      width: opts.width,
      height: opts.height,
      cacheBust: true,
    })
    return dataUrlToBytes(dataUrl)
  } finally {
    document.body.removeChild(container)
  }
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
    const path = await saveNoticeImage(opts.yearMonth, opts.students[i].studentName, bytes)
    paths.push(path)
    opts.onProgress?.(i + 1, total)
    await new Promise((r) => setTimeout(r, 0)) // UI 양보
  }
  return { saved: paths.length, paths }
}
