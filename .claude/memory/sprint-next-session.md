---
name: sprint-next-session
description: "Sprint 10 Session #7 완료 (T6 퇴교 보강 IPC). 다음: T7 — 선행 수업 검증 (출결 생성 충돌 방지)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session7-t6
---

Sprint 10 — Phase 3 완결 sprint. Session #7 (T6 퇴교 보강 IPC) 완료. 다음은 **T7 선행 수업 검증** (축소된 범위).

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 결정 | ✅ | `b4565d4` |
| T1' | V108 — CHECK 단순화 | ✅ | `1efd70f` |
| T3 | 소멸 자동 전이 백엔드 IPC | ✅ | `616021d` |
| T4 | 트리거 3개소 통합 | ✅ | `6b6cc47` |
| T5 | 환원 IPC | ❌ 폐기 | `1762e71` |
| T6 | 퇴교 보강 처리 IPC | ✅ | `6209d00` |
| **T7 (축소)** | **선행 수업 — 출결 생성 충돌 방지 검증 only (PI-08)** | ⬜ 다음 세션 | — |
| T8 | 캘린더 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9 (축소) | 소멸 알림 UI (앱 시작 토스트만 잔여) | ⬜ | — |
| T10 | 퇴교 보강 UI | ⬜ | — |
| T11 | 캘린더 뷰 UI | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T6 결과 요약

| 영역 | 변동 |
|------|------|
| 신규 IPC | `get_pending_makeup_for_withdrawal`, `process_withdrawal_makeup` |
| Enum | `WithdrawalChoice` — ImmediateExpire / ExternalExpire { memo } (defer 는 UI 처리) |
| 단일 트랜잭션 | memo 일괄 저장 → makeup_expired 전이 → withdraw_date 설정 |
| audit | MakeupExpired (per absence) + StudentWithdrawn |
| 단위 테스트 | 6건 (계획 5건+ 충족) |
| 자동 검증 | cargo test 265 passed (T4 259 → +6) / clippy clean |

## T7 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T7 작업 계획 (예상 2h, 축소된 범위)

PI-08 결정: 선행 수업 = 기존 상태 토글 흐름 활용 → **별도 IPC 없음**.

남은 작업:
1. **출결 생성 시 미래 일자 결석 보존 검증** (R69)
   - `generate_attendances` 가 기존 레코드 존재 시 skip 하는지 확인 (이미 구현됨)
   - 단위 테스트로 검증: 미래 일자 결석 사전 등록 → `generate_attendances` 호출 → 결석 레코드 보존

2. **사용자 흐름 문서화** (scope.md):
   - "출결 그리드의 미래 일자 셀 토글로 결석 등록 → 보강 등록 흐름 그대로 사용"
   - PRD §4.2.3 "별도 출결 타입 신설 없이 보강 메커니즘 통합 처리" 와 일치 확인

3. 시각 검증 시점에 사용자 확인 (T12)

### T7 작업 분량
- 단위 테스트 1-2건 추가
- scope.md 문서화
- 코드 변경 거의 없음 (이미 구현됨)

## Sprint 10 Capacity 추적

- 계획: 34h 구현 + 6h 시각 검증 버퍼 = **40h** (T5 폐기 반영)
- 실측 누적: T1 1.5 + T2 1 + T1' 0.5 + T3 2.5 + T4 3 + T5폐기 0.5 + T6 2 = **11h**
- 남은 capacity: 약 29h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **다음 사용자 결정 필요**: T8 진입 시 PI-03 (캘린더 라이브러리) — FullCalendar vs React Big Calendar
