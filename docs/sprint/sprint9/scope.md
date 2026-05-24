---
Sprint: 9  |  Date: 2026-05-24  |  Session: #1
---

> Sprint 9 Session #1 — T1 (PI-02 결정 반영 + 보강 도메인 설계 검토).
> 예상 2h. 본 세션은 **순수 설계/검증** task — 코드 변경 없음, scope.md 작성으로 종료.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1** | PI-02 확정 + 기존 스키마 검증 + scope.md 작성 | 2h |

> 사용자 결정 (2026-05-24): PI-02 = 옵션 A (일 단위 매칭) 확정. sprint9.md L62 갱신 완료.

---

## PI-02 확정 사항

**옵션 A — 일 단위 매칭** (사용자 결정 2026-05-24).

| 항목 | 규칙 |
|------|------|
| 매칭 단위 | 일(day) 단위 — 보강 1일 = 결석 N일 충당 |
| 시간값 검증 | 없음 — `class_minutes` 비교 생략 |
| 보강필요시간 표시 | 기존 `compute_summary` 유지 — `SUM(absence.class_minutes WHERE makeup_attendance_id IS NULL) - SUM(makeup_attended.class_minutes)` |
| 변경 시 영향 | T3 `create_makeup_with_absences` 의 검증 3만 활성/비활성 (분 단위 전환 시 단순) |

→ T3 코드에 PI-02 결정 명시 (주석 + 분 단위 전환 시 활성 위치 표시) — R58 대응.

---

## 기존 스키마 검증 결과

### 결론: **V108 신규 마이그레이션 불필요**

보강 도메인 전체 흐름이 V106/V107 + V102/V301 스키마로 구현 가능. 신규 도메인 컬럼/테이블 추가 없음.

### 검증 매트릭스

| Sprint 9 요구 사항 | 기존 스키마 항목 | 검증 결과 |
|----------------|----------------|----------|
| 미처리 결석 조회 (T2 `get_pending_absences`) | `regular_attendances.status='absent' AND makeup_attendance_id IS NULL` (V106) | ✅ 가능 — 인덱스 `idx_regular_att_makeup` (V107) 활용 |
| 소멸기한 임박 정렬 | `regular_attendances.makeup_deadline` (V106, YYYY-MM TEXT) | ✅ `ORDER BY makeup_deadline ASC, event_date ASC` |
| 보강 가능 일자 판별 (T2 `get_makeup_eligible_dates`) | `schedule_codes.allows_makeup_class` (V102) + V301 공휴수업일 보정 | ✅ `JOIN schedule_events ON event_date` 으로 가능 |
| 보강 등록 + 매칭 (T3 `create_makeup_with_absences`) | `INSERT makeup_attendances` + `UPDATE regular_attendances` (V107 FK) | ✅ 트랜잭션 + FK 강제로 무결성 보장 |
| 보강 취소 → 결석 환원 (T4 `cancel_makeup`) | `UPDATE regular_attendances SET makeup_attendance_id=NULL, status='absent'` + `DELETE makeup_attendances` | ✅ 트랜잭션 내 순차 — FK 위반 없음 (DELETE 전 NULL 처리) |
| 보강 미등원 (T4 `mark_makeup_absent`) | `makeup_attendances.status='makeup_absent'` (V106 CHECK 2상태) | ✅ |
| 결석 이력 조회 (T8) | 기존 SELECT 만 | ✅ |

### audit::AuditEventType 추가 variants

코드 변경 (마이그레이션 아님). T3/T4 에서 도입:
- `MakeupCreated` → "makeup-created"
- `MakeupCancelled` → "makeup-cancelled"
- `MakeupAbsent` → "makeup-absent"

---

## 신규 모듈/파일 결정

| 결정 | 이유 |
|------|------|
| **모듈 분리**: `src-tauri/src/commands/makeup.rs` 신규 (attendance.rs 에 누적 X) | attendance.rs 이미 1000+ 줄. 보강은 별개 도메인이므로 모듈 분리가 가독성/유지보수 측면 유리 |
| `mod.rs` 에 `pub mod makeup;` 추가 | T2 작업 |
| `lib.rs` invoke_handler 등록 — 보강 IPC 6종 추가 예정 (T2 2종 + T3 1종 + T4 3종) | — |
| 프론트엔드 라우트 신규 없음 — 보강 등록은 `/attendance` 의 비수업일 셀 클릭 다이얼로그 | T6 작업. `MakeupRegisterDialog` 신규 컴포넌트 |
| 보강데이 일괄 — `/attendance` 헤더 "보강데이 일괄" 버튼 → 별도 페이지 `/attendance/makeup-batch` | T7. 다중 원생 선택 UI 복잡도 분리 |
| 결석 이력 — `/students/[id]` 상세 페이지에 섹션 추가 | T8 |

---

## 데이터 흐름도

