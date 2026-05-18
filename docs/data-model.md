# 데이터 모델 1차 매핑 — SmartHB v1.5 (참고용)

> **목적**: PRD.md §6.1 도메인 개념도를 SQLite + SQLx 스키마로 1차 매핑한 **참고용 가이드**. Sprint 1~2의 sqlx 마이그레이션 작성 시 출발점이 된다.
> **강제력 없음**: 본 문서는 `sprint-planner` / `sprint-dev` / 개발자가 실제 스프린트 진입 시 조정 가능. PRD가 SSOT이며, 본 문서는 보조 가이드다.
> **v1.5 주요 변경**: 학습보고서 도메인을 **분기 단위**로 재설계 — 키 `(quarter, student_id)`, 단일 컬럼 `overall_opinion`, 단원평가 점수에 종속(저장 시 점수 복사 보관 금지).
> **PRD 정합성 이슈**: PI-02(보강-결석 매칭 시간값), PI-05(자동 채번 규칙) 등이 데이터 모델 결정에 영향을 미친다. 사용자 결정 후 본 문서를 갱신한다.
> **마이그레이션 파일 명명**: `V{NNN}__{설명}.sql` — 예: `V001__create_students.sql` (CLAUDE.md / `.claude/rules/backend.md` 기존 규칙 준수)

---

## 0. 공통 원칙

- **PK**: 모든 핵심 도메인 테이블은 `id INTEGER PRIMARY KEY AUTOINCREMENT` 사용
- **타임스탬프**: 모든 테이블에 `created_at`, `updated_at` (`DATETIME DEFAULT (datetime('now'))`)
- **소프트 삭제**: 코드성 테이블은 `is_active BOOLEAN DEFAULT 1` 으로 "사용안함" 표현 (PRD AC-4.12-1). 도메인 데이터는 hard delete 대신 상태 컬럼 활용
- **외래키**: `FOREIGN KEY ... ON DELETE RESTRICT` 기본 — cascade는 명시적으로만
- **ENUM 표현**: SQLite는 ENUM 미지원 → `TEXT CHECK(status IN ('...','...'))` 패턴
- **인덱스**: PRD §6.2 비즈니스 키 UNIQUE 제약 + 자주 필터링되는 컬럼(`student_id`, `attendance_date`, `bill_year_month`)에 인덱스
- **암호화**: 모든 테이블은 SQLCipher AES-256으로 암호화된 DB 파일 내부 (PRD §5.1)

---

## 1. 원생 관리 도메인

### 1.1 `students` (원생)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `serial_no` | TEXT | UNIQUE NOT NULL | 일련번호 (수동 입력 또는 자동 채번 — PI-05 결정 대기) |
| `name` | TEXT | NOT NULL | 원생 이름 |
| `gender` | TEXT | CHECK IN ('male','female') | 성별 (PRD §4.1.1) |
| `school_level` | TEXT | CHECK IN ('elementary','middle') | 학교급 |
| `grade` | INTEGER | CHECK BETWEEN 1 AND 9 | 학년 (초1~6 / 중1~3) |
| `school_id` | INTEGER | FK → `schools.id` | |
| `phone_student` | TEXT | | 원생 연락처 |
| `phone_mother` | TEXT | | 모 연락처 |
| `phone_father` | TEXT | | 부 연락처 |
| `enroll_date` | DATE | NOT NULL | 입교일 |
| `withdraw_date` | DATE | NULL ALLOWED | 퇴교일 (재원중이면 NULL) |
| `created_at`, `updated_at` | DATETIME | | |

- **재원생 조건**: `enroll_date <= today AND (withdraw_date IS NULL OR today <= withdraw_date)`
- **AC-4.1.1-3**: `serial_no UNIQUE`
- **AC-4.1.1-4**: 트리거 또는 어플리케이션 레벨 검증 — `withdraw_date >= enroll_date`

### 1.2 `student_schedules` (요일별 수업 스케줄)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `student_id` | INTEGER | FK → `students.id` ON DELETE CASCADE | |
| `day_of_week` | INTEGER | CHECK BETWEEN 1 AND 7 | 1=월, 7=일 |
| `start_time` | TEXT | NOT NULL | "HH:MM" 형식 |
| `duration_hours` | INTEGER | CHECK > 0 | 1회 수업 시간 (시간 단위, PRD §4.2.1) |
| `effective_from` | DATE | NOT NULL | 적용 시작일 (PRD §4.2.2 변경 이력) |
| `effective_to` | DATE | NULL ALLOWED | 적용 종료일 (현재 사용 중이면 NULL) |
| `created_at`, `updated_at` | DATETIME | | |

