---
Sprint: 19  |  Date: 2026-07-07  |  Session: #1
---

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/academic/AcademicSchedulePrint.tsx | [2회] | T0(A116 동적 행수) + T4(Red 테두리/밴드/폰트) 모두 완료 |
| .claude/rules/harness-engineering.md | [1회] | T0(A113 상수쌍 문서화, 완료) |
| src/hooks/useTableSort.ts (신규) | [1회] | T1 공통 정렬 훅 (완료, T2/T3에서 소비 예정) |
| src/app/students/page.tsx | [1회] | T1 정렬 훅 적용, 기본정렬 학년+이름 (완료) |
| src-tauri/src/commands/students.rs | [1회] | T1 ORDER BY 보강(완료), T8 승급 IPC 신규(예정) |
| src/types/student.ts | [1회] | T1 StudentSort 타입 확장 (완료) |
| src/components/attendance/AttendanceGrid.tsx | [1회] | T2 정렬+스크롤 (완료) |
| src/app/attendance/page.tsx | [1회] | T2 스크롤 컨테이너 정리 (완료) |
| src-tauri/src/commands/attendance.rs | [1회] | T2 school_level/grade 추가 + 정렬 (완료) |
| src/types/attendance.ts | [1회] | T2 schoolLevel/grade 필드 추가 (완료) |
| src/components/billing/BillingGrid.tsx | [1회] | T3 정렬+스크롤 (완료) |
| src-tauri/src/commands/billing.rs | [1회] | T3 3차 정렬키 학년+이름 (완료) |
| src/types/billing.ts | [1회] | T3 SchoolLevel 공유 타입 교체 (완료) |
| src/lib/calendar-image.ts | [1회] | T4 날짜유틸 공용화 리팩터 (완료) |
| src/lib/time.ts | [1회] | T4 isoDayOfWeek/isWeekday/nextIsoDate/prevIsoDate 공용화 (완료) |
| src/components/academic/ThreeMonthCalendar.tsx | [1회] | T4 날짜유틸 공용화 리팩터 (완료) |
| src/components/schedules/ClassCalendar.tsx | [1회] | T5 화살표 제거+2xN 근본수정(완료, 10xN도 통합됨). T6는 라인만 남음 |
| src/app/globals.css | [2회] | T4 인쇄 CSS + T6 FullCalendar 라인 진하게 모두 완료 |
| src/components/dashboard/DashboardView.tsx | [0회] | T7 레이아웃 상하 배치 |
| src-tauri/src/commands/mod.rs | [0회] | T8 invoke_handler 등록 |
| src-tauri/src/lib.rs | [0회] | T8 invoke_handler 등록 |
| src/lib/tauri/index.ts | [0회] | T8 IPC 래퍼 |
| src/components/layout/app-shell.tsx | [0회] | T8 세션당 1회 체크 |
| src-tauri/migrations/310__auto_correct_school_type.sql (신규) | [0회] | T9 school_type 자동 보정 |
| src/app/settings/codes/page.tsx | [0회] | T9 school_type 선택 UI |
| src/components/students/student-form.tsx | [0회] | T9 학교 드롭다운 필터링 교체 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] docker/, docker-compose*.yml — 컨테이너 인프라 (범위 외)
- [ ] docs/harness-engineering/ — Harness 정책 문서 (범위 외)
- [ ] 원생/스케줄/청구 등 기존 핵심 도메인 로직(계산/검증 규칙) — 이번 스프린트는 UI/UX+운영편의 기능이며 기존 비즈니스 계산 로직 변경 없음

## 완료 기준 (이번 세션)
- [x] T0: A113 상수쌍 문서화, A116 동적 행수 계산
- [x] T1: 공통 정렬 훅 + 원생 목록 정렬 통일
- [x] T2: 출결 그리드 정렬 + 스크롤 개선
- [x] T3: 청구 그리드 정렬 + 스크롤 개선 (billing/page.tsx는 이중 스크롤 구조 없어 수정 불필요로 확인됨)
- [x] T4: 교습일정 인쇄 캘린더 개선 (Red 테두리, 밴드, 폰트) — 인쇄 시각 확인은 T10으로 이연
- [x] T5: 주보기 화살표 제거 + 2xN 버그 수정 (10xN도 함께 구현됨, T6는 CSS 라인만 남음)
- [x] T6: 일보기 10xN(T5에서 선완료) + 캘린더 라인 진하게
- [ ] T7: 대시보드 레이아웃 변경
- [ ] T8: 학년 자동 승급
- [ ] T9: 학교급 기반 학교 선택 필터링
- [ ] T10: 통합 검증 (cargo test/clippy, pnpm lint/tsc/build, sqlx migrate run, 시각 검증)
