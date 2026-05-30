'use client'

/**
 * 비밀번호 분실 시 복구 코드 입력 + 새 비밀번호 설정 화면 — T5 PI-07.
 *
 * 흐름: 사용자가 LockScreen 의 "비밀번호를 잊으셨나요?" 링크 클릭 → 본 컴포넌트 진입 →
 * 12자리 코드 입력 + 새 비밀번호(+ 확인) 입력 → `resetPasswordWithCode` IPC 호출 →
 * 성공 시 LockScreen 잠금 해제 모드로 복귀하여 새 비밀번호로 로그인.
 *
 * 코드 입력 형식은 자유 (대소문자·공백·하이픈 모두 허용) — 백엔드에서 정규화한다.
 */

import { useState } from 'react'
import { resetPasswordWithCode } from '@/lib/tauri'
import { normalizeRecoveryCode } from '@/lib/recovery-code'

// ADR-007: 새 잠금 인증도 6자리 숫자 PIN.
const PIN_LENGTH = 6
const PIN_PATTERN = /^\d{6}$/
const CODE_LEN = 12

interface RecoveryCodeInputProps {
  /** 재설정 성공 후 호출 — 보통 LockScreen 으로 복귀. */
  onReset: () => void
  /** 취소 클릭 시 호출 — 보통 LockScreen 으로 복귀. */
  onCancel: () => void
}

export function RecoveryCodeInput({ onReset, onCancel }: RecoveryCodeInputProps) {
  const [code, setCode] = useState('')
  const [newPin, setNewPin] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  const normalizedCodeLength = normalizeRecoveryCode(code).length

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    if (normalizedCodeLength !== CODE_LEN) {
      setError(`복구 코드는 ${CODE_LEN}자리여야 합니다.`)
      return
    }
    if (!PIN_PATTERN.test(newPin)) {
      setError(`새 PIN 번호는 ${PIN_LENGTH}자리 숫자여야 합니다.`)
      return
    }
    if (newPin !== confirm) {
      setError('새 PIN 번호와 확인 입력이 일치하지 않습니다.')
      return
    }
    setSubmitting(true)
    try {
      await resetPasswordWithCode(code, newPin)
      onReset()
    } catch (e) {
      setError(typeof e === 'string' ? e : '복구 코드가 일치하지 않습니다.')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center px-4">
      <form onSubmit={handleSubmit} className="w-full max-w-md space-y-6">
        <header className="space-y-2 text-center">
          <h1 className="text-3xl font-bold">비밀번호 재설정</h1>
          <p className="text-base text-gray-600">
            보관해두신 복구 코드와 새 비밀번호를 입력해주세요.
          </p>
        </header>

        <div className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="code" className="block text-base font-medium">
              복구 코드 (12자리)
            </label>
            <input
              id="code"
              type="text"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              autoComplete="off"
              autoCapitalize="characters"
              spellCheck={false}
              placeholder="XXXX-XXXX-XXXX"
              className={`h-[56px] w-full rounded-lg border-2 px-4 font-mono text-lg tracking-wider focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
                error !== null ? 'border-[var(--danger)]' : 'border-[var(--border)]'
              }`}
            />
            <p className="text-sm text-gray-500">
              {normalizedCodeLength} / {CODE_LEN} 자
            </p>
          </div>

          <div className="space-y-2">
            <label htmlFor="newPin" className="block text-base font-medium">
              새 PIN 번호 (6자리 숫자)
            </label>
            <input
              id="newPin"
              type="password"
              value={newPin}
              onChange={(e) => setNewPin(e.target.value.replace(/\D/g, '').slice(0, PIN_LENGTH))}
              autoComplete="off"
              inputMode="numeric"
              maxLength={PIN_LENGTH}
              placeholder={'●'.repeat(PIN_LENGTH)}
              className={`h-[56px] w-full rounded-lg border-2 px-4 text-center text-2xl tracking-[0.4em] focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
                error !== null ? 'border-[var(--danger)]' : 'border-[var(--border)]'
              }`}
            />
          </div>

          <div className="space-y-2">
            <label htmlFor="confirmPin" className="block text-base font-medium">
              새 PIN 번호 확인
            </label>
            <input
              id="confirmPin"
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value.replace(/\D/g, '').slice(0, PIN_LENGTH))}
              autoComplete="off"
              inputMode="numeric"
              maxLength={PIN_LENGTH}
              placeholder={'●'.repeat(PIN_LENGTH)}
              className={`h-[56px] w-full rounded-lg border-2 px-4 text-center text-2xl tracking-[0.4em] focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
                error !== null ? 'border-[var(--danger)]' : 'border-[var(--border)]'
              }`}
            />
          </div>
        </div>

        {error !== null && (
          <p
            role="alert"
            className="rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        <div className="space-y-3">
          <button
            type="submit"
            disabled={submitting}
            className="h-[56px] w-full rounded-lg bg-[var(--accent)] text-lg font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            {submitting ? '확인 중...' : '비밀번호 재설정'}
          </button>
          <button
            type="button"
            onClick={onCancel}
            className="h-[44px] w-full rounded-lg border-2 border-[var(--border)] text-base font-medium hover:bg-gray-50"
          >
            취소
          </button>
        </div>
      </form>
    </main>
  )
}
