---
Sprint: 6  |  Date: 2026-05-22  |  Session: #3
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — 백엔드 IPC 마지막 세션.
> T7(학사 일정 배치 5 IPC) — academic.rs 확장.
> 예상 5h. T8 IPC 래퍼·T9 캘린더 진입 직전 마지막 백엔드 단계.

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 (A20 lock 재시도) | `2c5b8a1` |
| #1 | T3 (A21 paths.rs OnceLock 분리) | `c2be584` |
| #1 | T4 (A22 DnD 방법 B) | `83f19d1` |
| #2 | T5+T6 (academic.rs 신규 — study_periods 6 + schedule_codes 4) | `c8dc3c8` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T7** | schedule_events IPC 5개 — create/update/delete/list + auto_place_assessment_dates | 5h |

> academic.rs 기존 파일 확장(신규 모듈 아님). schedule_events 테이블은 V103(Sprint 2) — 스키마 변경 없음. lib.rs에 5 커맨드 추가 등록.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/academic.rs | [3회 ⚠️] | T7 — schedule_events 섹션 + ScheduleEvent struct + ScheduleEventListItem(평탄화) + 5 IPC + 단위 테스트 |
| src-tauri/src/lib.rs | [1회] | invoke_handler 에 5 커맨드 추가 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/migrations/` — 본 세션 마이그레이션 변경 없음
- [ ] `package.json` / `Cargo.toml` — 신규 의존성 없음
- [ ] `src/` 프론트엔드 — 래퍼는 T8(다른 세션)
- [ ] `src-tauri/src/commands/academic.rs` 기존 T5/T6 코드 — 추가만, 기존 부분 수정 금지

## 완료 기준 (이번 세션)

### T7 — schedule_events (PRD §4.4.4, §4.4.6, §4.4.7)
- [ ] AC-T7-1: 중복불가 코드(`is_duplicate_blocked=1`) 동일 일자 배치 시도 차단 + 한국어 에러
- [ ] AC-T7-2: 기간성(`is_period_type=1`) 코드는 `period_end_date` 필수, 단일 일자 코드는 `period_end_date=NULL` 강제
- [ ] AC-T7-3: 지난 달 일정 수정/삭제 차단 (event_date year-month < 현재 월)
- [ ] AC-T7-4: `auto_place_assessment_dates(year_month)` 가 study_period 안에서 **2주차 월~금 + 4주차 월~금** 정확히 INSERT
- [ ] AC-T7-5: 이미 해당 month 에 단원평가 1건 이상 존재 시 자동 배치 No-op (AC-4.4-6)
- [ ] AC-T7-6: 단위 테스트 — 중복불가 / 기간성 / 지난 달 차단 / 자동 배치 / 재실행 No-op

### 세션 종료 조건
- ✅ T7 단일 커밋 `a4c380e` (academic.rs +446줄 + lib.rs 5 커맨드 등록)
- ✅ Self-verify: `cargo test` **141 passed** (136 → +5 T7), `cargo clippy -- -D warnings` clean
- ✅ simplify 스킬 1회 실행 (변경 없음 — T5/T6 패턴 일관성. 헬퍼 `assessment_dates_for/year_month_of/find_assessment_code_id` 는 책임 분리·테스트 가능성으로 정당화)

## 설계 결정 (메모리 가이드 따름)

- **list_schedule_events 응답**: 평탄화 `ScheduleEventListItem { id, code_id, code_name, is_duplicate_blocked, is_period_type, event_date, period_end_date, display_name }` — 프론트 캘린더 셀 JSON 처리 편의
- **자동 배치 2/4주차 계산**: study_period 의 `start_date` 기준 주차 계산. ISO 주차(월요일 시작) 사용. chrono::NaiveDate 활용.
- **단원평가 코드 ID 조회**: 시드 `code_name = '단원평가 응시일'` 의 id 를 매번 SELECT. 트랜잭션 내 1회 캐싱.
- **트랜잭션**: auto_place_assessment_dates 는 study_period 조회 + 단원평가 코드 조회 + 기존 단원평가 카운트 + INSERT 10건(2주차5+4주차5)을 하나의 `tx.begin()` 안에서 처리.

## 코드 패턴 SSOT (메모리에서 발췌, T5/T6 그대로)

- 시그니처: `pub async fn xxx(...) -> Result<T, String>`
- 풀: `let pool = db::pool().map_err(String::from)?;`
- 에러: `.map_err(AppError::Db).map_err(String::from)?;`
- 테스트: `#[cfg(not(feature = "cipher"))] + #[tokio::test]` + `db::test_pool_in_memory()`
- V103 schedule_events 인덱스 활용: event_date 단일·범위 조회 빠름

## 발견된 이슈

> 코드 수정 중 예상 외 충돌·구조 발견 시 여기에 기록 후 사용자에게 보고 (step-back 프로토콜).

- 1차 컴파일에서 `chrono::Datelike` trait import 누락(weekday()/num_days_from_monday() trait method) → 단일 라인 `use chrono::Datelike;` 추가로 즉시 통과. 동일 오류 반복 없음 → 3-retry / loop-detection 미적용.
