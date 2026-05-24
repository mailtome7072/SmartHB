---
Sprint: 8  |  Date: 2026-05-23  |  Session: #4 (T1·T2·T3 완료, T4 진행)
---

> Sprint 8 Session #4 — T4 전체 (출결표 프론트엔드 UI).
> 예상 8h. 백엔드 IPC 6개를 소비하는 그리드 + 토글 + 메모 다이얼로그 + 사이드바 활성화.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1** | V106 마이그레이션 + 단위 테스트 + .sqlx 캐시 갱신 | 3h |

> 사용자 결정 (2026-05-23): Session #1 = T1 단독. T2 (IPC) 는 다음 세션.

## 설계 결정 (T1)

### 마이그레이션 번호
- 다음 번호: **V106** (V101~V105 도메인 블록 다음, V200~V299 시드 / V300~V399 보정과 분리)

### 테이블 1: `regular_attendances` (정규 출결)
- `id`, `student_id` (FK), `event_date` (YYYY-MM-DD), `year_month` (YYYY-MM)
- `status` TEXT CHECK IN ('present','absent','makeup_done','makeup_expired') DEFAULT 'present'
- `class_minutes` INTEGER (수업 시간)
- `absence_memo` TEXT (선택)
- `makeup_deadline` TEXT (YYYY-MM, 결석 발생 월+1 = 소멸기한)
- `makeup_attendance_id` INTEGER REFERENCES `makeup_attendances(id)` — 보강완료 시 연결
- `created_at`, `updated_at`
- **UNIQUE (student_id, event_date)** — PRD §6.2

### 테이블 2: `makeup_attendances` (보강 출결)
- `id`, `student_id` (FK), `event_date` (YYYY-MM-DD), `year_month` (YYYY-MM)
- `status` TEXT CHECK IN ('makeup_attended','makeup_absent') DEFAULT 'makeup_attended'
- `class_minutes` INTEGER
- `created_at`, `updated_at`
- **UNIQUE 없음** — 동일 일자 다중 보강 허용 (PRD §6.2)

### 인덱스
- `idx_regular_att_student` (student_id)
- `idx_regular_att_yearmonth` (year_month) — 출결 그리드 조회 최적화 (50×31 < 1초)
- `idx_regular_att_date` (event_date)
- `idx_makeup_att_student` (student_id)
- `idx_makeup_att_yearmonth` (year_month)

### 순환 참조 (regular ↔ makeup)
`regular_attendances.makeup_attendance_id → makeup_attendances.id` — SQLite는 forward reference 허용하지만 마이그레이션에서는 두 테이블 모두 CREATE 후 FK 활성. CHECK 제약은 동시 가능.

### 단위 테스트
1. UNIQUE (student_id, event_date) 동작 — 동일 row 두 번 INSERT 시 제약 위반
2. makeup_attendances 는 UNIQUE 없음 — 동일 (student_id, event_date) 다중 INSERT 가능
3. status CHECK 위반 — 'invalid' INSERT 시 실패
4. FK student_id 무효 — students 에 없는 id 시 (PRAGMA foreign_keys=ON 환경) 실패

### `.sqlx` 오프라인 캐시
- 본 세션은 query!/query_as! 매크로 사용 안 함 (마이그레이션 + 단위 테스트만) → 캐시 재생성 불필요
- T2 (IPC 구현) 시점에 query 매크로 사용 → 그때 sqlx prepare

### 신규 의존성
- 없음

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/106__create_attendance_tables.sql | [1회] | 신규 마이그레이션 |
| src-tauri/src/commands/db.rs 또는 별도 테스트 모듈 | [1회] | UNIQUE/CHECK 단위 테스트 (인메모리 pool) |
| docs/sprint/sprint8/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/src/commands/` 기존 모듈 — T1 범위 외 (T2 에서 attendance.rs 신규 추가)
- [ ] `src/` 프론트엔드 — T3 에서 다룸

## 완료 기준 (이번 세션)

### T1 — V106 마이그레이션 (sprint8.md L73-129)
- ✅ AC-T1-1: `test_pool_in_memory` 마이그레이션 적용 + 두 테이블 생성 (`v106_creates_attendance_tables`)
- ✅ AC-T1-2: UNIQUE 동작 (`regular_attendances_unique_student_date`)
- ✅ AC-T1-3: makeup 다중 INSERT 3건 누적 (`makeup_attendances_allows_multiple_same_date`)
- ✅ AC-T1-4: status CHECK + year_month GLOB + class_minutes>0 (`attendances_status_check_rejects_invalid` + `attendances_format_checks`)
- ⬜ AC-T1-5: `.sqlx/` 캐시 — T2 세션 이연 (query 매크로 도입 시점)

### 세션 종료 조건
- ✅ Self-verify: cipher off 191 passed / on 126 passed, clippy clean (양쪽)
- ✅ simplify — db.rs 단위 테스트 + V106 SQL 단일 책임 유지
- ✅ 단일 커밋 `f72778b` (3파일, +305)

## 발견된 이슈

(없음 — Step-back 트리거 발생 시 여기에 기록)

## carry-over

- Sprint 7 carry-over 흡수 (T6~T8) 는 별도 세션에서 진행
- AC-T1-5 (.sqlx) — T2 본 세션에서 query 매크로 사용 시 `sqlx prepare` 함께 실행

---

## Session #2 (T2 — 출결 생성 IPC, 2026-05-23)

### 이번 세션 Task
| Task | 작업 | 예상 |
|------|------|------|
| **T2** | `attendance.rs` 신규 + `generate_attendances` + `check_attendance_exists` IPC + 단위 테스트 | 6h |

### 설계 결정 (T2)

- **모듈 위치**: `src-tauri/src/commands/attendance.rs` (신규). T3 이후 IPC도 같은 모듈에 누적.
- **IPC 2종**:
  - `check_attendance_exists(year_month) -> bool` — 중복 검사 사전 호출용
  - `generate_attendances(year_month) -> GenerateResult { year_month, student_count, attendance_count }`
- **`_impl` 분리**: 두 IPC는 인메모리 풀 테스트가 가능하도록 `*_impl(pool, ...)` private 함수를 가지고 `#[tauri::command]` 래퍼가 전역 `pool()` 을 주입.
- **OFF 일자 계산**: `schedule_events JOIN schedule_codes WHERE allows_regular_class=0` 의 (event_date, period_end_date) 범위를 모두 펼친 `HashSet<String>` — 단일 쿼리 + 메모리 전개로 N+1 회피.
- **트랜잭션**: `pool.begin()` (SQLite는 BEGIN DEFERRED 기본 — write 시점에 자동 IMMEDIATE 승격). 전 원생 INSERT 후 commit.
- **요일 매핑**: chrono `NaiveDate::weekday().number_from_monday()` (1=월~7=일) — 기존 academic.rs 와 동일 매핑.
- **class_minutes 계산**: `student_schedules.duration_hours * 60` (V101에서 hours INTEGER 저장).
- **`is_none_or`** clippy 1.95 가 `map_or(true, ...)` 거부 → `withdraw_d.is_none_or(|wd| d <= wd)` 채용 (안정화된 표준 API).

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/attendance.rs | [신규] | IPC + 테스트 |
| src-tauri/src/commands/mod.rs | [2회] | `pub mod attendance;` 추가 |
| src-tauri/src/lib.rs | [3회 ⚠️] | `invoke_handler` 에 IPC 2개 등록 |
| src-tauri/.sqlx/ | [재생성] | query 매크로 미사용 (raw `sqlx::query` 만) → prepare skip |
| docs/sprint/sprint8/scope.md | [2회] | Session #2 추가 |

