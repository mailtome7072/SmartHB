'use client'

/**
 * `/lock` 라우트 — 잠금 화면.
 *
 * Sprint 5 T1-sub: 락 상태에 따라 두 화면을 분기 렌더.
 * - `owned-by-other`: LockWarning (다른 PC 사용 중 안내 + 5분 이상 미갱신 시 강제 점유 옵션)
 * - 그 외(`free` / `owned-by-self`): LockScreen (비밀번호 입력)
 *
 * 인증 성공 시 (LockScreen.onUnlocked):
 * 1. `app_startup_sequence(password)` 결과 `StartupResult` 를 `useSessionStore.markUnlocked` 저장
 * 2. `/` 로 redirect → 메인 화면 진입
 *
 * 강제 점유 성공 시 (LockWarning.onForceAcquired) → 락 상태 재조회 → LockScreen 으로 자동 전환.
 *
 * PRD §5.6 의 < 3000 ms 측정값은 루트 페이지에서 표시한다 (`StartupResult.elapsed_ms`).
 */

import { useCallback, useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { LockScreen } from '@/components/LockScreen'
import { LockWarning } from '@/components/LockWarning'
import { SplashScreen } from '@/components/splash-screen'
import { checkLockStatus } from '@/lib/tauri'
import { useSessionStore } from '@/stores/session-store'
import type { LockStatus, StartupResult } from '@/types'

export default function LockPage() {
  const router = useRouter()
  const markUnlocked = useSessionStore((s) => s.markUnlocked)
  const [lockStatus, setLockStatus] = useState<LockStatus | null>(null)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(() => {
    setError(null)
    setLockStatus(null)
    checkLockStatus()
      .then(setLockStatus)
      .catch((e) => setError(typeof e === 'string' ? e : '잠금 상태를 확인할 수 없습니다.'))
  }, [])

  useEffect(() => {
    refresh()
  }, [refresh])

  const handleUnlocked = (result: StartupResult) => {
    markUnlocked(result)
    router.replace('/')
  }

  if (error !== null) {
    return (
      <main className="flex min-h-screen items-center justify-center p-8">
        <div className="flex w-full max-w-md flex-col gap-4">
          <div
            role="alert"
            className="rounded-md border-2 border-[var(--danger)] bg-red-50 p-4 text-base text-[var(--danger)]"
          >
            {error}
          </div>
          <button
            type="button"
            onClick={refresh}
            className="h-[56px] w-full rounded-lg bg-[var(--accent)] text-lg font-semibold text-white hover:bg-[var(--accent-hover)]"
          >
            다시 시도
          </button>
        </div>
      </main>
    )
  }

  if (lockStatus === null) {
    return <SplashScreen message="잠금 상태를 확인하는 중입니다..." />
  }

  if (lockStatus.kind === 'owned-by-other') {
    return (
      <LockWarning
        initialSecondsAgo={lockStatus.last_heartbeat_seconds_ago}
        onForceAcquired={refresh}
        onRetry={refresh}
      />
    )
  }

  return <LockScreen onUnlocked={handleUnlocked} />
}
