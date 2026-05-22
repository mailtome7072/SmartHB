# Sprint Plan sprint6

## 기간
2026-05-22 ~ 2026-06-05 (2주, 예상)

## 목표
Phase 2 학사 스케줄 관리 -- 3개월 캘린더 뷰, 교습기간 설정, 학사 일정 코드 3속성 모델, 학사 일정 배치, 단원평가 응시일 자동 배치를 구현하여 원장이 월별 학사 운영의 핵심 흐름(UC-2)을 수행할 수 있게 한다. 더불어 Sprint 5 코드 리뷰에서 발견된 A20(lock/page.tsx 재시도 버튼 누락)과 3번째 이월된 기술 부채(A14 flaky 테스트, A22 DnD sort_order)를 해소한다.

## ROADMAP 연계 기능
- Phase 2: 학사 + 출결 (Sprint 6~7)
- §4.4.1 3개월 캘린더 뷰
- §4.4.2 교습기간 설정
- §4.4.3~4.4.5 학사 일정 코드 3속성 모델
- §4.4.6 학사 일정 배치
- §4.4.7 단원평가 응시일 자동 배치
- §4.4.8~4.4.9 보강데이/공휴수업일 특수 처리
- V102 schedule_codes 시드 보정 (보강데이 `is_duplicate_blocked` 불일치)

