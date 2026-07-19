---
name: sprint21-context
description: Sprint 21 계획 수립 시 발견한 주요 컨텍스트 -- 출결 다월 그리드 태깅/표시 수정(R136), A안 확정
metadata:
  type: project
---

## Sprint 21 요약
- **목표**: 출결 다월 교습기간 그리드 표시/태깅 불일치 근본 수정 (R136, A123)
- **Task 수**: 5개 (T0~T4), 예상 10.5~15.5h
- **DB 마이그레이션**: 없음 (V310 유지)
- **새 의존성**: 없음
- **계획 수립일**: 2026-07-19

## 핵심 기술 결정
- **A안 확정**: 그리드를 교습기간 날짜 범위(start_date~end_date)로 표시 — 유일 정합 (드롭다운이 교습기간 월만 표시하므로 달력월 기준 B안은 고아 날짜 발생)
- 백엔드 IPC 추가 불필요 — 페이지가 이미 listStudyPeriods 데이터 보유
- Sprint 20 인쇄 수정(주 월 달력)과 일관

## 근본 원인 (검증 완료)
1. `sync_single_date`(attendance.rs:1525): `let ym = &date[..7]` — 달력월로 태깅 (generate_impl은 교습기간 ym)
2. `daysOfMonth`(AttendanceGrid.tsx:134): 달력월 1~말일 고정 컬럼
3. `buildAttendanceByDay`(AttendanceGrid.tsx:150): DD 추출만으로 매핑 → 충돌
4. `MoveAttendanceDialog`(44-45행): 달력월 가정

## 릴리즈 연계
- Sprint 20+21 함께 v1.3.0 배포 예정 (사용자 결정)
- A115 cipher 스모크 테스트는 v1.3.0 deploy QA 시 수행

**Why:** Sprint 21 컨텍스트를 다음 세션에서 즉시 활용.
**How to apply:** sprint-dev 21 진입 시 이 메모를 참조하여 scope.md 작성.
