# Sprint Retrospective Sprint 21

> 대상: Sprint 21 (f18625d~52f45ea) — 출결 다월 교습기간 그리드 표시/태깅 불일치 수정 (R136)
> 리뷰 일자: 2026-07-19
> 코드 리뷰: Critical 0 / High 0 / Medium 0 / Low 1건
> 자동 검증: cargo test 444 passed (+3) / clippy clean / cargo check --features cipher 통과 / tsc clean / lint clean / build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint20-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A121 | delete_bill_impl BEGIN IMMEDIATE 트랜잭션 | ✅ 완료 | Sprint 21 진입 전 2026-07-19 hotfix로 처리 (R137 종결) |
| A122 | DEPLOY.md 스테이징 검증에 "교습일정 인쇄 미리보기 확인" 항목 추가 | ✅ 완료 | T0에서 처리 (f18625d) |
| A123 | 출결 그리드 다월 표시 재설계 | ✅ 완료 | 본 스프린트 핵심 목표(T1~T3) 완전 해결 |
| A114 | sync_single_date 이력 패턴 통일 | ⏸️ 계속 이연 | Post-MVP 유지. 태깅 통일(T1)로 기능 불일치는 해소됨 |
| A115 | cipher 스모크 테스트 수행 | ⏸️ 이연 | v1.3.0 배포 시 수행 예정 |

---

## 잘한 점

**근본 원인 2축 동시 완전 해결**

Sprint 20에서 후속 분리할 때 진단한 두 가지 근본 원인(태깅 불일치 + 그리드 컬럼 모델)을 T1/T2 두 Task로 정확하게 분리 해결했다. 태깅 수정(T1)이 그리드 매핑 수정(T2)과 독립적으로 테스트 가능하게 설계된 것이 코드 리뷰 추적을 쉽게 했다.

**isMakeupEligibleForCell 숨어있던 다월 요일 계산 버그 함께 수정**

기존 `isMakeupEligibleForCell`은 `yearMonth`로 year/month를 파싱하고 `eventDate.slice(8,10)`으로 day를 파싱했다. 다월 경계 날짜(예: 7/30에서 yearMonth="2026-08")가 들어오면 `new Date(2026, 7, 30)` — 즉 8월 30일로 잘못 계산되는 버그가 있었다. T2의 ISO 날짜 전파 리팩토링 과정에서 `eventDate`를 그대로 split하도록 변경되어 이 버그가 자연스럽게 해소됐다.

**단일월 폴백 안전망 설계로 회귀 위험(R138) 완전 차단**

`periodStartDate/EndDate`가 null이면 기존 `calendarMonthDates`(달력월 1~말일 ISO 배열)로 폴백하여, 교습기간 데이터를 불러오기 전 초기 렌더나 교습기간 미등록 월에서도 기존 동작을 유지한다. 단일월 교습기간이면 `periodDates`와 `calendarMonthDates`가 동일한 날짜 집합을 생성하여 실질적 회귀 없음.

**백엔드 IPC 추가 없이 프론트엔드 구현 완결**

`attendance/page.tsx`가 이미 TanStack Query로 보유한 `listStudyPeriods` 데이터에서 `selectedPeriod`를 찾아 props로 전달하는 방식으로, 새로운 IPC 호출 없이 다월 지원을 구현했다. 네트워크/IPC 오버헤드 없음.

**T3 백엔드 동월 한정 제약과 UI 일관성 확보**

`move_attendance_impl`이 달력월 동월 한정 제약을 갖는다는 점을 T3에서 명확히 반영했다. 다월 교습기간의 이웃 달 날짜(예: 7/30)에서 이동 다이얼로그를 열면 그 날짜가 속한 달(7월) 달력을 표시하여 백엔드 제약과 UI가 정확히 정합한다.

---

## 개선할 점

**T4 시각 QA가 수동 의존 구조 — 자동화 불가 영역 지속**

단일월/다월 그리드 표시, 출결 토글, 보강 등록, 이동 다이얼로그, 인쇄 미리보기 모두 실기기 시각 확인이 필요하며 `cargo test`나 빌드 검사로는 레이아웃·렌더링 결함을 감지할 수 없다. 이 문제는 Sprint 20 T6 인쇄 수정 때도 동일하게 발생했다. DEPLOY.md에 항목을 추가하는 것으로 의무화했으나 근본적인 자동화 방안은 없는 상태.

**A114(sync_single_date 이력 패턴 통일) 4스프린트 연속 이연**

Sprint 18→19→20→21로 이월됐다. T1에서 태깅 통일을 완료했으므로 기능 불일치는 해소됐지만, 이력(history) 기록 패턴 불일치는 여전히 남아 있다. 기능 버그가 아닌 내부 일관성 문제이므로 독립 Hotfix는 감안하지 않고, 다음 attendance.rs 관련 스프린트에서 함께 정리하는 것이 적절.

**MoveAttendanceDialog `yearMonth` prop 역할 모호성**

T3 수정 후 `yearMonth` prop이 실질적으로 invalidation 키로만 사용된다. prop 이름이 "교습기간 ym인가, 달력월인가"를 암시하지만 실제 의미는 "캐시 무효화 대상 year_month"다. 향후 코드를 읽는 사람이 혼란할 수 있다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 | 상태 |
|----|------|----------|-----------|------|------|
| A124 | v1.3.0 배포 시 A115 cipher 스모크 테스트 수행 — SQLCipher AES-256 암호화 DB 생성·열기 단일 사이클 확인 | High | deploy QA 체크리스트 | v1.3.0 배포 시 | 📋 예정 |
| A125 | T4 실기기 시각 QA 완료 — 단일월/다월 그리드, 출결 토글/보강/이동, 인쇄 미리보기(1/2/3개월) 확인 후 DEPLOY.md ✅ | High | DEPLOY.md 체크리스트 | 배포 전 수동 | 📋 예정 |
| A126 | MoveAttendanceDialog `yearMonth` prop을 `invalidationYm`으로 명확화 — 달력월 의존 제거 완료됐으므로 prop 이름도 역할 반영 | Low | `src/components/attendance/MoveAttendanceDialog.tsx:23` | 다음 출결 관련 스프린트 | 📋 예정 |
| A114 | sync_single_date 이력 패턴 통일 (Sprint 18~21 이월) — 태깅 불일치는 T1에서 해소, 이력 패턴만 잔존 | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | 다음 attendance.rs 수정 스프린트 | ⏸️ 이연 |
| A115 | cipher 스모크 테스트 수행 (Sprint 18~21 이월) | High | 배포 후 수동 검증 | v1.3.0 배포 시 | ⏸️ 이연 |
