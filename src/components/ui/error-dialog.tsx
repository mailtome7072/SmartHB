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
  if (!open || message.trim() === '') return null
  if (typeof document === 'undefined') return null
  // inline style 사용 — Tailwind JIT 누락 또는 stacking context 영향을 받지 않도록.
  const dialog = (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-label={title}
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 2147483646,
        background: 'rgba(0,0,0,0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 16,
      }}
    >
      <div
        style={{
          width: '100%',
          maxWidth: 480,
          background: 'white',
          borderRadius: 8,
          padding: 24,
          boxShadow: '0 10px 25px rgba(0,0,0,0.2)',
        }}
      >
        <h2 style={{ marginBottom: 8, fontSize: 20, fontWeight: 700, color: '#b91c1c' }}>
          {title}
        </h2>
        <p style={{ marginBottom: 16, fontSize: 16, color: '#1f2937', whiteSpace: 'pre-wrap' }}>
          {message}
        </p>
        <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
          <button
            type="button"
            onClick={onClose}
            style={{
              minHeight: 44,
              minWidth: 100,
              borderRadius: 6,
              background: '#2563eb',
              color: 'white',
              fontSize: 16,
              fontWeight: 600,
              padding: '0 16px',
              cursor: 'pointer',
            }}
          >
            확인
          </button>
        </div>
      </div>
    </div>
  )
  return createPortal(dialog, document.body)
}
