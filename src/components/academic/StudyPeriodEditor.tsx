'use client'

/**
 * 교습기간 설정 에디터 — Sprint 6 T10 + Sprint 7 T6 (PRD §4.4.2).
 *
 * Sprint 7 T6 (Issue 5): 토글 버튼 제거 + 미확정 월 자동 selection 모드.
 *
 * 흐름:
 *   1) `centerYearMonth` 의 교습기간 확정 여부를 `getStudyPeriod` 로 조회
 *   2) **확정 월**: 읽기 전용 — `start ~ end` 표시 (T8 에서 삭제 버튼 추가 예정)
 *   3) **미확정 월** + 일정 배치 모드 비활성: 셀 클릭이 자동으로 selection 갱신
 *      - 시작일 클릭 → "시작일 YYYY-MM-DD — 종료일을 클릭하세요"
 *      - 종료일 클릭 → "YYYY-MM-DD ~ YYYY-MM-DD 선택됨" + "확정" / "취소" 버튼 노출
 *   4) "확정" 버튼 → AlertDialog 확인 → createStudyPeriod + confirmStudyPeriod IPC
 *   5) 성공 시 ['study-periods'] / ['study-period', yearMonth] 캐시 무효화
 *   6) 실패(중첩 등) 시 백엔드 한국어 메시지 그대로 표시
 *
 * State: 부모(`/academic` page) 가 selection 보유 — ThreeMonthCalendar 와 onCellClick 공유.
 * Editor 는 UI · query · mutation 만 담당.
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  createStudyPeriod,
  confirmStudyPeriod,
  deleteStudyPeriodCascade,
  getCascadeDeletePreview,
  getStudyPeriod,
} from '@/lib/tauri'
import type { CascadeDeletePreview } from '@/types/academic'
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

interface StudyPeriodEditorProps {
  /** 현재 캘린더 중앙 월 (예: "2026-05") — 확정 여부 판정 기준. */
  centerYearMonth: string
  /** 일정 배치 모드 활성 여부 — true 면 교습기간 selection 비활성. */
  eventPlaceMode: boolean
  selection: SelectionRange
  setSelection: (s: SelectionRange) => void
}

/** 정렬된 [시작, 종료] 반환 — start > end 시 자동 swap. */
function normalizeRange(s: SelectionRange): { start: string; end: string } | null {
  if (!s.start || !s.end) return null
  return s.start <= s.end ? { start: s.start, end: s.end } : { start: s.end, end: s.start }
}