> **DB 현황**: `study_periods`, `schedule_codes`, `schedule_events` 테이블은 Sprint 2(V102, V103)에서 이미 생성 완료. 이번 스프린트는 **백엔드 IPC + 프론트엔드 UI** 중심.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint5-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 방법 |
|---------|------|----------------------|
| A19 | DB 시드 변경 마이그레이션 작성 시 관련 테스트 동일 커밋에 업데이트 규칙 도입 | **T2에서 V301 시드 보정 시 테스트도 동시 작성**. 향후 규칙은 sprint-close 시 `backend.md` 테스트 섹션에 명문화 |
| A20 | lock/page.tsx 에러 화면 재시도 버튼 추가 + refresh 시 lockStatus null 초기화 | **T1에서 초반 처리** (Medium+Low 동시 해소) |
| A21 | A14 flaky 테스트 격리 강화 -- OnceLock 테스트별 분리 구조 리팩토링 | **T3에서 처리** (3번째 이월 -- 더 이상 미룰 수 없음) |
| A22 | A15 DnD 필터링 sort_order 충돌 해소 (R26) -- 방법 B 구현 | **T4에서 처리** (3번째 이월 -- codes 관련 spike) |
| A17 | salt.bin 이전 (Keychain -> cloud/smarthb/) | **범위 외** -- Phase 2 진입 후 사용자 데이터가 생기기 전 별도 hotfix로 처리 권고. 현재 사용자 데이터 없는 상태 유지 |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-05-22.md`

| 리스크 ID | 항목 | 이번 스프린트 반영 |
|-----------|------|------------------|
| R26 | DnD 필터링 sort_order 충돌 | **T4에서 해소** -- 3번째 이월, `handleDragEnd` 방법 B 구현 |
| R27 | tauri-plugin-single-instance Tauri 2.x 호환 | ✅ Sprint 5에서 해소됨 (정상 동작 확인) |
| R28 | V201 시드 교체 조건 감지 실패 | ✅ Sprint 5에서 해소됨 (방어적 마이그레이션 동작 확인) |
| R29 | cross-env NODE_OPTIONS 전달 실패 | ✅ Sprint 5에서 해소됨 (양 환경 검증 통과) |

---

## 작업 목록

### T1: lock/page.tsx 에러 화면 재시도 버튼 + 스테일 렌더링 해소 (A20)
> **배경**: Sprint 5 코드 리뷰 Medium + Low 이슈. `checkLockStatus()` IPC 실패 시 에러만 표시되고 재시도 수단 없음. `refresh()` 시 이전 `lockStatus`로 순간 잘못된 화면 렌더 가능.

**프론트엔드**:
- `src/app/lock/page.tsx`:
  - 에러 화면에 `<button onClick={refresh}>다시 시도</button>` 추가 (44x44px, Pretendard 스타일 준수)
  - `refresh()` 함수 시작 시 `setLockStatus(null)` 추가하여 SplashScreen 재표시

**예상 변경 파일**: `src/app/lock/page.tsx` (1파일)
**예상 소요**: 1시간
**AC (Acceptance Criteria)**:
- AC-T1-1: IPC 에러 발생 시 "다시 시도" 버튼이 표시되고, 클릭 시 `checkLockStatus()` 재호출
- AC-T1-2: `refresh()` 호출 시 SplashScreen이 먼저 표시된 후 결과에 따라 LockScreen/LockWarning 전환
- AC-T1-3: 정상 동작 경로(에러 없음)에 영향 없음 확인

---

### T2: V301 schedule_codes 시드 보정 + 공휴일 시드 (빌드 타임 스크립트)
> **배경**: V102 시드에서 보강데이의 `is_duplicate_blocked`가 `0`이나 PRD §4.4.4에서는 `ON`(1)이어야 한다. 공휴수업일의 `allows_makeup_class`도 `0`이나 PRD §4.4.4에서는 `ON`(1)이어야 한다. 또한 단원평가 응시일의 `is_duplicate_blocked`가 `1`이나 PRD에서는 `OFF`(0), `is_period_type`이 `0`이나 PRD에서는 기간성(5일)이므로 `1`이어야 한다.
> 한국 법정 공휴일 데이터는 **빌드 타임 시드 생성 스크립트**로 외부 API에서 1회 수집한 뒤 V301 마이그레이션 시드 SQL에 INSERT로 포함한다. **앱 런타임은 외부 네트워크 호출을 하지 않는다** (PRD §5.5 준수).

이 Task는 3개 서브파트로 구성된다:

#### T2-a: 빌드 도구 스크립트 작성
- **파일**: `scripts/fetch-holidays.ts` (신규)
  - `src-tauri/` 및 `src/` 본체 코드에는 두지 않음 (빌드 보조 도구)
  - 실행 방법: `pnpm holidays:fetch` (package.json scripts에 등록)
  - 외부 API에서 한국 공휴일 다년치(2024~2030, 7년분) 수집
  - 대체공휴일(어린이날, 추석, 설날 등) 포함 여부 확인 및 처리
  - 결과를 SQL INSERT 문으로 stdout 출력 또는 V301 마이그레이션 파일에 직접 기록
  - API 소스는 T2-c ADR 결정에 따름 (공공데이터포털 또는 date.nager.at)
- **package.json**: `"holidays:fetch": "npx tsx scripts/fetch-holidays.ts"` 스크립트 추가

#### T2-b: V301 마이그레이션 시드
- `src-tauri/migrations/301__fix_schedule_codes_seed.sql` (V301):
  - schedule_codes 시드 보정:
    - 보강데이: `is_duplicate_blocked` 0 -> 1
    - 공휴수업일: `allows_makeup_class` 0 -> 1
    - 단원평가 응시일: `is_duplicate_blocked` 1 -> 0, `is_period_type` 0 -> 1
    - 방어적 UPDATE (WHERE code_name + is_system_reserved = 1)
  - 공휴일 시드 INSERT (7년치, 2024~2030):
    - 저장 위치: `schedule_events` 테이블에 공휴일 코드(`schedule_codes`의 공휴일 code_id) 참조로 INSERT
    - 또는 별도 `holidays` 테이블 신설 -- PRD §4.4 / §6.2 재확인 후 T2-c ADR에서 결정
    - 방어적 INSERT (INSERT OR IGNORE -- 이미 존재하는 행 무시)
- V301 시드 보정 + 공휴일 시드 관련 테스트도 **동일 Task에서 작성** (A19 규칙 이행)

#### T2-c: API 선택 ADR 작성
- **파일**: `docs/arch/adr-NNN-holiday-api-selection.md` (신규)
- 비교 대상:
  - **공공데이터포털** (data.go.kr 특일 정보 API): 인증키 필요, 한국 공식 데이터, 대체공휴일 정확
  - **date.nager.at API**: 무인증, 외부 서버 의존, 한국 공휴일 정확도 미검증
- 결정 항목: API 소스, 공휴일 저장 테이블(schedule_events vs 별도 holidays), 갱신 주기(2030년 만료 전 재실행 필요)
- ADR 저장 위치: `docs/arch/adr-{NNN}-{주제}.md`
- **skill**: brainstorming

**예상 변경 파일**: `scripts/fetch-holidays.ts` (신규), `package.json` (스크립트 등록), `src-tauri/migrations/301__fix_schedule_codes_seed.sql` (신규), `docs/arch/adr-NNN-holiday-api-selection.md` (신규), 관련 테스트 (5~6파일)
**예상 소요**: 7시간 (T2-a 스크립트 3h + T2-b 시드/테스트 2.5h + T2-c ADR 1.5h)
**AC (Acceptance Criteria)**:
- AC-T2-1: `sqlx migrate run` 후 보강데이 `is_duplicate_blocked = 1`, 공휴수업일 `allows_makeup_class = 1`, 단원평가 `is_duplicate_blocked = 0, is_period_type = 1`
- AC-T2-2: 기존 사용자가 코드명을 변경한 DB에서는 UPDATE 미적용 (방어적)
- AC-T2-3: `pnpm holidays:fetch` 실행 시 2024~2030년 한국 공휴일 SQL INSERT 문 출력 (대체공휴일 포함)
- AC-T2-4: V301 시드에 7년치 공휴일 INSERT 포함, `sqlx migrate run` 후 DB에서 조회 확인
- AC-T2-5: `.sqlx/` 오프라인 캐시 갱신 + 커밋
- AC-T2-6: V301 시드 보정 + 공휴일 시드 관련 단위 테스트 동일 커밋에 포함
- AC-T2-7: ADR 문서 작성 완료 (API 소스 + 저장 테이블 + 갱신 주기 결정)
- AC-T2-8: 스크립트가 `src-tauri/` 또는 `src/` 본체에 위치하지 않음 확인

---

### T3: A14 flaky 테스트 격리 강화 -- OnceLock 리팩토링 (A21)
> **배경**: `paths::tests::init_from_config_ignores_empty_path` 테스트가 OnceLock 전역 상태로 인해 `--test-threads=1`에서만 안정 통과. 3번째 이월 -- 더 이상 미룰 수 없음.

**백엔드**:
- `src-tauri/src/commands/paths.rs`: OnceLock 기반 전역 상태를 테스트에서 격리 가능한 구조로 리팩토링
  - 방안 1: `#[cfg(test)]` 블록에서 OnceLock을 리셋 가능한 Mutex<Option<T>>로 교체
  - 방안 2: 의존성 주입 패턴으로 paths 모듈을 테스트 가능하게 변경
  - 최종 방안은 구현 시 코드 구조에 맞게 결정 (Negotiable)