```
[보강 등록 — 개별 (UC-4 핵심)]
사용자: /attendance 비수업일 셀 클릭
   ↓
프론트엔드: MakeupRegisterDialog 오픈
   ↓
백엔드: get_makeup_eligible_dates(student_id, year_month) — 가능 일자 사전 검증
   ↓
백엔드: get_pending_absences(student_id) — 충당 결석 목록 (소멸기한 임박 순)
   ↓
사용자: 결석 N건 다중 선택 → "확정"
   ↓
백엔드: create_makeup_with_absences(student_id, event_date, class_minutes, absence_ids)
   ├── 트랜잭션 BEGIN
   ├── 검증 1: event_date 보강 가능 일자
   ├── 검증 2: absence_ids 모두 미처리 결석
   ├── 검증 3: (PI-02 일 단위 — 생략) / (분 단위 — class_minutes 합산 비교)
   ├── INSERT makeup_attendances → makeup_id
   ├── UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=makeup_id WHERE id IN (absence_ids)
   ├── audit::MakeupCreated 기록
   └── COMMIT
   ↓
프론트엔드: 출결표 invalidate → 결석 셀 빨강 → "보강" 표시로 전환

[보강 취소]
사용자: 보강 행 우클릭 → "취소"
   ↓
백엔드: cancel_makeup(makeup_id)
   ├── 트랜잭션
   ├── UPDATE regular_attendances SET makeup_attendance_id=NULL, status='absent' WHERE makeup_attendance_id=?
   ├── DELETE makeup_attendances WHERE id=?
   ├── audit::MakeupCancelled
   └── COMMIT

[보강 미등원]
사용자: 보강 행 마킹 → "미등원"
   ↓
백엔드: mark_makeup_absent(makeup_id)
   ├── 트랜잭션
   ├── UPDATE makeup_attendances SET status='makeup_absent' WHERE id=?
   ├── UPDATE regular_attendances SET makeup_attendance_id=NULL, status='absent' (연결된 결석 환원)
   │     ※ 결석 상태 유지 — 새 결석 미생성, 다음 보강 매칭 대상으로 재진입
   ├── audit::MakeupAbsent
   └── COMMIT
```

---

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint9/scope.md | [1회] | 본 세션 — 신규 |

