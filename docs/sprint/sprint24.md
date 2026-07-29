# Sprint Plan sprint24

> **완료**: 2026-07-29 | 커밋: 2414532(계획), ddfa1e0(구현) | 브랜치: `sprint24 → develop` 직접 머지 예정

## 기간
2026-07-29 (계획+구현 당일 완료, 2주 스프린트 목표 조기 달성)

## 목표
다월 교습기간(예: 8월 = 2026-07-30~2026-09-02)에서 `year_month` 오태깅 계열 버그를 일괄 수정하고, 기존 오태깅 데이터를 전수 자동 보정하며, 재발 방지 진단 검사 + 회귀 테스트를 추가하여 다월 교습기간 경계일에서의 데이터 정합성을 확립한다.

## 근본 원인 (단일 안티패턴)
코드 여러 곳이 "날짜의 달력월(YYYY-MM) = 그 날짜가 속한 교습기간의 year_month"라고 가정한다. 실제 `study_periods.year_month`는 사용자 지정 라벨이며, 다월 교습기간의 경계일(7/30, 7/31, 9/1, 9/2 등)에서 달력월과 어긋난다. Sprint 21 R136에서 `generate_impl`과 `sync_single_date`만 수정했으나 나머지가 잔존.

**정답 패턴 (참조)**: `attendance.rs:1547-1555` `sync_single_date` -- `SELECT year_month FROM study_periods WHERE start_date <= date AND end_date >= date AND is_confirmed=1`.

## ROADMAP 연계 기능
- Sprint 21 R136 동일 계열 잔존 버그 일괄 해소
- A114 (sync_single_date 이력 패턴 통일, 7회 이연) -- attendance.rs 대규모 수정 포함이므로 강제 포함

## 구현 범위

### 포함

**데이터 오염 유발 (CONFIRMED, 최우선)**
- **A1**: `attendance.rs:1440` `apply_schedule_change_impl` -- 재생성 출결을 `&ds[..7]`(달력월)로 태깅. `Period` struct에 year_month 필드 추가 + 교습기간 기준 태깅 전환
- **A2**: `makeup.rs:371,511` `create_makeup_with_absences_impl` -- 보강 출결 year_month를 보강일 달력월로 태깅. study_periods 조회 추가

**표시/동작 오류**
- **B1**: `attendance.rs:1224` `move_attendance_impl` -- 동월 한정 판정을 달력월에서 "같은 study_periods 소속" 기준으로 전환 + 이동 UPDATE year_month 갱신
- **B2**: `attendance.rs:768` `build_day_schedules` -- 일자 헤더 학사마커 달력월 범위 조회를 교습기간 범위로 교체
- **B3**: `notice.rs:609-615` `get_notice_month_info` -- 보강데이 필터 `LIKE 'YYYY-MM-%'`를 교습기간 범위 기반으로 통일
- **B4**: `dashboard.rs:461,487` -- 소멸 임박/미확정 청구 알림의 달력월 vs 교습기간값 비교를 활성 교습기간 라벨 기준으로 교체
- **B5**: `makeup.rs:216-269` `get_makeup_eligible_dates` -- 보강 가능일 조회를 교습기간 범위 기반으로 교체
- **B6**: `students.rs:509` `reinstate_student_impl` -- `makeup_deadline` vs `strftime('%Y-%m','now')` 비교를 study_periods 기준으로 교체
- **B7**: `academic.rs:1067,1164` -- 학사일정 "지난달 수정 차단" 가드가 달력월 기준. **설계 판단 필요 (ADR 여지)**: schedule_events는 순수 날짜 엔티티라 교습기간 기준 적용이 올바른지 확인 필요
- **B8**: `MakeupRegisterDialog.tsx:92` -- deadline 비교에 `eventDate.slice(0,7)` 사용. authoritative `yearMonth` prop으로 교체

**데이터 전수 보정**
- **E1**: regular_attendances + makeup_attendances 전체에서 `year_month != (event_date 소속 확정 교습기간의 year_month)`인 행 일괄 교정. V313 마이그레이션으로 구현

**재발 방지**
- **D1**: `diagnosis.rs` 신규 자가진단 검사 -- year_month 불변식 검증 + `run_diagnosis` 달력월 파라미터 점검
- **D2**: 다월 교습기간 회귀 테스트 -- 경계일 시드 기반 단위 테스트 일괄 추가

**이전 회고 액션 강제 포함**
- **A114**: `sync_single_date` 이력 패턴 통일 (7회 이연, attendance.rs 대규모 수정 포함으로 강제)