- **PRD §6.2 (원생, 요일) UNIQUE**: 현재 사용 중인 스케줄만 — `UNIQUE(student_id, day_of_week) WHERE effective_to IS NULL` (부분 인덱스)
- **AC-4.2-2**: 스케줄 변경 시 기존 행의 `effective_to` 갱신 + 신규 행 INSERT
- **주 총 수업시간** (PI-03 용어 통일 권장): `SELECT SUM(duration_hours) FROM student_schedules WHERE student_id=? AND effective_to IS NULL`

### 1.3 `schools` (학교 코드)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `name` | TEXT | UNIQUE NOT NULL | 학교명 |
| `sort_order` | INTEGER | DEFAULT 0 | |
| `is_active` | BOOLEAN | DEFAULT 1 | 사용안함 처리 |
| `created_at`, `updated_at` | DATETIME | | |

---

## 2. 학사 도메인

### 2.1 `study_periods` (교습기간)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `year_month` | TEXT | UNIQUE NOT NULL | "YYYY-MM" 형식 |
| `start_date` | DATE | NOT NULL | 교습기간 시작일 |
| `end_date` | DATE | NOT NULL | 교습기간 종료일 |
| `is_confirmed` | BOOLEAN | DEFAULT 0 | "확정" 상태 |
| `is_closed` | BOOLEAN | DEFAULT 0 | 지난 달 읽기 전용 잠금 (AC-4.4-1) |
| `created_at`, `updated_at` | DATETIME | | |

- **PRD §6.2 교습기간 일자 중첩 금지**: 어플리케이션 검증 + 트리거로 보강
- **AC-4.4-1**: `is_closed = 1` 이면 모든 학사 데이터 수정 차단

### 2.2 `schedule_codes` (학사 일정 코드 — 3속성 모델)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `code_name` | TEXT | UNIQUE NOT NULL | "보강데이", "공휴수업일", 사용자 추가 코드 등 |
| `is_system_reserved` | BOOLEAN | DEFAULT 0 | 시스템 예약 5종 여부 |
| `allows_regular_class` | BOOLEAN | NOT NULL | 정규수업 진행 (PRD §4.4.3) |
| `allows_makeup_class` | BOOLEAN | NOT NULL | 보강 진행 가능 |
| `is_duplicate_blocked` | BOOLEAN | NOT NULL | 중복불가 |
| `is_period_type` | BOOLEAN | DEFAULT 0 | 기간성 여부 (방학 등) |
| `is_active` | BOOLEAN | DEFAULT 1 | 사용 여부 |
| `created_at`, `updated_at` | DATETIME | | |

- **시스템 예약 5종 시드 데이터** (PRD §4.4.4): 보강데이 / 공휴수업일 / 방학 / 단원평가 응시일 / 휴원일
- **AC-4.4-5**: `is_system_reserved = 1` 행의 3속성 컬럼은 어플리케이션에서 변경 차단

### 2.3 `schedule_events` (학사 일정 배치)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `code_id` | INTEGER | FK → `schedule_codes.id` | |
| `event_date` | DATE | NOT NULL | 단일 일자 |
| `period_end_date` | DATE | NULL ALLOWED | 기간성 일정의 종료일 (단일 일정이면 NULL) |
| `display_name` | TEXT | | 셀에 표시할 일정명 (시스템 디폴트 또는 사용자 수정) |
| `created_at`, `updated_at` | DATETIME | | |

- **PRD §6.2 중복불가 코드 (일자, 코드) UNIQUE**: 부분 인덱스 — `UNIQUE(event_date, code_id) WHERE code_id IN (중복불가 코드 IDs)` 또는 어플리케이션 검증

---

## 3. 출결·보강 도메인

### 3.1 `regular_attendances` (정규 출결)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `student_id` | INTEGER | FK → `students.id` | |
| `attendance_date` | DATE | NOT NULL | |
| `duration_hours` | INTEGER | NOT NULL | 해당일 수업 시간 (스케줄 기준 스냅샷) |
| `status` | TEXT | CHECK IN ('present','absent','makeup_done','makeup_expired') NOT NULL DEFAULT 'present' | PRD §4.5.2 4종 |
| `expiry_year_month` | TEXT | NULL ALLOWED | 결석 상태일 때 소멸기한 (YYYY-MM, PRD §4.5.7) |
| `absence_memo` | TEXT | | 결석 사유 메모 (PRD §4.5.3) |
| `linked_makeup_id` | INTEGER | FK → `makeup_attendances.id` NULL | "보강완료"일 때 매칭된 보강 1건 |
| `created_at`, `updated_at` | DATETIME | | |