### 단위 테스트 (Acceptance Criteria 대응)

- AC-T2-1: `generate_creates_attendances_for_active_students` — 원생 2명×주 3일, 6/1~6/30 → 정확한 건수
- AC-T2-2: `generate_skips_off_days` — 6/6 현충일(allows_regular_class=0) 일자 INSERT 없음
- AC-T2-3: `generate_respects_enroll_withdraw_range` — enroll=6/15, withdraw=6/25 → 그 사이 일자만
- AC-T2-4: `generate_blocks_duplicate_month` — 두 번째 호출 시 에러
- AC-T2-5: `generate_requires_confirmed_period` — is_confirmed=0 인 교습기간 거부
- AC-T2-6: `class_minutes_matches_schedule_hours` — duration_hours=2 → class_minutes=120
- AC-T2-7: 단위 테스트 빌드 + 통과 → Self-verify 절차로 보장

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **200 passed** (T2 신규 9건) / cipher on **126 passed** (cipher 가드로 attendance test skip — 의도된 동작)
- ✅ Clippy off+on (lib) clean. 참고: `--tests` 옵션으로는 cipher on 에서 기존 students.rs/schedules.rs/academic.rs 의 cipher-off-only 헬퍼 6건이 `dead_code` 로 잡힘 — T2 범위 외 이슈 (T6 carry-over 또는 별도 정리)
- ✅ simplify — MINUTES_PER_HOUR 상수, StudentRow struct, 헬퍼 4종 분리 모두 단일 책임. 추가 단순화 불필요
- ✅ 단일 커밋 `366f880` (4파일, +647)

---

## Session #3 (T3 — 출결 조회 + 토글 IPC, 2026-05-23)

### 이번 세션 Task
| Task | 작업 | 예상 |
|------|------|------|
| **T3** | attendance.rs에 IPC 4개 추가 + audit::AttendanceToggled variant 추가 + 단위 테스트 | 6h |

### 설계 결정 (T3)

#### IPC 4종
- `get_attendance_grid(year_month) -> AttendanceGrid` — 출결표 그리드 (원생 × 일자)
- `toggle_attendance(attendance_id, new_status) -> ToggleResult` — 출석/결석 토글
- `update_absence_memo(attendance_id, memo)` — 결석 사유 메모
- `get_attendance_summary(student_id, year_month) -> AttendanceSummary` — 원생 월간 요약

#### 응답 구조 (camelCase)
```ts
AttendanceGrid {
  yearMonth: string,
  students: [{
    studentId, name, serialNo, scheduleDays: number[1~7],
    attendances: [{ id, eventDate, status, classMinutes, absenceMemo?, makeupDeadline? }],
    summary: AttendanceSummary,
  }]
}
AttendanceSummary {
  studentId, yearMonth, presentCount, absentCount,
  makeupNeededMinutes, makeupCompletedMinutes,
}
ToggleResult {
  attendanceId, newStatus, newMakeupDeadline?, updatedSummary: AttendanceSummary,
}
```

#### 토글 규칙
- `present` → `absent`: status='absent', makeup_deadline = (year_month + 1), absence_memo는 유지
- `absent` → `present`: status='present', makeup_deadline=NULL, absence_memo=NULL
- `makeup_done` / `makeup_expired` → 토글 차단 (사용자 친화 한글 에러)
- 보강 매칭(`makeup_done`) 또는 소멸(`makeup_expired`) 상태는 별도 보강 도메인에서 관리 — T3 범위 외

#### 보강필요시간 계산
- `makeup_needed_minutes = SUM(class_minutes WHERE status='absent' AND makeup_attendance_id IS NULL)`
- `makeup_completed_minutes = SUM(class_minutes FROM makeup_attendances WHERE status='makeup_attended')`
- `makeup_expired` 는 SUM에서 제외 (소멸 처리는 별도)

#### 소멸기한 계산
- `(year_month + 1)`: YYYY-MM + 1 month. 12월 → 다음해 01.
- chrono `NaiveDate::with_day(1)?.checked_add_months(Months::new(1))?` 사용

#### audit 이벤트
- `AuditEventType::AttendanceToggled` 신규 variant + `"attendance-toggled"` code
- `record()` 호출: `event_subject = Some(attendance_id)`, `details = JSON({student_id, year_month, from, to})`

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/attendance.rs | [6회 ⚠️] | IPC 4종 + 응답 구조체 + 헬퍼 + 단위 테스트 추가 |
| src-tauri/src/commands/audit.rs | [5회 ⚠️] | `AttendanceToggled` variant + as_code 매핑 |
| src-tauri/src/lib.rs | [3회] | invoke_handler 에 4개 등록 |
| docs/sprint/sprint8/scope.md | [3회] | Session #3 추가 |

### 단위 테스트 (T3 AC 매핑)

- ✅ AC-T3-1: `get_attendance_grid_returns_full_structure` — 그리드 + 요약 + 일자별 셀
- ✅ AC-T3-2: `toggle_present_to_absent_increases_makeup_needed` — class_minutes 증가
- ✅ AC-T3-3: `toggle_absent_to_present_decreases_makeup_needed` — class_minutes 감소 + memo/deadline NULL 환원
- ✅ AC-T3-4: `toggle_to_absent_sets_deadline_next_month` (5월→6월, 12월→다음해 01, 1월→2월)
- ✅ AC-T3-5: `toggle_blocked_for_makeup_done_and_expired` + `toggle_rejects_invalid_status`
- ✅ AC-T3-6: `update_absence_memo_writes_text_and_nulls` — set/clear/not-found
- ✅ AC-T3-7: 단위 테스트 통과 (T3 신규 9건)
- ✅ 보조: `summary_excludes_matched_makeup_from_needed` + `summary_aggregates_completed_makeup_minutes`

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **209 passed** (T3 신규 9건) / cipher on **126 passed**
- ✅ Clippy off+on (lib) clean
- ✅ simplify — 응답 구조체 5개, `_impl` 분리, `compute_summary` 단일 정의 — 단일 책임 + 중복 없음
- ✅ 단일 커밋 `4efc570` (4파일, +741)

---

## Session #4 (T4 — 출결표 프론트엔드 UI, 2026-05-23)

