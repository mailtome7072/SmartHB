---
name: sprint-next-session
description: "Sprint 6 Session #1 완료(T1/T3/T4 회고 carry-over 3건 해소, 2026-05-22). 다음: /sprint-dev 6 재진입 → Session #2 (T5+T6 academic.rs 신설)"
metadata: 
  node_type: memory
  type: project
  originSessionId: ec4dbd04-fb9a-48c2-82eb-4dab55bbfa1a
---

Sprint 6 Session #1 종료 상태. **브랜치 `sprint6` 가 origin 에 push 완료** (https://github.com/mailtome7072/SmartHB/tree/sprint6). 새 대화에서 `/sprint-dev 6` 재입력만 하면 동일 브랜치에서 이어갈 수 있다.

## Session #1 완료 (3 Task 회고 carry-over 모두 해소)

| Task | 커밋 | 영향 파일 |
|------|------|----------|
| ✅ T1 (A20) | `2c5b8a1` | `src/app/lock/page.tsx` — 재시도 버튼 + `setLockStatus(null)` |
| ✅ T3 (A21) | `c2be584` | `src-tauri/src/commands/paths.rs` — storage 모듈 cfg 분기. **`cargo test --test-threads` 제한 제거 가능** (130 passed, 5회 연속 안정) |
| ✅ T4 (A22, R26) | `83f19d1` | `src/app/settings/codes/page.tsx` — DnD 방법 B (전체 codes 재구성 후 1..N 재부여) |

scope.md 갱신 커밋: `ea4e8c1`.

## Session #2 진입 시 우선 액션

1. `/sprint-dev 6` 재진입 (사용자 직접 입력)
2. `docs/sprint/sprint6/scope.md` 읽고 **Session 번호 +1 (#2)**, 수정 횟수 [0회] 리셋
3. T5+T6 작업 — **academic.rs 신규 모듈** 신설

## T5+T6 구체 작업 안내 (다음 세션 가이드)

### 구조
- 신규 파일: `src-tauri/src/commands/academic.rs`
- `src-tauri/src/commands/mod.rs`: `pub mod academic;` 추가
- `src-tauri/src/lib.rs` invoke_handler: 10개 커맨드 등록

### 도메인 검증
- 스키마 SSOT: `src-tauri/migrations/102__create_study_periods_and_schedule_codes.sql`
  - `study_periods`: id / year_month UNIQUE / start_date / end_date / is_confirmed / is_closed
  - `schedule_codes`: id / code_name UNIQUE / is_system_reserved / 3속성(allows_regular_class/allows_makeup_class/is_duplicate_blocked) / is_period_type / is_active
- 시스템 예약 5종 시드: 보강데이/공휴수업일/방학/단원평가 응시일/휴원일 (V102 INSERT)

### T5 — 6 IPC (study_periods, 4h)
1. `create_study_period({ year_month, start_date, end_date })` — **일자 중첩 검증**: `NOT (end_date < new.start_date OR start_date > new.end_date)`. 중첩 시 한국어 에러.
2. `update_study_period(id, { start_date, end_date })` — **지난 달 차단** (year_month < current month 또는 is_closed=1).
3. `list_study_periods(from_month, to_month)` — 범위 조회.
4. `get_study_period(year_month)` — 단일.
5. `confirm_study_period(id)` — is_confirmed=1.
6. `delete_study_period(id)` — is_confirmed=0 인 경우만.

### T6 — 4 IPC (schedule_codes, 3h)
1. `list_schedule_codes()` — 전체 (is_active 포함).
2. `create_schedule_code({ code_name, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type })` — 사용자 추가 코드. 보수적 디폴트(OFF/OFF/ON)는 **프론트엔드**가 결정.
3. `update_schedule_code(id, ...)` — `is_system_reserved=1` 행은 차단 (AC-4.4-5).
4. `toggle_schedule_code_active(id)` — 시스템 코드도 활성/비활성 토글은 허용.

### 코드 패턴 (기존 모듈 참고)
- **시그니처**: `pub async fn xxx(...) -> Result<T, String>`
- **풀**: `let pool = db::pool().map_err(String::from)?;`
- **에러**: `.map_err(AppError::Db).map_err(String::from)?;`
- **응답 struct**: `Serialize + from_row(&SqliteRow)` 패턴 (참고: `schedules::StudentSchedule`)
- **테스트**: `#[cfg(not(feature = "cipher"))]` + `db::test_pool_in_memory()` (참고: `src-tauri/src/commands/schedules.rs:207~`)

### 단위 테스트 우선순위
1. 일자 중첩 검증 (T5 핵심 비즈니스 규칙, PRD §6.2)
2. 지난 달 수정 차단 (AC-4.4-1)
3. 시스템 예약 코드 3속성 변경 차단 (AC-4.4-5)
4. CRUD 정상 동작 smoke test

### 신규 의존성
- 없음. `tsx`(T2용)는 다음 세션의 다음(T2 진입 시)에 검토.

### 마이그레이션
- 본 세션(T5+T6) 범위에서는 변경 없음. V102 기존 스키마 그대로 활용.

## Sprint 6 전체 진행률

- 완료: 3/12 Task (T1, T3, T4 — 회고 carry-over 기술 부채)
- 다음 세션: T5+T6 (백엔드 IPC, 예상 7h)
- 남은: T2(시드+공휴일 7h) + T7~T12 (총 ~28h)

## Sprint 5 완료 산출물 (참고)

- `docs/sprint/sprint5.md` + `docs/sprint-retrospectives/sprint5-retrospective.md` + `docs/test-reports/sprint5-test-report.md`
- CHANGELOG.md `[0.2.1]`

## 정책 (재확인)

- **PR 단계 생략** ([[workflow-no-pr]]) — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **Forbidden Area**: `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/`
- **새 의존성 추가 시 사용자 허가 필수**

본 메모리는 Sprint 6 모든 Task 종료 후 sprint-close 시점에 다음 sprint-next-session으로 슬러그 갱신.