### 제외
- 청구/수납(billing.rs/fees.rs) -- 감사 결과 SAFE(영향 0원), 코드 변경 불필요. 회귀 테스트로만 확인
- A127 cancel_makeup N+1 배치 전환 -- makeup.rs 수정 범위에 해당하지만 독립적 리팩터링이므로 별도 유지
- A128/A129 -- Low 우선순위, 범위 밖 유지
- A130/A131/A132 -- db.rs/auth.rs/로깅 관련, 이번 스프린트 수정 대상 아님

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint23-retrospective.md`

| 액션 ID | 내용 | 반영 방법 |
|---------|------|-----------|
| A114 (Low, 7회 이연) | sync_single_date 이력 패턴 통일 | **T0에 강제 포함** -- attendance.rs 대규모 수정 스프린트이므로 더 이상 이연 불가 |
| A127 (Medium) | cancel_makeup N+1 배치 전환 | 범위 밖 유지 (독립 리팩터링) |
| A128 (Low) | cancel_makeup docstring 갱신 | 범위 밖 유지 |
| A129 (Low) | 보강 시간 입력 드롭다운 전환 | 범위 밖 유지 |
| A130 (Medium) | pool() TOCTOU 경합 모니터링 | 범위 밖 유지 (db.rs 변경 없음) |
| A131 (Medium) | LockScreen 오류 분기 구조화 | 범위 밖 유지 |
| A132 (Low) | 구조화 로깅 도입 검토 | 범위 밖 유지 |

## 작업 목록

---

### T0: 이전 회고 액션 A114 -- sync_single_date 이력 패턴 통일 -- 1h

> **7스프린트 이연 최종 해소. attendance.rs 대규모 수정에 선행하여 코드 일관성 확보.**

`sync_single_date` 내부의 이력 조회/갱신 패턴을 `generate_impl`과 통일한다. 구체적으로 attendance 이력 조회 시 달력월 기반 하드코딩을 교습기간 기반 헬퍼로 교체.

**완료 기준**:
- ⬜ 이력 조회 패턴이 `generate_impl`과 동일한 방식 사용
- ⬜ 기존 단위 테스트 회귀 없음

---

### T1: A1 apply_schedule_change_impl 태깅 수정 (데이터 오염 -- CRITICAL) -- 3h · skill: systematic-debugging

> **이 버그의 실제 트리거**: schedule-editor.tsx 저장 시 `set_schedule` 직후 `applyScheduleChange` 자동 호출 -> A1 발동. DELETE 후 재INSERT라 올바른 행도 덮어써 능동 오염.**

**수정 내용**:
1. `Period` struct(attendance.rs:1384)에 `year_month: String` 필드 추가
2. 조회 SELECT(attendance.rs:1375)에 `study_periods.year_month` JOIN 추가
3. attendance.rs:1440의 `&ds[..7]`(달력월) -> `period.year_month`(교습기간 라벨)로 교체

**단위 테스트** (다월 경계일 시드):
- ⬜ `seed_period("2026-08","2026-07-30","2026-09-02")` 기반 경계일(7/30, 9/2) 출결 재생성 시 year_month가 "2026-08"
- ⬜ 단일월 교습기간 회귀 테스트

**완료 기준**:
- ⬜ apply_schedule_change가 교습기간 year_month로 태깅
- ⬜ 경계일 단위 테스트 통과

---

### T2: A2 create_makeup_with_absences_impl 태깅 수정 (데이터 오염 -- CRITICAL) -- 2h · skill: systematic-debugging

**수정 내용**:
- `makeup.rs:371,511`에서 보강 출결 year_month 결정 시 보강일의 달력월 대신 `SELECT year_month FROM study_periods WHERE start_date <= date AND end_date >= date AND is_confirmed=1` 조회 결과 사용
- 조회 결과가 없으면(교습기간 미등록 날짜) 기존 달력월 폴백 + 경고 로그

**단위 테스트**:
- ⬜ 다월 교습기간 경계일에 보강 등록 시 year_month가 교습기간 라벨
- ⬜ 교습기간 미등록 날짜 폴백 동작 검증

**완료 기준**:
- ⬜ 보강 출결 태깅이 교습기간 year_month 기준
- ⬜ 경계일 단위 테스트 통과

---

### T3: B1 move_attendance_impl "동월 한정" 판정 + year_month 갱신 수정 -- 2h

**수정 내용**:
1. attendance.rs:1224의 달력월 비교(`format("%Y-%m")`)를 "두 날짜가 같은 study_periods 행 소속인지" 기준으로 전환
2. 이동 UPDATE(attendance.rs:1283-1293)에 year_month 갱신 추가 -- 이동 대상 날짜의 교습기간 year_month로 SET
3. 양방향(오차단/오허용) 모두 수정

**단위 테스트**:
- ⬜ 같은 교습기간 내 달력월 경계 이동 허용 (7/30 -> 8/1, 같은 교습기간)
- ⬜ 다른 교습기간 간 이동 차단 (8/31 -> 9/3, 서로 다른 교습기간)
- ⬜ 이동 후 year_month 정합 검증

**완료 기준**:
- ⬜ 동월 한정 판정이 교습기간 기준
- ⬜ 이동 후 year_month가 대상 날짜의 교습기간 라벨로 갱신

---

### T4: B2~B6 표시/동작 오류 일괄 수정 -- 3h

> **5건을 하나의 Task로 묶음 -- 모두 "달력월 -> 교습기간 범위" 동일 패턴 적용.**

**B2** `build_day_schedules` (attendance.rs:768):
- 학사마커 조회를 달력월 범위 -> `load_confirmed_period` 활용 교습기간 범위로 교체

**B3** `get_notice_month_info` (notice.rs:609-615):
- 보강데이 필터 `event_date LIKE 'YYYY-MM-%'` -> 교습기간 start_date/end_date 범위 조건

**B4** 대시보드 알림 (dashboard.rs:461,487):
- 달력월 vs 교습기간값 비교 -> 현재 활성 교습기간 라벨 기준으로 통일

**B5** `get_makeup_eligible_dates` (makeup.rs:216-269):
- 보강 가능일 조회를 달력월 범위 -> 교습기간 범위 기반으로 교체

**B6** `reinstate_student_impl` (students.rs:509):
- `makeup_deadline` vs `strftime('%Y-%m','now')` -> study_periods 기준 비교로 교체

**완료 기준**:
- ⬜ B2~B6 각각 교습기간 범위 기반으로 전환 완료
- ⬜ 기존 단위 테스트 회귀 없음

---

### T5: B7 학사일정 "지난달 수정 차단" 가드 설계 판단 -- 1h · skill: brainstorming

> **설계 판단 필요**: `academic.rs:1067,1164`의 "지난달 수정 차단" 가드가 달력월 기준. schedule_events는 순수 날짜 엔티티(교습기간과 독립적으로 존재 가능)이므로 교습기간 기준 적용이 올바른지 확인 필요.

**검토 사항**:
1. schedule_events 생성/수정/삭제 시 "지난달" 의미가 달력월인지 교습기간인지
2. 다월 교습기간 경계에서 차단 정책이 어떻게 동작해야 하는지
3. 수정 여부를 결정하고, 수정 시 구현 / 미수정 시 의도적 달력월 유지 사유를 주석으로 기록

**산출물**: 결정 내용을 scope.md에 기록. ADR 필요 시 `docs/arch/adr-013-academic-guard-scope.md` 작성.

**완료 기준**:
- ⬜ B7에 대한 설계 결정 완료 + 결정에 따른 구현 또는 의도적 유지 주석

---

### T6: B8 MakeupRegisterDialog 프론트엔드 수정 -- 0.5h

**수정 내용**:
- `src/components/attendance/MakeupRegisterDialog.tsx:92`의 `eventDate.slice(0,7)` -> authoritative `yearMonth` prop 사용

**완료 기준**:
- ⬜ deadline 비교가 prop 기반
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` 통과

