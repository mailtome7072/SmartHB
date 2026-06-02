'use client'

/**
 * 6자리 숫자 PIN 입력 필드 (ADR-007) — 한 자리씩 입력하는 6개 박스(OTP 스타일).
 *
 * 잠금 화면(LockScreen)·PIN 변경 화면(/settings/pin) 공용.
 * - 각 박스는 숫자 1자리. 입력 시 다음 칸으로, Backspace(빈 칸)는 이전 칸으로 자동 이동
 * - 붙여넣기 시 현재 칸부터 자릿수만큼 분배
 * - 모바일/터치는 `inputMode="numeric"` 숫자 키패드
 * - 기본은 가림(●), 라벨 우측 보기/숨김 토글(필드별 내부 상태)
 * - 부모와는 6자리 문자열(value/onChange)로 통신한다.
 */

import { useRef, useState } from 'react'

export const PIN_LENGTH = 6
export const PIN_PATTERN = /^\d{6}$/

interface PinFieldProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  autoFocus?: boolean
  hasError?: boolean
}

export function PinField({ id, label, value, onChange, autoFocus, hasError = false }: PinFieldProps) {
  const [show, setShow] = useState(false)
  const inputsRef = useRef<Array<HTMLInputElement | null>>([])
  const digits = Array.from({ length: PIN_LENGTH }, (_, i) => value[i] ?? '')

  const commit = (next: string[]) => onChange(next.join('').slice(0, PIN_LENGTH))

  const handleChange = (i: number, raw: string) => {
    const d = raw.replace(/\D/g, '')
    const next = digits.slice()
    if (d === '') {
      next[i] = ''
      commit(next)
      return
    }
    next[i] = d.slice(-1) // 마지막 숫자 채택 (기존 문자 + 새 입력 케이스)
    commit(next)
    if (i < PIN_LENGTH - 1) inputsRef.current[i + 1]?.focus()
  }

  const handleKeyDown = (i: number, e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Backspace' && digits[i] === '' && i > 0) {
      e.preventDefault()
      const next = digits.slice()
      next[i - 1] = ''
      commit(next)
      inputsRef.current[i - 1]?.focus()
    } else if (e.key === 'ArrowLeft' && i > 0) {
      e.preventDefault()
      inputsRef.current[i - 1]?.focus()
    } else if (e.key === 'ArrowRight' && i < PIN_LENGTH - 1) {
      e.preventDefault()
      inputsRef.current[i + 1]?.focus()
    }
  }

  const handlePaste = (i: number, e: React.ClipboardEvent<HTMLInputElement>) => {
    e.preventDefault()
    const text = e.clipboardData.getData('text').replace(/\D/g, '')
    if (text === '') return
    const next = digits.slice()
    let pos = i
    for (const c of text.split('')) {
      if (pos >= PIN_LENGTH) break
      next[pos] = c
      pos++
    }
    commit(next)
    inputsRef.current[Math.min(pos, PIN_LENGTH - 1)]?.focus()
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-center gap-2">
        <label htmlFor={`${id}-0`} className="text-base font-medium">
          {label}
        </label>
        <button
          type="button"
          onClick={() => setShow((s) => !s)}
          aria-label={show ? 'PIN 가리기' : 'PIN 표시'}
          className="flex h-[32px] min-w-[56px] items-center justify-center rounded-md border border-[var(--border)] bg-white px-2 text-sm text-gray-700 hover:bg-gray-50"
        >
          {show ? '숨김' : '보기'}
        </button>
      </div>
      <div className="flex items-center justify-center gap-2">
        {digits.map((digit, i) => (
          <input
            key={i}
            id={i === 0 ? `${id}-0` : undefined}
            ref={(el) => {
              inputsRef.current[i] = el
            }}
            type={show ? 'text' : 'password'}
            value={digit}
            onChange={(e) => handleChange(i, e.target.value)}
            onKeyDown={(e) => handleKeyDown(i, e)}
            onPaste={(e) => handlePaste(i, e)}
            onFocus={(e) => e.currentTarget.select()}
            autoComplete="off"
            autoFocus={autoFocus && i === 0}
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            inputMode="numeric"
            aria-label={`${label} ${i + 1}번째 자리`}
            className={`h-[76px] w-[52px] rounded-lg border-2 text-center text-3xl focus:outline-none focus:ring-2 focus:ring-[var(--accent)] ${
              hasError ? 'border-[var(--danger)]' : 'border-[var(--border)]'
            }`}
          />
        ))}
      </div>
      <p className="text-center text-xs text-gray-500">숫자 {PIN_LENGTH}자리를 한 칸씩 입력하세요.</p>
    </div>
  )
}