### 이번 세션 Task
| Task | 작업 | 예상 |
|------|------|------|
| **T4** | types/attendance.ts + IPC 래퍼 6 + /attendance 라우트 + AttendanceGrid + AbsenceMemoDialog + 사이드바 활성화 | 8h |

### 설계 결정 (T4)

#### 신규 라우트
- `/attendance` — 출결 관리 메인 페이지. 월 선택 → "출결 생성" 또는 그리드 표시.

#### 상태 관리
- TanStack Query: `useQuery(['attendance-exists', ym])`, `useQuery(['attendance-grid', ym])`, `useMutation(toggle/memo)` 낙관적 업데이트
- Undo: 마지막 토글 1건만 메모리 보관, Ctrl+Z로 역토글. localStorage 등 영속화 없음.
- React.memo 적용 — 행 단위 메모 (셀 단위는 prop 변화 잦아 비효율).

#### UI 패턴
- 결석 셀: `bg-red-100 text-red-900` 굵게
- 보강완료 셀(`makeup_done`): 빨강 배경 + 작은 "보강" 텍스트
- 보강소멸 셀(`makeup_expired`): 회색 + "소멸"
- 수업 요일 아닌 일자: `bg-gray-50 text-gray-300` (셀 없음 → 빈 placeholder)
- 클릭 영역 44×44px (Tailwind `min-h-[44px] min-w-[44px]`)
- Pretendard 18pt — globals.css 기 적용. 그리드 셀은 16pt 유지 (정보 밀도)
- 좌측 원생 컬럼 sticky — `position: sticky; left: 0` + 헤더 z-index

#### 사이드바
- `menu-config.ts` — `/attendance` 의 `disabledHint` 제거 → ACTIVE_MENU_ITEMS 자동 포함

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src/types/attendance.ts | [신규] | AttendanceCell/Summary/GridStudent/Grid/ToggleResult/GenerateResult 타입 |
| src/lib/tauri/index.ts | [3회 ⚠️] | IPC 래퍼 6개 추가 |
| src/lib/menu-config.ts | [7회 ⚠️] | `/attendance` disabledHint 제거 |
| src/app/attendance/page.tsx | [신규] | 출결 관리 메인 페이지 |
| src/components/attendance/AttendanceGrid.tsx | [신규] | 그리드 본체 |
| src/components/attendance/AbsenceMemoDialog.tsx | [신규] | 결석 사유 메모 다이얼로그 |
| docs/sprint/sprint8/scope.md | [4회] | Session #4 추가 |

### 완료 기준 (이번 세션)

- ✅ AC-T4-1: `/attendance` 페이지 — 월 선택 콤보 (현재월 + 과거 11개월)
- ✅ AC-T4-2: "출결 생성" 버튼 — exists=false 일 때만 노출, mutation 후 invalidate
- ✅ AC-T4-3: 셀 클릭 토글 (mutation 후 invalidate — 단순성/안정성 우선)
- ✅ AC-T4-4: present=○, absent=×(빨강), makeup_done=보강, makeup_expired=소멸 + 비수업일 placeholder
- ✅ AC-T4-5: 요약 4컬럼 (출석/결석/보강필요/보강완료) — invalidate 로 실시간 갱신
- ✅ AC-T4-6: AbsenceMemoDialog — 우클릭 진입, ESC/배경클릭 닫기, 빈 입력 NULL 환원
- ✅ AC-T4-7: Ctrl+Z / Cmd+Z 1단계 Undo — `lastToggle` state
- ✅ AC-T4-8: 메모이제이션 (`StudentRow` React.memo + `useMemo` byDay/days) — 실측은 시각 검수 단계
- ✅ AC-T4-9: 44×44px 최소 영역, 한국어 라벨/툴팁, WCAG 대비
- ✅ AC-T4-10: `pnpm lint` clean + `pnpm tsc --noEmit` clean
- ✅ 단일 커밋 `0a20c18` (7파일, +764)

### 미해결/이연
- 낙관적 업데이트 → 사용자 액션 빈도 보고 T9 통합 검증에서 보강 검토

---

## Session #4-follow-up (T4 UX 보강) — 2026-05-23 23:xx

### 사용자 시각 검수 피드백 반영
출결표 첫 사용 검수 중 발견된 UX 결함을 한 커밋으로 정리.

| 영역 | 변경 |
|------|------|
| 사이드바 너비 | `w-56`만으로는 flex 컨테이너에서 압축됨 → `shrink-0` + AppShell 메인 컬럼 `min-w-0` 병행 |
| 출결표 헤더 | 일자 행 위에 한글 요일 행 추가 (`rowSpan=2` 그룹 헤더 구조). 토·일은 `text-red-600` |
| 보강시간 단위 | `(분)` → `(시간)` 변환. `minutesToHours()` 헬퍼 (정수는 정수, 그 외 소수점 1자리) |
| 출석/결석 단위 | `출석` → `출석 (일)` 2단 헤더로 분리 표시 |
| 요약 컬럼 위치 | 일자 컬럼들 우측 → 원생 이름 바로 우측으로 이동, `border-r-2`로 일자 영역과 분리 |
| 요약 컬럼 배경 | `bg-amber-100` (헤더) / `bg-amber-50` (데이터) 베이지 톤 그룹화 (PRD §5.7 저자극) |
| 사이드바 메뉴 | "보강 관리" disabled 항목 추가 (Phase 3 안내). 출결/단원평가/청구/공지문/학습보고서 순서 재배치 |

### 수정 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src/components/layout/sidebar.tsx | [2회] | `shrink-0` |
| src/components/layout/app-shell.tsx | [신규-follow-up] | `min-w-0` |
| src/components/attendance/AttendanceGrid.tsx | [4회 ⚠️] | 요일 행/시간 변환/컬럼 재배치/배경색 |
| src/lib/menu-config.ts | [8회 ⚠️] | 보강 관리 추가 + 순서 재배치 |
| src/app/attendance/page.tsx | [3회 ⚠️] | (이번 세션에서 직접 수정은 없으나 git status에 잡혀 확인 필요) |

### AC 영향
- AC-T4-4/5/9 모두 유지 (단위 표기 변경은 PRD §4.5.3 그리드 요구 사항을 더 정확히 반영).
- 신규 회귀 없음 — `pnpm lint` clean.

---

## Session #5 (T5 — 보강필요시간/소멸기한 단위 테스트 100%, 2026-05-23)

### 이번 세션 Task
| Task | 작업 | 예상 |
|------|------|------|
| **T5** | attendance.rs 단위 테스트에서 누락된 4개 시나리오 보강 | 2h |

### 설계 결정 (T5)

