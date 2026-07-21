# Sprint Retrospective Sprint 22

> 대상: Sprint 22 (3ad9b1f~93ec011) — 보강 분 단위 부분 차감 전환 + 출결 그리드 z-index 수정 (ADR-011)
> 리뷰 일자: 2026-07-21
> 코드 리뷰: Critical 0 / High 0 / Medium 1 / Low 2건
> 자동 검증: cargo test 457 passed / clippy clean / tsc clean / lint clean / build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint21-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A126 | MoveAttendanceDialog `yearMonth` → `invalidationYm` prop 명확화 | ✅ 완료 | T7 보강 UI 수정 시 함께 처리 |
| A114 | sync_single_date 이력 패턴 통일 | ⏸️ 이연 유지 | 보강 잔여 계산과 무관한 별도 리팩터로 판단, 범위 외 유지 |
| A124 | v1.3.0 배포 cipher 스모크 테스트 | ⏸️ 이연 | v1.3.0 배포 완료 후 별도 수행 예정 |
| A125 | T4 실기기 시각 QA 완료 | ⏸️ 이연 | v1.3.0 배포 시 수행 예정 |

---

## 잘한 점

**ADR 기반 설계 우선 — 재작업 없는 구현**

T0에서 배분 링크 테이블(B안)과 누적 컬럼(A안) 두 대안을 Weighted Matrix(3.55 vs 3.95)로 비교·결정하고 ADR-011을 작성한 후 구현에 진입했다. 결과적으로 "취소 시 정확한 부분 환원"이라는 핵심 요구사항이 처음부터 스키마에 반영되어 재작업 없이 T2~T4가 진행됐다. 설계 선행 투자가 구현 단계의 방향 수정을 없앴다.

**remaining_minutes_expr 헬퍼로 8개 쿼리 일관성 확보 (R139 완화)**

잔여 보강필요분 계산 서브식을 `remaining_minutes_expr(alias: &str)` 단일 헬퍼로 추출하고, 8개 파일의 관련 쿼리를 이 헬퍼로 통일했다. 코드 리뷰에서 전수 확인 결과 누락된 파일 없음. 헬퍼에 "alias는 코드 상수만" docstring을 명시하여 SQL 인젝션 방지 가이드도 내재화했다.

**V312 백필 멱등성 + 안전장치 이중 설계 (R140 완화)**

- 멱등성: INSERT NOT EXISTS + UPDATE makeup_attendance_id IS NOT NULL 조건으로 재실행 시 결과 불변 보장
- 사전 스냅샷: 미적용 마이그레이션 존재 시 migrate 직전 자동 백업(`has_pending_migrations` + `try_create_backup`)
- 두 장치를 조합하여 "커밋 성공 후 계산 오류"라는 최악의 시나리오를 사전 백업으로 방어

**배포 안전성 우선 설계로 deferred FK 함정 회피 (R142)**

`makeup_attendance_id` 컬럼을 DROP하지 않고 레거시로 유지하는 결정이 ADR-011에 명시적으로 기록됐다. V311은 순수 `CREATE TABLE + CREATE INDEX`만 수행하여 기존 테이블을 전혀 건드리지 않았다. V108의 deferred FK 카운터 함정이 이번 스프린트에서 재발하지 않은 것은 이 판단 덕분이다.

**457개 단위 테스트로 비즈니스 규칙 회귀 방지**

sprint22.md T2 항목에 명시된 6가지 시나리오(완전 소진, 부분 소진 1회, 부분 소진 2회, 2결석 완전 소진, 초과 차단, 소멸기한 순서) + 취소 3케이스가 모두 테스트로 커버됐다. cargo test 457 passed는 Sprint 21 대비 13건 증가.

---

## 개선할 점

**cancel_makeup_impl 내 N쿼리 패턴 (F1 발견)**

취소 시 배분된 결석 수만큼 루프에서 잔여 재계산 쿼리를 개별 실행한다. 현재 50명 규모에서 실용적 영향은 없지만, 원생 수 증가 시 지연이 생길 수 있다. 배치 쿼리로 전환하면 루프 쿼리를 1~2 쿼리로 줄일 수 있다.

**docstring stale — cancel_makeup 구 로직 설명 유지 (F2 발견)**

`cancel_makeup` IPC 함수 주석이 구 V107 FK 방식("makeup_attendance_id=NULL")을 설명하는 채로 남아 있다. 실제 구현은 allocation 기반으로 올바르게 교체됐으나, 주석이 새 코드를 읽는 사람에게 혼란을 줄 수 있다.

**보강 시간 입력 UI 계획 대비 간소화 (F3 발견)**

T7 계획은 "1h/2h/3h 드롭다운 또는 라디오 버튼"이었으나 `<input type="number" step={1}>` 숫자 입력으로 구현됐다. 기능 이상은 없으나 소수 입력이 허용되는 UX 약점이 있다. 드롭다운으로 입력 범위를 제한하면 사용자 실수를 줄일 수 있다.

**A114 sync_single_date 이력 패턴 5스프린트 누적 이연**

Sprint 18→19→20→21→22로 이월됐다. 태깅 불일치(Sprint 21 해소)와 달리 이력 기록 패턴 불일치는 잔존한다. 기능 버그가 아닌 내부 일관성 문제이므로 독립 Hotfix는 불필요하나, 다음 attendance.rs 관련 스프린트에서 반드시 포함해야 한다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 | 상태 |
|----|------|----------|-----------|------|------|
| A127 | cancel_makeup_impl 루프 N쿼리 배치 쿼리 전환 — 취소된 보강의 배분 결석들을 IN 절로 일괄 조회·업데이트 | Medium | `src-tauri/src/commands/makeup.rs:cancel_makeup_impl` | 다음 makeup.rs 수정 스프린트 | 📋 예정 |
| A128 | cancel_makeup IPC docstring ADR-011 반영 갱신 — 구 V107 FK 설명 제거, allocation 기반 취소 흐름으로 갱신 | Low | `src-tauri/src/commands/makeup.rs:575-581` | 다음 makeup.rs 수정 스프린트 | 📋 예정 |
| A129 | MakeupRegisterDialog 보강 시간 입력을 1h/2h/3h 드롭다운으로 전환 — 소수 입력 방지 | Low | `src/components/attendance/MakeupRegisterDialog.tsx:230-243` | 다음 출결 UI 수정 스프린트 | 📋 예정 |
| A114 | sync_single_date 이력 패턴 통일 (Sprint 18~22 이월) — 5스프린트 누적 이연, 다음 attendance.rs 수정 스프린트에서 반드시 포함 | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | 다음 attendance.rs 수정 스프린트 | ⏸️ 이연 |
| A115 | cipher 스모크 테스트 수행 (Sprint 18~22 이월) — cipher-on 빌드에서 V311/V312 백필 실 DB 동작 확인 | High | 배포 후 수동 검증 | deploy-prod 단계 | ⏸️ 이연 |