- **PRD §6.2 (원생, 일자) UNIQUE**: `UNIQUE(student_id, attendance_date)`
- **AC-4.5-1**: 출결 생성 트랜잭션 1회만 — 어플리케이션에서 `study_periods.year_month + students.id` 조합으로 중복 검증
- **소멸 자동 전이** (PI-01 결정 대기): 별도 batch — 출결 생성 시점이 아니어도 발동 가능한 트리거 필요

### 3.2 `makeup_attendances` (보강 출결)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `student_id` | INTEGER | FK → `students.id` | |
| `attendance_date` | DATE | NOT NULL | 보강 일자 |
| `duration_hours` | INTEGER | NOT NULL | 보강 수업 시간 |
| `status` | TEXT | CHECK IN ('progressing','absent') NOT NULL DEFAULT 'progressing' | 보강진행/보강결석 |
| `created_at`, `updated_at` | DATETIME | | |

- **PRD §6.2 보강 출결은 같은 일자 중복 허용**: UNIQUE 제약 없음
- **보강-결석 1:N 매칭**: `regular_attendances.linked_makeup_id` 가 보강 1건을 가리킴 — 보강 1건이 여러 결석에 연결되는 1:N 구조
- **시간값 처리** (PI-02 결정 대기): 보강 1건의 `duration_hours` ≥ 충당 결석들의 `duration_hours` 합 제약 필요 여부 결정 후 반영

### 3.3 보강 데이터 정합성 (자가 진단 §6.6 활용)

- **고아 데이터 검출 쿼리** (PI-06 용어 정의 후 PRD에 명시 권장):
  ```sql
  -- 어떤 결석에도 연결되지 않은 보강
  SELECT m.* FROM makeup_attendances m
  WHERE m.status = 'progressing'
    AND NOT EXISTS (
      SELECT 1 FROM regular_attendances r
      WHERE r.linked_makeup_id = m.id AND r.status = 'makeup_done'
    );
  ```

---

## 4. 단원평가 도메인

### 4.1 `assessment_events` (단원평가 회차)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `name` | TEXT | NOT NULL | "2026-06 1차" 등 |
| `period_start` | DATE | NOT NULL | 응시 기간 시작 |
| `period_end` | DATE | NOT NULL | 응시 기간 종료 |
| `grade_units_memo` | TEXT | | 학년별 평가대상 단원 설명 |
| `created_at`, `updated_at` | DATETIME | | |

### 4.2 `assessment_scores` (원생별 단원평가 점수)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `event_id` | INTEGER | FK → `assessment_events.id` | |
| `student_id` | INTEGER | FK → `students.id` | |
| `test_date` | DATE | NULL ALLOWED | 실제 응시일 (PRD §4.7.2) |
| `score_first` | INTEGER | CHECK BETWEEN 0 AND 100 NULL | 1차 점수 |
| `score_second` | INTEGER | CHECK BETWEEN 0 AND 100 NULL | 2차 점수 |
| `created_at`, `updated_at` | DATETIME | | |

- **AC-4.7-2**: `score_first IS NULL` 시 `score_second` 입력 차단 (어플리케이션)
- **(event_id, student_id) UNIQUE**

### 4.3 `learning_reports` (분기 학습보고서) — **PRD v1.5에서 재설계**

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `quarter` | TEXT | NOT NULL | "YYYY-Q[1-4]" 형식 — 학사력 분기 (1Q: 3~5월 / 2Q: 6~8월 / 3Q: 9~11월 / 4Q: 12~2월) |
| `student_id` | INTEGER | FK → `students.id` | |
| `overall_opinion` | TEXT | NOT NULL | 종합의견 (멀티라인, 줄바꿈은 인쇄 시 그대로 반영) |
| `created_at`, `updated_at` | DATETIME | | |

