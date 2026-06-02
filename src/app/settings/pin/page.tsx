'use client'

/**
 * PIN 변경 페이지.
 *
 * 흐름: 현 PIN 6자리 확인 → 새 PIN 6자리 + 확인 입력 → `changePin` IPC 호출.
 * 성공 시 토스트 안내 후 /settings 로 복귀. 실패 시(현 PIN 불일치 등) 에러 메시지 표시.
 *
 * 보안: 입력값은 컴포넌트 unmount 시 자동 해제 (React state). devtools 우회 방어는 백엔드
 * `validate_pin` + `verify_password` 가 담당 — 본 페이지는 1차 UX 검증만 수행.
 */

import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SettingsHomeLink } from '@/components/settings/SettingsHomeLink'
import { PIN_LENGTH, PIN_PATTERN, PinField } from '@/components/ui/pin-field'
import { changePin } from '@/lib/tauri'

export default function ChangePinPage() {
  const router = useRouter()
  const [currentPin, setCurrentPin] = useState('')
  const [newPin, setNewPin] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [done, setDone] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    if (!PIN_PATTERN.test(currentPin)) {
      setError(`현재 PIN 은 ${PIN_LENGTH}자리 숫자여야 합니다.`)
      return
    }
    if (!PIN_PATTERN.test(newPin)) {
      setError(`새 PIN 은 ${PIN_LENGTH}자리 숫자여야 합니다.`)
      return
    }
    if (newPin !== confirm) {
      setError('새 PIN 과 확인 입력이 일치하지 않습니다.')
      return
    }
    if (newPin === currentPin) {
      setError('새 PIN 이 현재 PIN 과 동일합니다. 다른 PIN 을 입력해주세요.')
      return
    }
    setSubmitting(true)
    try {
      await changePin(currentPin, newPin)
      setDone(true)
      setCurrentPin('')
      setNewPin('')
      setConfirm('')
      setTimeout(() => router.push('/settings'), 1500)
    } catch (e) {
      setError(
        typeof e === 'string'
          ? e
          : 'PIN 변경에 실패했습니다. 현재 PIN 을 확인해주세요.',
      )
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-md">
        <SettingsHomeLink />
        <h1 className="mb-2 text-center text-2xl font-bold">PIN 번호 변경</h1>
        <p className="mb-6 text-center text-base text-gray-600">
          현재 PIN 확인 후 새 PIN 으로 즉시 변경됩니다.
        </p>

        {done ? (
          <div className="rounded-md border-2 border-[var(--accent)] bg-blue-50 p-4 text-base text-[var(--accent)]">
            PIN 이 변경되었습니다. 잠시 후 설정 화면으로 돌아갑니다.
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-5">
            <PinField
              id="current-pin"
              label="현재 PIN"
              value={currentPin}
              onChange={setCurrentPin}
              hasError={error !== null}
              autoFocus
            />
            <PinField
              id="new-pin"
              label="새 PIN"
              value={newPin}
              onChange={setNewPin}
              hasError={error !== null}
            />
            <PinField
              id="confirm-pin"
              label="새 PIN 확인"
              value={confirm}
              onChange={setConfirm}
              hasError={error !== null}
            />

            {error !== null && (
              <p
                role="alert"
                className="rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
              >
                {error}
              </p>
            )}

            <div className="flex gap-3">
              <button
                type="submit"
                disabled={submitting}
                className="h-[56px] flex-1 rounded-lg bg-[var(--accent)] text-lg font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
              >
                {submitting ? '변경 중...' : 'PIN 변경'}
              </button>
              <button
                type="button"
                onClick={() => router.push('/settings')}
                className="h-[56px] flex-1 rounded-lg border-2 border-[var(--border)] text-base font-medium hover:bg-gray-50"
              >
                취소
              </button>
            </div>
          </form>
        )}
      </div>
    </AppShell>
  )
}
