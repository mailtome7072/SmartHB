'use client'

/**
 * 미저장 이동 확인 다이얼로그 (Sprint 16 T1) — AppShell에 1회 마운트.
 *
 * `useUnsavedChanges` 훅이 등록한 가드가 메뉴 이동을 차단하면서 `unsavedNavTarget`에
 * 이동 대상 경로를 세팅한다. 이 컴포넌트는 그 값을 구독해 확인 다이얼로그를 띄우고,
 * "이동"을 누르면 해당 경로로 라우팅한다. Tauri WebView가 `window.confirm`/`alert`을
 * 차단(dialog 권한)하므로 자체 모달을 쓴다. 프로젝트의 다른 확인 모달과 동일한
 * plain `fixed` 오버레이 패턴(접근성: role=dialog, aria-modal).
 */

import { useRouter } from 'next/navigation'
import { useAppStore } from '@/stores/app-store'

export function UnsavedNavDialog() {
  const router = useRouter()
  const target = useAppStore((s) => s.unsavedNavTarget)
  const setTarget = useAppStore((s) => s.setUnsavedNavTarget)

  if (target === null) return null

  const move = () => {
    setTarget(null)
    router.push(target)
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-md rounded-lg bg-white p-5 shadow-xl">
        <p className="mb-4 text-base text-gray-800">
          저장하지 않은 변경 사항이 있습니다.
          <br />
          저장하지 않고 이동하시겠습니까?
        </p>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => setTarget(null)}
            className="min-h-[44px] flex-1 rounded-md border-2 border-[var(--border)] px-4 text-base text-gray-700 hover:bg-gray-50"
          >
            취소
          </button>
          <button
            type="button"
            onClick={move}
            className="min-h-[44px] flex-1 rounded-md bg-[var(--accent)] px-4 text-base font-semibold text-white hover:opacity-90"
          >
            이동
          </button>
        </div>
      </div>
    </div>
  )
}
