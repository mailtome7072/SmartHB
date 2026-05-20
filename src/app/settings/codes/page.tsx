'use client'

/**
 * 코드 테이블 관리 화면 (Sprint 3 T12, PRD §4.12).
 *
 * 4 탭: 학교 / 표준교습비 / 결제수단 / 카드사. 표준교습비는 별도 IPC(list_fees) 라서
 * 본 화면 안에서 분리 처리.
 *
 * MVP 기능: 항목 추가·라벨/정렬순서 수정·사용안함 토글. 정렬 드래그&드롭(reorderCodes)
 * 은 Phase 2+ 에서 별도 UI 컴포넌트로 분리.
 */

import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { AppShell } from '@/components/layout/app-shell'
import { GlobalSearch } from '@/components/layout/global-search'
import {
  createCode,
  createFee,
  listCodes,
  listFees,
  updateCode,
  updateFee,
} from '@/lib/tauri'
import type { CodeEntry, CodeTable } from '@/types/code'
import type { StandardFee } from '@/types/fee'

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

  const create = useMutation({
    mutationFn: async () => {
      await createCode(codeTable, {
        code: newCode.trim(),
        label: newLabel.trim() || newCode.trim(),
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

      <ul className="overflow-hidden rounded-md border border-[var(--border)] bg-white">
        {codes.length === 0 && (
          <li className="px-3 py-6 text-center text-sm text-gray-500">등록된 항목이 없습니다.</li>
        )}
        {codes.map((c) => (
          <CodeRow key={c.id} entry={c} onSave={(updated) => update.mutate(updated)} />
        ))}
      </ul>
    </section>
  )
}

/**
 * 코드 행 — local state 로 label/sort_order 입력을 보관하고 onBlur 에서만 onSave 호출.
 * 매 키 입력마다 IPC 호출되는 것을 방지 (50명 × 키 입력 IPC 폭증 해소).
 * is_active 체크박스는 1회 클릭이라 즉시 저장.
 */
function CodeRow({
  entry,
  onSave,
}: {
  entry: CodeEntry
  onSave: (next: CodeEntry) => void
}) {
  const [label, setLabel] = useState(entry.label)
  const [sortOrder, setSortOrder] = useState(entry.sort_order)

  const commit = (next: Partial<CodeEntry>) => {
    onSave({ ...entry, label, sort_order: sortOrder, ...next })
  }

  return (
    <li className="flex items-center gap-3 border-t border-[var(--border)] px-3 py-2 first:border-t-0">
      <span className="w-32 text-sm text-gray-500">{entry.code}</span>
      <input
        value={label}
        onChange={(e) => setLabel(e.target.value)}
        onBlur={() => label !== entry.label && commit({ label })}
        className="h-10 flex-1 rounded-md border border-[var(--border)] px-3"
      />
      <input
        type="number"
        value={sortOrder}
        onChange={(e) => setSortOrder(Number(e.target.value))}
        onBlur={() => sortOrder !== entry.sort_order && commit({ sort_order: sortOrder })}
        className="h-10 w-20 rounded-md border border-[var(--border)] px-3"
        aria-label="정렬순서"
      />
      <label className="flex h-10 items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={entry.is_active}
          onChange={(e) => commit({ is_active: e.target.checked })}
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

      <ul className="overflow-hidden rounded-md border border-[var(--border)] bg-white">
        {fees.length === 0 && (
          <li className="px-3 py-6 text-center text-sm text-gray-500">등록된 표준 교습비가 없습니다.</li>
        )}
        {fees.map((f) => (
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
        className="h-10 w-40 rounded-md border border-[var(--border)] px-3"
        aria-label="금액"
      />
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
