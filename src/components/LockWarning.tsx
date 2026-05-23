'use client'

/**
 * 다른 디바이스가 app.lock 을 점유 중일 때 표시되는 경고 화면 (T6 PRD §5.3).
 *
 * 흐름:
 * - 5분 미만 미갱신: 강제 점유 비활성. "다른 PC 종료 후 다시 시도" 안내 + 타이머 표시
 * - 5분 이상 미갱신(stale=true): 강제 점유 버튼 활성 — 사용자 확인 후 `force=true` 호출
 *
 * 타이머는 1초마다 last_heartbeat 경과 시간을 갱신 (백엔드 재조회 없이 클라이언트 측 계산).
 * 5분 임계 도달 시 강제 점유 버튼이 자동 활성화된다.
 */

import { useEffect, useState } from 'react'
import { acquireLock, checkLockStatus } from '@/lib/tauri'

const STALE_THRESHOLD_SECONDS = 300

interface LockWarningProps {
  /**
   * 백엔드 `check_lock_status` 응답의 초기 경과 시간 (서버 측 측정).
   * 컴포넌트는 이 값을 baseline 으로 1초마다 증가시킨다.
   */
  initialSecondsAgo: number
  /** 강제 점유 성공 후 호출 — 보통 메인 화면 라우팅. */
  onForceAcquired: () => void
  /** 사용자 재시도 클릭 시 호출 — 백엔드 상태를 재조회. */
  onRetry: () => void
}

export function LockWarning({ initialSecondsAgo, onForceAcquired, onRetry }: LockWarningProps) {
  const [elapsed, setElapsed] = useState(initialSecondsAgo)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 부모가 LockWarning 을 unmount 하지 않고 새 initialSecondsAgo 만 전달하는 케이스
  // (예: 사용자 "다시 시도" → check_lock_status 결과로 props 만 갱신) 에 대응.
  useEffect(() => {
    setElapsed(initialSecondsAgo)
  }, [initialSecondsAgo])

  useEffect(() => {
    const timer = setInterval(() => setElapsed((s) => s + 1), 1000)
    return () => clearInterval(timer)
  }, [])

  // V30 (Sprint 7 post-review): dev 빌드 자동 force-acquire 우회 — LockScreen 의 autologin 과
  // 동일 환경 변수. release 빌드에서는 NEXT_PUBLIC 자체가 없어 무동작.
  useEffect(() => {
    if (!process.env.NEXT_PUBLIC_DEV_AUTOLOGIN) return
    void (async () => {
      try {
        await acquireLock(true)
        const status = await checkLockStatus()
        if (status.kind === 'owned-by-self') onForceAcquired()
      } catch {
        // dev 자동 우회 실패 시 사용자 수동 처리로 fallback — 에러 표시 안 함.
      }
    })()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const isStale = elapsed >= STALE_THRESHOLD_SECONDS
  const remainingToStale = Math.max(0, STALE_THRESHOLD_SECONDS - elapsed)

  const handleForce = async () => {
    setError(null)
    setSubmitting(true)
    try {
      await acquireLock(true)
      // 강제 점유 후 상태 재확인 — 정말 우리가 점유했는지
      const status = await checkLockStatus()
      if (status.kind === 'owned-by-self') {
        onForceAcquired()
      } else {
        setError('강제 점유에 실패했습니다. 잠시 후 다시 시도해주세요.')
      }
    } catch (e) {
      setError(typeof e === 'string' ? e : '강제 점유 중 오류가 발생했습니다.')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center px-4">
      <div className="w-full max-w-md space-y-6">
        <header className="space-y-2 text-center">
          <p className="text-5xl" aria-hidden="true">
            ⚠️
          </p>
          <h1 className="text-3xl font-bold">다른 컴퓨터에서 사용 중</h1>
          <p className="text-base text-gray-700">
            다른 컴퓨터에서 이 프로그램을 사용 중입니다.
          </p>
        </header>

        <section className="rounded-lg border-2 border-[var(--border)] bg-white p-4 text-center">
          <p className="text-sm text-gray-500">마지막 활동</p>
          <p className="font-mono text-2xl font-semibold">{formatDuration(elapsed)} 전</p>
        </section>

        {!isStale && (
          <section className="rounded-md border-2 border-blue-200 bg-blue-50 p-4 text-sm text-blue-900">
            <p className="font-medium">다른 컴퓨터에서 프로그램을 종료한 후 다시 시도해주세요.</p>
            <p className="mt-2 text-blue-800">
              {formatDuration(remainingToStale)} 이후에는 강제로 점유할 수 있습니다.
            </p>
          </section>
        )}

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
            type="button"
            onClick={onRetry}
            disabled={submitting}
            className="h-[56px] w-full rounded-lg bg-[var(--accent)] text-lg font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            다시 시도
          </button>
          <button
            type="button"
            onClick={handleForce}
            disabled={!isStale || submitting}
            className="h-[44px] w-full rounded-lg border-2 border-[var(--danger)] text-base font-medium text-[var(--danger)] hover:bg-red-50 disabled:opacity-40 disabled:hover:bg-transparent"
          >
            {submitting ? '확인 중...' : '강제로 점유 (위험)'}
          </button>
        </div>

        <section className="rounded-md bg-gray-50 p-3 text-sm text-gray-600">
          <p>
            <strong>강제 점유 시 주의:</strong> 다른 컴퓨터에서 작업 중인 내용이 손실될 수
            있습니다. 5분 이상 활동이 없을 때만 사용해주세요.
          </p>
        </section>
      </div>
    </main>
  )
}

/** 초 단위 시간을 사용자 친화 형식("X분 Y초", "Y초")으로 변환. */
function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}초`
  const minutes = Math.floor(seconds / 60)
  const remainder = seconds % 60
  return remainder === 0 ? `${minutes}분` : `${minutes}분 ${remainder}초`
}
