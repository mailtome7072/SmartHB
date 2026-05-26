---
name: sprint-next-session
description: "Sprint 10 Session #9 완료 (T8 캘린더 ADR + IPC). 다음: T9-T11 UI 통합 작업 (소멸 알림 + 퇴교 보강 + 캘린더 뷰)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session9-t8
---

Sprint 10 — Phase 3 완결 sprint. Session #9 (T8) 완료. **백엔드 작업 전부 완료**. 다음은 **T9~T11 UI 통합**.

## Sprint 10 현황 — 백엔드 완료 ✅

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 | ✅ | `b4565d4` |
| T1' | V108 CHECK 단순화 | ✅ | `1efd70f` |
| T3 | 소멸 자동 전이 IPC (7 tests) | ✅ | `616021d` |
| T4 | 트리거 3개소 통합 (+1 test) | ✅ | `6b6cc47` |
| T5 | 환원 IPC | ❌ 폐기 | `1762e71` |
| T6 | 퇴교 보강 IPC (6 tests) | ✅ | `6209d00` |
| T7 | 선행 수업 검증 (+1 test) | ✅ | `840e9c7` |
| T8 | 캘린더 ADR-006 + IPC (6 tests) | ✅ | `21f8719` |
| **T9 (축소)** | **소멸 알림 UI 잔여** (앱 시작 토스트) | ⬜ 다음 세션 | — |
| **T10** | **퇴교 보강 UI** | ⬜ | — |
| **T11** | **캘린더 뷰 UI** (FullCalendar) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T8 결과 요약

| 영역 | 변동 |
|------|------|
| ADR | `docs/arch/adr-006-calendar-library.md` — FullCalendar 채택 (WDM 3.95 vs RBC 3.85) |
| 신규 모듈 | `src-tauri/src/commands/calendar.rs` |
| IPC 2종 | `get_calendar_data` + `get_makeup_management_data` |
| 응답 구조체 | CalendarMonth/Day/Session + MakeupManagementStudent |
| 소멸 임박 판정 | study_periods.end_date 가 (today + 7일) 이내 (AC-4.6-2) |
| 단위 테스트 | 6건 (계획 4건+ 충족) |
| 자동 검증 | cargo test 272 passed (T7 266 → +6) / clippy clean |
| flaky carry-over | `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` — 단독 통과, 병렬 시 가끔 실패. T12 시점 재확인 |

## T9-T11 (다음 세션) UI 통합 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### 작업 분담 추정

**T9 — 소멸 알림 UI 잔여 (1.5h)**:
1. 앱 시작 시 `StartupResult.expiration_report` 를 layout/page 에서 토스트 표시
2. 기존 attendance/page.tsx, StudyPeriodEditor 의 토스트 패턴 활용
3. 환원 다이얼로그는 폐기 결정 (T5 폐기로 이미 처리됨)

**T10 — 퇴교 보강 UI (3h)**:
1. 학생 상세에서 퇴교 클릭 → `get_pending_makeup_for_withdrawal` 호출
2. 미보강 결석 보유 시 다이얼로그 표시 (3가지 선택지)
3. `process_withdrawal_makeup` 호출 + memo 입력 textarea (ExternalExpire)
4. defer 선택은 다이얼로그 닫기 (IPC 호출 없음, PI-08 결정)
5. TS 래퍼 + 타입 + 컴포넌트 신규

**T11 — 캘린더 뷰 UI (6h)** · skill: frontend-design:
1. `@fullcalendar/react` + plugins 설치 — daygrid + timegrid + interaction
2. `/calendar` 페이지 + 사이드바 메뉴 추가
3. 일/주/월 뷰 전환 (Outlook 스타일)
4. 원생 상세 팝업 (PRD §4.6.2)
5. 보강 관리 뷰 (소멸 임박 강조)
6. dynamic import + 'use client' (Next.js static export 호환)
7. TS 래퍼 + 타입 추가

### TS 타입 신규 필요
- `src/types/calendar.ts` — CalendarMonth/Day/Session, MakeupManagementStudent
- `src/types/withdrawal.ts` — WithdrawalChoice, WithdrawalPendingMakeup

## Sprint 10 Capacity 추적

- 계획 40h
- 실측 누적: T1 1.5 + T2 1 + T1' 0.5 + T3 2.5 + T4 3 + T5폐기 0.5 + T6 2 + T7 1 + T8 3 = **15h**
- 남은 capacity: 약 25h (T9 1.5h + T10 3h + T11 6h + T12 3h + 6h 시각 검증 버퍼 = 19.5h 필요. 약 5.5h 여유)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 두 곳 모두 갱신 후 commit
- **FullCalendar 패키지 설치 시점** — T11 진입 직전 pnpm add. dynamic import + 'use client' 필수
