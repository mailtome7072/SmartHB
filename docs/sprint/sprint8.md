# Sprint Plan sprint8

> **완료**: 2026-05-24 / develop 머지 완료 (sprint8 → develop, --no-ff)

## 기간
2026-05-23 ~ 2026-05-24 (9 세션)

## 목표
Phase 2의 마지막 마일스톤인 출결 관리(PRD §4.5)를 완성하여, 원장이 월별 출결 생성 -> 출결표 확인 -> 출석/결석 토글 -> 보강필요시간 실시간 추적의 핵심 흐름(UC-3)을 수행할 수 있게 한다. 동시에 Sprint 7에서 이월된 Keychain/auth 보안 carry-over High 4건 + Medium-High 1건 + Medium 4건을 흡수하여 Phase 2를 기술 부채 없이 마감한다.

## ROADMAP 연계 기능
- Phase 2: 학사 + 출결 (Sprint 6~8) -- Sprint 8은 Phase 2 최종 마일스톤
- §4.5.1 월별 출결 생성
- §4.5.2 출결 상태값 정의 (정규 4종 + 보강 2종)
- §4.5.3 출결표 UI 및 상태 토글
- §6.2 정규 출결 `(원생, 일자)` UNIQUE / 보강 출결 중복 허용
- Sprint 7 carry-over I-S2-2~10 흡수 (R40~R48 리스크 해소)

## Phase 위치
```
Phase 2 (학사 + 출결)
  Sprint 6: 학사 스케줄 .................. [v] 완료
  Sprint 7: carry-over 해소 .............. [v] 완료
  Sprint 8: 출결 관리 .................... [현재] <-- Phase 2 마감
```

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint7-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 방법 |
|---------|------|----------------------|
| A28 | `create_study_period` overlap 검사에 `AND is_confirmed = 1` 추가 (R39) | **T8에서 처리** -- carry-over Medium 묶음으로 overlap 검사 보정. 단일 원자 IPC보다 조건 추가가 최소 변경 |
| A29 | R40~R43 High 4건 우선 처리 (is_salt_corrupted / set_password 재진입 / shutdown cache 무효화 / check_auth_status 마이그레이션) | **T6에서 처리** -- carry-over High 통합 태스크로 4건 일괄 처리 |
| A30 | R44 테스트 격리: `load_salt_backs_up_corrupted_file`에 `#[ignore]` 마킹 | **완료 (Hotfix v0.3.2에서 처리됨)** -- I-S2-6은 hotfix로 선행 해소 |
| A31 | lock 테스트 격리 강화 (temp dir 기반) | **T8에서 처리** -- carry-over Medium 묶음 |
| A32 | R46 Mutex poison 방지: `lock().ok()` 패턴 | **T8에서 처리** -- carry-over Medium 묶음 |
| A33 | R47 migrate audit 누락 | **T8에서 처리** -- carry-over Medium 묶음 |
| A34 | A24 이연: `.claude/skills/` 파일 추가 | **범위 외** -- spare-time 작업, Sprint scope 아님 |
| A35 | 2027년 공휴일 V401 | **범위 외** -- 2027년 1월 시점 작업 |
| A36 | R50 NEXT_PUBLIC_ 주석 정정 | **완료 (Hotfix v0.3.2에서 처리됨)** |
| A37 | R51 배지 클릭 UX -- 사용자 피드백 후 결정 | **T8에서 처리** -- carry-over Medium 묶음으로 조건부 비활성화 구현 |
| A38 | 시각 검증을 sprint 계획에 명시 예약 | **반영** -- T9(통합 검증)에 "사용자 시각 검증 세션 (1시간)" 명시 포함 |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-05-22.md`

| 리스크 ID | 항목 | 이번 스프린트 반영 |
|-----------|------|------------------|
| R39 | createStudyPeriod + confirmStudyPeriod 원자성 결여 | **T8에서 해소** -- overlap 검사 `AND is_confirmed = 1` 조건 추가 |
| R40 | is_salt_corrupted partial-NULL 미감지 | **T6에서 해소** -- 검사 로직 강화 |
| R41 | set_password 재진입 가드 없음 | **T6에서 해소** -- Mutex guard 추가 |
| R42 | CRED_CACHE static Drop 미보장 | **T6에서 해소** -- shutdown hook + 주석 정정 |
| R43 | check_auth_status 마이그레이션 미트리거 | **T6에서 해소** -- legacy fallback 재검증 + 수정 |
| R44 | 테스트가 실제 Keychain 삭제 가능 | **완료** -- Hotfix v0.3.2에서 `#[ignore]` 마킹 처리됨 |
| R45 | tokio::join! concurrent verify_password race | **T7에서 해소** -- startup sequence 순서 보장 |
| R46 | Mutex poison 영구 brick | **T8에서 해소** -- `lock().ok()` 패턴 |
| R47 | migrate audit 누락 | **T8에서 해소** -- audit 기록 추가 |
| R48 | Low 잡다 이슈 묶음 | **T8에서 해소** -- 적용 가능 항목 선별 처리 |
| R49 | CalendarCell 한국어 리터럴 잔존 | **범위 외** -- 기능 영향 없음, 후속 스프린트 |
| R50 | NEXT_PUBLIC_ 주석 오류 | **완료** -- Hotfix v0.3.2 처리 |
| R51 | selection 모드 중 배지 클릭 삭제 다이얼로그 | **T8에서 해소** -- 조건부 비활성화 |
| R52 | V302 UPDATE 시드 마킹 범위 | **범위 외** -- pre-release 영향 없음, 문서화만 |

