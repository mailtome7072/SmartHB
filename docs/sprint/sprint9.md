# Sprint Plan sprint9

## 기간
2026-05-24 ~ 2026-06-07 (2주 예상)

## 목표
**Phase 3 (보강 + 소멸) 첫 sprint** — 보강 등록(개별/일괄) + 보강-결석 매칭 로직 + 보강 약속 취소/미등원 처리 + 결석 이력 조회를 완성하여, 원장이 UC-4(결석 원생 보강 처리) 핵심 흐름을 수행할 수 있게 한다.

### Phase 위치
- **Phase 3**: 보강 + 소멸 (Sprint 9~10)
- **Sprint 9**: 보강 등록 + 매칭 (본 sprint)
- **Sprint 10**: 소멸 자동 전이 + 퇴교 보강 처리 + 캘린더 뷰

### ROADMAP 연계
- ROADMAP L460-500: Sprint 9 청사진
- 의존성 맵: Phase 2 (학사+출결) 완료 필수 -- **v0.4.0 배포 완료 (2026-05-24)**
- PI-02: 보강-결석 시간값 매칭 규칙 — **일 단위 매칭 확정** (사용자 결정 2026-05-24)

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint8-retrospective.md` (Sprint 8 회고, 2026-05-24)

| 액션 ID | 항목 | 반영 방법 | 비고 |
|---------|------|-----------|------|
| A39 | 마이그레이션 self-check -- scope.md 설계 vs 실제 SQL 1:1 대조 강제 | T9 통합 검증에 "마이그레이션 self-check" 항목 명시. sprint-close 직전 scope.md 설계와 SQL 대조 의무화 | **High** -- 프로세스 개선 |
| A40 | sprint-review 산출물 파일 작성 강제 (4종: test-report / risk-register / retrospective / code-review) | T9 통합 검증 DoD에 "sprint-review 산출물 4종 경로 명시" 체크리스트 포함 | **High** -- 프로세스 개선 |
| A41 | "결석(일)" 라벨 → "미처리 결석(일)" 변경 + `compute_summary` 주석 명확화 | **T7에서 흡수** -- 보강 도메인 진입으로 라벨 의미 혼동이 실제 사용자 영향 발생 시점 | **Medium** |
| A42 | `get_attendance_grid` N+1 → batch 쿼리 리팩토링 | **검토 후 이연** -- 보강 매칭 IPC 신규 도입으로 scope 압박. 현재 PRD 요건(50명x31일 < 1초) 충족 중이므로 Sprint 10 이후로 이연 | Medium |
| A43 | `validate_year_month` 월 범위(01-12) 검증 강화 | **T2에서 흡수** -- 보강 IPC가 year_month 입력받으므로 자연스러운 타이밍 | **Low** |
| A44 | R48-b salt buffer ZeroizeOnDrop 시그니처 변경 | **이연** -- 보안 도메인, 보강과 무관. 광범위 영향 | Medium |
| A45 | 반응형 폰트/셀 너비 | **이연** -- UX 전반, 본 sprint scope 외 | Medium |
| A46 | 한글 자모 부분 일치 검색 | **이연** -- 검색 영역, 본 sprint 흡수 시 capacity 초과 | Medium |
| A47 | 한국어 리터럴 잔존 (R49) | **이연** -- 기능 영향 없음 | Low |
| A48 | 2027년 공휴일 V401 | **이연** -- 시점 미도래 | Low |

**흡수 항목**: A41, A43 (2건)
**프로세스 반영**: A39, A40 (2건)
**이연**: A42, A44, A45, A46, A47, A48 (6건)

---

## PI-02 사용자 결정: 보강-결석 시간값 매칭 규칙

### 배경
보강 1건이 결석 N건을 충당할 때, **시간값(class_minutes)을 어떻게 매칭**하는가?

### 옵션

| 옵션 | 설명 | 장점 | 단점 |
|------|------|------|------|
| **A. 일 단위 매칭 (보수적)** | 보강 1일 = 결석 N일 충당. 시간값 비교 없이 일(day) 기준으로 매칭 | 단순, 원장이 이해하기 쉬움 | 1시간 수업 결석 = 3시간 수업 결석과 동등하게 처리됨 |
| **B. 분 단위 매칭** | 보강 class_minutes >= SUM(결석 class_minutes) 검증 | 정밀, 공정 | 원장이 시간 계산 부담, UI 복잡 |

### 현재 구현 상태
- `regular_attendances.class_minutes`: 결석별 수업시간 이미 저장
- `makeup_attendances.class_minutes`: 보강별 수업시간 이미 저장
- `보강필요시간 = SUM(absence.class_minutes) - SUM(makeup_attended.class_minutes)` (PRD §4.5)

### 기본 채택 (PI-02 미결정 시)
**옵션 A (일 단위 매칭)** -- PRD v1.2 "보강은 일 단위 매칭" 명시, ROADMAP R5 보수적 채택 방침과 일치.

### ✅ 사용자 확정 (2026-05-24)
**옵션 A (일 단위 매칭)** 채택. 사유: 단순/이해 용이 + 원장이 시간 계산 부담 없음. 시간값 검증 없이 일 단위로 결석 N건 ↔ 보강 1건 매칭. 향후 분 단위 전환은 R58 리스크로 추적 — T3 검증 로직 수정만으로 가능.

---

## 작업 목록

### T1: PI-02 결정 반영 + 보강 도메인 설계 검토 (2h)
> 배경: 보강 매칭 규칙(일 vs 분 단위)을 확정하고, 기존 스키마(V106/V107)로 보강 전체 흐름이 구현 가능한지 검증한다.

- ⬜ PI-02 사용자 결정 수렴 (미결정 시 일 단위 매칭 채택)
- ⬜ 기존 스키마 검증: `regular_attendances.makeup_attendance_id` FK + `makeup_attendances` 2상태 -- 신규 마이그레이션 필요 여부 최종 판단
- ⬜ scope.md 작성: 보강 도메인 전체 데이터 흐름도 + 변경 파일 목록

**예상 변경 파일**: `docs/sprint/sprint9/scope.md`
**예상 소요**: 2h
**AC**: scope.md 완성 + PI-02 확정 기록

---

### T2: 보강 IPC 백엔드 -- 미처리 결석 조회 + validate_year_month 보강 (6h)
> 배경: 보강 등록 UI가 사용하는 핵심 데이터 소스. 원생별 미처리 결석 목록(소멸기한 임박 순) + 보강 가능 일자 판별 로직.

- ⬜ `get_pending_absences(student_id) -> Vec<PendingAbsence>` IPC 신규
  - 조건: `status = 'absent' AND makeup_attendance_id IS NULL`
  - 정렬: `makeup_deadline ASC, event_date ASC` (소멸기한 임박 순)
  - 반환: `{id, event_date, class_minutes, makeup_deadline, absence_memo}`
- ⬜ `get_makeup_eligible_dates(student_id, year_month) -> Vec<EligibleDate>` IPC 신규
  - 비수업일 중 `allows_makeup_class = 1` 인 학사일정이 있는 일자 반환
  - 정규 수업 요일이 아닌 날짜 + `schedule_events` JOIN으로 `allows_makeup_class` 확인
  - "보강 진행 가능 OFF" 일자 제외 (AC-4.4-3)
- ⬜ (A43) `validate_year_month` 월 범위 강화: `2026-00` / `2026-13` 같은 무효 입력 차단 + 사용자 친화 에러 메시지
- ⬜ 단위 테스트: 미처리 결석 조회 + 소멸기한 정렬 + 보강 가능 일자 필터링 + validate_year_month 범위 검증

**예상 변경 파일**: `src-tauri/src/commands/attendance.rs` (또는 `makeup.rs` 신규 서브모듈), `src-tauri/src/lib.rs`
**예상 소요**: 6h
**AC**: IPC 2종 + validate 강화 + 단위 테스트 통과

---

### T3: 보강 IPC 백엔드 -- 보강 등록 + 매칭 (6h) · skill: karpathy-guidelines
> 배경: 보강 등록의 핵심 트랜잭션. 보강 출결 1건 생성 + 선택된 결석 N건을 "보강완료"로 원자적 전이.

- ⬜ `create_makeup_with_absences(student_id, event_date, class_minutes, absence_ids: Vec<i64>) -> Result<MakeupResult>` IPC 신규
  - 트랜잭션 내 실행 (원자성 필수)
  - 검증 1: `event_date`가 보강 가능 일자인지 확인 (`allows_makeup_class` + 해당 학생의 비수업일)
  - 검증 2: `absence_ids` 모두 해당 학생의 미처리 결석인지 확인 (`status='absent' AND makeup_attendance_id IS NULL`)
  - 검증 3: PI-02 결정에 따른 시간값 매칭 규칙 적용 (일 단위: 검증 생략 / 분 단위: `class_minutes >= SUM(absences.class_minutes)`)
  - 실행: `INSERT INTO makeup_attendances` → `UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=? WHERE id IN (absence_ids)`
  - 감사 로그: `MakeupCreated` 이벤트 기록
- ⬜ 단위 테스트: 정상 매칭 / 무효 absence_id 거부 / 이미 매칭된 결석 거부 / 트랜잭션 원자성 / 보강 불가 일자 차단

**예상 변경 파일**: `src-tauri/src/commands/attendance.rs` (또는 `makeup.rs`), `src-tauri/src/commands/audit.rs`
**예상 소요**: 6h
**AC**: 보강 등록 트랜잭션 + 5종 검증 + 단위 테스트 100% 커버

---

### T4: 보강 IPC 백엔드 -- 취소 + 미등원 + 일괄 (5h)
> 배경: 보강 약속 취소(결석 환원) + 보강결석(미등원) + 보강데이 일괄 등록 지원 IPC.

- ⬜ `cancel_makeup(makeup_id) -> Result<()>` IPC 신규
  - 트랜잭션: 연결된 `regular_attendances` 의 `makeup_attendance_id=NULL, status='absent'` 환원 → `DELETE makeup_attendances WHERE id=?`
  - 감사 로그: `MakeupCancelled`
  - AC-4.5-4: 연결된 결석들이 결석 상태로 정확히 환원되는지 검증
- ⬜ `mark_makeup_absent(makeup_id) -> Result<()>` IPC 신규
  - `UPDATE makeup_attendances SET status='makeup_absent'`
  - 연결된 결석: `makeup_attendance_id=NULL, status='absent'` 환원 (결석 상태 유지, 새 결석 미생성)
  - 감사 로그: `MakeupAbsent`
- ⬜ `batch_create_makeups(event_date, entries: Vec<{student_id, absence_ids}>) -> Result<BatchResult>` IPC 신규
  - 단일 트랜잭션으로 다중 원생의 보강 등록 일괄 처리
  - 실패 원생은 건너뛰고 성공/실패 결과 반환 (`BatchResult { succeeded: Vec, failed: Vec<{student_id, reason}> }`)
- ⬜ 단위 테스트: 취소 후 결석 환원 / 미등원 후 결석 상태 유지 / 일괄 등록 부분 성공/부분 실패

**예상 변경 파일**: `src-tauri/src/commands/attendance.rs` (또는 `makeup.rs`), `src-tauri/src/commands/audit.rs`, `src-tauri/src/lib.rs`
**예상 소요**: 5h
**AC**: IPC 3종 + 환원 정합성 + 일괄 처리 + 단위 테스트

---

### T5: TypeScript IPC 래퍼 + 도메인 타입 (2h) · skill: frontend-design
> 배경: T2~T4에서 추가한 IPC 커맨드의 프론트엔드 추상화 레이어.

- ⬜ `src/lib/tauri/index.ts` 래퍼 추가 (7~8종):
  - `getPendingAbsences(studentId)`, `getMakeupEligibleDates(studentId, yearMonth)`
  - `createMakeupWithAbsences(...)`, `cancelMakeup(makeupId)`, `markMakeupAbsent(makeupId)`
  - `batchCreateMakeups(...)`, `getAbsenceHistory(studentId)`
- ⬜ `src/types/makeup.ts` 도메인 타입 신규:
  - `PendingAbsence`, `EligibleDate`, `MakeupResult`, `BatchResult`, `AbsenceHistoryItem`
- ⬜ dev mode fallback 구현 (Tauri 미실행 시 mock 데이터 반환)

**예상 변경 파일**: `src/lib/tauri/index.ts`, `src/types/makeup.ts`
**예상 소요**: 2h
**AC**: 래퍼 7~8종 + 타입 5종 + dev fallback

---

### T6: 보강 등록 (개별) 프론트엔드 UI (6h) · skill: frontend-design
> 배경: 출결표에서 비수업일 셀 클릭 → 보강 등록 다이얼로그 → 충당 결석 선택 → 확정. PRD §4.5.4.

- ⬜ `MakeupRegistrationDialog` 컴포넌트 신규
  - 트리거: 출결표 그리드에서 보강 가능 일자 셀 클릭 (기존 `AttendanceGrid` 확장)
  - 좌측: 보강 일자 + 수업시간 표시
  - 우측: 미처리 결석 리스트 (소멸기한 임박 순, 체크박스 다중 선택)
    - 소멸기한 임박 항목 시각 강조 (색상 구분)
    - 각 항목: 결석일 / 수업시간 / 소멸기한 / 메모
  - 확정 버튼: `createMakeupWithAbsences` 호출 → 성공 시 출결 그리드 리프레시
  - "보강 진행 가능 OFF" 일자는 셀 비활성화 + 안내 메시지 (AC-4.4-3)
- ⬜ `AttendanceGrid` 확장: 보강 가능 일자 셀에 시각적 표시 (배경색 또는 아이콘)
- ⬜ 출결 셀 상태 표시 업데이트: `makeup_done` → 빨강 + 보강일자 표시 (PRD §4.5.2)
- ⬜ TanStack Query 무효화: 보강 등록 성공 시 attendance grid + pending absences 캐시 무효화

**예상 변경 파일**: `src/components/attendance/MakeupRegistrationDialog.tsx` (신규), `src/components/attendance/AttendanceGrid.tsx`, `src/app/attendance/page.tsx`
**예상 소요**: 6h
**AC**: 보강 등록 다이얼로그 → 결석 선택 → 확정 → 그리드 반영 전체 흐름

---

### T7: 보강데이 일괄 등록 + 결석 라벨 보강 (5h) · skill: frontend-design
> 배경: 보강데이 셀 클릭 → 보강 필요 원생 리스트 → 다중 선택 → 일괄 확정. PRD §4.5.5. + A41 흡수.

- ⬜ `BatchMakeupDialog` 컴포넌트 신규
  - 트리거: 보강데이 코드가 배치된 일자 셀 클릭
  - 보강 필요 원생 리스트 (소멸기한 임박 순, 색상 강조)
  - 원생별 체크박스 + 충당 결석 일수 지정 (기본: 소멸기한 임박 순 자동 선택)
  - 일괄 확정: `batchCreateMakeups` 호출 → 부분 성공/실패 결과 표시
- ⬜ 보강 약속 취소 + 미등원 UI
  - 보강 등록된 셀 우클릭 또는 상세 보기 → 취소/미등원 선택
  - 취소: `cancelMakeup` → 결석 환원 확인 다이얼로그
  - 미등원: `markMakeupAbsent` → "보강결석" 마킹 확인 다이얼로그
- ⬜ (A41) 출결표 헤더 "결석(일)" → "미처리 결석(일)" 라벨 변경 + `compute_summary` 주석 명확화

**예상 변경 파일**: `src/components/attendance/BatchMakeupDialog.tsx` (신규), `src/components/attendance/AttendanceGrid.tsx`, `src-tauri/src/commands/attendance.rs` (주석)
**예상 소요**: 5h
**AC**: 일괄 등록 → 결과 표시 + 취소/미등원 동작 + 라벨 변경

---

### T8: 결석 이력 조회 (3h)
> 배경: 원생 상세에서 결석 이력 표. 처리 상태별 시각 구분. PRD §4.5.10.

- ⬜ `get_absence_history(student_id) -> Vec<AbsenceHistoryItem>` IPC 신규
  - 대상: `regular_attendances WHERE status IN ('absent', 'makeup_done', 'makeup_expired')`
  - 컬럼: 결석일 / 수업시간 / 처리 상태 / 보강일 / 소멸기한
  - `makeup_done` 인 경우: `makeup_attendances.event_date`를 JOIN으로 보강일 표시
  - 정렬: `event_date DESC`
- ⬜ `AbsenceHistoryTable` 컴포넌트 신규
  - 원생 상세 화면(`/students/[id]`) 또는 출결 화면 내 탭으로 배치
  - 처리 상태별 시각 구분 (AC-4.5-7):
    - 미처리 결석(`absent`): 빨간 배경
    - 보강완료(`makeup_done`): 초록 배경 + 보강일 표시
    - 보강소멸(`makeup_expired`): 회색 배경 + "소멸" 라벨
  - 보강일이 있는 결석은 어떤 보강 일자에 충당되었는지 명확히 표시

**예상 변경 파일**: `src-tauri/src/commands/attendance.rs` (또는 `makeup.rs`), `src/components/attendance/AbsenceHistoryTable.tsx` (신규), `src/lib/tauri/index.ts`
**예상 소요**: 3h
**AC**: 결석 이력 조회 + 3종 상태 시각 구분 + 보강일 연결 표시

---

### T9: 통합 검증 (3h)
> 배경: 자동 검증 7항목 + 사용자 시각 검증 + 마이그레이션 self-check (A39) + sprint-review 산출물 경로 명시 (A40).

- ⬜ 자동 검증 7항목:
  1. `cargo test --lib --manifest-path src-tauri/Cargo.toml` (cipher off) -- 전체 통과
  2. `cargo test --lib --manifest-path src-tauri/Cargo.toml --features cipher` (cipher on) -- 전체 통과
  3. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` (cipher off) -- clean
  4. `cargo clippy --manifest-path src-tauri/Cargo.toml --features cipher -- -D warnings` (cipher on) -- clean
  5. `pnpm lint` -- clean
  6. `pnpm tsc --noEmit` -- clean
  7. `pnpm build` -- static export 성공
