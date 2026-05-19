# Sprint Plan sprint2

## 기간
2026-05-20 ~ 2026-06-02 (2주, 10 영업일)

## 목표
Sprint 1 잔여 작업(루트 라우팅/인증 게이트, risk-register Medium 해소)을 해결하고, 원생/스케줄/표준교습비/코드 테이블의 DB 스키마와 IPC 커맨드를 구현하여 Sprint 3 프론트엔드가 호출할 백엔드 API를 완성한다.

## ROADMAP 연계 기능
- Phase 1 > Sprint 2: 기반 도메인 백엔드 (원생 CRUD, 수업 스케줄, 표준 교습비, 코드 테이블)
- Phase 1 > Sprint 2: DB 마이그레이션 V100~V103 (students, schedules, study_periods, schedule_codes, schedule_events)
- Sprint 1 잔여: 루트 라우팅 + 인증 게이트 미들웨어 (PRD SS5.6)
- Sprint 1 잔여: R6/R7/R8 risk-register Medium 해소

## 핵심 제약
- **마이그레이션 번호 예약**: V100~V199 (Sprint 2). 기존 V001, V008은 Sprint 1 산출물.
- **PI-05 확정**: 일련번호 자동 채번 구현. `MAX(serial_no)+1` + 트랜잭션(BEGIN IMMEDIATE). 사용자 override 허용.
- **PR 단계 생략**: 단일 개발자 단일 저장소. `gh pr create` 호출 금지.
- **단일 사용자 가정**: PRD 원장 1인 모델.
- **테스트 우선**: 비즈니스 규칙 100% 단위 테스트 커버 (PRD SS6.5).

---

## PI 결정 사항

| PI | 결정 | 근거 | 영향 Task |
|----|------|------|-----------|
| PI-05 | **자동 채번 구현** — `MAX(serial_no)+1` + `BEGIN IMMEDIATE` 트랜잭션. 사용자 override(`Some(n)`) 허용. | 단일 사용자라 race 불가, 코드 단순성 우선. PRD SS6.2 `serial_no UNIQUE` 제약 유지. | T5 (마이그레이션), T9 (IPC), T13 (프론트 래퍼), T14 (테스트) |

---

## 이전 회고 반영

> Sprint 1 회고 문서(`docs/sprint-retrospectives/sprint1.md`)가 존재하지 않으므로, sprint-review 코드 리뷰 결과(`docs/risk-register/sprint1-risks.md`)의 Medium 이슈를 직접 반영한다.

- **R6** (salt Keychain 저장): `app_settings` 테이블에 salt blob 컬럼 추가 마이그레이션 + 마법사 통합 시 이전 로직 준비 → T2에서 해소
- **R7** (release_lock advisory lock 미적용): `release_lock`에도 `try_lock_exclusive` 적용 → T3에서 해소
- **R8** (cipher on 실측 미수행): startup `elapsed_ms` 로깅 코드 추가 + 사용자 환경 시연 → T4에서 해소

---

## 작업 목록

### Day 1~2: Sprint 1 잔여 해소

- ⬜ **T1: 루트 라우팅 + 인증 게이트 미들웨어** (1일)
  - `src/app/page.tsx` 데모 `greet` 코드 제거, 클라이언트 가드 구현
  - `checkAuthStatus()` 분기: `not-initialized` → `/lock?mode=setup`, `locked` → `/lock`, 인증 후 → 메인
  - `src/app/lock/page.tsx` 인증 성공 시 `app_startup_sequence(password)` 호출 + 메인 redirect
  - 모든 IPC 호출 try/catch + 한국어 에러 메시지
  - **검증**: 첫 실행/재시작/인증 성공 후 흐름 모두 확인

- ⬜ **T2: R6 salt 저장 위치 이전 준비** (0.5일)
  - `V100__add_salt_to_app_settings.sql`: `app_settings` 테이블에 `password_salt BLOB` 컬럼 추가 (ALTER TABLE)
  - `auth.rs`에 `migrate_salt_to_db()` 헬퍼: Keychain에서 salt 읽기 → `app_settings`에 저장 → Keychain 항목 삭제
  - 마법사 미구현 상태에서는 앱 시작 시 자동 마이그레이션 (salt가 DB에 없고 Keychain에 있으면 이전)
  - **검증**: salt 마이그레이션 후 인증 정상 동작, Keychain 항목 삭제 확인

