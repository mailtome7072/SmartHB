---
name: sprint-next-session
description: "Sprint 9 Session #8 완료 (T1~T8, 8/9). 다음: T9 통합 검증"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session8-t8
---

Sprint 9 — Phase 3 핵심 도메인 완성. T9 통합 검증만 남음.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 | ✅ `e0e3659` |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ✅ `a62150d` |
| T5 | TS IPC 래퍼 + 도메인 타입 | ✅ `6f761f5` |
| T6 | 보강 등록 (개별) UI | ✅ `76c2ede` |
| T7 | 보강 관리 + 일괄 + A41 라벨 | ✅ `ef06b43` |
| T8 | 결석 이력 조회 — get_absence_history + AbsenceHistoryDialog | ✅ `f2a5689` |
| **T9** | **통합 검증 + A39/A40 프로세스 적용** | ⬜ 다음 세션 |

검증 상태: `cargo test --lib` cipher off **250 passed** / cipher on **133 passed** / clippy 양쪽 clean / pnpm lint/tsc clean.

## Session #8 (T8) 핵심 변경

- 백엔드 `get_absence_history` IPC + AbsenceHistoryItem struct — LEFT JOIN makeup_attendances 로 보강일/시간 포함
- `AbsenceHistoryDialog` 신규 — 상태별 시각 구분 (absent red / makeup_done green / makeup_expired gray)
- AttendanceGrid 학생명 button 분기 — `onStudentNameClick` Props prop 미전달 시 기존 div 유지 (호환)
- 배치: `/students/[id]` 라우트 미존재로 출결표 학생명 클릭 진입 (차기 sprint 라우트 도입 시 다이얼로그 컨텐츠 재사용 가능)

## 보강 도메인 — IPC 7종 (Sprint 9 백엔드 최종)

| IPC | Session | 책임 |
|-----|---------|------|
| `get_pending_absences` | T2 | 미처리 결석 임박순 |
| `get_makeup_eligible_dates` | T2 | 보강 가능 일자 |
| `create_makeup_with_absences` | T3 | 등록 + 매칭 (트랜잭션) |
| `cancel_makeup` | T4 | 취소 + 결석 환원 |
| `mark_makeup_absent` | T4 | 미등원 + 재매칭 가능 |
| `batch_create_makeups` | T4 | 일괄 (부분 성공) |
| `get_absence_history` | T8 | 이력 조회 (3상태 + JOIN) |

## 다음 세션 (T9) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #9 진입 (T9)
2. T9 작업 (sprint9.md L224~, 예상 3h):
   - **자동 검증 7항목** 재실행 + 결과 기록
     1. cargo test cipher off / on
     2. cargo clippy off / on
     3. pnpm lint / tsc --noEmit / build
   - **A39 마이그레이션 self-check**: V108 신규 마이그레이션 없음 (scope.md Session #1 검증 매트릭스 명시) → 통과
   - **A40 sprint-review 산출물 강제 대비**: scope.md 에 산출물 경로 명시 (test-reports, retrospective, code-reviews, risk-register)
   - sprint8.md L? 의 AC 일괄 마킹
   - 사용자 시각 검증 위임 (UC-4 전체 흐름)

## Sprint 9 잔여 마일스톤

- T9 통합 검증 (3h) — 누적 35h / 38h (92%)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정 명시 (scope.md Session #1)
- **A40 sprint-review 산출물 강제** — T9 종료 후 sprint-review 가 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
