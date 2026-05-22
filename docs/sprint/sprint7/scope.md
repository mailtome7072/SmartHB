---
Sprint: 7  |  Date: 2026-05-22  |  Session: #7
---

> Sprint 7 Session #7 — T7 + T9 묶음 (백엔드 가드 일괄 강화).
> Issue 4/R34 + Issue 7 carry-over 해소. 예상 6h.

## 이전 세션 결과

- Session #1 (`8eb1c92`): T1 — Keychain 통합 캐싱
- Session #2 (`4178324`): T2 — salt.bin 이전 + 보안 패치 6건
- Session #3 (`2fad4fb`): T3 — device_id 영속화
- Session #4 (`6b5f8de`): T4 — is_system_reserved JOIN
- Session #5 (`ba7ef09`): T5 — 코드 관리 /settings 이동
- Session #6 (`2405ca5`): T6 — 교습기간 UX 재설계

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T7** | 학사 일정 배치 제약 강화 — 중복불가 상호 차단 + 교습기간 내만 (Issue 4/R34) | 4h |
| **T9** | 공휴일 삭제 차단 (Issue 7) | 2h |

> 사용자 결정 (2026-05-22): T7+T9 묶음. 두 Task 모두 academic.rs 백엔드 가드 강화 + 프론트 에러 메시지 — 한 세션으로 통합 효율적.

## 설계 결정

### T7 — 배치 제약 강화 (sprint7.md L203-229)

`create_schedule_event` + `update_schedule_event` 양쪽 동일 검증 추가:

**제약 1: 중복불가 상호 차단** (sprint7.md L210-211)
- **현재** (Sprint 6): `is_duplicate_blocked=1` 코드는 동일 `code_id` + 동일 `event_date` 만 차단
- **변경**: `is_duplicate_blocked=1` 코드 배치 시, 해당 일자에 **어떤 다른 일정도** 있으면 차단
- **역방향**: 해당 일자에 이미 `is_duplicate_blocked=1` 일정이 있으면 새 코드(중복불가/허용 무관) 배치도 차단
- 한국어 에러: "중복불가 코드는 다른 일정이 있는 날짜에 배치할 수 없습니다" / "해당 일자에 중복불가 일정이 있어 배치할 수 없습니다"

**제약 2: 교습기간 내만 배치** (sprint7.md L212)
- `event_date` 가 어떤 확정된 교습기간(`is_confirmed=1`) 의 `[start_date, end_date]` 안에 있어야 허용
- 기간성 코드의 `period_end_date` 도 동일 교습기간 내에 있어야 함
- 한국어 에러: "학사 일정은 확정된 교습기간 내 일자에만 배치할 수 있습니다"

**예외**: 시스템 시드된 공휴일(V301) 은 교습기간과 무관하게 미리 시드된 데이터 — 본 IPC 가드는 사용자 명시 배치 호출에만 적용. 시드 INSERT 는 마이그레이션 직접 실행이라 영향 없음.

### T9 — 공휴일 삭제 차단 (sprint7.md L264-286)

`delete_schedule_event` 가드 추가:
- 삭제 대상 이벤트의 코드가 `is_system_reserved=1` **AND** `code_name='공휴일'` 이면 차단
- 단원평가 자동 배치 이벤트는 수동 삭제 허용 (is_system_reserved=1 이지만 공휴일 아님)
- 한국어 에러: "공휴일은 삭제할 수 없습니다."

**프론트 가드** (`CalendarCell.tsx`):
- 공휴일 배지 클릭 시 삭제 다이얼로그 차단 — `event.is_system_reserved && event.code_name === '공휴일'` 조건으로 onEventClick 호출 자체 차단 또는 토스트 안내

### 기존 단위 테스트
- `cargo test create_schedule_event_*` 류 다수 존재 — 시그니처 변경 없으므로 신규 가드만 추가 검증
- 신규 테스트: 중복불가 상호 차단, 교습기간 외 차단, 공휴일 삭제 차단

### 신규 의존성
- 없음.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/academic.rs | [4회 ⚠️] | T7 가드 + T9 가드 + 단위 테스트 |
| src/components/academic/CalendarCell.tsx | [1회] | 공휴일 배지 삭제 UI 가드 |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/migrations/` — 스키마 변경 없음
- [ ] `src/types/academic.ts`, `src/lib/tauri/index.ts` — 시그니처 변경 없음 (IPC 인터페이스 동일, 에러 메시지만 변경)

## 완료 기준 (이번 세션)

### T7 — 배치 제약 강화 (sprint7.md L223-229)
- ✅ AC-T7-1: 중복불가 코드 배치 시 다른 일정 존재 일자에서 차단 (`placement_blocks_when_dup_blocked_meets_other_event`)
- ✅ AC-T7-2: 역방향 차단 (`placement_blocks_when_target_date_has_dup_blocked_event`)
- ✅ AC-T7-3: 교습기간 외 일자 배치 차단 (`placement_blocks_when_date_outside_study_period`)
- ✅ AC-T7-4: `update_schedule_event` 도 동일 가드 + 자기 자신 row 제외 (`placement_excludes_self_event_on_update`)
- ✅ AC-T7-5: 공휴일이 이미 배치된 일자 — V301 공휴일은 `is_duplicate_blocked=1` 시드라 AC-T7-2 로 자동 차단
- ✅ AC-T7-6: 단위 테스트 4건 신규 (placement_*)

### T9 — 공휴일 삭제 차단 (sprint7.md L282-286)
- ✅ AC-T9-1: 공휴일 시스템 코드 이벤트 삭제 시 백엔드 가드 차단 (`delete_event_blocks_holiday_system_code`)
- ✅ AC-T9-2: 공휴일 배지 클릭 차단 + title "공휴일은 삭제할 수 없습니다" (CalendarCell.tsx EventBadge)
- ✅ AC-T9-3: 비공휴일 시스템 코드(단원평가 등) 삭제 허용 (`delete_event_allows_non_holiday_system_code`)
- ✅ AC-T9-4: 단위 테스트 2건 신규

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off 173 / on 127 passed, clippy clean (양쪽), pnpm lint clean, pnpm tsc clean
- ✅ simplify — `check_placement_constraints` 헬퍼로 중복 로직 분리, create/update 양쪽 공유
- ⬜ 단일 커밋 (2파일 + scope.md)

## 발견된 이슈

(없음 — Step-back 트리거 발생 시 여기에 기록)

## carry-over

- Session #2 발견 9건 (I-S2-2 ~ I-S2-10) — 후속
- Session #4 발견 1건 (I-S4-1: CalendarCell hasHoliday/hasAssessment) — 후속
