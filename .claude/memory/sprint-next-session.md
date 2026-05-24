---
name: sprint-next-session
description: "Sprint 9 Session #6 완료 (T1~T6, 6/9). UC-4 핵심 흐름 가능. 다음: T7 보강데이 일괄 + 라벨"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session6-t6
---

Sprint 9 (Phase 3 보강 + 소멸) — UC-4 핵심 흐름 UI 완성. T7~T9 남음.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 확정 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 | ✅ `e0e3659` |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ✅ `a62150d` |
| T5 | TS IPC 래퍼 6종 + 도메인 타입 | ✅ `6f761f5` |
| T6 | 보강 등록 (개별) UI — MakeupRegisterDialog + 비수업일 클릭 | ✅ `76c2ede` |
| **T7** | **보강데이 일괄 + "미처리 결석(일)" 라벨 (A41 흡수)** | ⬜ 다음 세션 |
| T8 | 결석 이력 조회 | ⬜ |
| T9 | 통합 검증 + A39/A40 프로세스 적용 | ⬜ |

검증 상태: `cargo test --lib` cipher off **247 passed** / cipher on **133 passed** / `pnpm lint` clean / `pnpm tsc --noEmit` clean.

## Session #6 (T6) 핵심 변경

- `MakeupRegisterDialog` 신규 — eligibility query → pending absences query → mutation 3단계 흐름. AbsenceRow 하위 컴포넌트로 결석 1건 표시 (소멸기한/메모 포함)
- AttendanceGrid 확장 — `onNonClassDayClick` prop. CellView `cell=null` 분기에 `onEmptyCellClick` 핸들러 (호버 amber + `+` 표시)
- attendance/page.tsx — `MakeupDialogTarget` state + 학생 lookup + invalidate
- **흐름 옵션 F**: 다이얼로그 마운트 시 1회 eligibility query (그리드 진입 시 N명 미리 호출 회피)

## UC-4 핵심 흐름 완성 (T1~T6)

1. 출결표 비수업일 셀 클릭
2. MakeupRegisterDialog 열림 → `getMakeupEligibleDates` 검증
3. eligible 시 `getPendingAbsences` 로 결석 목록 표시
4. 결석 N건 다중 선택 + class_minutes 입력
5. "확정" → `createMakeupWithAbsences` → invalidate → 셀 표시 갱신 (결석 셀 빨강 → "보강")

## 다음 세션 (T7) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #7 진입 (T7)
2. T7 작업 (sprint9.md L?, 예상 5h):
   - 보강데이 일괄 등록 — `/attendance/makeup-batch` 신규 페이지 또는 `/attendance` 헤더 "보강데이 일괄" 버튼 → 다이얼로그
   - 다중 원생 선택 UI (보강 필요 원생 리스트 + 원생별 충당 결석 선택)
   - `batchCreateMakeups` 호출 → BatchResult 표시 (succeeded/failed)
   - **A41 흡수**: AttendanceGrid 헤더 "결석(일)" → "미처리 결석(일)" 라벨 변경 + compute_summary 주석 명확화
   - 보강 행 표시 (출결표에 보강 일자 셀에 보강 정보 표시) — 또는 별도 보강 캘린더

## Sprint 9 잔여 마일스톤

- T7 일괄 + 라벨 (5h) — 누적 27h / 38h (71%)
- T8 결석 이력 (3h)
- T9 통합 검증 (3h)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정 명시
- **A40 sprint-review 산출물 강제** — T9 후 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