> T1 은 순수 설계 task — 코드 변경 없음. 다음 세션(T2)부터 백엔드 신규.

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/migrations/` — V108 신규 마이그레이션 불필요 결정. 기존 V106/V107/V102/V301 활용
- [ ] `src/` 전체 — T1 범위 외 (T5~T8 에서 다룸)
- [ ] `src-tauri/src/` 전체 — T1 범위 외 (T2~T4 에서 다룸)

## 완료 기준 (이번 세션) — T1 AC (sprint9.md L80)

- ✅ AC-T1-1: PI-02 확정 기록 (옵션 A 일 단위 매칭) — "PI-02 확정 사항" 섹션
- ✅ AC-T1-2: 기존 스키마 검증 + V108 불필요 결정 명문화 — "검증 매트릭스" 표
- ✅ AC-T1-3: 보강 도메인 데이터 흐름도 작성 — "데이터 흐름도" 섹션
- ✅ AC-T1-4: 모듈 분리 결정 (`makeup.rs` 신규) — "신규 모듈/파일 결정" 표

## 세션 종료 조건

- ✅ scope.md 완성 (본 파일)
- ⬜ 단일 커밋 (sprint9 브랜치 첫 커밋)

## 발견된 이슈
(없음 — T1 은 설계 task)

## 다음 세션 (T2) 미리보기

- 신규 모듈 `src-tauri/src/commands/makeup.rs` 생성
- IPC 2종: `get_pending_absences`, `get_makeup_eligible_dates`
- A43 흡수: `validate_year_month` 월 범위(01-12) 검증 강화 — 기존 `attendance.rs::validate_year_month` 수정
- `src-tauri/src/commands/mod.rs` 에 `pub mod makeup;` 추가
- 단위 테스트: 소멸기한 정렬 + 보강 가능 일자 필터 + validate 무효 입력 거부

---

## carry-over

- A39/A40 프로세스 개선이 본 sprint 부터 강제 — T9 통합 검증에서 검증
- T1 코드 변경 없음으로 self-verify 단계 생략 가능 (scope.md 단일 커밋)

---

## Session #2 (T2 — 보강 IPC 백엔드 미처리 결석 + 보강 가능 일자 + A43, 2026-05-24)

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T2** | `makeup.rs` 신규 + `get_pending_absences` + `get_makeup_eligible_dates` + A43 validate 강화 | 6h |

### 설계 결정 (T2)

#### IPC 2종 — `makeup.rs` 신규 모듈

- `get_pending_absences(student_id) -> Vec<PendingAbsence>`
- `get_makeup_eligible_dates(student_id, year_month) -> Vec<EligibleDate>`

#### 응답 구조 (camelCase)
```ts
PendingAbsence {
  id, eventDate, yearMonth, classMinutes,
  makeupDeadline?, absenceMemo?,
}
EligibleDate {
  eventDate, scheduleCodeName,
}
```

#### 정렬 규칙 — `get_pending_absences`
- `ORDER BY (makeup_deadline IS NULL), makeup_deadline ASC, event_date ASC`
- NULL 마지막, 임박순, 동일 deadline 내 event_date 오름차순

#### 책임 분담 — `get_makeup_eligible_dates`
- 학사일정 기반 (`schedule_codes.allows_makeup_class=1`) + 학생 입퇴교 범위 필터
- **정규 수업 요일 필터는 본 IPC 가 아닌 T3 트랜잭션 검증에서** — 책임 단순화
- `class_minutes` 응답 제거 — 학생 스케줄 다중 가능성 + 다이얼로그에서 직접 입력이 명확

#### A43 — `validate_year_month` 강화

```rust
// 신규 검증 추가
let m: u8 = month.parse().expect("digits checked above");
if !(1..=12).contains(&m) {
    return Err(format!("year_month 의 월은 01~12 사이여야 합니다 (입력: {}).", ym));
}
```

- `pub(crate)` 노출 — `makeup.rs` 등 동일 crate 의 다른 도메인 모듈에서 재사용
- 사용자 친화 한글 에러 메시지 + 입력값 echo

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/makeup.rs | [신규] | IPC 2종 + `_impl` 분리 + 응답 구조체 2종 + 단위 테스트 8건 |
| src-tauri/src/commands/attendance.rs | [1회] | `validate_year_month` 강화 + `pub(crate)` 노출 + 신규 단위 테스트 1건 |
| src-tauri/src/commands/mod.rs | [1회] | `pub mod makeup;` 추가 |
| src-tauri/src/lib.rs | [4회 ⚠️] | invoke_handler 에 IPC 2개 등록 |
| docs/sprint/sprint9/scope.md | [2회] | Session #2 추가 |

### 단위 테스트 (T2 AC 매핑)

- ✅ AC-T2-1: `pending_absences_sorts_by_makeup_deadline_nulls_last` — 소멸기한 임박순 + NULL 마지막
- ✅ AC-T2-1 보강: `pending_absences_excludes_matched_absences` — 이미 매칭된 결석 제외
- ✅ AC-T2-1 보강: `pending_absences_excludes_present_status` — 출석 상태 제외
- ✅ AC-T2-2: `eligible_dates_returns_makeup_class_dates` — allows_makeup_class=1 만 반환
- ✅ AC-T2-2 보강: `eligible_dates_excludes_makeup_off_codes` — 방학(=0) 제외
- ✅ AC-T2-2 보강: `eligible_dates_expands_period_codes` — 기간성 코드 일자 펼침
- ✅ AC-T2-3: `eligible_dates_excludes_before_enroll` — 입교일 이전 제외
- ✅ AC-T2-3 보강: `eligible_dates_excludes_after_withdraw` — 퇴교일 이후 제외
- ✅ A43: `validate_year_month_rejects_out_of_range_month` — 월 00/13/99 거부 + 친화 메시지

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **231 passed** (T1 222 → +9) / cipher on **133 passed** (변동 없음)
- ✅ Clippy `--lib -- -D warnings` cipher off + on clean
- ✅ simplify — `_impl` 분리 + BTreeMap 으로 동일 일자 중복 회피 자동 정렬. 추가 추상화 없음
- ⬜ 단일 커밋 (makeup.rs 신규 + attendance.rs validate 강화 + mod/lib 등록 + scope.md)

### 발견된 이슈
(없음 — 기존 attendance.rs::seed_student 패턴 재사용으로 students 컬럼 규약 일관 유지)

---

## Session #3 (T3 — 보강 등록 + 매칭 트랜잭션, 2026-05-24)

> **skill: karpathy-guidelines** 자동 배정 (sprint9.md L104) — 트랜잭션 원자성 핵심.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T3** | `create_makeup_with_absences` IPC 트랜잭션 + audit `MakeupCreated/Cancelled/Absent` 3종 추가 | 6h |

### 설계 결정 (T3)

#### IPC 본체 — 단일 페이로드 + 단일 응답

```rust
CreateMakeupPayload { student_id, event_date, class_minutes, absence_ids }
MakeupResult { makeup_id, student_id, event_date, matched_count }
```

다중 i64 + `Vec<i64>` 를 페이로드 struct 로 통일 — Tauri IPC 직렬화 안정성 + 향후 필드 추가 용이.

#### 트랜잭션 검증 5종 (순서 중요)

1. **이벤트 일자 보강 가능** — `schedule_events JOIN schedule_codes WHERE allows_makeup_class=1 AND event_date <= ? AND COALESCE(period_end_date, event_date) >= ?`
2. **학생 일관성** — 학생 존재 + 입퇴교 범위 내 event_date
3. **정규 수업 요일 차단** — `student_schedules.day_of_week` 일치 시 거부 (보강은 비수업일 한정)
4. **결석 유효성** — 학생 일치 → matched 체크 → status 체크 (matched 가 먼저 — 이미 매칭된 결석에 "이미 다른 보강" 메시지 정확)
5. **PI-02 시간값** — 옵션 A 일 단위 채택으로 검증 생략. 분 단위 전환 시 주석 활성 위치에서 1줄 SUM 비교 추가

#### audit::AuditEventType 신규 variants

`MakeupCreated/Cancelled/Absent` 3종 — kebab-case `"makeup-created/cancelled/absent"`. T3 (Created) + T4 (Cancelled/Absent) 양쪽 진입점 대비 한 번에 추가.

#### 트랜잭션 구조

```text
검증 5종 (pool 직접) → tx = pool.begin()
   → INSERT makeup_attendances (status='makeup_attended') → makeup_id
   → for each absence_id: UPDATE regular_attendances SET makeup_done + makeup_attendance_id
      (WHERE status='absent' AND makeup_attendance_id IS NULL 재확인 → rows_affected=1 검증으로 race 차단)
   → COMMIT
   → audit::MakeupCreated (커밋 후 fire-and-forget)
