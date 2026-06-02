'use client'

/**
 * 글로벌 검색바 (Sprint 3 T6, PRD §4.14).
 *
 * 모든 화면 상단에 상시 노출. 원생 이름·메뉴명을 한글 자모 부분 일치로 검색하여
 * 결과 클릭 시 해당 화면으로 1클릭 이동.
 *
 * - 검색 대상: 활성 메뉴(`ACTIVE_MENU_ITEMS`) + TanStack Query 가 캐싱한 listStudents 응답
 * V19 (Sprint 7 post-review): 단축키 (Ctrl+F / Ctrl+/) 제거 — 50대 사용자 친화 UX 단순화. ESC 클리어만 유지.
 *   preventDefault 로 입력 포커스만 빼앗는다.
 * - 디바운스: `useDeferredValue` (React 18+) 가 입력 후 다음 idle 까지 갱신 지연 — 200ms
 *   타이머 수동 관리 불필요.
 * - 분해 캐싱: students 의 분해 결과는 `useMemo` 로 1회 계산. ~100명 × 키 입력마다
 *   재계산되던 비용 제거.
 */

import { useDeferredValue, useEffect, useMemo, useRef, useState } from 'react'
import { useRouter } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import { listStudents } from '@/lib/tauri'
import { decompose } from '@/lib/hangul-search'
import { ACTIVE_MENU_ITEMS } from '@/lib/menu-config'
import { guardedNavigate } from '@/stores/app-store'
import type { Student } from '@/types/student'

interface SearchHit {
  label: string
  sublabel?: string
  href: string
}

const MAX_STUDENT_HITS = 10

