---
Sprint: 10  |  Date: 2026-05-26  |  Session: #1
---

> Sprint 10 Session #1 — T1 (Sprint 9 dead code 정리).
> 예상 2h. 단순 삭제 작업 + 단위 테스트 제거.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1** | `mark_makeup_absent` + `batch_create_makeups` 폐기 코드 완전 제거 (A49 carry-over) | 2h |

> Sprint 9에서 사용자 결정으로 폐기되었으나 dead code 상태로 남은 항목 정리.

---

## T1 작업 범위

### 백엔드 정리
1. `src-tauri/src/commands/makeup.rs`:
   - `mark_makeup_absent` 함수 + IPC `#[tauri::command]` 핸들러 삭제
   - `batch_create_makeups` 함수 + IPC 핸들러 삭제
   - 관련 payload struct(BatchMakeupEntry 등) 삭제
   - 관련 단위 테스트 삭제 (`mark_makeup_absent_*`, `batch_create_makeups_*` 등)
2. `src-tauri/src/lib.rs`:
   - `invoke_handler!`에서 `makeup::mark_makeup_absent`, `makeup::batch_create_makeups` 제거
3. `src-tauri/src/commands/audit.rs`:
   - `AuditEventType::MakeupAbsent` variant 삭제 (다른 참조 0건 확인 후)
   - 관련 직렬화 string 매핑 정리
4. (선택) V108 마이그레이션:
   - `makeup_attendances.status` CHECK 제약에서 `'makeup_absent'` 제거
   - 데이터 행 없음 (Sprint 9 J5에서 폐기, 운용 데이터 없음) — 안전한 변경
   - 본 세션에서는 코드 정리에만 집중, 마이그레이션은 별도 판단

### 프론트엔드 정리
5. `src/lib/tauri/index.ts`:
   - `markMakeupAbsent`, `batchCreateMakeups` 래퍼 이미 제거됨 (Sprint 9 T12) — 재확인만
6. `src/types/makeup.ts`:
   - `BatchMakeupEntry`, `BatchCreateMakeupsPayload`, `BatchFailure`, `BatchResult` 타입 이미 제거됨 — 재확인만

---

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/makeup.rs | [0회] | 함수/IPC 핸들러/단위 테스트 삭제 |
| src-tauri/src/commands/audit.rs | [0회] | MakeupAbsent variant 삭제 |
| src-tauri/src/lib.rs | [0회] | invoke_handler 정리 |
| docs/sprint/sprint10/scope.md | [0회] | 본 파일 |

> 프론트엔드 파일은 Sprint 9 T12에서 이미 정리됨 — 확인만, 수정 없음 예상.

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [x] .github/workflows/ — CI/CD 파이프라인
- [x] SETUP.sh — 초기화 스크립트
- [x] src-tauri/migrations/ — 본 세션에서 마이그레이션 추가하지 않음 (V108은 별도 판단)
- [x] src/components/attendance/ — Sprint 9 UI는 이미 정리 완료

---

## 완료 기준 (이번 세션) — T1 AC

- ✅ `cargo test --manifest-path src-tauri/Cargo.toml` 251 passed / 0 failed (256 → -5, 삭제한 단위 테스트 5건과 일치)
- ✅ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- ✅ dead code warning 0건
- ✅ TS 영향 없음 — Sprint 9 T12에서 이미 정리됨 확인

## 세션 종료 조건

