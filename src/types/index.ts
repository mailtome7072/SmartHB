/**
 * 공유 TypeScript 타입 정의
 * Tauri 커맨드 요청/응답 타입은 여기에 정의
 */

export interface ApiResult<T> {
  data: T
  error?: string
}
