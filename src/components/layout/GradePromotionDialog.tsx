'use client'

/**
 * 학년 자동 승급 확인 다이얼로그 (Sprint 19 T8) — AppShell에 1회 마운트.
 *
 * `UnsavedNavDialog`와 동일하게 최상위 다이얼로그를 별도 컴포넌트로 분리한다 — AppShell은
 * 레이아웃 셸 책임만 지고, IPC 호출·다이얼로그 상태는 이 컴포넌트가 캡슐화한다.
 *
 * 매년 1월 이후 최초 실행(unlock) 시 `check_grade_promotion`을 세션당 1회 호출해 대상이
 * 있으면 확인 다이얼로그를 띄운다. "자동 조용히 적용 금지" 원칙 — 사용자가 승인해야만
 * `promote_grades`(일괄 UPDATE)를 실행한다. 거부 시 이번 세션에서는 다시 묻지 않는다
 * (모듈 레벨 플래그 — 백엔드는 아직 미기록 상태라 다음 앱 실행 때 다시 확인한다).
 */

import { useEffect, useState } from 'react'
import { useSessionStore } from '@/stores/session-store'
import { checkGradePromotion, promoteGrades } from '@/lib/tauri'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

let gradePromotionAttempted = false

export function GradePromotionDialog() {
  const unlocked = useSessionStore((s) => s.unlocked)
  const [prompt, setPrompt] = useState<{ count: number } | null>(null)
  const [promoting, setPromoting] = useState(false)

  useEffect(() => {
    if (!unlocked || gradePromotionAttempted) return
    gradePromotionAttempted = true
    void (async () => {
      try {
        const result = await checkGradePromotion()
        if (result.needed && result.count > 0) {
          setPrompt({ count: result.count })
        }
      } catch {
        /* 조회 실패는 무시 — 다음 세션에서 재시도 */
      }
    })()
  }, [unlocked])

  return (
    <AlertDialog
      open={prompt !== null}
      onOpenChange={(open) => {
        if (!open) setPrompt(null)
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>학년 자동 승급</AlertDialogTitle>
          <AlertDialogDescription>
            올해 <strong>{prompt?.count}명</strong>의 학년이 자동으로 상향됩니다
            (초등학교 6학년, 중학교 3학년은 제외).
            <br />
            진행하시겠습니까?
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={promoting}>취소</AlertDialogCancel>
          <AlertDialogAction
            onClick={async (e) => {
              e.preventDefault()
              setPromoting(true)
              try {
                await promoteGrades()
                setPrompt(null)
              } finally {
                setPromoting(false)
              }
            }}
            disabled={promoting}
          >
            {promoting ? '처리 중...' : '진행'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
