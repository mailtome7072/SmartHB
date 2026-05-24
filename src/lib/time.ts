/**
 * 시간 단위 변환 유틸 (Sprint 9 Session #10 I1).
 *
 * 백엔드는 `class_minutes` 정수(분 단위) 유지. UI 표시/입력은 시간 단위로 변환한다.
 * - 분 → 시간: 60 으로 나눔. 정수 시간이면 `"1"`, 소수면 `"1.5"` 형태.
 * - 시간 → 분: 60 곱한 후 round (사용자가 1.5h 입력 → 90분).
 *
 * 사용자 요구 (시각 검증 2026-05-24): "보강 관리의 모든 보강대상 및 보강진행 단위는 시간".
 */

/** 분(정수) → 시간(decimal, 표시용 문자열). 60 미만은 소수, 60 이상은 정수 우선. */
export function minutesToHours(minutes: number): number {
  return minutes / 60
}

/** 시간(decimal) → 분(정수, 반올림). 음수/NaN 은 0 반환. */
export function hoursToMinutes(hours: number): number {
  if (!Number.isFinite(hours) || hours <= 0) return 0
  return Math.round(hours * 60)
}

/**
 * 시간을 사용자 친화적으로 포맷한다 — 정수면 "1", 소수면 "1.5" / "0.5".
 * 0 또는 음수는 "0".
 */
export function formatHours(hours: number): string {
  if (!Number.isFinite(hours) || hours <= 0) return '0'
  // 정수면 그대로, 소수면 첫째자리까지
  if (Number.isInteger(hours)) return hours.toString()
  return hours.toFixed(1).replace(/\.0$/, '')
}

/** 분 → 표시용 시간 문자열 ("1.5") — 짧은 헬퍼. */
export function minutesToHoursText(minutes: number): string {
  return formatHours(minutesToHours(minutes))
}
