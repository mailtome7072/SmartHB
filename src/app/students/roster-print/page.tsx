'use client'

/**
 * 수강생대장 인쇄 전용 창 — 교습일정 인쇄(academic/print)와 동일 아키텍처.
 *
 * Tauri의 네이티브 창 생성 API(WebviewWindow)로 열리는 정식 앱 창이므로 브라우저 팝업
 * 차단과 무관하다. 마운트 시 직접 IPC로 재원중 원생 목록 + 학원 정보를 조회한 뒤
 * `buildStudentRosterHtml`로 만든 완결 HTML 문서로 자기 자신을 완전히 대체하고 인쇄한다.
 */

import { useEffect, useState } from 'react'
import { getAcademyInfo, listStudents } from '@/lib/tauri'
import { buildStudentRosterHtml } from '@/lib/student-roster-print-html'

export default function StudentRosterPrintPage() {
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const [students, academyInfo] = await Promise.all([
          // 퇴교 원생도 퇴교일자와 함께 표시해야 하므로 재원 여부와 무관하게 전체 조회.
          listStudents({ active_only: false, sort: 'serial-asc', limit: 100000 }),
          getAcademyInfo(),
        ])
        if (cancelled) return
        const html = buildStudentRosterHtml({ students, academyName: academyInfo.academy_name })
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
  }, [])

  if (error !== null) {
    return (
      <p style={{ padding: 24, fontSize: 16, color: '#dc2626' }}>
        수강생대장 데이터를 불러오지 못했습니다: {error}
      </p>
    )
  }
  return null
}
