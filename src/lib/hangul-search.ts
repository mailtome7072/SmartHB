/**
 * 한글 자모 부분 일치 검색 헬퍼 (Sprint 3 T6, PRD §4.14).
 *
 * 외부 라이브러리(hangul-js 등) 대신 짧은 자체 구현 — 한글 음절(가–힣)을 자모로 분해한 뒤
 * 분해된 문자열에 대해 부분 일치(`includes`) 를 수행한다. 영문은 대소문자 무관.
 *
 * 검색 대상이 50대 원장 1인의 원생 ~100명 + 메뉴 ~10개 + 학교 ~수십개로 작아 O(n) 선형
 * 스캔으로 충분. 별도 인덱스·트라이 미사용.
 *
 * ## 예시
 *
 * - `decompose('홍길동')` → `'ㅎㅗㅇㄱㅣㄹㄷㅗㅇ'`
 * - `decompose('ABC')` → `'abc'` (영문은 lowercase)
 * - `matches('홍길동', 'ㅎㄱㄷ')` → `true` (초성 부분 일치)
 * - `matches('Test', 'TES')` → `true`
 */

const CHO = [
  'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ',
  'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
]

const JUNG = [
  'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ',
  'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
]

const JONG = [
  '', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ',
  'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ',
  'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
]

const SYLLABLE_BASE = 0xac00
const SYLLABLE_LAST = 0xd7a3
const JUNG_COUNT = JUNG.length
const JONG_COUNT = JONG.length

/** 한글·영문 혼합 문자열을 자모 + lowercase 형태로 분해한다. */
export function decompose(text: string): string {
  let out = ''
  for (const ch of text) {
    const code = ch.codePointAt(0)
    if (code !== undefined && code >= SYLLABLE_BASE && code <= SYLLABLE_LAST) {
      const offset = code - SYLLABLE_BASE
      const cho = Math.floor(offset / (JUNG_COUNT * JONG_COUNT))
      const jung = Math.floor((offset % (JUNG_COUNT * JONG_COUNT)) / JONG_COUNT)
      const jong = offset % JONG_COUNT
      out += CHO[cho] + JUNG[jung] + JONG[jong]
    } else {
      out += ch.toLowerCase()
    }
  }
  return out
}

/** `query` 가 `target` 의 분해 형태에 부분 일치하는지 여부. */
export function matches(target: string, query: string): boolean {
  if (query.length === 0) return true
  return decompose(target).includes(decompose(query))
}
