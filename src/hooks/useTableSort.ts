'use client'

/**
 * 클라이언트 사이드 테이블 정렬 공통 훅 (Sprint 19 T1, 사용자 요청 1·2번).
 *
 * 서버 페이지네이션이 있는 원생 목록(`students/page.tsx`)은 백엔드 `StudentSort` enum으로
 * 정렬하므로 이 훅을 쓰지 않는다. 이 훅은 출결/청구 그리드처럼 전체 데이터를 한 번에 받아
 * 클라이언트에서 정렬하는 화면(T2/T3)을 위한 것이다.
 */

import { useMemo, useState } from 'react'

export type SortDirection = 'asc' | 'desc'

export interface SortState<K extends string> {
  key: K
  direction: SortDirection
}

/** withTiebreak가 반환하는 비교 함수 — 내림차순 시 주 기준만 뒤집고 tiebreak는
 *  항상 오름차순으로 유지하기 위해 원본 두 함수를 함께 태깅해둔다. */
type TiebreakCompare<T> = ((a: T, b: T) => number) & {
  primary: (a: T, b: T) => number
  tiebreak: (a: T, b: T) => number
}

/**
 * 비교 함수에 tie-break 비교 함수를 덧붙인다.
 *
 * 사용자 요청 2번: "동일 학년(또는 동일 값) 정렬의 경우 원생 이름은 자동 가나다순으로 함께
 * 정렬" — 각 컬럼 comparator를 이 헬퍼로 감싸면 어떤 컬럼을 정렬해도 동일 값 내에서는
 * 항상 이름순으로 2차 정렬된다. `primary`/`tiebreak`를 함수에 태깅해두는 이유는
 * useTableSort가 내림차순 적용 시 tiebreak는 뒤집지 않고 주 기준만 뒤집기 위함
 * (Sprint 19 sprint-review F2 — 단순 reverse()는 tiebreak까지 뒤집는 버그였음).
 */
export function withTiebreak<T>(
  compare: (a: T, b: T) => number,
  tiebreak: (a: T, b: T) => number,
): TiebreakCompare<T> {
  const fn = ((a: T, b: T) => compare(a, b) || tiebreak(a, b)) as TiebreakCompare<T>
  fn.primary = compare
  fn.tiebreak = tiebreak
  return fn
}

/** 한글(가나다순) 안전 비교 — Intl.Collator 캐싱으로 매 호출 생성 비용 회피. */
const KOREAN_COLLATOR = new Intl.Collator('ko')
export function compareKorean(a: string, b: string): number {
  return KOREAN_COLLATOR.compare(a, b)
}

/**
 * `comparators`는 호출부에서 모듈 최상단 상수로 선언해 넘길 것 — 렌더마다 새
 * 객체 리터럴을 만들어 넘기면 `useMemo`가 매 렌더 무효화된다.
 */
export function useTableSort<T, K extends string>(
  data: T[],
  comparators: Record<K, (a: T, b: T) => number>,
  defaultSort: SortState<K>,
) {
  const [sort, setSort] = useState<SortState<K>>(defaultSort)

  const sorted = useMemo(() => {
    const compare = comparators[sort.key]
    if (sort.direction === 'asc') return [...data].sort(compare)
    // 내림차순 — withTiebreak로 만든 비교 함수라면 주 기준만 뒤집고 tiebreak(이름순)는
    // 그대로 유지한다. 단순 `.reverse()`는 tiebreak까지 뒤집어 동일 값 그룹의 이름
    // 정렬이 역순으로 보이는 버그가 있었다(Sprint 19 sprint-review F2).
    const tagged = compare as Partial<TiebreakCompare<T>>
    if (tagged.primary && tagged.tiebreak) {
      const { primary, tiebreak } = tagged
      return [...data].sort((a, b) => {
        const p = primary(a, b)
        return p !== 0 ? -p : tiebreak(a, b)
      })
    }
    return [...data].sort(compare).reverse()
  }, [data, sort, comparators])

  function toggleSort(key: K) {
    setSort((cur) =>
      cur.key === key
        ? { key, direction: cur.direction === 'asc' ? 'desc' : 'asc' }
        : { key, direction: 'asc' },
    )
  }

  function indicator(key: K): string {
    if (sort.key !== key) return ''
    return sort.direction === 'asc' ? ' ▲' : ' ▼'
  }

  return { sorted, sort, toggleSort, indicator }
}
