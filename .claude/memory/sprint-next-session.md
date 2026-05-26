---
name: sprint-next-session
description: "Sprint 10 Session #10 완료 (T9 앱 시작 토스트). 다음: T10 — 퇴교 보강 UI 다이얼로그"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session10-t9
---

Sprint 10 — Phase 3 완결 sprint. Session #10 (T9) 완료. 다음은 **T10 퇴교 보강 UI**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1-T8 | 백엔드 전부 완료 | ✅ | — |
| T9 | 앱 시작 expiration_report 토스트 | ✅ | `b7b6fcb` |
| **T10** | **퇴교 보강 UI 다이얼로그** | ⬜ 다음 세션 | — |
| T11 | 캘린더 뷰 UI (FullCalendar) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T9 결과 요약

| 영역 | 변동 |
|------|------|
| 타입 | StartupResult.expiration_report 필드 추가 |
| 상태 | session-store.expirationNoticeDismissed 플래그 + dismissExpirationNotice 액션 |
| UI | app/page.tsx amber 배너 + 닫기 버튼 (transitionedCount > 0 시만, PI-09) |
| 패턴 일관성 | attendance/page.tsx + StudyPeriodEditor 와 동일 토스트 디자인 |
| 자동 검증 | pnpm lint / tsc clean |

## T10 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T10 작업 계획 (3h, sprint10.md L262-279)

PRD §4.5.9 퇴교 처리 다이얼로그. T6 백엔드 IPC 활용.

1. **TS 타입 신규** `src/types/withdrawal.ts`:
   - `WithdrawalChoice` (Tagged union: `immediate_expire` / `external_expire`)
   - `WithdrawalPendingMakeup` + `PendingAbsenceForWithdrawal`

2. **TS 래퍼** `src/lib/tauri/index.ts`:
   - `getPendingMakeupForWithdrawal(studentId)`
   - `processWithdrawalMakeup(studentId, choice, withdrawDate)`

3. **신규 컴포넌트** `src/components/students/WithdrawalMakeupDialog.tsx`:
   - 학생 퇴교 버튼 클릭 시 `get_pending_makeup_for_withdrawal` 호출
   - 빈 리스트면 다이얼로그 미표시, 바로 `withdraw_student` 호출 (기존 흐름)
   - 결석 있으면 다이얼로그:
     - 표시: 원생명, 잔여 보강필요시간, 미보강 결석 일자 리스트
     - 3가지 선택지:
       - "즉시 소멸" → ImmediateExpire
       - "보강 후 퇴교" → 다이얼로그 닫기 (IPC 호출 없음, PI-08 결정)
       - "외부 처리 후 소멸" → memo textarea + ExternalExpire { memo }
   - 성공 시 TanStack Query 무효화 (학생 목록 + 출결)

4. **기존 원생 퇴교 흐름과 통합** — 학생 상세 페이지 또는 목록 페이지의 퇴교 버튼

### T10 진입 시 사용자 확인 필요 (선택)
- 퇴교 흐름이 학생 상세 vs 목록 어느 페이지에서 트리거되는지 (현재 코드 확인 필요)
- 다이얼로그 진입점: 퇴교 버튼 직접 호출 vs 별도 "퇴교 처리" 메뉴

## Sprint 10 Capacity 추적

- 계획 40h
- 실측 누적: T1 1.5 + T2 1 + T1' 0.5 + T3 2.5 + T4 3 + T5폐기 0.5 + T6 2 + T7 1 + T8 3 + T9 1 = **15.5h**
- 남은 작업: T10 3h + T11 6h + T12 3h + 버퍼 6h = 18h 필요
- 여유: 약 6.5h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 두 곳 모두 갱신 후 commit
- **FullCalendar 패키지 설치** — T11 진입 직전 (T10 에는 불필요)
