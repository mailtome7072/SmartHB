'use client'

/**
 * 루트 페이지 (Sprint 2 T1) — 인증 게이트.
 *
 * `checkAuthStatus()` IPC 호출로 분기:
 * - `not-initialized` → `/lock?mode=setup` redirect (최초 비밀번호 설정)
 * - `locked` → `/lock` redirect (잠금 해제)
 * - 본 세션에서 이미 잠금 해제됨 (`auth-state.isUnlocked()`) → 메인 화면 표시
 *
 * PRD §5.6 인수 기준 "최초 실행 시 비밀번호 설정 화면 자동 진입" 충족.
 *
 * 메인 화면 자체는 후속 sprint 에서 대시보드 등으로 채워진다.
 */

import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { checkAuthStatus } from '@/lib/tauri'
import { getLastStartup, isUnlocked } from '@/lib/auth-state'

export default function Home() {
  const router = useRouter()
  const [ready, setReady] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (isUnlocked()) {
      setReady(true)
      return
    }
    let cancelled = false
    checkAuthStatus()
      .then((status) => {
        if (cancelled) return
        const target = status === 'not-initialized' ? '/lock?mode=setup' : '/lock'
        router.replace(target)
      })
      .catch((e: unknown) => {
        if (cancelled) return
        setError(typeof e === 'string' ? e : '인증 상태를 확인할 수 없습니다.')
      })
    return () => {
      cancelled = true
    }
  }, [router])

  if (error !== null) {
    return (
      <main className="flex min-h-screen items-center justify-center p-8">
        <div
          role="alert"
          className="max-w-md rounded-md border-2 border-[var(--danger)] bg-red-50 p-4 text-base text-[var(--danger)]"
        >
          {error}
        </div>
      </main>
    )
  }

  if (!ready) {
    return (
      <main className="flex min-h-screen items-center justify-center">
        <p className="text-lg text-gray-600">불러오는 중...</p>
      </main>
    )
  }

  const startup = getLastStartup()
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <h1 className="mb-4 text-4xl font-bold">스마트해법수학</h1>
      <p className="mb-8 text-lg text-gray-600">정쌤의 교습소 관리 시스템</p>
      {startup !== null && (
        <p className="text-sm text-gray-500">
          시작 시간: {startup.elapsed_ms} ms
          {startup.elapsed_ms > 3000 && ' (PRD §5.6 < 3000 ms 초과 — 환경 점검 권장)'}
        </p>
      )}
      <p className="mt-8 text-sm text-gray-500">메인 대시보드는 후속 sprint 에서 구축됩니다.</p>
    </main>
  )
}