- ⬜ **T3: R7 release_lock advisory lock 적용** (0.5일)
  - `lock.rs` `release_lock()`: `OpenOptions::open` + `try_lock_exclusive` 획득 후 `remove_file` 수행
  - 락 획득 실패 시 에러 반환 (다른 프로세스 점유 중)
  - **검증**: 기존 lock 관련 단위 테스트 통과 + release 시 advisory lock 획득 확인

- ⬜ **T4: R8 cipher on 시작 시퀀스 성능 측정 코드** (0.5일)
  - `app_startup_sequence`에 `std::time::Instant` 기반 `elapsed_ms` 로깅 추가
  - `tracing::info!` 또는 `log::info!`로 각 단계별 소요 시간 출력 (동기화/락/무결성/인증)
  - cipher on 빌드에서 `cargo test` 통과 확인
  - **검증**: 콘솔/로그에서 단계별 소요 시간 확인 가능. 3초 초과 시 `PRAGMA cache_size` 튜닝 검토 항목 문서화

### Day 3~4: DB 마이그레이션

- ⬜ **T5: V101 students + student_schedules 마이그레이션** (1일)
  - `src-tauri/migrations/V101__create_students_and_schedules.sql`
  - `students`: `serial_no UNIQUE`, `enroll_date`, `withdraw_date`, `school_id FK`, 성별/학교급/학년 CHECK 제약
  - `student_schedules`: `day_of_week`, `start_time`, `duration_hours`, `effective_from/to`
  - 부분 인덱스: `UNIQUE(student_id, day_of_week) WHERE effective_to IS NULL`
  - `serial_no INTEGER NOT NULL UNIQUE` — 자동 채번 기본, 사용자 override 허용 (PI-05 확정)
  - **검증**: `sqlx migrate run` 성공, 테이블 구조 확인, `serial_no` UNIQUE 제약 동작 확인

- ⬜ **T6: V102 study_periods + schedule_codes 마이그레이션** (0.5일)
  - `src-tauri/migrations/V102__create_study_periods_and_schedule_codes.sql`
  - `study_periods`: `year_month UNIQUE`, `start_date`, `end_date`, `is_confirmed`, `is_closed`
  - `schedule_codes`: 3속성 모델 (`allows_regular_class`, `allows_makeup_class`, `is_duplicate_blocked`)
  - 시스템 예약 5종 시드 데이터 INSERT (보강데이/공휴수업일/방학/단원평가 응시일/휴원일)
  - **검증**: 시드 데이터 5건 존재, 3속성 값 PRD SS4.4.4 일치

- ⬜ **T7: V103 schedule_events 마이그레이션** (0.5일)
  - `src-tauri/migrations/V103__create_schedule_events.sql`
  - `schedule_events`: `code_id FK`, `event_date`, `period_end_date`, `display_name`
  - 중복불가 코드 검증 어플리케이션 레벨 구현 예정 (Sprint 4 IPC에서 처리)
  - **검증**: `sqlx migrate run` 성공

- ⬜ **T8: .sqlx/ 오프라인 캐시 갱신** (0.5일)
  - `sqlx prepare --manifest-path src-tauri/Cargo.toml` 실행
  - `.sqlx/` 디렉토리 갱신 확인 + 커밋
  - **검증**: `SQLX_OFFLINE=true cargo build --manifest-path src-tauri/Cargo.toml` 성공

### Day 5~7: 원생 CRUD + 스케줄 IPC 커맨드

