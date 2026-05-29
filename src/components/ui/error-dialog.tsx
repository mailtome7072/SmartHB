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
  const dialog = (
    <>
      {/* 임시 마커 — 화면 하단 녹색 띠. ErrorDialog render 호출이 일어났음을 확인 */}
      <div
        style={{
          position: 'fixed',
          bottom: 0,
          left: 0,
          right: 0,
          background: 'lime',
          color: 'black',
          padding: 8,
          fontSize: 14,
          zIndex: 999999,
          textAlign: 'center',
        }}
      >
        🟢 ErrorDialog MOUNTED — message: {message}
      </div>
      {/* 본 모달 — inline style 로 변경 (Tailwind JIT 누락 가능성 차단) */}
      <div
        role="alertdialog"
        aria-modal="true"
        aria-label={title}
        style={{
          position: 'fixed',
          inset: 0,
          zIndex: 999998,
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
              autoFocus
              style={{
                minHeight: 44,
                minWidth: 100,
                borderRadius: 6,
                background: '#2563eb',
                color: 'white',
                fontSize: 16,
                fontWeight: 600,
                padding: '0 16px',
              }}
            >
              확인
            </button>
          </div>
        </div>
      </div>
    </>
  )
  return createPortal(dialog, document.body)
}
