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
import { SplashScreen } from '@/components/splash-screen'
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

  // V30 (Sprint 7 post-review): dev 빌드 자동 로그인 우회. 환경 변수
  // `NEXT_PUBLIC_DEV_AUTOLOGIN` 에 평문 비밀번호 (8자 이상) 가 설정되어 있으면 자동 입력 + 제출.
  // 이미 한 번 `set_password` 한 상태 (`status==='locked'`) 에서만 우회 — 첫 설치 시 마법사는
  // 사용자가 직접 진행. release 빌드에서는 NEXT_PUBLIC 환경 변수 자체가 없으므로 무동작.
  useEffect(() => {
    const devPw = process.env.NEXT_PUBLIC_DEV_AUTOLOGIN
    if (status !== 'locked' || !devPw || devPw.length < MIN_PASSWORD_LENGTH) return
    if (submitting) return
    setPasswordInput(devPw)
    // 다음 tick 에서 form submit — state 갱신 후 동기 호출이라 안전한 microtask.
    void (async () => {
      setSubmitting(true)
      try {
        const startup = await appStartupSequence(devPw, false)
        onUnlocked?.(startup)
      } catch (e) {
        setError(
          typeof e === 'string'
            ? e
            : 'dev 자동 로그인 실패 — 비밀번호 또는 락 상태 확인 필요.',
        )
      } finally {
        setSubmitting(false)
      }
    })()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status])

  if (status === null) {
    return <SplashScreen message="잠금 상태를 확인하는 중입니다..." />
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

/** V37b — 마지막 입력 문자 종류 감지. 한글 자모/음절 / 영문 / 숫자 / 특수 / null(빈 입력). */
function detectInputMode(text: string): '한글' | '영문' | '숫자' | '특수' | null {
  if (text.length === 0) return null
  const ch = text[text.length - 1]
  // U+1100~ 한글 자모, U+3130~ 한글 호환 자모, U+AC00~ 한글 음절
  if (/[ᄀ-ᇿ㄰-㆏가-힣]/.test(ch)) return '한글'
  if (/[a-zA-Z]/.test(ch)) return '영문'
  if (/[0-9]/.test(ch)) return '숫자'
  return '특수'
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
  // V37b (Sprint 7 post-review): 한글 차단 제거 → 사용자가 한글 비밀번호도 가능. 단, 마지막
  // 입력 문자의 종류를 실시간 배지로 표시하여 의도와 다른 IME 모드 즉시 인지 가능.
  const [composing, setComposing] = useState(false)
  const mode = composing ? '한글' : detectInputMode(value)
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
          onCompositionStart={() => setComposing(true)}
          onCompositionEnd={() => setComposing(false)}
          autoComplete={id === 'password' ? 'current-password' : 'new-password'}
          autoFocus={autoFocus}
          lang="en"
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          inputMode="text"
          className={`h-[56px] w-full rounded-lg border-2 px-4 pr-32 text-lg focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
            hasError ? 'border-[var(--danger)]' : 'border-[var(--border)]'
          }`}
        />
        {/* V37b — 입력 모드 배지 (한글일 때 강조). */}
        {mode !== null && (
          <span
            aria-live="polite"
            title="마지막으로 입력한 문자 종류"
            className={`absolute right-[72px] top-1/2 flex h-[44px] min-w-[44px] -translate-y-1/2 items-center justify-center rounded-md px-2 text-sm ${
              mode === '한글'
                ? 'bg-orange-100 font-semibold text-orange-700'
                : 'bg-gray-100 text-gray-700'
            }`}
          >
            {mode}
          </span>
        )}
        {/* V36 — 보기/숨김 버튼. */}
        <button
          type="button"
          onClick={onToggleShow}
          aria-label={show ? '비밀번호 가리기' : '비밀번호 표시'}
          className="absolute right-2 top-1/2 flex h-[44px] min-w-[60px] -translate-y-1/2 items-center justify-center rounded-md border border-[var(--border)] bg-white px-2 text-sm text-gray-700 hover:bg-gray-50"
        >
          {show ? '숨김' : '보기'}
        </button>
      </div>
      <p className="text-xs text-gray-500">
        입력 문자 종류가 우측에 표시됩니다. 의도와 다르면 키보드 한/영 전환 후 다시 입력하세요.
      </p>
    </div>
  )
}
