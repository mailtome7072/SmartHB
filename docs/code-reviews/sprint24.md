# Sprint 24 코드 리뷰

> 대상: Sprint 24 (2414532..ddfa1e0) — 다월 교습기간 year_month 오태깅 계열 버그 일괄 수정 + V313 보정 + 회귀 테스트
> 리뷰 일자: 2026-07-29
> 자동 검증 결과: cargo test 491 passed / clippy clean / cipher check OK / lint clean / tsc clean / build 성공

## 고위험 판단 포인트 검토 결과

Sprint 24는 데이터 오염 버그(A1/A2) + 대규모 보정 마이그레이션(V313) + 7개 판단 지점이 있어 검토를 집중 수행했다.

### GP-1 V313 마이그레이션 안전성

- NULL 처리: SQLite에서 `year_month != (서브쿼리 결과 NULL)`은 NULL(거짓) → UPDATE 제외. 교습기간 미등록 날짜의 출결은 기존 값 보존. 안전.
- 멱등성: WHERE 절이 "현재 값 != 올바른 값"인 행만 갱신. 반복 실행 시 0건 → 안전.
- UNIQUE 충돌: event_date 변경 없음 → UNIQUE(student_id, event_date) 충돌 없음. 안전.
- 트랜잭션: sqlx migrate가 각 파일을 트랜잭션으로 실행 → 부분 적용 없음. 안전.
- 앱 시작 자동 적용: db.rs:245 기존 migrate 자동 실행 구조 재사용. 이상 없음.
- **판정: 안전 확인됨.**

### GP-2 경계 비교 반개구간 일관성

- `period_year_month_for_date`: `start_date <= date AND end_date >= date` (closed interval).
- `build_day_schedules` / `get_makeup_eligible_dates`: `succ_opt()`로 `end_date + 1`을 구해 반개구간으로 변환 → 실질적으로 동일한 범위.
- `study_periods.end_date`가 마지막 날(inclusive)이므로 두 표현 모두 일관성 있음.
- **판정: 일관성 확인됨.**

### GP-3 B6 students.rs reinstate 하이브리드 로직

- expiration.rs 소멸 기준: `makeup_deadline IN (SELECT year_month FROM study_periods WHERE end_date <= today)`.
- B6 환원 제외 기준(소멸된 것): 첫 번째 조건 `makeup_deadline IN (end_date <= today 교습기간)`으로 동일 기준 사용 → 대칭 확인됨.
- 레거시 폴백: `NOT EXISTS(study_periods에 해당 year_month 없음) AND makeup_deadline < 달력월` — 과거 교습기간이 미등록된 레거시 데이터 안전망. 기능상 정확함.
- **판정: expiration.rs 대칭 확인됨. 로직 복잡도는 Medium 발견 사항으로 기록.**

### GP-4 B7 academic.rs 설계 판단

- schedule_events는 year_month 컬럼 없는 순수 달력-날짜 엔티티 → 교습기간 기준 전환 불필요.
- "지난 달 수정/삭제 차단"의 '지남'은 물리적 시간 기준 → 달력월이 올바름.
- 다월 교습기간(8월) 경계일인 7/30이 8월 시점에서 수정 불가한 것은 의도된 동작.
- 결정 내용이 주석으로 명확히 문서화됨.
- **판정: 설계 결정 타당. 조건부 ADR 불필요(scope.md 기록으로 대체).**

### GP-5 A114 sync_single_date 리팩터 동작 보존

- 기존 인라인 SQL: `student_schedules.effective_from/to + students.enroll/withdraw` 조건을 단일 INSERT로 처리.
- 신규 코드: 재원 원생 조회(enroll/withdraw 조건) → 원생별 `load_schedule_slices` → `minutes_for_date` 필터로 동일한 판정 수행.
- `minutes_for_date`는 날짜에 유효한 슬라이스를 선택하여 기존 effective_from/to 처리와 동일 결과 반환.
- 동작 보존 확인됨. 14건 회귀 테스트로 검증 완료.
- **판정: 동작 보존됨. N+1 루프 발생은 아래 M1 참조.**

