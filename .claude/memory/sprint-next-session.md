---
name: sprint-next-session
description: "Sprint 9 Session #7 완료 (T1~T7, 7/9). 보강 UI 완성. 다음: T8 결석 이력 조회"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session7-t7
---

Sprint 9 — **보강 UI 완성**. T8 결석 이력 조회 + T9 통합 검증 남음.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 | ✅ `e0e3659` |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ✅ `a62150d` |
| T5 | TS IPC 래퍼 6종 + 도메인 타입 | ✅ `6f761f5` |
| T6 | 보강 등록 (개별) UI — MakeupRegisterDialog | ✅ `76c2ede` |
| T7 | 보강 관리 + 일괄 + A41 라벨 (MakeupManageDialog + BatchMakeupDialog) | ✅ `ef06b43` |
| **T8** | **결석 이력 조회 — /students/[id] 상세 페이지에 섹션 추가** | ⬜ 다음 세션 |
| T9 | 통합 검증 + A39/A40 프로세스 적용 | ⬜ |

검증 상태: `cargo test --lib` cipher off **247 passed** / cipher on **133 passed** / `pnpm lint` clean / `pnpm tsc --noEmit` clean.

## Session #7 (T7) 핵심 변경

**3 작업 통합**:
- **A41 흡수**: 출결표 "결석" → "미처리\n결석" 라벨 (text-sm leading-tight 2줄, width 62px 유지). title 속성에 SQL 조건 명시
- **MakeupManageDialog**: `makeup_done` 셀 클릭 → 메뉴 (취소/미등원) → confirm panel 2단계 → `cancelMakeup`/`markMakeupAbsent`. ConfirmPanel 하위 컴포넌트 (isDanger 옵션)
- **BatchMakeupDialog**: 헤더 "보강데이 일괄" 버튼 → date input + 학생 자동 추출 (client-side 필터로 IPC 절약) + 체크박스 다중 → `batchCreateMakeups` → BatchResult 표시

**AttendanceGrid 확장**: `onMakeupCellClick` Props + StudentRow 내부 분기 (책임 분담 — 외부 handleCellClick 은 토글만)

## 보강 UI 흐름 (T6~T7 완성)

```
출결표 셀 클릭 분기:
├── present/absent → 토글 (기존 T3)
├── makeup_done → MakeupManageDialog (취소/미등원)
├── makeup_expired → 에러 메시지 (기존)
└── null (비수업일) → MakeupRegisterDialog (개별 등록)

헤더 "보강데이 일괄" 버튼:
└── BatchMakeupDialog → 다중 학생 일괄 등록
```

## 다음 세션 (T8) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #8 진입 (T8)
2. T8 작업 (sprint9.md L202~, 예상 3h):
   - **`get_absence_history(student_id) -> Vec<AbsenceHistoryItem>` IPC 신규** (makeup.rs)
     - 대상: `regular_attendances WHERE status IN ('absent', 'makeup_done', 'makeup_expired')`
     - JOIN: `makeup_attendances` (makeup_done 인 경우 보강일 표시)
     - 정렬: `event_date DESC`
   - **`AbsenceHistoryTable` 컴포넌트 신규** + 원생 상세 화면(`/students/[id]`) 또는 출결 화면 내 탭 배치
   - 처리 상태별 시각 구분 (미처리 결석/보강완료/소멸)
   - 단위 테스트: 결석 이력 조회 정렬 + 상태 필터 + JOIN

## Sprint 9 잔여 마일스톤

- T8 결석 이력 (3h) — 누적 32h / 38h (84%)
- T9 통합 검증 (3h)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정 명시
- **A40 sprint-review 산출물 강제** — T9 후 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
