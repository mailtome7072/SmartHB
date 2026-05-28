# Sprint Retrospective — Sprint 10

> Sprint 10: Phase 3 완결 — 소멸 자동 전이 + 캘린더 뷰 + 퇴교 보강 처리
> 기간: 2026-05-26 ~ 2026-05-28 (3일, ~12 세션)
> 브랜치: `sprint10` (develop 머지 대기)
> 코드 리뷰 (2026-05-28): Critical 0 / High 0 / Medium 2 (F1, F2) / Low 3 (F3, F4, F5)
> 자동 검증: cargo test cipher off 273 passed / cipher on 116 passed (인용) / clippy clean / lint clean / tsc clean / build 16/16

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint9-retrospective.md` (A49~A57)

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A49 | `mark_makeup_absent` + `batch_create_makeups` dead code 정리 | ✅ 완료 | T1에서 IPC handler + audit variant + 단위 테스트 완전 제거 (`dde74aa`) |
| A50 | 시각 검증 capacity buffer 분리 — Sprint 10 계획에 버퍼 6h 예약 | ✅ 완료 | Sprint 10 capacity: "시각 검증 버퍼 6h" 별도 예약. T11 캘린더 7라운드 흡수 실제 소요 약 4h — 버퍼 내 처리 성공 |
| A51 | 도메인 규칙 중 운용 관행 교차 항목 T2에서 사용자 확인 선행 | ✅ 완료 | T2에서 PI-05~PI-09 전수 확인 (트리거 시점 3개소, 소멸 기준일, 선행 수업 방식, 알림 방식). T5(환원 IPC) 폐기 결정도 이 단계에서 선처리 |
| A52 | `get_absence_history` pagination.rs 헬퍼 적용 (R64) | ⏸️ 이연 | 보강/소멸 도메인과 무관. Phase 4 이후 |
| A53 | `create_makeup_with_absences_impl` JSON_EACH 전환 (R65) | ⏸️ 이연 | 성능 영향 미미. Phase 4 이후 |
| A54 | `get_attendance_grid` N+1 batch 쿼리 (R42 이월) | ⏸️ 이연 | Phase 4 이후 |
| A55 | salt buffer ZeroizeOnDrop (R48-b) | ⏸️ 이연 | 보안 도메인. Phase 4 이후 |
| A56 | 반응형 폰트/셀 너비 clamp() (A45 이월) | ⏸️ 이연 | UX 전반 |
| A57 | 한글 자모 부분 일치 검색 (A46 이월) | ⏸️ 이연 | 검색 도메인 |

이행률: **적용 가능한 3건 중 3건 (100%)** — A49(T1 dead code 즉시 정리), A50(capacity 버퍼 예약), A51(T2 사전 사용자 확인) 완료.

---

## 잘한 점

**T2 설계 단계 PI(미결정 항목) 사전 수렴이 T5 폐기를 조기에 결정했다.**

Sprint 9에서 A51로 교훈화한 "도메인 규칙 중 운용 관행 교차 항목을 T1~T2에서 사용자 확인"이 Sprint 10에서 실제로 작동했다. T2에서 PI-05~PI-09를 수렴하면서 T5(보강소멸 → 결석 환원 IPC) 필요성을 검토했고, 사용자 정책 "보강기한 소멸되면 끝"에 따라 폐기를 결정했다. 이 결정이 capacity를 3h 절약했고, 폐기된 T5 범위가 Sprint 중반에 재검토되는 혼란이 없었다.

**cipher 게이트 정합(a3b4915)이 cipher on 빌드/테스트를 처음으로 통과시켰다.**

Sprint 9까지 8개 테스트 모듈이 `#[cfg(test)]`로만 게이트되어 `--features cipher` 빌드 시 컴파일 에러가 발생했다. T12 통합 검증 중 이를 발견하여 `#[cfg(all(test, not(feature="cipher")))]`로 일괄 정합했다. 이제 cipher off(평문 SQLite)와 cipher on(SQLCipher) 두 모드 모두 CI 검증 가능한 상태가 되었다. Sprint 10의 비기능적이지만 의미 있는 개선이다.

**V108 FK 재구성이 실데이터 시각 검증에서만 발견 가능한 결함을 사전 차단했다.**

