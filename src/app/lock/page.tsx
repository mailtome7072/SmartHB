'use client'

/**
 * `/lock` 라우트 — 잠금 화면.
 *
 * Sprint 2 T1 에서 `onUnlocked` 콜백을 연결하여 인증 성공 시:
 * 1. `LockScreen` 내부에서 `app_startup_sequence(password)` 가 호출되어 DB pool 초기화·
 *    백그라운드 task spawn·audit 정리가 일괄 수행되고
 * 2. 그 결과 `StartupResult` 를 받아 `auth-state` 에 저장한 뒤
 * 3. `useRouter().replace('/')` 로 메인 화면에 진입한다.
 *
 * PRD §5.6 의 < 3000 ms 측정값은 루트 페이지에서 표시한다 (`StartupResult.elapsed_ms`).
 */

import { useRouter } from 'next/navigation'
import { LockScreen } from '@/components/LockScreen'
import { markUnlocked } from '@/lib/auth-state'
import type { StartupResult } from '@/types'

export default function LockPage() {
  const router = useRouter()

  const handleUnlocked = (result: StartupResult) => {
    markUnlocked(result)
    router.replace('/')
  }

  return <LockScreen onUnlocked={handleUnlocked} />
}
