'use client'

/**
 * 원생 등록/수정 공통 폼 (Sprint 3 T11, PRD §4.1).
 *
 * `initial` 이 주어지면 수정 모드(submitLabel="저장"), 없으면 신규 모드("등록").
 * 임시저장은 3분마다 localStorage 에 저장하고 이탈 시 미저장 변경분에 대해 경고를 띄운다
 * (PRD §5.7).
 *
 * 백엔드 IPC 호출은 호출자가 `onSubmit` 핸들러에서 수행 — 폼은 valid payload 만 만들어
 * 전달하고 결과(성공/실패) 표시 책임도 호출자가 갖는다. 라우터 의존성을 컴포넌트 외부로
 * 빼서 신규/수정/모달 등 다양한 호출 패턴을 단순화한다.
 */

import { useEffect, useRef, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { listCodes } from '@/lib/tauri'
import { formatPhone, todayLocalISO } from '@/lib/format'
import { useUnsavedChanges } from '@/lib/use-unsaved-changes'
import type { CodeEntry } from '@/types/code'
import type { Gender, NewStudent, SchoolLevel, Student } from '@/types/student'

const AUTOSAVE_INTERVAL_MS = 3 * 60 * 1000
const STORAGE_PREFIX = 'smarthb:student-draft:'

interface FormState {
  serial_no: string
  name: string
  gender: Gender
  school_level: SchoolLevel
  grade: number
  school_id: number | null
  phone_student: string
  phone_mother: string
  phone_father: string
  birth_date: string
  enroll_date: string
}

function emptyForm(): FormState {
  return {
    serial_no: '',
    name: '',
    gender: 'male',
    school_level: 'elementary',
    grade: 1,
    school_id: null,
    phone_student: '',
    phone_mother: '',
    phone_father: '',
    birth_date: '',
    // P0-3: 로컬 기준 오늘 — toISOString()은 UTC라 KST 오전 9시 전 어제가 됨
    enroll_date: todayLocalISO(),
  }
}

function studentToForm(s: Student): FormState {
  return {
    serial_no: s.serial_no,
    name: s.name,
    gender: s.gender,
    school_level: s.school_level,
    grade: s.grade,
    school_id: s.school_id ?? null,
    phone_student: s.phone_student ?? '',
    phone_mother: s.phone_mother ?? '',
    phone_father: s.phone_father ?? '',
    birth_date: s.birth_date ?? '',
    enroll_date: s.enroll_date,
  }
}

function formToPayload(f: FormState): NewStudent {
  return {
    serial_no: f.serial_no === '' ? null : f.serial_no,
    name: f.name.trim(),
    gender: f.gender,
    school_level: f.school_level,
    grade: f.grade,
    school_id: f.school_id,
    phone_student: f.phone_student.trim() || null,
    phone_mother: f.phone_mother.trim() || null,
    phone_father: f.phone_father.trim() || null,
    birth_date: f.birth_date === '' ? null : f.birth_date,
    enroll_date: f.enroll_date,
  }
}

export function StudentForm({
  draftKey,
  initial,
  submitLabel,
  onSubmit,
  onCancel,
  extraActions,
}: {
  /** localStorage 임시저장 키 (예: 'new' 또는 student id). */
  draftKey: string
  initial?: Student
  submitLabel: string
  onSubmit: (payload: NewStudent) => Promise<void>
  onCancel?: () => void
  /** 폼 하단 추가 액션(퇴교 처리 등) */
  extraActions?: React.ReactNode
}) {
  const storageKey = `${STORAGE_PREFIX}${draftKey}`
  const isEdit = initial !== undefined
  const { data: schools = [] } = useQuery<CodeEntry[]>({
    queryKey: ['codes', 'schools'],
    queryFn: () => listCodes('schools'),
  })
  // P0-5 (2026-06 코드리뷰): 임시저장본을 무통보 자동 적용하지 않는다 — 항상 서버/기본값으로
  // 초기화하고, draft 발견 시 배너로 "이어서 작성 / 새로 시작"을 사용자가 선택한다 (PRD §5.7).
  // 특히 수정 모드에서 묵은 draft가 DB 최신값을 덮어 보여 데이터가 역행하는 사고를 방지.
  const [form, setForm] = useState<FormState>(() =>
    initial ? studentToForm(initial) : emptyForm(),
  )
  const [pendingDraft, setPendingDraft] = useState<FormState | null>(null)
  const [dirty, setDirty] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const formRef = useRef(form)
  formRef.current = form

  // mount 시 1회 draft 탐지 — 파싱 실패본은 즉시 폐기.
  useEffect(() => {
    const raw = localStorage.getItem(storageKey)
    if (raw === null) return
    try {
      setPendingDraft(JSON.parse(raw) as FormState)
    } catch {
      localStorage.removeItem(storageKey)
    }
  }, [storageKey])

  const resumeDraft = () => {
    if (pendingDraft === null) return
    setForm(pendingDraft)
    setDirty(true)
    setPendingDraft(null)
  }

  const discardDraft = () => {
    localStorage.removeItem(storageKey)
    setPendingDraft(null)
  }

  useEffect(() => {
    if (!dirty) return
    const id = setInterval(() => {
      localStorage.setItem(storageKey, JSON.stringify(formRef.current))
    }, AUTOSAVE_INTERVAL_MS)
    return () => clearInterval(id)
  }, [dirty, storageKey])

  const submitForm = async () => {
    if (submitting) return
    setError(null)
    if (formRef.current.name.trim() === '') {
      setError('이름을 입력해주세요.')
      return
    }
    setSubmitting(true)
    try {
      await onSubmit(formToPayload(formRef.current))
      localStorage.removeItem(storageKey)
      setDirty(false)
    } catch (err) {
      setError(
        typeof err === 'string'
          ? err
          : err instanceof Error
            ? err.message
            : '저장 중 오류가 발생했습니다.',
      )
    } finally {
      setSubmitting(false)
    }
  }

  // 미저장 이탈 경고 + Ctrl+S 저장 — 공통 훅으로 통일 (Sprint 16 T1 R105, P1-9)
  useUnsavedChanges(dirty, () => void submitForm())

  const update = <K extends keyof FormState>(key: K, value: FormState[K]) => {
    setForm((f) => ({ ...f, [key]: value }))
    setDirty(true)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    void submitForm()
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {/* P0-5: 임시저장본 발견 — 사용자 선택 (이어서 작성 / 새로 시작) */}
      {pendingDraft !== null && (
        <div
          role="alert"
          className="flex flex-wrap items-center justify-between gap-3 rounded-md border-2 border-amber-400 bg-amber-50 p-3"
        >
          <p className="text-base text-amber-900">
            저장하지 않은 <strong>작성 중이던 내용</strong>이 있습니다. 이어서 작성할까요?
            {isEdit && ' (새로 시작을 누르면 현재 저장된 최신 정보가 유지됩니다)'}
          </p>
          <div className="flex shrink-0 gap-2">
            <button
              type="button"
              onClick={resumeDraft}
              className="h-11 rounded-md bg-[var(--accent)] px-4 font-bold text-white hover:bg-[var(--accent-hover)]"
            >
              이어서 작성
            </button>
            <button
              type="button"
              onClick={discardDraft}
              className="h-11 rounded-md border border-[var(--border)] bg-white px-4 hover:bg-gray-50"
            >
              새로 시작
            </button>
          </div>
        </div>
      )}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Field
          label={
            isEdit
              ? '일련번호 (수정 불가 — PI-05)'
              : '일련번호 (비우면 자동 채번)'
          }
        >
          <input
            value={form.serial_no}
            onChange={(e) => !isEdit && update('serial_no', e.target.value)}
            readOnly={isEdit}
            aria-readonly={isEdit}
            className={`h-11 w-full rounded-md border border-[var(--border)] px-3 ${
              isEdit ? 'cursor-not-allowed bg-gray-100 text-gray-500' : 'bg-white'
            }`}
          />
        </Field>
        <Field label="이름 *">
          <input
            value={form.name}
            onChange={(e) => update('name', e.target.value)}
            required
            // V5 (Sprint 7 post-review): 신규 원생 등록 진입 시 이름에 첫 포커스 — 50대 사용자
            // 친화 UX (PRD §5.7). 수정 모드(isEdit)에서는 autoFocus 비활성화하여 다른 동작과 충돌 회피.
            autoFocus={!isEdit}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="성별">
          <select
            value={form.gender}
            onChange={(e) => update('gender', e.target.value as Gender)}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          >
            <option value="male">남</option>
            <option value="female">여</option>
          </select>
        </Field>
        <Field label="학교급">
          <select
            value={form.school_level}
            onChange={(e) => update('school_level', e.target.value as SchoolLevel)}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          >
            <option value="elementary">초등</option>
            <option value="middle">중등</option>
          </select>
        </Field>
        <Field label="학년">
          <input
            type="number"
            value={form.grade}
            onChange={(e) => update('grade', Number(e.target.value))}
            min={1}
            max={6}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="학교">
          <select
            value={form.school_id ?? ''}
            onChange={(e) =>
              update('school_id', e.target.value === '' ? null : Number(e.target.value))
            }
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          >
            <option value="">(미지정)</option>
            {schools.filter((s) => s.is_active).map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
        </Field>
        <Field label="생년월일">
          <input
            type="date"
            value={form.birth_date}
            onChange={(e) => update('birth_date', e.target.value)}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="입교일">
          <input
            type="date"
            value={form.enroll_date}
            onChange={(e) => update('enroll_date', e.target.value)}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="원생 연락처">
          <input
            value={form.phone_student}
            onChange={(e) => update('phone_student', formatPhone(e.target.value))}
            placeholder="010-1234-5678"
            inputMode="tel"
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="모 연락처">
          <input
            value={form.phone_mother}
            onChange={(e) => update('phone_mother', formatPhone(e.target.value))}
            placeholder="010-1234-5678"
            inputMode="tel"
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="부 연락처">
          <input
            value={form.phone_father}
            onChange={(e) => update('phone_father', formatPhone(e.target.value))}
            placeholder="010-1234-5678"
            inputMode="tel"
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
      </div>

      {error !== null && (
        <p role="alert" className="text-sm text-[var(--danger)]">
          {error}
        </p>
      )}

      <div className="flex items-center justify-between">
        <div>{extraActions}</div>
        <div className="flex gap-2">
          {onCancel !== undefined && (
            <button
              type="button"
              onClick={onCancel}
              className="h-11 rounded-md border border-[var(--border)] px-4"
            >
              취소
            </button>
          )}
          <button
            type="submit"
            disabled={submitting}
            className="h-11 rounded-md bg-[var(--accent)] px-4 font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
          >
            {submitting ? '저장 중...' : submitLabel}
          </button>
        </div>
      </div>
    </form>
  )
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-sm font-bold text-gray-700">{label}</span>
      {children}
    </label>
  )
}