```

검증 4가 검증 시점과 UPDATE 시점 사이 race 가능성 — UPDATE WHERE 절에 `AND status='absent' AND makeup_attendance_id IS NULL` 재차 적용 + `rows_affected() != 1` 검출로 트랜잭션 롤백.

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/audit.rs | [1회] | MakeupCreated/Cancelled/Absent 3 variants + as_code 매핑 |
| src-tauri/src/commands/makeup.rs | [8회 ⚠️] | 응답 struct 2종 + create_makeup_with_absences IPC + 단위 테스트 9건. 기존 `seed_student` 에 schedules 인자 확장 (effective_from 포함) |
| src-tauri/src/lib.rs | [3회 ⚠️] | invoke_handler 에 `create_makeup_with_absences` 등록 |
| docs/sprint/sprint9/scope.md | [3회] | Session #3 추가 |

### 단위 테스트 (T3 AC 매핑, 9건 신규)

- ✅ AC-T3-1: `create_makeup_matches_absences_atomically` — 결석 2건 → makeup_id 발급 + 양쪽 makeup_done 전이
- ✅ AC-T3-2: `create_makeup_rejects_empty_absences` — 빈 absence_ids 거부
- ✅ AC-T3-3: `create_makeup_blocks_when_event_date_not_makeup_eligible` — 보강 불가 일자 차단
- ✅ AC-T3-4: `create_makeup_blocks_regular_class_weekday` — 정규 수업 요일 차단
- ✅ AC-T3-5: `create_makeup_rejects_nonexistent_absence_id` — 미존재 id 거부
- ✅ AC-T3-6: `create_makeup_rejects_other_students_absence` — 학생 일관성
- ✅ AC-T3-7: `create_makeup_rejects_already_matched_absence` — 이미 매칭된 결석 거부 (matched 체크 우선)
- ✅ AC-T3-8: `create_makeup_rolls_back_on_validation_failure` — 검증 4 실패 시 makeup_attendances 0건 + 유효 결석 absent 유지
- ✅ AC-T3-9: `create_makeup_rejects_before_enroll_date` — 입교일 이전 거부

### 발견된 이슈

- `seed_student` 의 `student_schedules` INSERT 에서 `effective_from` NOT NULL 누락 → 첫 실행 시 9 테스트 모두 panic. attendance.rs::seed_student 패턴 대조하여 즉시 보완 (`enroll_date` 값 재사용).
- 검증 4 의 matched/status 순서 — 처음엔 status 먼저였으나 매칭된 결석에 "상태가 makeup_done" 메시지 노출이 부정확. matched 체크를 먼저로 변경하여 "이미 다른 보강" 메시지 노출.

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **240 passed** (T2 231 → +9) / cipher on **133 passed**
- ✅ Clippy `--lib -- -D warnings` cipher off + on clean
- ✅ simplify — 검증 5단계 명확 분리, 픽스처 헬퍼 3종(fixture_*) 단일 책임, PI-02 분 단위 활성 위치 주석 명시. 추가 단순화 없음
- ✅ 단일 커밋 `e0e3659`

---

## Session #4 (T4 — 보강 취소 + 미등원 + 일괄, 2026-05-24)

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T4** | `cancel_makeup` + `mark_makeup_absent` + `batch_create_makeups` IPC 3종 + 테스트 7건 | 5h |

### 설계 결정 (T4)

#### IPC 3종

- **`cancel_makeup(makeup_id)`** — 결석 환원 + makeup DELETE
  - 트랜잭션 순서 (FK 위반 회피): UPDATE absences SET NULL → DELETE makeup
  - 환원 결석 수를 audit details 에 기록
- **`mark_makeup_absent(makeup_id)`** — 결석 상태 유지, 보강만 'makeup_absent' 마킹
  - 멱등성: 이미 makeup_absent 면 0 반환, 트랜잭션 미실행
  - 연결된 결석은 absent 환원 (재매칭 가능 상태)
- **`batch_create_makeups(event_date, entries)`** — 다중 원생 일괄
  - **학생별 독립 트랜잭션** — 한 학생 실패가 다른 학생을 차단하지 않음 (PRD §4.5.5 "실패 원생은 건너뛰고")
  - `create_makeup_with_absences_impl` 재사용 — T3 검증 5종 동일 적용
  - 결과: `BatchResult { succeeded: Vec<MakeupResult>, failed: Vec<BatchFailure {student_id, reason}> }`

#### 페이로드 struct 신규

```ts
BatchMakeupEntry { studentId, classMinutes, absenceIds }
BatchCreateMakeupsPayload { eventDate, entries }
BatchFailure { studentId, reason }
BatchResult { succeeded, failed }
```

학생별로 `class_minutes` 다를 수 있어 entry 에 포함. `event_date` 는 batch 공통.

#### audit 활용

T3 에서 추가한 `MakeupCancelled` / `MakeupAbsent` variant 사용. batch 내부 성공도 학생별로 `MakeupCreated` fire-and-forget 기록 (`"batch":true` 표식).

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/makeup.rs | [3회 ⚠️] | 페이로드 struct 4종 + IPC 3종 + `_impl` + 단위 테스트 7건 |
| src-tauri/src/lib.rs | [4회 ⚠️] | invoke_handler 에 IPC 3개 등록 |
| docs/sprint/sprint9/scope.md | [4회] | Session #4 추가 |

> makeup.rs 3회 / lib.rs 4회 — Session 별 IPC 누적 추가 패턴이라 loop-detection 의 "동일 파일 3회 이상 반복 수정" 의도(버그 루프)와 무관.

### 단위 테스트 (T4 AC 매핑, 7건 신규)

- ✅ `cancel_makeup_reverts_absences_and_deletes_makeup` — 결석 2건 환원 + makeup 0건
- ✅ `cancel_makeup_rejects_nonexistent_id` — 미존재 친화 에러
- ✅ `mark_makeup_absent_preserves_makeup_but_reverts_absence` — 보강 status='makeup_absent' + 결석 absent + 재매칭 가능
- ✅ `mark_makeup_absent_is_idempotent` — 이미 미등원 상태 시 0 반환, 추가 변경 없음
- ✅ `batch_create_all_succeed` — 2명 모두 정상 → succeeded 2, failed 0
- ✅ `batch_create_partial_failure` — s2 무효 absence_id → s1 정상 / s2 failed 분리. s1 의 결석은 makeup_done
- ✅ `batch_create_rejects_empty_entries` — 빈 entries 거부

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **247 passed** (T3 240 → +7) / cipher on **133 passed**
- ✅ Clippy `--lib -- -D warnings` cipher off + on clean
- ✅ simplify — IPC 3종 단일 책임, cancel/absent 트랜잭션 순서 주석 명시 (FK 위반 회피), 멱등성 사전 체크로 간결, batch 학생별 독립 트랜잭션 명시. 추가 단순화 없음
- ⬜ 단일 커밋 (makeup.rs + lib.rs + scope.md)

### 발견된 이슈
(없음 — T3 의 `create_makeup_with_absences_impl` 재사용으로 batch 검증 로직 일관성 자동 확보)

---

## Session #5 (T5 — TS IPC 래퍼 + 도메인 타입, 2026-05-24)

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T5** | `src/types/makeup.ts` 신규 + `src/lib/tauri/index.ts` 래퍼 6종 | 2h |

### 설계 결정 (T5)

- **타입 매핑 1:1**: 백엔드 `makeup.rs` serde struct 와 정확히 동일 필드 (camelCase). PendingAbsence / EligibleDate / CreateMakeupPayload / MakeupResult / BatchMakeupEntry / BatchCreateMakeupsPayload / BatchFailure / BatchResult — 총 8종 신규
- **래퍼 패턴**: 기존 `getInvoke()` + dev fallback 패턴 일관 적용
  - 조회 (`getPendingAbsences`, `getMakeupEligibleDates`, `batchCreateMakeups`): dev 모드 시 빈 배열/객체 반환 (브라우저 테스트 가능)
  - mutation (`createMakeupWithAbsences`): dev 모드 시 `throw` — 의도적 명시
  - void (`cancelMakeup`, `markMakeupAbsent`): dev 모드 시 silent return
- **payload 전달 방식**: 단일 객체 페이로드는 백엔드 시그니처 `fn cmd(payload: T)` → `invoke('cmd', { payload })` 형태로 wrapper key 동일

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src/types/makeup.ts | [신규] | 8 interface 1:1 매핑 |
| src/lib/tauri/index.ts | [3회 ⚠️] | makeup 타입 import + 래퍼 6종 추가 |
| docs/sprint/sprint9/scope.md | [5회 ⚠️] | Session #5 추가 |

### 세션 종료 조건
- ✅ Self-verify: `pnpm lint` clean / `pnpm tsc --noEmit` clean
- ✅ simplify — 단순 1:1 래퍼라 추가 단순화 없음
- ✅ 단일 커밋 `6f761f5`

### 발견된 이슈
(없음)

---

## Session #6 (T6 — 보강 등록 (개별) UI, 2026-05-24)

> **skill: frontend-design** — UC-4 핵심 흐름의 첫 UI.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T6** | `MakeupRegisterDialog` 신규 + AttendanceGrid 비수업일 클릭 핸들러 + page.tsx 통합 | 6h |

### 설계 결정 (T6)

#### 다이얼로그 흐름 — 옵션 F (절충)

1. **비수업일 셀 클릭** → 즉시 다이얼로그 오픈 (로딩 상태)
2. **eligibility query** (`getMakeupEligibleDates`) 마운트 시 호출 → eventDate 매칭 검사
3. 매칭 없으면: "보강 가능 일자 아님" 안내 + 닫기만 가능
4. 매칭 시: **`getPendingAbsences`** 조회 → 결석 다중 체크박스 선택 → class_minutes 입력 → "확정"
5. mutation 성공 시: `attendance-grid` + `pending-absences` invalidate → 다이얼로그 닫힘

**옵션 F 채택 이유**: 그리드 진입 시 N명 학생별 eligibility 미리 호출은 IPC 폭증 (50명 × 1IPC). 다이얼로그 마운트 시 1회 호출이 합리적 + UX 명확 (사용자가 어디 클릭해도 다이얼로그가 결과를 알려줌).

#### 비수업일 셀 UX

- 기존: `cell === null` → 단순 placeholder (`bg-gray-50`)
- 신규: `onEmptyCellClick` prop 있으면 클릭 가능 셀 + 호버 시 `bg-amber-50` + `+` 표시
- prop 없으면 기존 placeholder 유지 (다른 호출처 호환)

#### TanStack Query 통합

- `useQuery(['makeup-eligibility', sid, ym])` — eligibility 일자 목록
- `useQuery(['pending-absences', sid])` — 결석 목록 (eligibility=true 시 enabled)
- `useMutation(createMakeupWithAbsences)` — 성공 시 onSuccess → invalidate

#### MakeupDialogTarget 구조

```ts
{
  studentId, studentName, studentSerialNo, eventDate
}
```

학생 정보를 grid.students 에서 lookup 후 다이얼로그에 전달 — 별도 학생 IPC 호출 회피.

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src/components/attendance/MakeupRegisterDialog.tsx | [신규] | 다이얼로그 + AbsenceRow 하위 컴포넌트 + eligibility/pending 두 query + mutation |
| src/components/attendance/AttendanceGrid.tsx | [16회 ⚠️] | Props onNonClassDayClick 추가 / StudentRow yearMonth+onNonClassDayClick 전파 / CellView onEmptyCellClick 추가 / 비수업일 클릭 가능 분기 |
| src/app/attendance/page.tsx | [13회 ⚠️] | MakeupDialogTarget state + 다이얼로그 렌더링 + 학생 lookup + invalidate |
| docs/sprint/sprint9/scope.md | [6회 ⚠️] | Session #6 추가 |

### UX 가드

- 다이얼로그 ESC / 배경 클릭 / 취소 버튼 모두 닫기
- eligibility 미충족 시 "확정" 버튼 비표시 (취소만)
- 결석 0건 선택 시 확정 버튼 disabled
- mutation 진행 중 "등록 중..." 라벨

### 세션 종료 조건
- ✅ Self-verify: `pnpm lint` clean / `pnpm tsc --noEmit` clean
- ✅ simplify — 다이얼로그는 단일 책임 (등록 흐름), AbsenceRow 분리로 가독성 유지. 추가 추상화 없음
- ✅ 단일 커밋 `76c2ede`

### 발견된 이슈
(없음)

---

## Session #7 (T7 — 보강데이 일괄 + 보강 관리 + A41 라벨, 2026-05-24)

> **skill: frontend-design** — UC-4 후속 흐름 + Sprint 8 회고 A41 흡수.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T7** | A41 라벨 변경 + `MakeupManageDialog` + `BatchMakeupDialog` + page.tsx 통합 | 5h |

### 설계 결정 (T7)

#### A41 (Sprint 8 회고 흡수)

- 헤더 라벨 `"결석"` → `"미처리\n결석"` (text-sm + leading-tight 자동 2줄, width 62px 유지)
- `title` 속성: "status='absent' AND makeup_attendance_id IS NULL — 보강완료·소멸 결석은 제외"

#### MakeupManageDialog 신규 — `makeup_done` 셀 클릭 분기

- 진입: AttendanceGrid `onMakeupCellClick` prop (StudentRow 내부 분기)
- 3 모드: `menu` (취소/미등원 선택) → `confirm-cancel` / `confirm-absent`
- 액션: `cancelMakeup` (취소·DELETE) / `markMakeupAbsent` (미등원 마킹 + 재매칭 가능)
- ConfirmPanel 하위 — `isDanger` 옵션으로 색상 구분

#### BatchMakeupDialog 신규 — 보강데이 일괄

- 진입: `/attendance` 헤더 "보강데이 일괄" 버튼 (showGrid 시 활성)
- 학생별 미처리 결석은 grid 데이터에서 **client-side 필터** — IPC 폭증 회피
- 흐름: date input → 학생 자동 추출 + 체크박스 → `batchCreateMakeups` → BatchResult 표시
- 단순화: 학생별 entry 의 `classMinutes` 는 첫 결석 값 (학생별 다른 시간은 개별 다이얼로그 fallback)
- 부분 성공이라도 invalidate (`succeeded.length > 0` 시)

#### AttendanceGrid 확장

- Props: `onMakeupCellClick?: (studentId, cell) => void`
- StudentRow 내부 `handleCellClick` — `makeup_done` 시 분기, 그 외 일반 토글
- 책임 분담: 외부 `handleCellClick` 은 토글만, StudentRow 가 makeup_done 분기 (학생 ID 보유 위치)

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src/components/attendance/MakeupManageDialog.tsx | [신규] | 취소/미등원 + ConfirmPanel |
| src/components/attendance/BatchMakeupDialog.tsx | [신규] | 일괄 등록 — grid client-side 필터 |
| src/components/attendance/AttendanceGrid.tsx | [5회 ⚠️] | A41 라벨 / `onMakeupCellClick` Props / StudentRow 분기 |
| src/app/attendance/page.tsx | [4회 ⚠️] | `manageTarget` + `batchOpen` state / 헤더 일괄 버튼 / 다이얼로그 2종 |
| docs/sprint/sprint9/scope.md | [7회 ⚠️] | Session #7 추가 |

### UX 가드
- 위험 동작 (취소/미등원) 2단계 확인 (menu → confirm)
- BatchMakeupDialog 결과 후 재제출 차단
- ESC / 배경 / 닫기 버튼 일관

### 세션 종료 조건
- ✅ Self-verify: `pnpm lint` clean / `pnpm tsc --noEmit` clean (백엔드 변경 없음 — cargo 재검증 불요)
- ✅ simplify — ConfirmPanel 하위 분리, client-side 필터로 IPC 절약, AttendanceGrid 분기 책임 명확
- ✅ 단일 커밋 `ef06b43`

### 발견된 이슈
(없음 — `AttendanceCell.makeupAttendanceId` 가 이미 T3 응답에 포함되어 있어 별도 IPC 불필요)

---

## Session #8 (T8 — 결석 이력 조회, 2026-05-24)

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T8** | `get_absence_history` IPC + TS 래퍼 + `AbsenceHistoryDialog` + 학생명 클릭 | 3h |

### 설계 결정 (T8)

#### IPC + 응답 struct

- `get_absence_history(student_id) -> Vec<AbsenceHistoryItem>`
- SQL: `LEFT JOIN makeup_attendances` — `makeup_done` 행의 보강 일자/시간 포함
- 정렬: `event_date DESC` / WHERE: `status IN ('absent', 'makeup_done', 'makeup_expired')` (출석 제외)

#### 배치 위치 결정

- `/students/[id]` 동적 라우트 **미존재** → 차기 sprint 작업으로 분리
- **출결표 학생명 클릭 → `AbsenceHistoryDialog`** 패턴 (sprint9.md L211 대안)
- 차기 sprint 라우트 도입 시 본 다이얼로그 컨텐츠 재사용 가능

#### 시각 구분 (AC-4.5-7)

| 상태 | 배경 | 라벨 |
|------|------|------|
| `absent` | bg-red-50 | "미처리" (red-700) |
| `makeup_done` | bg-green-50 | "보강완료" (green-700) + 보강일/분 |
| `makeup_expired` | bg-gray-100 | "소멸" (gray-600) |

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/makeup.rs | [4회 ⚠️] | AbsenceHistoryItem struct + IPC + 테스트 3건 |
| src-tauri/src/lib.rs | [5회 ⚠️] | invoke_handler 등록 |
| src/types/makeup.ts | [1회] | AbsenceHistoryItem |
| src/lib/tauri/index.ts | [1회] | getAbsenceHistory 래퍼 |
| src/components/attendance/AbsenceHistoryDialog.tsx | [신규] | 다이얼로그 + HistoryRow + statusRowClass |
| src/components/attendance/AttendanceGrid.tsx | [6회 ⚠️] | `onStudentNameClick` Props + 학생명 button 분기 |
| src/app/attendance/page.tsx | [5회 ⚠️] | `historyTarget` state + 다이얼로그 렌더링 |
| docs/sprint/sprint9/scope.md | [8회 ⚠️] | Session #8 추가 |

### 단위 테스트 (T8 AC, 3건 신규)

- ✅ `absence_history_includes_three_states_in_desc_order` — 3상태 + 출석 제외 + DESC + makeup JOIN
- ✅ `absence_history_returns_empty_when_no_absences` — 빈 vec
- ✅ `absence_history_filters_by_student_id` — student_id 필터

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **250 passed** (T7 247 → +3) / cipher on **133 passed** / pnpm lint clean / pnpm tsc clean
- ✅ Clippy `--lib -- -D warnings` 양쪽 clean
- ✅ simplify — HistoryRow + statusRowClass 분리, 학생명 분기 단일 책임
- ✅ 단일 커밋 `f2a5689`

### 발견된 이슈
(없음)

---

## Session #9 (T9 — 통합 검증 + A39/A40 프로세스 적용, 2026-05-24)

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T9** | 자동 검증 7항목 + A39 마이그레이션 self-check + A40 산출물 경로 명시 + sprint9.md AC 일괄 마킹 + 사용자 시각 검증 위임 | 3h |

### 자동 검증 7항목 (전수 통과)

| # | 명령 | 결과 |
|---|------|------|
| 1 | `cargo test --lib --manifest-path src-tauri/Cargo.toml` (cipher off) | ✅ **250 passed** / 0 failed / 3 ignored (27.39s) |
| 2 | `cargo test --lib --manifest-path src-tauri/Cargo.toml --features cipher` (cipher on) | ✅ **133 passed** / 0 failed / 3 ignored (26.02s) |
| 3 | `cargo clippy --lib --manifest-path src-tauri/Cargo.toml -- -D warnings` (cipher off) | ✅ clean |
| 4 | `cargo clippy --lib --manifest-path src-tauri/Cargo.toml --features cipher -- -D warnings` (cipher on) | ✅ clean |
| 5 | `pnpm lint` | ✅ "No ESLint warnings or errors" |
| 6 | `pnpm tsc --noEmit` | ✅ clean (출력 없음) |
| 7 | `pnpm build` | ✅ static export 12개 라우트 — `/attendance` (9.85 kB), `/students/*` 3종, `/academic`, `/settings/*` 4종, `/setup`, `/lock` |

> **테스트 누적 추이**: T1 222 (cipher off baseline) → T2 +9 (231) → T3 +9 (240) → T4 +7 (247) → T8 +3 (250). T9 코드 변경 없음 → 250 유지. cipher on 133 동일.

### A39 — 마이그레이션 self-check

**결과**: ✅ 통과. Sprint 9에서 신규 마이그레이션 없음.

| 확인 항목 | 결과 |
|----------|------|
| scope.md Session #1 결정 ("V108 신규 마이그레이션 불필요") | 일치 |
| `src-tauri/migrations/` 디렉토리 마지막 파일 | `107__add_makeup_attendance_fk.sql` (Sprint 8 산출물) |
| `git log develop..HEAD -- 'src-tauri/migrations/*'` | 빈 결과 (신규 마이그레이션 0건) |

→ scope.md 검증 매트릭스(L40~L49)에 명시된 대로 V106/V107/V102/V301 + audit::AuditEventType 3 variant 신규(코드 변경)로 보강 도메인 전체 흐름 커버됨.

### A40 — sprint-review 산출물 4종 경로 명시

sprint-review 에이전트 호출 시 다음 4종 산출물 작성을 강제한다 (sprint9.md L364~L369 일치):

1. `docs/test-reports/sprint9.md` — 자동 검증 결과 + 사용자 시각 검증 결과 + UC-4 흐름별 상세
2. `docs/risk-register/2026-05-24.md` (또는 sprint-review 실행 일자) — Sprint 9 잔여 리스크 (R58~R62)
3. `docs/sprint-retrospectives/sprint9-retrospective.md` — Sprint 9 회고 (Capacity 38h / Velocity / 잘된 점 / 개선 점)
4. `docs/code-reviews/sprint9.md` — Sprint 9 코드 리뷰 (보강 IPC 7종 + UI 4 다이얼로그 중심)

### sprint9.md AC 일괄 마킹

T1~T8 각 작업 항목 + Definition of Done 의 자동 검증/단위 테스트/프로세스 항목 ⬜ → ✅ 전환. **사용자 시각 검증 의존 항목은 ⬜ 유지** — 사용자 응답 후 별도 마킹.

### 사용자 시각 검증 위임 (Sprint 8 A38 패턴)

`pnpm tauri:dev` 로 앱 기동 후 아래 5가지 흐름을 사용자가 직접 확인:

1. **보강 등록(개별)**: 비수업일 셀 클릭 → 결석 선택 → 확정 → 그리드에 "보강완료" 반영
2. **보강데이 일괄**: 보강데이 셀 → 원생 선택 → 확정 → 부분 성공/실패 결과 표시
3. **보강 취소**: 등록된 보강 셀 클릭 → "취소" → 결석 환원 확인
4. **보강결석(미등원)**: 등록된 보강 셀 클릭 → "미등원" → 결석 상태 유지 + 보강 status 변경
5. **결석 이력**: 출결표 학생명 클릭 → `AbsenceHistoryDialog` 3종 상태(미처리/보강완료/소멸) 시각 구분 + 헤더 라벨 "미처리\n결석"

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| docs/sprint/sprint9.md | [11회 ⚠️] | T1~T8 AC + DoD 일괄 마킹 (시각 검증 제외) |
| docs/sprint/sprint9/scope.md | [9회 ⚠️] | Session #9 추가 |
| src-tauri/Cargo.lock | [—] | 버전 0.3.2 → 0.4.0 (Cargo.toml 기준 정상 동기화) |

> scope.md 9회는 세션마다 누적된 패턴이라 loop-detection 의도(버그 루프)와 무관.

### 세션 종료 조건
- ✅ 자동 검증 7항목 전수 통과
- ✅ A39 self-check 통과 (V108 없음 결정 일치)
- ✅ A40 산출물 4종 경로 명시
- ✅ sprint9.md AC 일괄 마킹 (자동 검증/단위 테스트 범위)
- ⬜ 사용자 시각 검증 5종 흐름 (별도 진행)
- ⬜ 단일 커밋

### 발견된 이슈
(없음 — Cargo.lock 의 smarthb 버전 0.3.2 → 0.4.0 변경은 Cargo.toml 의 `version = "0.4.0"` 과 동기화 결과로 정상)
