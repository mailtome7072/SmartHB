'use client'

/**
 * 글로벌 검색바 (Sprint 3 T6, PRD §4.14).
 *
 * 모든 화면 상단에 상시 노출. 원생 이름·메뉴명을 한글 자모 부분 일치로 검색하여
 * 결과 클릭 시 해당 화면으로 1클릭 이동.
 *
 * - 검색 대상: 활성 메뉴(`ACTIVE_MENU_ITEMS`) + TanStack Query 가 캐싱한 listStudents 응답
 * - 단축키: Ctrl+F / Ctrl+/ — Tauri WebView 환경이라 브라우저 기본 검색은 의미 없음.
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
  const [query, setQuery] = useState('')
  const debouncedQuery = useDeferredValue(query)

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === '/')) {
        e.preventDefault()
        inputRef.current?.focus()
      }
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
        href: `/students/${s.student.id}`,
      }))
    return [...menuHits, ...studentHits]
  }, [debouncedQuery, decomposedMenus, decomposedStudents])

  const open = debouncedQuery.length > 0

  return (
    <div className="relative mx-auto w-full max-w-md">
      <input
        ref={inputRef}
        type="search"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="원생·메뉴 검색 (Ctrl+F)"
        aria-label="글로벌 검색"
        className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3 text-base focus:border-[var(--accent)] focus:outline-none"
      />
      {open && (
        <ul
          role="listbox"
          className="absolute left-0 right-0 top-12 z-10 max-h-80 overflow-y-auto rounded-md border border-[var(--border)] bg-white shadow-lg"
        >
          {hits.length === 0 ? (
            <li className="px-3 py-3 text-sm text-gray-500">검색 결과가 없습니다.</li>
          ) : (
            hits.map((hit) => (
              <li key={`${hit.label}-${hit.href}`}>
                <button
                  type="button"
                  onClick={() => {
                    router.push(hit.href)
                    setQuery('')
                  }}
                  className="flex min-h-[44px] w-full items-center justify-between px-3 py-2 text-left hover:bg-[var(--background)]"
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
