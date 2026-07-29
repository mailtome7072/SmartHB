# Sprint Retrospective Sprint 24

> 대상: Sprint 24 (2414532..ddfa1e0) — 다월 교습기간 year_month 오태깅 계열 버그 일괄 수정 + V313 데이터 전수 보정
> 리뷰 일자: 2026-07-29
> 코드 리뷰: Critical 0 / High 0 / Medium 2 / Low 2건
> 자동 검증: cargo test 491 passed / clippy clean / cipher check OK / tsc clean / lint clean / build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint23-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A114 (Low, 7회 이연) | sync_single_date 이력 패턴 통일 | ✅ **완료** | T0에 강제 포함. `load_schedule_slices` / `minutes_for_date` 공유 헬퍼로 통일. 7스프린트 이연 최종 해소 |
| A127 (Medium) | cancel_makeup N+1 배치 쿼리 전환 | ⏸️ 이연 유지 | makeup.rs 수정 범위에 있었으나 cancel_makeup 배치 전환은 독립 리팩터로 범위 밖 유지 |
| A128 (Low) | cancel_makeup docstring 갱신 | ⏸️ 이연 유지 | makeup.rs 수정이 있었으나 해당 함수 docstring 미반영 |
| A129 (Low) | 보강 시간 입력 드롭다운 전환 | ⏸️ 이연 유지 | 출결 UI 범위 밖 유지 |
| A130 (Medium) | pool() TOCTOU 경합 모니터링 | ⏸️ 이연 유지 | db.rs 미수정 |
| A131 (Medium) | LockScreen 오류 분기 구조화 | ⏸️ 이연 유지 | auth.rs 미수정 |
| A132 (Low) | 구조화 로깅 도입 검토 | ⏸️ 이연 유지 | makeup A2 폴백 eprintln! 추가로 필요성 재확인됨 |

---

## 잘한 점

### CS 사례에서 전 코드베이스 계열 버그 감사로 확장한 프로세스

단일 고객 문의("8월 교습기간의 7/30 원생이 '수업 없는 날'로 표시됨")에서 출발해, 동일 안티패턴("달력월 = 교습기간 year_month 가정")이 적용될 수 있는 코드 전체를 감사했다. 결과적으로 9개 파일 680줄 변경, 버그 9건(A1/A2 데이터 오염 포함) 일괄 해소로 확장됐다. 점별 수정이 아닌 계열 단위 수정으로 R136 잔존분을 전수 해소한 점이 우수하다.

### V313 데이터 전수 보정 — 마이그레이션으로 자동화

기존 오태깅 데이터를 수동 SQL이 아닌 V313 마이그레이션으로 구현하여, 앱 업데이트 시 양 PC(원장 PC / 자택 Mac)에서 자동 적용되도록 했다. NULL 처리(교습기간 미등록 날짜 보호)·멱등성·UNIQUE 충돌 방지를 설계 단계에서 모두 반영하고, 마이그레이션 파일 주석에 각 안전성 근거를 명시했다. 재현이 필요 없는 일회성 데이터 보정을 CI/CD 자동화 경로에 태운 점이 인상적이다.

### A114 7스프린트 이연 최종 해소

Sprint 17부터 7스프린트 이연됐던 A114(`sync_single_date` 이력 패턴 통일)를 T0에 강제 포함해 완료했다. attendance.rs 대규모 수정 스프린트에서 선행 작업으로 포함하여 관련 코드 일관성을 확보한 뒤 이후 수정을 진행한 순서가 올바랐다. 이로써 세 개의 INSERT 경로(generate / apply_schedule_change / sync_single_date)가 모두 `load_schedule_slices` / `minutes_for_date` 단일 소스로 수렴했다.

### simplify 검토로 5중 복제 쿼리 제거 + 공유 헬퍼 설계

"날짜 → 확정 교습기간 year_month" 조회 패턴이 여러 함수에서 중복 작성될 수 있었으나, `period_year_month_for_date` / `load_confirmed_period`를 pub(crate)로 승격하여 단일 소스로 통합했다. dashboard, diagnosis, makeup, notice가 모두 동일 헬퍼를 재사용함으로써 향후 정책 변경 시 1곳만 수정하면 된다. simplify 4관점(재사용·효율·고도) 적용으로 코드 중복 제거를 달성했다.

### 재발 방지 2중 구조 (진단 검사 + 회귀 테스트)

D1(자가진단 검사 8번 — year_month 불변식 검증)과 D2(다월 경계일 회귀 테스트 14건)를 함께 추가했다. D1은 "V313 보정 이후 코드가 다시 오태깅하면 즉시 잡는" 지속적 감시 역할이고, D2는 "다월 교습기간이 테스트 공백이었던 근본 원인"(Sprint 21 R136 미검출의 직접 원인)을 해소하는 구조적 대응이다. 같은 계열 버그의 재발 가능성을 코드 수정 + 진단 + 테스트 세 레이어로 동시에 차단한 점이 우수하다.

