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

import { useEffect, useMemo, useRef, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { listCodes } from '@/lib/tauri'
import { formatPhone, todayLocalISO } from '@/lib/format'
import { useUnsavedChanges } from '@/lib/use-unsaved-changes'
import { ErrorDialog } from '@/components/ui/error-dialog'
import type { CodeEntry } from '@/types/code'
import type { Gender, NewStudent, SchoolLevel, Student } from '@/types/student'

/** 원생 폼 임시저장 localStorage 키 prefix — 목록(students/page)에서 draft 존재 표시에 공유. */
export const STUDENT_DRAFT_PREFIX = 'smarthb:student-draft:'

interface FormState {
  serial_no: string
  name: string
  // 성별·학년은 신규 등록 시 기본값 없이 '(미지정)' 으로 시작 — 빈 값('') 허용, 저장 시 필수 검증.
  gender: Gender | ''
  school_level: SchoolLevel
  grade: number | ''
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
    gender: '',
    school_level: 'elementary',
    grade: '',
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

/** 검증 통과(gender·grade 비어있지 않음 보장) 후에만 호출된다. */
function formToPayload(f: FormState): NewStudent {
  return {
    serial_no: f.serial_no === '' ? null : f.serial_no,
    name: f.name.trim(),
    gender: f.gender as Gender,
    school_level: f.school_level,
    grade: f.grade as number,
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
  const storageKey = `${STUDENT_DRAFT_PREFIX}${draftKey}`
  const isEdit = initial !== undefined
  const { data: schools = [], isLoading: schoolsLoading } = useQuery<CodeEntry[]>({
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

  // Sprint 19 T9(사용자 요청) — 학교급(school_level)에 맞는 학교만 드롭다운에 노출.
  // school_type='etc'(미분류) 또는 null 인 학교는 학교급 무관하게 항상 노출(기타 학교).
  const filteredSchools = useMemo(
    () =>
      schools.filter(
        (s) => s.is_active && (s.extra === form.school_level || s.extra === 'etc' || s.extra === null),
      ),
    [schools, form.school_level],
  )
  // 학교급 변경으로 현재 선택된 학교가 목록에서 사라지면 선택값 초기화 — 학교급과 안 맞는
  // 학교가 그대로 남아있는 채 저장되는 것을 방지 (기존 텍스트 매칭 경고 로직을 대체).
  // schoolsLoading 가드 필수 — 목록 로딩 완료 전(schools=[]) 에는 기존 선택값을
  // "목록에 없음"으로 오판해 수정 모드 진입 직후 school_id 를 지워버리는 사고를 방지.
  useEffect(() => {
    if (schoolsLoading || form.school_id === null) return
    if (filteredSchools.some((s) => s.id === form.school_id)) return
    setForm((f) => ({ ...f, school_id: null }))
  }, [schoolsLoading, filteredSchools, form.school_id])

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

  // 입력하는 즉시 임시저장 (2026-06: 3분 interval → 변경 즉시). 기존 interval 은 이탈 시점에
  // 저장이 안 돼, 입력 후 곧장 떠나면(이동/취소 경고에서 '이동') 복원할 draft 가 없었다.
  // 이제 변경마다 localStorage 에 저장 → 이탈·창닫기 후 재진입 시 '이어서 작성' 배너로 복원.
  useEffect(() => {
    if (!dirty) return
    localStorage.setItem(storageKey, JSON.stringify(form))
  }, [form, dirty, storageKey])

  const submitForm = async () => {
    if (submitting) return
    setError(null)
    const f = formRef.current
    // 필수 입력 검증 — 성별·학년은 (미지정) 상태면 저장 차단하고 입력 유도.
    if (f.name.trim() === '') {
      setError('이름을 입력해주세요.')
      return
    }
    if (f.gender === '') {
      setError('성별을 선택해주세요.')
      return
    }
    if (f.grade === '' || Number.isNaN(f.grade)) {
      setError('학년을 입력해주세요.')
      return
    }
    if (f.school_id === null) {
      setError('학교를 선택해주세요.')
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
      {/* 작성 중 자동 임시저장 안내 — 이탈/창닫기 후 다시 와도 이어서 작성 가능함을 사전 인지. */}
      {dirty && pendingDraft === null && (
        <p role="status" className="text-sm text-muted-foreground">
          ✓ 작성 중인 내용은 자동으로 임시저장됩니다. 중간에 나가더라도 다시 들어오면 이어서 작성할 수 있어요.
        </p>
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
              isEdit ? 'cursor-not-allowed bg-gray-100 text-muted-foreground' : 'bg-white'
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
        <Field label="성별 *">
          <select
            value={form.gender}
            onChange={(e) => update('gender', e.target.value as Gender | '')}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          >
            <option value="">(미지정)</option>
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
        <Field label="학년 *">
          {/* 신규 시 (미지정) = 빈 값. P2-14: 학교급에 따라 상한 — 초등 1~6 / 중등 1~3 (DB CHECK 1~9). */}
          <input
            type="number"
            value={form.grade}
            onChange={(e) => update('grade', e.target.value === '' ? '' : Number(e.target.value))}
            placeholder="(미지정)"
            min={1}
            max={form.school_level === 'middle' ? 3 : 6}
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          />
        </Field>
        <Field label="학교 *">
          <select
            value={form.school_id ?? ''}
            onChange={(e) =>
              update('school_id', e.target.value === '' ? null : Number(e.target.value))
            }
            className="h-11 w-full rounded-md border border-[var(--border)] bg-white px-3"
          >
            <option value="">(미지정)</option>
            {filteredSchools.map((s) => (
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

      {/* 필수 입력·정합성·저장 실패 메시지는 팝업으로 표시 (인라인 대체) */}
      <ErrorDialog open={error !== null} message={error ?? ''} onClose={() => setError(null)} />

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
