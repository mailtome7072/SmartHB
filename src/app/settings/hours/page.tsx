'use client'

/**
 * 교습소 운영 시간 편집 (Sprint 4 T2 / 사용자 이슈 #0, PRD §4.0/§4.12).
 *
 * 요일별(월~일) 시작/종료 시간을 1시간 단위 콤보로 편집. 미운영 토글로 해당 요일 휴무.
 * `app_settings.operating_hours` 에 JSON 직렬화 저장 — DB 마이그레이션 불필요.
 *
 * T9 (수업 스케줄 시작시간 콤보) 가 본 설정값을 참조하여 1시간 단위 선택지를 생성한다.
 */

import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SettingsHomeLink } from '@/components/settings/SettingsHomeLink'
import { SplashScreen } from '@/components/splash-screen'
import { getOperatingHours, saveOperatingHours, type DayHours } from '@/lib/tauri'

const DAY_LABELS = ['', '월요일', '화요일', '수요일', '목요일', '금요일', '토요일', '일요일']

/** 교습소 운영 가능 시간대 — 10:00~20:00 한 시간 단위 (11개 옵션). */
const HOUR_OPTIONS = Array.from({ length: 11 }, (_, i) => {
  const h = (i + 10).toString().padStart(2, '0')
  return `${h}:00`
})

export default function OperatingHoursPage() {
  const [hours, setHours] = useState<DayHours[] | null>(null)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [savedAt, setSavedAt] = useState<string | null>(null)

  useEffect(() => {
    getOperatingHours()
      .then((data) => setHours(data))
      .catch((e: unknown) =>
        setError(typeof e === 'string' ? e : '운영 시간을 불러올 수 없습니다.'),
      )
  }, [])

  const updateDay = (idx: number, patch: Partial<DayHours>) => {
    if (!hours) return
    const next = hours.map((h, i) => (i === idx ? { ...h, ...patch } : h))
    setHours(next)
    setSavedAt(null)
  }

  const toggleClosed = (idx: number) => {
    if (!hours) return
    const current = hours[idx]
    const closed = current.open_time === null
    updateDay(idx, {
      open_time: closed ? '13:00' : null,
      close_time: closed ? '19:00' : null,
    })
  }

  const handleSave = async () => {
    if (!hours) return
    setError(null)
    // 클라이언트 사전 검증 — 백엔드도 검증하지만 빠른 피드백.
    for (const h of hours) {
      if (h.open_time !== null && h.close_time !== null) {
        if (h.open_time >= h.close_time) {
          setError(`${DAY_LABELS[h.day_of_week]}의 시작 시간은 종료 시간보다 빨라야 합니다.`)
          return
        }
      }
    }
    setSaving(true)
    try {
      await saveOperatingHours(hours)
      setSavedAt(new Date().toLocaleTimeString('ko-KR'))
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : '저장에 실패했습니다. 잠시 후 다시 시도해주세요.')
    } finally {
      setSaving(false)
    }
  }

  if (hours === null && error === null) {
    return <SplashScreen message="운영 시간을 불러오는 중입니다..." />
  }

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      {/* 사용자 요청 — 전체 행간 1.25(leading-tight)로 통일. */}
      <div className="mx-auto max-w-3xl leading-tight">
        <SettingsHomeLink />
        <div className="mb-6">
          <h1 className="text-2xl font-bold">교습소 운영 시간</h1>
          <p className="mt-1 text-sm text-gray-600">
            요일별 운영 시작/종료 시간 설정. 수업 스케줄 시작 시간 선택지 기준.
          </p>
        </div>

        {error !== null && (
          <p
            role="alert"
            className="mb-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            {error}
          </p>
        )}

        {hours !== null && (
          <>
            <table className="w-full border-collapse rounded-lg border border-[var(--border)] bg-white text-base">
              <thead className="bg-gray-50">
                <tr>
                  <th className="border-b border-[var(--border)] px-4 py-2 text-left">요일</th>
                  <th className="border-b border-[var(--border)] px-4 py-2 text-left">운영 여부</th>
                  <th className="border-b border-[var(--border)] px-4 py-2 text-left">시작</th>
                  <th className="border-b border-[var(--border)] px-4 py-2 text-left">종료</th>
                </tr>
              </thead>
              <tbody>
                {hours.map((h, idx) => {
                  const closed = h.open_time === null
                  return (
                    <tr key={h.day_of_week} className="border-b border-[var(--border)] last:border-0">
                      <td className="px-4 py-2 font-medium">{DAY_LABELS[h.day_of_week]}</td>
                      <td className="px-4 py-2">
                        <label className="inline-flex min-h-[44px] cursor-pointer items-center gap-2">
                          <input
                            type="checkbox"
                            checked={!closed}
                            onChange={() => toggleClosed(idx)}
                            className="h-5 w-5"
                          />
                          <span className="text-sm">{closed ? '미운영' : '운영'}</span>
                        </label>
                      </td>
                      <td className="px-4 py-2">
                        <select
                          value={h.open_time ?? ''}
                          onChange={(e) => updateDay(idx, { open_time: e.target.value })}
                          disabled={closed}
                          className="h-11 min-w-[110px] rounded-md border border-[var(--border)] px-2 disabled:bg-gray-100 disabled:text-gray-600"
                        >
                          {HOUR_OPTIONS.map((opt) => (
                            <option key={opt} value={opt}>
                              {opt}
                            </option>
                          ))}
                        </select>
                      </td>
                      <td className="px-4 py-2">
                        <select
                          value={h.close_time ?? ''}
                          onChange={(e) => updateDay(idx, { close_time: e.target.value })}
                          disabled={closed}
                          className="h-11 min-w-[110px] rounded-md border border-[var(--border)] px-2 disabled:bg-gray-100 disabled:text-gray-600"
                        >
                          {HOUR_OPTIONS.map((opt) => (
                            <option key={opt} value={opt}>
                              {opt}
                            </option>
                          ))}
                        </select>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>

            <div className="mt-6 flex items-center justify-end gap-3">
              {savedAt !== null && (
                <p className="text-sm text-gray-600">저장 완료 — {savedAt}</p>
              )}
              <button
                type="button"
                onClick={handleSave}
                disabled={saving}
                className="h-11 rounded-md bg-[var(--accent)] px-5 font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
              >
                {saving ? '저장 중...' : '저장'}
              </button>
            </div>
          </>
        )}
      </div>
    </AppShell>
  )
}