export function GlobalSearch() {
  const router = useRouter()
  const inputRef = useRef<HTMLInputElement | null>(null)
  const listRef = useRef<HTMLUListElement | null>(null)
  const [query, setQuery] = useState('')
  const debouncedQuery = useDeferredValue(query)
  // 방향키로 선택 이동하는 활성 결과 인덱스.
  const [activeIndex, setActiveIndex] = useState(0)
  // 한글 IME 조합 상태 + 조합 중 눌린 Enter 보류 플래그 (조합 확정 후 선택 처리).
  const composingRef = useRef(false)
  const pendingEnterRef = useRef(false)
  // 비동기 compositionend 핸들러에서 최신 hits/activeIndex 를 읽기 위한 ref 미러.
  const hitsRef = useRef<SearchHit[]>([])
  const activeIndexRef = useRef(0)

  // V19 (Sprint 7 post-review): 단축키 핸들러 제거. ESC 만 활성 상태일 때 검색어 클리어 유지.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        setQuery('')
        inputRef.current?.blur()
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  const { data: students = [] } = useQuery<Student[]>({
    queryKey: ['students', 'global-search'],
    queryFn: () => listStudents({ active_only: true, limit: 1000 }),
  })

  const decomposedMenus = useMemo(
    () => ACTIVE_MENU_ITEMS.map((m) => ({ item: m, key: decompose(m.label) })),
    [],
  )
  const decomposedStudents = useMemo(
    () => students.map((s) => ({ student: s, key: decompose(s.name) })),
    [students],
  )

  const hits: SearchHit[] = useMemo(() => {
    if (debouncedQuery.length === 0) return []
    const needle = decompose(debouncedQuery)
    const menuHits: SearchHit[] = decomposedMenus
      .filter((m) => m.key.includes(needle))
      .map((m) => ({ label: m.item.label, sublabel: '메뉴', href: m.item.href }))
    const studentHits: SearchHit[] = decomposedStudents
      .filter((s) => s.key.includes(needle))
      .slice(0, MAX_STUDENT_HITS)
      .map((s) => ({
        label: s.student.name,
        sublabel: `원생 #${s.student.serial_no}`,
        // 원생 상세 라우트(/students/[id])는 없음 — 편집 라우트(쿼리 기반)로 이동 (원생 목록과 동일).
        href: `/students/edit?id=${s.student.id}`,
      }))
    return [...menuHits, ...studentHits]
  }, [debouncedQuery, decomposedMenus, decomposedStudents])

  const open = debouncedQuery.length > 0

  // 결과가 바뀌면 활성 인덱스를 첫 항목으로 초기화.
  useEffect(() => {
    setActiveIndex(0)
  }, [debouncedQuery, hits.length])

  // 활성 항목이 보이도록 스크롤.
  useEffect(() => {
    if (!open) return
    listRef.current?.querySelector<HTMLElement>(`[data-idx="${activeIndex}"]`)?.scrollIntoView({
      block: 'nearest',
    })
  }, [activeIndex, open])

  // 비동기 compositionend 에서 최신 값 참조용 미러 (렌더마다 동기화).
  hitsRef.current = hits
  activeIndexRef.current = activeIndex

  const selectHit = (hit: SearchHit) => {
    // 미저장 가드(공지문 편집 등)를 거쳐 이동.
    guardedNavigate(hit.href, () => router.push(hit.href))
    setQuery('')
  }

  // 입력창에서 ↑/↓ 로 결과 이동, Enter 로 활성 항목 선택.
  // 한글 IME 조합 중에는 키가 IME 로 전달되므로(WebKit), 조합 상태를 인지해 처리한다.
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!open || hits.length === 0) return
    const composing = e.nativeEvent.isComposing || composingRef.current
    if (e.key === 'ArrowDown') {
      if (composing) return // 조합 중엔 IME 양보 — 확정 후 다시 누르면 이동
      e.preventDefault()
      setActiveIndex((i) => (i + 1) % hits.length)
    } else if (e.key === 'ArrowUp') {
      if (composing) return
      e.preventDefault()
      setActiveIndex((i) => (i - 1 + hits.length) % hits.length)
    } else if (e.key === 'Enter') {
      if (composing) {
        // 조합 확정용 Enter — IME 가 commit 하도록 두고, compositionend 에서 선택 처리.
        pendingEnterRef.current = true
        return
      }
      e.preventDefault()
      const hit = hits[activeIndex]
      if (hit) selectHit(hit)
    }
  }

  const handleCompositionStart = () => {
    composingRef.current = true
  }
  const handleCompositionEnd = () => {
    composingRef.current = false
    // 조합 중 Enter 를 눌렀다면, 확정 직후 현재 활성 결과를 선택 (한 번의 Enter 로 선택 완료).
    if (pendingEnterRef.current) {
      pendingEnterRef.current = false
      const hit = hitsRef.current[activeIndexRef.current]
      if (hit) selectHit(hit)
    }
  }

  return (
    <div className="relative mx-auto w-full max-w-md">
      <input
        ref={inputRef}
        type="search"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={handleKeyDown}
        onCompositionStart={handleCompositionStart}
        onCompositionEnd={handleCompositionEnd}
        placeholder="원생·메뉴 검색"
        aria-label="글로벌 검색"
        role="combobox"
        aria-expanded={open && hits.length > 0}
        aria-controls="global-search-listbox"
        aria-activedescendant={open && hits.length > 0 ? `global-search-opt-${activeIndex}` : undefined}
        autoComplete="off"
        className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3 text-base focus:border-[var(--accent)] focus:outline-none"
      />
      {open && (
        <ul
          ref={listRef}
          id="global-search-listbox"
          role="listbox"
          className="absolute left-0 right-0 top-12 z-10 max-h-80 overflow-y-auto rounded-md border border-[var(--border)] bg-white shadow-lg"
        >
          {hits.length === 0 ? (
            <li className="px-3 py-3 text-sm text-gray-500">검색 결과가 없습니다.</li>
          ) : (
            hits.map((hit, i) => (
              <li key={`${hit.label}-${hit.href}`}>
                <button
                  type="button"
                  id={`global-search-opt-${i}`}
                  data-idx={i}
                  role="option"
                  aria-selected={i === activeIndex}
                  onClick={() => selectHit(hit)}
                  onMouseEnter={() => setActiveIndex(i)}
                  className={`flex min-h-[44px] w-full items-center justify-between px-3 py-2 text-left ${
                    i === activeIndex ? 'bg-[var(--background)]' : ''
                  }`}
                >
                  <span>{hit.label}</span>
                  {hit.sublabel !== undefined && (
                    <span className="text-sm text-gray-500">{hit.sublabel}</span>
                  )}
                </button>
              </li>
            ))
          )}
        </ul>
      )}
    </div>
  )
}