**예상 변경 파일**: `src-tauri/src/commands/paths.rs` (1~2파일)
**예상 소요**: 2시간
**skill**: systematic-debugging
**AC (Acceptance Criteria)**:
- AC-T3-1: `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (`--test-threads` 제한 없이)
- AC-T3-2: `paths` 관련 테스트가 병렬 실행에서도 안정적으로 통과 (5회 연속 실행 확인)
- AC-T3-3: 프로덕션 코드 동작에 영향 없음

---

### T4: DnD 필터링 sort_order 충돌 해소 (A22, R26)
> **배경**: 코드 테이블 관리 화면에서 활성/비활성 필터링 적용 중 DnD 순서 변경 시 visibleCodes와 전체 codes의 sort_order가 불일치. 3번째 이월.

**프론트엔드**:
- `handleDragEnd`에서 visibleCodes 기준 재정렬 후 전체 codes 배열 재매핑 (방법 B)
- 대상 파일: codes 관련 컴포넌트 (코드 테이블 관리 화면)

**예상 변경 파일**: 코드 테이블 관련 프론트엔드 파일 (1~2파일)
**예상 소요**: 2시간
**AC (Acceptance Criteria)**:
- AC-T4-1: 활성/비활성 필터 적용 상태에서 DnD 순서 변경 시 전체 codes의 sort_order가 정확하게 반영
- AC-T4-2: 필터 해제 후 전체 목록에서 순서가 일관성 유지
- AC-T4-3: 기존 DnD 동작(필터 미적용 상태)에 영향 없음

---

### T5: 교습기간 CRUD IPC 커맨드
> **배경**: study_periods 테이블은 V102에서 생성 완료. 교습기간 생성/수정/조회/확정 IPC를 구현한다.

**백엔드**:
- `src-tauri/src/commands/academic.rs` (신규 모듈):
  - `create_study_period(year_month, start_date, end_date)` -- 교습기간 생성
    - 제약: start_date <= end_date, 이전 교습기간과 일자 중첩 불가 (PRD §6.2)
    - 월 수업일수 20일 충족을 위해 전/후월 일자 포함 가능
  - `update_study_period(id, start_date, end_date)` -- 교습기간 수정
    - AC-4.4-1: 지난 달은 수정 차단 (is_closed 또는 현재 월 이전)
  - `list_study_periods(from_month, to_month)` -- 교습기간 목록 조회 (범위)
  - `get_study_period(year_month)` -- 단일 교습기간 조회
  - `confirm_study_period(id)` -- 교습기간 확정
  - `delete_study_period(id)` -- 미확정 교습기간 삭제
- `src-tauri/src/commands/mod.rs` -- academic 모듈 등록
- `src-tauri/src/lib.rs` -- invoke_handler에 커맨드 등록

**예상 변경 파일**: `src-tauri/src/commands/academic.rs` (신규), `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` (3파일)
**예상 소요**: 4시간
**AC (Acceptance Criteria)**:
- AC-T5-1: 교습기간 생성 시 일자 중첩 검증 동작 (중첩 시 에러 반환)
- AC-T5-2: 지난 달 교습기간 수정/삭제 시도 시 차단 (AC-4.4-1)
- AC-T5-3: 교습기간 확정 후 is_confirmed = 1
- AC-T5-4: 단위 테스트: 중첩 검증, 지난 달 차단, CRUD 정상 동작

---

### T6: 학사 일정 코드 IPC 커맨드
> **배경**: schedule_codes 테이블은 V102에서 생성 완료. 학사 일정 코드 조회/생성/수정 IPC를 구현한다. 기존 `codes.rs`의 범용 CRUD와 분리하여 학사 코드 전용 로직(3속성 모델, 시스템 예약 잠금)을 구현한다.

**백엔드**:
- `src-tauri/src/commands/academic.rs`에 추가:
  - `list_schedule_codes()` -- 전체 학사 일정 코드 목록 (is_active 포함)
  - `create_schedule_code(code_name, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type)` -- 사용자 추가 코드 생성
    - 디폴트: 정규수업 OFF / 보강 OFF / 중복불가 ON (PRD §4.4.5 보수적 디폴트)
  - `update_schedule_code(id, ...)` -- 사용자 코드 3속성 편집
    - AC-4.4-5: is_system_reserved = 1인 코드는 is_active만 토글 가능
  - `toggle_schedule_code_active(id)` -- 활성/비활성 토글

**예상 변경 파일**: `src-tauri/src/commands/academic.rs` (기존 T5에서 생성), `src-tauri/src/lib.rs` (2파일)
**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- AC-T6-1: 시스템 예약 코드 5종의 3속성 수정 시도 시 차단 + 에러 반환
- AC-T6-2: 시스템 예약 코드의 활성/비활성 토글은 허용
- AC-T6-3: 사용자 추가 코드 생성 시 보수적 디폴트 적용 확인
- AC-T6-4: code_name UNIQUE 제약 위반 시 한국어 에러 메시지 반환
- AC-T6-5: 단위 테스트: 시스템 코드 보호, 사용자 코드 CRUD

---

### T7: 학사 일정 배치 IPC 커맨드
> **배경**: schedule_events 테이블은 V103에서 생성 완료. 학사 일정 등록/수정/삭제/조회 IPC를 구현한다.

**백엔드**:
- `src-tauri/src/commands/academic.rs`에 추가:
  - `create_schedule_event(code_id, event_date, period_end_date?, display_name?)` -- 일정 배치
    - 중복불가 코드 검증: 동일 일자에 is_duplicate_blocked = 1인 코드 배치 시도 차단 (AC-4.4-4)
    - 기간성 코드: period_end_date 필수, 단일 코드: period_end_date NULL
  - `update_schedule_event(id, event_date, period_end_date?, display_name?)` -- 일정 수정/이동
    - AC-4.4-1: 지난 달 일정 수정 차단
  - `delete_schedule_event(id)` -- 일정 삭제
    - AC-4.4-1: 지난 달 일정 삭제 차단
  - `list_schedule_events(from_date, to_date)` -- 기간 내 일정 목록 (코드 정보 JOIN, 공휴일 이벤트 포함)
  - `auto_place_assessment_dates(year_month)` -- 단원평가 응시일 자동 배치 (§4.4.7)
    - 교습기간 내 2주차 월~금(1차) + 4주차 월~금(2차) 자동 생성
    - 이미 해당 월에 수동 배치된 단원평가가 있으면 재실행 안 됨 (AC-4.4-6)
    - 자동 배치 후 원장이 드래그 이동/삭제/추가 가능

**예상 변경 파일**: `src-tauri/src/commands/academic.rs` (기존), `src-tauri/src/lib.rs` (2파일)
**예상 소요**: 5시간
**AC (Acceptance Criteria)**:
- AC-T7-1: 중복불가 코드 동일 일자 배치 시도 시 차단 + 경고 메시지
- AC-T7-2: 기간성 코드 배치 시 event_date ~ period_end_date 범위 일관성
- AC-T7-3: 지난 달 일정 수정/삭제 차단 확인
- AC-T7-4: 단원평가 자동 배치가 교습기간 내 2/4주차 월~금에 정확히 생성
- AC-T7-5: 이미 단원평가가 배치된 월에 재실행 시 중복 생성 없음 (AC-4.4-6)
- AC-T7-6: 단위 테스트: 중복불가 검증, 자동 배치, 지난 달 차단

---

### T8: 프론트엔드 IPC 래퍼 + 도메인 타입
> **배경**: T5~T7에서 구현한 IPC 커맨드의 프론트엔드 래퍼와 TypeScript 타입을 정의한다.

**프론트엔드**:
- `src/types/academic.ts` (신규): StudyPeriod, ScheduleCode, ScheduleEvent 타입 정의
- `src/lib/tauri/index.ts`: Sprint 6 래퍼 추가
  - createStudyPeriod, updateStudyPeriod, listStudyPeriods, getStudyPeriod, confirmStudyPeriod, deleteStudyPeriod
  - listScheduleCodes, createScheduleCode, updateScheduleCode, toggleScheduleCodeActive
  - createScheduleEvent, updateScheduleEvent, deleteScheduleEvent, listScheduleEvents, autoPlaceAssessmentDates

**예상 변경 파일**: `src/types/academic.ts` (신규), `src/lib/tauri/index.ts` (2파일)
**예상 소요**: 2시간
**AC (Acceptance Criteria)**:
- AC-T8-1: 모든 IPC 래퍼가 dev mode fallback 포함
- AC-T8-2: TypeScript strict 모드 통과 (`pnpm tsc --noEmit`)
- AC-T8-3: 타입이 백엔드 커맨드 시그니처와 1:1 대응

---

### T9: 3개월 캘린더 컴포넌트 (§4.4.1)
> **배경**: 좌(이전월)/중앙(기본=다음 달)/우(이후월) 3개 캘린더를 가로 배치하는 커스텀 컴포넌트. shadcn/ui Calendar 기반 커스터마이징.

**프론트엔드**:
- `src/app/academic/page.tsx` (신규 라우트): 학사 스케줄 관리 페이지
- `src/components/academic/ThreeMonthCalendar.tsx` (신규): 3개월 캘린더 뷰
  - 중앙 년월 좌/우 화살표로 월 이동
  - 법정 공휴일 자동 표시 (T2의 V301 시드로 DB에 삽입된 공휴일 데이터를 IPC 조회)
  - 공휴일 표시: 날짜 아래 작은 텍스트 또는 색상 마킹
  - 교습기간 셀: 파스텔 배경색
  - 학사 일정 셀: 일정명 배지 표시
  - 단원평가 응시일: 셀 상단 띠 배지 (§4.4.7)
  - 보강데이/공휴수업일: 구분 색상 배지
- `src/components/academic/CalendarCell.tsx` (신규): 셀 컴포넌트
  - 클릭 이벤트: 교습기간 설정 모드 / 일정 배치 모드에 따라 동작 분기
  - 지난 달 셀: 읽기 전용 표시 (AC-4.4-1)
- 사이드바 메뉴에 "학사 스케줄" 항목 추가

**예상 변경 파일**: `src/app/academic/page.tsx` (신규), `src/components/academic/ThreeMonthCalendar.tsx` (신규), `src/components/academic/CalendarCell.tsx` (신규), 사이드바 컴포넌트 (4~5파일)
**예상 소요**: 6시간
**skill**: frontend-design
**AC (Acceptance Criteria)**:
- AC-T9-1: 3개월 캘린더가 가로 배치로 렌더링 (반응형 -- 좁은 화면에서는 세로 스택)
- AC-T9-2: 중앙 년월 화살표로 월 이동 시 좌/우도 연동 이동
- AC-T9-3: 법정 공휴일이 해당 날짜에 표시
- AC-T9-4: 교습기간 셀에 파스텔 배경, 학사 일정 셀에 배지 표시
- AC-T9-5: 지난 달 셀 클릭 시 수정 동작 차단 (읽기 전용)
- AC-T9-6: Pretendard 18pt 기준, WCAG AA 명도 대비 준수

---

### T10: 교습기간 설정 UI (§4.4.2)
> **배경**: 캘린더에서 시작일/종료일 셀 클릭으로 교습기간을 설정하는 UI.

**프론트엔드**:
- `src/components/academic/StudyPeriodEditor.tsx` (신규):
  - "교습기간 설정" 모드 진입 버튼
  - 시작일 셀 클릭 -> 종료일 셀 클릭 -> "확정" 버튼으로 등록
  - 선택 중 범위 시각적 하이라이트 (파스텔 프리뷰)
  - 중첩 검증: 기존 교습기간과 겹치면 경고
  - 확정 후 IPC 호출 (createStudyPeriod -> confirmStudyPeriod)
  - 지난 달 교습기간: 읽기 전용 표시, 수정/삭제 버튼 비활성
- TanStack Query로 교습기간 데이터 캐싱

**예상 변경 파일**: `src/components/academic/StudyPeriodEditor.tsx` (신규), ThreeMonthCalendar 수정 (2파일)
**예상 소요**: 4시간
**AC (Acceptance Criteria)**:
- AC-T10-1: 시작일 -> 종료일 순차 클릭으로 교습기간 설정 동작
- AC-T10-2: 기존 교습기간과 일자 중첩 시 경고 다이얼로그
- AC-T10-3: 확정된 교습기간이 파스텔 배경으로 표시
- AC-T10-4: 지난 달 교습기간 수정/삭제 차단 확인

---

### T11: 학사 일정 코드 관리 + 일정 배치 UI (§4.4.3~4.4.6)
> **배경**: 학사 일정 코드(3속성) 관리 패널 + 캘린더 셀 클릭으로 일정 배치하는 UI.

**프론트엔드**:
- `src/components/academic/ScheduleCodePanel.tsx` (신규):
  - 코드 목록 표시 (시스템 5종 + 사용자 추가)
  - 시스템 코드: 3속성 잠금 표시, 활성/비활성 토글만 가능
  - 사용자 코드: 추가/편집/3속성 자유 변경
  - 코드 선택 시 "일정 배치 모드" 활성화
- `src/components/academic/EventPlacer.tsx` (신규):
  - 코드 선택 후 캘린더 셀 클릭 -> 일정 등록
  - 단일 일자 코드: 셀 1회 클릭으로 등록
  - 기간성 코드: 시작일 -> 종료일 클릭으로 범위 등록
  - 등록된 일정: 셀에 일정명 배지 표시
  - 단일 일자 일정 드래그 이동 (교습기간 내)
  - 중복불가 코드 경고 (AC-4.4-4)
  - 일정 삭제: 배지 클릭 -> 삭제 확인
- "단원평가 자동 배치" 버튼: autoPlaceAssessmentDates IPC 호출 (§4.4.7)

**예상 변경 파일**: `src/components/academic/ScheduleCodePanel.tsx` (신규), `src/components/academic/EventPlacer.tsx` (신규), ThreeMonthCalendar 수정 (3파일)
**예상 소요**: 6시간
**skill**: frontend-design
**AC (Acceptance Criteria)**:
- AC-T11-1: 시스템 코드 5종의 3속성이 잠금 표시되고 편집 불가
- AC-T11-2: 사용자 추가 코드 생성 시 보수적 디폴트(OFF/OFF/ON) 적용
- AC-T11-3: 코드 선택 -> 셀 클릭으로 단일 일자 일정 등록 동작
- AC-T11-4: 기간성 코드 시작/종료 선택으로 범위 일정 등록 동작
- AC-T11-5: 중복불가 코드 동일 일자 배치 시도 시 경고 후 차단
- AC-T11-6: 단원평가 자동 배치 버튼 클릭 시 2/4주차 월~금 배치 확인
- AC-T11-7: 지난 달 일정 수정/삭제 차단

---

### T12: 통합 검증
> 전체 변경사항 검증

- `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (`--test-threads` 제한 없이, T3 성공 전제)
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- `pnpm tauri:dev` 실행 후 전수 검증:
  - T1: lock/page.tsx 에러 화면 재시도 버튼 동작
  - T2: schedule_codes 시드 보정 + 공휴일 7년치 시드 확인 (V301)
  - T3: 전체 테스트 병렬 실행 안정성
  - T4: DnD 순서 변경 + 필터링 정합성
  - T5~T7: 교습기간/일정코드/일정배치 IPC 동작
  - T9: 3개월 캘린더 렌더링 + 공휴일 표시
  - T10: 교습기간 설정 + 확정 + 파스텔 배경
  - T11: 학사 일정 코드 관리 + 일정 배치 + 단원평가 자동 배치
