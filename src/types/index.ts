/**
 * 공유 TypeScript 타입 정의
 * Tauri 커맨드 요청/응답 타입은 여기에 정의
 */

export interface ApiResult<T> {
  data: T
  error?: string
}

/**
 * 사용자 인증 상태.
 *
 * `src-tauri/src/commands/auth.rs::AuthStatus` 와 serde rename_all="kebab-case" 정합.
 * Rust enum 의 `NotInitialized` / `Locked` 가 각각 `"not-initialized"` / `"locked"` 로
 * 직렬화되어 IPC 응답으로 전달된다.
 *
 * `Unlocked` 는 메모리 상태로만 관리되어 IPC 응답에 포함되지 않으므로 본 타입에 없다.
 */
export type AuthStatus = 'not-initialized' | 'locked'

/**
 * app.lock 점유 상태 — T6 PRD §5.3.
 *
 * `src-tauri/src/commands/lock.rs::LockStatus` 와 serde `tag = "kind"` + `rename_all = "kebab-case"` 정합.
 * - `free`: 락 파일 없음, 즉시 점유 가능
 * - `owned-by-self`: 본 디바이스가 점유 중
 * - `owned-by-other`: 다른 디바이스 점유 중 (`stale: true` 면 5분 이상 미갱신 — 강제 점유 가능)
 */
export type LockStatus =
  | { kind: 'free' }
  | { kind: 'owned-by-self'; last_heartbeat_seconds_ago: number }
  | { kind: 'owned-by-other'; stale: boolean; last_heartbeat_seconds_ago: number }
