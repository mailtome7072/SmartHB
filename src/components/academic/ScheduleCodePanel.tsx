'use client'

/**
 * 학사 일정 코드 관리 패널 — Sprint 6 T11 (PRD §4.4.3~4.4.5).
 *
 * 코드 목록 + 시스템 코드 토글 + 사용자 코드 CRUD + 코드 선택.
 *
 * 표시 규칙:
 * - 시스템 코드(is_system_reserved=1): 🔒 + 3속성 readonly + 활성 토글만 가능
 * - 사용자 코드: 3속성 편집 가능 + 활성 토글 + (편집 다이얼로그)
 * - 비활성 코드는 흐릿하게 표시되지만 선택 불가
 *
 * 코드 선택 = 라디오 — onCodeSelect(code) 콜백. 활성 코드만 선택 가능.
 * 신규 추가 시 보수적 디폴트 (AC-T11-2): 정규수업 OFF / 보강 OFF / 중복불가 ON / 단일 일자.
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  createScheduleCode,
  listScheduleCodes,
  toggleScheduleCodeActive,
  updateScheduleCode,
} from '@/lib/tauri'
import type { ScheduleCode } from '@/types/academic'
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

/**
 * Sprint 7 T5: selection props 가 옵셔널로 변경.
 * - `/academic` 에서는 [[ScheduleCodeSelector]] 를 사용하므로 본 패널은 더 이상 마운트 안 함.
 * - `/settings/schedule-codes` 페이지에서는 CRUD 만 — props 생략 시 카드 클릭은 무동작.
 */
interface ScheduleCodePanelProps {
  selectedCodeId?: number | null
  onSelect?: (code: ScheduleCode | null) => void
}

interface CodeFormState {
  code_name: string
  allows_regular_class: boolean
  allows_makeup_class: boolean
  is_duplicate_blocked: boolean
  is_period_type: boolean
}

const DEFAULT_FORM: CodeFormState = {
  code_name: '',
  allows_regular_class: false,
  allows_makeup_class: false,
  is_duplicate_blocked: true,
  is_period_type: false,
}

function AttributeRow({ label, value }: { label: string; value: boolean }) {
  return (
    <span
      className={[
        'inline-block rounded px-1.5 py-0.5 text-xs',
        value ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500',
      ].join(' ')}
      title={`${label}: ${value ? 'ON' : 'OFF'}`}
    >
      {label}: {value ? 'ON' : 'OFF'}
    </span>
  )
}