- `.sqlx/` 오프라인 캐시 갱신 + 커밋

**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- AC-T12-1: 위 검증 항목 전수 통과
- AC-T12-2: 콘솔에 에러/경고 없음
- AC-T12-3: UC-2(월말 학사 일정 수립) 전체 흐름이 단일 세션으로 완주 가능

---

## Task 의존성 그래프

```
T1 (lock 재시도 버튼) ── 독립, 최우선 (A20 처리)
T2 (V301 시드 보정 + 공휴일 스크립트/ADR) ── 독립 (T9 렌더링 전 완료 필요)
T3 (flaky 테스트 격리) ── 독립 (A21 처리)
T4 (DnD sort_order 해소) ── 독립 (A22 처리)

T5 (교습기간 IPC) ── 독립 (백엔드)
T6 (일정 코드 IPC) ── 독립 (백엔드)
T7 (일정 배치 IPC) ── T5 + T6 완료 필요 (교습기간/코드 의존)
  |
T8 (IPC 래퍼 + 타입) ── T5 + T6 + T7 완료 후

T9 (3개월 캘린더) ── T2 완료 필요 (공휴일 데이터), T8 완료 필요 (IPC 래퍼) · skill: frontend-design
T10 (교습기간 설정 UI) ── T9 완료 필요 (캘린더 컴포넌트)
T11 (일정 코드 + 배치 UI) ── T9 + T10 완료 필요 · skill: frontend-design

T12 (통합 검증) ── 모든 Task 완료 후 최종
```