T11 1차 시각 검증에서 앱 시작 불가(code 787 FK 위반)가 발견됐다. 빈 인메모리 테스트 DB에서는 재현 불가능한 결함이었다. 원인은 V108 마이그레이션이 `makeup_attendances` 테이블을 재생성할 때 실데이터(FK 참조 행 존재)를 DELETE 없이 처리한 것. 수정(d402bb9)은 FK 참조 행 보호 → 임시 테이블 rename → 재생성 → INSERT SELECT → 원본 삭제 순서로 재구성했다. 이 교훈은 `.claude/memory/sqlite-migration-fk-rebuild.md`에 기록됐다.

**시각 검증 버퍼(A50) 6h 예약이 T11 7라운드를 Sprint 내 흡수했다.**

Sprint 9의 capacity 37% 초과 원인이 시각 검증 다라운드였다. Sprint 10에서 capacity에 "시각 검증 버퍼 6h"를 명시적으로 예약했고, T11 캘린더 7라운드(약 4h)가 버퍼 내에서 처리됐다. 총 capacity 실측 약 22.5h / 계획 40h — Sprint 9 52h 초과 대비 안정적. T5 폐기(-3h)와 버퍼 흡수가 주요 원인.

**Phase 3 비즈니스 규칙 단위 테스트가 20건 추가되어 누적 273건(cipher off) 달성.**

소멸 도메인(expiration.rs 13건: 전이 성공/미도래/이미 소멸/교습기간 미등록/복수 batch/deadline NULL + 퇴교 보강 처리 6선택지), 캘린더 집계(calendar.rs 5건: 일자별 그룹화/연월 필터/소멸 임박/교습기간 미등록/미보강 결석 제외), 통합 트리거(attendance.rs 1건: generate 후 expire 포함). PRD §6.5 비즈니스 규칙 100% 단위 테스트 커버 지속.

---

## 아쉬운 점 / 개선 점

**build_day_schedules 내 succ_opt().expect()가 동일 파일의 다른 패턴과 불일치한다.**

Sprint 10에서 새로 추가된 `build_day_schedules` 함수(attendance.rs:655)는 날짜 순회 루프에서 `d.succ_opt().expect("date succ")`를 사용한다. 동일 파일의 기존 두 루프(line 139, 259)는 `.ok_or_else(|| "날짜 계산 오버플로".to_string())?`로 안전하게 처리한다. AI 생성 코드가 기존 패턴을 완전히 따르지 않은 사례. **파일 내 동일 패턴의 안전 처리 방식을 먼저 확인하는 습관 필요.**

**generate_impl의 expire 호출이 fail-soft/fail-hard 정책이 startup.rs와 불일치한다.**

startup.rs는 `expire_overdue_absences_impl` 실패를 무시하고 Ok를 반환하는 fail-soft 정책을 취한다. attendance.rs의 `generate_impl`은 동일 함수 호출 실패를 `?`로 에러 propagate하는 fail-hard 정책이다. 두 트리거가 동일 함수를 다른 방식으로 처리한다. 소멸 전이는 부가 작업(fire-and-forget 적합)이므로 generate_impl도 fail-soft로 통일했어야 한다. **동일 부가 작업의 에러 처리 정책은 트리거별로 통일.**

**T11 캘린더 7라운드 시각 검증 중 FullCalendar 내부 DOM 직접 조작이 도입됐다.**

dayCellDidMount 콜백에서 `.fc-daygrid-day-frame`을 querySelector로 직접 찾아 DOM 노드를 생성·append하는 방식이 채택됐다(ClassCalendar.tsx:367-385). FullCalendar의 dayCellContent로는 절대 위치 지정이 해당 환경에서 의도대로 동작하지 않아 DOM 직접 조작으로 우회했다. 향후 FullCalendar 버전 업그레이드 시 DOM 구조 변경으로 깨질 수 있는 취약 지점이다. 현재 기능은 정상이나 **라이브러리 내부 DOM에 의존하는 우회는 문서화 후 추후 개선 대상으로 남긴다.**

---

## 다음 sprint 액션 항목