- ⬜ **T9: 원생 CRUD IPC 커맨드** (1.5일)
  - `src-tauri/src/commands/students.rs` 신규 생성
  - `next_serial_number() -> i32`: `SELECT COALESCE(MAX(serial_no), 0) + 1 FROM students` — UI 기본값 표시용 (PI-05)
  - `create_student(payload: NewStudent)`: `payload.serial_no`가 `None`이면 자동 채번, `Some(n)`이면 UNIQUE 검증 후 사용 (PI-05)
    - 자동 채번 시 `BEGIN IMMEDIATE` 트랜잭션으로 race condition 안전망 (단일 사용자라 실질적 충돌 없음)
    - UNIQUE 위반 시 사용자 친화 한국어 메시지: "일련번호 {n}은(는) 이미 사용 중입니다. 다른 번호를 지정하거나 자동 채번을 사용해 주세요."
    - 필수 필드 검증 + audit log 기록
  - `update_student`: 기존 데이터 변경 + audit log (serial_no 변경 시에도 UNIQUE 검증)
  - `get_student`: ID 기반 단건 조회
  - `list_students`: 다중 필터 (이름/학교급/학년/학교명/요일/성별) + 정렬 (이름순/입교일순/학년순) + 재원 상태 필터
  - `withdraw_student`: `withdraw_date` 설정 + audit log (보강 관련 처리는 Phase 3)
  - 모든 커맨드 `Result<T, String>` 반환, `thiserror` 활용
  - `lib.rs` `invoke_handler`에 등록
  - **검증**: 인메모리 DB 단위 테스트:
    - CRUD 전체 흐름, 필터 조합
    - 자동 채번 연속 호출 시 1, 2, 3, ... 증가
    - 사용자가 100으로 override 후 다음 자동 채번이 101
    - UNIQUE 충돌 시 한국어 에러 메시지 정확성
    - `next_serial_number()` 반환값 정확성

- ⬜ **T10: 수업 스케줄 IPC 커맨드** (1일)
  - `src-tauri/src/commands/schedules.rs` 신규 생성
  - `set_schedule`: (원생, 요일) UNIQUE 검증 (현행 스케줄 기준), 기존 스케줄 `effective_to` 갱신 + 신규 INSERT
  - `get_schedules`: 원생별 현행 스케줄 목록 (effective_to IS NULL)
  - `get_schedule_history`: 원생별 변경 이력 전체
  - `get_weekly_hours`: 원생별 주 총 수업시간 산정 (`SUM(duration_hours) WHERE effective_to IS NULL`)
  - `lib.rs` `invoke_handler`에 등록
  - **검증**: 스케줄 변경 이력 자동 생성 테스트, 주 총 수업시간 계산 정확성 테스트

### Day 8~9: 표준교습비 + 코드 테이블 IPC + 프론트 래퍼

- ⬜ **T11: 표준 교습비 IPC 커맨드** (0.5일)
  - `src-tauri/src/commands/fees.rs` 신규 생성
  - `list_fees`: 전체 표준 교습비 목록
  - `create_fee`: 주 수업시간 구간 + 교습비 등록
  - `update_fee`: 교습비 수정 + audit log
  - `match_fee_by_hours`: 주 수업시간 → 해당 교습비 자동 매칭 함수
  - `lib.rs` `invoke_handler`에 등록
  - **검증**: 매칭 함수 정확성 테스트 (경계값: 정확 일치, 범위 초과, 미등록 구간)

- ⬜ **T12: 코드 테이블 CRUD IPC 커맨드** (0.5일)
  - `src-tauri/src/commands/codes.rs` 신규 생성
  - `list_codes`: 테이블 유형별 목록 (schools/payment_methods/card_companies)
  - `create_code`: 신규 등록 + sort_order 자동 부여
  - `update_code`: 수정 (is_active 소프트 삭제 포함)
  - `reorder_codes`: sort_order 일괄 변경
  - `lib.rs` `invoke_handler`에 등록
  - **검증**: 소프트 삭제 후 재활성화 테스트, 정렬 순서 변경 테스트

- ⬜ **T13: 프론트엔드 IPC 래퍼 업데이트** (0.5일)
  - `src/lib/tauri/index.ts`에 Sprint 2 신규 IPC 커맨드 래퍼 추가
  - 학생 CRUD: `nextSerialNumber`, `createStudent`, `updateStudent`, `getStudent`, `listStudents`, `withdrawStudent`
  - 스케줄: `setSchedule`, `getSchedules`, `getScheduleHistory`, `getWeeklyHours`
  - 교습비: `listFees`, `createFee`, `updateFee`, `matchFeeByHours`
  - 코드: `listCodes`, `createCode`, `updateCode`, `reorderCodes`
  - 개발 모드 fallback 값 포함
  - TypeScript 타입 정의: `src/types/` 하위에 `student.ts`, `schedule.ts`, `fee.ts`, `code.ts`
  - **검증**: `pnpm tsc --noEmit` 통과