**권장 실행 순서**: T1 -> T2 -> T3 -> T4 -> T5 -> T6 -> T7 -> T8 -> T9 -> T10 -> T11 -> T12

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ cargo test 전체 통과 (`--test-threads` 제한 없이 병렬 실행 안정)
- ⬜ cargo clippy -- -D warnings 통과
- ⬜ pnpm build 성공 (Next.js static export)
- ⬜ pnpm lint + pnpm tsc --noEmit 통과
- ⬜ 3개월 캘린더에서 교습기간 설정/확정/읽기전용 동작
- ⬜ 학사 일정 코드 5종 + 사용자 추가 코드 배치 동작
- ⬜ 단원평가 응시일 자동 배치 + 수동 조정 동작
- ⬜ 지난 달 데이터 수정 차단 확인
- ⬜ lock/page.tsx 재시도 버튼 동작 확인 (A20)
- ⬜ 전체 테스트 병렬 실행 안정 (A21)
- ⬜ DnD 필터링 sort_order 정합성 확인 (A22)
- ⬜ .sqlx/ 오프라인 캐시 갱신 및 커밋

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트 (Sprint 6 완료 반영)
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 신규 의존성

> 3개월 캘린더는 shadcn/ui Calendar 기반 커스텀 구현. 공휴일 데이터는 빌드 타임 스크립트로 수집 후 시드 삽입.

