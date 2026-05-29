'use client'

/**
 * 전역 에러 다이얼로그 — 빨간 인라인 박스를 대체하는 명시적 확인 모달.
 *
 * 사용처: 각 페이지의 mutation/검증 에러 알림. message 비어있으면(빈 문자열·null) 렌더하지 않음.
 * `createPortal` 로 `document.body` 직접 렌더 — 부모(AppShell main 의 overflow-y-auto 등)
 * stacking context 영향 없이 viewport 최상위에 노출.
 */

import { createPortal } from 'react-dom'

interface Props {
  open: boolean
  title?: string
  message: string
  onClose: () => void
}

export function ErrorDialog({ open, title = '오류', message, onClose }: Props) {
  // 임시 디버그 (다음 commit 에서 제거)
  console.log('[DEBUG ErrorDialog]', { open, messageLen: message.length, message })
  if (!open || message.trim() === '') return null
  if (typeof document === 'undefined') return null
  const dialog = (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-label={title}
      className="fixed inset-0 z-[70] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
        <h2 className="mb-2 text-xl font-bold text-[var(--danger)]">{title}</h2>
        <p className="mb-4 whitespace-pre-wrap text-base text-gray-800">{message}</p>
        <div className="flex justify-end">
          <button
            type="button"
            onClick={onClose}
            autoFocus
            className="min-h-[44px] min-w-[100px] rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90"
          >
            확인
          </button>
        </div>
      </div>
    </div>
  )
  return createPortal(dialog, document.body)
}
