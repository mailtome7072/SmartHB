/**
 * Tauri IPC 추상화 레이어
 * 컴포넌트에서 invoke() 직접 호출 금지 — 이 파일을 통해서만 Tauri 커맨드 호출
 */

import type { AuthStatus } from '@/types'

let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null

async function getInvoke() {
  if (typeof window === 'undefined') return null
  if (!invoke) {
    try {
      const tauri = await import('@tauri-apps/api/core')
      invoke = tauri.invoke
    } catch {
      // 브라우저 환경 (Tauri 없이 실행 시) — 개발용 mock 가능
      invoke = null
    }
  }
  return invoke
}

export async function greet(name: string): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return `[개발 모드] 안녕하세요, ${name}!`
  return inv('greet', { name }) as Promise<string>
}

/**
 * 현재 인증 상태를 조회한다.
 *
 * - `'not-initialized'`: 비밀번호 미설정 — 최초 설정 모드 진입
 * - `'locked'`: 비밀번호 설정됨, 잠금 해제 모드 진입
 *
 * 브라우저 개발 모드(Tauri 없이)에서는 `'not-initialized'` 를 반환하여 UI 흐름 테스트 가능.
 */
export async function checkAuthStatus(): Promise<AuthStatus> {
  const inv = await getInvoke()
  if (!inv) return 'not-initialized'
  return inv('check_auth_status') as Promise<AuthStatus>
}

/**
 * 최초 비밀번호를 설정한다 (NotInitialized → Locked 전이).
 *
 * 이미 설정되어 있으면 백엔드에서 에러 반환 → throw.
 */
export async function setPassword(password: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) {
    // 개발 모드 — no-op
    return
  }
  await inv('set_password', { password })
}

/**
 * 비밀번호로 DB 잠금을 해제한다 (Locked → Unlocked 전이).
 *
 * 비밀번호 불일치 시 throw — 호출자가 catch 하여 사용자 친화 메시지 표시.
 */
export async function unlockDb(password: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) {
    // 개발 모드 — 어떤 비밀번호든 허용
    return
  }
  await inv('unlock_db', { password })
}
