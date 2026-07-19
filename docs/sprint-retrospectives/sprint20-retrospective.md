# Sprint Retrospective Sprint 20

> 대상: Sprint 20 (0ce55d5~52441d5) — 청구 규칙 교습기간 기준 전환 + 청구 삭제 기능 + 인쇄/출결 다월 버그 수정
> 리뷰 일자: 2026-07-19
> 코드 리뷰: Critical 0 / High 0 / Medium 1건(F1) / Low 1건(F2)
> 자동 검증: cargo test 441 passed / clippy clean / tsc clean / lint clean / build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint19-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A117 | GradePromotionDialog promoteGrades() catch 블록 추가 | ✅ 완료 | Sprint 19 후속 커밋 e72c50f에서 처리 (이번 스프린트 전 이미 반영) |
| A118 | useTableSort desc 정렬 tiebreak 오름차순 유지 수정 | ✅ 완료 | Sprint 19 후속 커밋 e72c50f에서 처리 |
| A119 | students.rs EnrollDate tiebreak `name ASC`로 통일 | ✅ 완료 | Sprint 19 후속 커밋 e72c50f에서 처리 |
| A120 | AttendanceGrid school_level SCHOOL_LEVEL_ORDER fallback | ✅ 완료 | Sprint 19 후속 커밋 e72c50f에서 처리 (SchoolLevel enum 검증으로 대체) |
| A114 | sync_single_date 이력 패턴 통일 | ⏸️ 이연 | Post-MVP 유지, 이번 스프린트 범위 외 |
| A115 | cipher 스모크 테스트 수행 | ⏸️ 이연 | deploy QA 시 수행 예정 |

---

## 잘한 점

**공유 헬퍼 `load_billing_period_range`로 R135 유령 버튼 근본 해결**

T1에서 `generate_bills_impl`과 `get_billing_summary_impl` 두 함수가 각자 독립적인 대상 선정 쿼리를 갖고 있던 것을, 단일 공유 헬퍼로 통일하여 구조적으로 불일치 재발을 차단했다. sprint20 계획 문서에서 T1 완료 조건으로 "두 함수 동시 수정"을 명시한 것이 실제 구현에서 이행 여부를 검증 가능하게 만들었고, test(f) `summary_total_billable_matches_generate_target_teaching_period`가 회귀 감지 안전망이 되었다. "같은 로직 2곳 트랩"을 계획 단계에서 사전 식별하고 구조로 해결한 사례다.

**T3 FK CASCADE 테스트 패턴 도입 — `enable_fk()` 헬퍼**

인메모리 SQLite 테스트 풀은 기본값 `foreign_keys=OFF`이라 CASCADE 동작을 검증할 수 없는 구조적 한계가 있었다. `enable_fk(pool)` 테스트 헬퍼를 도입하여 각 삭제 테스트에서 명시적으로 FK를 켜고 CASCADE 삭제를 검증했다. 이 패턴은 앞으로 FK 관련 로직이 있는 테스트에 재사용 가능하다.

**T7 버그A 판정 로직 교체 — generate_impl과 동일 규칙 재사용**

기존 `count_ungenerated`의 "행 0개인 학생"이라는 단순 기준을 "교습기간 기대 수업일 대비 실제 생성 비교"로 교체하면서, `generate_impl`이 사용하는 `load_schedule_slices`, `load_active_students`, `load_off_dates`, `minutes_for_date` 함수들을 그대로 재사용했다. 두 함수가 독립적인 판정 로직을 가질 때 생기는 불일치 위험을 원천 차단하는 설계 원칙의 일관된 적용이다(T1의 `load_billing_period_range`와 동일 취지).

**T6 "주 월 한 장" 접근 — 실사용 데이터 기반 단순화**

3개 달력 멀티페이지 분할 방식이 초기 수정 방향이었으나, 실사용 케이스를 검토한 결과 "이전달 말주 + 주 월 + 다음달 첫주" 구조가 대부분이어 교습기간 전체가 주 월 그리드(앞뒤 이웃 달 칸 포함) 안에 들어온다는 것을 확인하고 단일 달력 표기로 방향을 전환했다. `buildCalendarGrid`의 `iso` 속성이 이웃 달 칸도 정확한 날짜를 계산하도록 설계되어 있어 전환이 자연스러웠다. 코드 복잡도를 줄이면서 실사용 시각 품질을 높이는 판단이었다.