#### 기존 커버 vs 신규 커버
T3 세션에서 이미 비즈니스 규칙 핵심 5개 시나리오가 커버됨 — `compute_summary`, `toggle_impl`, `next_month_str` 모두 잘 분리되어 별도 `calculate_makeup_needed` / `set_makeup_deadline` 추상화 추가는 의도적으로 생략 (karpathy: don't add abstractions beyond what the task requires).

| 시나리오 | 위치 |
|---------|------|
| 1. 결석 1건 → needed=class_minutes | `toggle_present_to_absent_increases_makeup_needed` (T3) |
| 2. 결석 2건 → needed 합산 | **신규** `t5_two_absents_sum_makeup_needed` |
| 3. 결석 → 출석 환원 → needed 감소 | `toggle_absent_to_present_decreases_makeup_needed` (T3) |
| 4. makeup_done → needed 제외 | `summary_excludes_matched_makeup_from_needed` (T3) |
| 5. makeup_expired → needed 제외 | **신규** `t5_expired_absent_excluded_from_needed` |
| 6. 5월 결석 → deadline 2026-06 | `toggle_to_absent_sets_deadline_next_month` (T3) |
| 7. 12월 결석 → deadline 다음해 01 | `toggle_to_absent_sets_deadline_next_month` (T3) |
| 8. 출석 환원 → deadline NULL | `toggle_absent_to_present_decreases_makeup_needed` (T3) |
| 9. 동일 월 다중 결석 → 독립 deadline | **신규** `t5_multiple_absents_have_independent_deadlines` |
| 10. class_minutes=0 엣지 | **신규** `t5_class_minutes_check_rejects_zero_and_negative` (DB CHECK 검증) |

시나리오 9 해석 — "독립"의 의미를 "row 별 deadline 컬럼 + 한 row 토글이 타 row 무영향"으로 정의 (T3 정책상 동월 결석은 모두 동일 월+1 deadline).

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/attendance.rs | [4회] | T5 단위 테스트 4건 추가 |
| docs/sprint/sprint8/scope.md | [5회 ⚠️] | Session #5 추가 |

### 완료 기준 (이번 세션) — T5 (sprint8.md L266-270)
- ✅ AC-T5-1: 보강필요시간 = SUM(class_minutes WHERE status='absent' AND makeup_attendance_id IS NULL) — makeup_done/makeup_expired/매칭 결석 모두 제외 확인
- ✅ AC-T5-2: 소멸기한 = 결석 발생 월 + 1 (5월→6월, 12월→다음해 1월, 다중 결석 독립)
- ✅ AC-T5-3: 10개 시나리오 전수 통과 (T3 기존 6건 + T5 신규 4건)
- ✅ AC-T5-4: 비즈니스 규칙 100% 단위 테스트 커버 (PRD §6.5)

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off **213 passed** (attendance 모듈 22 passed — T5 신규 4건 추가)
- ✅ clippy --lib clean
- ✅ simplify — 신규 추상화 추가 없이 기존 헬퍼(`toggle_impl`/`compute_summary`/`fetch_cell`/`set_cell_state`/`first_attendance_id`) 재사용
- ⬜ 단일 커밋 — 본 섹션 작성 후 진행

### 발견된 이슈
(없음)

---

## Session #6 (T6 — Sprint 7 carry-over High 4건, 2026-05-24)

> **skill: systematic-debugging** 자동 배정 (보안 경로 변경).
> Sprint 7 Session #2 review에서 발견된 High 등급 보안/안정성 4건 (R40~R43) 통합 처리.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T6** | I-S2-2/3/4/5 통합 — auth.rs partial-NULL 강화, set_password 재진입 가드, CRED_CACHE exit 등록, legacy fallback 검증 | 5h |

### 사전 점검 결과 (scope 선언 전)

현재 `auth.rs` 상태를 읽은 결과 일부 인프라가 이미 마련되어 있음:

| 항목 | 현 상태 | T6 변경 |
|------|---------|---------|
| `is_salt_corrupted` (L305) | length≠32 + all-zero | + **first 8바이트 동일 바이트 반복 패턴** 감지 추가 |
| `set_password` (L508) | 재진입 가드 없음 | **`AtomicBool` compare_exchange** 가드 신규 + Drop 시 자동 해제 (RAII) |
| `invalidate_credential_cache()` (L88) | `pub(crate)` 존재, 주석 정확 | **`pub` 로 승격** + `startup::exit_hook()`에서 호출 |
| `CRED_CACHE` 주석 (L64) | "프로세스 종료 또는 명시적 invalidate" — 이미 정확 | (변경 불필요 — sprint8.md 요구는 "명시적 무효화 필요" 명시이므로 현 표현 유지) |
| `salt_exists_at` (L453) | 파일 미존재 시 `LEGACY_KEYRING_USER_SALT` fallback 이미 구현 | **단위 테스트** 신규 + `check_auth_status` 경로 검증 |

### 설계 결정 (T6)

#### I-S2-2 (R40): partial-NULL 패턴 감지

- 신규 검증: salt 32바이트 중 **첫 8바이트가 모두 동일한 단일 값** 이면 손상으로 간주 (NTFS power-loss 시 페이지 일부가 0x00 또는 0xFF 등 단일 패턴으로 잔존하는 사례 방어).
- 이미 all-zero 케이스는 `bytes.iter().all(|&b| b == 0)` 로 커버 — 8바이트 반복 패턴은 그 일반화.
- 함수 시그니처 유지 (`fn is_salt_corrupted(bytes: &[u8]) -> bool`) — 호출자 변경 없음.
- 테스트: 0x00/0xFF/0x42 단일 바이트가 첫 8바이트에 반복되면 손상 판정. 9번째부터 다른 값이면 정상 (단일 바이트 도배는 아니므로).

#### I-S2-3 (R41): `set_password` 재진입 가드

- `static SET_PASSWORD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);`
- 진입 시 `compare_exchange(false, true, ...)` — 이미 true면 에러 반환 ("비밀번호 설정이 이미 진행 중입니다.")
- **RAII 가드 struct** 로 해제 보장 — 함수 exit/panic 시 자동 false 복원. `tokio::task::spawn_blocking` panic 시 `keyring`/`salt`가 일관성 깨진 채 lock 남는 사고 방지.
- 단위 테스트: 두 번째 진입이 즉시 에러 (실제 keyring/salt store 호출 없이 가드만 검증 → `*_impl` 분리 불필요, AtomicBool 자체 테스트로 충분).

#### I-S2-4 (R42): `invalidate_credential_cache` exit 등록

- `pub(crate)` → `pub` 로 승격 (lib.rs → startup.rs → auth.rs cross-module 호출).
- `startup::exit_hook()` 내부에 `commands::auth::invalidate_credential_cache()` 호출 추가.
- exit_hook은 이미 `AtomicBool RAN` 가드로 idempotent — 중복 호출 안전.
- 위치: backup → lock 해제 **이후**가 자연스러움 (백업 작업 중 캐시 키 필요할 수 있으나, 백업은 별도 spawn 경로로 cipher feature 분기 — 검증 필요).
- 안전 순서: `try_create_backup` → `release_lock_atomic` → `invalidate_credential_cache` (캐시 무효화는 최후).

#### I-S2-5 (R43): legacy keyring fallback 단위 테스트

- 현재 `salt_exists_at(path)` 는 `path.exists() || keyring_get_or_none(LEGACY_KEYRING_USER_SALT)?.is_some()` — 코드는 이미 fallback 구현.
- 신규 테스트:
  - `salt_exists_at_returns_true_when_file_present` (이미 L834에 존재 — 재확인)
  - `salt_exists_at_returns_false_when_neither_file_nor_keyring` — 둘 다 부재 시 false (OS Keychain 의존 → `#[ignore]` 또는 dev 머신만)
  - `check_auth_status_returns_locked_when_legacy_keyring_only` — 통합 테스트 수준 (실제 keyring 의존). **단위 가능한 부분만** 다루고 OS daemon 의존부는 `#[ignore]` 처리.
- AC-T6-5의 "NotInitialized 오분류 방지" 는 `salt_exists_at` 단위 테스트 + 코드 리뷰로 보장.

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/auth.rs | [22회 ⚠️] | partial-NULL 검증 강화 + 재진입 가드 + pub 승격 + 신규 단위 테스트 |
| src-tauri/src/startup.rs | [2회] | `exit_hook` 에 `invalidate_credential_cache()` 호출 추가 |
| docs/sprint/sprint8/scope.md | [6회 ⚠️] | Session #6 추가 (loop-detection 임계 도달 — 단순 문서 추적이므로 무해, scope/Session 누적 특성) |

> scope.md `[6회 ⚠️]` 는 세션별 누적 기록(코드와 무관) — loop-detection 트리거 대상이 아니며 정상.

### 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/src/lib.rs` — exit_hook 호출 체계 변경 없음 (startup.rs 만 수정)
- [ ] `src/` 프론트엔드 — T6 범위 외 (백엔드 보안)
- [ ] `src-tauri/migrations/` — 스키마 변경 없음

### 완료 기준 (이번 세션) — T6 AC (sprint8.md L287-292)

- ✅ AC-T6-1: `is_salt_corrupted` partial-NULL 패턴(첫 8바이트 단일 반복) 감지 — `is_salt_corrupted_detects_partial_null_patterns` 신규 통과
- ✅ AC-T6-2: `set_password` 동시 호출 시 두 번째 호출 에러 반환 — `set_password_guard_blocks_concurrent_entry` + `releases_on_drop` + `releases_on_panic_unwind`
- ✅ AC-T6-3: `invalidate_credential_cache()` `pub(crate)` → `pub` 승격 — startup.rs cross-module 호출 가능
- ✅ AC-T6-4: `startup::exit_hook()` 가 backup/lock 해제 후 `auth::invalidate_credential_cache()` 호출 (startup.rs L229)
- ✅ AC-T6-5: `salt_exists_at` 정상 경로 + cache hit 경로 단위 테스트 — `check_auth_status_returns_locked_on_cache_hit`, OS daemon 의존 케이스 `#[ignore]`
- ✅ AC-T6-6: cipher off **218 passed** / cipher on **131 passed** (T6 신규 5건 — partial-null + 가드 3건 + cache-hit 1건)

### 세션 종료 조건

- ✅ Self-verify: `cargo test --lib` cipher off **218 passed** / cipher on **131 passed**
- ✅ Clippy `--lib -- -D warnings` clean (cipher off + on)
- ✅ simplify — 신규 추상화 없음, 가드/헬퍼 단일 책임 유지, `reset_credential_cache_for_tests` 재사용
- ⬜ 단일 커밋 (auth.rs + startup.rs + scope.md)

### 발견된 이슈
- `[1u8; SALT_LEN]` 등 단일 바이트 도배 salt 를 직접 사용하던 5개 store/load 라운드트립 테스트가 강화된 `is_salt_corrupted` 의 partial-NULL 감지에 걸려 실패 → `varied_salt(seed)` 헬퍼 도입으로 일괄 정리 (의도 변경 없음, 다양성만 추가). `derive_key`/`matches`/`cache_credentials` 테스트는 store/load 미경유라 영향 없음.

---

## Session #7 (T7 — Sprint 7 carry-over Medium-High R45, 2026-05-24)

> **skill: systematic-debugging** 자동 배정 (보안/동시성 경로 변경).
> Sprint 7 Session #2 review 의 I-S2-7 (R45) — Keychain concurrent race.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T7** | `get_cached_or_load_key` + `verify_password` 의 double-checked locking 누락 race 제거 | 3h |

### 사전 점검 결과 — R45 실제 race 확인

sprint8.md T7 의 가설(`tokio::join!` 지점에서 Keychain 직접 접근이 병렬 실행되는가) 을 코드 조사 결과:

| 위치 | 직접 Keychain 접근? |
|------|---------------------|
| `startup::tokio::join!(acquire_lock + check_integrity_quick)` | cipher on 시 integrity 가 `get_cached_or_load_key()` 경유 |
| `verify_password` (join! 외부 순차) | cache 미스 시 `load_credentials_to_cache()` 호출 |

**문제**: `get_cached_or_load_key` (auth.rs L117-126) 와 `verify_password` (L559-561) 둘 다 **double-checked locking 패턴 누락**:

```rust
// 현재 코드 — race 발생 가능
let guard = cred_cache().lock();
if let Some(c) = guard.as_ref() { return Ok(...); }
drop(guard);  // ← 여기서 lock 해제
load_credentials_to_cache()?;  // ← T1/T2 동시 진입 시 keyring 2회 호출
```

`tokio::join!` 안에서 `integrity::run_pragma_check` 가 `get_cached_or_load_key` 를 호출하는 시점과, 그 직후 `verify_password` 가 `load_credentials_to_cache` 를 호출하는 시점 사이에 race 가 발생. 둘 다 캐시 미스 상태에서 동시 진입하면 **macOS Keychain 다이얼로그 2회** (AC-T1-1 "다이얼로그 최대 1회" 위반).

> Sprint 7 T1 CredentialCache 가 의도한 race 해소는 **fast path 캐시 hit 시 keyring 0회**만 보장 — slow path(첫 로드) 의 직렬화는 미흡.

### 설계 결정 (T7)

#### 해결 방식: `ensure_cache_loaded` 헬퍼 + `LOAD_MUTEX` 직렬화

```rust
static LOAD_MUTEX: Mutex<()> = Mutex::new(());

fn ensure_cache_loaded() -> Result<(), AppError> {
    // Fast path — 캐시 hit (대다수 호출 경로)
    if cred_cache().lock().expect("cred_cache poisoned").is_some() {
        return Ok(());
    }
    // Slow path — load 직렬화. 다른 스레드가 load 중이면 대기.
    let _load_guard = LOAD_MUTEX.lock().expect("LOAD_MUTEX poisoned");
    // Double-check — 대기 중 다른 스레드가 이미 load 완료했을 수 있음.
    if cred_cache().lock().expect("cred_cache poisoned").is_some() {
        return Ok(());
    }
    load_credentials_to_cache()
}
```

`get_cached_or_load_key` 와 `verify_password` 의 캐시 채움 로직을 모두 이 헬퍼로 통합. 결과: keyring 호출은 첫 진입자 **정확히 1회**.

#### Mutex 선택 — `std::sync::Mutex` vs `tokio::sync::Mutex`

`std::sync::Mutex` 채택. 이유:
- 캐시 접근은 이미 `std::sync::Mutex<Option<CachedCredentials>>` 사용 — 일관성 유지
- `load_credentials_to_cache` 는 sync 함수 (keyring 호출도 sync) — async lock 불필요
- 호출자 `get_cached_or_load_key` 는 sync 함수 (cipher feature 가드 안)
- `verify_password` 는 async 이지만 spawn_blocking 으로 PBKDF2 만 분리 — Mutex 자체는 sync 컨텍스트에서 안전 (`tokio::task::block_in_place` 불요)

#### `LOAD_MUTEX` 와 `cred_cache().lock()` 중첩 안전성

- Fast path: `cred_cache` lock 만 잡고 즉시 drop.
- Slow path: `LOAD_MUTEX` 먼저 → `cred_cache` lock (double-check) → drop → `load_credentials_to_cache` (내부에서 `cache_credentials` 가 `cred_cache` lock).
- 락 순서: 항상 `LOAD_MUTEX` → `cred_cache`. Deadlock 불가.

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/auth.rs | [1회] | `LOAD_MUTEX` + `ensure_cache_loaded` 추가, `get_cached_or_load_key` + `verify_password` 리팩토링, multi-thread race 단위 테스트 |
| docs/sprint/sprint8/scope.md | [7회 ⚠️] | Session #7 추가 (세션별 누적, loop-detection 무관) |

### 수정하지 않을 파일

- [ ] `src-tauri/src/startup.rs` — tokio::join! 구조 변경 없음 (auth.rs 내부 직렬화로 해소)
- [ ] `src-tauri/src/commands/integrity.rs`, `db.rs`, `backup.rs` — `get_cached_or_load_key` 호출자 변경 없음
- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src/` 프론트엔드 — T7 범위 외

### 완료 기준 (이번 세션) — T7 AC (sprint8.md L307-310)

- ✅ AC-T7-1: startup sequence 에서 Keychain 다이얼로그 ≤ 1회 — `ensure_cache_loaded` LOAD_MUTEX 직렬화로 보장
- ✅ AC-T7-2: `tokio::join!` 지점에서 Keychain 직접 접근이 병렬 실행되지 않음 — slow-path 가 LOAD_MUTEX 잡은 첫 진입자만 keyring hit, 후속 진입자는 double-check 에서 캐시 hit. `ensure_cache_loaded_fast_path_is_concurrent_safe` (16 스레드) 통과, slow-path 직렬화 검증 테스트는 `#[ignore]` (OS keychain 의존)
- ✅ AC-T7-3: 캐시 적중 경로에서 Keychain 호출 0회 — Fast path 가 `cred_cache().lock()` 만 잡고 즉시 반환. 16 스레드 동시 진입에서도 같은 캐시 값 반환 확인

### 세션 종료 조건

- ✅ Self-verify: `cargo test --lib` cipher off **219 passed** / cipher on **132 passed** (T7 신규 2건 — fast-path concurrent + slow-path race ignored)
- ✅ Clippy `--lib -- -D warnings` 양쪽 clean
- ✅ simplify — `ensure_cache_loaded` 헬퍼 분리로 `verify_password` 캐시 미스 분기가 5줄 → 1줄로 자연 단순화 (의도된 부수효과). `get_cached_or_load_key` 도 fast/slow path 분기 제거하고 헬퍼 호출 + 캐시 읽기만으로 축약.
- ⬜ 단일 커밋 (auth.rs + scope.md)

### 발견된 이슈
(없음 — race 가설이 코드 조사로 확인되었고, 설계대로 LOAD_MUTEX 직렬화로 해소)

---

## Session #8 (T8 — carry-over Medium 6항목, 2026-05-24)

> Sprint 7 Session #2 review 의 Medium 잔여 (I-S2-8/9/10, R39, A31, R51) 통합 처리.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T8** | R46(Mutex poison) + R47(audit SecurityEvent) + R48-a(device.id 0o600) + R39(overlap is_confirmed) + R51(eventClick 차단) + A31 검토 | 4h |

### 사전 점검 결과

| 항목 | 위치 | 작업 |
|------|------|------|
| R46 | auth.rs `.expect("cred_cache poisoned")` 7곳 + `.expect("LOAD_MUTEX poisoned")` 1곳 | `cred_cache_lock()` 헬퍼 + `LOAD_MUTEX` 인라인 |
| R47 | auth.rs `migrate_keyring_salt_to` (L480) / audit.rs `AuditEventType` (L34) | `SecurityEvent` variant 추가 + `tokio::spawn(try_record(...))` fire-and-forget |
| R48-a | lock.rs `write_device_id_atomic` (L111) | `#[cfg(unix)]` + `PermissionsExt::set_mode(0o600)` |
| R48-b (skip) | salt buffer ZeroizeOnDrop | 시그니처 광범위 변경 필요 — 별도 task. 캐시 진입 후엔 이미 보호됨 (`CachedCredentials` ZeroizeOnDrop) |
| R48-c | stale doc comment | 작업 중 자연 정리 |
| R39 | academic.rs `create_study_period` (L115) + `update_study_period` (L183) overlap | `AND is_confirmed = 1` 추가 |
| R51 | academic/page.tsx `calendarEventClick` (L245) | `studyPeriodMode` 진입 시 early return |
| A31 | lock.rs `release_lock_atomic_removes_self_owned_lock` (L600) | 현 코드는 이미 `acquired.is_err()` skip 가드 보유. flaky 잔존 시그널 없으면 코멘트 정리만 |

### 설계 결정 (T8)

#### R46 — `cred_cache_lock()` 헬퍼

```rust
fn cred_cache_lock() -> std::sync::MutexGuard<'static, Option<CachedCredentials>> {
    cred_cache().lock().unwrap_or_else(|e| e.into_inner())
}
```

`std::sync::PoisonError::into_inner()` 로 poisoned guard 의 inner 를 회수 — panic 흔적이 남았어도 캐시 자체는 무결할 가능성이 높으므로 graceful 복구. 호출 사이트 7곳 일괄 단순화.

`LOAD_MUTEX` 는 단발 1회이므로 헬퍼 없이 인라인 `.unwrap_or_else(|e| e.into_inner())` 적용.

#### R47 — `AuditEventType::SecurityEvent` + fire-and-forget

- variant 추가: `SecurityEvent` → kebab-case "security-event"
- `migrate_keyring_salt_to` 는 sync 함수이고 `audit::try_record` 는 async. tokio runtime 검출 후 spawn:
  ```rust
  if let Ok(handle) = tokio::runtime::Handle::try_current() {
      handle.spawn(async {
          audit::try_record(AuditEventType::SecurityEvent,
              Some("salt-migration"),
              Some(r#"{"path":"cloud/smarthb/salt.bin"}"#)).await;
      });
  }
  ```
- migrate 호출 경로: `verify_password` (async, tokio runtime 있음) → `load_credentials_to_cache` (sync) → `load_salt` (sync) → `load_salt_from` (sync) → `migrate_keyring_salt_to`. tokio handle 검출 가능.

#### R48-a — Unix only

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
}
```

rename 전 tmp 파일에 0o600 설정 (소유자 read/write only). Windows 는 ACL 모델이라 별도 처리 불필요 (sprint8.md 명시).

#### R39 — overlap + `AND is_confirmed = 1`

`create_study_period` (L121) + `update_study_period` (L183) 둘 다 같은 패턴.

**의도**: 미확정 교습기간 (is_confirmed=0) 은 임시 작성 중인 상태로 간주, 다른 신규 교습기간 등록을 차단하지 않아야 한다. 확정된 것만 overlap 검사 대상.

#### R51 — selection 모드에서 배지 클릭 차단

```typescript
const calendarEventClick = (event: ScheduleEventListItem) => {
  // R51: studyPeriodMode 활성 중 배지 클릭 무시 — selection 모드에서 의도치 않은 삭제 다이얼로그 방지.
  if (studyPeriodMode) return
  setEventToDelete(event)
}
```

V27 코멘트는 갱신.

#### A31 검토

현 코드 (lock.rs:600-614):
```rust
let acquired = acquire_lock_atomic(false);
if acquired.is_err() { return; } // 외부 점유 — skip
let result = release_lock_atomic();
assert!(result.is_ok(), ...);
```

이미 외부 점유 시 skip 가드 보유. lock_path 격리는 글로벌 `paths::lock_path()` 의존성을 깨야 하므로 광범위 변경. **현 상태로 충분 — 추가 변경 없음**. 단, sprint8.md 의도에 따라 코멘트만 "flaky 조건은 외부 점유 skip 가드로 차단됨" 으로 명시.

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src-tauri/src/commands/auth.rs | [1회] | cred_cache_lock 헬퍼 + LOAD_MUTEX 인라인 + migrate audit spawn |
| src-tauri/src/commands/audit.rs | [1회] | SecurityEvent variant + as_code 매핑 |
| src-tauri/src/commands/lock.rs | [3회 ⚠️] | write_device_id_atomic 0o600 + 테스트 코멘트 |
| src-tauri/src/commands/academic.rs | [4회 ⚠️] | overlap 쿼리 2곳 + 단위 테스트 (미확정 무시) |
| src/app/academic/page.tsx | [1회] | calendarEventClick studyPeriodMode early return |
| docs/sprint/sprint8/scope.md | [8회 ⚠️] | Session #8 추가 (세션별 누적, loop-detection 무관) |

### 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/src/startup.rs` — T7 의 LOAD_MUTEX 직렬화로 race 해소. 추가 변경 없음
- [ ] `src-tauri/migrations/` — 스키마 변경 없음
- [ ] R48-b salt buffer ZeroizeOnDrop — 시그니처 광범위 변경 필요. 별도 task 로 이연

