'use client'

/**
 * 원생 목록 화면 (Sprint 3 T10, PRD §4.1).
 *
 * 필터·정렬·페이지네이션이 모두 URL state 가 아닌 컴포넌트 state — Phase 2+ 라우터 통합
 * 시점에 useSearchParams 로 이전. 현 단계는 단일 화면 SPA-like.
 *
 * - TanStack Query 가 listStudents/countStudents 두 IPC 를 캐싱·재검증. 동일 필터 키로
 *   `keepPreviousData` 효과(staleTime 30s) — 페이지 전환 시 깜빡임 최소.
 * - 200ms `useDeferredValue` 디바운스 (T6 와 동일 패턴) 로 이름 검색 입력 반응성 확보.
 * - 44×44px 행, 본문 18pt, WCAG AA — PRD §5.7.
 */

import { useDeferredValue, useState } from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import { countStudents, listCodes, listStudents } from '@/lib/tauri'
import type { CodeEntry } from '@/types/code'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import type {
  Gender,
  SchoolLevel,
  Student,
  StudentFilter,
  StudentSort,
} from '@/types/student'

const PAGE_SIZE = 50

const GENDER_LABEL: Record<Gender, string> = { male: '남', female: '여' }
const LEVEL_LABEL: Record<SchoolLevel, string> = { elementary: '초', middle: '중' }

const DAY_LABEL_SHORT = ['', '월', '화', '수', '목', '금', '토', '일']

/** "1,3,5" → "월/수/금". null/빈 = "-". 중복 요일은 dedupe (스케줄 표시 안정성). */
function formatScheduleDays(csv: string | null | undefined): string {
  if (!csv) return '-'
  const uniq = Array.from(new Set(csv.split(',').map((d) => Number(d.trim()))))
    .filter((d) => d >= 1 && d <= 7)
    .sort((a, b) => a - b)
  if (uniq.length === 0) return '-'
  return uniq.map((d) => DAY_LABEL_SHORT[d]).join('/')
}
const SORT_OPTIONS: { value: StudentSort; label: string }[] = [
  { value: 'serial-asc', label: '번호순' },
  { value: 'serial-desc', label: '번호 역순' },
  { value: 'name-asc', label: '이름순' },
  { value: 'name-desc', label: '이름 역순' },
  { value: 'grade-asc', label: '학년순' },
  { value: 'grade-desc', label: '학년 역순' },
  { value: 'enroll-date-asc', label: '오래된 입교순' },
  { value: 'enroll-date-desc', label: '최근 입교순' },
]

/** 헤더 클릭으로 정렬 가능한 컬럼 매핑 (T11 사용자 요청 #3). */
const SORTABLE_COLUMNS: Record<string, { asc: StudentSort; desc: StudentSort }> = {
  serial: { asc: 'serial-asc', desc: 'serial-desc' },
  name: { asc: 'name-asc', desc: 'name-desc' },
  grade: { asc: 'grade-asc', desc: 'grade-desc' },
  enroll: { asc: 'enroll-date-asc', desc: 'enroll-date-desc' },
}

function sortIndicator(sort: StudentSort, col: keyof typeof SORTABLE_COLUMNS): string {
  const map = SORTABLE_COLUMNS[col]
  if (sort === map.asc) return ' ▲'
  if (sort === map.desc) return ' ▼'
  return ''
}

function toggleSort(current: StudentSort, col: keyof typeof SORTABLE_COLUMNS): StudentSort {
  const map = SORTABLE_COLUMNS[col]
  return current === map.asc ? map.desc : map.asc
}

