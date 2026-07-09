# Sprint Retrospective Sprint 19

> 대상: Sprint 19 (e6d6189) — UX 개선 + 인쇄 아키텍처 재설계 + 수강생대장 출력
> 리뷰 일자: 2026-07-08
> 코드 리뷰: Critical 0 / High 1건 / Medium 1건 / Low 2건
> 자동 검증: cargo test 431 passed / clippy clean / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint18-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A113 | 백엔드-프론트엔드 상수 쌍 목록화 (`STALE_THRESHOLD_SECONDS` 등) | ✅ 완료 | `harness-engineering.md`에 상수 쌍 테이블 추가 (T0) |
| A116 | AcademicSchedulePrint 동적 행 수 계산 (5행 달 빈 행 제거) | ✅ 완료 | JS 로직 수정(T0) + CSS `--print-rows` 변수 반영(T4) |
| A114 | sync_single_date 이력 패턴 통일 | ⏸️ 이연 | Post-MVP 유지, 이번 스프린트 범위 외 |
| A115 | cipher 스모크 테스트 수행 | ⏸️ 이연 | 배포 후 수동 검증 항목 유지 |

---

## 잘한 점

**T5 2xN 버그 근본 원인 확정 + 즉시 수정 — systematic-debugging 스킬 효과**

Sprint 19 계획 당시 "원인 미확정"으로 디버깅 버퍼 7h를 별도 확보한 T5 버그를 systematic-debugging 5단계 절차로 당일 근본 원인을 확정했다. `overlapTotal` pairwise 최대값 방식의 결함이 원인이었으며, Node 스크립트로 2026-06 실제 시드 데이터(월요일 16시 19명 클러스터)를 재현한 후 `rowHeightMin = 60/rowsNeeded` 동적 등분으로 수정했다. 몇 명이 겹치든 시간 경계를 넘지 않음을 수식으로 보장 — 재발 가능성 차단.

**인쇄 아키텍처 전환 — Tauri 네이티브 창으로 교습일정 + 수강생대장 통합**

기존 같은 창 인쇄 방식에서 `WebviewWindow` 독립 팝업 방식으로 전환하면서 교습일정 인쇄(`academic-print`)와 수강생대장 출력(`roster-print`) 두 기능을 동일 아키텍처로 처리했다. `on_window_event` 핸들러를 "main" 창 한정으로 가드하여 인쇄 창 닫기 시 DB pool 종료 버그를 사전 차단한 것도 올바른 선택이었다.

**T1 공통 정렬 훅(`useTableSort`) 도입 — 재사용으로 T2/T3 구현 가속**

T1에서 `useTableSort` + `withTiebreak` 공통 인프라를 먼저 구축하고 T2(출결), T3(청구)에서 재사용한 것이 당일 병렬 완료로 이어졌다. 훅의 `comparators` 외부 상수 선언 패턴(`useMemo` 무효화 방지)은 성능 고려가 반영된 설계다.

**단위 테스트 신규 14건 — 핵심 비즈니스 로직 커버**

학년 자동 승급 5건(정상/최대학년 제외/퇴교생 제외/연도 중복 스킵/대상 없음), 정렬 2건, 학교 school_type 4건, 출결 그리드 정렬 1건, 기타 2건을 구현과 동일 커밋에서 작성했다. Sprint 18 414건 → Sprint 19 431건으로 증가.

---

## 개선할 점

**GradePromotionDialog.tsx: 에러 catch 누락 — 조용한 실패 패턴**

`promoteGrades()` IPC 호출 실패 시 `catch` 블록이 없어 사용자에게 아무 피드백 없이 실패한다. `checkGradePromotion`에는 catch가 있는데 같은 컴포넌트 내 `promoteGrades` 호출에 catch가 누락된 것은 일관성 없는 에러 처리다. IPC 쓰기 경로(데이터 변경)에서는 반드시 catch 블록 + 사용자 피드백(toast/에러 메시지)을 명시하는 원칙이 필요하다.

**useTableSort.ts: `copy.reverse()`가 tiebreak까지 역전**

desc 방향 적용 시 배열 전체를 뒤집으면 tiebreak(이름 가나다순)도 함께 역순이 된다. "동일 값 내 이름 자동 가나다순" 사용자 요청 2번의 의도와 다르다. 이 버그는 테스트로 검출하기 어렵다 — desc 정렬 시 동일 값 원소가 2개 이상 있는 케이스를 단위 테스트로 커버해야 발견된다. 프론트엔드 클라이언트 정렬 로직은 실데이터(동일 값 동시 존재 케이스)를 포함한 테스트가 필요하다.

**EnrollDate 정렬 tiebreak 불일치 — 단위 테스트 범위 외 케이스**

`EnrollDateAsc/Desc`의 tiebreak가 `id ASC/DESC`로, 다른 정렬 기준의 `name ASC` 패턴과 다르다. 단위 테스트 `gender_and_weekly_hours_sort_have_name_tiebreak`에서 EnrollDate를 포함하지 않아 미검출됐다. 테스트 케이스 추가 시 신규 정렬 기준을 빠짐없이 포함해야 한다 — 목록으로 문서화하면 누락 방지에 도움.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A117 | GradePromotionDialog.tsx promoteGrades() catch 블록 추가 + toast/에러 메시지 표시 | High | `src/components/layout/GradePromotionDialog.tsx:74` | Sprint 20 T0 |
| A118 | useTableSort.ts desc 정렬 시 tiebreak 오름차순 유지 — `dir * primary(a,b) \|\| tiebreak(a,b)` 방식으로 수정 | Medium | `src/hooks/useTableSort.ts:54` | Sprint 20 T0 |
| A119 | students.rs EnrollDateAsc/Desc tiebreak를 `name ASC`로 통일 + 단위 테스트 추가 | Low | `src-tauri/src/commands/students.rs:131-132` | Sprint 20 또는 안정화 스프린트 |
| A120 | AttendanceGrid school_level SCHOOL_LEVEL_ORDER fallback 추가 — 예상 외 값 수신 시 정렬 안전 보장 | Low | `src/components/attendance/AttendanceGrid.tsx` | Sprint 20 또는 안정화 스프린트 |
| A114 | sync_single_date 이력 패턴 통일 (Sprint 18 이월) | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | Post-MVP |
| A115 | cipher 스모크 테스트 수행 (Sprint 18, 19 이월) | High | 배포 후 수동 검증 | 즉시 (deploy QA 시) |