### 완료 기준 (이번 세션) — T8 AC (sprint8.md L332-339)

- ✅ AC-T8-1: `cred_cache` Mutex poison 시 앱 crash 대신 graceful 복구 — `cred_cache_lock` 헬퍼 + `LOAD_MUTEX` 인라인 (`unwrap_or_else(|e| e.into_inner())`)
- ✅ AC-T8-2: `migrate_keyring_salt_to` 실행 시 audit 로그에 SecurityEvent 기록 — tokio runtime 검출 시 fire-and-forget spawn
- ✅ AC-T8-3: `device.id` 파일 권한 0o600 (Unix) — `device_id_file_has_owner_only_permissions` 단위 테스트 통과
- ✅ AC-T8-4: `create_study_period` / `update_study_period` 미확정 교습기간이 있어도 overlap 미차단 — overlap 쿼리 `AND is_confirmed = 1` + `overlap_skips_unconfirmed_periods` SQL 단위 테스트
- ✅ AC-T8-5: selection 모드 중 배지 클릭 시 삭제 다이얼로그 미표시 — `calendarEventClick` `if (studyPeriodMode) return`
- ✅ AC-T8-6: lock 테스트 flaky 검토 — 외부 점유 skip 가드 (`if acquired.is_err() { return; }`) 가 이미 본 시나리오를 차단. 추가 변경 불필요. R48-c 의 "stale doc comment 정리" 도 이미 충분히 명시적이라 변경 없음
- ✅ AC-T8-7: 기존 단위 테스트 전체 통과 — cipher off **221 passed** / cipher on **133 passed** (T7 219/132 → +2/+1)

