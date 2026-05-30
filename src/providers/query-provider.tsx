'use client'

/**
 * TanStack Query Provider (Sprint 3 T4).
 *
 * IPC 응답(`listStudents`, `listCodes`, `getAuditLogs` 등)을 캐싱·재검증한다.
 *
 * ## 기본 정책
 *
 * - `staleTime: 0` — 항상 stale. 다른 화면/탭에서 수정한 데이터가 화면 진입(mount)·재포커스 시
 *   즉시 반영되도록 한다. 로컬 SQLite 라 재조회 비용이 낮아 30초 캐시로 인한 staleness 보다 일관성 우선.
 * - `refetchOnWindowFocus: true` — 앱 창에 다시 포커스가 오면 재조회 (양 PC 동기화 후 복귀 등 반영).
 * - `refetchOnMount: 'always'` — 화면 진입 시 캐시가 있어도 항상 재조회.
 * - `retry: 1` — Tauri IPC 실패는 백엔드 에러가 대부분이라 재시도 1회로 제한.
 *
 * QueryClient 는 client 컴포넌트 안에서 `useState` 로 1회 생성하여 React StrictMode 의
 * 이중 렌더링에도 클라이언트가 중복 생성되지 않도록 한다.
 */

import { useState } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

export function QueryProvider({ children }: { children: React.ReactNode }) {
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 0,
            refetchOnWindowFocus: true,
            refetchOnMount: 'always',
            retry: 1,
          },
        },
      }),
  )

  return <QueryClientProvider client={client}>{children}</QueryClientProvider>
}