- ⬜ 마이그레이션 self-check (A39): scope.md 설계 vs 실제 SQL 1:1 대조 (신규 마이그레이션이 있는 경우)
- ⬜ sprint-review 산출물 경로 명시 (A40):
  - `docs/test-reports/sprint9.md`
  - `docs/risk-register/YYYY-MM-DD.md`
  - `docs/sprint-retrospectives/sprint9-retrospective.md`
  - `docs/code-reviews/sprint9.md`
- ⬜ 사용자 시각 검증 세션 (1h) -- Sprint 8 A38 패턴 유지
  - 보강 등록(개별): 비수업일 셀 → 결석 선택 → 확정 → 그리드 반영
  - 보강데이 일괄: 보강데이 셀 → 원생 선택 → 확정 → 결과 표시
  - 보강 취소: 등록된 보강 → 취소 → 결석 환원 확인
  - 보강결석: 미등원 → 마킹 → 결석 상태 유지 확인
  - 결석 이력: 원생 상세에서 3종 상태 시각 구분 확인
  - 출결표 라벨: "미처리 결석(일)" 변경 확인

**예상 소요**: 3h (자동 1h + 시각 검증 1h + self-check/AC 마킹 1h)
**AC**: 자동 7항목 전수 통과 + 시각 검증 "이상없음" + self-check 통과

