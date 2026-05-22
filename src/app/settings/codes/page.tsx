'use client'

/**
 * 코드 테이블 관리 화면 (Sprint 3 T12 + Sprint 4 T10, PRD §4.12).
 *
 * 4 탭: 학교 / 표준교습비 / 결제수단 / 카드사. 표준교습비는 별도 IPC(list_fees) 라서
 * 본 화면 안에서 분리 처리.
 *
 * T10 (사용자 이슈 #11, #12):
 * - 드래그앤드롭 정렬 (codes 3탭: @dnd-kit/sortable). reorderCodes IPC 일괄 호출
 * - 신규 추가 시 sort_order = MAX + 1 자동 부여 (프론트 계산 — 단일 사용자라 race 무관)
 * - 화면 상단 전체/사용/미사용 라디오 필터 (4탭 모두, client-side)
 *
 * fees DnD 는 reorder_fees IPC 미존재로 본 sprint 미포함 — sort_order 직접 입력 유지.
 */

import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  DndContext,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  SortableContext,
  arrayMove,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import { SettingsHomeLink } from '@/components/settings/SettingsHomeLink'
import {
  createCode,
  createFee,
  listCodes,
  listFees,
  reorderCodes,
  updateCode,
  updateFee,
} from '@/lib/tauri'
import { formatCurrency } from '@/lib/format'
import type { CodeEntry, CodeTable } from '@/types/code'
import type { StandardFee } from '@/types/fee'

type ActiveFilter = 'all' | 'active' | 'inactive'

function ActiveFilterRadio({
  value,
  onChange,
}: {
  value: ActiveFilter
  onChange: (next: ActiveFilter) => void
}) {
  const options: { value: ActiveFilter; label: string }[] = [
    { value: 'all', label: '전체' },
    { value: 'active', label: '사용' },
    { value: 'inactive', label: '미사용' },
  ]
  return (
    <div role="radiogroup" aria-label="활성 상태 필터" className="mb-3 flex gap-3">
      {options.map((o) => (
        <label
          key={o.value}
          className="flex min-h-[44px] cursor-pointer items-center gap-2 px-2"
        >
          <input
            type="radio"
            name="active-filter"
            checked={value === o.value}
            onChange={() => onChange(o.value)}
            className="h-5 w-5"
          />
          <span className="text-sm">{o.label}</span>
        </label>
      ))}
    </div>
  )
}

function applyActiveFilter<T extends { is_active: boolean }>(
  items: T[],
  filter: ActiveFilter,
): T[] {
  if (filter === 'all') return items
  return items.filter((i) => (filter === 'active' ? i.is_active : !i.is_active))
}

type TabId = 'schools' | 'fees' | 'payment-methods' | 'card-companies'

const TABS: { id: TabId; label: string }[] = [
  { id: 'schools', label: '학교' },
  { id: 'fees', label: '표준 교습비' },
  { id: 'payment-methods', label: '결제 수단' },
  { id: 'card-companies', label: '카드사' },
]

export default function CodesPage() {
  const [tab, setTab] = useState<TabId>('schools')

  return (
    <AppShell topBarSlot={<GlobalSearch />}>
      <div className="mx-auto max-w-4xl">
        <SettingsHomeLink />
        <h1 className="mb-4 text-2xl font-bold">설정 — 코드 테이블</h1>

        <nav role="tablist" className="mb-4 flex gap-2 border-b border-[var(--border)]">
          {TABS.map((t) => (
            <button
              key={t.id}
              type="button"
              role="tab"
              aria-selected={tab === t.id}
              onClick={() => setTab(t.id)}
              className={`min-h-[44px] px-4 py-2 text-base ${
                tab === t.id
                  ? 'border-b-2 border-[var(--accent)] font-bold text-[var(--accent)]'
                  : 'text-gray-600'
              }`}
            >
              {t.label}
            </button>
          ))}
        </nav>

        {tab === 'fees' ? <FeesPanel /> : <CodePanel table={tab} />}
      </div>
    </AppShell>
  )
}

