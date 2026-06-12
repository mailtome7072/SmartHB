'use client'

/**
 * 전역 키보드 단축키 (Sprint 15 T3 / Sprint 16 T1, PRD §5.7).
 *
 * - Ctrl/⌘ + F : 글로벌 검색바 포커스 (#global-search-input)
 * - Ctrl/⌘ + N : 신규 원생 등록 화면 이동 (입력 필드 포커스 중에는 억제 — A99)
 * - Ctrl/⌘ + S : 활성 폼 저장 (app:save 이벤트 dispatch, useUnsavedChanges가 구독)
 *
 * 이미 동작하는 것은 별도 구현하지 않는다:
 * - ESC(다이얼로그 닫기): Radix(shadcn) Dialog/AlertDialog 기본 동작
 * - Ctrl+Z(Undo): 출결 그리드 국소 구현(AttendanceGrid)
 * - Ctrl+P(인쇄): WebView 기본 인쇄
 *
 * F1(도움말)은 화면 맥락 의존이 커 추후 이연.
 * AppShell 에 1회 마운트(`/lock`·`/setup` 제외).
 */

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { APP_SAVE_EVENT } from '@/lib/use-unsaved-changes'

// 입력 중인 요소에서 발동하면 안 되는 단축키 방어용 — 텍스트 입력/편집 컨텍스트 판별.
// AttendanceGrid Ctrl+Z 등 국소 단축키 구현도 재사용한다 (P0-6).
export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  const tag = target.tagName
  return (
    tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target.isContentEditable
  )
}

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
        // A99: 입력 필드 포커스 중에는 Ctrl+N(신규 원생)이 발동하지 않도록 억제.
        if (isEditableTarget(e.target)) return
        e.preventDefault()
        router.push('/students/new')
      } else if (key === 's') {
        // WebView 기본 저장 다이얼로그를 억제하고 활성 폼에 저장 신호를 보낸다.
        e.preventDefault()
        window.dispatchEvent(new CustomEvent(APP_SAVE_EVENT))
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [router])

  return null
}