- ✅ T1 AC 모두 통과
- ⬜ 단일 커밋
- ⬜ 사용자 메모리 미러 동기화 (Session #1 → #2 갱신)
- ⬜ 다음 세션(T2 — 소멸 자동 전이 설계 + 사용자 확인) 진입점 준비

## 실제 수정 결과

| 파일 | 변동 |
|------|------|
| src-tauri/src/commands/makeup.rs | -343 라인 (mark_makeup_absent 함수/impl + batch 함수/impl + payload struct 4종 + 단위 테스트 5건 + 모듈 헤더 주석 정리) |
| src-tauri/src/commands/audit.rs | -2 라인 (`MakeupAbsent` variant + string 매핑) |
| src-tauri/src/lib.rs | -2 라인 (invoke_handler 2건) |
| docs/sprint/sprint10/scope.md | 신규 — Session #1 |

## 발견된 이슈

(없음 — 진행 중 발견 시 기록)

## 다음 세션 (T2) 미리보기

- 소멸 전이 트리거 3개소 설계 (앱 시작 / 출결 생성 / 교습기간 등록)
- 소멸기한 판정 로직 확정
- 사용자 확인: 교습기간 미등록 월의 소멸 처리 방식 (A51 패턴)
- V108 마이그레이션 필요 여부 최종 판단

---

## carry-over

(Session #1 시작 시점에 carry-over 없음 — Sprint 9 완전 종료 + develop 머지 완료)

---

## Session #2 (T2 — 소멸 자동 전이 설계 + 사용자 확인, 2026-05-26)

> Sprint 10 Session #2 — T2 (소멸 도메인 설계).
> 예상 2h. 본 세션은 **순수 설계/사용자 확인** task — 코드 변경 없음, scope.md 작성 + 사용자 결정 기록으로 종료.

### 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T2** | 소멸 자동 전이 설계 + 사용자 확인 항목 수렴 | 2h |

---

### PRD §4.5.7 소멸 규칙 (확정 사항 — 사용자 확인 불필요)

| 항목 | 규칙 |
|------|------|
| `makeup_deadline` 저장 단위 | **년월 (YYYY-MM)** — 결석 발생 월 + 1 |
| 실제 소멸일 판정 | `makeup_deadline` 년월의 `study_periods.end_date` |
| 다음 월 교습기간 미등록 | "소멸기한 미확정" 표시 → 교습기간 등록 시 자동 확정 |
| 자동 전이 시점 | "다음 월 출결 생성 시 소멸기한 도래 + 미보강 결석" (PRD §4.5.7 명시) |

→ Sprint 8 T1 V106 마이그레이션에서 이미 `makeup_deadline TEXT` (YYYY-MM 형식 CHECK 포함) + 토글 시 `결석 발생월 + 1` 자동 설정 구현됨.

→ PRD §4.5.7이 "다음 월 출결 생성 시"만 명시하므로, sprint10.md L17의 **앱 시작 시 batch** + **교습기간 등록 직후** 트리거 2건은 PRD 명시 범위 밖 — 사용자 확인 필수.

---

### 사용자 결정 사항 (2026-05-26)

| ID | 질문 | ✅ 결정 | 비고 |
|----|------|---------|------|
| **PI-05** | 소멸 자동 전이 트리거 범위 | **3개소 — 앱 시작 + 출결 생성 + 교습기간 등록** | 앱 시작 시 누락 대비 backup + 교습기간 등록 시 "소멸기한 미확정" 즉시 해소. T4에서 통합 |
| **PI-06** | 소멸 판정 기준일 | **오늘(`chrono::Local::now()`)** | 단위 테스트는 `Option<NaiveDate>` 주입 시그니처로 결정성 확보 |
| **PI-07** | V108 마이그레이션 (`makeup_attendances.status` CHECK 정리) | **Sprint 10에서 진행** | **T1' 신규 작업으로 추가** — T1 직후 진행. CHECK 제약에서 `'makeup_absent'` 제거, 데이터 행 0건이므로 안전 |
| **PI-08** | 선행 수업(§4.2.3) 구현 범위 | **기존 상태 토글 흐름 활용** | 그리드의 미래 일자 셀을 토글로 결석 등록 → 즉시 보강 매칭. PRD §4.2.3 "별도 출결 타입 신설 없이 보강 메커니즘 통합 처리"와 일치. **T7 작업 범위 축소** — 신규 IPC 불필요, 출결 생성 충돌 방지 검증만 수행 |
| **PI-09** | 자동 전이 결과 사용자 알림 방식 | **토스트 "소멸 처리된 결석 N건"** (건수 > 0일 때만) | 50대 친화 비강제 알림. 상세는 audit 로그/결석 이력 메뉴에서 확인 |

---

### 설계서 — 소멸 자동 전이 모듈

신규 모듈: `src-tauri/src/commands/expiration.rs`

```rust
// 핵심 함수
async fn expire_overdue_absences_impl(
    pool: &SqlitePool,
    as_of: Option<NaiveDate>,  // None → chrono::Local::now()
) -> Result<ExpirationReport, String>;

// IPC 커맨드 (사용자 호출 가능 — 수동 발동 + 디버깅)
#[tauri::command]
pub async fn expire_overdue_absences() -> Result<ExpirationReport, String>;

// 응답 구조체
pub struct ExpirationReport {
    pub transitioned_count: usize,
    pub details: Vec<ExpiredAbsenceDetail>,
}
pub struct ExpiredAbsenceDetail {
    pub student_name: String,
    pub event_date: String,    // YYYY-MM-DD
    pub makeup_deadline: String, // YYYY-MM
}
```

**핵심 SQL**:

```sql
UPDATE regular_attendances
SET status = 'makeup_expired',
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE status = 'absent'
  AND makeup_attendance_id IS NULL
  AND makeup_deadline IS NOT NULL
  AND makeup_deadline IN (
    SELECT year_month FROM study_periods
    WHERE end_date <= ?  -- as_of (오늘)
  )
RETURNING id, student_id, event_date, makeup_deadline;
```

**핵심 동작**:
1. `study_periods.end_date <= 오늘` 인 모든 교습기간을 찾고
2. 그 `year_month` 가 `makeup_deadline` 인 미보강 결석을 일괄 `makeup_expired` 전이
3. `study_periods` 미등록 월의 결석은 자연스럽게 제외됨 (서브쿼리에 매칭 안 됨)
4. RETURNING 으로 전이된 레코드 메타데이터 수집 → audit 로그 + UI 토스트용

**트랜잭션**: 단일 트랜잭션 내 UPDATE → audit insert (per row) → commit.

**audit**: `MakeupExpired` variant 신규 추가 (audit.rs).

---

### 트리거 3개소 통합 설계 (T4 작업)

| 트리거 | 위치 | 호출 방식 |
|--------|------|----------|
| 앱 시작 | `src-tauri/src/lib.rs` setup() 또는 startup 모듈 (DB 풀 생성 직후) | `expire_overdue_absences_impl(pool, None).await` — 결과는 startup 로그에 기록, UI는 첫 화면 로드 후 조회 |
| 출결 생성 | `attendance.rs::generate_attendances` 종료 직전 (PRD §4.5.1 4번째 동작) | 같은 IPC 함수 호출 후 `GenerateResult` 응답에 `expiration_report` 필드 추가 |
| 교습기간 등록 | `academic.rs` 교습기간 생성/수정 커맨드 종료 직전 | 같은 IPC 함수 호출 → 새로 등록된 month 의 deadline 확정 즉시 반영 |

**프론트엔드 통합** (T9 작업):
- 출결 생성 응답 → `expiration_report.transitioned_count > 0` 시 토스트
- 교습기간 등록 응답 → 동일하게 토스트
- 앱 시작 → 별도 IPC `get_recent_expiration_report` 또는 layout mount 시 polling (설계 시 PI-09 결정 따라 선택)

---

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint10/scope.md | [1회] | Session #2 추가 |

> T2는 설계 task — 코드 파일 수정 없음.

### 완료 기준 (이번 세션) — T2 AC

- ✅ PI-05~PI-09 사용자 결정 수렴
- ✅ scope.md 에 결정 사항 기록
- ✅ 운용 관행 교차 항목 0건 잔존 (A51 패턴)

### 세션 종료 조건

- ✅ 5건 PI 모두 사용자 결정 완료
- ✅ scope.md 갱신
- ✅ sprint10.md 에 T1' (V108 마이그레이션) 신규 task 추가 + Capacity 갱신
- ⬜ 단일 커밋 + 사용자 메모리 미러 동기화
- ⬜ 다음 세션(T1' → T3) 진입점 준비

### T1' 신규 작업 (PI-07 결정 반영)

| Task | 작업 | 예상 |
|------|------|------|
| **T1'** | V108 마이그레이션 — `makeup_attendances.status` CHECK 제약에서 `'makeup_absent'` 제거 | 0.5h |

**작업 내용**:
1. `src-tauri/migrations/108__cleanup_makeup_status_check.sql` 신규
2. CHECK 제약 단순화: `CHECK (status IN ('makeup_attended', 'makeup_absent'))` → `CHECK (status = 'makeup_attended')`
3. SQLite는 CHECK 제약 ALTER 미지원 → table rename + 새 테이블 생성 + INSERT SELECT 패턴
4. `.sqlx/` 오프라인 캐시 갱신 + 커밋
5. `cargo test` 통과 확인 (CHECK 제약 위반 테스트 없음 — 데이터 0건이므로 안전)

**AC**: V108 적용 후 `makeup_attendances.status` 단일 값 강제 + `cargo test` 통과

---

## Session #3 (T1' — V108 마이그레이션, 2026-05-26)

> Sprint 10 Session #3 — T1' (V108 적용).
> 예상 0.5h. PI-07 결정 반영.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1'** | V108 마이그레이션 — `makeup_attendances.status` CHECK 제약에서 `'makeup_absent'` 제거 | 0.5h |

### V108 구현 패턴

SQLite는 CHECK ALTER 미지원 → 다음 패턴 사용:
1. `PRAGMA foreign_keys=OFF;` (FK 참조 임시 해제 — V107이 `regular_attendances.makeup_attendance_id → makeup_attendances.id` 참조)
2. 새 테이블 `makeup_attendances_new` 생성 (CHECK 단순화)
3. `INSERT INTO ... SELECT * FROM makeup_attendances;`
4. `DROP TABLE makeup_attendances;`
5. `ALTER TABLE makeup_attendances_new RENAME TO makeup_attendances;`
6. 인덱스 재생성 (테이블 rename 시 인덱스는 따라가지만 명시적으로 보장)
7. `PRAGMA foreign_keys=ON;`
8. `PRAGMA foreign_key_check;` 검증

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/108__cleanup_makeup_status_check.sql | [신규] | V108 마이그레이션 |
| docs/sprint/sprint10/scope.md | [2회] | Session #3 추가 |

### 완료 기준

- ✅ V108 SQL 파일 작성 (`108__cleanup_makeup_status_check.sql`, 50 라인)
- ✅ `cargo test --manifest-path src-tauri/Cargo.toml` 251 passed / 0 failed (T1 동일, 회귀 없음 — 인메모리 DB가 V108까지 자동 적용)
- ✅ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- ✅ `.sqlx/` 오프라인 캐시 영향 없음 — makeup.rs는 `sqlx::query()` (런타임 매크로) 사용, compile-time `query!` 매크로 없음

### 세션 종료 조건

- ✅ T1' AC 통과
- ✅ 단일 커밋 (`1efd70f`)
- ✅ 다음 세션(T3 — 소멸 자동 전이 IPC) 진입점 준비

---

## Session #4 (T3 — 소멸 자동 전이 백엔드 IPC, 2026-05-26)

> Sprint 10 Session #4 — T3 (expiration.rs 신규 모듈 + 단위 테스트 6건+).
> 예상 4h. PI-05/PI-06 결정 반영.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T3** | 소멸 자동 전이 핵심 IPC + audit + 단위 테스트 | 4h |

### 구현 범위 (Session #2 설계 그대로)

1. 신규 모듈 `src-tauri/src/commands/expiration.rs`
   - `expire_overdue_absences_impl(pool, as_of: Option<NaiveDate>)` — `Local::now` 기본값 (PI-06)
   - `expire_overdue_absences()` IPC 핸들러
   - 응답 구조체 `ExpirationReport { transitioned_count, details }` + `ExpiredAbsenceDetail`
2. `audit.rs` — `MakeupExpired` variant 추가 (PRD §6.6 자가 진단/감사 로그)
3. `mod.rs` — `pub mod expiration;` 등록
4. `lib.rs` — `invoke_handler!` 등록
5. 단위 테스트 6건+:
   - 소멸기한 도래 + 미보강 → 전이 성공
   - 소멸기한 미도래(study_periods.end_date 미경과) → 전이 없음
   - 이미 `makeup_done` → 전이 대상 아님
   - 이미 `makeup_expired` → 중복 전이 없음
   - 교습기간 미등록 월(study_periods 행 없음) → 전이 보류 (PI-05 정책)
   - 복수 원생 batch 전이 + ExpirationReport.details 정확성

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/expiration.rs | [신규] | 모듈 신설 |
| src-tauri/src/commands/mod.rs | [1회] | `pub mod expiration;` |
| src-tauri/src/lib.rs | [1회] | invoke_handler 등록 |
| src-tauri/src/commands/audit.rs | [1회] | `MakeupExpired` variant |
| docs/sprint/sprint10/scope.md | [3회] | Session #4 추가 |

### 완료 기준 — T3 AC (sprint10.md L114-118)

- ✅ `expire_overdue_absences` 단위 테스트 **7건 통과** (계획 6건+ 충족)
- ✅ 트랜잭션 내 원자적 실행 (`pool.begin()` → RETURNING → `tx.commit()`)
- ✅ audit 로그 기록 (`MakeupExpired` variant, fire-and-forget per row)
- ✅ `cargo test` 258 passed (T1' 251 → +7 T3) / `cargo clippy` clean

### 발견된 이슈 + 해결

- 회귀 1건: `summary_aggregates_completed_makeup_minutes` 실패 (V108 CHECK 단순화로 `makeup_absent` INSERT 차단)
  - 해결: 테스트의 `makeup_absent` 시드 행 제거. 검증 의도(출석한 보강만 합산)는 동일 유지

### 세션 종료 조건

- ✅ T3 AC 통과
- ✅ 단일 커밋 (`616021d`)
- ✅ 다음 세션(T4 — 트리거 3개소 통합) 진입점 준비

---

## Session #5 (T4 — 트리거 3개소 통합, 2026-05-26)

> Sprint 10 Session #5 — T4 (소멸 자동 전이 트리거 통합).
> 예상 3h. PI-05 결정 반영.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T4** | 앱 시작 + 출결 생성 + 교습기간 등록 3개소에 `expire_overdue_absences_impl` 통합 | 3h |

### 설계 — 응답 동봉 패턴 (PI-09 토스트 알림 지원)

#### A. 앱 시작 트리거
- 위치: `src-tauri/src/startup.rs::app_startup_sequence` 의 `db::initialize` 직후
- 호출: `expire_overdue_absences_impl(pool, None).await`
- 응답: `StartupResult` 에 `expiration_report: ExpirationReport` 필드 추가
- 프론트엔드: startup IPC 응답에서 toast 표시

#### B. 출결 생성 트리거
- 위치: `attendance.rs::generate_impl` 의 `tx.commit()` 직후
- 호출: `expire_overdue_absences_impl(pool, None).await`
- 응답: `GenerateResult` 에 `expiration_report: ExpirationReport` 필드 추가

#### C. 교습기간 등록 트리거 (create/update/confirm)
- 위치: `academic.rs::create_study_period`, `update_study_period`, `confirm_study_period`
- 응답: 기존 `Result<StudyPeriod, String>` → `Result<StudyPeriodResult, String>` 변경
  - `StudyPeriodResult { study_period: StudyPeriod, expiration_report: ExpirationReport }` 신규 wrapper
- `delete_study_period` 는 트리거 제외 (반대 방향 — deadline 미확정 효과)

#### D. TS 래퍼 + 타입
- `src/types/expiration.ts` 신규 — `ExpirationReport`, `ExpiredAbsenceDetail`
- `src/types/attendance.ts` — `GenerateResult` 에 `expirationReport` 추가
- `src/types/academic.ts` — `StudyPeriodResult` wrapper 신규
- `src/lib/tauri/index.ts`:
  - `expireOverdueAbsences` 래퍼 신규
  - `createStudyPeriod` / `updateStudyPeriod` / `confirmStudyPeriod` 반환 타입 변경
  - `generateAttendances` 반환 타입 갱신 (GenerateResult 확장)

#### E. 호출처 컴포넌트 영향
- `src/app/attendance/page.tsx` — `generateMutation` 응답에서 `expirationReport` 확인 후 토스트
- `src/components/academic/StudyPeriodEditor.tsx` — 등록/수정/확정 응답에서 동일 처리

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/attendance.rs | [1회] | `GenerateResult.expiration_report` 추가 + `generate_impl` 트리거 |
| src-tauri/src/commands/academic.rs | [1회] | `StudyPeriodResult` wrapper + 3개 IPC 응답 변경 |
| src-tauri/src/startup.rs | [1회] | `StartupResult.expiration_report` + 호출 |
| src/lib/tauri/index.ts | [1회] | 4개 래퍼 시그니처 변경 + `expireOverdueAbsences` 신규 |
| src/types/expiration.ts | [신규] | `ExpirationReport`, `ExpiredAbsenceDetail` |
| src/types/attendance.ts | [1회] | `GenerateResult.expirationReport` 추가 |
| src/types/academic.ts | [1회] | `StudyPeriodResult` wrapper |
| src/app/attendance/page.tsx | [1회] | 토스트 |
| src/components/academic/StudyPeriodEditor.tsx | [1회] | 토스트 + 응답 타입 갱신 |
| docs/sprint/sprint10/scope.md | [4회] | Session #5 |

### 완료 기준 — T4 AC (sprint10.md L132-138)

- ✅ 앱 시작 → `expire_overdue_absences_impl` 호출 + StartupResult.expiration_report 동봉 (fail-soft)
- ✅ 출결 생성 → 트랜잭션 커밋 직후 호출 + GenerateResult.expiration_report 동봉
- ✅ 교습기간 등록(create/update/confirm) 3개 IPC → StudyPeriodResult wrapper 응답
- ✅ 단위 테스트: `generate_includes_expiration_report_when_deadline_reached` (응답 필드 존재 검증) — T3 7건과 합쳐 충분
- ✅ `cargo test` 259 passed (T3 258 → +1) / `cargo clippy` clean
- ✅ `pnpm lint` / `pnpm tsc --noEmit` / `pnpm build` clean

### 세션 종료 조건

- ✅ T4 AC 통과
- ✅ 단일 커밋 (`6b6cc47`)
- ✅ 다음 세션(T5 — 소멸 환원 IPC) 진입점 준비

---

## Session #6 (T5 폐기 결정, 2026-05-26)

> Sprint 10 Session #6 — T5 환원 기능 폐기 + 보강완료/소멸 시각 구분 확인.
> 예상 0.5h. 코드 변경 없음, 결정 기록 + 문서 갱신.

### 사용자 결정 (PI-10 대체)

> "보강기한 소멸되면 끝임. 출결관리에 결석이 보강완료된 것과 소멸된 것은 구분해 표현해 주는 것이 필요함."

→ **T5(소멸 → 결석 환원 IPC) + T9 환원 다이얼로그 완전 폐기**. PRD §4.5.3 AC-4.5-5 요건 해제 (사용자 운영 정책 결정).

→ 추가 요구: 출결 그리드에서 보강완료(`makeup_done`) vs 보강소멸(`makeup_expired`) 시각 구분.

### 보강완료/소멸 시각 구분 현황 (AttendanceGrid.tsx::statusCellClass)

| 상태 | 라벨 | 배경 | 비고 |
|------|------|------|------|
| `present` | ○ | 흰색 | 출석 |
| `absent` | 결석 | bg-red-100 (빨강) | 미보강 결석 |
| `makeup_done` | 결석 | bg-emerald-100 (초록) | 보강완료 (Sprint 9 J7) |
| `makeup_expired` | **소멸** | bg-gray-200 (회색) | 보강소멸 |

→ Sprint 9 J7에서 `absent`/`makeup_done` 라벨을 '결석'으로 통일하면서 배경색(red vs emerald)으로 구분. `makeup_expired`는 라벨 '소멸' + 회색 배경으로 별도 구분 — 사용자 요구 시각 구분 **이미 충족**.

→ 사용자 시각 검증 시점(T12 통합 검증 또는 별도 라운드)에 최종 확인.

### 폐기 영향 범위

| 영향 | 처리 |
|------|------|
| sprint10.md T5 (소멸 환원 IPC, 3h) | **폐기** — 작업 미수행 |
| sprint10.md T9 (소멸 환원 UI, 3h 중 일부) | 환원 다이얼로그 부분 폐기. 토스트 알림 부분만 유지 (이미 T4 attendance/page.tsx + StudyPeriodEditor에서 처리됨) |
| sprint10.md DoD "보강소멸 → 결석 환원 시 확인 다이얼로그 동작 (AC-4.5-5)" | **폐기** |
| Capacity 절감 | T5 3h + T9 환원 부분 약 1.5h = **약 4.5h** 절감 |

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint10.md | [2회] | T5/T9/DoD 폐기 마킹 |
| docs/sprint/sprint10/scope.md | [5회] | Session #6 추가 |

### 완료 기준

- ✅ T5/T9 폐기 결정 scope.md 기록
- ✅ sprint10.md DoD AC-4.5-5 항목 폐기 + Capacity 40h 로 갱신
- ✅ 단일 커밋 + 메모리 동기화
- ✅ 다음 세션(T6 — 퇴교 보강 처리 IPC) 진입점 준비

### 다음 세션 (T6) 미리보기

- PRD §4.5.9 퇴교 시 미사용 보강 처리 — 3가지 선택지
- `students.rs` 또는 `expiration.rs` 에 IPC 2종:
  - `get_pending_makeup_for_withdrawal(student_id)` — 미보강 결석 리스트 조회
  - `process_withdrawal_makeup(student_id, choice)` — 3가지 선택지 처리

---

## Session #7 (T6 — 퇴교 시 미사용 보강 처리 IPC, 2026-05-26)

> Sprint 10 Session #7 — T6 (퇴교 시 미보강 결석 일괄 처리, PRD §4.5.9).
> 예상 3h. PI-11/PI-12 사용자 결정 반영.

### 사용자 결정 (2026-05-26)

| ID | 결정 |
|----|------|
| **PI-11** | 모듈 = `expiration.rs` 에 추가 (소멸 도메인 일치성) |
| **PI-12** | `external_expire` 메모 = 결석별 `absence_memo` 일괄 저장 (결석 이력 §4.5.10 에서 직접 확인) |

### 설계

#### 응답 구조체

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalPendingMakeup {
    pub student_id: i64,
    pub remaining_minutes: i64,  // 잔여 보강필요시간
    pub absences: Vec<PendingAbsenceForWithdrawal>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAbsenceForWithdrawal {
    pub id: i64,
    pub event_date: String,
    pub class_minutes: i64,
    pub makeup_deadline: Option<String>,
}

/// 퇴교 처리 선택지.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum WithdrawalChoice {
    ImmediateExpire,                      // 전체 → makeup_expired
    ExternalExpire { memo: String },      // 전체 → makeup_expired + absence_memo 일괄 저장
    // defer_withdrawal 은 UI 에서 다이얼로그 닫기로 처리 — IPC 호출 없음
}
```

#### IPC 2종

1. `get_pending_makeup_for_withdrawal(student_id) -> WithdrawalPendingMakeup`
   - SQL: `SELECT id, event_date, class_minutes, makeup_deadline FROM regular_attendances WHERE student_id = ? AND status='absent' AND makeup_attendance_id IS NULL ORDER BY event_date`
   - 잔여 시간 계산: `SUM(class_minutes)`
   - 빈 리스트면 UI에서 다이얼로그 미표시

2. `process_withdrawal_makeup(student_id, choice, withdraw_date) -> ()`
   - 트랜잭션 내:
     - 미보강 결석 → `makeup_expired` 전이
     - `ExternalExpire { memo }` 인 경우 동일 트랜잭션에서 `UPDATE absence_memo`
     - 학생 `withdraw_date` 설정
   - audit:
     - `MakeupExpired` (전이된 결석 수만큼)
     - `StudentWithdrawn`

#### defer_withdrawal 처리
IPC 옵션에서 제외 — UI에서 다이얼로그 닫기 = 퇴교 미실행. 사용자가 보강 완료 후 다시 퇴교 흐름 진입.

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/expiration.rs | [2회] | 응답 struct 2종 + IPC 2종 + 단위 테스트 5건+ |
| src-tauri/src/lib.rs | [2회] | invoke_handler 2건 추가 |
| docs/sprint/sprint10/scope.md | [6회] | Session #7 추가 |

### 완료 기준 — T6 AC (sprint10.md L186-189)

- ✅ IPC 2종 등록 — `get_pending_makeup_for_withdrawal` + `process_withdrawal_makeup` (TS 래퍼는 T10 UI 진입 시 추가)
- ✅ 단위 테스트 **6건 통과** (계획 5건+ 충족):
  - withdrawal_lists_pending_absences_with_remaining_minutes — 정렬/잔여 시간 합
  - withdrawal_returns_empty_when_no_pending_absence — 빈 리스트
  - withdrawal_immediate_expire_transitions_all_and_sets_withdraw_date — 전체 전이 + 퇴교
  - withdrawal_external_expire_saves_memo_and_transitions_all — memo 일괄 저장 + 전이
  - withdrawal_zero_absences_still_sets_withdraw_date — 결석 0건도 퇴교 정상
  - withdrawal_rejects_missing_student — 미존재 학생 거부
- ✅ `cargo test` 265 passed (T4 259 → +6) / `cargo clippy` clean

### 세션 종료 조건

- ✅ T6 AC 통과
- ✅ 단일 커밋 (`6209d00`)
- ✅ 다음 세션(T7) 진입점 준비

---

## Session #8 (T7 — 선행 수업 검증, 2026-05-26)

> Sprint 10 Session #8 — T7 (PRD §4.2.3 선행 수업, 축소된 범위).
> 예상 2h. PI-08 결정 반영 — 별도 IPC 신설 없이 기존 토글 + 보강 흐름 활용.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T7** | 선행 수업 시나리오 검증 단위 테스트 + scope.md 흐름 문서화 | 2h |

### 코드 분석 결과 — 백엔드 이미 PRD §4.2.3 지원

`create_makeup_with_absences_impl` 검증 항목:
1. 보강 가능 일자 (케이스 A/B)
2. 학생 존재 + 입퇴교 범위 (event_date 기준)
3. ~~정규 수업 요일 차단~~ (Sprint 9 T10 폐기)
4. 결석 유효성 — 학생 일치 + status='absent' + 미매칭

**→ 보강 event_date vs 결석 absence.event_date 순서 검증 없음** (의도). 즉 백엔드는 미래 결석을 현재 보강이 충당하는 것을 자연스럽게 허용. PRD §4.2.3 선행 수업 시나리오와 정합.

> UI 측 `MakeupRegisterDialog::filteredPending` 은 `a.eventDate < eventDate` 필터를 적용 — 현재 보강일 이전 결석만 표시. 미래 결석 매칭은 UI 차원에서 차단됨. 본 sprint 에서는 UI 필터 그대로 유지 (Sprint 9 검증 통과). 사용자가 PRD §4.2.3 실제 운영을 시작할 때 시각 검증으로 확인 후 별도 task 에서 UI 필터 완화 검토.

### 선행 수업 운영 흐름 (PRD §4.2.3 + PI-08 정합)

```
[Step 1] 월 초: 원장이 generate_attendances 호출
   → regular_attendances 일괄 INSERT (status='present')

[Step 2] 학부모 사전 통보로 미래 결석 확정 (예: 6/15)
   → 출결 그리드에서 6/15 셀 토글 (present → absent)
   → makeup_deadline 자동 설정 (2026-07)

[Step 3] 선행 등원일(예: 6/10) 보강 등록
   → 백엔드 create_makeup_with_absences: event_date=6/10, absence_ids=[6/15 결석]
   → 검증 통과 (백엔드는 보강일 < 결석일 시나리오 허용)
   → 6/15 결석 status='makeup_done' 전이

[제약] Step 3 의 UI 진입 — 현재 MakeupRegisterDialog 는 보강일 이전 결석만 표시.
       사용자가 운영 시작 시 시각 검증으로 확인 후 필터 완화 검토.
```

### 출결 생성 충돌 방지 (R69)

- `generate_attendances` 는 `check_attendance_exists` 로 같은 월에 이미 출결이 있으면 거부 (이미 구현됨, `attendance.rs::generate_impl` L79-84)
- 사전 등록 결석이 있는 경우라도 출결 생성은 거부 → 운영 흐름 보호 (월 초 generate 표준)

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/makeup.rs | [1회] | 선행 수업 단위 테스트 1건 추가 |
| docs/sprint/sprint10/scope.md | [7회] | Session #8 추가 |

### 완료 기준 — T7 AC (sprint10.md L209-212, 축소)

- ✅ 단위 테스트 1건 — `create_makeup_supports_future_absence_for_advance_class` (미래 결석을 현재 보강이 충당 성공)
- ✅ scope.md 에 선행 수업 운영 흐름 문서화 (PRD §4.2.3 정합)
- ✅ `cargo test` 266 passed (T6 265 → +1) / `cargo clippy` clean

### 세션 종료 조건

- ✅ T7 AC 통과
- ✅ 단일 커밋 (`840e9c7`)
- ✅ 다음 세션(T8) 진입점 준비

---

## Session #9 (T8 — 캘린더 ADR + 백엔드 집계 IPC, 2026-05-26)

> Sprint 10 Session #9 — T8 (PI-03 캘린더 라이브러리 결정 + 백엔드 집계 IPC 2종).
> 예상 4h. skill: brainstorming.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T8-A** | ADR 작성 + PI-03 사용자 결정 (FullCalendar vs React Big Calendar) | 1h |
| **T8-B** | 백엔드 집계 IPC 2종 + 단위 테스트 4건+ | 3h |

### Weighted Decision Matrix — PI-03

> SmartHB 컨텍스트: Tauri 2 + Next.js 15 static export, 50대 운영자 1인, ≤50명 원생, 외부 네트워크 없음, React 19.

| 기준 | 가중치 | FullCalendar | 점수 | React Big Calendar | 점수 |
|------|-------|--------------|------|---------------------|------|
| 라이선스 (MVP 범위) | 0.20 | MIT (premium 기능은 상용, 우리는 MIT만 사용) | 4 | MIT 완전 | 5 |
| 번들 크기 (50대 PC 로딩) | 0.15 | ~150KB+ 큼 | 2 | ~80KB 중간 | 4 |
| 일/주/월 뷰 완성도 | 0.20 | 표준, 매우 강력 | 5 | 보통, 커스터마이징 필요 | 3 |
| 커스텀 렌더러 (원생 + 시간 셀) | 0.15 | eventContent prop 강력 | 5 | components prop 자유도 | 4 |
| TypeScript 지원 | 0.05 | 공식 @types 풍부 | 5 | 공식 .d.ts 보통 | 4 |
| Next.js static export 호환 | 0.10 | `'use client'` + dynamic import 필요 | 3 | `'use client'` 충분 | 4 |
| 한국어 i18n / 운영 안정성 | 0.05 | 로케일 패키지 풍부, 한국 사용 사례 많음 | 5 | 로케일 자유 설정 | 3 |
| React 19 호환 | 0.05 | `@fullcalendar/react` 호환 확인 필요 | 3 | 호환 보고됨 | 4 |
| 50대 친화 검증 사례 | 0.05 | 풍부한 community + 한국어 가이드 | 5 | 보통 | 3 |
| **총점** |        |              | **3.95** |                | **3.85** |

→ **차이 0.10 — 통계적 동등 수준**. 2단계 SWOT 으로 결정.

### SWOT — FullCalendar

- **Strengths**: 시각적 완성도, 풍부한 한국어 자료, eventContent 커스터마이징 강력
- **Weaknesses**: 번들 크기, premium 기능 분리 (시간그리드 일부 premium — MVP 범위는 무료)
- **Opportunities**: 사용자(50대)에게 친숙한 UI 패턴 (Google Calendar 류)
- **Threats**: premium 기능 사용 시 라이선스 비용 (MVP 범위 외)

### SWOT — React Big Calendar

- **Strengths**: 가벼움, MIT 전체, 컴포넌트 자유도 높음
- **Weaknesses**: 커뮤니티 작아 한국어 자료 부족, 시간 그리드 디자인 보강 필요
- **Opportunities**: 완전한 MIT — 미래 확장 시 라이선스 부담 없음
- **Threats**: 커스터마이징 부담 — 50대 친화 UI 직접 구현 필요

### 추천

**FullCalendar** — 50대 운영자 친화 + 시각 완성도가 핵심. premium 기능 미사용으로 라이선스 부담 없음. 번들 크기는 한 번 로딩이라 50명 미만 규모에서 영향 미미. 사용자 결정 필요.

### T8-B 백엔드 집계 IPC 설계 (사용자 결정 후 작업)

1. **`get_calendar_data(year_month)`** — 캘린더 뷰용
   - 응답: 일자별 시간대별 수업 원생 목록
   - 정규 수업 + 보강 수업 모두 포함, 시작/종료 시간 + 원생명
   - AC-4.6-1: 시간대별 인원 = 시작 원생 + 진행 중 원생 합산 (백엔드에서 시간 범위 겹침 계산)

2. **`get_makeup_management_data(year_month)`** — 보강 관리 뷰용
   - 응답: 보강 필요 원생 리스트 (잔여 보강필요시간 + 소멸기한 + 소멸 임박 플래그)
   - 정렬: makeup_deadline ASC, event_date ASC (소멸기한 임박 순)
   - 소멸 임박: 교습기간 종료일 - 7일 이내

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/arch/adr-001-calendar-library.md | [신규] | ADR (PI-03 결정 기록) |
| src-tauri/src/commands/expiration.rs 또는 신규 calendar.rs | [신규/2회] | 집계 IPC 2종 — 위치 PI-03 결정 후 |
| src-tauri/src/lib.rs | [3회] | invoke_handler 2건 추가 |
| docs/sprint/sprint10/scope.md | [8회] | Session #9 추가 |

### 완료 기준 — T8 AC (sprint10.md L234-237)

- ✅ ADR 문서 작성 — `docs/arch/adr-006-calendar-library.md` (FullCalendar 채택, 사용자 결정 2026-05-26)
- ✅ 캘린더 데이터 집계 IPC — `get_calendar_data` + 단위 테스트 2건 (그룹화 + year_month 필터)
- ✅ 보강 관리 데이터 IPC — `get_makeup_management_data` + 단위 테스트 4건 (정렬, 임박 판정, 교습기간 미등록, 보강완료/소멸 제외)
- ✅ `cargo test` 272 passed (T7 266 → +6 calendar) / `cargo clippy` clean
- ⚠️ `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` 병렬 실행 시 가끔 실패 (단독 실행 통과) — flaky, calendar 변경과 무관. carry-over

### 세션 종료 조건

- ✅ T8 AC 통과
- ✅ 단일 커밋 (`21f8719`)
- ✅ 다음 세션(T9) 진입점 준비

---

## Session #10 (T9 — 소멸 알림 UI 잔여, 2026-05-26)

> Sprint 10 Session #10 — T9 (앱 시작 시 expiration_report 토스트).
> 예상 1.5h. PI-09 결정 적용 — 건수 > 0 시만 토스트.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T9** | 앱 시작 startup 응답의 expiration_report 를 토스트로 표시 | 1.5h |

### 설계

1. **types/index.ts** — `StartupResult` 에 `expirationReport` 추가 (백엔드 응답 정합)
2. **stores/session-store** — `dismissedExpirationNotice` 플래그 추가 (한 번 닫으면 같은 세션에서 재표시 안 함)
3. **루트 페이지** (`src/app/page.tsx`) — `unlocked && lastStartup` 시 토스트 영역 표시
   - 기존 attendance/page.tsx의 amber 배너 패턴 재사용 (일관성)
   - 닫기 버튼 → `dismissExpirationNotice()` 호출

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/types/index.ts | [1회] | StartupResult.expirationReport 추가 |
| src/stores/session-store.ts | [1회] | dismissExpirationNotice 액션 |
| src/app/page.tsx | [1회] | 토스트 영역 + 닫기 |
| docs/sprint/sprint10/scope.md | [9회] | Session #10 추가 |

### 완료 기준 — T9 AC (축소된 범위)

- ✅ 앱 시작 (unlock) 직후 메인 페이지에서 expiration_report 토스트 표시 (amber 배너)
- ✅ 건수 > 0 일 때만 표시 (PI-09 일치)
- ✅ 닫기 버튼 → `expirationNoticeDismissed` 플래그로 같은 세션 재표시 차단
- ✅ `pnpm lint` clean / `pnpm tsc --noEmit` clean

### 세션 종료 조건

- ✅ T9 AC 통과
- ✅ 단일 커밋 (`b7b6fcb`)
- ✅ 다음 세션(T10) 진입점 준비

---

## Session #11 (T10 — 퇴교 보강 UI 다이얼로그, 2026-05-26)

> Sprint 10 Session #11 — T10 (PRD §4.5.9 퇴교 처리 다이얼로그).
> 예상 3h. T6 백엔드 IPC 활용 + 3가지 선택지 + defer 는 UI 처리.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T10** | 퇴교 보강 처리 UI — TS 타입 + 래퍼 + WithdrawalMakeupDialog + 기존 흐름 통합 | 3h |

### 현재 퇴교 흐름 (Sprint 4 T8)

`src/app/students/edit/page.tsx`:
1. "퇴교 처리" 버튼 클릭 → AlertDialog (퇴교일자 선택)
2. "확정" → `withdrawStudent(id, withdrawDate)` 직접 호출
3. 안내: "보강 잔여 처리는 Phase 3 에서 별도 제공" (← 본 T10 으로 채워짐)

### T10 구현 방향

기존 AlertDialog 의 "확정" 클릭 시 흐름 변경:
1. `getPendingMakeupForWithdrawal(studentId)` 호출 (잔여 보강 조회)
2. `absences.length === 0` → 기존 `withdrawStudent` 직접 호출 (no-change 경로)
3. 잔여 보강 있음 → 새 `WithdrawalMakeupDialog` 전환:
   - 표시: 원생명, 잔여 보강필요시간(분→시간), 미보강 결석 일자 리스트
   - 3가지 선택지:
     - **즉시 소멸**: `ImmediateExpire` → `processWithdrawalMakeup`
     - **보강 후 퇴교**: 다이얼로그 닫기 (IPC 호출 없음, PI-08 결정)
     - **외부 처리 후 소멸**: memo textarea 입력 → `ExternalExpire { memo }` → `processWithdrawalMakeup`
4. 성공 시 TanStack Query 무효화 + `/students` 이동

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/types/withdrawal.ts | [신규] | WithdrawalChoice, WithdrawalPendingMakeup, PendingAbsenceForWithdrawal |
| src/lib/tauri/index.ts | [1회] | getPendingMakeupForWithdrawal + processWithdrawalMakeup 래퍼 |
| src/components/students/WithdrawalMakeupDialog.tsx | [신규] | 신규 다이얼로그 |
| src/app/students/edit/page.tsx | [1회] | handleWithdrawConfirmed 흐름 변경 + 보강 보유 분기 |
| docs/sprint/sprint10/scope.md | [10회 ⚠️] | Session #11 추가 |

⚠️ scope.md 10회 도달 — loop-detection 스킬 체크: 본 sprint 의 정상적인 다단계 진행 (Sprint 9 의 12 sessions 보다 적음). 동일 코드 파일 반복 수정 아님 — 정상 진행으로 판단.

### 완료 기준 — T10 AC (sprint10.md L273-275)

- ✅ 퇴교 시 `getPendingMakeupForWithdrawal` 검증 → 결석 보유 시만 다이얼로그 mount
- ✅ 3가지 선택지: 즉시 소멸 (ImmediateExpire IPC) / 보강 후 퇴교 (다이얼로그 닫기, PI-08) / 외부 처리 (ExternalExpire memo 입력)
- ✅ 결석 0건 원생 → 기존 `withdrawStudent` 직접 호출 + `/students` 이동
- ✅ `pnpm lint` clean / `pnpm tsc --noEmit` clean

### 세션 종료 조건

- ✅ T10 AC 통과
- ✅ 단일 커밋 (`7de4dbb`)
- ✅ 다음 세션(T11 — 캘린더 뷰 UI) 진입점 준비

---

## Session #12 (T11 — 수업 관리 캘린더 뷰, 2026-05-27)

> Sprint 10 Session #12 — T11 (PRD §4.6.1~4.6.3 캘린더 뷰).
> 예상 6h. T8 ADR-006(FullCalendar) + 백엔드 집계 IPC 2종 활용.

### 이번 세션의 Task

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T11** | 수업 관리 캘린더 뷰 (일/주/월) + 원생 상세 팝업 + 보강 관리 뷰 | 6h |

### 사용자 결정 (2026-05-27, T11 진입)

| ID | 질문 | ✅ 결정 |
|----|------|---------|
| **메뉴 배치** | 캘린더 뷰 라우트/메뉴 | **'수업 관리'(`/schedules`) 메뉴 활성화** — 일/주/월 + 보강관리는 내부 탭. 메뉴 추가 없음, PRD §4.6 명칭 일치 |
| **PI-04** | 보강데이 일괄 등록 진입점 | **진입점 없음 — 이동 버튼만** — 보강관리 뷰는 소멸 임박 순 목록 + 행별 '출결관리 이동' 버튼. 실제 보강 등록은 기존 출결 그리드 흐름 (Sprint 9 J7 폐기 일관) |

### 설계

#### 라이브러리 (ADR-006 FullCalendar)
- `@fullcalendar/react` + `daygrid` + `timegrid` + `interaction` + `core` (^6.x, React 19 호환 버전)
- `dynamic(() => import(...), { ssr: false })` 강제 (R67, static export 호환)
- 한국어 로케일 `@fullcalendar/core/locales/ko`

#### 라우트 구조 — `/schedules`
- 탭 2개: **캘린더**(일/주/월) / **보강 관리**
- 캘린더 탭: FullCalendar — `dayGridMonth` / `timeGridWeek` / `timeGridDay` 전환
  - 정규 수업 = `start_time` 있는 이벤트 (start~end), 보강 = `start_time` 없음 → allDay 이벤트
  - `eventContent` 커스텀: 원생명 + 시간
  - 이벤트 클릭 → 원생 상세 팝업 (StudentDetailPopup)
- 보강 관리 탭: `getMakeupManagementData` 목록
  - 소멸기한 임박 순 정렬 (백엔드 정렬 그대로)
  - `isImminent` 행 강조 (red/amber 배경 + 아이콘)
  - 각 행 "출결관리 이동" 버튼 → `/attendance` 라우팅 (PI-04 결정)

#### 원생 상세 팝업 (§4.6.2)
- 이벤트 클릭 → 원생 ID + 이벤트 메타 → 팝업
- 표시: 이름, 정규/보강 구분, 시간(시작·class_minutes→시간), 해당 월 요약(`getAttendanceSummary`)
- "출결/보강관리 이동" 버튼 → `/attendance`

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| package.json | [신규 의존성] | FullCalendar 5종 (ADR-006 사전 승인) |
| src/types/calendar.ts | [신규] | CalendarMonth/Day/Session, MakeupManagementStudent |
| src/lib/tauri/index.ts | [1회] | getCalendarData + getMakeupManagementData 래퍼 |
| src/lib/menu-config.ts | [1회] | '수업 관리' disabledHint 제거 (활성화) |
| src/app/schedules/page.tsx | [신규] | 캘린더 뷰 페이지 + 탭 |
| src/components/schedules/ClassCalendar.tsx | [신규] | FullCalendar 래퍼 (dynamic) |
| src/components/schedules/StudentDetailPopup.tsx | [신규] | 원생 상세 팝업 |
| src/components/schedules/MakeupManagementView.tsx | [신규] | 보강 관리 뷰 |
| docs/sprint/sprint10/scope.md | [11회 ⚠️] | Session #12 |

> ⚠️ scope.md 11회 — 정상적 다단계 sprint 진행 (세션당 1회). 동일 코드 파일 반복 수정 아님 — loop 아님.

### 완료 기준 — T11 AC (sprint10.md L314-318)

- ✅ AC-4.6-1: 시간대별 인원수 — timeGrid 주/일 뷰에서 동시간대 이벤트 겹침 시각화 (시각 검증 T12에서 최종 확인)
- ✅ AC-4.6-2: 소멸 임박 데이터 시각 식별 — 보강관리 뷰 `isImminent` 행 amber 배경 + '⚠ 소멸 임박' 배지
- ✅ 일/주/월 뷰 전환 — FullCalendar headerToolbar (dayGridMonth/timeGridWeek/timeGridDay) + datesSet → 월 변경 시 refetch
- ✅ 원생 상세 팝업 → 출결관리 이동 (StudentDetailPopup, `router.push('/attendance')`)
- ✅ `pnpm tsc --noEmit` clean / `pnpm lint` clean / `pnpm build` (static export 16/16 + Exporting 3/3) clean — R67 검증 통과

### 발견된 이슈

(없음 — FullCalendar 6.1.20 + React 19 + Next 15 static export 빌드 정상. dynamic ssr:false 로 코드 분할됨)

### 세션 종료 조건

- ✅ T11 AC 통과 (자동 검증 3종 + AC 4항목)
- ✅ 단일 커밋 (`2d8fdb3`)
- ✅ 다음 세션(T12 — 통합 검증) 진입점 준비

---

## Session #13 (T12 — 통합 검증 + 자동 검증, 2026-05-27)

> Sprint 10 Session #13 — T12 (Sprint 10 마지막 Task). 코드 변경 없음 — 검증 + 문서 갱신.

### 자동 검증 결과 (7항목)

| # | 항목 | 결과 |
|---|------|------|
| 1 | `cargo test --lib` (cipher off) | ✅ **272 passed / 0 failed** / 3 ignored |
| 2 | `cargo test --lib --features cipher` (cipher on) | ⚠️ 로컬 빌드 불가 — Strawberry Perl 미설치 (vendored OpenSSL 빌드 실패). **CI(`ci.yml`/`deploy.yml`)에서 검증**. T11 Rust 변경 0건이라 cipher 영향 없음 |
| 3 | `cargo clippy --lib -- -D warnings` (cipher off) | ✅ clean |
| 4 | `cargo clippy` (cipher on) | ⚠️ #2 와 동일 환경 제약 — CI 위임 |
| 5 | `pnpm lint` | ✅ clean |
| 6 | `pnpm tsc --noEmit` | ✅ clean |
| 7 | `pnpm build` (static export) | ✅ 16/16 static + Exporting 3/3 — R67 FullCalendar+static export 호환 확정 |

### 마이그레이션 self-check (A39)

| 계획 (scope/sprint10.md) | 실제 migrations | 일치 |
|--------------------------|-----------------|------|
| V108 makeup_status CHECK 정리 (T1') | `108__cleanup_makeup_status_check.sql` | ✅ |
| (T3/T6/T8 추가 마이그레이션 없음 — 기존 스키마 활용) | 신규 없음 | ✅ |

→ 1:1 일치. 누락/잉여 0건.

### 통합 시나리오 검증

| 시나리오 | 검증 방식 | 결과 |
|----------|----------|------|
| 결석 → 소멸기한 도래 → 자동 전이 | 단위 테스트 (expiration.rs 7건) + 트리거 통합 (T4) | ✅ |
| 퇴교 보강 처리 3선택지 | 단위 테스트 (expiration.rs 6건) + UI (T10) | ✅ |
| 선행 수업: 미래 결석 → 현재 보강 매칭 | 단위 테스트 (makeup.rs 1건) | ✅ |
| 캘린더 뷰: 일/주/월 + 원생 팝업 + 보강관리 | build 통과 + T11 구현 | ✅ (사용자 시각 검증 대기) |
| 보강완료/소멸 시각 구분 | Sprint 8 구현 (emerald/gray) | ✅ (사용자 시각 검증 대기) |

### sprint-review 산출물 경로 (A40)

- 코드 리뷰: 본 sprint 변경 전반 (특히 expiration.rs, calendar.rs, T11 프론트 4종)
- 테스트 리포트: `docs/test-reports/sprint10-*.md`
- 회고: `docs/sprint-retrospectives/sprint10-retrospective.md`

### 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint10.md | [2회] | DoD ✅ 전환 + T12/Capacity 마킹 |
| docs/sprint/sprint10/scope.md | [12회] | Session #13 |

### 완료 기준 — T12 AC

- ✅ 자동 검증 cipher off 전수 통과 + cipher on CI 위임 사유 명시
- ✅ 마이그레이션 self-check 1:1
- ✅ 통합 시나리오 4개 (단위 테스트/빌드 기반) — 사용자 시각 검증은 sprint-review 단계
- ✅ sprint-review 산출물 경로 명시

### 세션 종료 조건

- ✅ T12 AC 통과
- ⬜ 단일 커밋
- ✅ **Sprint 10 전 Task 완료 (T1~T12, T5 폐기)** → sprint-close 진입 준비

### ⚠️ sprint-review 인계 사항

1. **cipher on 검증**: 로컬 Strawberry Perl 미설치로 cipher feature 빌드/테스트 불가. CI 에서 반드시 확인 필요. T11 은 프론트 전용이라 Rust 회귀 없음.
2. **사용자 시각 검증 대기**: 캘린더 뷰(일/주/월 전환, 원생 팝업, 보강관리 강조) + 보강완료/소멸 색상 구분 — 실제 데이터로 시각 검증 필요.
3. **FullCalendar 신규 의존성**: 5종 (~150KB, dynamic 코드분할). ADR-006 사전 승인. React 19 + static export 빌드 정상 확인됨.
4. **carry-over (Session #9)**: `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` 병렬 실행 시 가끔 flaky (단독 통과) — calendar 무관.

---

## Session #14 (cipher feature 로컬 검증 환경 구축 + 테스트 게이트 정합, 2026-05-27)

> 사용자 요청: "cipher on 로컬 검증 불가 문제를 해결하고 다시 시도하라."

### 근본 원인 (2단계)

1. **환경**: 이 PC 에 Strawberry Perl 미설치 → `--features cipher` 의 vendored OpenSSL 소스 빌드 실패 (`Locale::Maketext::Simple` 누락). Bash 셸은 git-bash msys perl 을 잡아 더 악화.
2. **코드 (잠재 결함, codebase 전반)**: `db::test_pool_in_memory()` 는 `#[cfg(all(test, not(feature = "cipher")))]` 게이트 (인메모리 DB 는 SQLCipher 적용 불가 — db.rs 설계 주석). 이 헬퍼를 쓰는 테스트 모듈 11개 중 **attendance/makeup 만 게이트**되어 있고 나머지 8개(academic/audit/students/schedules/fees/codes/**calendar**/**expiration**)는 `#[cfg(test)]` 만 → `cargo test --features cipher` 컴파일 불가. CI 는 cipher 로 **빌드만**(`tauri build --features cipher`) 하고 테스트는 cipher-off 라 미발견.

### 해결

1. **Strawberry Perl 설치** — `winget install StrawberryPerl.StrawberryPerl`. PowerShell 에서 `C:\Strawberry\{c\bin,perl\bin}` 를 PATH 선두에 두고 cargo 실행.
2. **테스트 게이트 정합** — un-gated 8개 모듈의 `mod tests` 를 `#[cfg(all(test, not(feature = "cipher")))]` 로 통일 (attendance/makeup 패턴). 테스트 전용 cfg 변경이라 cipher-off 동작 불변.

### 검증 결과 (Strawberry Perl PATH, PowerShell)

| 항목 | 결과 |
|------|------|
| `cargo build --features cipher` | ✅ Finished (라이브러리+바이너리, OpenSSL+SQLCipher 통합 — CI/배포 동일 경로) |
| `cargo test --lib --features cipher` | ✅ **116 passed** / 1 flaky / 3 ignored |
| `cargo clippy --lib --features cipher -- -D warnings` | ✅ Finished clean (게이트아웃 dead_code 경고 0) |
| `cargo test --lib` (cipher off) | ✅ **271 passed** / 1 flaky / 3 ignored (총 272 보존, 누락 0) |
| flaky 단독 실행 | ✅ `ensure_cache_loaded_fast_path_is_concurrent_safe` 단독 통과 (5.27s) — 회귀 아님 |

### 수정 파일 (8개, 테스트 cfg only)

| 파일 | 변경 |
|------|------|
| calendar.rs, expiration.rs | Sprint 10 추가분 — `mod tests` cipher 게이트 (in-scope) |
| academic.rs, audit.rs, students.rs, schedules.rs, fees.rs, codes.rs | 기존 모듈 — 동일 게이트로 정합 (cipher-test 컴파일 차단 해소, 사용자 요청 범위) |

### 잔여 (sprint-review 인계 갱신)

- **cipher on 은 이제 로컬 검증 가능** (Strawberry Perl 설치 완료). 이전 인계 1번 항목 해소.
- flaky 동시성 테스트는 별도 carry-over 로 유지 (cipher 무관).

---

## Session #15 (T11 시각 검증 — V108 마이그레이션 FK 실패 수정, 2026-05-27)

> `pnpm tauri:dev` 로 실제 앱 기동 시각 검증 중 발견. 실데이터 DB 에서 앱 시작 실패:
> "설정 정보를 불러오는 중 오류 / 마이그레이션 실행 실패: code 787 FOREIGN KEY constraint failed".

### 근본 원인 (V108 — T1' 작성)

V108 은 makeup_attendances 를 재생성(CHECK 단순화)하는데, `regular_attendances.makeup_attendance_id
→ makeup_attendances.id` 자식 FK 가 있다. 앱 연결은 `foreign_keys = ON`(db.rs:109) + sqlx 가
마이그레이션을 트랜잭션으로 감쌈:
- `PRAGMA foreign_keys = OFF` 는 트랜잭션 내부에서 무시 (SQLite 공식 재구성은 BEGIN 밖 OFF 요구).
- `PRAGMA defer_foreign_keys = ON` 도 실패: DROP 암묵적 DELETE 가 deferred 카운터 +1 하나,
  부모 행을 makeup_attendances_new(다른 이름)에 INSERT 한 시점엔 감소 안 되고 RENAME 으로도
  감소 안 됨 → COMMIT 시 카운터>0 → 787. (`foreign_key_check` 는 0건이지만 카운터 잔존)
- 빈 인메모리 테스트는 자식 행이 없어 통과 → 잠재 결함이 단위 테스트를 통과했던 것.

### 해결 (NULL-복원 재구성)

1) 자식 FK 값(ra_id→mk_id)을 TEMP 테이블 보존 + NULL → 2) 부모 재구성(dangling 없음) →
3) 보존값으로 복원. foreign_keys ON + 트랜잭션 내부 전 구간 정합. Perl DBD::SQLite 로 실데이터
재현·검증 후 적용.

### 번호 재검토 → V108 유지 (재번호 불필요로 정정)

- 초기엔 "108 < 적용된 302 → 순서 역행" 으로 보고 V303 재번호 진행했으나, **WAL 파일을 빼고 DB 를
  복사해 오판**한 것. 실제로는 NULL-복원 수정본이 **version 108 로 이미 정상 적용**(success=1,
  CHECK 단순화 반영, FK 0, 보강 링크 ra9→mk2 보존)됨 — sqlx 0.8 은 순서 역행 pending 도 적용.
- 303 재번호는 "DB엔 108, 파일엔 303" 충돌(`108 previously applied but missing`)을 유발 → **108 로
  되돌림**. 적용된 체크섬과 파일 내용 일치 → 앱 정상 시작 확인.

### 검증 결과 (실데이터 DB)

| 항목 | 결과 |
|------|------|
| 앱 시작 (잠금 해제) | ✅ db_init=17ms, 마이그레이션 오류 없음 |
| 라우트 로드 | ✅ `/`, `/schedules`, `/students`, `/academic`, `/attendance` 모두 200 |
| `_sqlx_migrations` | ✅ v108 success=1 (CHECK `status='makeup_attended'` 반영) |
| FK 무결성 / 데이터 | ✅ `foreign_key_check` 0건, 보강 링크 ra9→mk2 보존 |
| `cargo test --lib` (cipher off) | ✅ 272 passed / 0 failed |

### 교훈 (메모리화)

- **SQLite WAL DB 복사 시 `-wal`/`-shm` 동반 또는 체크포인트 후 복사** — 안 하면 낡은 스냅샷.
- **sqlx 트랜잭션 내 테이블 재구성 + 자식 FK** → `defer_foreign_keys` 불가, NULL-복원 패턴 사용.
- **빈 인메모리 테스트는 FK 데이터 경로를 못 잡음** → 마이그레이션은 시드 데이터 있는 실DB 시각 검증 필요.

### 수정 파일

| 파일 | 변경 |
|------|------|
| src-tauri/migrations/108__cleanup_makeup_status_check.sql | NULL-복원 재구성으로 FK 787 해소 (번호 108 유지) |

---

## Session #16 (T11 시각 검증 완료, 2026-05-28)

> 1차~7차 시각 검증을 거치며 캘린더 UI 다듬기 — 사용자 "시각검수 완료" 확정 (2026-05-28).

### 시각 검증 라운드 요약 (반영 결과)

| 라운드 | 주요 변경 |
|--------|-----------|
| 1차 (14건) | 월요일 시작 / 창 높이 / 토일·공휴일 색·학사일정 표시 / 오늘 / 툴바 배치 / 년월 클릭 date picker / 뷰 버튼 색·너비 / 월셀 인원수+툴팁 / 30분 라인 제거 / 주·일 시간대 원생명 / 이름 클릭→출결관리 / 보강관리 이동 / 보강관리 필터 |
| 2차 (8건) | 보강관리 원생 검색 위치 부적합 → 제거(재원중 체크 유지) · 달 이동 튕김 → `keepPreviousData` · 학사일정 바→텍스트 · 인원수 위치/폰트 · 툴팁 '오후 4시' 표기 · 종일 행 제거 · 상단 디자인 |
| 3차 (6건) + 결정적 버그 | 오늘 삭제 / 커스텀 툴바 / 년월 클릭 datepicker 위치 / 셀 배경 amber·gray / 학사일정 좌측 상단 / **주·일 원생명 미표시 → 시작시간 "HH:MM:SS" 초 중복 datetime 무효 버그 수정** |
| 4차 (5건) | 교습기간 amber 배경 (study period 조회) / 툴바 중앙정렬 / 주·일 → 오늘 / 학사일정 allDay 중앙정렬 행 |
| 5차 (3건) | 셀 배경 정밀화 (보강데이 우선·정규off·토일) / 단원평가 응시일 기간 확장 / 학사일정 3중 중복 → dayHeader 통합 (날짜/코드/총 N명 수업) |
| 6차 (3건) | 토·일 볼드 제거 / 시간 열 폰트 12px + 'am./pm.' / 비헤더 셀 학사·인원 표기 제거 (dayCellContent 월 전용 게이트) |
| 7차 (다회) | 오늘 셀 민트(#DCEFD0) / 일 보기 셀 배경 없음·오늘도 transparent / 일 수업 블록 배경/외곽선 투명 / 일 원생명 파랑 볼드 + 중앙정렬 / 월 인원수 정중앙(dayCellDidMount 직접 주입) 28px 블랙 노볼드 + 'N일' 표기 / 학사 이전·다음 버튼 통일 / 학사일정 텍스트 한글 1자 간격 + pt-1 / 월 셀 hover outline / 주·일 이벤트 hover outline |

### 발견·수정된 핵심 결함 (시각 검증 가치)
1. **V108 마이그레이션 FK 787** — 실데이터 DB 자식 FK 위반 (Session #15) → NULL-복원 패턴
2. **주·일 원생명 미표시** — `T${startTime}:00` 이중 콜론으로 datetime 무효 (3차)
3. **달 이동 시 캘린더 초기 날짜로 튕김** — refetch 중 `data` undefined → remount 루프 → `keepPreviousData`

### 최종 상태
- 모든 T11 follow-up 라운드 사용자 승인
- 자동 검증(cargo test/clippy/lint/tsc/build) 통과 유지
- 작업 트리 클린, 모든 변경 커밋 완료

### 다음 단계
- `sprint-close` 진입 — ROADMAP Phase 3 완료 표기 + CHANGELOG + develop 직접 머지 ([[workflow-no-pr]])
- `sprint-review` 진입 — 코드 리뷰 + 회고 작성