### 세션 종료 조건

- ✅ Self-verify: `cargo test --lib` cipher off **221 passed** / cipher on **133 passed** / `pnpm lint` clean / `pnpm tsc --noEmit` clean
- ✅ Clippy `--lib -- -D warnings` 양쪽 clean
- ✅ simplify — `cred_cache_lock` 헬퍼로 7곳 expect 패턴 일괄 정리 (의도된 simplification). `cached_salt` 5줄 → 3줄 축소 부수효과. 추가 작업 불필요
- ⬜ 단일 커밋

### 발견된 이슈
- R48-b (salt buffer ZeroizeOnDrop) 는 함수 시그니처 광범위 변경 필요 — `load_salt_from`/`migrate_keyring_salt_to`/`generate_salt`/`store_salt_to` 가 모두 `[u8; SALT_LEN]` raw array 시그니처. `Zeroizing<[u8; SALT_LEN]>` 또는 신규 wrapper struct 도입 시 호출 사이트 광범위 영향. T8 범위에서 skip, 별도 후속 task 로 분리. 캐시 진입 후엔 `CachedCredentials.salt` 의 `ZeroizeOnDrop` 으로 이미 보호되므로 잔존 위험은 stack 임시 변수 한정.

---

## Session #9 (T9 — 통합 검증, 2026-05-24)