function CodePanel({ table }: { table: Exclude<TabId, 'fees'> }) {
  const qc = useQueryClient()
  const codeTable: CodeTable = table
  const { data: codes = [] } = useQuery<CodeEntry[]>({
    queryKey: ['codes', codeTable],
    queryFn: () => listCodes(codeTable),
  })

  const [newCode, setNewCode] = useState('')
  const [newLabel, setNewLabel] = useState('')
  const [activeFilter, setActiveFilter] = useState<ActiveFilter>('all')

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  )

  const create = useMutation({
    mutationFn: async () => {
      // T10: 신규 항목은 맨 마지막 정렬순서 자동 부여 (사용자 이슈 #11 후반)
      const nextOrder = codes.reduce((max, c) => Math.max(max, c.sort_order), 0) + 1
      await createCode(codeTable, {
        code: newCode.trim(),
        label: newLabel.trim() || newCode.trim(),
        sort_order: nextOrder,
      })
    },
    onSuccess: () => {
      setNewCode('')
      setNewLabel('')
      qc.invalidateQueries({ queryKey: ['codes', codeTable] })
    },
  })

  const update = useMutation({
    mutationFn: async (entry: CodeEntry) => {
      await updateCode(codeTable, entry.id, {
        label: entry.label,
        sort_order: entry.sort_order,
        is_active: entry.is_active,
      })
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['codes', codeTable] }),
  })

  const reorder = useMutation({
    mutationFn: async (orders: [number, number][]) => {
      await reorderCodes(codeTable, orders)
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['codes', codeTable] }),
  })

  // 활성 필터 적용 후 sort_order 오름차순 — DnD 가 보이는 행만 정렬 대상으로 둔다.
  const visibleCodes = useMemo(
    () => applyActiveFilter(codes, activeFilter),
    [codes, activeFilter],
  )

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over === null || active.id === over.id) return
    const oldIdx = visibleCodes.findIndex((c) => String(c.id) === String(active.id))
    const newIdx = visibleCodes.findIndex((c) => String(c.id) === String(over.id))
    if (oldIdx < 0 || newIdx < 0) return
    const reorderedVisible = arrayMove(visibleCodes, oldIdx, newIdx)

    // A22 / R26: 필터링 중 DnD 시 visible 만 재할당하면 hidden 과 sort_order 충돌.
    // 전체 codes 의 정렬 위치에서 visible 슬롯에 reorderedVisible 을 차례로 끼워 넣어
    // hidden 행의 상대 순서를 보존한 채 전체에 새 sort_order 1..N 을 부여한다.
    const sortedAll = [...codes].sort((a, b) => a.sort_order - b.sort_order)
    const visibleIds = new Set(visibleCodes.map((c) => c.id))
    let vi = 0
    const merged = sortedAll.map((c) =>
      visibleIds.has(c.id) ? reorderedVisible[vi++] : c,
    )
    const orders: [number, number][] = merged.map((c, idx) => [c.id, idx + 1])
    reorder.mutate(orders)
  }

  return (
    <section>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          if (newCode.trim() === '') return
          create.mutate()
        }}
        className="mb-4 flex flex-wrap gap-2"
      >
        <input
          value={newCode}
          onChange={(e) => setNewCode(e.target.value)}
          placeholder={table === 'schools' ? '학교명' : '코드 (예: cash)'}
          aria-label="새 코드"
          className="h-11 flex-1 rounded-md border border-[var(--border)] px-3"
        />
        {table !== 'schools' && (
          <input
            value={newLabel}
            onChange={(e) => setNewLabel(e.target.value)}
            placeholder="표시 라벨 (예: 현금)"
            aria-label="새 라벨"
            className="h-11 flex-1 rounded-md border border-[var(--border)] px-3"
          />
        )}
        <button
          type="submit"
          disabled={create.isPending}
          className="h-11 rounded-md bg-[var(--accent)] px-4 font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          추가
        </button>
      </form>
      {create.isError && (
        <p role="alert" className="mb-2 text-sm text-[var(--danger)]">
          {String(create.error)}
        </p>
      )}

      <ActiveFilterRadio value={activeFilter} onChange={setActiveFilter} />

      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
        <SortableContext
          items={visibleCodes.map((c) => c.id)}
          strategy={verticalListSortingStrategy}
        >
          <ul className="overflow-hidden rounded-md border border-[var(--border)] bg-white">
            {visibleCodes.length === 0 && (
              <li className="px-3 py-6 text-center text-sm text-gray-500">
                {codes.length === 0 ? '등록된 항목이 없습니다.' : '필터 조건에 해당하는 항목이 없습니다.'}
              </li>
            )}
            {visibleCodes.map((c) => (
              <SortableCodeRow
                key={c.id}
                entry={c}
                onSave={(updated) => update.mutate(updated)}
              />
            ))}
          </ul>
        </SortableContext>
      </DndContext>
    </section>
  )
}

/**
 * 코드 행 — Sprint 4 T10 sortable 버전.
 * - 드래그 핸들(좌측 ⋮⋮)로 정렬 → @dnd-kit/sortable
 * - label 은 onBlur 저장, is_active 는 즉시 저장
 * - sort_order 직접 입력 필드 제거 (DnD 로 대체) — 표시용으로만 우측에 노출
 */
