---
Sprint: 6  |  Date: 2026-05-22  |  Session: #2
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — 백엔드 IPC 세션.
> T5(교습기간 6 IPC) + T6(학사 일정 코드 4 IPC) — academic.rs 신규 모듈.
> 예상 7h. UI(T9~T11) 진입 전 필수 기반.

## Session #1 결과 (참고 — 모두 완료)

| Task | 커밋 | 내용 |
|------|------|------|
| ✅ T1 (A20) | `2c5b8a1` | lock/page.tsx 재시도 버튼 + lockStatus 초기화 |
| ✅ T3 (A21) | `c2be584` | paths.rs OnceLock → cfg(test) thread_local 분리. cargo test 130 passed, 5회 안정 |
| ✅ T4 (A22, R26) | `83f19d1` | 코드 DnD 방법 B (전체 codes 재구성 후 1..N) |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T5** | 교습기간 CRUD 6 IPC (create/update/list/get/confirm/delete_study_period) — 일자 중첩 검증 + 지난 달 차단 | 4h |
| **T6** | 학사 일정 코드 4 IPC (list/create/update_schedule_code, toggle_active) — 시스템 예약 5종 3속성 변경 차단 | 3h |

> 두 Task 모두 동일 신규 모듈 `academic.rs`에 작성. study_periods·schedule_codes 테이블은 V102(Sprint 2)에서 이미 생성됨 — 스키마 변경 없음.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/academic.rs | [1회] | **신규 모듈** — T5+T6 IPC 10개 + 단위 테스트 |
| src-tauri/src/commands/mod.rs | [1회] | `pub mod academic;` 한 줄 추가 |
| src-tauri/src/lib.rs | [1회] | invoke_handler 에 10개 커맨드 등록 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/migrations/` — 본 세션 마이그레이션 변경 없음 (V301 시드는 T2, 다른 세션)
- [ ] `package.json` / `Cargo.toml` — 신규 의존성 없음
- [ ] `src/` 프론트엔드 — IPC 래퍼는 T8(다른 세션)

## 완료 기준 (이번 세션)

### T5 — study_periods (PRD §4.4.2 / §6.2)
- ✅ AC-T5-1: 교습기간 생성 시 **일자 중첩 검증** — 중첩 시 한국어 에러 반환
- ✅ AC-T5-2: **지난 달** 교습기간 수정/삭제 차단 (year_month < 현재 월 또는 is_closed=1)
- ✅ AC-T5-3: 교습기간 확정 후 `is_confirmed = 1`
- ✅ AC-T5-4: 단위 테스트 — 중첩 검증, 지난 달 차단, CRUD smoke

### T6 — schedule_codes (PRD §4.4.3~4.4.5)
- ✅ AC-T6-1: 시스템 예약 코드 5종의 **3속성 수정 시도 차단** (한국어 에러)
- ✅ AC-T6-2: 시스템 예약 코드의 **활성/비활성 토글은 허용**
- ✅ AC-T6-4: `code_name` UNIQUE 위반 시 한국어 에러
- ✅ AC-T6-5: 단위 테스트 — 시스템 코드 보호, 사용자 코드 CRUD
- *(AC-T6-3 보수적 디폴트는 프론트엔드 책임 — T11에서 검증)*

### 세션 종료 조건
- ✅ T5+T6 커밋 `c8dc3c8` — academic.rs 한 파일에 두 도메인 통합. 분할 대신 단일 커밋 채택(같은 신규 파일 변경)
- ✅ Self-verify: `cargo test` **136 passed** (130 → +6 academic), `cargo clippy -- -D warnings` clean
- ✅ simplify 스킬 1회 실행 (변경 사항 없음 — `.map_err(AppError::Db).map_err(String::from)?` 반복은 기존 모듈과의 일관성 우선이라 헬퍼 추출 보류)

## 코드 패턴 SSOT (메모리에서 발췌)

- 시그니처: `pub async fn xxx(...) -> Result<T, String>`
- 풀: `let pool = db::pool().map_err(String::from)?;`
- 에러: `.map_err(AppError::Db).map_err(String::from)?;`
- 응답 struct: `Serialize + from_row(&SqliteRow)` 패턴 (참고: `schedules::StudentSchedule`)
- 테스트: `#[cfg(not(feature = "cipher"))] + #[tokio::test]` + `db::test_pool_in_memory()` (참고: `schedules.rs:207~`)
- 시스템 예약 5종: V102 시드로 이미 존재 → 테스트에서 별도 INSERT 불필요

## 발견된 이슈

> 코드 수정 중 예상 외 충돌·구조 발견 시 여기에 기록 후 사용자에게 보고 (step-back 프로토콜).

(없음 — Session #2 정상 종료)