---

## 작업 목록

### T1: DB 마이그레이션 V106 -- 정규 출결 + 보강 출결 테이블
> **배경**: 출결 도메인의 데이터 기반. PRD §4.5.2의 정규 출결 4상태 + 보강 출결 2상태를 지원하는 스키마 설계. 마이그레이션 번호는 도메인 블록 V101~V199 내 다음 번호(V106).

**DB 스키마**:

```sql
-- V106__create_attendance_tables.sql

-- 정규 출결
CREATE TABLE IF NOT EXISTS regular_attendances (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id  INTEGER NOT NULL REFERENCES students(id),
    event_date  TEXT    NOT NULL,  -- YYYY-MM-DD
    year_month  TEXT    NOT NULL,  -- YYYY-MM (출결 생성 월 기준)
    status      TEXT    NOT NULL DEFAULT 'present'
                CHECK (status IN ('present','absent','makeup_done','makeup_expired')),
    class_minutes INTEGER NOT NULL,  -- 해당 수업 시간(분)
    absence_memo TEXT,               -- 결석 사유 메모 (선택)
    makeup_deadline TEXT,            -- YYYY-MM (결석 발생 월+1, 소멸기한)
    makeup_attendance_id INTEGER REFERENCES makeup_attendances(id),  -- 보강완료 시 연결
    created_at  TEXT    NOT NULL DEFAULT (datetime('now','localtime')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now','localtime')),
    UNIQUE(student_id, event_date)   -- PRD §6.2
);

-- 보강 출결
CREATE TABLE IF NOT EXISTS makeup_attendances (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id  INTEGER NOT NULL REFERENCES students(id),
    event_date  TEXT    NOT NULL,  -- YYYY-MM-DD (보강 진행일)
    year_month  TEXT    NOT NULL,  -- YYYY-MM (보강 등록 월)
    status      TEXT    NOT NULL DEFAULT 'makeup_attended'
                CHECK (status IN ('makeup_attended','makeup_absent')),
    class_minutes INTEGER NOT NULL,  -- 보강 수업 시간(분)
    created_at  TEXT    NOT NULL DEFAULT (datetime('now','localtime')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now','localtime'))
    -- 보강 출결은 같은 일자 중복 허용 (PRD §6.2) -- UNIQUE 없음
);

CREATE INDEX idx_regular_att_student ON regular_attendances(student_id);
CREATE INDEX idx_regular_att_yearmonth ON regular_attendances(year_month);
CREATE INDEX idx_regular_att_date ON regular_attendances(event_date);
CREATE INDEX idx_makeup_att_student ON makeup_attendances(student_id);
CREATE INDEX idx_makeup_att_yearmonth ON makeup_attendances(year_month);
```

**백엔드**:
- `src-tauri/migrations/106__create_attendance_tables.sql` (신규)
- `sqlx migrate run` + `.sqlx/` 오프라인 캐시 갱신

**예상 변경 파일**: `migrations/106__create_attendance_tables.sql` (1파일)
**예상 소요**: 2시간
**AC (Acceptance Criteria)**:
- ✅ AC-T1-1: `sqlx migrate run` 성공 + 테이블 2개 생성 확인 (`v106_creates_attendance_tables` 단위 테스트)
- ✅ AC-T1-2: `regular_attendances`에 `(student_id, event_date)` UNIQUE 제약 동작 확인 (`regular_attendances_unique_student_date`)
- ✅ AC-T1-3: `makeup_attendances`에 UNIQUE 없이 동일 student_id + event_date 다중 INSERT 가능 확인 (`makeup_attendances_allows_multiple_same_date`)
- ✅ AC-T1-4: status CHECK 제약 위반 시 INSERT 실패 확인 (`attendances_status_check_rejects_invalid`)
- ✅ AC-T1-5: `.sqlx/` 오프라인 캐시 — T2 에서 raw `sqlx::query` 만 사용 (매크로 미사용) 으로 캐시 갱신 불필요. CI `SQLX_OFFLINE=true` 영향 없음

---

### T2: 출결 생성 IPC 구현 (§4.5.1)
> **배경**: "출결생성" 버튼 클릭 시 해당 월 재원 원생 x 수업 요일 일자에 정규 출결 데이터를 일괄 생성하는 핵심 로직. 교습기간 내 수업일만, 입교일/퇴교일 범위 내만, "정규수업 진행 OFF" 일자 건너뜀, 중복 생성 방지.

