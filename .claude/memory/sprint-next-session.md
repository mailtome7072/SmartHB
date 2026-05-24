---
name: sprint-next-session
description: "Sprint 9 Session #5 완료 (T1~T5, 5/9). 다음: T6 보강 등록 (개별) UI"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session5-t5
---

Sprint 9 (Phase 3 보강 + 소멸) — **백엔드 + TS 래퍼 완료**. T6~T9 프론트엔드/검증 남음.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 확정 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 | ✅ `e0e3659` |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ✅ `a62150d` |
| T5 | TS IPC 래퍼 6종 + 도메인 타입 8 interface | ✅ `6f761f5` |
| **T6** | **보강 등록 (개별) UI — /attendance 비수업일 셀 클릭 → MakeupRegisterDialog** | ⬜ 다음 세션 |
| T7 | 보강데이 일괄 + 결석 라벨 (A41) | ⬜ |
| T8 | 결석 이력 조회 | ⬜ |
| T9 | 통합 검증 + A39/A40 프로세스 적용 | ⬜ |

검증 상태: `cargo test --lib` cipher off **247 passed** / cipher on **133 passed** / clippy 양쪽 clean / `pnpm lint` clean / `pnpm tsc --noEmit` clean.

## 백엔드 + TS 인터페이스 — 완성 IPC 6종

| IPC (Rust) | TS 래퍼 | 페이로드 / 응답 |
|-----|---------|---------|
| `get_pending_absences` | `getPendingAbsences(studentId)` | `PendingAbsence[]` (소멸기한 임박순) |
| `get_makeup_eligible_dates` | `getMakeupEligibleDates(studentId, yearMonth)` | `EligibleDate[]` |
| `create_makeup_with_absences` | `createMakeupWithAbsences(payload)` | `MakeupResult` |
| `cancel_makeup` | `cancelMakeup(makeupId)` | `void` |
| `mark_makeup_absent` | `markMakeupAbsent(makeupId)` | `void` |
| `batch_create_makeups` | `batchCreateMakeups(payload)` | `BatchResult` (succeeded/failed) |

## 다음 세션 (T6) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #6 진입 (T6)
2. T6 작업 (sprint9.md L143~, 예상 6h):
   - `/attendance` 출결표 — 비수업일 셀 클릭 → `MakeupRegisterDialog` 열림
   - 다이얼로그 흐름:
     - `getPendingAbsences(studentId)` 호출 → 충당 결석 목록 (소멸기한 임박순) 표시
     - 결석 N건 체크박스 선택
     - class_minutes 입력 (defaulter 60 또는 학생 schedule 기본값)
     - "확정" → `createMakeupWithAbsences(payload)` → 성공 시 그리드 invalidate
   - 클릭 가능 셀 조건: 비수업일 + `getMakeupEligibleDates` 반환 일자 (사전 검증)
   - TanStack Query mutation + invalidate (`attendance-grid`, `pending-absences`)
   - 토스트/알림: 성공 "보강 등록 완료" / 실패 친화 에러 메시지

## Sprint 9 잔여 마일스톤

- T6 보강 등록 UI (6h) — 누적 21h / 38h (55%)
- T7 일괄 + 라벨 (5h)
- T8 결석 이력 (3h)
- T9 통합 검증 (3h)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정 명시
- **A40 sprint-review 산출물 강제** — T9 후 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
