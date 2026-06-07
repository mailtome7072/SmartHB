'use client'

/**
 * 전역 툴팁 (Sprint 15 T5) — 앱 전체에서 `title` 속성을 가진 요소의 마우스 오버 팝업을
 * 가로채 큰 폰트(20px) 커스텀 툴팁으로 표시한다.
 *
 * native `title` 툴팁은 브라우저가 직접 그려 폰트 크기를 CSS 로 조정할 수 없다(50대 사용자
 * 가독성 문제). document 이벤트 위임으로 title 가진 요소를 일괄 대체하므로 개별 컴포넌트
 * 코드는 수정하지 않는다 — mouseover 시 title 을 `data-shb-title` 로 백업하고 제거(native
 * 억제), mouseout 시 복원한다. AppShell 에 1회 마운트(`/lock`·`/setup` 제외).
 *
 * 멀티라인(`\n`) 텍스트는 `white-space:pre-line` 으로 줄바꿈 반영 (예: 수업 캘린더 명단).
 */

import { useEffect } from 'react'

const TOOLTIP_ID = 'shb-global-tooltip'

function removeTip() {
  document.getElementById(TOOLTIP_ID)?.remove()
}

function showTip(text: string, anchor: HTMLElement) {
  removeTip()
  if (text.trim() === '') return
  const tip = document.createElement('div')
  tip.id = TOOLTIP_ID
  tip.textContent = text
  tip.style.cssText =
    'position:fixed;z-index:9999;pointer-events:none;background:#111;color:#fff;' +
    'padding:8px 12px;border-radius:8px;font-size:20px;line-height:1.5;' +
    'white-space:pre-line;max-width:520px;box-shadow:0 4px 12px rgba(0,0,0,0.3);'
  document.body.appendChild(tip)
  // 앵커 중앙 위쪽에 배치하되 화면 경계를 넘지 않도록 보정. 위 공간 부족 시 아래로.
  const a = anchor.getBoundingClientRect()
  const t = tip.getBoundingClientRect()
  const left = Math.max(8, Math.min(a.left + a.width / 2 - t.width / 2, window.innerWidth - t.width - 8))
  const top = a.top - t.height - 8
  tip.style.left = `${left}px`
  tip.style.top = `${top < 8 ? a.bottom + 8 : top}px`
}

export function GlobalTooltip() {
  useEffect(() => {
    let active: HTMLElement | null = null

    const restore = () => {
      if (active) {
        const saved = active.getAttribute('data-shb-title')
        if (saved !== null) {
          active.setAttribute('title', saved)
          active.removeAttribute('data-shb-title')
        }
        active = null
      }
      removeTip()
    }

    const onOver = (e: MouseEvent) => {
      const el = (e.target as HTMLElement | null)?.closest('[title]') as HTMLElement | null
      if (el === null || el === active) return
      restore() // 이전 앵커 정리
      const text = el.getAttribute('title')
      if (text === null || text.trim() === '') return
      el.setAttribute('data-shb-title', text)
      el.removeAttribute('title') // native 툴팁 억제
      active = el
      showTip(text, el)
    }

    const onOut = (e: MouseEvent) => {
      if (active === null) return
      const to = e.relatedTarget as Node | null
      if (to !== null && active.contains(to)) return // 자식으로 이동 — 유지
      restore()
    }

    document.addEventListener('mouseover', onOver, true)
    document.addEventListener('mouseout', onOut, true)
    window.addEventListener('scroll', restore, true)
    return () => {
      document.removeEventListener('mouseover', onOver, true)
      document.removeEventListener('mouseout', onOut, true)
      window.removeEventListener('scroll', restore, true)
      restore()
    }
  }, [])

  return null
}
