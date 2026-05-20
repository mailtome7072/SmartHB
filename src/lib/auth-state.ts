/**
 * 인증 상태 — 모듈 스코프 단순 변수 (Sprint 2 T1).
 *
 * Sprint 3 에서 Zustand 도입 시 본 파일이 store 어댑터로 교체된다. 그 사이 사용자가 잠금
 * 해제에 성공한 사실과 마지막 `app_startup_sequence` 결과를 메모리에만 보관한다.
 *
 * ## 보안
 *
 * - 비밀번호·키·복구 코드는 절대 저장하지 않는다 (백엔드 Keychain·DB가 책임).
 * - sessionStorage 도 사용하지 않는다 — 페이지 새로고침 시 메모리 초기화되어 잠금 화면 재진입.
 * - `StartupResult` 만 보관 — `elapsed_ms` 등 측정 정보만 포함되어 민감하지 않다.
 */

import type { StartupResult } from '@/types'

let unlocked = false
let lastStartup: StartupResult | null = null

/** 본 세션에서 잠금 해제가 완료되었는지 여부. 새로고침 시 false 로 초기화. */
export function isUnlocked(): boolean {
  return unlocked
}

/** 잠금 해제 + startup 시퀀스 성공 시 호출 — LockScreen onUnlocked 콜백에서 사용. */
export function markUnlocked(result: StartupResult): void {
  unlocked = true
  lastStartup = result
}

/** 마지막 startup 측정 결과 — 디버그·관리자 화면에서 참조 가능. */
export function getLastStartup(): StartupResult | null {
  return lastStartup
}