---

### T7: E1 데이터 전수 보정 마이그레이션 V313 -- 3h

> **사용자 결정: 자동 보정. 구현 방식: 마이그레이션 (앱 업데이트 시 일회 자동 실행).**

**V313 `313__fix_year_month_tagging.sql` 설계**:
```sql
-- regular_attendances: event_date 소속 확정 교습기간의 year_month로 교정
UPDATE regular_attendances SET year_month = (
  SELECT sp.year_month FROM study_periods sp
  WHERE sp.start_date <= regular_attendances.event_date
    AND sp.end_date >= regular_attendances.event_date
    AND sp.is_confirmed = 1
)
WHERE year_month != (
  SELECT sp.year_month FROM study_periods sp
  WHERE sp.start_date <= regular_attendances.event_date
    AND sp.end_date >= regular_attendances.event_date
    AND sp.is_confirmed = 1
);
-- makeup_attendances: 동일 패턴
```

**교습기간에 속하지 않는 event_date 행 처리 정책**:
- 서브쿼리 결과가 NULL인 행(미확정/미등록 교습기간 소속)은 UPDATE 대상에서 자동 제외(WHERE 절 NULL != 비교는 FALSE)
- 해당 행은 기존 year_month 유지 -- 데이터 손실 없음
- D1 진단 검사에서 이런 고아 행을 별도 경고 항목으로 표시