### GP-6 pub(crate) 공용화 모듈 결합

- `period_year_month_for_date` / `load_confirmed_period`를 attendance.rs에서 pub(crate) 승격.
- dashboard, diagnosis, makeup, notice가 `crate::commands::attendance::...`로 호출.
- 두 함수 모두 순수 DB 조회(사이드 이펙트 없음) → 결합 위험 낮음.
- 단일 소스(single source of truth) 유지 관점에서 올바른 결정. 향후 commands 계층 분리 시 헬퍼 모듈로 이동 검토.
- **판정: 결합 타당. Low 등급으로 기록.**

### GP-7 청구 SAFE 전제

- billing.rs / fees.rs 미변경 확인(scope.md Forbidden Areas + 커밋 diff 모두 확인).
- 출결 year_month가 청구 집계에 영향 주지 않음: 청구는 study_periods.year_month 기준으로 직접 집계, 출결 미소비·스케줄 정액 구조 — 감사 결론 재확인.
- **판정: SAFE. 변경 없음 올바름.**

---

## 발견 사항 (4건)

### M1 — sync_single_date N+1 루프 (Medium, 이연)

- 위치: `src-tauri/src/commands/attendance.rs` — `sync_single_date` 신규 구현
- 내용: 재원 원생 목록 조회 후 원생별 `load_schedule_slices`를 루프 내 호출. 원생 N명에 대해 N+1 쿼리 발생.
- 현재 영향: 교습소 원생 규모(보통 20~40명)에서는 SAFE. 100명 이상 시 sync 지연 가능성.
- 조치: A127 N+1 배치 전환 이슈와 연계하여 ROADMAP 이연. 원생 규모 모니터링.

### M2 — B6 reinstate 이중 조건 복잡도 (Medium, 허용)

- 위치: `src-tauri/src/commands/students.rs:512`
- 내용: 레거시 폴백(NOT EXISTS + 달력월 비교)과 교습기간 기준 조건의 이중 구조. 주석으로 충분히 문서화됨.
- 조치: 코드 주석이 충분하므로 별도 리팩터 불필요. 향후 레거시 데이터 소멸 시 폴백 제거 검토.

### L1 — commands 모듈 간 직접 결합 (Low, 허용)

- 위치: `dashboard.rs`, `diagnosis.rs`, `makeup.rs`, `notice.rs` → `crate::commands::attendance::period_year_month_for_date`
- 내용: 헬퍼 함수가 attendance 모듈에 위치하여 타 모듈이 직접 참조. 순수 조회 함수라 결합 위험 낮음.
- 조치: 향후 commands 내 공유 헬퍼 모듈(`shared.rs` 등) 분리 시 이동 검토.

### L2 — makeup A2 폴백 eprintln! 프로덕션 관찰 불가 (Low, 이연)

- 위치: `src-tauri/src/commands/makeup.rs:398`
- 내용: 교습기간 미등록 날짜의 보강 달력월 폴백 시 `eprintln!`으로 경고 기록. 프로덕션 Tauri에서 stderr 캡처 불가.
- 조치: A132 구조화 로깅 도입 이슈와 연계.

---

## 영역별 추가 점검

### 보안 (backend.md Critical)

- SQL 인젝션: 모든 새 쿼리가 `.bind()` 파라미터 사용. raw concat 없음. 이상 없음.
- 하드코딩 시크릿: 없음.
- Tauri 권한 과다 허용: 신규 Tauri 커맨드 없음. capabilities 변경 없음.
- SQLCipher 키 노출: 없음.

### 보안 (backend.md High)