**백엔드**:
- `src-tauri/src/commands/attendance.rs` (신규 모듈):
  - `generate_attendances(year_month: String) -> Result<GenerateResult, String>`:
    1. 해당 월 확정 교습기간 조회 -> 시작/종료일 범위 확정
    2. 재원 원생(withdraw_date IS NULL OR withdraw_date >= 교습기간 시작일) 목록 조회
    3. 각 원생의 수업 스케줄(요일별) 조회
    4. 교습기간 내 각 원생 수업 요일 일자 계산
    5. "정규수업 진행 OFF" 학사일정 일자 필터링 (schedule_codes.is_class_off = 1)
    6. 원생 입교일 이전 / 퇴교일 이후 일자 제외
    7. 중복 검사: 이미 해당 월 출결 존재 시 에러 반환 (AC-4.5-1)
    8. 일괄 INSERT (BEGIN IMMEDIATE 트랜잭션)
    9. 반환: 생성된 출결 건수, 원생 수
  - `check_attendance_exists(year_month: String) -> Result<bool, String>`: 중복 생성 방지 사전 검사
- `src-tauri/src/commands/mod.rs`: `pub mod attendance;` 등록
- `src-tauri/src/lib.rs`: `invoke_handler`에 커맨드 등록

**예상 변경 파일**: `attendance.rs` (신규), `mod.rs`, `lib.rs` (3파일)
**예상 소요**: 6시간
**AC (Acceptance Criteria)**:
- ✅ AC-T2-1: 월 선택 -> `generate_attendances` 호출 -> 재원 원생 x 수업 요일 일자에 출결 레코드 생성 확인 (`generate_creates_attendances_for_active_students`)
- ✅ AC-T2-2: "정규수업 진행 OFF" 일자 건너뜀 확인 (공휴일/휴원일/방학 등) (`generate_skips_off_days`)
- ✅ AC-T2-3: 입교일 이전, 퇴교일 이후 일자 제외 확인 (`generate_respects_enroll_withdraw_range`)
- ✅ AC-T2-4: 동일 월 재실행 시 에러 반환 (AC-4.5-1 중복 방지) (`generate_blocks_duplicate_month`)
- ✅ AC-T2-5: 교습기간 미확정 월에서는 "교습기간을 먼저 확정하세요" 에러 반환 (`generate_requires_confirmed_period`)
- ✅ AC-T2-6: 생성된 출결의 class_minutes가 해당 원생 수업 스케줄의 시간(분)과 일치 (`class_minutes_matches_schedule_hours`)
- ✅ AC-T2-7: 단위 테스트: 인메모리 DB에서 정상 생성 / 중복 차단 / OFF 일자 필터 / 입퇴교 범위 필터 (cipher off 221 passed)

---

### T3: 출결 조회 + 상태 토글 IPC (§4.5.3)
> **배경**: 출결표 UI에 필요한 그리드 데이터 조회 + 셀 클릭 시 출석/결석 토글. 토글 시 보강필요시간 자동 계산, 소멸기한 자동 설정/해제.

**백엔드**:
- `src-tauri/src/commands/attendance.rs` (T2 모듈에 추가):
  - `get_attendance_grid(year_month: String) -> Result<AttendanceGrid, String>`:
    - 행: 재원 원생 목록 (이름, student_id, 수업 요일 정보)
    - 열: 해당 월 일자별 출결 상태
    - 요약 컬럼: 출석일수, 결석일수, 보강필요시간(분), 보강시간(분)
    - 50명 x 31일 쿼리 최적화: 단일 쿼리로 해당 월 전체 데이터 조회 후 프론트에서 그리드 구성
  - `toggle_attendance(attendance_id: i64, new_status: String) -> Result<ToggleResult, String>`:
    - 출석 -> 결석: 보강필요시간 += class_minutes, 소멸기한 = 결석 발생 월 + 1
    - 결석 -> 출석: 보강필요시간 -= class_minutes, 소멸기한 NULL로 초기화
    - 보강완료/보강소멸 상태에서 토글 시도: 차단 + 에러 메시지
    - audit 로그 기록
  - `update_absence_memo(attendance_id: i64, memo: Option<String>) -> Result<(), String>`:
    - 결석 셀 사유 메모 업데이트
  - `get_attendance_summary(student_id: i64, year_month: String) -> Result<AttendanceSummary, String>`:
    - 원생별 월간 요약: 출석일수, 결석일수, 보강필요시간, 보강시간

**예상 변경 파일**: `attendance.rs`, `lib.rs` (2파일)
**예상 소요**: 6시간
**AC (Acceptance Criteria)**:
- ✅ AC-T3-1: `get_attendance_grid` 응답에 원생별 일자별 출결 상태 + 요약 포함 (`get_attendance_grid_returns_full_structure`)
- ✅ AC-T3-2: 출석 -> 결석 토글 시 보강필요시간이 class_minutes만큼 증가 (`toggle_present_to_absent_increases_makeup_needed`)
- ✅ AC-T3-3: 결석 -> 출석 토글 시 보강필요시간이 class_minutes만큼 감소 (`toggle_absent_to_present_decreases_makeup_needed`)
- ✅ AC-T3-4: 결석 토글 시 소멸기한 = 결석 발생 월 + 1 자동 설정 (`toggle_to_absent_sets_deadline_next_month` — 5월/12월/1월 분기 포함)
- ✅ AC-T3-5: 보강완료/보강소멸 상태에서 토글 차단 (`toggle_blocked_for_makeup_done_and_expired`)
- ✅ AC-T3-6: 결석 사유 메모 업데이트 동작 (`update_absence_memo_writes_text_and_nulls`)
- ✅ AC-T3-7: 단위 테스트: 토글 정합성, 보강필요시간 계산, 소멸기한 설정/해제, 상태별 토글 차단 (attendance 모듈 22 passed)

