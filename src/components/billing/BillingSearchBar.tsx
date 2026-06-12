'use client'

/**
 * 청구·수납 통합 검색 바 (Sprint 16 — 청구/수납 페이지 공유).
 * 원생 이름 / 연락처(- 제거 후 완전 일치) / 입금자 이름. Enter=검색, Esc=초기화.
 */

interface BillingSearchBarProps {
  searchInput: string
  setSearchInput: (s: string) => void
  appliedSearch: string
  applySearch: () => void
  clearSearch: () => void
  resultCount: number
}

export function BillingSearchBar({
  searchInput,
  setSearchInput,
  appliedSearch,
  applySearch,
  clearSearch,
  resultCount,
}: BillingSearchBarProps) {
  return (
    <>
      <input
        type="search"
        value={searchInput}
        onChange={(e) => setSearchInput(e.target.value)}
        onKeyDown={(e) => {
          if (e.nativeEvent.isComposing) return
          if (e.key === 'Enter') applySearch()
          else if (e.key === 'Escape') clearSearch()
        }}
        placeholder="이름 / 연락처 / 입금자"
        aria-label="청구·수납 통합 검색"
        className="h-11 w-56 rounded-md border-2 border-[var(--border)] px-3 text-base"
      />
      <button
        type="button"
        onClick={applySearch}
        disabled={searchInput.trim() === ''}
        className="h-11 rounded-md border border-[var(--accent)] px-3 text-base text-[var(--accent)] hover:bg-blue-50 disabled:opacity-50"
      >
        검색
      </button>
      {appliedSearch !== '' && (
        <button
          type="button"
          onClick={clearSearch}
          className="h-11 rounded-md border border-[var(--border)] px-3 text-base text-gray-700 hover:bg-gray-50"
        >
          초기화 ({resultCount}건)
        </button>
      )}
    </>
  )
}