- `unwrap()` 남용: 새 코드에서 `unwrap()` 사용은 인메모리 날짜 계산 실패 불가능한 경우에만 사용(`.expect("validated")`). 프로덕션 panic 위험 없음.
- 마이그레이션 없는 스키마 변경: V313 마이그레이션 파일 존재. 이상 없음.
- 새 쿼리 단위 테스트: `period_year_month_for_date`, `check_year_month_period_mismatch` 등 신규 함수 모두 인메모리 단위 테스트 포함. 이상 없음.

### 프론트엔드 (frontend.md Critical/High)

- XSS: `dangerouslySetInnerHTML` 없음. `MakeupRegisterDialog.tsx` 변경은 prop 참조 교체만. 이상 없음.
- `invoke()` 직접 호출: 없음. IPC 추상화 레이어(`src/lib/tauri/`) 경유 유지.
- TypeScript any 남용: 변경된 tsx 파일에서 any 없음.

### AI 생성 코드 추가 체크

- SQL 쿼리 정답 패턴(`start_date <= d AND end_date >= d AND is_confirmed = 1`) 6개 함수에서 일관 적용 확인.
- 반개구간 표현과 closed interval 혼용이 있으나 의미적으로 동일(GP-2 참조).
- V313의 WHERE 조건이 SET 서브쿼리와 동일한 서브쿼리 사용 → 중복 평가 있으나 SQLite 최적화 범위, 멱등성 보장.

## 결론

Critical 0건, High 0건, Medium 2건, Low 2건. V313 마이그레이션 안전성·멱등성·NULL 처리 모두 확인. B6 expiration.rs 대칭 확인. B7 설계 결정 타당. A114 최종 해소 및 동작 보존 확인. 하드코딩 시크릿 없음. 자동 검증 6종 전수 통과. 배포 진행 가능.

## 리뷰 지적 반영 (2026-07-29, 사용자 요청 — 4건 전부 수정)

| ID | 반영 내용 |
|----|----------|
| M1 | `sync_single_date` 를 원생별 slice 루프(N+1)에서 **이력 인식 단일 set-based `INSERT...SELECT`** 로 복원. `effective_from ≤ date AND (effective_to IS NULL OR effective_to > date)` 는 `minutes_for_date` 반개구간과 동치 → A114 이력 정확성 유지하면서 N+1 제거. sync 는 날짜 범위 순회 호출이라 set-based 가 유리. |
| M2 | `reinstate_student_impl` 의 `NOT(... OR (NOT EXISTS ... AND ...))` 이중부정을 **양성 `CASE`** 로 교체 — "라벨 교습기간 있으면 종료일 미래일 때만, 없으면 달력월 폴백" 의미 명확화. 동작 동등(기존 테스트 통과). |
| L1 | `period_year_month_for_date` / `load_confirmed_period` 를 attendance.rs 에서 **도메인 중립 신규 모듈 `commands/periods.rs`** 로 이동(academic.rs 는 이미 attendance 참조 → 양방향 결합 회피). 전 호출부(attendance/makeup/dashboard/diagnosis)를 `crate::commands::periods::` 로 재지정. 헬퍼 단위 테스트 2건 추가. |
| L2 | makeup A2 폴백의 `eprintln!`(Tauri stderr 미캡처)을 **감사 로그 기록**(`AuditEventType::MakeupCalendarFallback`)으로 교체 → 프로덕션에서 관찰 가능. |

재검증: cargo test **493 passed** / clippy clean / cipher check OK. (프론트엔드 변경 없음)

---

## 재리뷰 — 수동검증 추가 커밋 5건 (2026-07-29)

> 대상: 8e4825e..HEAD (38bc5f5 / 7efa8e1 / cd79826 / 5f65b03 / 670d8bc)
> 자동 검증: cargo test **497 passed** / clippy clean / cipher check OK / lint clean / tsc clean / build 성공

### RR-GP-1 billing weekly_hours 이력 인식 변경 회귀 안전성 (5f65b03)