| 패키지 | 구분 | 용도 | 사용자 허가 |
|--------|------|------|-----------|
| `tsx` | devDependencies (npm) | `scripts/fetch-holidays.ts` 실행 (`npx tsx`) | 불필요 (devDependency, 이미 프로젝트에 포함 가능) |

> `tsx`는 TypeScript 스크립트 직접 실행용. 이미 설치되어 있으면 추가 불필요. 없으면 `pnpm add -D tsx` 실행.

---

## DB 마이그레이션

| 번호 | 파일명 (권장) | 내용 |
|------|--------------|------|
| V301 | `301__fix_schedule_codes_seed.sql` | schedule_codes 시드 속성 보정 + 공휴일 7년치(2024~2030) 시드 INSERT (빌드 타임 스크립트 산출물) |

> Sprint 6 마이그레이션 예약 범위: V301~V399.
> 기존 study_periods, schedule_codes, schedule_events 테이블 스키마는 변경 없음 -- IPC 레벨 구현만.
> 공휴일 저장 위치(schedule_events vs 별도 holidays 테이블)는 T2-c ADR에서 결정. 별도 테이블 신설 시 V302가 추가될 수 있음.

---

## Capacity 확인

- 팀: AI 페어 프로그래밍 1인 개발
- 스프린트 기간: 2주 (10 영업일)
- 실작업 가능 시간: 하루 4시간 = 총 40시간
- Task 수: 12개 (T12 통합 검증 포함)
- 예상 소요: T1(1h) + T2(7h) + T3(2h) + T4(2h) + T5(4h) + T6(3h) + T7(5h) + T8(2h) + T9(6h) + T10(4h) + T11(6h) + T12(3h) = **45시간**
- 여유율: -12.5% (40h 대비 45h)
- 결론: **초과 상태** -- T2가 스크립트 작성(3h) + ADR(1.5h) 추가로 기존 대비 +3h 증가. 위험 시 T11의 드래그 이동 기능을 Sprint 7로 이연하여 ~3h 절감 가능. T1~T4(기술 부채+시드 12h)는 코드 변경량이 적어 예상보다 빠를 수 있음. T9/T11(프론트엔드)은 shadcn/ui 기반이라 생산성 높음.

