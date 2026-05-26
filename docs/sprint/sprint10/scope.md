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
