'use client'

/**
 * 잠금 화면 컴포넌트 — Sprint 1 T4.
 *
 * 인증 상태에 따라 두 모드로 자동 분기한다:
 * - **최초 설정 모드** (`not-initialized`): 비밀번호 + 확인 입력 → `setPassword` IPC
 * - **잠금 해제 모드** (`locked`): 비밀번호 1개 입력 → `unlockDb` IPC
 *
 * PRD §5.7 + Phase 1 UX 검토 준수:
 * - 입력 필드 높이 56px, 클릭 영역 ≥ 44×44px
 * - 색상 팔레트 `#F9F7F4` / `#2563EB` / `#1A1A1A`
 * - 한국어 오류 메시지 (기술 디테일 비공개)
 * - 비밀번호 표시 토글 (Unicode 아이콘 임시 사용, T6 lucide-react 도입 시 교체)
 *
 * 후속 작업:
 * - "비밀번호를 잊으셨나요?" 링크 → T5 복구 코드 입력 화면
 * - 인증 성공 시 메인 화면 라우팅 → T9 통합 단계
 */

import { useEffect, useState } from 'react'
import { appStartupSequence, checkAuthStatus, setPassword } from '@/lib/tauri'
import type { AuthStatus, StartupResult } from '@/types'

const MIN_PASSWORD_LENGTH = 8

export function LockScreen({ onUnlocked }: { onUnlocked?: (result: StartupResult) => void }) {
  const [status, setStatus] = useState<AuthStatus | null>(null)
  const [password, setPasswordInput] = useState('')
  const [confirm, setConfirm] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    checkAuthStatus()
      .then(setStatus)
      .catch((e) => setError(typeof e === 'string' ? e : '인증 상태를 확인할 수 없습니다.'))
  }, [])

  if (status === null) {
    return <div className="flex min-h-screen items-center justify-center">불러오는 중...</div>
  }

  const isInitialSetup = status === 'not-initialized'
  const title = isInitialSetup ? '비밀번호 설정' : '비밀번호 입력'
  const subtitle = isInitialSetup
    ? '앱 보호를 위해 비밀번호를 설정해주세요.'
    : '계속하려면 비밀번호를 입력해주세요.'

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    if (password.length < MIN_PASSWORD_LENGTH) {
      setError(`비밀번호는 최소 ${MIN_PASSWORD_LENGTH}자 이상이어야 합니다.`)
      return
    }
    if (isInitialSetup && password !== confirm) {
      setError('비밀번호와 확인 입력이 일치하지 않습니다.')
      return
    }
    setSubmitting(true)
    try {
      if (isInitialSetup) {
        // 최초 비밀번호 설정 — keyring 에 salt + key 저장 (audit log 포함).
        await setPassword(password)
      }
      // 양 모드 공통: startup 시퀀스로 락 + 무결성 + DB pool 초기화 + 백그라운드 task 시작.
      // 백엔드가 내부적으로 verify_password 수행 → unlockDb IPC 별도 호출 불필요.
      const startup = await appStartupSequence(password, false)
      onUnlocked?.(startup)
    } catch (e) {
      setError(typeof e === 'string' ? e : '처리 중 오류가 발생했습니다. 다시 시도해주세요.')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center px-4">
      <form onSubmit={handleSubmit} className="w-full max-w-md space-y-6">
        <header className="space-y-2 text-center">
          <h1 className="text-3xl font-bold">{title}</h1>
          <p className="text-base text-gray-600">{subtitle}</p>
        </header>

        <div className="space-y-4">
          <PasswordField
            id="password"
            label="비밀번호"
            value={password}
            onChange={setPasswordInput}
            show={showPassword}
            onToggleShow={() => setShowPassword(!showPassword)}
            autoFocus
            hasError={error !== null}
          />
          {isInitialSetup && (
            <PasswordField
              id="confirm"
              label="비밀번호 확인"
              value={confirm}
              onChange={setConfirm}
              show={showPassword}
              onToggleShow={() => setShowPassword(!showPassword)}
              hasError={error !== null}
            />
          )}
        </div>

        {error !== null && (
          <p
            role="alert"
            className="rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        <button
          type="submit"
          disabled={submitting}
          className="h-[56px] w-full rounded-lg bg-[var(--accent)] text-lg font-semibold text-white transition-colors hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          {submitting ? '처리 중...' : isInitialSetup ? '설정하기' : '잠금 해제'}
        </button>

        {!isInitialSetup && (
          <button
            type="button"
            className="block w-full text-center text-base text-[var(--accent)] underline-offset-2 hover:underline"
          >
            비밀번호를 잊으셨나요?
          </button>
        )}
      </form>
    </main>
  )
}

interface PasswordFieldProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  show: boolean
  onToggleShow: () => void
  autoFocus?: boolean
  hasError: boolean
}

function PasswordField({
  id,
  label,
  value,
  onChange,
  show,
  onToggleShow,
  autoFocus,
  hasError,
}: PasswordFieldProps) {
  return (
    <div className="space-y-2">
      <label htmlFor={id} className="block text-base font-medium">
        {label}
      </label>
      <div className="relative">
        <input
          id={id}
          type={show ? 'text' : 'password'}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          autoComplete={id === 'password' ? 'current-password' : 'new-password'}
          autoFocus={autoFocus}
          className={`h-[56px] w-full rounded-lg border-2 px-4 pr-14 text-lg focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
            hasError ? 'border-[var(--danger)]' : 'border-[var(--border)]'
          }`}
        />
        <button
          type="button"
          onClick={onToggleShow}
          aria-label={show ? '비밀번호 가리기' : '비밀번호 표시'}
          className="absolute right-2 top-1/2 flex h-[44px] w-[44px] -translate-y-1/2 items-center justify-center rounded-md text-xl text-gray-600 hover:bg-gray-100"
        >
          {show ? '🙈' : '👁'}
        </button>
      </div>
    </div>
  )
}
