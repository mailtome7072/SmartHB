'use client'

/**
 * 결석 이력 다이얼로그 — Sprint 9 T8 (PRD §4.5.10).
 *
 * 출결표 학생명 클릭으로 진입. 해당 원생의 결석/보강완료/보강소멸 이력을 표 형태로 표시.
 * 처리 상태별 시각 구분 (AC-4.5-7):
 * - 미처리 결석: 빨간 배경
 * - 보강완료: 초록 배경 + 보강일/시간
 * - 보강소멸: 회색 배경 + "소멸" 라벨
 *
 * 원생 상세 라우트(`/students/[id]`)가 아직 없어 출결 화면 내 다이얼로그로 배치.
 * 차기 sprint 에서 라우트 도입 시 본 컴포넌트 재사용 가능.
 */

import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { getAbsenceHistory } from '@/lib/tauri'
import type { AbsenceHistoryItem } from '@/types/makeup'

interface Props {
  studentId: number
  studentName: string
  studentSerialNo: string
  onClose: () => void
}

export function AbsenceHistoryDialog({
  studentId,
  studentName,
  studentSerialNo,
  onClose,
}: Props) {
  const query = useQuery({
    queryKey: ['absence-history', studentId],
    queryFn: () => getAbsenceHistory(studentId),
  })

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault()
        onClose()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="absence-history-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-3xl rounded-lg bg-white p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="absence-history-title" className="text-xl font-bold">
          결석 이력
        </h2>
        <p className="mt-1 text-sm text-gray-600">
          <span className="font-semibold">{studentName}</span>
          <span className="ml-1 text-gray-500">#{studentSerialNo}</span>
        </p>

        {query.isLoading && (
          <p className="mt-4 text-base text-gray-600">결석 이력 조회 중...</p>
        )}
        {query.isError && (
          <p
            role="alert"
            className="mt-4 rounded-md border-2 border-[var(--danger)] bg-red-50 p-3 text-base text-[var(--danger)]"
          >
            결석 이력 조회 실패: {(query.error as Error).message}
          </p>
        )}
        {query.isSuccess && query.data.length === 0 && (
          <p className="mt-4 rounded-md border border-[var(--border)] bg-gray-50 p-4 text-center text-base text-gray-700">
            결석 이력이 없습니다.
          </p>
        )}
        {query.isSuccess && query.data.length > 0 && (
          <div className="mt-4 max-h-96 overflow-y-auto rounded-md border border-[var(--border)]">
            <table className="w-full border-collapse text-base">
              <thead className="sticky top-0 bg-gray-100">
                <tr>
                  <th className="border-b border-r border-[var(--border)] px-3 py-2 text-left">결석일</th>
                  <th className="border-b border-r border-[var(--border)] px-3 py-2 text-center">수업(분)</th>
                  <th className="border-b border-r border-[var(--border)] px-3 py-2 text-center">상태</th>
                  <th className="border-b border-r border-[var(--border)] px-3 py-2 text-center">보강 정보</th>
                  <th className="border-b border-[var(--border)] px-3 py-2 text-left">사유 메모</th>
                </tr>
              </thead>
              <tbody>
                {query.data.map((item) => (
                  <HistoryRow key={item.id} item={item} />
                ))}
              </tbody>
            </table>
          </div>
        )}

        <div className="mt-6 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="min-h-[44px] rounded-md border border-[var(--border)] bg-white px-4 text-base text-gray-700 hover:bg-gray-50"
          >
            닫기
          </button>
        </div>
      </div>
    </div>
  )
}

interface HistoryRowProps {
  item: AbsenceHistoryItem
}

function HistoryRow({ item }: HistoryRowProps) {
  const cls = statusRowClass(item.status)
  return (
    <tr className={cls.bg}>
      <td className="border-b border-r border-[var(--border)] px-3 py-2 font-medium">
        {item.eventDate}
      </td>
      <td className="border-b border-r border-[var(--border)] px-3 py-2 text-center">
        {item.classMinutes}
      </td>
      <td className="border-b border-r border-[var(--border)] px-3 py-2 text-center font-semibold">
        <span className={cls.label}>{cls.text}</span>
      </td>
      <td className="border-b border-r border-[var(--border)] px-3 py-2 text-center text-sm">
        {item.status === 'makeup_done' && item.makeupEventDate !== null ? (
          <>
            <span className="font-semibold">{item.makeupEventDate}</span>
            {item.makeupClassMinutes !== null && (
              <span className="ml-1 text-gray-600">({item.makeupClassMinutes}분)</span>
            )}
          </>
        ) : item.status === 'absent' && item.makeupDeadline !== null ? (
          <span className="text-amber-700">소멸기한 {item.makeupDeadline}</span>
        ) : (
          <span className="text-gray-400">—</span>
        )}
      </td>
      <td className="border-b border-[var(--border)] px-3 py-2 text-sm text-gray-700">
        {item.absenceMemo ?? <span className="text-gray-400">—</span>}
      </td>
    </tr>
  )
}

function statusRowClass(status: AbsenceHistoryItem['status']): {
  bg: string
  label: string
  text: string
} {
  switch (status) {
    case 'absent':
      return {
        bg: 'bg-red-50',
        label: 'text-red-700',
        text: '미처리',
      }
    case 'makeup_done':
      return {
        bg: 'bg-green-50',
        label: 'text-green-700',
        text: '보강완료',
      }
    case 'makeup_expired':
      return {
        bg: 'bg-gray-100',
        label: 'text-gray-600',
        text: '소멸',
      }
  }
}