---

## 작업 요약 및 Capacity

| Task | 설명 | 예상 소요 | 스킬 |
|------|------|----------|------|
| T1 | PI-02 결정 + 보강 도메인 설계 검토 | 2h | — |
| T2 | 미처리 결석 조회 + validate 보강 IPC | 6h | — |
| T3 | 보강 등록 + 매칭 IPC | 6h | karpathy-guidelines |
| T4 | 취소 + 미등원 + 일괄 IPC | 5h | — |
| T5 | TS IPC 래퍼 + 도메인 타입 | 2h | frontend-design |
| T6 | 보강 등록(개별) UI | 6h | frontend-design |
| T7 | 보강데이 일괄 + 취소/미등원 UI + A41 라벨 | 5h | frontend-design |
| T8 | 결석 이력 조회 | 3h | — |
| T9 | 통합 검증 | 3h | — |
| **합계** | | **38h** | |

**Capacity**: 40h (1인 x 10일 x 4h/일)
**여유**: +2h (5%) -- Sprint 8(41h) 대비 2h 여유. 신규 도메인 복잡도 고려하여 보수적 배분.

---

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 방안 |
|----|--------|--------|-----------|
| R58 | PI-02 미결정으로 보강 매칭 규칙이 sprint 중간에 변경 | 중간 | T1에서 확정. 미결정 시 일 단위 매칭 보수적 채택. 분 단위 전환은 T3 검증 로직만 수정하면 되므로 영향 제한적 |
| R59 | 보강 등록 트랜잭션에서 결석 N건 UPDATE 시 partial failure | 높음 | SQLite 단일 파일 트랜잭션이므로 ACID 보장. 트랜잭션 내 모든 UPDATE를 한 번에 커밋. 테스트에서 중간 실패 시나리오 검증 |
| R60 | 보강 가능 일자 판별 로직이 `schedule_events` + `student_schedules` + `allows_makeup_class` 3-way JOIN으로 복잡 | 중간 | T2에서 쿼리 설계를 scope.md에 먼저 문서화 후 구현. 단위 테스트로 엣지 케이스(보강데이 + 공휴수업일 + 휴원일 조합) 검증 |
| R61 | `AttendanceGrid` 확장 시 기존 출결 UI 퇴행 | 중간 | T6에서 기존 출결 토글 기능 재검증 필수. TanStack Query 캐시 무효화 범위 주의 |
| R62 | 보강 취소/미등원 시 `regular_attendances` 상태 환원이 `compute_summary` 계산과 불일치 | 높음 | T4에서 취소/미등원 후 `compute_summary` 재계산 결과 검증하는 통합 테스트 작성 |

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ 보강 등록(개별) → 결석 "보강완료" 전이 동작 (AC-4.5-3)
- ⬜ 보강데이 일괄 등록 → 다중 원생 일괄 확정 동작 (PRD §4.5.5)
- ⬜ 보강 취소 → 결석 환원 정합성 유지 (AC-4.5-4)
- ⬜ 보강결석(미등원) → 결석 상태 유지, 새 결석 미생성 (PRD §4.5.6)
- ⬜ "보강 진행 가능 OFF" 일자 보강 등록 차단 (AC-4.4-3)
- ⬜ 결석 이력에서 보강완료/보강소멸 시각 구분 (AC-4.5-7)
- ⬜ 보강 비즈니스 규칙 단위 테스트 100% 커버 (PRD §6.5)
- ⬜ `cargo test --lib` cipher off/on 전체 통과 (예상: cipher off 240+ / on 140+)
- ⬜ `cargo clippy -- -D warnings` cipher off/on clean
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과

