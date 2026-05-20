'use client'

/**
 * 글로벌 키보드 단축키 훅 (Sprint 3 T14, PRD §5.7).
 *
 * Ctrl+F / Ctrl+/ 는 [[global-search]] 가 자체 처리하므로 본 훅에서 다루지 않는다.
 * 본 훅은 라우팅·다이얼로그 전역 동작만 담당한다.
 *
 * - F1: 도움말 — placeholder (Phase 7)
 * - Ctrl+N: 신규 원생 등록 → `/students/new`
 * - Ctrl+S: 저장 — 활성 form 의 submit 이벤트 트리거 (input/textarea 포커스 시)
 * - Ctrl+Z: Undo — placeholder (Phase 3 출결 토글 등)
 * - Ctrl+P: 인쇄 — `window.print()` (Phase 4 공지문, Phase 5 보고서)
 * - Esc: 모달 닫기 등 — placeholder (모달 자체 onKeyDown 으로 처리 가능)
 *
 * IME composition 중에는 모든 단축키를 무시 (한글 입력 보호).
 */

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'

export function useKeyboardShortcuts() {
  const router = useRouter()

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.isComposing) return
      const ctrl = e.ctrlKey || e.metaKey
      const target = e.target as HTMLElement | null
      const isFormElement =
        target !== null &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.tagName === 'SELECT' ||
          target.isContentEditable)

      if (ctrl && e.key.toLowerCase() === 'n') {
        e.preventDefault()
        router.push('/students/new')
        return
      }

      if (ctrl && e.key.toLowerCase() === 's') {
        e.preventDefault()
        if (isFormElement) {
          target.closest('form')?.requestSubmit()
        }
        return
      }

      if (ctrl && e.key.toLowerCase() === 'p') {
        e.preventDefault()
        window.print()
        return
      }

      if (e.key === 'F1') {
        e.preventDefault()
        // 향후 도움말 모달. 현재는 토스트 형식 안내가 없어 무동작.
        return
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [router])
}
