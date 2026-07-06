---
Sprint: 19  |  Date: 2026-07-07  |  Session: #1
---

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/academic/AcademicSchedulePrint.tsx | [0회] | T0(A116 동적 행수) + T4(Red 테두리/밴드/폰트) |
| src/hooks/useTableSort.ts (신규) | [0회] | T1 공통 정렬 훅 |
| src/app/students/page.tsx | [0회] | T1 정렬 훅 적용, 기본정렬 학년+이름 |
| src-tauri/src/commands/students.rs | [0회] | T1 ORDER BY 보강, T8 승급 IPC 신규 |
| src/components/attendance/AttendanceGrid.tsx | [0회] | T2 정렬+스크롤 |
| src/app/attendance/page.tsx | [0회] | T2 스크롤 컨테이너 정리 |
| src/components/billing/BillingGrid.tsx | [0회] | T3 정렬+스크롤 |
| src/app/billing/page.tsx | [0회] | T3 스크롤 점검 |
| src/lib/calendar-image.ts | [0회] | T4 참조용(수정 없을 수 있음) |
| src/components/academic/CalendarCell.tsx | [0회] | T4 참조용(수정 없을 수 있음) |
| src/components/schedules/ClassCalendar.tsx | [0회] | T5 화살표 제거+2xN 수정, T6 10xN+라인 |
| src/app/globals.css | [0회] | T6 FullCalendar CSS override |
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
- [ ] T0: A113 상수쌍 문서화, A116 동적 행수 계산
- [ ] T1: 공통 정렬 훅 + 원생 목록 정렬 통일
- [ ] T2: 출결 그리드 정렬 + 스크롤 개선
- [ ] T3: 청구 그리드 정렬 + 스크롤 개선
- [ ] T4: 교습일정 인쇄 캘린더 개선 (Red 테두리, 밴드, 폰트)
- [ ] T5: 주보기 화살표 제거 + 2xN 버그 수정
- [ ] T6: 일보기 10xN + 캘린더 라인 진하게
- [ ] T7: 대시보드 레이아웃 변경
- [ ] T8: 학년 자동 승급
- [ ] T9: 학교급 기반 학교 선택 필터링
- [ ] T10: 통합 검증 (cargo test/clippy, pnpm lint/tsc/build, sqlx migrate run, 시각 검증)
