---
name: sprint-next-session
description: "Sprint 6 Session #2 완료(T5+T6 academic.rs 신설, 2026-05-22). 다음: /sprint-dev 6 재진입 → Session #3 (T7 schedule_events IPC)"
metadata: 
  node_type: memory
  type: project
  originSessionId: ec4dbd04-fb9a-48c2-82eb-4dab55bbfa1a
---

Sprint 6 Session #2 종료. **브랜치 `sprint6` 가 origin 동기화 완료** (https://github.com/mailtome7072/SmartHB/tree/sprint6). 새 대화에서 `/sprint-dev 6` 재입력하면 깨끗한 컨텍스트로 Session #3 진입.

## Sprint 6 진행률: 5/12 (≈42%)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 (A20 lock 재시도) | `2c5b8a1` |
| #1 | T3 (A21 paths.rs OnceLock 분리) | `c2be584` |
| #1 | T4 (A22 DnD 방법 B) | `83f19d1` |
| #2 | **T5+T6** (academic.rs 신규 — study_periods 6 IPC + schedule_codes 4 IPC) | `c8dc3c8` |

검증 현황 (Session #2 종료 시점): `cargo test` 136 passed, `cargo clippy -- -D warnings` clean.

## Session #3 진입 시 우선 액션

1. `/sprint-dev 6` 재진입 (사용자 직접 입력)
2. `docs/sprint/sprint6/scope.md` 읽고 **Session 번호 +1 (#3)**, 수정 횟수 [0회] 리셋
3. **T7 작업** — `academic.rs` 에 schedule_events IPC 5개 추가 (신규 모듈 아님, 기존 파일 확장)

## T7 구체 작업 안내 (다음 세션 가이드)

### 구조
- 기존 파일 확장: `src-tauri/src/commands/academic.rs` — schedule_events 섹션 신설 (// schedule_events (T7) 구분선 이후)
- `src-tauri/src/lib.rs` invoke_handler: 5개 커맨드 추가 등록

### 도메인 스키마 (V103 — Sprint 2)
- `schedule_events(id, code_id FK, event_date, period_end_date?, display_name?, created_at, updated_at)`
- CHECK: `period_end_date IS NULL OR period_end_date >= event_date`
- period_end_date NULL = 단일 일자, NOT NULL = 기간성

### T7 — 5 IPC (5h)
1. `create_schedule_event({ code_id, event_date, period_end_date?, display_name? })`
   - **중복불가 검증** (AC-4.4-4 / AC-T7-1): `is_duplicate_blocked=1` 코드는 동일 `event_date` 에 이미 존재하면 차단. schedule_codes JOIN 필요.
   - **기간성 일관성** (AC-T7-2): `is_period_type=1` 인 코드는 period_end_date 필수 (None 거부), `is_period_type=0` 은 period_end_date None 강제.
2. `update_schedule_event(id, ...)` — **지난 달 차단** (AC-T7-3): event_date 의 year-month < 현재 월 이면 거부.
3. `delete_schedule_event(id)` — 지난 달 차단 동일.
4. `list_schedule_events(from_date, to_date)` — 코드 JOIN 으로 코드 정보 포함. 공휴일 이벤트(T2 V301 시드)는 schedule_events 자체에서 같이 조회됨.
5. `auto_place_assessment_dates(year_month)` (AC-T7-4/5):
   - 해당 year_month 의 study_period 조회 → start_date/end_date 안에서 2주차 월~금 + 4주차 월~금 자동 INSERT (단원평가 응시일 code_id 사용).
   - **AC-4.4-6**: 이미 해당 month 에 단원평가가 1건이라도 있으면 No-op (중복 생성 금지).
   - 자동 배치 후 사용자가 드래그 이동/삭제 가능 — IPC 자체에는 후속 편집 권한 무관.

### 응답 struct (academic.rs 에 추가)
```rust
pub struct ScheduleEvent {
    pub id: i64,
    pub code_id: i64,
    pub event_date: String,
    pub period_end_date: Option<String>,
    pub display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    // list_schedule_events 가 JOIN 으로 함께 반환할 코드 정보 — 별도 ScheduleEventWithCode 구조 고려
}
```

list_schedule_events 는 캘린더 렌더링용이므로 코드명·3속성을 함께 반환하는 게 효율적. 두 가지 패턴:
- **A**: `ScheduleEventWithCode { event: ScheduleEvent, code: ScheduleCode }` 중첩
- **B**: 평탄화된 `ScheduleEventListItem { id, code_id, code_name, is_duplicate_blocked, is_period_type, event_date, period_end_date, display_name }`
> 권장 B — 프론트 캘린더 셀 렌더링에 JSON 평탄 구조가 편함.

### 단위 테스트 우선순위 (AC-T7-6)
1. 중복불가 코드 동일 일자 INSERT 거부 (AC-T7-1)
2. 기간성 코드의 period_end_date 필수 검증 (AC-T7-2)
3. 지난 달 update/delete 차단 (AC-T7-3)
4. auto_place_assessment_dates 가 2주차/4주차 월~금 정확히 생성 (AC-T7-4)
5. 자동 배치 재실행 시 No-op (AC-T7-5)

### 코드 패턴 (academic.rs 의 기존 T5/T6 패턴 그대로 따름)
- 시그니처: `pub async fn ... -> Result<T, String>`
- pool: `db::pool().map_err(String::from)?;`
- 에러: `.map_err(AppError::Db).map_err(String::from)?;`
- 테스트: `#[cfg(not(feature = "cipher"))] + #[tokio::test]` + `db::test_pool_in_memory()` + V102 시스템 5종 시드 활용

### 마이그레이션
- 본 세션(T7) 범위에서는 변경 없음. V103 스키마 그대로 활용.

## Sprint 6 남은 작업 (T7 이후)

| Task | 작업 | 의존성 | 권장 세션 묶음 |
|------|------|--------|---------------|
| T7 | schedule_events IPC | T5+T6 (충족) | Session #3 단독 |
| T2 | V301 시드 + 공휴일 + ADR | 없음 | Session #4 (brainstorming skill) |
| T8 | IPC 래퍼 + 도메인 타입 (프론트) | T5+T6+T7 | Session #5 (T7 다음, 작음) |
| T9 | 3개월 캘린더 컴포넌트 | T2+T8 | Session #6 (frontend-design skill) |
| T10 | 교습기간 설정 UI | T9 | Session #7 |
| T11 | 일정 코드 + 배치 UI | T9+T10 | Session #8 (frontend-design skill) |
| T12 | 통합 검증 | 전부 | Session #9 (마지막) |

## Sprint 5 완료 산출물 (참고)

- `docs/sprint/sprint5.md` + `docs/sprint-retrospectives/sprint5-retrospective.md` + `docs/test-reports/sprint5-test-report.md`
- CHANGELOG.md `[0.2.1]`

## 정책 (재확인)

- **PR 단계 생략** ([[workflow-no-pr]]) — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **Forbidden Area**: `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/`
- **새 의존성 추가 시 사용자 허가 필수**

본 메모리는 Sprint 6 모든 Task 종료 후 sprint-close 시점에 다음 sprint-next-session으로 슬러그 갱신.
