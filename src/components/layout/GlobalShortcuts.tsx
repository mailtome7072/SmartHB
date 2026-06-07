'use client'

/**
 * 전역 키보드 단축키 (Sprint 15 T3, PRD §5.7).
 *
 * - Ctrl/⌘ + F : 글로벌 검색바 포커스 (#global-search-input)
 * - Ctrl/⌘ + N : 신규 원생 등록 화면 이동
 *
 * 이미 동작하는 것은 별도 구현하지 않는다:
 * - ESC(다이얼로그 닫기): Radix(shadcn) Dialog/AlertDialog 기본 동작
 * - Ctrl+Z(Undo): 출결 그리드 국소 구현(AttendanceGrid)
 * - Ctrl+P(인쇄): WebView 기본 인쇄
 *
 * F1(도움말)·Ctrl+S(저장)는 화면 맥락 의존이 커 Sprint 16 이연(감사 보고서 기록).
 * AppShell 에 1회 마운트(`/lock`·`/setup` 제외).
 */

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'

export function GlobalShortcuts() {
  const router = useRouter()

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey)) return
      const key = e.key.toLowerCase()
      if (key === 'f') {
        e.preventDefault() // WebView 기본 '찾기' 억제 — 앱 검색바로 대체
        const input = document.getElementById('global-search-input') as HTMLInputElement | null
        input?.focus()
        input?.select()
      } else if (key === 'n') {
        e.preventDefault()
        router.push('/students/new')
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [router])

  return null
}
