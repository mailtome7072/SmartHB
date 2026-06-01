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
 * - 인증 성공 시 메인 화면 라우팅 → T9 통합 단계
 */

import { useEffect, useState } from 'react'
import { appStartupSequence, checkAuthStatus, setPassword } from '@/lib/tauri'
import { SplashScreen } from '@/components/splash-screen'
import type { AuthStatus, StartupResult } from '@/types'

// ADR-007: 앱 잠금은 6자리 숫자 PIN.
const PIN_LENGTH = 6
const PIN_PATTERN = /^\d{6}$/

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
  // `NEXT_PUBLIC_DEV_AUTOLOGIN` 에 6자리 숫자 PIN 이 설정되어 있으면 자동 입력 + 제출 (ADR-007).
  // 이미 한 번 `set_password` 한 상태 (`status==='locked'`) 에서만 우회 — 첫 설치 시 마법사는
  // 사용자가 직접 진행.
  //
  // ⚠️ 보안 주의 (R50, Sprint 7 post-review): `NEXT_PUBLIC_*` 환경 변수는 Next.js 빌드 타임에
  // 클라이언트 번들에 **인라인**된다. `.env` 에 설정된 채 `pnpm tauri:build` (release) 를 실행
  // 하면 dev 비밀번호가 인스톨러 JS 번들에 포함되어 배포된다. release 빌드 전에 반드시 `.env`
  // 에서 본 변수를 제거하거나 빈 값으로 설정할 것. 또는 `unset NEXT_PUBLIC_DEV_AUTOLOGIN` 후
  // 빌드. CI 환경에서는 환경 변수 미설정 상태가 기본이라 안전.
  useEffect(() => {
    const devPw = process.env.NEXT_PUBLIC_DEV_AUTOLOGIN
    if (status !== 'locked' || !devPw || !PIN_PATTERN.test(devPw)) return
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
  const title = isInitialSetup ? 'PIN 번호 설정' : 'PIN 번호 입력'
  const subtitle = isInitialSetup
    ? '앱 보호를 위해 6자리 숫자 PIN 을 설정해주세요.'
    : '계속하려면 6자리 PIN 번호를 입력해주세요.'

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    if (!PIN_PATTERN.test(password)) {
      setError(`PIN 번호는 ${PIN_LENGTH}자리 숫자여야 합니다.`)
      return
    }
    if (isInitialSetup && password !== confirm) {
      setError('PIN 번호와 확인 입력이 일치하지 않습니다.')
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
          <PinField
            id="password"
            label="PIN 번호"
            value={password}
            onChange={setPasswordInput}
            show={showPassword}
            onToggleShow={() => setShowPassword(!showPassword)}
            autoFocus
            hasError={error !== null}
          />
          {isInitialSetup && (
            <PinField
              id="confirm"
              label="PIN 번호 확인"
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

      </form>
    </main>
  )
}

interface PinFieldProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  show: boolean
  onToggleShow: () => void
  autoFocus?: boolean
  hasError: boolean
}

/**
 * 6자리 숫자 PIN 입력 필드 (ADR-007).
 *
 * 숫자 외 입력은 onChange 단계에서 필터링하고 6자리로 제한한다. 모바일/터치 환경에서는
 * `inputMode="numeric"` 으로 숫자 키패드가 노출된다. 기본은 가림(●), 보기 토글 제공.
 */
function PinField({
  id,
  label,
  value,
  onChange,
  show,
  onToggleShow,
  autoFocus,
  hasError,
}: PinFieldProps) {
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
          onChange={(e) => onChange(e.target.value.replace(/\D/g, '').slice(0, PIN_LENGTH))}
          autoComplete="off"
          autoFocus={autoFocus}
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          inputMode="numeric"
          maxLength={PIN_LENGTH}
          placeholder={'●'.repeat(PIN_LENGTH)}
          className={`h-[56px] w-full rounded-lg border-2 px-4 pr-24 text-center text-2xl tracking-[0.4em] focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
            hasError ? 'border-[var(--danger)]' : 'border-[var(--border)]'
          }`}
        />
        {/* 보기/숨김 버튼. */}
        <button
          type="button"
          onClick={onToggleShow}
          aria-label={show ? 'PIN 가리기' : 'PIN 표시'}
          className="absolute right-2 top-1/2 flex h-[44px] min-w-[60px] -translate-y-1/2 items-center justify-center rounded-md border border-[var(--border)] bg-white px-2 text-sm text-gray-700 hover:bg-gray-50"
        >
          {show ? '숨김' : '보기'}
        </button>
      </div>
      <p className="text-xs text-gray-500">숫자 {PIN_LENGTH}자리를 입력하세요.</p>
    </div>
  )
}
