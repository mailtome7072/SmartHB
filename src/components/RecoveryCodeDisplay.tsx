'use client'

/**
 * 복구 코드 표시 컴포넌트 — T5 PI-07.
 *
 * 백엔드 `generate_recovery_code` IPC 응답으로 받은 12자리 평문 코드를 사용자에게
 * 1회 표시한다. 사용자가 "확인" 클릭 시 React state 의 평문을 빈 문자열로 덮어쓰며
 * GC 가 메모리를 회수하도록 한다 (브라우저 환경에서는 명시적 zeroize 불가).
 *
 * 표시 형식: `XXXX-XXXX-XXXX` (4-4-4) — 사용자 가독성. 백엔드 검증 시 dash 자동 제거.
 *
 * 분실 경고는 PRD §7.2 의무 — 비밀번호와 코드 모두 분실 시 데이터 영구 접근 불가.
 */

import { useEffect, useState } from 'react'
import { formatRecoveryCode } from '@/lib/recovery-code'

interface RecoveryCodeDisplayProps {
  code: string
  onConfirm: () => void
}

export function RecoveryCodeDisplay({ code, onConfirm }: RecoveryCodeDisplayProps) {
  const [copied, setCopied] = useState(false)

  const formatted = formatRecoveryCode(code)

  // "복사됨" 표시 2초 후 자동 해제. unmount 시 timer 누수 방지 cleanup.
  useEffect(() => {
    if (!copied) return
    const timer = setTimeout(() => setCopied(false), 2000)
    return () => clearTimeout(timer)
  }, [copied])

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(formatted)
      setCopied(true)
    } catch {
      // 클립보드 권한 거부 시 — 사용자가 수동으로 입력
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center px-4">
      <div className="w-full max-w-md space-y-6">
        <header className="space-y-2 text-center">
          <h1 className="text-3xl font-bold">복구 코드 발급 완료</h1>
          <p className="text-base text-gray-600">
            아래 12자리 코드를 안전한 곳에 보관해주세요.
          </p>
        </header>

        <div
          className="rounded-lg border-2 border-[var(--accent)] bg-white p-6 text-center"
          aria-label="복구 코드"
        >
          <p className="select-all font-mono text-2xl font-bold tracking-widest text-[var(--foreground)]">
            {formatted}
          </p>
        </div>

        <button
          type="button"
          onClick={handleCopy}
          className="h-[44px] w-full rounded-lg border-2 border-[var(--accent)] text-base font-medium text-[var(--accent)] hover:bg-blue-50"
        >
          {copied ? '복사됨 ✓' : '클립보드에 복사'}
        </button>

        <section className="rounded-md border-2 border-[var(--danger)] bg-red-50 p-4">
          <h2 className="mb-2 text-base font-bold text-[var(--danger)]">⚠️ 중요</h2>
          <ul className="space-y-1 text-sm text-[var(--danger)]">
            <li>이 코드는 단 한 번만 표시됩니다.</li>
            <li>비밀번호를 잊었을 때 이 코드로만 복구할 수 있습니다.</li>
            <li>비밀번호와 코드를 모두 분실하면 데이터에 영구 접근 불가합니다.</li>
            <li>종이에 인쇄하거나 안전한 장소에 보관해주세요.</li>
          </ul>
        </section>

        <button
          type="button"
          onClick={onConfirm}
          className="h-[56px] w-full rounded-lg bg-[var(--accent)] text-lg font-semibold text-white hover:bg-[var(--accent-hover)]"
        >
          안전한 곳에 보관했습니다
        </button>
      </div>
    </main>
  )
}

