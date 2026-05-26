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
| src-tauri/src/commands/attendance.rs | [6회 ⚠️] | `validate_year_month` 강화 + `pub(crate)` 노출 + 신규 단위 테스트 1건 |
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
| src-tauri/src/commands/makeup.rs | [19회 ⚠️] | 응답 struct 2종 + create_makeup_with_absences IPC + 단위 테스트 9건. 기존 `seed_student` 에 schedules 인자 확장 (effective_from 포함) |
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
| src/lib/tauri/index.ts | [6회 ⚠️] | makeup 타입 import + 래퍼 6종 추가 |
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
| src/components/attendance/AttendanceGrid.tsx | [49회 ⚠️] | Props onNonClassDayClick 추가 / StudentRow yearMonth+onNonClassDayClick 전파 / CellView onEmptyCellClick 추가 / 비수업일 클릭 가능 분기 |
| src/app/attendance/page.tsx | [24회 ⚠️] | MakeupDialogTarget state + 다이얼로그 렌더링 + 학생 lookup + invalidate |
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
| src/types/makeup.ts | [2회] | AbsenceHistoryItem |
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
| docs/sprint/sprint9.md | [14회 ⚠️] | T1~T8 AC + DoD 일괄 마킹 (시각 검증 제외) |
| docs/sprint/sprint9/scope.md | [9회 ⚠️] | Session #9 추가 |
| src-tauri/Cargo.lock | [—] | 버전 0.3.2 → 0.4.0 (Cargo.toml 기준 정상 동기화) |

> scope.md 9회는 세션마다 누적된 패턴이라 loop-detection 의도(버그 루프)와 무관.

