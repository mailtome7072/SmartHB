---
Sprint: 8  |  Date: 2026-05-23  |  Session: #1
---

> Sprint 8 Session #1 — T1 단독 (V106 마이그레이션: regular_attendances + makeup_attendances).
> Phase 2 마감 출결 도메인의 데이터 기반. 예상 3h.

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
- AC-T1-5 (.sqlx) 는 T2 세션에서 query 매크로와 함께 처리