/** 현재 년-월 ("YYYY-MM") — 클라이언트 로컬 시간 기준. 백엔드 가드(`current_year_month`) 와 일관. */
function currentYearMonth(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`
}

export function StudyPeriodEditor({
  centerYearMonth,
  eventPlaceMode,
  selection,
  setSelection,
}: StudyPeriodEditorProps) {
  const queryClient = useQueryClient()
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [deletePreview, setDeletePreview] = useState<CascadeDeletePreview | null>(null)

  // 중앙 월의 교습기간 확정 여부 조회 — null 이면 미확정 (자동 selection 모드).
  const periodQuery = useQuery({
    queryKey: ['study-period', centerYearMonth],
    queryFn: () => getStudyPeriod(centerYearMonth),
    staleTime: 30_000,
    enabled: centerYearMonth.length > 0,
  })

  const confirmedPeriod = periodQuery.data ?? null
  const isUnconfirmed = !periodQuery.isLoading && confirmedPeriod === null
  const isSelectionActive = isUnconfirmed && !eventPlaceMode

  const normalized = normalizeRange(selection)
  const canConfirm = normalized !== null && isSelectionActive

  const mutation = useMutation({
    mutationFn: async (range: { start: string; end: string }) => {
      // V1 fix (Sprint 7 post-review): year_month 는 사용자가 보고 있는 중앙 캘린더 월 기준.
      // 시작일이 이전 월(예: 6월 교습기간에 5/29 시작)이어도 year_month=2026-06 으로 저장하여
      // 사용자 의도(=중앙 월의 교습기간)와 일치. 기간(start/end)은 선택값 그대로 보존.
      const created = await createStudyPeriod({
        year_month: centerYearMonth,
        start_date: range.start,
        end_date: range.end,
      })
      // 등록 후 즉시 확정 — 사용자가 "확정" 버튼 클릭한 흐름 (PRD §4.4.2).
      await confirmStudyPeriod(created.id)
    },
    onSuccess: async () => {
      // 두 캐시 키 모두 무효화 — ThreeMonthCalendar 의 list 와 본 컴포넌트의 단일 조회.
      await queryClient.invalidateQueries({ queryKey: ['study-periods'] })
      await queryClient.invalidateQueries({ queryKey: ['study-period'] })
      setSelection({ start: null, end: null })
      setConfirmOpen(false)
    },
    onError: (err) => {
      setConfirmOpen(false)
      setErrorMessage(err instanceof Error ? err.message : String(err))
    },
  })

  // Sprint 7 T8: cascade 삭제 mutation — 공휴일 제외 학사 일정 + 교습기간 일괄 삭제.
  const cascadeDeleteMutation = useMutation({
    mutationFn: async (id: number) => deleteStudyPeriodCascade(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['study-periods'] })
      await queryClient.invalidateQueries({ queryKey: ['study-period'] })
      await queryClient.invalidateQueries({ queryKey: ['schedule-events'] })
      setDeletePreview(null)
    },
    onError: (err) => {
      setDeletePreview(null)
      setErrorMessage(err instanceof Error ? err.message : String(err))
    },
  })

  async function handleRequestDelete() {
    if (!confirmedPeriod) return
    try {
      const preview = await getCascadeDeletePreview(confirmedPeriod.id)
      setDeletePreview(preview)
    } catch (err) {
      setErrorMessage(err instanceof Error ? err.message : String(err))
    }
  }

  function handleCancel() {
    setSelection({ start: null, end: null })
  }

  function statusText(): string {
    if (!isSelectionActive) return ''
    if (!selection.start) return '캘린더에서 교습기간을 선택하세요 — 먼저 시작일을 클릭'
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
        {isSelectionActive && canConfirm && (
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleCancel}
              disabled={mutation.isPending}
              className="min-h-[44px] rounded-md border border-gray-400 bg-white px-4 py-2 text-base text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            >
              취소
            </button>
            <button
              type="button"
              onClick={() => setConfirmOpen(true)}
              disabled={mutation.isPending}
              className="min-h-[44px] rounded-md border border-amber-500 bg-amber-500 px-4 py-2 text-base font-semibold text-white hover:bg-amber-600 disabled:opacity-50"
            >
              확정
            </button>
          </div>
        )}
      </div>

      {periodQuery.isLoading && (
        <p className="text-sm text-gray-500">교습기간 정보 불러오는 중...</p>
      )}

      {confirmedPeriod !== null && (
        <div className="flex flex-wrap items-center justify-between gap-2">
          <p className="text-sm text-[var(--foreground)]">
            <strong>{confirmedPeriod.start_date} ~ {confirmedPeriod.end_date}</strong> 확정됨
          </p>
          {/* Sprint 7 T8: 지난 달이 아닐 때만 삭제 버튼 노출 — 백엔드 가드와 동일 조건. */}
          {confirmedPeriod.year_month >= currentYearMonth() && (
            <button
              type="button"
              onClick={handleRequestDelete}
              disabled={cascadeDeleteMutation.isPending}
              className="min-h-[44px] rounded-md border border-red-500 bg-white px-4 py-2 text-base text-red-700 hover:bg-red-50 disabled:opacity-50"
            >
              교습기간 삭제
            </button>
          )}
        </div>
      )}

      {isSelectionActive && (
        <p className="text-sm text-amber-700">{statusText()}</p>
      )}

      {isUnconfirmed && eventPlaceMode && (
        <p className="text-sm text-gray-500">
          일정 배치 모드 중에는 교습기간을 선택할 수 없습니다. 코드 선택을 해제하세요.
        </p>
      )}

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

      {/* Sprint 7 T8: cascade 삭제 확인 다이얼로그 — preview 영향 건수 표시 */}
      <AlertDialog
        open={deletePreview !== null}
        onOpenChange={(open) => {
          if (!open) setDeletePreview(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>교습기간 삭제 — cascade</AlertDialogTitle>
            <AlertDialogDescription>
              {deletePreview && !deletePreview.deletable && (
                <span className="text-red-700">{deletePreview.reason}</span>
              )}
              {deletePreview && deletePreview.deletable && (
                <>
                  교습기간을 삭제하면 공휴일을 제외한{' '}
                  <strong>{deletePreview.affected_count}건</strong>의 학사 일정이 함께
                  삭제됩니다.
                  <br />
                  보존되는 공휴일: <strong>{deletePreview.holiday_count}건</strong>
                  <br />
                  <br />
                  되돌릴 수 없습니다. 삭제하시겠습니까?
                </>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={cascadeDeleteMutation.isPending}>
              취소
            </AlertDialogCancel>
            {deletePreview?.deletable && confirmedPeriod && (
              <AlertDialogAction
                onClick={(e) => {
                  e.preventDefault()
                  cascadeDeleteMutation.mutate(confirmedPeriod.id)
                }}
                disabled={cascadeDeleteMutation.isPending}
                className="bg-red-600 hover:bg-red-700"
              >
                {cascadeDeleteMutation.isPending ? '삭제 중...' : '삭제'}
              </AlertDialogAction>
            )}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={errorMessage !== null}
        onOpenChange={(open) => {
          if (!open) setErrorMessage(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>교습기간 처리 실패</AlertDialogTitle>
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
