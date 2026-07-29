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
