---
Sprint: 7  |  Date: 2026-05-22  |  Session: #8
---

> Sprint 7 Session #8 — T8 단독 (교습기간 삭제 cascade).
> Issue 6 carry-over 해소. T6 완료로 unblock. 예상 4h.

## 이전 세션 결과

- Session #1 (`8eb1c92`): T1 — Keychain 통합 캐싱
- Session #2 (`4178324`): T2 — salt.bin 이전 + 보안 패치 6건
- Session #3 (`2fad4fb`): T3 — device_id 영속화
- Session #4 (`6b5f8de`): T4 — is_system_reserved JOIN
- Session #5 (`ba7ef09`): T5 — 코드 관리 /settings 이동
- Session #6 (`2405ca5`): T6 — 교습기간 UX 재설계
- Session #7 (`84aa86f`): T7+T9 — 배치 제약 + 공휴일 삭제 차단

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T8** | 교습기간 삭제 + cascade 삭제 (공휴일 제외) (Issue 6) | 4h |

> 사용자 결정 (2026-05-22): Session #8 = T8 단독.

## 설계 결정 (T8)

### 백엔드 — 신규 IPC 2개 추가 (기존 `delete_study_period` 보존)

기존 `delete_study_period` 는 미확정 전용 — 이름·동작 유지하여 호환성 보장.
T8 의 cascade 삭제는 별도 IPC 2개로 추가:

**1. `get_cascade_delete_preview(id) -> CascadeDeletePreview`**

```rust
pub struct CascadeDeletePreview {
    pub affected_count: i64,    // 삭제될 schedule_events (공휴일 제외) 건수
    pub holiday_count: i64,     // 보존되는 공휴일 건수
    pub deletable: bool,        // 삭제 가능 여부 (confirmed + 지난 달 아님)
    pub reason: Option<String>, // 불가 사유 (한국어)
}
```

- 가드: 교습기간이 존재 + `is_confirmed=1` + 시작 월이 현재 월 이상 (지난 달 아님)
- 영향 건수 계산: `[start_date, end_date]` 범위 내 `schedule_events` 중 `c.code_name != '공휴일'` 카운트
- 공휴일 보존 건수: `[start_date, end_date]` 범위 내 공휴일 카운트

**2. `delete_study_period_cascade(id) -> ()`**

- preview 와 동일 가드 사전 검증
- 트랜잭션 안에서:
  1. `DELETE FROM schedule_events WHERE event_date BETWEEN ? AND ? AND code_id IN (SELECT id FROM schedule_codes WHERE code_name != '공휴일')`
  2. `DELETE FROM study_periods WHERE id = ?`
- 공휴일은 시스템 시드 — 캘린더에 그대로 표시되어야 하므로 보존

### 프론트엔드

**타입** (`src/types/academic.ts`):
- `CascadeDeletePreview` 인터페이스 추가

**IPC 래퍼** (`src/lib/tauri/index.ts`):
- `getCascadeDeletePreview(id) -> Promise<CascadeDeletePreview>`
- `deleteStudyPeriodCascade(id) -> Promise<void>`
- dev fallback: preview 는 `{ affected_count: 0, holiday_count: 0, deletable: false, reason: '개발 모드' }`

**UI** (`src/components/academic/StudyPeriodEditor.tsx`):
- 확정 월 표시 영역에 "삭제" 버튼 추가 — 단, `start_date` 가 현재 월 이상일 때만 노출
- 클릭 시 `getCascadeDeletePreview` 호출 → AlertDialog 영향 건수 표시
- "교습기간을 삭제하면 공휴일을 제외한 N건의 학사 일정이 함께 삭제됩니다. 보존되는 공휴일 M건. 삭제하시겠습니까?"
- 확인 시 `deleteStudyPeriodCascade` 호출 → invalidate `['study-period']`, `['study-periods']`, `['schedule-events']`

### 신규 의존성
- 없음.

### lib.rs 등록
- 신규 IPC 2개 `invoke_handler` 에 추가.

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/academic.rs | [2회] | `CascadeDeletePreview` + 2개 IPC + 단위 테스트 |
| src-tauri/src/lib.rs | [1회] | 신규 IPC 2개 등록 |
| src/types/academic.ts | [1회] | `CascadeDeletePreview` 인터페이스 |
| src/lib/tauri/index.ts | [2회] | 래퍼 2개 + dev fallback |
| src/components/academic/StudyPeriodEditor.tsx | [7회 ⚠️] | 삭제 버튼 + AlertDialog |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/migrations/` — 스키마 변경 없음 (기존 schedule_events 활용)

## 완료 기준 (이번 세션)

### T8 — 교습기간 삭제 cascade (sprint7.md L254-260)
- ✅ AC-T8-1: 확정 교습기간 cascade — `code_name != '공휴일'` SQL 분기 (`cascade_delete_preserves_holidays`)
- ✅ AC-T8-2: 공휴일은 cascade 에서 보존 (테스트로 검증)
- ✅ AC-T8-3: `getCascadeDeletePreview` 호출 후 AlertDialog 에 affected_count + holiday_count 표시
- ✅ AC-T8-4: `confirmedPeriod.year_month >= currentYearMonth()` 일 때만 삭제 버튼 노출
- ✅ AC-T8-5: cascade 후 study_periods row 삭제 → 동일 월 재확정 가능 (기존 흐름)
- ✅ AC-T8-6: 단위 테스트 4건 신규 (cascade_guard_rejects_unconfirmed_period / rejects_past_month / preserves_holidays / preview_counts_match)

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off 177 / on 127, clippy clean (양쪽), pnpm lint clean, pnpm tsc clean
- ✅ simplify — `check_cascade_delete_guard` 헬퍼로 preview/cascade 양쪽 가드 공유, 트랜잭션은 schedule_events + study_periods 두 DELETE 만
- ⬜ 단일 커밋 (5파일 + scope.md)

## 발견된 이슈

(없음 — Step-back 트리거 발생 시 여기에 기록)

## carry-over

- Session #2 발견 9건 (I-S2-2 ~ I-S2-10) — 후속 hotfix
- Session #4 발견 1건 (I-S4-1) — 후속