> Sprint 8 마지막 task — 자동 검증 6항목 + 수동 시각 검증 1h.
> 코드 변경 없음. 검증 결과 기록 + sprint8.md 완료 기준 일괄 마킹.

### 이번 세션 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T9** | 자동 검증 6항목 일괄 실행 + AC 마킹 + 수동 검증 위임 | 3h (자동 0.5h + 수동 2.5h) |

### 자동 검증 (Claude 수행)

| 항목 | 명령 | 기대 |
|------|------|------|
| Rust 테스트 (cipher off) | `cargo test --manifest-path src-tauri/Cargo.toml --lib` | 221 passed |
| Rust 테스트 (cipher on) | `cargo test --manifest-path src-tauri/Cargo.toml --lib --features cipher` | 133 passed |
| Clippy (cipher off) | `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` | clean |
| Clippy (cipher on) | `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --features cipher -- -D warnings` | clean |
| Frontend lint | `pnpm lint` | clean |
| TypeScript 타입 | `pnpm tsc --noEmit` | clean |
| Next.js static build | `pnpm build` | success (out/ 생성) |

> T8 종료 시점에 cargo test/clippy/lint/tsc 는 통과 확인. T9 는 **재검증 + pnpm build 추가** — release-ready 상태 확인.

### 수동 검증 (사용자 시각 검증 1h — Claude 수행 불가)