### Day 10: 통합 검증 + 정리

- ⬜ **T14: 비즈니스 규칙 단위 테스트 보강 + 통합 검증** (1일)
  - 주 총 수업시간 계산: 다양한 스케줄 조합
  - 재원생 판정: 입교일/퇴교일 경계값
  - 스케줄 변경 이력: 연속 변경 시 이력 정합성
  - PI-05 자동 채번: 연속 호출 증가, override 후 채번 연속성, UNIQUE 충돌 한국어 메시지
  - `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
  - `pnpm lint` + `pnpm tsc --noEmit` 통과
  - `pnpm build` 성공 (static export)
  - **검증**: 전체 CI 수준 검증 완료

---

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 팀 인원 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10 영업일 |
| 실작업 가능 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |
| 총 Task 수 | 14개 |
| 예상 총 소요 | ~38시간 (Day 배정 기준) |
| 여유율 | ~5% (긴급 이슈 대응용) |

> Sprint 1 velocity 데이터 없음 (첫 회고 파일 미생성). 보수적으로 배분.

---

## 의존성 및 리스크

| # | 리스크 | 영향도 | 대응 |
|---|--------|--------|------|
| ~~R9~~ | ~~PI-05 미결정~~ — **해소됨**: PI-05 자동 채번 확정. `MAX+1` + `BEGIN IMMEDIATE` + override 허용. 추가 마이그레이션 불필요 (V101에 반영) | - | - |
| R10 | V100 ALTER TABLE 호환성 — SQLite ALTER TABLE ADD COLUMN은 지원하나 NOT NULL 제약 추가 시 DEFAULT 필요 | 낮음 | salt 컬럼을 NULLABLE로 추가 (NULL = 미마이그레이션 상태) |
| R11 | 부분 인덱스(`WHERE effective_to IS NULL`) SQLite 지원 여부 — SQLite 3.8.0+ 지원 | 낮음 | Tauri 번들 SQLite 버전 확인. 미지원 시 어플리케이션 레벨 UNIQUE 검증으로 대체 |

---

## 기술 접근 방법

### 루트 라우팅 (T1)
- Next.js App Router의 root layout에 클라이언트 가드 패턴 적용. `src/middleware.ts`는 static export에서 미동작하므로, root `page.tsx`에서 `useEffect` + `checkAuthStatus()` IPC 호출로 분기.
- 인증 상태는 React state로 관리 (Zustand 도입은 Sprint 3).

### DB 마이그레이션 (T5~T8)
- `data-model.md` 참조하되 PRD가 SSOT. 컬럼 타입/제약은 PRD SS6.2 준수.
- `query!` / `query_as!` 매크로 사용으로 컴파일 타임 타입 검사.
- 마이그레이션 번호: V100(salt), V101(students+schedules), V102(study_periods+schedule_codes), V103(schedule_events).

### IPC 커맨드 구조 (T9~T12)
- `src-tauri/src/commands/` 하위에 도메인별 파일 분리 (students.rs, schedules.rs, fees.rs, codes.rs).
- 공통 패턴: `#[tauri::command] async fn ...` + `Result<T, String>` + `AppError` 변환.
- 필터링 쿼리는 동적 SQL 빌더 사용 금지 — `query!` 매크로와 WHERE 조건 분기로 처리.

### PI-05 일련번호 자동 채번 (T9)

**확정 알고리즘**: `SELECT COALESCE(MAX(serial_no), 0) + 1 FROM students` + `BEGIN IMMEDIATE` 트랜잭션.

