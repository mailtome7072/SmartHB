'use client'

/**
 * 교습일정 인쇄 전용 창 — Sprint 19 후속수정.
 *
 * 기존 `window.open()` 팝업 방식은 WebView2의 팝업 차단(사용자가 수동으로 설정을
 * 풀어줘야 함) 대상이 될 수 있었다. 이 페이지는 Tauri의 네이티브 창 생성 API
 * (`WebviewWindow`, academic/page.tsx 참조)로 열리는 **정식 앱 창**이므로 브라우저
 * 팝업 차단과 무관하다 — 사용자 설정 변경이 전혀 필요 없다.
 *
 * 쿼리스트링(start/end/ym)으로 교습기간을 전달받아 직접 IPC로 데이터를 조회한 뒤
 * `buildAcademicPrintHtml`로 만든 완결 HTML 문서로 자기 자신을 완전히 대체하고
 * (그 문서의 인라인 스크립트가 로드 완료 후 스스로 print() 호출) 인쇄한다.
 */

import { Suspense, useEffect, useState } from 'react'
import { useSearchParams } from 'next/navigation'
import { getOperatingHours, listScheduleEvents } from '@/lib/tauri'
import { buildAcademicPrintHtml } from '@/lib/academic-print-html'
import type { StudyPeriod } from '@/types/academic'

export default function AcademicPrintPage() {
  return (
    <Suspense fallback={null}>
      <AcademicPrintContent />
    </Suspense>
  )
}

function AcademicPrintContent() {
  const searchParams = useSearchParams()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const start = searchParams.get('start')
    const end = searchParams.get('end')
    const yearMonth = searchParams.get('ym')
    if (!start || !end || !yearMonth) {
      setError('인쇄할 교습기간 정보가 전달되지 않았습니다.')
      return
    }

    let cancelled = false
    void (async () => {
      try {
        const [events, operatingHours] = await Promise.all([
          listScheduleEvents(start, end),
          getOperatingHours(),
        ])
        if (cancelled) return
        const period: StudyPeriod = {
          id: 0,
          year_month: yearMonth,
          start_date: start,
          end_date: end,
          is_confirmed: true,
          is_closed: false,
          created_at: '',
          updated_at: '',
        }
        const html = buildAcademicPrintHtml({ period, events, operatingHours })
        document.open()
        document.write(html)
        document.close()
      } catch (err) {
        if (!cancelled) setError(err instanceof Error ? err.message : String(err))
      }
    })()
    return () => {
      cancelled = true
    }
  }, [searchParams])

  if (error !== null) {
    return (
      <p style={{ padding: 24, fontSize: 16, color: '#dc2626' }}>
        인쇄 데이터를 불러오지 못했습니다: {error}
      </p>
    )
  }
  return null
}