---

### T4: 출결표 프론트엔드 UI (§4.5.3) · skill: frontend-design
> **배경**: 행(원생) x 열(일자) 그리드 UI. 50명 x 31일 1초 이내 렌더링. 셀 클릭으로 출석/결석 토글, 결석 셀 빨간색, 요약 컬럼 실시간 업데이트. TanStack Query 캐싱 + 낙관적 업데이트.

**프론트엔드**:
- `src/app/attendance/page.tsx` (신규): 출결 관리 메인 페이지
  - 월 선택 콤보박스 (year_month)
  - "출결 생성" 버튼 (해당 월 미생성 시만 활성)
  - 출결 그리드 컴포넌트 렌더링
- `src/components/attendance/AttendanceGrid.tsx` (신규): 그리드 본체
  - 행: 원생 이름 (좌측 고정 컬럼)
  - 열: 일자별 셀 (수업 요일 아닌 일자는 회색/비활성)
  - 셀 클릭: 출석 <-> 결석 토글 (낙관적 업데이트)
  - 결석 셀: 빨간색 배경/폰트
  - 보강완료 셀: 빨간색 + 보강일자 표시
  - 보강소멸 셀: 회색 + "소멸" 표시
  - 요약 컬럼 (우측): 출석일수 / 결석일수 / 보강필요시간 / 보강시간
  - 1단계 Undo 지원 (최근 토글 1건 Ctrl+Z로 복원)
- `src/components/attendance/AbsenceMemoDialog.tsx` (신규): 결석 사유 메모 입력 다이얼로그
  - 결석 셀 우클릭 또는 아이콘 클릭으로 진입
- `src/types/attendance.ts` (신규): 출결 도메인 타입 정의
  - `RegularAttendance`, `MakeupAttendance`, `AttendanceGrid`, `AttendanceSummary`, `ToggleResult`
- `src/lib/tauri/index.ts`: IPC 래퍼 추가 (T2/T3 커맨드 대응)
- 사이드바: "출결 관리" 메뉴 추가

**성능 고려사항**:
- 50명 x 31일 = 1,550 셀. 가상 스크롤 없이 native React 렌더링으로 충분 (React 19 + 메모이제이션)
- 셀 컴포넌트 `React.memo()` 적용 -> 토글 시 해당 행+요약만 re-render
- TanStack Query `useMutation` + 낙관적 업데이트로 UI 즉시 반영

**예상 변경 파일**: `attendance/page.tsx` (신규), `AttendanceGrid.tsx` (신규), `AbsenceMemoDialog.tsx` (신규), `attendance.ts` (신규), `src/lib/tauri/index.ts`, 사이드바 컴포넌트 (6파일)
**예상 소요**: 8시간
**AC (Acceptance Criteria)**:
- ✅ AC-T4-1: `/attendance` 페이지에서 월 선택 후 출결 그리드 렌더링
- ✅ AC-T4-2: "출결 생성" 버튼으로 해당 월 출결 일괄 생성 -> 그리드 표시
- ✅ AC-T4-3: 셀 클릭으로 출석 <-> 결석 토글 동작 (mutation 후 invalidate — 단순성 우선)
- ✅ AC-T4-4: 결석 셀 빨간색 표시, 보강완료 셀에 보강일자, 보강소멸 셀 회색
- ✅ AC-T4-5: 요약 컬럼(출석/결석/보강필요시간/보강시간) 토글 시 실시간 업데이트
- ✅ AC-T4-6: 결석 사유 메모 입력 다이얼로그 동작 (`AbsenceMemoDialog`)
- ✅ AC-T4-7: 1단계 Undo (Ctrl+Z / Cmd+Z) 동작
- ⬜ AC-T4-8: 50명 x 31일 렌더링 1초 이내 (사용자 시각 검증 — `pnpm tauri:dev` Chrome DevTools Performance)
- ✅ AC-T4-9: Pretendard 18pt, WCAG AA 명도 대비, 44x44px 클릭 영역 준수
- ✅ AC-T4-10: `pnpm tsc --noEmit` + `pnpm lint` 통과 (T9 자동 검증)

---

### T5: 보강필요시간 계산 + 소멸기한 로직 (§4.5.7) 단위 테스트 100%
> **배경**: 출결 토글의 핵심 비즈니스 규칙. 보강필요시간 계산 공식과 소멸기한 설정/해제가 모든 시나리오에서 정확히 동작해야 한다. PRD §6.5에서 비즈니스 규칙 100% 테스트 커버를 요구.

**백엔드**:
- `src-tauri/src/commands/attendance.rs`: 비즈니스 로직 모듈화
  - `calculate_makeup_needed(student_id, year_month)`: 보강필요시간 = SUM(class_minutes WHERE status='absent')
  - `set_makeup_deadline(attendance_id)`: 소멸기한 = 결석 발생 월 + 1 (YYYY-MM 형식)
  - `clear_makeup_deadline(attendance_id)`: 출석 환원 시 소멸기한 NULL