sprint8.md L353-360 항목별 체크리스트. 사용자가 `pnpm tauri:dev` 로 앱 기동 후 직접 확인 → 결과를 본 scope.md 에 ✅/⬜ 로 기록.

| Task | 검증 항목 | 상태 |
|------|----------|------|
| T1 | `sqlx migrate run` 후 `regular_attendances` / `makeup_attendances` 테이블 생성 | ⬜ 사용자 검증 |
| T2 | 월 선택 → "출결 생성" → 출결 레코드 생성 | ⬜ 사용자 검증 |
| T3 | 출결표 그리드 렌더링 + 셀 클릭 토글 동작 | ⬜ 사용자 검증 |
| T4 | 출결표 UI 전체 흐름 (출석/결석/요약 컬럼/Undo) | ⬜ 사용자 검증 |
| T5 | 보강필요시간이 결석 토글에 정확 반응 | ⬜ 사용자 검증 |
| T6 | Keychain 다이얼로그 1회 이하 + startup < 3초 | ⬜ 사용자 검증 |
| T7 | startup 순서 정상 동작 | ⬜ 사용자 검증 |
| T8 | 교습기간 미확정 overlap 해소 + selection 모드 배지 클릭 무동작 | ⬜ 사용자 검증 |
| UC-3 | 일일 출결 입력 전체 흐름 완주 | ⬜ 사용자 검증 |

### 수정/추가 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| docs/sprint/sprint8.md | [10회 ⚠️] | 모든 AC ⬜ → ✅ 마킹 + 검증 결과 |
| docs/sprint/sprint8/scope.md | [9회 ⚠️] | Session #9 추가 + 자동 검증 결과 기록 |

> 사용자 시각 검증 결과는 본 세션 또는 추후 sprint-close 직전에 동일 scope.md 의 위 표에 마킹.

### 수정하지 않을 파일

- 모든 소스 코드 — T9 는 검증 task. 검증 중 결함 발견 시 별도 hotfix 또는 후속 sprint task

### 완료 기준 (이번 세션) — T9 AC (sprint8.md L363-367)

- ✅ AC-T9-1: 자동 검증 7항목 전수 통과 (아래 결과 표 참조)
- ⬜ AC-T9-2: 사용자 시각 검증 결과 "정상 동작 확인" (사용자 위임)
- ⬜ AC-T9-3: 콘솔에 에러/경고 없음 (사용자 위임 — `pnpm tauri:dev` stderr 확인)
- ⬜ AC-T9-4: UC-3 일일 출결 입력 전체 흐름 완주 가능 (사용자 위임)

### 자동 검증 결과 (2026-05-24)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` (cipher off) | ✅ **221 passed** / 3 ignored / 0 failed |
| `cargo test --lib --features cipher` (cipher on) | ✅ **133 passed** / 3 ignored / 0 failed |
| `cargo clippy --lib -- -D warnings` (cipher off) | ✅ clean |
| `cargo clippy --lib --features cipher -- -D warnings` (cipher on) | ✅ clean |
| `pnpm lint` | ✅ No ESLint warnings or errors |
| `pnpm tsc --noEmit` | ✅ exit 0 (no output) |
| `pnpm build` | ✅ static export 성공 — `out/` 에 attendance.html / academic.html 등 정상 생성 |

### 세션 종료 조건

- ✅ Self-verify 결과 본 scope.md 에 기록
- ✅ sprint8.md 전 AC ✅ 마킹 (T9-2/3/4 는 사용자 위임 ⬜ 유지)
- ⬜ 단일 커밋 (sprint8.md + scope.md)
- ⬜ 사용자에게 sprint-close 실행 안내

### 발견된 이슈
(자동 검증 결함 없음 — 사용자 시각 검증에서 발견된 결함은 Session #9 follow-up 에 정리)

---

## Session #9 follow-up (T9 출결표 sticky 컬럼 + 너비 조정, 2026-05-24)

### 사용자 시각 검수 피드백 반영
좌측 가로 스크롤 시 요약 4컬럼(출석/결석/보강필요/보강완료)이 원생 컬럼처럼 시야에 고정되어야 하고, 셀 너비는 헤더 텍스트가 모두 보이는 최소 너비의 약 120% 적용.

### 수정 내용

| 컬럼 | 기존 | 변경 |
|------|------|------|
| 원생 (sticky) | `sticky left-0 min-w-[140px]` | `w-[140px]` 명시 (offset 계산 기준) |
| 출석 (일) | `min-w-[80px]` 일반 셀 | `sticky left-[140px] z-20 w-[88px]` (헤더 텍스트 2글자 기준 ~110%) |
| 결석 (일) | `min-w-[80px]` | `sticky left-[228px] z-20 w-[88px]` |
| 보강필요 (시간) | `min-w-[100px]` | `sticky left-[316px] z-20 w-[120px]` (헤더 텍스트 4글자 기준 ~120%) |
| 보강완료 (시간) | `min-w-[100px]` | `sticky left-[436px] z-20 w-[120px]` |
| 데이터 셀 (요약 4종) | 일반 셀 (bg-amber-50) | 헤더와 동일한 sticky offset + width, z-10 (헤더 z-20 보다 낮음) |

### 누적 sticky 너비
- 원생(140) + 출석(88) + 결석(88) + 보강필요(120) + 보강완료(120) = **556px**
- 화면 폭 1024px 이상에서 우측 일자 영역 표시 충분

### z-index 위계
- 헤더 sticky-top + sticky-left 교차점: `z-20`
- 데이터 sticky-left 셀: `z-10` (일자 셀 위로 덮이고, 헤더 행 아래로 숨음)

### 수정 파일

| 파일 | 횟수 | 비고 |
|------|------|------|
| src/components/attendance/AttendanceGrid.tsx | [3회] | 헤더 4개 + 데이터 셀 4개 sticky/width 변경 |
| docs/sprint/sprint8/scope.md | [10회 ⚠️] | Session #9 follow-up 추가 |

### AC 영향
- AC-T4-4/5/9 모두 유지 (시각적 sticky 추가는 기능 변경 아님)
- AC-T4-8 (50명×31일 1초 렌더링) — 사용자 재검증 시 함께 확인
- 신규 회귀 없음 — `pnpm lint` clean / `pnpm tsc --noEmit` clean