### 세션 종료 조건
- ✅ 자동 검증 7항목 전수 통과
- ✅ A39 self-check 통과 (V108 없음 결정 일치)
- ✅ A40 산출물 4종 경로 명시
- ✅ sprint9.md AC 일괄 마킹 (자동 검증/단위 테스트 범위)
- ✅ 사용자 시각 검증 진행 — 8건 이슈 발견 (Session #10 으로 carry)
- ✅ 단일 커밋 `70c856a`

### 발견된 이슈
사용자 시각 검증 중 8건 발견 → Sprint 9 확장 결정 (사용자 2026-05-24, "Sprint 9 확장 — T10~T11 신규, 전부 흡수 +10h"). Session #10 으로 carry-over.

---

## Session #10 (T10/T11 — 시각 검증 이슈 8건 흡수, 2026-05-24)

> **Re-planning 트리거** 발동: T9 시각 검증 중 백엔드 정의/UI UX 8건 신규 요구 발견. Harness 원칙 1 (Planning First) 에 따라 사용자 결정 ("Sprint 9 확장 +10h") 후 본 세션 진입.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T10** | 백엔드 — `get_makeup_eligible_dates` 재설계 + T3 검증 3(정규 수업 요일 차단) 폐기 + 테스트 정책 전환 | 3h |
| **T11** | 프론트 — 시간 단위 변환(I1) + 일괄 버튼 버그(I2) + 충당 결석 필터(I4) + 자동 합산/min 차감(I5+I6) + 보강데이 헤더 강조(I7) + 비수업일 셀 사전 판단(I8) | 7h |

> 누적 capacity 35h + 10h = **45h** (원래 38h 대비 18% 초과). 사용자 승인.

### 시각 검증 발견 이슈 8건

| # | 이슈 | T10/T11 |
|---|------|---------|
| I1 | 보강수업시간 단위 분 → 시간(hours) | T11 |
| I2 | "보강데이 일괄" 버튼 활성화 버그 | T11 |
| I3 | 보강 가능일 정의 확장 | **T10** |
| I4 | 충당 결석 = 선택한 보강 일자 이전 + 소멸기한 미도래 | T11 (클라이언트 필터) |
| I5 | 결석 체크 시 보강수업시간 자동 합산 | T11 |
| I6 | 수동 변경 후 해제 시 차감 양 = min(결석시간, 현재 표시값) | T11 |
| I7 | 출결표 일자 헤더에 보강데이 시각 강조 | T11 |
| I8 | 비수업일 셀 사전 판단 — "+" 자체 비표시 | T11 |

### I3 정의 확정 (사용자 답변 2026-05-24)

| 케이스 | 가능 여부 | 비고 |
|--------|----------|------|
| `study_periods` 범위 **외** 평일 (보강불가 코드 없음) | ✅ **가능** | 보강 만료일 도래 전 월 구분 없이 가능 |
| `study_periods` 범위 내 토/일 (보강데이 코드 없음) | ❌ 불가 | |
| `study_periods` 범위 내 평일 + 보강불가 코드(공휴일/방학/휴원일) | ❌ 불가 | |
| `study_periods` 범위 내 평일 + 학사코드 없음 | ✅ 가능 | |
| `allows_makeup_class=1` 명시 일자 (보강데이/단원평가 응시일) | ✅ 가능 | 요일 무관 |

**확정 룰**:
```
보강 가능일 = 
  ( 평일(월~금) AND NO (allows_regular_class=0 AND allows_makeup_class=0 인 schedule_code) )
  OR
  ( EXISTS schedule_event WHERE schedule_code.allows_makeup_class=1 )
  AND 학생 입퇴교 범위 내
```

→ `study_periods` 범위 제약 **완전 제거**. 소멸기한 기준 + 학생 입퇴교 범위만 필터.

### T3 검증 3 정책 변경 (파급)

사용자 부연: "수업 요일에도 추가 시간 써서 수업 완료 후 보강 진행 가능".

| 항목 | 기존 (T3) | 변경 |
|------|----------|------|
| 검증 3 (정규 수업 요일 차단) | 정규 수업 요일이면 보강 거부 | **검증 3 폐기** — 같은 요일에도 보강 등록 허용 |
| 테스트 `create_makeup_blocks_regular_class_weekday` | "거부" 검증 | **"허용" 으로 전환** (정책 반영) |
| audit `MakeupCreated` 페이로드 | 변동 없음 | 변동 없음 |

> **PI-02 영향**: 옵션 A(일 단위 매칭) 의 검증 정책이 사용자 요구에 따라 완화. 검증 1·2·4·5 는 유지.

### T10 쿼리 설계 (재설계)

```sql
SELECT DISTINCT se.event_date AS event_date, sc_makeup.code_name AS schedule_code_name
FROM (
  -- 학생 입퇴교 범위 내 모든 후보 일자
  SELECT date FROM possible_dates  -- 연 단위 캘린더 (or generate_series 대체)
) candidates
LEFT JOIN schedule_events se ON se.event_date = candidates.date
LEFT JOIN schedule_codes sc ON se.code_id = sc.id
WHERE candidates.date BETWEEN ? AND ?  -- 학생 입퇴교 범위
  AND (
    -- 케이스 A: 평일 + 보강불가 코드 없음
    (
      strftime('%w', candidates.date) NOT IN ('0', '6')
      AND NOT EXISTS (
        SELECT 1 FROM schedule_events se2
        JOIN schedule_codes sc2 ON se2.code_id = sc2.id
        WHERE se2.event_date = candidates.date
          AND sc2.allows_regular_class = 0
          AND sc2.allows_makeup_class = 0
      )
    )
    OR
    -- 케이스 B: 보강 가능 코드 명시
    (sc.allows_makeup_class = 1)
  )
```

> 실제 구현은 SQLite 의 calendar table 가용성에 따라 — `recursive CTE` 로 일자 생성 또는 `schedule_events` UNION + 학생 입퇴교 범위 평일 generate. **year_month 파라미터 폐기 검토** (월 구분 없이 가능) — 단, 다이얼로그 UX 상 month 단위 표시는 유지 → year_month 파라미터는 유지하되 백엔드는 해당 월 내 가능 일자만 반환.

### T10 테스트 갱신/신규 (예상 8건)

- ✅ 갱신: `eligible_dates_returns_makeup_class_dates` (보강데이) — 기존 동작 유지
- ✅ 갱신: `eligible_dates_excludes_makeup_off_codes` (방학) — 기존 동작 유지
- 🆕 신규: `eligible_dates_includes_weekdays_without_schedule_code` — 평일 + 학사코드 없음 → 가능
- 🆕 신규: `eligible_dates_excludes_weekends_without_makeup_code` — 토/일 + 보강데이 없음 → 불가
- 🆕 신규: `eligible_dates_excludes_holiday_code` — 공휴일 코드 → 불가
- 🆕 신규: `eligible_dates_includes_weekends_with_makeup_code` — 토/일 + 보강데이 코드 → 가능
- ✅ 갱신: `eligible_dates_expands_period_codes` (기간성 코드) — 기존 동작 유지
- ✅ 갱신: `eligible_dates_excludes_before_enroll/after_withdraw` — 기존 동작 유지

### T11 작업 분해

| # | 작업 | 파일 | 예상 |
|---|------|------|------|
| I1 | 시간 단위 변환 헬퍼 `minutesToHours(m)/hoursToMinutes(h)` + 4 다이얼로그/AbsenceHistory 적용 | `src/lib/time.ts` (신규) + Dialog 4종 | 2h |
| I2 | "보강데이 일괄" 버튼 활성화 조건 디버그 | `/attendance/page.tsx` 헤더 | 0.5h |
| I4 | `MakeupRegisterDialog` — `pending_absences` 결과를 `event_date < target` + `makeup_deadline >= target.year_month` 클라이언트 필터 | `MakeupRegisterDialog.tsx` | 1h |
| I5+I6 | 결석 체크 시 합산 + 해제 시 `min(absenceHours, currentHours)` 차감 | `MakeupRegisterDialog.tsx` | 2h |
| I7 | `AttendanceGrid` 일자 헤더에 보강데이 시각 강조 (sky-100/sky-700 등) | `AttendanceGrid.tsx` | 1h |
| I8 | 비수업일 셀 사전 판단 — 클라이언트 측 `isMakeupEligible(date, student)` 로 "+" 표시 조건 분기 | `AttendanceGrid.tsx` | 0.5h |

→ T11 합계 7h.

### 수정/추가 파일 (Session #10 예상)

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/makeup.rs | [5회 ⚠️] | get_makeup_eligible_dates SQL 재설계 + T3 검증 3 제거 + 테스트 갱신/신규 |
| src-tauri/src/commands/attendance.rs | [—] | 변경 없음 |
| src/lib/time.ts | [신규] | minutesToHours / hoursToMinutes |
| src/components/attendance/MakeupRegisterDialog.tsx | [—] | I1/I4/I5/I6 |
| src/components/attendance/BatchMakeupDialog.tsx | [—] | I1 (시간 표시) |
| src/components/attendance/MakeupManageDialog.tsx | [—] | I1 (취소/미등원 확인 다이얼로그 시간 표시) |
| src/components/attendance/AbsenceHistoryDialog.tsx | [—] | I1 (이력 시간 표시) |
| src/components/attendance/AttendanceGrid.tsx | [7회 ⚠️] | I7 헤더 강조 + I8 셀 사전 판단 |
| src/app/attendance/page.tsx | [6회 ⚠️] | I2 일괄 버튼 활성화 |
| docs/sprint/sprint9.md | [2회] | T10/T11 작업 항목 추가 + AC |
| docs/sprint/sprint9/scope.md | [10회 ⚠️] | Session #10 추가 |

> AttendanceGrid 7회 / page.tsx 6회 / makeup.rs 5회 — Session 별 누적 패턴, loop-detection 의도(버그 루프)와 무관.

### 다음 단계 진입

1. T10 백엔드 진입 — `get_makeup_eligible_dates_impl` SQL 재설계 + 테스트 갱신/신규 + T3 검증 3 제거
2. Self-verify: cargo test cipher off/on + clippy
3. 단일 커밋 (T10)
4. T11 프론트엔드 진입 — I1~I8 (I3 제외) 순서 적용
5. Self-verify: pnpm lint + tsc
6. 단일 커밋 (T11)
7. 사용자 2차 시각 검증 → 이상없음 시 sprint9.md DoD 시각 검증 항목 ✅ → sprint-close

### carry-over (Sprint 10 으로 명시 이연)

(없음 — 사용자 결정으로 8건 전부 흡수)

---

## Session #11 (T12 — 2/3차 시각 검증 J1~J10 흡수, 2026-05-25~26)

> T10/T11 완료 후 사용자 시각 검증 추가 라운드 — 작은 UX 보완 + 도메인 모델 정제(미등원 폐기, 진입점 이동, 일괄 기능 제거).

### J 시리즈 변경 8건 (2/3차 시각 검증)

| # | 발견 | 해결 |
|---|------|------|
| J1 | "보강데이 일괄" 버튼 비활성 | 일괄 기능 자체 폐기 (J7) — 헤더 버튼/Dialog/래퍼 모두 삭제 |
| J2 | 보강데이 헤더 강조 색상 약함 | sky-100 원복 (사용자 결정 — amber 강조 시도 후 되돌림) |
| J3-1 | 시간 입력 step 0.5 → **1** |
| J3-2 | 1시간 결석 체크 시 3시간 표시 | React Strict Mode 더블 실행 — `setSelected` 콜백 내부 `setClassHours` 호출 → 외부 분리. 초기값 0 |
| J4 | 보강일 셀에 보강 표기 누락 | `AttendanceGridStudent.makeups` 백엔드 응답 추가 + emerald 라벨 표시 |
| J5 | 보강 미등원 개념 폐기 | `MakeupManageDialog` "미등원" 옵션 + `markMakeupAbsent` 호출 제거. status='makeup_absent' 사실상 미사용 (백엔드 IPC 유지 — 마이그레이션 회피) |
| J6 | 보강 삭제 진입점 결석 셀 → **보강 셀**로 이동 | `onMakeupCellClick`(결석셀) → `onMakeupDayCellClick`(보강셀). MakeupManageDialog props (cell → makeupId/eventDate/classMinutes) |
| J7 | 결석 셀 라벨 통일 + 보강매칭 배경 emerald | `statusCellClass`: absent '×' → '결석', makeup_done bg-red-50 → bg-emerald-100 (보강 셀과 동일) |
| J8 | 셀 hover 시 매칭 정보 노출 | 결석 셀 → 매칭 보강일자 / 보강 셀 → 충당 결석일자(들). StudentRow 양방향 매핑 (`absenceDatesByMakeupId` + `makeupEventDateById`) |
| J9 | tooltip 줄바꿈 | `parts.join(' · ')` → `\n` (HTML title 줄바꿈 native 지원) |
| J10 | 충당 결석 다건 줄바꿈 | 다건 시 라인 분리 (`길이 === 1` 분기) |

### 보강데이 일괄 기능 폐기 영향

- 삭제: `BatchMakeupDialog.tsx` (283줄), `batchCreateMakeups` 래퍼, `BatchMakeupEntry/BatchCreateMakeupsPayload/BatchFailure/BatchResult` 타입
- 헤더 "보강데이 일괄" 버튼 + `batchOpen` state + `pendingStudentsCount` 제거
- 백엔드 `batch_create_makeups` IPC 는 유지 (마이그레이션 회피, dead code) — Sprint 10 이후 정리

### 도메인 모델 정제

| 개념 | 변경 |
|------|------|
| 보강 미등원 (`mark_makeup_absent`) | UI 폐기 — 보강 등록은 "결과 기록" 의미 (사용자 결정 2026-05-25) |
| 결석 표기 | makeup_done 셀이 보강 후에도 "결석" 표기 유지 — 결석은 불변 사실 |
| 보강 셀 의미 | 결석일과 다른 일자에 표시 (J4) — 보강 진행 사실의 별도 기록 |

### 자동 검증
- ✅ cargo test cipher off **254** / cipher on **133** (T10/T11 동일 — J 시리즈는 UI 위주)
- ✅ cargo clippy cipher off/on clean
- ✅ pnpm lint/tsc clean

### 시각 검증 (사용자, 2026-05-26)
- ✅ 7차 라운드까지 누적 검증 — J10 까지 모든 흡수 완료 후 "검증완료" 보고

### 수정 파일 누적 (Session #11)

| 파일 | 변동 |
|------|------|
| src-tauri/src/commands/attendance.rs | +42 — `GridMakeupCell` struct + `makeups` 응답 + `build_day_schedules` (Session #10 잔여) |
| src/app/attendance/page.tsx | -55+0 — 일괄 버튼/state/Dialog 렌더링 제거, manageTarget 시그니처 변경 |
| src/components/attendance/AttendanceGrid.tsx | +168 — J4/J6/J7/J8/J9/J10 |
| src/components/attendance/BatchMakeupDialog.tsx | **DELETE** -283 |
| src/components/attendance/MakeupManageDialog.tsx | -113 (재작성) — props 변경 + 미등원 제거 |
| src/components/attendance/MakeupRegisterDialog.tsx | +26 — J3 strict mode + step 1 + 초기값 0 |
| src/lib/tauri/index.ts | -20 — `batchCreateMakeups`/`markMakeupAbsent` 래퍼 제거 |
| src/types/attendance.ts | +9 — `GridMakeupCell` interface |
| src/types/makeup.ts | -26 — Batch 4종 타입 제거 |
| docs/sprint/sprint9/scope.md | Session #11 추가 |

### DB 클린업 (2026-05-26)
- 사용자 시각 검증을 위해 prod DB (`~/Documents/smarthb/app.db`) 의 보강 3건 삭제 + 매칭 결석 1건 환원 (총 결석 2건 미처리)

### Session 종료 조건
- ✅ 사용자 시각 검증 "검증완료" (2026-05-26)
- ✅ 자동 검증 7항목 통과
- ⬜ 단일 커밋 (Session #11)
- ⬜ sprint9.md DoD 시각 검증 마킹 + sprint-close 진입

### Sprint 10 carry-over (정리용)
- `mark_makeup_absent` 백엔드 IPC + `markMakeupAbsent` audit variant 정리 (dead code)
- `batch_create_makeups` 백엔드 IPC + 관련 audit/payload 정리 (dead code)
- `makeup_attendances.status='makeup_absent'` CHECK 제약 마이그레이션 정리 (선택적)

---

## Session #12 (T13 — 4차 시각 검증 K1~K4 흡수, 2026-05-26)

> sprint-close + sprint-review 완료 후 `pnpm tauri:dev` 수동 스테이징 검증에서 4건 추가 발견 — 사용자 결정 "전부 Sprint 9 흡수".

### Task
| Task | 작업 | 예상 |
|------|------|------|
| **T13** | K1~K4 4건 흡수 — UI 보강 + 수업 셀 우클릭 진입점 + 단원평가 헤더 분리 | 4~6h |

### 시각 검증 4차 라운드 — 7건 (K1~K7, 4차+5차 누적)

| 코드 | 영역 | 내용 | 결정 |
|------|------|------|------|
| **K1** | 공통 (AttendanceGrid) | 충당 가능한 결석(미보강)이 없는 원생은 보강일 셀의 '+' 버튼 비표시 | `isEligible` 조건 추가 — 1차는 `summary.makeupNeededMinutes > 0`, **K1' (5차)** 에서 백엔드 응답 확장 후 정밀화 |
| **K1'** | 백엔드 (attendance.rs) | K1 정밀화 — "이전 일자에 만기 미도래 미보강 결석이 있을 때만" + 표시. 이전 월 결석 포함 | `AttendanceGridStudent.earliest_pending_absence_date: Option<String>` 추가. SQL: `MIN(event_date) WHERE absent AND makeup_attendance_id IS NULL AND (deadline IS NULL OR deadline >= grid yearMonth)`. 단위 테스트 3건 추가 |
| **K2** | 공통 (attendance/page) | 상단 '재원중만' 체크박스 — 퇴교 원생 필터링 | 새 state + `student.withdrawDate === null` 필터. **K2' (5차)** 디폴트 ON |
| **K3** | 보강 등록 진입점 (AttendanceGrid) | 수업있는 날에도 보강 등록 가능 — 정규 수업 셀에서 진입점 누락 | 수업 셀(present/makeup_done/expired) 우클릭 = 보강 등록. 결석 셀 우클릭 = 메모 (기존 유지) |
| **K4** | 헤더 라벨 (AttendanceGrid) | 단원평가/보강데이 헤더 배경색 동일 — 단원평가 헤더는 배경색 제거, 보강데이는 날짜 밑 작은 폰트 '보강데이' 표기 (셀 너비 유지) | label 기반 분기 — 코드명에 "단원평가" 포함 시 배경 미적용 / "보강데이" 포함 시 라벨 표기 |
| **K6** | 공통 (attendance/page) | '재원중만' 우측 '보강대상' 체크박스 — 보강 필요 원생만 필터 | `needsMakeupOnly` state, `earliestPendingAbsenceDate !== null` 필터, 디폴트 OFF |
| **K7** | 카운트 표기 (attendance/page) | 기존 'N / M 명' 별도 카운터 제거. "재원중(N명)" / "보강대상(M명)" 라벨 표기. 보강대상 산정은 재원중 필터와 연계 | 라벨에 카운트 직접 병기. 보강대상 카운트는 `enrolledOnly ON` 시 재원중 원생 한정 |

### K3 진입점 UX (사용자 결정)
- 결석(absent) 셀 우클릭 → 메모 (기존 유지)
- present / makeup_done / makeup_expired 셀 우클릭 → 보강 등록 다이얼로그 진입
- 비수업일(cell=null) + 보강 가능 → 좌클릭 + 버튼 (기존 유지)
- 보강(emerald) 셀 클릭 → 보강 관리(취소) (기존 유지)

### 수정 파일

| 파일 | 변동 |
|------|------|
| src-tauri/src/commands/attendance.rs | K1' — `earliest_pending_absence_date` 필드 + SQL + 단위 테스트 3건 |
| src/types/attendance.ts | K1' — `earliestPendingAbsenceDate` 타입 추가 |
| src/components/attendance/AttendanceGrid.tsx | K1' isEligible 정밀화 / K3 우클릭 라우팅 / K4 헤더 라벨 분기 |
| src/app/attendance/page.tsx | K2/K2' 재원중 필터 + 디폴트 ON / K6 보강대상 체크박스 / K7 카운트 라벨 |
| docs/sprint/sprint9/scope.md | Session #12 추가 |

### 자동 검증
- ✅ cargo test cipher off **256** passed (sprint-close 254 → K1' 신규 3건 추가)
- ✅ cargo clippy cipher off clean
- ✅ pnpm lint / tsc --noEmit clean
- ✅ pnpm build 13 라우트 정상

### 시각 검증 (사용자, 2026-05-26)
- ✅ 4차 라운드 K1~K4 검증 진행 → 정밀화/추가 요청(K1'/K2'/K6) 도출
- ✅ 5차 라운드 K1'/K2'/K6 검증 진행 → 카운트 표기 요청(K7) 도출
- ✅ 6차 라운드 K7 — "검수완료. 모두 pass"

### Session 종료 조건
- ✅ K1~K7 모두 사용자 시각 검증 통과
- ✅ 자동 검증 7항목 통과
- ✅ sprint9.md DoD 갱신 + CHANGELOG.md 항목 추가
- ⬜ 단일 커밋 + 메모리 동기화
