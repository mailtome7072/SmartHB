'use client'

/**
 * TanStack Query Provider (Sprint 3 T4).
 *
 * IPC 응답(`listStudents`, `listCodes`, `getAuditLogs` 등)을 캐싱·재검증한다.
 *
 * ## 기본 정책
 *
 * - `staleTime: 30000` — 30초 동안 fresh. 단일 사용자·로컬 DB 환경이라 갱신 빈도가 낮다.
 * - `refetchOnWindowFocus: false` — Tauri WebView 는 OS 창 포커스 이벤트가 잦지 않아 의미 없음.
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
            staleTime: 30_000,
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      }),
  )

  return <QueryClientProvider client={client}>{children}</QueryClientProvider>
}