---

## 위험 및 대응

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R30 | 3개월 캘린더 커스텀 구현 복잡도 -- shadcn/ui Calendar는 단일 월 위젯이므로 3개월 합성 + 교습기간 하이라이트 + 배지 오버레이가 예상보다 복잡할 수 있음 | 중간 | 최소 기능부터 점진 구현: 1) 기본 월 그리드 3개 배치, 2) 교습기간 하이라이트, 3) 배지 오버레이 순서. 복잡도 초과 시 드래그 이동을 Sprint 7로 이연 |
| R31 | 공휴일 API 의존성 -- 공공데이터포털(인증키 만료/API 변경) 또는 date.nager.at(서비스 중단/한국 데이터 부정확) 위험. 대체공휴일(어린이날, 추석, 설날 등) 누락 가능 | 낮음 | 빌드 타임 1회 수집이므로 런타임 영향 없음. 두 API 모두 실패 시 수동 SQL 작성 대안 보유. 수집 후 단위 테스트로 주요 공휴일(신정~성탄절 + 대체공휴일) 존재 검증. 2030년 만료 전 `pnpm holidays:fetch` 재실행으로 갱신 |
| R32 | OnceLock 리팩토링(T3)이 프로덕션 paths 동작에 영향 -- 전역 상태 관리 방식 변경 시 앱 시작 시퀀스에 부작용 발생 가능 | 중간 | 리팩토링 후 `app_startup_sequence` 통합 테스트 + `pnpm tauri:dev` 수동 검증 필수. 영향 범위가 크면 `#[cfg(test)]` 분리 방식(방안 1)으로 프로덕션 코드 변경 최소화 |

