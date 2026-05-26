---
name: sprint-next-session
description: "Sprint 10 Session #11 완료 (T10 퇴교 보강 UI). 다음: T11 — 캘린더 뷰 UI (FullCalendar 6h)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session11-t10
---

Sprint 10 — Phase 3 완결 sprint. Session #11 (T10) 완료. 다음은 **T11 캘린더 뷰 UI** (FullCalendar 6h, sprint 의 큰 작업).

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1-T8 (백엔드) | 완료 | ✅ | — |
| T9 | 앱 시작 토스트 | ✅ | `b7b6fcb` |
| T10 | 퇴교 보강 UI | ✅ | `7de4dbb` |
| **T11** | **캘린더 뷰 UI (FullCalendar)** | ⬜ 다음 세션 | — |
| T12 | 통합 검증 | ⬜ | — |

## T10 결과 요약

| 영역 | 변동 |
|------|------|
| 타입 | `src/types/withdrawal.ts` 신규 — WithdrawalChoice tagged union + WithdrawalPendingMakeup |
| 래퍼 | `getPendingMakeupForWithdrawal` + `processWithdrawalMakeup` |
| 컴포넌트 | `WithdrawalMakeupDialog` — 3가지 선택지 (메뉴 → external 모드 전환), memo textarea 필수 |
| 통합 | edit page handleWithdrawConfirmed — 잔여 보강 검증 후 분기 |
| 자동 검증 | pnpm lint clean / pnpm tsc clean |

## T11 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T11 작업 계획 (6h, sprint10.md L283-307)

1. **패키지 설치**
   ```bash
   pnpm add @fullcalendar/core @fullcalendar/react @fullcalendar/daygrid @fullcalendar/timegrid @fullcalendar/interaction
   ```

2. **TS 타입 신규** `src/types/calendar.ts`:
   - CalendarMonth, CalendarDay, CalendarSession
   - MakeupManagementStudent

3. **TS 래퍼** `src/lib/tauri/index.ts`:
   - `getCalendarData(yearMonth)`
   - `getMakeupManagementData(yearMonth)`

4. **신규 라우트** `src/app/calendar/page.tsx`:
   - dynamic import + 'use client' (static export 호환)
   - 한국어 로케일 (`@fullcalendar/core/locales/ko`)
   - 일/주/월 뷰 전환 (FullCalendar 표준)
   - 시간대별 인원수 (시작 + 진행 중 합산) — 헤더 셀 커스텀
   - 원생 상세 팝업 (eventClick → 새 다이얼로그)

5. **보강 관리 뷰** — `/calendar` 페이지 내 토글 또는 별도 섹션:
   - 보강 필요 원생 리스트 (소멸기한 임박 순)
   - is_imminent 강조 (색상)

6. **사이드바 메뉴 추가** — `/calendar` 진입점

7. **신규 컴포넌트**:
   - `CalendarPage` (주 컴포넌트)
   - `StudentSessionPopup` (원생 상세 — 이름, 학년, 정규/보강, 시간, 결석일, 미수업 시간, 소멸기한)
   - `MakeupManagementSection` (보강 관리 뷰)

### T11 진입 시 확인 사항
- React 19 호환 — `@fullcalendar/react@6.x` 동작 확인. 문제 발생 시 ADR-006 미해결 사항에 따라 6.1.x 핀.
- Next.js static export 호환 — `dynamic(() => import('...'), { ssr: false })` 패턴 필수.
- `'use client'` 페이지 자체 + 내부 컴포넌트 분리

### TS 타입 예시 (src/types/calendar.ts)
```ts
export interface CalendarMonth {
  yearMonth: string
  days: CalendarDay[]
}
export interface CalendarDay {
  eventDate: string
  regularSessions: CalendarSession[]
  makeupSessions: CalendarSession[]
}
export interface CalendarSession {
  studentId: number
  studentName: string
  startTime: string | null  // 보강은 null
  classMinutes: number
}
export interface MakeupManagementStudent {
  studentId: number
  studentName: string
  serialNo: string
  remainingMinutes: number
  earliestDeadline: string | null
  isImminent: boolean
}
```

## Sprint 10 Capacity 추적

- 계획 40h
- 실측 누적: T1 1.5 + T2 1 + T1' 0.5 + T3 2.5 + T4 3 + T5폐기 0.5 + T6 2 + T7 1 + T8 3 + T9 1 + T10 2 = **18h**
- 남은 작업: T11 6h + T12 3h + 버퍼 6h = 15h 필요
- 여유: 약 7h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 두 곳 모두 갱신 후 commit
- **FullCalendar 패키지 추가** — T11 진입 시 pnpm add. 의존성 추가는 scope.md 에 명시