- **선택 사유**: 단일 사용자(원장 1인)라 race condition 실질 불가. 코드 단순성 우선. `AUTOINCREMENT`나 별도 `serial_counters` 테이블 대비 구현 부담 최소.
- **PRD SS6.2 정합**: `serial_no INTEGER NOT NULL UNIQUE` 제약 유지. 자동 채번이 기본 동작이되 사용자 override(`Some(n)`) 허용으로 마이그레이션/복구 시 유용.
- **인터페이스 설계** (2-IPC 패턴):
  1. `next_serial_number() -> i32` — UI가 등록 폼 진입 시 호출하여 기본값 표시
  2. `create_student(payload)` — `payload.serial_no: Option<i32>`. `None`이면 내부에서 재산출(등록 폼 진입~저장 사이 다른 등록이 끼어들 가능성 대비), `Some(n)`이면 그대로 사용
- **UNIQUE 위반 처리**: `sqlx::Error::Database` 에서 UNIQUE constraint 감지 → 한국어 사용자 메시지 변환 ("일련번호 {n}은(는) 이미 사용 중입니다")
- **override 후 채번 연속성**: `MAX(serial_no)+1` 방식이므로 사용자가 100을 지정하면 다음 자동 채번은 101. 빈 번호(gap)는 의도적으로 허용.

### 프론트엔드 래퍼 (T13)
- `src/lib/tauri/index.ts` 확장. 개발 모드 fallback으로 브라우저 테스트 가능.
- TypeScript 타입은 Rust struct와 1:1 매칭 (`src/types/` 하위).

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ V100~V103 마이그레이션 정상 적용 + `.sqlx/` 오프라인 캐시 갱신/커밋
- ⬜ 루트 라우팅: 첫 실행 시 잠금 화면 자동 진입 (PRD SS5.6)
- ⬜ IPC 커맨드별 단위 테스트 통과 (인메모리 DB)
- ⬜ 비즈니스 규칙 단위 테스트 100% 커버 (주 총 수업시간, 재원생 판정, 스케줄 이력)
- ⬜ 원생 50명 기준 CRUD 응답 300ms 이내
- ⬜ R6 salt 이전 완료 + R7 release_lock advisory lock 적용
- ⬜ R8 cipher on 성능 측정 코드 추가 + 로그 출력 확인
- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ DEPLOY.md 업데이트
- ⬜ ROADMAP.md 상태 갱신

---

## 예상 산출물

### 백엔드
- `src-tauri/migrations/V100__add_salt_to_app_settings.sql`
- `src-tauri/migrations/V101__create_students_and_schedules.sql`
- `src-tauri/migrations/V102__create_study_periods_and_schedule_codes.sql`
- `src-tauri/migrations/V103__create_schedule_events.sql`
- `src-tauri/src/commands/students.rs`
- `src-tauri/src/commands/schedules.rs`
- `src-tauri/src/commands/fees.rs`
- `src-tauri/src/commands/codes.rs`
- `src-tauri/.sqlx/` (갱신)

### 프론트엔드
- `src/app/page.tsx` (인증 게이트 적용)
- `src/lib/tauri/index.ts` (IPC 래퍼 확장)
- `src/types/student.ts`
- `src/types/schedule.ts`
- `src/types/fee.ts`
- `src/types/code.ts`

### 문서
- `docs/sprint/sprint2.md` (본 문서)
- `docs/risk-register/2026-05-20.md` (리스크 기록)

---

## 참고 사항

- **ci.yml 트리거 정리 (C항목)**: Forbidden Area 정책 적용 대상. 본 sprint에서는 scope 외로 분류. 사용자가 명시적으로 요청 시 scope.md에 추가 후 진행.
- **초기 설정 마법사**: Sprint 3 범위. T2의 salt 이전은 마법사 없이도 앱 시작 시 자동 수행되도록 구현.
- **PI-05 확정 (2026-05-20)**: 자동 채번 구현. `MAX+1` + `BEGIN IMMEDIATE` + override 허용. 상세는 "PI 결정 사항" 섹션 참조.
- **ROADMAP Sprint 2 설명과 차이**: ROADMAP에는 Sprint 1 잔여 작업(T1~T4)이 미포함. 본 계획은 Sprint 1 잔여를 통합한 실행 계획.
