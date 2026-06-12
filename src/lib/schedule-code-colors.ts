/**
 * 학사일정 시스템 코드 색상 SSOT (P2-13, 2026-06 코드리뷰).
 *
 * 기존에는 학사 캘린더 셀(`CalendarCell` Tailwind 배지) / 수업 캘린더(`ClassCalendar` inline hex) /
 * 공지문 달력 이미지(`calendar-image` canvas hex)가 각자 색을 정의해, 같은 코드(보강데이·
 * 공휴수업일)가 화면마다 다른 색으로 표시됐다. 본 모듈을 단일 출처로 삼아 한 곳만 고치면
 * 세 표현이 함께 바뀌도록 한다.
 *
 * 시스템 예약 코드는 V102 시드 6종. 신규 추가 시 본 객체에만 한 줄 추가.
 * 비시스템(사용자) 코드는 `USER_CODE_COLOR` 로 폴백.
 */

interface CodeColor {
  /** 캔버스/inline 스타일용 hex. */
  hex: string
  /** Tailwind 배지 클래스 (배경 + 텍스트). */
  badgeClass: string
}

export const SYSTEM_CODE_COLOR: Record<string, CodeColor> = {
  공휴일: { hex: '#dc2626', badgeClass: 'bg-red-100 text-red-800' },
  보강데이: { hex: '#0d9488', badgeClass: 'bg-teal-100 text-teal-800' },
  공휴수업일: { hex: '#db2777', badgeClass: 'bg-pink-100 text-pink-800' },
  방학: { hex: '#9333ea', badgeClass: 'bg-purple-100 text-purple-800' },
  휴원일: { hex: '#6b7280', badgeClass: 'bg-gray-200 text-gray-700' },
  '단원평가 응시일': { hex: '#2563eb', badgeClass: 'bg-blue-100 text-blue-800' },
}

/** 사용자 정의 코드(비시스템) 또는 매핑 누락 시 색. */
export const USER_CODE_COLOR: CodeColor = {
  hex: '#d97706',
  badgeClass: 'bg-amber-100 text-amber-800',
}

/** 코드명 + 시스템 여부 → 색. 시스템 코드는 lookup, 누락/사용자 코드는 amber 폴백. */
export function codeColor(codeName: string, isSystemReserved: boolean): CodeColor {
  if (!isSystemReserved) return USER_CODE_COLOR
  return SYSTEM_CODE_COLOR[codeName] ?? USER_CODE_COLOR
}