**프로세스**
- ⬜ 마이그레이션 self-check (A39): scope.md 설계 vs 실제 SQL 1:1 대조
- ⬜ sprint-review 산출물 4종 경로 명시 (A40)
- ⬜ ROADMAP.md Sprint 9 상태 `📋 예정` → `🔄 진행 중` 업데이트
- ⬜ CHANGELOG.md 업데이트

---

## 🧪 Playwright MCP 검증 시나리오 (UC-4 보강 흐름)

```
1. browser_navigate → http://localhost:1420/attendance
2. browser_snapshot → 출결표 렌더링 확인 (보강 가능 일자 셀 시각 표시)
3. browser_click → 비수업일(보강데이) 셀 클릭
4. browser_snapshot → 보강 등록 다이얼로그 (충당 결석 리스트, 소멸기한 임박 순)
5. browser_click → 결석 2건 체크 → "확정" 클릭
6. browser_snapshot → 보강 등록 완료 + 결석 상태 "보강완료" 전이 확인
7. browser_click → 등록된 보강 셀 → "취소" 선택
8. browser_snapshot → 결석 환원 확인 (다시 "결석" 상태)
9. browser_click → 보강데이 셀 → 일괄 등록 다이얼로그
10. browser_snapshot → 보강 필요 원생 리스트 (소멸기한 임박 순)
11. browser_click → 원생 3명 선택 → "일괄 확정"
12. browser_snapshot → 일괄 등록 결과 (성공/실패 표시)
13. browser_navigate → 원생 상세 → 결석 이력 탭
14. browser_snapshot → 결석 이력 표 (3종 상태 시각 구분)
15. browser_console_messages(level: "error") → 콘솔 에러 없음
```