### B7 설계 판단 — "교습기간 기준 전환 불필요"를 명시적으로 결정하고 문서화

schedule_events의 "지난 달 수정 차단" 가드를 달력월 기준으로 유지하는 것이 올바른지 brainstorming 후 판단, ADR 불필요(scope.md + 코드 주석으로 대체)로 결정했다. 수정하지 않기로 한 결정도 코드 주석으로 근거를 남긴 점이 미래 유지보수에서 혼란을 방지한다.

---

## 개선할 점

### 다월 교습기간 테스트 공백이 Sprint 21 R136 미검출의 직접 원인

Sprint 21에서 `generate_impl` + `sync_single_date`의 태깅을 수정했을 때, 경계일 시나리오 테스트가 해당 두 함수에만 존재했다. `apply_schedule_change` / `create_makeup` 등 나머지 함수에는 다월 교습기간 기반 테스트가 없었다. 이번 D2(14건 신규 테스트)로 공백을 메웠으나, 새 함수를 추가할 때마다 경계 케이스 테스트를 체계적으로 작성하는 관행을 정착시켜야 한다.

### sync_single_date A114 리팩터에서 N+1 루프 발생 (M1)

`load_schedule_slices`를 원생별 루프 내에서 호출하는 패턴이 N+1 쿼리를 유발한다. 현재 교습소 규모(20~40명)에서는 SAFE하나, A127 배치 전환 이슈와 연계하여 cleanup 스프린트에서 검토가 필요하다. 리팩터 시 "동작 보존 검증"을 목적으로 작성한 현재 회귀 테스트가 그대로 활용될 수 있다.

### eprintln! 폴백 로그 관찰 불가 (L2 — A132 재확인)

makeup A2 폴백 시 `eprintln!`으로 경고를 기록하지만, 프로덕션 Tauri에서 stderr는 캡처되지 않는다. 이번 스프린트에서 새로운 `eprintln!` 1건이 추가됨으로써 A132(구조화 로깅)의 필요성이 재확인됐다. 다음 Phase 로드맵 검토 시 `tauri-plugin-log` 또는 `tracing` 도입을 우선 검토할 것을 권장한다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 | 상태 |
|----|------|----------|-----------|------|------|
| A133 | 새 함수 추가 시 다월 교습기간 경계일 테스트 체크리스트 항목 추가 — study_periods 범위 기반 조회를 포함하는 함수는 반드시 seed_period 기반 경계일 테스트를 포함 | Medium | 개발 관행 / sprint-planner 템플릿 | 다음 스프린트 | 📋 예정 |
| A134 | sync_single_date N+1 루프 배치 최적화 — 재원 원생 전체의 schedule_slices를 단일 쿼리로 사전 조회 후 student_id 키 맵으로 분배 | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | A127 cleanup 스프린트 포함 | 📋 예정 |
| A127 | cancel_makeup_impl N+1 배치 쿼리 전환 (Sprint 22→24 이월) | Medium | `src-tauri/src/commands/makeup.rs:cancel_makeup_impl` | 다음 makeup.rs 수정 스프린트 | ⏸️ 이연 |
| A128 | cancel_makeup docstring 갱신 (Sprint 22→24 이월) | Low | `src-tauri/src/commands/makeup.rs:575-581` | 다음 makeup.rs 수정 스프린트 | ⏸️ 이연 |
| A129 | 보강 시간 입력 드롭다운 전환 (Sprint 22→24 이월) | Low | `src/components/attendance/MakeupRegisterDialog.tsx:230-243` | 다음 출결 UI 수정 스프린트 | ⏸️ 이연 |
| A130 | pool() TOCTOU 경합 모니터링 (Sprint 23→24 이월) | Medium | `src-tauri/src/commands/db.rs` | 관찰 후 다음 db.rs 수정 스프린트 | ⏸️ 이연 |
| A131 | LockScreen 채택 오류 분기 구조화 (Sprint 23→24 이월) | Medium | `src-tauri/src/commands/auth.rs`, `src/components/LockScreen.tsx` | 다음 auth.rs 수정 스프린트 | ⏸️ 이연 |
| A132 | 구조화 로깅 도입 검토 (Sprint 23→24 이월, 이번 스프린트에서 eprintln! 1건 추가로 필요성 재확인) | Low | `src-tauri/` 전반 | 다음 Phase 로드맵 검토 시 | ⏸️ 이연 |
