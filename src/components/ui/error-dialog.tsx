'use client'

/**
 * 전역 에러 다이얼로그 — 빨간 인라인 박스를 대체하는 명시적 확인 모달.
 *
 * 사용처: 각 페이지의 mutation/검증 에러 알림. message 비어있으면(빈 문자열·null) 렌더하지 않음.
 * 다이얼로그 패턴은 Sprint 10/11 `WithdrawalMakeupDialog` / `CloseReasonDialog` 와 동일
 * (native fixed inset modal). z-[70] 으로 기존 다이얼로그(z-[60]) 위에 표시.
 */

interface Props {
  open: boolean
  title?: string
  message: string
  onClose: () => void
}

export function ErrorDialog({ open, title = '오류', message, onClose }: Props) {
  if (!open || message.trim() === '') return null
  return (
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
}