function SortableCodeRow({
  entry,
  onSave,
}: {
  entry: CodeEntry
  onSave: (next: CodeEntry) => void
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id: entry.id })
  const [label, setLabel] = useState(entry.label)

  const style: React.CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.6 : 1,
  }

  return (
    <li
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-3 border-t border-[var(--border)] bg-white px-3 py-2 first:border-t-0"
    >
      <button
        type="button"
        aria-label="순서 변경 드래그 핸들"
        className="flex h-10 w-8 cursor-grab items-center justify-center text-gray-400 hover:text-gray-700 active:cursor-grabbing"
        {...attributes}
        {...listeners}
      >
        <span aria-hidden="true">⋮⋮</span>
      </button>
      <span className="w-32 text-sm text-gray-500">{entry.code}</span>
      <input
        value={label}
        onChange={(e) => setLabel(e.target.value)}
        onBlur={() => label !== entry.label && onSave({ ...entry, label })}
        className="h-10 flex-1 rounded-md border border-[var(--border)] px-3"
      />
      <span className="w-12 text-right text-sm text-gray-400" title="정렬순서">
        {entry.sort_order}
      </span>
      <label className="flex h-10 items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={entry.is_active}
          onChange={(e) => onSave({ ...entry, is_active: e.target.checked })}
          className="h-5 w-5"
        />
        사용
      </label>
    </li>
  )
}

function FeesPanel() {
  const qc = useQueryClient()
  const { data: fees = [] } = useQuery<StandardFee[]>({
    queryKey: ['fees'],
    queryFn: () => listFees(),
  })

  const [hours, setHours] = useState('2')
  const [amount, setAmount] = useState('100000')
  const [activeFilter, setActiveFilter] = useState<ActiveFilter>('all')
  const visibleFees = useMemo(
    () => applyActiveFilter(fees, activeFilter),
    [fees, activeFilter],
  )

  const create = useMutation({
    mutationFn: async () => {
      await createFee({ weekly_hours: Number(hours), amount: Number(amount) })
    },
    onSuccess: () => {
      setHours('2')
      setAmount('100000')
      qc.invalidateQueries({ queryKey: ['fees'] })
    },
  })

  const update = useMutation({
    mutationFn: async (f: StandardFee) => {
      await updateFee(f.id, {
        weekly_hours: f.weekly_hours,
        amount: f.amount,
        sort_order: f.sort_order,
        is_active: f.is_active,
      })
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ['fees'] }),
  })

  return (
    <section>
      <form
        onSubmit={(e) => {
          e.preventDefault()
          create.mutate()
        }}
        className="mb-4 flex flex-wrap gap-2"
      >
        <input
          type="number"
          value={hours}
          onChange={(e) => setHours(e.target.value)}
          placeholder="주 수업시간"
          aria-label="주 수업시간"
          step="0.5"
          min="0.5"
          className="h-11 w-40 rounded-md border border-[var(--border)] px-3"
        />
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="금액(원)"
          aria-label="금액"
          step="1000"
          className="h-11 w-40 rounded-md border border-[var(--border)] px-3"
        />
        <button
          type="submit"
          disabled={create.isPending}
          className="h-11 rounded-md bg-[var(--accent)] px-4 font-bold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
        >
          추가
        </button>
      </form>
      {create.isError && (
        <p role="alert" className="mb-2 text-sm text-[var(--danger)]">
          {String(create.error)}
        </p>
      )}

      <ActiveFilterRadio value={activeFilter} onChange={setActiveFilter} />

      <ul className="overflow-hidden rounded-md border border-[var(--border)] bg-white">
        {visibleFees.length === 0 && (
          <li className="px-3 py-6 text-center text-sm text-gray-500">
            {fees.length === 0 ? '등록된 표준 교습비가 없습니다.' : '필터 조건에 해당하는 항목이 없습니다.'}
          </li>
        )}
        {visibleFees.map((f) => (
          <FeeRow key={f.id} fee={f} onSave={(updated) => update.mutate(updated)} />
        ))}
      </ul>
    </section>
  )
}

/** CodeRow 와 동일 패턴 — 금액은 onBlur 저장, is_active 는 즉시 저장. */
function FeeRow({
  fee,
  onSave,
}: {
  fee: StandardFee
  onSave: (next: StandardFee) => void
}) {
  const [amount, setAmount] = useState(fee.amount)

  return (
    <li className="flex items-center gap-3 border-t border-[var(--border)] px-3 py-2 first:border-t-0">
      <span className="w-28 text-sm text-gray-500">주 {fee.weekly_hours}시간</span>
      <input
        type="number"
        value={amount}
        onChange={(e) => setAmount(Number(e.target.value))}
        onBlur={() => amount !== fee.amount && onSave({ ...fee, amount })}
        step="1000"
        className="h-10 w-40 rounded-md border border-[var(--border)] px-3 text-right"
        aria-label="금액"
      />
      <span className="w-24 text-sm text-gray-500" title="천단위 콤마 표시 — T5 utils 적용 (사용자 이슈 #13)">
        = {formatCurrency(amount)}원
      </span>
      <label className="flex h-10 items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={fee.is_active}
          onChange={(e) => onSave({ ...fee, is_active: e.target.checked })}
          className="h-5 w-5"
        />
        사용
      </label>
    </li>
  )
}