핵심 변경: `generate_bills_impl` / `get_billing_summary_impl`의 JOIN 조건을
`effective_to IS NULL AND effective_from <= period_end`에서
`effective_from <= period_end AND (effective_to IS NULL OR effective_to > period_end)`로 교체.

- **period_end 대표일 타당성**: "교습기간 종료일 시점에 유효한 스케줄"은 해당 기간의 주된 요금 결정 근거로 합리적. 변경 적용일이 period_end 이후인 스케줄은 해당 기간에 영향 없음 — 올바름.
- **중월 입퇴교 케이스**: `LEFT JOIN` + `enroll_date <= period_end AND withdraw_date >= period_start` 필터는 변경 없음. 스케줄 이력 조건만 교체. 안전.
- **스케줄 없는 학생**: `LEFT JOIN + COALESCE to 0 → weekly_hours=0 → skipped`. 변경 없음. 안전.
- **generate vs summary 일관성**: 두 함수 모두 동일한 조건으로 교체되어 weekly_hours 산정 기준이 일치 → 유령 버튼 재발 방지 확인됨.
- **bind 순서 확인**: 기존 `period_end` 1회 → 신규 `period_end` 2회(effective_from <=, effective_to >) + 이후 `period_end`(enroll_date) / `period_start`(withdraw_date). 바인딩 순서 정상.
- **회귀 테스트**: `generate_bills_uses_schedule_effective_during_period_not_current` — 목요일 1h→2h 변경(적용일 7/30), 7월 period_end=7/29 → 3h 정상 확인. 8월 → 4h 정상 확인. 케이스 충분.
- **판정: 안전 확인됨. SAFE로 판정했던 초기 리뷰 전제 자체가 잘못됐음을 솔직히 기록 — billing.rs 미변경 가정이 "교습기간이 변경 적용일보다 앞선" 엣지케이스를 누락했다. 수정 후 로직 정확성 확인.**

### RR-GP-2 reconcile_attendance_year_month 안전성 (7efa8e1)

- **멱등성**: `WHERE year_month != 서브쿼리` → 이미 맞는 행은 UPDATE 제외. 재실행 0건. 테스트 확인.
- **NULL 안전성**: 확정 교습기간 없는 날짜 → 서브쿼리 NULL → `year_month != NULL` = NULL(거짓) → UPDATE 제외. 안전.
- **UNIQUE 충돌**: `event_date` 변경 없음 → `UNIQUE(student_id, event_date)` 충돌 없음. 안전.
- **V313 SQL과 동기화**: 동일 로직의 런타임 판. 함수 주석에 "V313 SQL을 바꾸면 이 함수도 함께 동기화할 것" 명시됨. 적절.
- **academic.rs 호출 위치**: `create_study_period` / `update_study_period` / `confirm_study_period` 직후 fail-soft 호출. 교습기간 삭제(delete) 경로는 미포함 — 삭제 시에도 오태깅 행이 생길 수 있으나, 삭제 후 해당 날짜의 확정 교습기간이 없으면 서브쿼리 NULL → 재태깅 안 됨(안전). startup 재동기화가 다음 실행 시 정리 역할. 허용 범위.
- **startup 호출**: 단계 5-2에서 pool 획득 후 호출. fail-soft. 정상.
- **판정: 안전 확인됨.**

### RR-GP-3 list_affected_bill_months 정확성 (cd79826)

- **쿼리 판정**: `sp.end_date >= effective_date` — 교습기간 종료일이 변경 적용일 이후인 경우만 포함. 날짜 문자열 비교(YYYY-MM-DD) SQLite에서 사전순 = 날짜순. 안전.
- **확정/미확정 모두 대상**: status를 반환하고 UI에서 "(확정)"/"(미확정)" 구분 표시. 올바름.
- **bind 파라미터**: student_id, effective_date 모두 bind — SQL 인젝션 없음.
- **테스트**: 단일 기간 영향, 양 기간 영향, 청구 없는 원생 3케이스 커버. 충분.
- **판정: 안전 확인됨.**

