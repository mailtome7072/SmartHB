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
| src-tauri/src/commands/audit.rs | [3회 ⚠️] | `AttendanceToggled` variant + as_code 매핑 |
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
| src/components/attendance/AttendanceGrid.tsx | [2회] | 요일 행/시간 변환/컬럼 재배치/배경색 |
| src/lib/menu-config.ts | [8회 ⚠️] | 보강 관리 추가 + 순서 재배치 |
| src/app/attendance/page.tsx | [2회] | (이번 세션에서 직접 수정은 없으나 git status에 잡혀 확인 필요) |

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