- **`(quarter, student_id) UNIQUE`** — PRD §4.8.3, §6.2 "한 원생은 한 분기에 보고서 1건"
- **점수 종속 원칙 (PRD v1.5)**: 보고서는 단원평가 점수 데이터를 **직접 참조**한다. 1차/2차 점수, 점수 추이 차트 등은 저장 시 복사 보관하지 않으며, 조회 시점에 `unit_test_scores` 테이블에서 동적 산출. 점수 수정 시 보고서 표시도 자동 반영.
- **작성 시점 제약 (AC-4.8-6)**: 분기 마지막 월의 2차 단원평가 점수 입력 완료 전에는 보고서 생성을 어플리케이션에서 차단 (백엔드 IPC 커맨드에서 검증)
- **6회 미만 시행 대응 (AC-4.8-7)**: 점수 조회 IPC는 실제 시행 회차만 반환 (NULL 패딩 금지)
- **분기 계산**: `quarter` 값은 `unit_test_events.event_date` 또는 입력일 기준 학사력으로 산출하는 헬퍼 함수를 backend에 두는 것을 권장 (예: `2026-04-15` → `"2026-Q1"`, `2026-12-10` → `"2026-Q4"`)

---

## 5. 청구·수납 도메인

### 5.1 `standard_fees` (표준 교습비 코드)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `weekly_hours` | INTEGER | UNIQUE NOT NULL | 주 총 수업시간 |
| `amount` | INTEGER | NOT NULL | 표준 교습비(원) |
| `sort_order` | INTEGER | DEFAULT 0 | |
| `is_active` | BOOLEAN | DEFAULT 1 | |
| `created_at`, `updated_at` | DATETIME | | |

### 5.2 `bills` (청구)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `student_id` | INTEGER | FK → `students.id` | |
| `year_month` | TEXT | NOT NULL | "YYYY-MM" |
| `weekly_hours_snapshot` | INTEGER | NOT NULL | 청구 시점 주 총 수업시간 |
| `amount` | INTEGER | NOT NULL | 청구 교습비 (조정 후 최종값) |
| `is_mid_month_change` | BOOLEAN | DEFAULT 0 | 월 중 입퇴교 플래그 |
| `status` | TEXT | CHECK IN ('draft','confirmed','closed') NOT NULL DEFAULT 'draft' | 미확정/확정/마감 (PRD §4.9.7) |
| `closing_note` | TEXT | | 마감 후 수정 시 사유 (AC-4.9-8) |
| `created_at`, `updated_at` | DATETIME | | |

- **PRD §6.2 (원생, 청구년월) UNIQUE**: `UNIQUE(student_id, year_month)`
- **상태 전이 (PRD v1.5 §4.9.7)**: `draft` → `confirmed` → `closed` 순서만 허용
  - `draft` → `confirmed`: 원장 검토 완료, 수정 시 확인 다이얼로그 필수 (AC-4.9-3)
  - `confirmed` → `closed`: 해당 월 모든 청구가 `confirmed` 상태일 때만 가능 (AC-4.9-7)
  - `closed` 상태에서 금액 수정 시 `closing_note` 입력 필수 (AC-4.9-8)
- `closed_at DATETIME NULL` 컬럼 추가 검토 (마감 시각 기록용)

### 5.3 `payments` (수납)

| 컬럼 | 타입 | 제약 | 비고 |
|------|------|------|------|
| `id` | INTEGER | PK | |
| `bill_id` | INTEGER | FK → `bills.id` UNIQUE | 청구당 1건 |
| `paid_date` | DATE | NULL ALLOWED | 입금일 (NULL이면 미입금) |
| `payer_name` | TEXT | | 입금자명 |
| `payment_method_id` | INTEGER | FK → `payment_methods.id` | |
| `card_company_id` | INTEGER | FK → `card_companies.id` NULL | 카드 결제 시 필수 (AC-4.9-4) |
| `created_at`, `updated_at` | DATETIME | | |

### 5.4 `payment_methods`, `card_companies` (코드성)

- `payment_methods`: 시드 데이터 6종 (계좌이체, 현금, 신용카드, 체크카드, 결제선생, 지역화폐) + 사용자 추가
- `card_companies`: 시드 데이터 8종 (삼성, 신한, KB국민, 우리, 현대, 롯데, 카카오뱅크, 케이뱅크) + 사용자 추가

---

## 6. 시스템 설정 / 보안

### 6.1 `app_settings` (전역 설정, 단일 행)