- `#[cfg(test)]` 블록:
  - 테스트 시나리오 목록:
    1. 결석 1건 -> 보강필요시간 = class_minutes
    2. 결석 2건 -> 보강필요시간 = 합산
    3. 결석 -> 출석 환원 -> 보강필요시간 감소
    4. 보강완료 상태 -> 보강필요시간에서 제외
    5. 보강소멸 상태 -> 보강필요시간에서 제외
    6. 소멸기한 설정: 5월 결석 -> 소멸기한 2026-06
    7. 소멸기한 설정: 12월 결석 -> 소멸기한 다음해 01
    8. 출석 환원 시 소멸기한 NULL
    9. 동일 월 다중 결석 -> 각각 독립 소멸기한
    10. class_minutes 0인 엣지 케이스

**예상 변경 파일**: `attendance.rs` (T2/T3에서 생성된 파일) (1파일)
**예상 소요**: 4시간
**AC (Acceptance Criteria)**:
- ✅ AC-T5-1: 보강필요시간 계산 공식 = SUM(class_minutes WHERE status='absent' AND makeup_attendance_id IS NULL) — 보강완료/보강소멸/매칭 모두 제외
- ✅ AC-T5-2: 소멸기한 = 결석 발생 월 + 1 (5월→6월, 12월→다음해 1월, 다중 결석 독립 deadline)
- ✅ AC-T5-3: 10개 시나리오 전수 단위 테스트 통과 (T3 6건 + T5 신규 4건)
- ✅ AC-T5-4: 보강필요시간 계산 100% 단위 테스트 커버 (PRD §6.5)

---

### T6: Sprint 7 carry-over High 4건 통합 처리 (I-S2-2/3/4/5) · skill: systematic-debugging
> **배경**: Sprint 7 Session #2 code review에서 발견된 High 등급 보안/안정성 이슈 4건. R40~R43. Keychain/auth 보안 경로의 구조적 취약점.

**백엔드**:
- `src-tauri/src/commands/auth.rs`:
  - **I-S2-2 (R40)**: `is_salt_corrupted` 강화 -- ALL-zero 외 추가 조건: length != 32, first 8바이트 동일 패턴 반복 감지
  - **I-S2-3 (R41)**: `set_password` 재진입 가드 -- 함수 진입 시 Mutex flag 확인, 이미 진행 중이면 에러 반환
  - **I-S2-4 (R42)**: `CRED_CACHE` 주석 정정 ("프로세스 종료 시 자동 폐기" -> "명시적 무효화 필요") + `invalidate_credential_cache()` pub 함수 추가 + Tauri `on_exit` hook에서 호출 등록
  - **I-S2-5 (R43)**: `check_auth_status`에서 legacy keyring salt 잔존 시 마이그레이션 트리거 추가 -- `salt_exists_at` 경로에 legacy keyring fallback이 동작하도록 순서 검증 + 필요 시 수정

**예상 변경 파일**: `auth.rs`, `lib.rs` (on_exit 등록) (2파일)
**예상 소요**: 5시간
**AC (Acceptance Criteria)**:
- ✅ AC-T6-1: `is_salt_corrupted`: 32바이트 all-zero 감지 + length 불일치 감지 + partial-NULL 패턴 감지 (`is_salt_corrupted_detects_partial_null_patterns`)
- ✅ AC-T6-2: `set_password` 동시 호출 시 두 번째 호출이 에러 반환 (재진입 차단) — `SetPasswordGuard` RAII 패턴, `set_password_guard_blocks_concurrent_entry`/`releases_on_drop`/`releases_on_panic_unwind`
- ✅ AC-T6-3: `CRED_CACHE` 주석에 "명시적 invalidate" 명시 + `invalidate_credential_cache()` `pub` 함수 노출 (cross-module 호출용)
- ✅ AC-T6-4: 앱 종료 시 `invalidate_credential_cache()` 호출 확인 (startup::exit_hook L229)
- ✅ AC-T6-5: `check_auth_status` 캐시 적중 시 즉시 Locked 반환 (`check_auth_status_returns_locked_on_cache_hit`), legacy keyring fallback 단위 테스트 `#[ignore]` (OS 의존)
- ✅ AC-T6-6: 기존 단위 테스트 전체 통과 + 각 이슈별 새 테스트 추가 (T6 +5건)

---

### T7: Sprint 7 carry-over Medium-High 1건 (I-S2-7, R45) · skill: systematic-debugging
> **배경**: `tokio::join!`으로 concurrent하게 `verify_password`와 `get_cached_or_load_key`가 실행될 때, 두 함수가 모두 Keychain에 접근하면 macOS에서 두 개의 Keychain 다이얼로그가 동시 표시될 수 있음. AC-T1-1(다이얼로그 최대 1회) 잠재 위반.

**백엔드**:
- `src-tauri/src/commands/auth.rs` 또는 `db.rs`:
  - startup sequence 확인: `verify_password` 완료 -> `retrieve_key_from_keyring` 순서가 보장되는지 점검
  - `tokio::join!` 사용 지점에서 Keychain 접근 함수가 병렬 실행되지 않도록 순차 호출로 변경 (필요 시)
  - 또는 T1(Sprint 7)의 `CredentialCache`가 이미 이 문제를 해소했는지 검증 -- 캐시 적중 시 Keychain 직접 호출 0회이므로 race 불가