**단위 테스트 431 → 441건 (+10건) — 핵심 비즈니스 로직 완전 커버**

T1 청구 규칙 전환 6건, T3 삭제 가드 4건, T7 부분생성 판정 1건. 특히 T1 test(a)~(f)는 sprint 계획 문서에 테스트 케이스가 명시되어 있고 그 모두를 구현과 동일 커밋에서 완성했다. 계획 → 테스트 케이스 명시 → 구현 → 검증의 루프가 완결된 사례다.

---

## 개선할 점

**delete_bill_impl: DoD 명시 요건(BEGIN IMMEDIATE 트랜잭션) 미이행**

Sprint 20 T3 DoD에서 "트랜잭션 내 실행 (BEGIN IMMEDIATE)"을 명시했으나 실제 구현에서는 적용되지 않았다. 단일 사용자 데스크톱 앱 구조상 TOCTOU 위험이 실질적으로 없어 동작상 문제는 없다. 하지만 DoD에 명시된 항목이 구현에 반영되지 않은 것은 계획-구현 간 추적성이 떨어진다는 신호다. DoD 요건이 선택적이라면 처음부터 "단일 사용자 앱이라 트랜잭션 불필요"로 계획에서 제외했어야 한다.

**T6 인쇄 시각 QA 자동화 불가 — 수동 검증 의존 구조**

인쇄 레이아웃 버그(T6)의 완전한 검증은 실기기 인쇄 미리보기 시각 확인이 필요하며, cargo test나 pnpm build로는 레이아웃 깨짐을 감지할 수 없다. Sprint 19에 인쇄 아키텍처를 팝업창 독립 문서로 전환했을 때와 동일한 문제다. 인쇄 관련 수정이 반복되는 만큼, 최소한 DEPLOY.md 스테이징 검증에 "인쇄 미리보기 확인" 항목을 추가하여 배포 전 수동 검증 의무화가 필요하다.

**T7 버그B 후속 분리(R136) — 그리드 컬럼 모델 재설계 부채**

출결 그리드가 달력월 고정 컬럼(`daysOfMonth`)으로 설계되어 있어, 교습기간이 달력월 밖으로 걸치는 날짜(7/30, 9/1~2)가 표시되지 않는 구조적 결함이 남아 있다. 버그A(버튼 숨김)는 이번 스프린트에서 해결했지만, 버그B(그리드 표시)는 컬럼 모델 재설계가 필요해 Sprint 21로 분리됐다. 그리드 컬럼 구조가 달력월에 고정된 것이 출결 관련 기능의 반복 버그 원인이 되고 있다 — Sprint 21 계획 시 근본 재설계 여부를 검토해야 한다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 | 상태 |
|----|------|----------|-----------|------|------|
| A121 | delete_bill_impl에 BEGIN IMMEDIATE 트랜잭션 추가 — check(SELECT)와 DELETE를 단일 트랜잭션으로 래핑 | Low | `src-tauri/src/commands/billing.rs:345` | Sprint 21 백엔드 정리 작업 시 | 📋 예정 |
| A122 | DEPLOY.md 스테이징 검증에 "교습일정 인쇄 미리보기 확인(1/2/3개월 걸침)" 항목 추가 | Medium | `DEPLOY.md` | 즉시 (다음 배포 전) | 📋 예정 |
| A123 | T7 버그B 출결 그리드 다월 표시 — 그리드 컬럼 모델을 달력월 고정에서 교습기간 실제 일자 범위 기반으로 재설계 | High | `src/components/attendance/AttendanceGrid.tsx`, `src-tauri/src/commands/attendance.rs::sync_single_date` | Sprint 21 계획 시 R136으로 포함 | 📋 예정 (R136) |
| A114 | sync_single_date 이력 패턴 통일 (Sprint 18 이월 지속) | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | Post-MVP | ⏸️ 이연 |
| A115 | cipher 스모크 테스트 수행 (Sprint 18, 19, 20 이월) | High | 배포 후 수동 검증 | 즉시 (deploy QA 시) | ⏸️ 이연 |
