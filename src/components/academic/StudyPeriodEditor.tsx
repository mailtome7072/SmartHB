'use client'

/**
 * 교습기간 설정 에디터 — Sprint 6 T10 (PRD §4.4.2).
 *
 * 흐름:
 *   1) "교습기간 설정" 버튼 → mode='editing'
 *   2) 캘린더에서 시작일/종료일 셀 클릭 → selection state 갱신 (부모 관리)
 *   3) "확정" 버튼 → AlertDialog 확인 → createStudyPeriod + confirmStudyPeriod IPC
 *   4) 성공 시 ['study-periods'] 캐시 무효화 + 모드 종료 + 선택 초기화
 *   5) 실패(중첩 등) 시 백엔드 한국어 메시지 그대로 표시
 *
 * State 는 부모(`/academic` page) 가 관리 — ThreeMonthCalendar 와 selection 공유 + onCellClick 분기.
 * Editor 는 토글·UI·mutation 만 담당.
 */

import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createStudyPeriod, confirmStudyPeriod } from '@/lib/tauri'
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

export interface SelectionRange {
  start: string | null
  end: string | null
}

export type EditorMode = 'view' | 'editing'

interface StudyPeriodEditorProps {
  mode: EditorMode
  setMode: (m: EditorMode) => void
  selection: SelectionRange
  setSelection: (s: SelectionRange) => void
}

/** 정렬된 [시작, 종료] 반환 — start > end 시 자동 swap. */
function normalizeRange(s: SelectionRange): { start: string; end: string } | null {
  if (!s.start || !s.end) return null
  return s.start <= s.end ? { start: s.start, end: s.end } : { start: s.end, end: s.start }
}

export function StudyPeriodEditor({
  mode,
  setMode,
  selection,
  setSelection,
}: StudyPeriodEditorProps) {
  const queryClient = useQueryClient()
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const normalized = normalizeRange(selection)
  const canConfirm = normalized !== null && mode === 'editing'

  const mutation = useMutation({
    mutationFn: async (range: { start: string; end: string }) => {
      // year_month 는 시작일 기준 (PRD §4.4.2: 월 수업일수 20일 충족 위해 전후월 포함 가능)
      const yearMonth = range.start.slice(0, 7)
      const created = await createStudyPeriod({
        year_month: yearMonth,
        start_date: range.start,
        end_date: range.end,
      })
      // PRD §4.4.2 — 등록 후 즉시 확정 (사용자가 "확정" 버튼 클릭한 흐름)
      await confirmStudyPeriod(created.id)
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['study-periods'] })
      setMode('view')
      setSelection({ start: null, end: null })
      setConfirmOpen(false)
    },
    onError: (err) => {
      setConfirmOpen(false)
      setErrorMessage(err instanceof Error ? err.message : String(err))
    },
  })

  function handleToggleEdit() {
    if (mode === 'editing') {
      setMode('view')
      setSelection({ start: null, end: null })
    } else {
      setMode('editing')
    }
  }

  function statusText(): string {
    if (mode !== 'editing') return ''
    if (!selection.start) return '캘린더에서 시작일을 클릭하세요'
    if (!selection.end) return `시작일 ${selection.start} — 종료일을 클릭하세요`
    return `${normalized!.start} ~ ${normalized!.end} 선택됨`
  }

  return (
    <section
      aria-label="교습기간 설정"
      className="flex flex-col gap-2 rounded-lg border border-[var(--border)] bg-white p-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-lg font-bold text-[var(--foreground)]">교습기간</h2>
        <div className="flex items-center gap-2">
          {mode === 'editing' && canConfirm && (
            <button
              type="button"
              onClick={() => setConfirmOpen(true)}
              disabled={mutation.isPending}
              className="min-h-[44px] rounded-md border border-amber-500 bg-amber-500 px-4 py-2 text-base font-semibold text-white hover:bg-amber-600 disabled:opacity-50"
            >
              확정
            </button>
          )}
          <button
            type="button"
            onClick={handleToggleEdit}
            disabled={mutation.isPending}
            className={[
              'min-h-[44px] rounded-md border px-4 py-2 text-base disabled:opacity-50',
              mode === 'editing'
                ? 'border-gray-400 bg-gray-100 text-gray-700 hover:bg-gray-200'
                : 'border-[var(--border)] bg-white text-[var(--foreground)] hover:bg-gray-50',
            ].join(' ')}
          >
            {mode === 'editing' ? '취소' : '교습기간 설정'}
          </button>
        </div>
      </div>

      {mode === 'editing' && (
        <p className="text-sm text-amber-700">{statusText()}</p>
      )}

      {/* 확정 확인 다이얼로그 — controlled */}
      <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>교습기간 등록</AlertDialogTitle>
            <AlertDialogDescription>
              {normalized && (
                <>
                  <strong>
                    {normalized.start} ~ {normalized.end}
                  </strong>
                  {' '}
                  교습기간을 등록하고 확정합니다.
                  <br />
                  이후 지난 달이 되면 수정·삭제가 차단됩니다.
                </>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={mutation.isPending}>취소</AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => {
                e.preventDefault()
                if (normalized) mutation.mutate(normalized)
              }}
              disabled={mutation.isPending}
            >
              {mutation.isPending ? '등록 중...' : '확정'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 에러 다이얼로그 — 백엔드 한국어 메시지 그대로 노출 */}
      <AlertDialog
        open={errorMessage !== null}
        onOpenChange={(open) => {
          if (!open) setErrorMessage(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>교습기간 등록 실패</AlertDialogTitle>
            <AlertDialogDescription>{errorMessage}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setErrorMessage(null)}>
              확인
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}