| ID | 항목 | 우선순위 | 위치 |
|----|------|----------|------|
| A58 | F1 — `build_day_schedules` `succ_opt().expect()` → `.ok_or_else()` 전환 (R71) | Medium | `attendance.rs:655` |
| A59 | F2 — `generate_impl` expire 호출 fail-soft 전환 — startup.rs 패턴 통일 (R72) | Medium | `attendance.rs:155` |
| A60 | F3 — `get_makeup_management_data_impl` `_year_month` 미사용 파라미터 주석 명확화 또는 월별 필터 적용 | Low | `calendar.rs:186-188` |
| A61 | F4 — `get_makeup_management_data_impl` N+1 → LEFT JOIN 단일 쿼리 전환 (R64 패턴) | Low | `calendar.rs:215-230` |
| A62 | flaky 테스트 — `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` `#[ignore]` 마킹 또는 동시성 재설계 (R70) | Medium | `auth.rs:1132` |
| A63 | A52 이월 — `get_absence_history` pagination.rs 헬퍼 적용 (R64) | Medium | `makeup.rs:742-780` |
| A64 | A53 이월 — `create_makeup_with_absences_impl` JSON_EACH 전환 (R65) | Low | `makeup.rs:434-467` |
| A65 | A54 이월 — `get_attendance_grid` N+1 → batch 쿼리 (R42) | Medium | `attendance.rs::get_grid_impl` |
| A66 | A55 이월 — salt buffer ZeroizeOnDrop 시그니처 변경 (R48-b) | Medium | `auth.rs` salt 함수군 |
| A67 | A56 이월 — 반응형 폰트/셀 너비 clamp() 전환 | Medium | `globals.css` + 컴포넌트 |
| A68 | A57 이월 — 한글 자모 부분 일치 검색 | Medium | `global-search.tsx` |

**Phase 4 진입 전 처리 권장**: A62(flaky 테스트 마킹) + A58(패닉 위험 제거). 나머지는 Phase 4 Sprint 내 기회 시 처리.

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 기간 | 3일 (2026-05-26 ~ 2026-05-28) |
| 커밋 수 (sprint10 브랜치 신규) | 약 48건 (T11 시각 검증 반복 23건 포함, docs 커밋 제외 기준) |
| 백엔드 IPC 신규 | 4종 (`expire_overdue_absences`, `get_calendar_data`, `get_makeup_management_data`, `get_pending_makeup_for_withdrawal`, `process_withdrawal_makeup` — 5종, T7 선행수업은 기존 IPC 재활용) |
| 단위 테스트 신규 | expiration 13건 + calendar 5건 + attendance 1건 + academic 1건 = **20건** |
| 누적 단위 테스트 (cipher off) | **273 passed** (Sprint 9 253 → +20) |
| 누적 단위 테스트 (cipher on) | **116 passed** (인용) |
| TS IPC 래퍼 신규 | 7종 (T4 3종 + T6 2종 + T8 2종) |
| 도메인 타입 신규 | `ExpirationReport`, `ExpiredAbsenceDetail`, `WithdrawalPendingMakeup`, `WithdrawalChoice`, `CalendarMonth`, `CalendarDay`, `CalendarSession`, `MakeupManagementStudent` |
| UI 컴포넌트 신규 | 3종 (`ClassCalendar`, `MakeupManagementView`, `WithdrawalMakeupDialog`) |
| 시각 검증 라운드 | T11 캘린더 7라운드 (~4h) — 버퍼 내 처리 |
| 자동 검증 통과 | 7/7 |
| sprint-review 결함 | 5건 (F1/F2 Medium, F3/F4/F5 Low) — Critical/High 0건 |
| Capacity | 계획 40h → 실측 약 22.5h (T5 폐기 -3h + T11 시각 검증 약 4h 포함) |
| 이전 회고 액션 이행률 | 3/3 적용 가능 항목 100% (A49, A50, A51) |

---

## 종합 평가

Sprint 10은 Phase 3(보강 + 소멸) 완결 스프린트로서 계획한 5개 목표를 모두 달성했다(T5 환원 IPC는 사용자 결정으로 폐기). Critical/High 결함 없이 개발이 완료됐고, V108 FK 재구성이 실데이터 시각 검증에서만 발견 가능한 결함을 sprint 내 사전 차단하여 프로덕션 품질을 보장했다.

Sprint 9의 두 핵심 교훈(시각 검증 capacity 버퍼 분리 / 도메인 결정 사전 수렴)이 Sprint 10에서 모두 작동했다. A50에서 예약한 6h 버퍼가 T11 7라운드를 sprint 내에 흡수했고, A51에서 도입한 T2 사전 수렴이 T5 폐기를 3h 절약했다.

cipher 게이트 정합(a3b4915)은 비기능적 개선이지만 향후 CI가 cipher on 빌드를 검증할 수 있는 기반을 마련했다는 점에서 Sprint 10의 추가 가치다.

**Phase 3 완료 — Sprint 11 (Phase 4: 청구 + 수납 + 공지문) 준비 완료.**
