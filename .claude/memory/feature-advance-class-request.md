---
name: feature-advance-class-request
description: 원장 기능 요청(2026-08-01) — 선(先)수업/미리 수업. 미래 결석 예정 수업을 이전 날에 미리 진행. 핵심 갭=이미 수업 있는 날에 추가 수업 등록 불가. 다음 스프린트 후보. ROADMAP 백로그 등재됨
metadata: 
  node_type: memory
  type: project
  originSessionId: 2de8b3ba-b8a7-4d3d-bec8-87adf8a9081b
  modified: 2026-08-01T11:55:13.863Z
---

**요청 (원장, 2026-08-01)**: 원생 사정으로 **미래 정규 수업일 X에 못 나올** 때, X 이전의 다른 날에 그 수업을 **미리** 진행하는 방법.
- 대상 날에 수업이 없으면 → 기존 "수업일 이동"(MoveAttendanceDialog)으로 처리 가능(이미 됨).
- **대상 날에 이미 정규 수업이 있으면 → 그날 수업을 하나 더 추가할 방법이 없음** ← 이번에 구현 필요한 핵심.

**Why**: 실사용 중 발생한 실제 니즈. 지금 미구현이라 원장이 수기로 우회 중.

**How to apply (다음 계획 시)**:
- ROADMAP.md "🔮 향후 계획(Backlog)" > "기능 요청 — 선(先)수업/미리 수업"에 등재됨. 다음 스프린트 계획(sprint-planner) 시 이 항목 포함.
- 데이터 모델: `makeup_attendances`는 **동일 (student_id, event_date) 중복 허용**(UNIQUE 없음, PRD §6.2 / [[multi-month-period-pitfall]] 도메인) → 한 날 추가 세션 저장은 이미 가능. **갭은 UI/워크플로우 + 미래 결석 예정일 연결**.
- 설계 결정 필요: (a) 기존 보강(사후) 도메인과 통합 vs 별도 "선보강", (b) 청구 시수 반영, (c) 보강 소멸 로직 관계, (d) year_month 태깅 규칙 준수([[multi-month-period-pitfall]]), (e) 대상 날 "없음(이동)/있음(추가)" UI 일원화 여부.
- 관련 코드: 출결/보강 = `commands/attendance.rs`·`makeup.rs`, 이동 UI = `MoveAttendanceDialog.tsx`, 교습기간 헬퍼 = `commands/periods.rs`.
- 우선순위 관계: v1.5.2(마이그레이션 hang, [[v151-migration-hang]])와 별개 성격(도메인 기능 vs 버그). 묶을지 분리할지는 계획 시 결정.
