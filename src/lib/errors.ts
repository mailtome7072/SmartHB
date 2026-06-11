/**
 * IPC 에러 → 사용자 표시 메시지 추출 (P1-7, 2026-06 코드리뷰).
 *
 * Tauri 커맨드는 `Result<T, String>` 이라 reject 값이 **plain string** 이다.
 * `e instanceof Error ? e.message : fallback` 패턴은 백엔드가 보낸 한국어 사유를
 * 버리고 generic 메시지를 띄웠다 — string / Error / 기타를 모두 수용한다.
 */
export function errMsg(e: unknown, fallback: string): string {
  if (typeof e === 'string' && e !== '') return e
  if (e instanceof Error && e.message !== '') return e.message
  return fallback
}
