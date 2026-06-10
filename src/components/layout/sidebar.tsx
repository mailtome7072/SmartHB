'use client'

/**
 * 사이드바 (Sprint 3 T5, PRD §5.1 + §5.7).
 *
 * Phase 1 에서 활성: 원생 관리 / 설정.
 * Phase 2+ 메뉴(대시보드/수업/출결/청구/단원평가/학습보고서/공지문)는 disabled 처리하여
 * 사용자에게 "다음 업데이트 예정" 안내. 메뉴 항목에 단축키 표기 병기 (PRD §5.7).
 *
 * - 클릭 영역 44×44px 이상 (Tailwind py-3 = 12px × 2 + 본문 18px line-height ≈ 51px)
 * - WCAG AA 명도 대비 보장 (foreground/border 토큰 사용)
 *
 * Sprint 7 보강: 사이드바 하단 "종료" 항목.
 * `quitApp()` IPC → `AppHandle::exit(0)` → `RunEvent::ExitRequested` →
 * `startup::exit_hook` 의 release_lock + exit 백업 (R15 보장).
 * 기존 `getCurrentWindow().close()` 는 capabilities `core:window:allow-close` 권한 부재로
 * 거부됐었음. 백엔드 IPC 경유로 권한 + macOS 닥 잔존 이슈 동시 회피.
 *
 * scope-외 추가 (2026-06-04): "종료" 클릭 시 즉시 종료하지 않고 확인 다이얼로그를 거치도록
 * 변경 (PRD §5.7 실수 복구 — 위험 동작 명시적 확인). 기존 AlertDialog 컴포넌트 재사용.
 */

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useAppStore } from '@/stores/app-store'
import { MENU_ITEMS } from '@/lib/menu-config'
import { quitApp } from '@/lib/tauri'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'

/** 현재 경로가 해당 메뉴에 속하는지 — '/'(대시보드)는 정확 일치, 그 외는 하위 경로 포함. */
function isMenuActive(pathname: string, href: string): boolean {
  if (href === '/') return pathname === '/'
  return pathname === href || pathname.startsWith(`${href}/`)
}

// 구분선 — li 상단에 좌우 20px 안쪽으로 들어간 라인(pseudo-element). 메뉴 텍스트 정렬은 불변.
const MENU_DIVIDER =
  "relative before:absolute before:inset-x-[20px] before:top-0 before:border-t before:border-[var(--border)] before:content-['']"

/** 메뉴 항목 li 의 그룹 구분 여백/구분선 (디자인 조정). */
function menuItemClass(href: string): string | undefined {
  if (href === '/students') return 'mt-10' // 대시보드와 원생 관리 사이 40px 여백
  if (href === '/academic') return MENU_DIVIDER // 원생 관리와 일정 관리 사이 구분선
  if (href === '/billing') return MENU_DIVIDER // 수업 관리와 청구 관리 사이 구분선
  if (href === '/notices') return MENU_DIVIDER // 수납 관리와 공지문 사이 구분선
  if (href === '/settings') return 'mt-10' // 공지문과 설정 사이 40px 여백
  return undefined
}

export function Sidebar() {
  const sidebarOpen = useAppStore((s) => s.sidebarOpen)
  const pathname = usePathname()

  if (!sidebarOpen) return null

  return (
    <nav
      aria-label="메인 메뉴"
      className="flex h-full w-[11.2rem] shrink-0 flex-col border-r border-[var(--border)] bg-white"
    >
      <div className="flex h-16 flex-col justify-center border-b border-[var(--border)] px-4">
        <p className="text-base font-bold leading-tight text-[var(--foreground)]">스마트해법수학</p>
        <p className="text-sm leading-tight text-gray-600">서현효자점</p>
      </div>
      <ul className="flex-1 overflow-y-auto py-2">
        {MENU_ITEMS.map((item) => (
          <li key={item.href} className={menuItemClass(item.href)}>
            {item.disabledHint !== undefined ? (
              <span
                aria-disabled="true"
                title={item.disabledHint}
                className="flex min-h-[44px] cursor-not-allowed items-center border-l-4 border-transparent px-4 py-3 text-gray-600"
              >
                {item.label}
              </span>
            ) : (
              (() => {
                const active = isMenuActive(pathname, item.href)
                return (
                  <Link
                    href={item.href}
                    aria-current={active ? 'page' : undefined}
                    onClick={(e) => {
                      // 미저장 가드(공지문 편집 등)가 차단하면 기본 이동을 막고 가드에 위임.
                      const guard = useAppStore.getState().unsavedGuard
                      if (guard && !guard(item.href)) e.preventDefault()
                    }}
                    className={`flex min-h-[44px] items-center border-l-4 px-4 py-3 ${
                      active
                        ? 'border-[var(--accent)] bg-[var(--background)] font-semibold text-[var(--accent)]'
                        : 'border-transparent text-[var(--foreground)] hover:bg-[var(--background)]'
                    }`}
                  >
                    {item.label}
                  </Link>
                )
              })()
            )}
          </li>
        ))}
        <li>
          <AlertDialog>
            <AlertDialogTrigger
              type="button"
              className="flex min-h-[44px] w-full items-center justify-between border-l-4 border-transparent px-4 py-3 text-left text-[var(--foreground)] hover:bg-[var(--background)]"
            >
              <span>종료</span>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>프로그램 종료</AlertDialogTitle>
                <AlertDialogDescription>
                  스마트해법수학 관리 앱을 종료하시겠습니까?
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>취소</AlertDialogCancel>
                <AlertDialogAction onClick={() => void quitApp()}>종료</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </li>
      </ul>
    </nav>
  )
}