### RR-GP-4 periods.rs 모듈 분리 정확성 (38bc5f5)

- **전 호출부 재지정**: attendance.rs(generate_impl, count_ungenerated), dashboard.rs, diagnosis.rs, makeup.rs — `crate::commands::periods::` 경로로 전환 확인.
- **academic.rs 호출 유지**: academic.rs의 reconcile 헬퍼는 periods.rs 함수를 직접 호출 — 양방향 결합(academic↔attendance) 회피 목적 달성.
- **pub(crate) 범위**: 외부 크레이트 노출 없음. 적절.
- **헬퍼 단위 테스트**: `period_year_month_for_date` / `load_confirmed_period` 2건 periods.rs 내 신규 테스트 포함.
- **판정: 분리 정확성 확인됨.**

### RR-GP-5 MoveAttendanceDialog 범위 달력 (670d8bc)

- **periodStart/periodEnd prop**: nullable, 미제공 시 fromDate 달력월로 폴백. 하위 호환 유지.
- **범위 외 날짜 비활성**: `inRange` 플래그로 교습기간 밖 날짜 `cursor-not-allowed + text-gray-200` 비활성. 올바름.
- **window.confirm 미사용**: 커스텀 달력 렌더링. Tauri 차단 패턴 없음. ✓
- **월 경계 표기**: 1일에 "M/1" 표기로 달 전환 시점 사용자 인지 가능. 적절.
- **판정: 프론트엔드 구현 정확성 확인됨.**

---

## 재리뷰 발견 사항 (2건, Low only)

### L3 — reconcile fail-soft 및 startup 로그 eprintln! (Low, A132 이연 동일)

- 위치: `src-tauri/src/commands/academic.rs::reconcile_year_month_fail_soft` / `src-tauri/src/startup.rs` 5-2단계
- 내용: 재동기화 결과 로그가 `eprintln!`으로 출력. Tauri 프로덕션 환경에서 stderr 미캡처 — 관찰 불가. L2(makeup 폴백 eprintln!)와 동일 패턴.
- 조치: A132(구조화 로깅) 이연 유지. 동일 이슈 인스턴스 2건 추가 확인.

### L4 — 스케줄 삭제 warnAffectedBills(today) 커트라인 (Low, 허용)

- 위치: `src/components/students/schedule-editor.tsx` — remove mutation onSuccess
- 내용: 스케줄 삭제 시 `warnAffectedBills(today)`로 호출 → `sp.end_date >= today`. 삭제된 스케줄의 `effective_from`이 오늘 이전이면, 오늘 이전에 종료된 교습기간의 청구는 팝업 대상에서 누락 가능.
- 실제 영향: 팝업은 advisory only. 기존 inline notice("주당 수업시간이 바뀌어 이번 달 청구액 재확인이 필요합니다")가 별도 유지됨. 청구 자체 계산 영향 없음.
- 조치: 허용. 완전하게 수정하려면 삭제할 스케줄의 effective_from을 전달해야 하나, 1인 교습소 운영 맥락에서 현재 달의 inline notice로 충분. 향후 스케줄 편집기 리팩터 시 개선 검토.

---

## 재리뷰 결론

Critical 0건, High 0건, Low 2건(L3/L4 — 기존 이연 사항 동일 패턴). 초기 리뷰에서 SAFE로 판정했던 billing.rs가 수동검증으로 실제 버그(유령 버튼 포함)가 발견되어 수정됨 — 이력 인식 전환 후 로직 정확성 재확인. reconcile 안전성·periods.rs 분리 정확성·팝업 모달 구현 모두 확인. 자동 검증 6종 재실행 전수 통과(cargo test 497). 배포 게이트 통과 조건 충족.