---

## 기술 고려사항

### 데이터 모델 (기존 활용)
- `regular_attendances` (V106/V107): `status` 4상태, `makeup_attendance_id` FK, `makeup_deadline`
- `makeup_attendances` (V106): `status` 2상태 (`makeup_attended` / `makeup_absent`)
- `schedule_codes` (V102): `allows_makeup_class` 필드로 보강 가능 여부 판별
- **신규 마이그레이션 불요 가능성 높음** -- 기존 스키마로 전체 흐름 구현 가능. T1에서 최종 확인.

### 트랜잭션 원자성
- 보강 등록 = `INSERT makeup_attendances` + `UPDATE regular_attendances` (N건) -- 반드시 단일 트랜잭션
- 보강 취소 = `UPDATE regular_attendances` (N건, 환원) + `DELETE makeup_attendances` -- 반드시 단일 트랜잭션
- SQLite `BEGIN IMMEDIATE` + `COMMIT` 패턴 적용

### 보강 가능 일자 판별 쿼리 설계
```sql
-- 해당 월의 날짜 중, 해당 학생의 정규 수업 요일이 아닌 + allows_makeup_class=1 인 학사일정 존재
-- (또는 해당 학생의 수업 요일이어도 allows_makeup_class=1 이면 보강 가능)
SELECT DISTINCT se.event_date
FROM schedule_events se
JOIN schedule_codes sc ON se.schedule_code_id = sc.id
WHERE se.event_date LIKE ? || '%'  -- year_month prefix
  AND sc.allows_makeup_class = 1
  AND NOT EXISTS (
    SELECT 1 FROM regular_attendances ra
    WHERE ra.student_id = ? AND ra.event_date = se.event_date
  )
```
> 상세는 T1 scope.md에서 확정. 위는 초안.

