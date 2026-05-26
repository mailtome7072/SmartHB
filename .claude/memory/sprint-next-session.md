---
name: sprint-next-session
description: "Sprint 10 Session #6 완료 (T5 폐기 결정). 다음: T6 — 퇴교 시 미사용 보강 처리 IPC (PRD §4.5.9)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session6-t5-cancel
---

Sprint 10 — Phase 3 완결 sprint. Session #6 (T5 폐기) 완료. 다음은 **T6 퇴교 시 미사용 보강 처리 IPC**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 결정 | ✅ | `b4565d4` |
| T1' | V108 — `makeup_attendances.status` CHECK 단순화 | ✅ | `1efd70f` |
| T3 | 소멸 자동 전이 백엔드 IPC (expiration.rs) | ✅ | `616021d` |
| T4 | 소멸 전이 트리거 3개소 통합 | ✅ | `6b6cc47` |
| **T5** | **소멸 → 결석 환원 IPC** | ❌ **폐기** (사용자 결정 2026-05-26) | — |
| **T6** | **퇴교 시 미사용 보강 처리 IPC** (PRD §4.5.9) | ⬜ 다음 세션 | — |
| T7 (축소) | 선행 수업 — 출결 생성 충돌 방지 검증 only (PI-08) | ⬜ | — |
| T8 | 캘린더 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9 (축소) | 소멸 **알림** UI (환원 부분 폐기) | ⬜ | — |
| T10 | 퇴교 보강 UI | ⬜ | — |
| T11 | 캘린더 뷰 UI | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T5 폐기 결정 (PI-10 대체)

사용자 결정 (2026-05-26): **"보강기한 소멸되면 끝임"**

→ **T5 환원 IPC + T9 환원 다이얼로그 완전 폐기**. PRD §4.5.3 AC-4.5-5 요건 해제.

추가 요구: **출결관리에서 보강완료 vs 보강소멸 시각 구분**
- 현재 코드 (Sprint 9 J7) 이미 충족:
  - `makeup_done` (보강완료): `bg-emerald-100` + '결석' 라벨
  - `makeup_expired` (보강소멸): `bg-gray-200` + '소멸' 라벨
- T12 시각 검증에서 사용자 확정

## Capacity 절감

| 항목 | 변동 |
|------|------|
| T5 폐기 | -3h |
| T9 환원 부분 폐기 | -1.5h |
| **총 절감** | **-4.5h** |
| Sprint 10 총 capacity | 44.5h → **40h** |

## T6 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T6 작업 계획 (예상 3h, sprint10.md L181-)

PRD §4.5.9 퇴교 시 미사용 보강 처리.

1. **신규 IPC 2종** — `students.rs` 또는 `expiration.rs`
   - `get_pending_makeup_for_withdrawal(student_id)`: 미보강 결석 리스트 + 잔여 보강필요시간 조회
   - `process_withdrawal_makeup(student_id, choice)`: 3가지 선택지 처리
     - `immediate_expire`: 전체 미보강 → `makeup_expired` 전이
     - `defer_withdrawal`: 퇴교일 보류 (퇴교 취소)
     - `external_expire(memo)`: 사유 메모 + 전체 → `makeup_expired` 전이

2. **TS 래퍼 2종**: `getPendingMakeupForWithdrawal`, `processWithdrawalMakeup`

3. **단위 테스트 5건+**:
   - 미보강 결석 조회 정확성
   - 즉시 소멸 → 전체 makeup_expired
   - 보강 진행 후 퇴교 → 퇴교일 미변경
   - 외부 처리 → memo 저장 + 전체 makeup_expired
   - 보강필요시간 0인 원생 → 다이얼로그 미표시 (조회 결과 빈 리스트)

### T6 설계 결정 필요 항목 (다음 세션 진입 시 사용자 확인)
- **PI-11**: 모듈 위치 — `students.rs` (학생 도메인 함수) vs `expiration.rs` (소멸 도메인 함수) vs `withdrawal.rs` 신규
- **PI-12**: `external_expire` 의 `memo` 저장 위치 — `regular_attendances.absence_memo` (기존 컬럼) vs 학생 audit log

## Sprint 10 Capacity 추적

- 계획: 34h 구현 (T5 -3h, T9 -1.5h 반영) + 6h 시각 검증 버퍼 = **40h**
- 실측 누적: T1 1.5h + T2 1h + T1' 0.5h + T3 2.5h + T4 3h + T5폐기 0.5h = **9h**
- 남은 capacity: 약 31h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **PRD 정합성**: T5 폐기는 사용자 운영 정책 결정으로 PRD §4.5.3 AC-4.5-5 해제. CHANGELOG 작성 시 명시 필요