**멱등성**: WHERE 절이 "현재 값 != 올바른 값"인 행만 갱신하므로 반복 실행해도 안전.
**안전성**: 트랜잭션 내 실행(sqlx migrate 기본 동작). UNIQUE(student_id, event_date) 제약은 event_date를 변경하지 않으므로 충돌 없음.

**완료 기준**:
- ⬜ V313 마이그레이션 파일 작성 + `sqlx migrate run` 정상 적용
- ⬜ 보정 전후 행수 불변 (데이터 손실 없음)
- ⬜ 멱등성 검증 (2회 실행 시 0건 갱신)

---

### T8: D1 자가진단 신규 검사 -- year_month 불변식 + 진단 파라미터 점검 -- 2h

**신규 검사 항목** (diagnosis.rs 검사 8번):
- `regular_attendances.year_month`(및 `makeup_attendances.year_month`) vs `event_date 소속 확정 교습기간의 year_month` 불일치 검출
- 기존 검사 2(당월 출결 미생성) / 검사 4(스케줄-출결 요일 불일치)는 구조적으로 이 버그를 검출하지 못함

**진단 파라미터 점검** (diagnosis.rs:78-79,584):
- `run_diagnosis`가 달력월을 "당월"로 넘기는 부분을 활성 교습기간 year_month 기준으로 변경 검토
- 검사 2("재원중 당월 출결 미생성"), 검사 3("재원중 당월 청구 미생성")의 "당월" 기준이 달력월인지 교습기간인지 정합성 확인

**단위 테스트**:
- ⬜ 오태깅된 행이 있을 때 검사 8이 이상 건수를 정확히 반환
- ⬜ 모든 행이 정합할 때 검사 8이 0건 반환

**완료 기준**:
- ⬜ 검사 8번 동작 + 기존 검사 1~7 회귀 없음
- ⬜ 진단 파라미터 정합성 확인/수정 완료

---

### T9: D2 다월 교습기간 회귀 테스트 일괄 추가 -- 3h

> **현재 다월 테스트는 `sync_attendance_on_schedule_change`에만 존재 -> 미검출 원인.**

모든 수정 대상 함수에 `seed_period("2026-08","2026-07-30","2026-09-02")` 기반 경계일 테스트를 추가:

| 대상 함수 | 테스트 시나리오 |
|-----------|---------------|
| `apply_schedule_change_impl` | 경계일(7/30, 9/2) 재생성 시 year_month 검증 |
| `move_attendance_impl` | 교습기간 내 달력월 경계 이동 허용/차단 |
| `build_day_schedules` | 경계일 학사마커 조회 범위 검증 |
| `count_ungenerated` | 다월 교습기간에서 생성 건수 정합성 |
| `create_makeup_with_absences_impl` | 경계일 보강 등록 year_month 검증 |
| `get_makeup_eligible_dates` | 교습기간 범위 기반 가능일 반환 |

**하류 전파 확인 테스트** (C 그룹 -- 코드 수정 불필요, 테스트로 확인):
- ⬜ `toggle` -> `makeup_deadline` 소멸 기한 정합 (경계일)
- ⬜ `compute_summary` 보강완료시간 정합 (경계일)
- ⬜ `dashboard:347` 출결일수 집계 (경계일)
- ⬜ `export.rs` 시트 출력 (경계일)

**청구 회귀 확인** (SAFE 확인):
- ⬜ 다월 교습기간 경계에서 청구 금액 불변 검증

**완료 기준**:
- ⬜ 신규 테스트 최소 15건 추가
- ⬜ 전체 `cargo test` 통과

---

### T10: 통합 검증 -- 1h

**자동 검증 체크리스트**:
- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (목표: 490건 이상)
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ⬜ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ⬜ `pnpm lint` clean
- ⬜ `pnpm tsc --noEmit` clean
- ⬜ `pnpm build` static export 전수 통과

**마이그레이션 self-check**:
- ⬜ V313 1건 정상 적용 확인

---

## Capacity 추정