---

## 참고 사항

- **PRD 확인**: §4.4(학사 스케줄 관리), §4.4.1~4.4.9(세부 기능), §6.2(무결성 제약)
- **DB 현황**: V102(study_periods + schedule_codes), V103(schedule_events) 이미 생성 완료. Sprint 6은 IPC + UI 중심
- **마이그레이션 번호**: V201(Sprint 5) 사용 완료 -> V301부터 시작
- **캘린더 라이브러리 ADR**: ROADMAP.md에 Sprint 7 예정으로 명시. Sprint 6에서는 학사 스케줄용 커스텀 3개월 캘린더를 직접 구현(shadcn/ui 기반). 수업 관리 캘린더 뷰(§4.6)에서 사용할 FullCalendar/React Big Calendar ADR은 Sprint 7에서 진행
- **V102 시드 보정**: 보강데이, 공휴수업일, 단원평가 응시일의 3속성이 PRD와 불일치하므로 V301에서 보정
- **공휴일 데이터 갱신**: 2030년 만료 전 `pnpm holidays:fetch` 재실행 필요 -- ROADMAP 또는 Sprint 6 회고에 메모
- **A17(salt.bin 이전)**: 이번에도 범위 외 -- 별도 hotfix 또는 Sprint 7에서 처리. 현재 사용자 데이터 없음
