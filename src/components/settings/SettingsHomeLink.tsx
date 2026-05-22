/**
 * 설정 sub 페이지에서 `/settings` 허브로 돌아가는 링크 버튼 — V2 (Sprint 7 post-review).
 *
 * 모든 `/settings/*` 페이지 상단에 일관된 위치로 배치하여 50대 사용자가 설정 허브로
 * 쉽게 되돌아갈 수 있도록 한다. 44×44px 클릭 영역 + 18pt 폰트 (PRD §5.7).
 */

import Link from 'next/link'

export function SettingsHomeLink() {
  return (
    <Link
      href="/settings"
      className="mb-4 inline-flex min-h-[44px] items-center gap-1 rounded-md border border-[var(--border)] bg-white px-3 py-2 text-base text-[var(--foreground)] hover:bg-gray-50"
    >
      ← 설정메인
    </Link>
  )
}
