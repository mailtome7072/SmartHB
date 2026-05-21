/**
 * 표시 포맷 유틸리티 (Sprint 4 T5 / 사용자 이슈 #4, #13).
 *
 * - 연락처: 숫자 입력 → 자동 하이픈 (휴대폰 / 지역번호 / 일반전화 패턴)
 * - 금액: 천단위 콤마 (ko-KR Intl.NumberFormat)
 *
 * 본 파일은 도메인 무관 순수 함수 모음. students 도메인 적용은 세션 #4 에서 T5 흡수.
 */

/**
 * 한국 전화번호를 자동 하이픈 포맷팅한다.
 *
 * 패턴:
 * - 휴대폰: 010 / 011 / 016 / 017 / 018 / 019 — 3-4-4
 * - 서울 지역번호: 02 — 2-3-4 또는 2-4-4
 * - 기타 지역번호: 031 ~ 064 등 3자리 — 3-3-4 또는 3-4-4
 * - 050x / 070 / 080 / 0505 등 특수번호: 일반 3-4-4 패턴 fallback
 *
 * 비숫자 문자는 모두 제거 후 자릿수 기반 분리.
 * 자릿수가 너무 짧거나 패턴이 안 맞으면 원본 그대로(숫자만) 반환.
 */
export function formatPhone(raw: string): string {
  const digits = raw.replace(/\D/g, '')

  if (digits.length === 0) return ''

  // 서울 02 (총 9~10자리)
  if (digits.startsWith('02')) {
    if (digits.length <= 2) return digits
    if (digits.length <= 5) return `${digits.slice(0, 2)}-${digits.slice(2)}`
    if (digits.length <= 9) {
      // 02-XXX-XXXX
      return `${digits.slice(0, 2)}-${digits.slice(2, 5)}-${digits.slice(5)}`
    }
    // 02-XXXX-XXXX
    return `${digits.slice(0, 2)}-${digits.slice(2, 6)}-${digits.slice(6, 10)}`
  }

  // 3자리 지역번호/특수번호 (031, 070, 010 등) — 총 10~11자리
  if (digits.length <= 3) return digits
  if (digits.length <= 7) return `${digits.slice(0, 3)}-${digits.slice(3)}`
  if (digits.length <= 10) {
    // XXX-XXX-XXXX (10자리)
    return `${digits.slice(0, 3)}-${digits.slice(3, 6)}-${digits.slice(6)}`
  }
  // XXX-XXXX-XXXX (11자리)
  return `${digits.slice(0, 3)}-${digits.slice(3, 7)}-${digits.slice(7, 11)}`
}

/** 금액(원)을 천단위 콤마로 포맷팅. NaN/Infinity 는 빈 문자열 반환. */
export function formatCurrency(amount: number): string {
  if (!Number.isFinite(amount)) return ''
  return new Intl.NumberFormat('ko-KR').format(amount)
}