**예상 변경 파일**: `auth.rs` 또는 `db.rs` (1~2파일)
**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- ✅ AC-T7-1: startup sequence에서 Keychain 다이얼로그 최대 1회 — `ensure_cache_loaded` LOAD_MUTEX 직렬화 + double-check 로 첫 진입자 1회만 keyring 호출 보장
- ✅ AC-T7-2: `tokio::join!` 지점에서 Keychain 직접 접근 함수가 병렬 실행되지 않음 — `ensure_cache_loaded_fast_path_is_concurrent_safe` (16 스레드) 통과, slow-path 직렬화 검증 테스트는 `#[ignore]` (OS 의존)
- ✅ AC-T7-3: 캐시 적중 경로에서 Keychain 호출 0회 확인 — Fast path `cred_cache_lock()` 만 잡고 즉시 반환

---

### T8: Sprint 7 carry-over Medium + R51/R52 (I-S2-8/9/10, R39, A31) 통합 처리
> **배경**: Medium 등급 carry-over 잔여 처리 + sprint7 코드 리뷰 잔여 이슈. 개별 영향도는 중간이나 누적되면 기술 부채.

**백엔드**:
- **I-S2-8 (R46)**: `auth.rs` -- `Mutex::lock().expect("cred_cache poisoned")` -> `lock().unwrap_or_else(|e| e.into_inner())` 패턴으로 poison 복구
- **I-S2-9 (R47)**: `auth.rs` -- `migrate_keyring_salt_to`에 `try_record(AuditEventType::SecurityEvent, ...)` 추가
- **I-S2-10 (R48)**: 적용 가능 항목 선별:
  - `device.id` 권한 0o644 -> 0o600 (T3 후속)
  - `salt buffer` ZeroizeOnDrop 적용 (T2 후속)
  - stale doc comment 정리
- **R39 (A28)**: `academic.rs` -- `create_study_period` overlap 검사에 `AND is_confirmed = 1` 조건 추가
- **A31**: lock 테스트 `release_lock_atomic_removes_self_owned_lock` temp dir 기반 격리 검토 (flaky 해소)

**프론트엔드**:
- **R51 (A37)**: `academic/page.tsx` -- `studyPeriodMode` 활성 중 `calendarEventClick` 조건부 비활성화 (selection 모드에서 배지 삭제 다이얼로그 방지)

**예상 변경 파일**: `auth.rs`, `lock.rs`, `academic.rs`, `academic/page.tsx` (4파일)
**예상 소요**: 4시간
**AC (Acceptance Criteria)**:
- ✅ AC-T8-1: `cred_cache` Mutex poison 시 앱 crash 대신 graceful 복구 — `cred_cache_lock()` 헬퍼 + LOAD_MUTEX 인라인 (`unwrap_or_else(|e| e.into_inner())`)
- ✅ AC-T8-2: `migrate_keyring_salt_to` 실행 시 audit 로그에 SecurityEvent 기록 — tokio runtime 검출 후 fire-and-forget spawn
- ✅ AC-T8-3: `device.id` 파일 권한이 소유자 전용(0o600) — `device_id_file_has_owner_only_permissions` 단위 테스트
- ✅ AC-T8-4: `create_study_period` / `update_study_period` 미확정 교습기간이 존재해도 overlap 미차단 — overlap 쿼리 `AND is_confirmed = 1` + `overlap_skips_unconfirmed_periods` SQL 단위 테스트
- ✅ AC-T8-5: selection 모드 중 배지 클릭 시 삭제 다이얼로그 미표시 — `calendarEventClick` `if (studyPeriodMode) return`
- ✅ AC-T8-6: lock 테스트 flaky 검토 — 외부 점유 skip 가드 (`if acquired.is_err() { return; }`) 가 본 시나리오 차단. 추가 변경 불필요로 확인 (3회 연속 실행 통과)
- ✅ AC-T8-7: 기존 단위 테스트 전체 통과 — cipher off 221 / cipher on 133

---

### T9: 통합 검증
> 전체 변경사항 자동 검증 + 사용자 시각 검증 세션

**자동 검증**:
- `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (T1~T8 백엔드 변경 포함)
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과

**수동 검증 (사용자 시각 검증 세션 -- A38 반영, 1시간)**:
- T1: `sqlx migrate run` 후 테이블 생성 확인
- T2: 월 선택 -> "출결 생성" -> 출결 레코드 생성 확인
- T3: 출결표 그리드 렌더링 + 셀 클릭 토글 동작
- T4: 출결표 UI 전체 흐름 (출석/결석/요약 컬럼/Undo)
- T5: 보강필요시간이 결석 토글에 정확히 반응
- T6: Keychain 다이얼로그 1회 이하 + startup < 3초
- T7: startup 순서 정상 동작
- T8: 교습기간 미확정 overlap 해소 + selection 모드 배지 클릭 무동작
- UC-3: 월별 출결 생성 -> 출결표 확인 -> 토글 전체 흐름 완주

**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- ✅ AC-T9-1: 자동 검증 7항목 전수 통과 — cargo test cipher off **221 passed** / on **133 passed**, clippy off+on clean, pnpm lint/tsc/build clean (out/ 정상)
- ✅ AC-T9-2: 사용자 시각 검증 결과 "정상 동작 확인" (2026-05-24, T9 follow-up 3건 — sticky 4컬럼, 셀 너비 30% 감소, 원생 검색 필터 — 포함 전수 통과)
- ✅ AC-T9-3: 콘솔에 에러/경고 없음 (사용자 확인)
- ✅ AC-T9-4: UC-3(일일 출결 입력) 전체 흐름 완주 가능 (사용자 확인)

---

## Task 의존성 그래프

```
T1 (DB 마이그레이션) ── 최우선 (모든 출결 Task의 기반)
  |