| Task | 예상 시간 | 비고 |
|------|----------|------|
| T0 (A114 이력 패턴) | 1h | 패턴 통일, 소규모 |
| T1 (A1 apply_schedule_change) | 3h | struct 수정 + SELECT 변경 + 테스트 |
| T2 (A2 create_makeup) | 2h | 태깅 패턴 동일 |
| T3 (B1 move_attendance) | 2h | 판정 로직 + UPDATE 갱신 |
| T4 (B2~B6 일괄) | 3h | 5건 동일 패턴 |
| T5 (B7 설계 판단) | 1h | brainstorming + 결정 |
| T6 (B8 프론트엔드) | 0.5h | prop 교체 1줄 |
| T7 (V313 마이그레이션) | 3h | SQL 작성 + 검증 |
| T8 (진단 검사) | 2h | 검사 1건 + 파라미터 점검 |
| T9 (회귀 테스트) | 3h | 15건+ 테스트 작성 |
| T10 (통합 검증) | 1h | 자동 검증 실행 |
| **합계** | **21.5h** | |

**Capacity 계산**: AI 페어 프로그래밍 1인 x 10일(2주) x 4h/일 = **40h 가용**
**여유율**: 21.5h / 40h = 53.75% -- 충분한 여유 (시각 검증 + 예기치 않은 이슈 대응 버퍼 확보)

## 완료 기준 (Definition of Done)

**필수**
- ⬜ A1/A2 데이터 오염 버그 2건 수정 완료 -- 재생성/보강 태깅이 교습기간 year_month 기준
- ⬜ B1~B6 표시/동작 오류 6건 수정 완료 -- 교습기간 범위 기반 통일
- ⬜ B7 설계 판단 완료 + 결정에 따른 처리
- ⬜ B8 프론트엔드 수정 완료
- ⬜ V313 데이터 전수 보정 마이그레이션 적용 + 멱등성 검증
- ⬜ 자가진단 검사 8번(year_month 불변식) 추가
- ⬜ 다월 교습기간 회귀 테스트 15건+ 추가
- ⬜ A114 (sync_single_date 이력 패턴) 최종 해소
- ⬜ 하류 전파(C 그룹) 테스트로 정합성 확인
- ⬜ 청구/수납 회귀 없음 테스트 확인
- ⬜ `cargo test` 전체 통과 (490건+ 목표)
- ⬜ `cargo clippy --all-targets -- -D warnings` clean
- ⬜ `cargo check --features cipher` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 계획 |
|----|--------|--------|-----------|
| R152 | V313 보정 마이그레이션에서 교습기간 미등록 날짜의 출결 행이 보정 대상에서 제외됨 -- 해당 행은 기존 값 유지되나 진단 검사에서 경고 표시 | 낮음 | D1 진단 검사에서 고아 행 별도 경고. 교습기간 등록 후 재진단으로 자연 해소 |
| R153 | B7 학사일정 가드 설계 판단에서 달력월 유지/교습기간 전환 중 어느 쪽이든 edge case 존재 | 중간 | T5에서 brainstorming 스킬로 대안 비교 후 결정. 결정 후 scope.md에 기록 |
| R154 | move_attendance 동월 한정 정책 변경으로 기존 UX 변화 -- 이전에 달력월 기준으로 차단됐던 이동이 허용될 수 있음 | 중간 | 교습기간 기준이 의미적으로 올바름. 시각 검증에서 확인 |

## 검증 매트릭스

| 검증 항목 | Sprint 24 |
|-----------|-----------|
| cargo test | 필수 (490건+) |
| cargo clippy --all-targets | 필수 |
| cargo check --features cipher | 필수 |
| pnpm lint | 필수 |
| pnpm tsc --noEmit | 필수 |
| pnpm build | 필수 |
| V313 마이그레이션 적용 | 필수 |
| 다월 경계일 시각 검증 | 권장 (배포 단계) |

## 예상 산출물

- 수정 파일: `attendance.rs`, `makeup.rs`, `students.rs`, `notice.rs`, `dashboard.rs`, `academic.rs`, `diagnosis.rs`, `MakeupRegisterDialog.tsx`
- 신규 파일: `src-tauri/migrations/313__fix_year_month_tagging.sql`
- ADR (조건부): `docs/arch/adr-013-academic-guard-scope.md` (T5 결과에 따라)
- 테스트 추가: 15건+ 신규 단위 테스트

## 참고 사항

- **정답 패턴 참조**: `attendance.rs:1547-1555` `sync_single_date`의 study_periods 조회 패턴을 모든 수정 대상에 일관 적용
- **Sprint 21 R136 계열**: 당시 `generate_impl` + `sync_single_date`만 수정, 나머지 잔존 -- 이번 스프린트에서 전수 해소
- **하류 전파 (C 그룹)**: A1/A2 수정으로 자동 해소되나 별도 코드 수정 불필요, 테스트로 확인
- **DB 마이그레이션 번호**: V313 (현재 최신 V312, 300대 보정/확장 블록)
