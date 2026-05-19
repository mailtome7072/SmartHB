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