T2 (출결 생성 IPC) ── T1 완료 필요
  |
T3 (출결 조회 + 토글 IPC) ── T1 완료 필요 (T2와 병행 가능하나 순차 권장)
  |
T4 (출결표 프론트엔드 UI) ── T2 + T3 완료 필요
  |
T5 (보강필요시간 단위 테스트) ── T3 완료 필요 (로직 확정 후 테스트 보강)

T6 (carry-over High 4건) ── 독립 (auth.rs 수정, 출결과 무관)
T7 (carry-over Medium-High) ── T6 완료 후 권장 (auth.rs 동일 파일)
T8 (carry-over Medium + R39/R51) ── T6 완료 후 권장

T9 (통합 검증) ── 모든 Task 완료 후 최종
```

**권장 실행 순서**: T1 -> T2 -> T3 -> T6 -> T7 -> T4 -> T5 -> T8 -> T9

> T6/T7을 T4 이전에 배치하는 이유: auth.rs 변경이 완료된 상태에서 프론트엔드 작업을 진행하면 startup 안정성이 보장된 환경에서 UI 개발이 가능하다.

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ DB 마이그레이션 V106 적용 + `.sqlx/` 캐시 갱신 커밋
- ⬜ 출결 생성 -> 출결표 표시 -> 출석/결석 토글 전체 흐름 동작
- ⬜ 50명 x 31일 렌더링 1초 이내
- ⬜ 보강필요시간 정확 계산 (결석 토글 시 +- 변경, 100% 단위 테스트 커버)
- ⬜ 소멸기한 자동 설정/해제 동작
- ⬜ 1단계 Undo (출결 토글) 동작
- ⬜ Sprint 7 carry-over High 4건 (R40~R43) 해소
- ⬜ Sprint 7 carry-over Medium-High 1건 (R45) 해소
- ⬜ Sprint 7 carry-over Medium 4건 (R46/R47/R39/R51) + R48 일부 해소
- ⬜ cargo test 전체 통과 (cipher off + on)
- ⬜ cargo clippy -- -D warnings 통과
- ⬜ pnpm build 성공 (Next.js static export)
- ⬜ pnpm lint + pnpm tsc --noEmit 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트 (Sprint 8 완료 + Phase 2 완료 반영)
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 신규 의존성

없음. 50명 x 31일 = 1,550 셀은 가상 스크롤 없이 React 19 native 렌더링 + `React.memo()` 메모이제이션으로 충분. 별도 virtualization 라이브러리 불필요.

---

## DB 마이그레이션

| 파일 | 내용 |
|------|------|
| `V106__create_attendance_tables.sql` | `regular_attendances` + `makeup_attendances` 테이블 생성 |

**번호 결정 근거**: V101~V105 (핵심 도메인 블록) 다음 번호. ROADMAP의 `V005` 표기는 구 번호 체계 — 실제 번호 정책(3자리 zero-pad, 도메인 블록 100단위)에 따라 V106 사용.

---

## Capacity 확인

- 팀: AI 페어 프로그래밍 1인 개발
- 스프린트 기간: 2주 (10 영업일)
- 실작업 가능 시간: 하루 4시간 = 총 40시간
- Task 수: 9개 (T9 통합 검증 포함)

| Task | 예상 소요 | 비고 |
|------|----------|------|
| T1 (DB 마이그레이션) | 2h | 스키마 설계 + 마이그레이션 적용 |
| T2 (출결 생성 IPC) | 6h | 핵심 비즈니스 로직 — 다중 조건 필터링 |
| T3 (출결 조회 + 토글 IPC) | 6h | 그리드 데이터 최적화 + 상태 전이 |
| T4 (출결표 프론트엔드 UI) | 8h | 그리드 UI + 낙관적 업데이트 + Undo |
| T5 (보강필요시간 테스트) | 4h | 10개 시나리오 전수 테스트 |
| T6 (carry-over High 4건) | 5h | 보안 로직 4건 통합 수정 |
| T7 (carry-over Medium-High) | 3h | startup 순서 검증/수정 |
| T8 (carry-over Medium) | 4h | 5건 통합 처리 |
| T9 (통합 검증) | 3h | 자동 + 사용자 시각 검증 |
| **합계** | **41h** | |

- Capacity: 40시간
- 예상 소요: 41시간
- 초과율: +2.5% (1시간 초과)
- **분석**: Sprint 6(45h, -12.5% 초과)과 Sprint 7(33h, +17.5% 여유) 사이의 중간 밀도. 출결 도메인(T1~T5, 26h)이 본 작업이고 carry-over 흡수(T6~T8, 12h)가 부가 작업. 1시간 초과는 T4(프론트엔드 UI)의 메모이제이션 최적화가 예상보다 빠르게 완료될 경우 자연 흡수 가능. 만약 지연 시 T8의 R48(Low 잡다 이슈) 일부를 후속 스프린트로 이연.
- **결론**: **적정** -- Sprint 7 대비 작업량은 증가했으나, 출결 도메인이 학사 스케줄(Sprint 6)보다 UI 상태 복잡도가 낮고 carry-over는 이미 원인 분석 완료 상태.

---

## 위험 및 대응

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R54 | 출결 생성 로직의 다중 조건 필터링(교습기간/수업요일/OFF일자/입퇴교) 조합에서 엣지 케이스 누락 -- 특히 월 중 입교(입교일이 월 중순)나 월 중 퇴교(퇴교일이 월 중순) 시나리오 | 중간 | T2에서 입교일/퇴교일 경계 조건을 명시적으로 테스트. 인메모리 DB 단위 테스트에 월 중 입교, 월 중 퇴교, 입교 전 퇴교(재원 기간 0일) 시나리오 포함 |
| R55 | 50명 x 31일 그리드 렌더링 성능 -- React.memo()만으로 토글 시 re-render 범위 제어가 불충분할 가능성 | 낮음 | 1,550 셀은 가상 스크롤 없이 처리 가능한 규모. 토글 시 해당 행 + 요약 컬럼만 re-render되도록 key 전략 적용. 만약 1초 초과 시 `react-window` 도입 (1시간 추가 작업) |
| R56 | carry-over T6의 `is_salt_corrupted` 강화가 기존 정상 salt를 오탐 -- partial-NULL 감지 조건이 과도하면 정상 salt를 손상으로 판정 | 중간 | 새 조건은 "length != 32 OR all-zero" 에서 시작하여 점진적으로 추가. first 8바이트 동일 패턴 반복은 실제 salt에서 극히 드문 경우이므로 오탐 위험 낮음. 기존 salt로 테스트 실행 후 적용 |
| R57 | T6(auth.rs)과 T2/T3(attendance.rs) 동시 수정 시 `lib.rs` invoke_handler 등록 충돌 -- merge 과정에서 커맨드 누락 | 낮음 | 권장 실행 순서(T1->T2->T3->T6)를 따르면 `lib.rs` 수정이 순차적으로 이루어져 충돌 없음 |

---

## Playwright MCP 검증 시나리오

```
1. browser_navigate -> http://localhost:1420/attendance
2. browser_snapshot -> 출결 관리 화면 (월 선택 + "출결 생성" 버튼)
3. browser_click -> 월 선택 콤보박스에서 현재 월 선택
4. browser_click -> "출결 생성" 버튼 클릭
5. browser_snapshot -> 출결표 그리드 렌더링 확인 (원생 x 일자)
6. browser_click -> 출석 셀 클릭 (결석 토글)
7. browser_snapshot -> 셀 빨간색 변경 + 요약 컬럼 보강필요시간 업데이트
8. browser_click -> 결석 셀 우클릭 -> 사유 메모 입력
9. browser_snapshot -> 메모 저장 확인
10. browser_console_messages(level: "error") -> 콘솔 에러 없음
```

---

## 참고 사항

- **PRD 확인**: §4.5(출결 관리 전체), §4.5.1~§4.5.10, §5.7(50명x31일 1초 이내), §6.2(UNIQUE 제약), §6.5(비즈니스 규칙 100% 테스트)
- **Phase 2 마감**: Sprint 8 완료 = Phase 2 완료. Phase 3(보강+소멸)은 Phase 2 완료 필수.
- **보강 매칭 로직**: Sprint 8에서는 출결 생성 + 상태 토글까지만 구현. 보강 등록(개별/일괄)은 Phase 3 Sprint 9 범위.
- **캘린더 뷰**: ROADMAP Sprint 8에 "캘린더 라이브러리 ADR" + "수업 관리 캘린더 뷰 기초"가 포함되어 있으나, 캘린더 뷰는 Phase 3 Sprint 10으로 이연. Sprint 8은 출결 그리드 UI + carry-over 흡수에 집중.
- **입교일/퇴교일 변경 시 출결 재조정 (§4.5.8)**: 복잡도 높은 기능. Sprint 8에서는 기본 출결 생성 시 입퇴교 범위 필터만 적용. 동적 재조정은 Phase 3 이후 검토.
- **Hotfix v0.3.2 선행 해소**: R50(NEXT_PUBLIC_ 주석), I-S2-6(Keychain 테스트 #[ignore])은 이미 처리됨.
- **A38 (시각 검증 패턴 표준화)**: T9에 "사용자 시각 검증 세션 (1시간)" 명시 예약.
- **R5 (보강 매칭 로직 복잡도)**: Sprint 9 착수 전 PI-02 사용자 결정 필요. Sprint 8에서는 아직 불필요.
