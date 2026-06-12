'use client'

/**
 * 미저장 변경 경고 + 저장 단축키 공통 훅 (Sprint 16 T1 — R105·A100, PRD §5.7).
 *
 * - `dirty`가 true인 동안 창 닫기·새로고침·앱 종료 시 브라우저 beforeunload 경고를 띄운다.
 * - `onSave`를 넘기면 Ctrl/⌘+S로 실행한다. 단축키 자체는 항상 마운트된 `GlobalShortcuts`가
 *   감지해 `app:save` 커스텀 이벤트를 dispatch하고, 이 훅은 그 이벤트를 구독한다
 *   (전역 store·click 캡처 없이 느슨하게 결합).
 * - 내부 메뉴 이동(사이드바·글로벌 검색) 시 `unsavedGuard`(app-store)를 등록해, dirty면
 *   이동을 차단하고 `unsavedNavTarget`을 세팅 → AppShell의 `UnsavedNavDialog`가 확인
 *   다이얼로그를 띄운다. Next.js App Router에는 `routeChangeStart`가 없어 라우팅 이벤트
 *   대신 메뉴 클릭 인터셉트(사이드바/검색의 guardedNavigate)에 의존한다. Tauri WebView가
 *   `window.confirm`/`alert`을 차단하므로 자체 다이얼로그를 쓴다. 창 닫기·새로고침·앱
 *   종료는 별도로 beforeunload가 처리한다.
 */

import { useEffect, useRef } from 'react'
import { useAppStore } from '@/stores/app-store'

export const APP_SAVE_EVENT = 'app:save'

export function useUnsavedChanges(dirty: boolean, onSave?: () => void) {
  // 매 렌더 최신 dirty/onSave 를 참조해 effect 재등록을 피한다.
  const dirtyRef = useRef(dirty)
  dirtyRef.current = dirty
  const onSaveRef = useRef(onSave)
  onSaveRef.current = onSave

  // 창 닫기·새로고침·앱 종료 경고 (dirty일 때만 등록).
  useEffect(() => {
    if (!dirty) return
    const handler = (e: BeforeUnloadEvent) => {
      e.preventDefault()
      e.returnValue = ''
    }
    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [dirty])

  // Ctrl+S(app:save) 저장.
  useEffect(() => {
    const handler = () => onSaveRef.current?.()
    window.addEventListener(APP_SAVE_EVENT, handler)
    return () => window.removeEventListener(APP_SAVE_EVENT, handler)
  }, [])

  // 내부 메뉴 이동 가드 — 사이드바/글로벌 검색이 이동 직전 호출한다.
  // dirty가 아니면 즉시 통과(true). dirty면 이동을 차단(false)하고 대상 경로를 store에
  // 세팅 → UnsavedNavDialog가 확인 다이얼로그를 띄우고, 확인 시 직접 이동을 수행한다.
  const setUnsavedGuard = useAppStore((s) => s.setUnsavedGuard)
  const setUnsavedNavTarget = useAppStore((s) => s.setUnsavedNavTarget)
  useEffect(() => {
    const guard = (href: string) => {
      if (!dirtyRef.current) return true
      setUnsavedNavTarget(href)
      return false
    }
    setUnsavedGuard(guard)
    return () => setUnsavedGuard(null)
  }, [setUnsavedGuard, setUnsavedNavTarget])
}