export function ScheduleCodePanel({ selectedCodeId, onSelect }: ScheduleCodePanelProps) {
  const queryClient = useQueryClient()
  const [formOpen, setFormOpen] = useState(false)
  const [editingCode, setEditingCode] = useState<ScheduleCode | null>(null)
  const [form, setForm] = useState<CodeFormState>(DEFAULT_FORM)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const codesQuery = useQuery({
    queryKey: ['schedule-codes'],
    queryFn: listScheduleCodes,
    staleTime: 60_000,
  })

  function invalidate() {
    void queryClient.invalidateQueries({ queryKey: ['schedule-codes'] })
  }

  const toggleMutation = useMutation({
    mutationFn: toggleScheduleCodeActive,
    onSuccess: invalidate,
    onError: (err) =>
      setErrorMessage(err instanceof Error ? err.message : String(err)),
  })

  const saveMutation = useMutation({
    mutationFn: async ({ editing, payload }: { editing: ScheduleCode | null; payload: CodeFormState }) => {
      if (editing) {
        await updateScheduleCode(editing.id, {
          allows_regular_class: payload.allows_regular_class,
          allows_makeup_class: payload.allows_makeup_class,
          is_duplicate_blocked: payload.is_duplicate_blocked,
          is_period_type: payload.is_period_type,
        })
      } else {
        await createScheduleCode(payload)
      }
    },
    onSuccess: () => {
      invalidate()
      setFormOpen(false)
      setEditingCode(null)
      setForm(DEFAULT_FORM)
    },
    onError: (err) => setErrorMessage(err instanceof Error ? err.message : String(err)),
  })

  function openNewForm() {
    setEditingCode(null)
    setForm(DEFAULT_FORM)
    setFormOpen(true)
  }

  function openEditForm(code: ScheduleCode) {
    setEditingCode(code)
    setForm({
      code_name: code.code_name,
      allows_regular_class: code.allows_regular_class,
      allows_makeup_class: code.allows_makeup_class,
      is_duplicate_blocked: code.is_duplicate_blocked,
      is_period_type: code.is_period_type,
    })
    setFormOpen(true)
  }

  const codes = codesQuery.data ?? []
  const systemCodes = codes.filter((c) => c.is_system_reserved)
  const userCodes = codes.filter((c) => !c.is_system_reserved)

  function renderCodeRow(code: ScheduleCode) {
    const isSelected = onSelect !== undefined && code.id === selectedCodeId
    const selectable = code.is_active && onSelect !== undefined
    return (
      <li
        key={code.id}
        className={[
          'flex items-center gap-2 rounded border px-2 py-2',
          isSelected ? 'border-blue-500 bg-blue-50' : 'border-[var(--border)] bg-white',
          !code.is_active ? 'opacity-50' : '',
        ].join(' ')}
      >
        <button
          type="button"
          onClick={() => onSelect?.(selectable ? code : null)}
          disabled={!selectable}
          aria-pressed={isSelected}
          className="flex flex-1 flex-col items-start text-left disabled:cursor-not-allowed"
        >
          <span className="text-base font-semibold text-[var(--foreground)]">
            {code.is_system_reserved && <span title="시스템 예약 코드">🔒 </span>}
            {code.code_name}
          </span>
          <span className="mt-0.5 flex flex-wrap gap-1">
            <AttributeRow label="정규" value={code.allows_regular_class} />
            <AttributeRow label="보강" value={code.allows_makeup_class} />
            <AttributeRow label="중복불가" value={code.is_duplicate_blocked} />
            <AttributeRow label="기간성" value={code.is_period_type} />
          </span>
        </button>
        <div className="flex flex-col gap-1">
          {!code.is_system_reserved && (
            <button
              type="button"
              onClick={() => openEditForm(code)}
              className="rounded border border-[var(--border)] px-2 py-1 text-xs hover:bg-gray-50"
            >
              편집
            </button>
          )}
          <button
            type="button"
            onClick={() => toggleMutation.mutate(code.id)}
            disabled={toggleMutation.isPending}
            className={[
              'rounded border px-2 py-1 text-xs disabled:opacity-50',
              code.is_active
                ? 'border-gray-400 bg-white hover:bg-gray-50'
                : 'border-green-500 bg-green-50 text-green-700 hover:bg-green-100',
            ].join(' ')}
          >
            {code.is_active ? '비활성' : '활성'}
          </button>
        </div>
      </li>
    )
  }

  return (
    <section
      aria-label="학사 일정 코드"
      className="flex flex-col gap-2 rounded-lg border border-[var(--border)] bg-white p-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-lg font-bold text-[var(--foreground)]">학사 일정 코드</h2>
        <button
          type="button"
          onClick={openNewForm}
          className="min-h-[36px] rounded-md border border-[var(--border)] bg-white px-3 py-1 text-sm hover:bg-gray-50"
        >
          + 코드 추가
        </button>
      </div>

      {codesQuery.isLoading && <p className="text-sm text-gray-500">코드 목록 불러오는 중...</p>}
      {codesQuery.isError && (
        <p role="alert" className="text-sm text-red-700">
          코드 목록을 불러오지 못했습니다.
        </p>
      )}

      <ul className="flex flex-col gap-1">
        {systemCodes.map(renderCodeRow)}
        {userCodes.length > 0 && (
          <li className="mt-2 border-t border-[var(--border)] pt-2 text-xs text-gray-500">
            사용자 추가 코드
          </li>
        )}
        {userCodes.map(renderCodeRow)}
      </ul>

      {/* 신규/편집 폼 다이얼로그 */}
      <AlertDialog open={formOpen} onOpenChange={setFormOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {editingCode ? `${editingCode.code_name} 편집` : '새 학사 일정 코드'}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {editingCode
                ? '3속성을 수정합니다. 코드명은 변경할 수 없습니다.'
                : '코드명 + 4 속성을 지정합니다. 보수적 디폴트(정규 OFF / 보강 OFF / 중복불가 ON / 단일 일자) 권장.'}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="flex flex-col gap-3 px-1 py-2">
            {!editingCode && (
              <label className="flex flex-col gap-1 text-sm">
                코드명
                <input
                  type="text"
                  value={form.code_name}
                  onChange={(e) => setForm({ ...form, code_name: e.target.value })}
                  required
                  className="h-10 rounded-md border border-[var(--border)] px-3"
                />
              </label>
            )}
            <fieldset className="flex flex-wrap gap-3 text-sm">
              <legend className="sr-only">3속성 + 기간성</legend>
              {([
                ['allows_regular_class', '정규수업 허용'],
                ['allows_makeup_class', '보강수업 허용'],
                ['is_duplicate_blocked', '중복불가'],
                ['is_period_type', '기간성 (시작·종료일)'],
              ] as const).map(([key, label]) => (
                <label key={key} className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={form[key]}
                    onChange={(e) => setForm({ ...form, [key]: e.target.checked })}
                    className="h-4 w-4"
                  />
                  {label}
                </label>
              ))}
            </fieldset>
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={saveMutation.isPending}>취소</AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => {
                e.preventDefault()
                if (!editingCode && !form.code_name.trim()) {
                  setErrorMessage('코드명을 입력하세요.')
                  return
                }
                saveMutation.mutate({ editing: editingCode, payload: form })
              }}
              disabled={saveMutation.isPending}
            >
              {saveMutation.isPending ? '저장 중...' : '저장'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 에러 다이얼로그 */}
      <AlertDialog
        open={errorMessage !== null}
        onOpenChange={(open) => {
          if (!open) setErrorMessage(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>오류</AlertDialogTitle>
            <AlertDialogDescription>{errorMessage}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setErrorMessage(null)}>확인</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}
