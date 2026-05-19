/**
 * Tauri IPC 추상화 레이어
 * 컴포넌트에서 invoke() 직접 호출 금지 — 이 파일을 통해서만 Tauri 커맨드 호출
 */

import type { AuthStatus, LockStatus } from '@/types'

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

/**
 * 12자리 복구 코드를 발급한다 (PI-07 PRD v1.5.1).
 *
 * 평문 코드는 호출 직후 화면에 1회 표시하고 즉시 폐기해야 한다 — React state 보유는 표시
 * 중에만, 사용자가 "확인" 클릭 시 빈 문자열로 덮어쓰기 권장.
 *
 * 이미 발급된 코드가 있으면 무효화하고 새 코드를 반환 (재발급 정책).
 */
export async function generateRecoveryCode(): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return 'DEVM-ODEX-TEST'
  return inv('generate_recovery_code') as Promise<string>
}

/**
 * 사용자가 입력한 복구 코드를 검증한다 (constant-time).
 *
 * 공백·하이픈은 백엔드에서 자동 제거되며 대문자로 통일된다.
 */
export async function verifyRecoveryCode(code: string): Promise<boolean> {
  const inv = await getInvoke()
  if (!inv) return code.replace(/[-\s]/g, '').toUpperCase() === 'DEVMODEXTEST'
  return inv('verify_recovery_code', { code }) as Promise<boolean>
}

/**
 * 복구 코드로 비밀번호를 재설정한다.
 *
 * 코드 검증 실패 시 throw. 성공 시 keyring 의 salt + key 가 새 비밀번호로 갱신된다.
 * SQLCipher DB rekey 는 T9 통합 시점에 추가된다.
 */
export async function resetPasswordWithCode(code: string, newPassword: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('reset_password_with_code', { code, newPassword })
}

/**
 * 현재 app.lock 점유 상태를 조회한다 (T6 PRD §5.3).
 *
 * 브라우저 개발 모드에서는 `free` 를 반환하여 UI 흐름만 검증 가능.
 */
export async function checkLockStatus(): Promise<LockStatus> {
  const inv = await getInvoke()
  if (!inv) return { kind: 'free' }
  return inv('check_lock_status') as Promise<LockStatus>
}

/**
 * app.lock 점유를 시도한다.
 *
 * `force=true` 는 5분 이상 미갱신(stale) 락만 강제 점유 — 정상 동작 중인 다른 디바이스
 * 락은 백엔드가 보호한다. UI 가 사전 사용자 확인 후 force=true 호출.
 */
export async function acquireLock(force: boolean): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('acquire_lock', { force })
}

/**
 * 본 디바이스의 app.lock 을 해제한다 (다른 디바이스 락은 보호됨).
 */
export async function releaseLock(): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('release_lock')
}
