'use client'

/**
 * 설정 허브 (Sprint 4 T2, PRD §4.0/§4.12).
 *
 * 영구 설정 메뉴 진입점. 마법사(`/setup`) 와 분리 — unlock 이후 사용자가 운영 중 변경하는
 * 항목들을 카드 그리드로 노출. 각 카드 = 별도 라우트.
 */

import Link from 'next/link'
import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { getPinSkipSetting, setPinSkipSetting } from '@/lib/tauri'

interface SettingCard {
  href: string
  title: string
  description: string
  disabledHint?: string
}

const CARDS: SettingCard[] = [
  {
    href: '/settings/hours',
    title: '교습소 운영 시간',
    description: '요일별 시작/종료 시간 설정. 수업 스케줄 시작 시간 콤보 기준.',
  },
  {
    href: '/settings/codes',
    title: '코드 테이블 관리',
    description: '학교 / 표준 교습비 / 결제 수단 / 카드사 — CRUD 및 정렬',
  },
  {
    href: '/settings/schedule-codes',
    title: '학사 일정 코드 관리',
    description: '공휴일·보강데이 등 시스템 코드 + 사용자 추가 코드의 활성 토글 및 CRUD',
  },
  {
    href: '/settings/diagnosis',
    title: '데이터 자가 진단',
    description: '원생·출결·청구 데이터 정합성 7종 점검. 매월 1일 자동 + 수동 실행, 최근 12개월 이력.',
  },
  {
    href: '/settings/data',
    title: '데이터 내보내기',
    description: '원생·출결·청구 데이터를 CSV 파일로 저장. 엑셀에서 바로 열람 가능 (전체/월별).',
  },
  {
    href: '/settings/pin',
    title: 'PIN 번호 변경',
    description: '현재 6자리 PIN 확인 후 새 PIN 으로 변경합니다. (변경 즉시 적용)',
  },
  {
    href: '/settings/info',
    title: '교습소 정보',
    description: '교습소명 / 주소 / 대표자 / 연락처 등 사업자 정보 (예정)',
    disabledHint: '후속 sprint 에서 제공',
  },
  {
    href: '/setup',
    title: '초기 설정 마법사 재실행',
    description: '클라우드 폴더 변경 등 마법사를 다시 진행합니다. (예정)',
    disabledHint: '재실행 흐름 정비 후 활성화',
  },
]

/** 실행 시 PIN 인증 사용 토글 (ADR-008). 끄면 이 PC에서 앱 실행 시 PIN 입력을 건너뛴다. */
function PinAuthToggle() {
  // skipPin = config 의 skip_pin_on_launch (true = 스킵). 스위치는 'PIN 인증 사용' = !skipPin 으로 표시.
  const [skipPin, setSkipPin] = useState<boolean | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    getPinSkipSetting()
      .then(setSkipPin)
      .catch(() => setSkipPin(false))
  }, [])

  const usePin = skipPin === false
  const handleToggle = async () => {
    if (skipPin === null || saving) return
    const nextSkip = !skipPin
    setSaving(true)
    try {
      await setPinSkipSetting(nextSkip)
      setSkipPin(nextSkip)
    } catch {
      /* 저장 실패 — 상태 유지 */
    } finally {
      setSaving(false)
    }
  }

  return (
    <section className="mb-6 rounded-lg border border-[var(--border)] bg-white p-5">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-lg font-bold text-[var(--foreground)]">실행 시 PIN 인증 사용</h2>
          <p className="mt-1 text-sm text-gray-600">
            끄면 이 PC에서 앱을 실행할 때 PIN 입력을 건너뜁니다. (이 PC에만 적용)
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={usePin}
          aria-label="실행 시 PIN 인증 사용"
          onClick={handleToggle}
          disabled={skipPin === null || saving}
          className={`relative inline-flex h-7 w-12 shrink-0 items-center rounded-full transition-colors disabled:opacity-50 ${
            usePin ? 'bg-[var(--accent)]' : 'bg-gray-300'
          }`}
        >
          <span
            className={`inline-block h-5 w-5 transform rounded-full bg-white transition-transform ${
              usePin ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>
      {skipPin === true && (
        <p className="mt-3 rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-800">
          ⚠️ 이 PC에서 PIN 입력을 건너뜁니다. 데이터 보호는 OS 계정 로그인에 의존합니다. PIN을
          잊으면 앱 데이터를 초기화해야 합니다.
        </p>
      )}
    </section>
  )
}

export default function SettingsHubPage() {
  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-4xl">
        <h1 className="mb-2 text-2xl font-bold">설정</h1>
        <p className="mb-6 text-base text-gray-600">
          교습소 운영 환경을 설정합니다. 변경 즉시 저장되며 마법사 재실행 없이 반영됩니다.
        </p>

        <PinAuthToggle />

        <div className="grid gap-4 sm:grid-cols-2">
          {CARDS.map((card) =>
            card.disabledHint !== undefined ? (
              <div
                key={card.href}
                aria-disabled="true"
                title={card.disabledHint}
                className="cursor-not-allowed rounded-lg border border-[var(--border)] bg-gray-50 p-5 opacity-60"
              >
                <h2 className="mb-2 text-lg font-bold text-gray-500">{card.title}</h2>
                <p className="text-sm text-gray-500">{card.description}</p>
                <p className="mt-3 text-xs text-gray-400">{card.disabledHint}</p>
              </div>
            ) : (
              <Link
                key={card.href}
                href={card.href}
                className="block min-h-[44px] rounded-lg border border-[var(--border)] bg-white p-5 transition-colors hover:border-[var(--accent)] hover:bg-[var(--background)]"
              >
                <h2 className="mb-2 text-lg font-bold text-[var(--foreground)]">{card.title}</h2>
                <p className="text-sm text-gray-600">{card.description}</p>
              </Link>
            ),
          )}
        </div>
      </div>
    </AppShell>
  )
}