### 프론트엔드 패턴
- `MakeupRegistrationDialog` / `BatchMakeupDialog`: shadcn/ui Dialog + Checkbox 조합
- TanStack Query: `useQuery(['pending-absences', studentId])`, `useMutation(createMakeupWithAbsences)`
- 낙관적 업데이트 대신 mutation 성공 후 invalidateQueries 패턴 (정합성 우선)

### sprint-review 산출물 (A40)
sprint-review 에이전트 호출 시 아래 4종 파일 경로를 prompt에 명시:
1. `docs/test-reports/sprint9.md`
2. `docs/risk-register/YYYY-MM-DD.md`
3. `docs/sprint-retrospectives/sprint9-retrospective.md`
4. `docs/code-reviews/sprint9.md`

---

## 마이그레이션 설계 (scope.md 사전 문서화 -- A39 대응)

### 신규 마이그레이션 후보 분석

| 후보 | 필요 여부 | 근거 |
|------|----------|------|
| V108: `makeup_attendances` 에 `cancelled_reason` 컬럼 | **불요** | 취소 = 레코드 삭제 (PRD §4.5.6 명시). 미등원 = `status='makeup_absent'` 기존 2상태로 처리 가능 |
| V108: `schedule_codes.is_makeup_off` 컬럼 | **불요** | 기존 `allows_makeup_class` 필드가 동일 역할 수행. `allows_makeup_class = 0` 이면 보강 OFF |
| V108: 기타 | **T1에서 최종 판단** | 기존 V106/V107 스키마로 전체 흐름 구현 가능한지 scope.md에 검증 결과 기록 |

**현재 판단: 신규 마이그레이션 불필요** -- V106(출결 테이블) + V107(FK 보강)으로 보강 전체 흐름 커버 가능.

---

## 다음 단계

> `/sprint-dev 9` 커맨드로 구현 단계에 진입하세요.

Sprint 9 완료 후:
1. `sprint-close` 에이전트: 문서화 + develop 머지
2. `sprint-review` 에이전트: 코드 리뷰 + 자동 검증 + 회고 작성 (산출물 4종)
3. Sprint 10 진입: 소멸 자동 전이 + 퇴교 보강 처리 + 캘린더 뷰