| 컬럼 | 타입 | 비고 |
|------|------|------|
| `cloud_sync_folder_path` | TEXT | 사용자 지정 클라우드 동기화 폴더 (PRD §5.3) |
| `notice_image_output_folder` | TEXT | 공지문 PNG 저장 경로 |
| `wizard_completed_at` | DATETIME | 마법사 완료 시각 (NULL이면 마법사 자동 진입) |
| `business_hours_json` | TEXT | 요일별 운영시간 JSON (PRD §4.12.2) |
| ... | | |

> **참고**: `cloud_sync_folder_path` 와 `app.db` 위치는 같은 폴더 — 사용자가 마법사 단계 6에서 지정한 경로 (PI-10 명확화 권장).
> 클라우드 동기화 폴더 경로 자체는 DB 외부의 secure config 파일에 보관 (DB 위치를 결정하는 메타정보이므로 DB 내부 저장 불가).

### 6.2 `audit_logs` (감사 로그)

PRD §7.3 — 1년 롤링 보관. 출결/청구/코드/원생 등록·퇴교 우선 기록.

| 컬럼 | 타입 | 비고 |
|------|------|------|
| `id` | INTEGER PK | |
| `occurred_at` | DATETIME | |
| `entity_type` | TEXT | "students" / "bills" / ... |
| `entity_id` | INTEGER | |
| `action` | TEXT | "create" / "update" / "delete" / "status_change" |
| `before_value_json` | TEXT | |
| `after_value_json` | TEXT | |
| `user_id` | TEXT | 단일 사용자 — 고정값 또는 OS 사용자명 |

---

## 7. 자가 진단 (PRD §6.6) — 검사 항목 의사 SQL

| 검사 항목 | 의사 SQL |
|-----------|----------|
| 보강필요시간 음수/이상값 | 별도 view 또는 함수로 계산: 결석 합산 - 보강완료 합산 — 음수면 이상 |
| 재원중 원생의 당월 출결 미생성 | `재원생 - (regular_attendances WHERE year_month=?의 student_id 집합)` |
| 재원중 원생의 당월 청구 미생성 | `재원생 - (bills WHERE year_month=?의 student_id 집합)` |
| 스케줄 vs 출결 불일치 | `regular_attendances`의 요일이 `student_schedules`에 없거나 duration_hours 다름 |
| 결석에 소멸기한 미설정 | `regular_attendances WHERE status='absent' AND expiry_year_month IS NULL` |
| 고아 보강 데이터 | §3.3 쿼리 |
| 입금 완료 표시 + 입금 정보 누락 | `payments WHERE paid_date IS NOT NULL AND (payer_name IS NULL OR payment_method_id IS NULL)` |

진단 결과는 별도 테이블에 누적 보관 (최근 12개월, PRD AC-6.6-4).

---

## 8. 마이그레이션 순서 권장 (Sprint 1~2 참고)

1. `V001__create_schools_and_payment_codes.sql` — 코드성 (학교, 결제수단, 카드사, 표준교습비)
2. `V002__create_students_and_schedules.sql` — 원생 + 스케줄 + 스케줄 이력
3. `V003__create_study_periods_and_schedule_codes.sql` — 교습기간 + 학사 코드 (시스템 예약 5종 시드 포함)
4. `V004__create_schedule_events.sql` — 학사 일정 배치
5. `V005__create_attendances.sql` — 정규 출결 + 보강 출결
6. `V006__create_assessments_and_reports.sql` — 단원평가 회차/점수, 학습보고서
7. `V007__create_bills_and_payments.sql` — 청구 + 수납
8. `V008__create_app_settings_and_audit_logs.sql` — 전역 설정 + 감사 로그
9. `V009__create_diagnosis_history.sql` — 자가 진단 이력 (PRD §6.6)

---

## 9. 미해결 항목 (PRD 결정 대기)

본 문서는 다음 결정에 따라 갱신된다 — `docs/prd-issues.md` 참조.

- **PI-01**: 소멸기한 자동 전이 트리거 시점 → `regular_attendances` 스타트업 batch 또는 trigger 설계
- **PI-02**: 보강-결석 시간값 매칭 규칙 → `linked_makeup_id` 외 시간 차감 컬럼 필요 여부 결정
- **PI-05**: 일련번호 자동 채번 규칙 → `students.serial_no` 채번 함수 또는 어플리케이션 카운터
- **PI-07**: 복구 코드 발급/검증 → `app_settings` 또는 별도 `recovery_codes` 테이블 추가 여부