export default function StudentsPage() {
  const router = useRouter()
  const [nameInput, setNameInput] = useState('')
  const nameQuery = useDeferredValue(nameInput)
  const [schoolLevel, setSchoolLevel] = useState<SchoolLevel | ''>('')
  const [grade, setGrade] = useState<string>('')
  const [gender, setGender] = useState<Gender | ''>('')
  const [activeOnly, setActiveOnly] = useState(true)
  const [sort, setSort] = useState<StudentSort>('serial-asc')
  const [page, setPage] = useState(0)
  // T4 (이슈 #3): 학교명 필터
  const [schoolId, setSchoolId] = useState<string>('')
  const { data: schools = [] } = useQuery<CodeEntry[]>({
    queryKey: ['codes', 'schools'],
    queryFn: () => listCodes('schools'),
  })

  const baseFilter: StudentFilter = {
    name_query: nameQuery.length > 0 ? nameQuery : undefined,
    school_level: schoolLevel === '' ? undefined : schoolLevel,
    grade: grade === '' ? undefined : Number(grade),
    gender: gender === '' ? undefined : gender,
    school_id: schoolId === '' ? undefined : Number(schoolId),
    active_only: activeOnly,
    sort,
  }
  const listFilter: StudentFilter = { ...baseFilter, limit: PAGE_SIZE, offset: page * PAGE_SIZE }
  const baseKey = JSON.stringify(baseFilter)

  const { data: students = [], isFetching } = useQuery<Student[]>({
    queryKey: ['students', 'list', baseKey, page],
    queryFn: () => listStudents(listFilter),
  })
  const { data: total = 0 } = useQuery<number>({
    queryKey: ['students', 'count', baseKey],
    queryFn: () => countStudents(baseFilter),
  })

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-5xl">
        <header className="mb-4 flex items-center justify-between">
          <h1 className="text-2xl font-bold">원생 관리</h1>
          <Link
            href="/students/new"
            className="inline-flex h-11 items-center rounded-md bg-[var(--accent)] px-4 text-base font-bold text-white hover:bg-[var(--accent-hover)]"
          >
            신규 등록
          </Link>
        </header>

        <section
          aria-label="필터"
          className="mb-4 grid grid-cols-1 gap-3 rounded-md border border-[var(--border)] bg-white p-4 sm:grid-cols-3"
        >
          <input
            type="search"
            value={nameInput}
            onChange={(e) => {
              setNameInput(e.target.value)
              setPage(0)
            }}
            placeholder="이름 검색"
            aria-label="이름 검색"
            className="h-11 rounded-md border border-[var(--border)] px-3"
          />
          <select
            value={schoolLevel}
            onChange={(e) => {
              setSchoolLevel(e.target.value as SchoolLevel | '')
              setPage(0)
            }}
            aria-label="학교급"
            className="h-11 rounded-md border border-[var(--border)] px-3"
          >
            <option value="">학교급 (전체)</option>
            <option value="elementary">초등</option>
            <option value="middle">중등</option>
          </select>
          <input
            type="number"
            value={grade}
            onChange={(e) => {
              setGrade(e.target.value)
              setPage(0)
            }}
            placeholder="학년"
            aria-label="학년"
            min={1}
            max={6}
            className="h-11 rounded-md border border-[var(--border)] px-3"
          />
          <select
            value={gender}
            onChange={(e) => {
              setGender(e.target.value as Gender | '')
              setPage(0)
            }}
            aria-label="성별"
            className="h-11 rounded-md border border-[var(--border)] px-3"
          >
            <option value="">성별 (전체)</option>
            <option value="male">남</option>
            <option value="female">여</option>
          </select>
          <select
            value={schoolId}
            onChange={(e) => {
              setSchoolId(e.target.value)
              setPage(0)
            }}
            aria-label="학교"
            className="h-11 rounded-md border border-[var(--border)] px-3"
          >
            <option value="">학교 (전체)</option>
            {schools.filter((s) => s.is_active).map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
          <select
            value={sort}
            onChange={(e) => setSort(e.target.value as StudentSort)}
            aria-label="정렬"
            className="h-11 rounded-md border border-[var(--border)] px-3"
          >
            {SORT_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
          <label className="flex h-11 items-center gap-2">
            <input
              type="checkbox"
              checked={activeOnly}
              onChange={(e) => {
                setActiveOnly(e.target.checked)
                setPage(0)
              }}
              className="h-5 w-5"
            />
            재원 중만
          </label>
        </section>

        <section className="overflow-hidden rounded-md border border-[var(--border)] bg-white">
          <table className="w-full">
            <thead className="bg-[var(--background)]">
              <tr className="text-left">
                <th className="px-3 py-3 text-sm font-bold">
                  <button
                    type="button"
                    onClick={() => setSort((cur) => toggleSort(cur, 'serial'))}
                    className="hover:text-[var(--accent)]"
                    aria-label="번호 정렬 토글"
                  >
                    번호{sortIndicator(sort, 'serial')}
                  </button>
                </th>
                <th className="px-3 py-3 text-sm font-bold">
                  <button
                    type="button"
                    onClick={() => setSort((cur) => toggleSort(cur, 'name'))}
                    className="hover:text-[var(--accent)]"
                    aria-label="이름 정렬 토글"
                  >
                    이름{sortIndicator(sort, 'name')}
                  </button>
                </th>
                <th className="px-3 py-3 text-sm font-bold">학교급</th>
                <th className="px-3 py-3 text-sm font-bold">
                  <button
                    type="button"
                    onClick={() => setSort((cur) => toggleSort(cur, 'grade'))}
                    className="hover:text-[var(--accent)]"
                    aria-label="학년 정렬 토글"
                  >
                    학년{sortIndicator(sort, 'grade')}
                  </button>
                </th>
                <th className="px-3 py-3 text-sm font-bold">성별</th>
                <th className="px-3 py-3 text-sm font-bold">수업 시간/요일</th>
                <th className="px-3 py-3 text-sm font-bold">
                  <button
                    type="button"
                    onClick={() => setSort((cur) => toggleSort(cur, 'enroll'))}
                    className="hover:text-[var(--accent)]"
                    aria-label="입교일 정렬 토글"
                  >
                    입교일{sortIndicator(sort, 'enroll')}
                  </button>
                </th>
                <th className="px-3 py-3 text-sm font-bold">생년월일</th>
              </tr>
            </thead>
            <tbody>
              {students.length === 0 && !isFetching && (
                <tr>
                  <td colSpan={8} className="px-3 py-8 text-center text-sm text-muted-foreground">
                    {total === 0 ? '등록된 원생이 없습니다.' : '필터에 맞는 원생이 없습니다.'}
                  </td>
                </tr>
              )}
              {students.map((s) => (
                <tr
                  key={s.id}
                  onClick={() => router.push(`/students/edit?id=${s.id}`)}
                  className="cursor-pointer border-t border-[var(--border)] hover:bg-[var(--background)]"
                >
                  <td className="min-h-[44px] px-3 py-3 text-base">{s.serial_no}</td>
                  <td className="px-3 py-3 text-base">
                    {s.name}
                    {s.withdraw_date !== null && (
                      <span className="ml-2 text-sm text-muted-foreground">(퇴교)</span>
                    )}
                  </td>
                  <td className="px-3 py-3 text-base">{LEVEL_LABEL[s.school_level]}</td>
                  <td className="px-3 py-3 text-base">{s.grade}</td>
                  <td className="px-3 py-3 text-base">{GENDER_LABEL[s.gender]}</td>
                  <td className="px-3 py-3 text-base text-gray-700">
                    {s.weekly_hours !== null && s.weekly_hours !== undefined && s.weekly_hours > 0
                      ? `주 ${s.weekly_hours}시간 · ${formatScheduleDays(s.schedule_days_csv)}`
                      : '-'}
                  </td>
                  <td className="px-3 py-3 text-base">{s.enroll_date}</td>
                  <td className="px-3 py-3 text-base text-gray-700">{s.birth_date ?? '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>

        <nav aria-label="페이지네이션" className="mt-4 flex items-center justify-between">
          <p className="text-sm text-gray-600">
            총 {total} 명 / {page + 1} / {totalPages} 페이지
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={page === 0}
              className="h-11 rounded-md border border-[var(--border)] px-4 disabled:opacity-50"
            >
              이전
            </button>
            <button
              type="button"
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              disabled={page >= totalPages - 1}
              className="h-11 rounded-md border border-[var(--border)] px-4 disabled:opacity-50"
            >
              다음
            </button>
          </div>
        </nav>
      </div>
    </AppShell>
  )
}
