---
name: sprint-next-session
description: "Sprint 9 Session #4 완료 (T1~T4 백엔드 전부, 4/9). 다음: T5 TS IPC 래퍼"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session4-t4
---

Sprint 9 (Phase 3 보강 + 소멸) — **백엔드 전체 완료** (T1~T4). T5~T9 프론트엔드/검증 단계 남음.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 확정 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 | ✅ `e0e3659` |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ✅ `a62150d` |
| **T5** | **TS IPC 래퍼 + 도메인 타입** | ⬜ 다음 세션 |
| T6 | 보강 등록 (개별) UI | ⬜ |
| T7 | 보강데이 일괄 + 결석 라벨 (A41) | ⬜ |
| T8 | 결석 이력 조회 | ⬜ |
| T9 | 통합 검증 + A39/A40 프로세스 적용 | ⬜ |

검증 상태: `cargo test --lib` cipher off **247 passed** (T3 240 → +7) / cipher on **133 passed** / clippy --lib clean 양쪽.

## 백엔드 보강 도메인 — 완성 IPC 6종

`src-tauri/src/commands/makeup.rs` (단일 모듈 + lib.rs 등록 완료):

| IPC | Session | 책임 |
|-----|---------|------|
| `get_pending_absences(student_id)` | T2 | 미처리 결석 임박순 정렬 |
| `get_makeup_eligible_dates(student_id, year_month)` | T2 | 보강 가능 일자 (학사일정 + 학생 입퇴교) |
| `create_makeup_with_absences(payload)` | T3 | 보강 등록 + 매칭 (트랜잭션, 검증 5종) |
| `cancel_makeup(makeup_id)` | T4 | 결석 환원 + makeup DELETE |
| `mark_makeup_absent(makeup_id)` | T4 | 보강 'makeup_absent' + 결석 재매칭 가능 |
| `batch_create_makeups(payload)` | T4 | 다중 원생 일괄, 부분 성공 처리 |

audit 신규 3 variants (T3 추가): `MakeupCreated/Cancelled/Absent`.

## Session #4 (T4) 핵심 변경

- `cancel_makeup` — FK 위반 회피 위해 UPDATE absences SET NULL → DELETE makeup 순서
- `mark_makeup_absent` — 보강 status 마킹 + 결석 환원 (재매칭 가능). 멱등성 (이미 미등원이면 0 반환)
- `batch_create_makeups` — 학생별 독립 트랜잭션 + `create_makeup_with_absences_impl` 재사용. 부분 성공 (`succeeded` / `failed: BatchFailure`) 처리
- 페이로드 struct 4종 신규 (BatchMakeupEntry/CreateMakeupsPayload/Failure/Result)

## 다음 세션 (T5) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #5 진입 (T5)
2. T5 작업 (sprint9.md L143~, 예상 2h):
   - `src/types/makeup.ts` 신규 — PendingAbsence / EligibleDate / CreateMakeupPayload / MakeupResult / BatchEntry / BatchResult 등 백엔드 응답 struct 1:1 매핑
   - `src/lib/tauri/index.ts` — IPC 래퍼 6종 추가 (`getPendingAbsences`, `getMakeupEligibleDates`, `createMakeupWithAbsences`, `cancelMakeup`, `markMakeupAbsent`, `batchCreateMakeups`)
   - `pnpm tsc --noEmit` + `pnpm lint` 통과

## Sprint 9 잔여 마일스톤

- T5 TS 래퍼 (2h) — 누적 21h / 38h (55%)
- T6 보강 등록 UI (6h)
- T7 보강데이 일괄 + 라벨 (5h)
- T8 결석 이력 (3h)
- T9 통합 검증 (3h)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정 명시
- **A40 sprint-review 산출물 강제** — T9 후 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
