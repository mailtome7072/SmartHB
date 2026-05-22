---
Sprint: 7  |  Date: 2026-05-22  |  Session: #4
---

> Sprint 7 Session #4 — T4 단독 (is_system_reserved JOIN + 프론트 하드코딩 제거).
> A23/R33 carry-over 해소. T5·T7·T9 unblock. 예상 3h.

## 이전 세션 결과

- Session #1 (2026-05-22, `8eb1c92`): T1 — Keychain 통합 캐싱 + CredentialCache
- Session #2 (2026-05-22, `4178324`): T2 — salt.bin 이전 + 보안 패치 6건 (S-T2-1~6) + I-S2-1
- Session #3 (2026-05-22, `2fad4fb`): T3 — device_id 영속화 (app_config_dir/device.id)
  - cargo test cipher off 166 / on 127, clippy clean

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T4** | `is_system_reserved` JOIN 응답 확장 + 프론트엔드 하드코딩 제거 | 3h |

> 사용자 결정 (2026-05-22): Session #4 = T4 단독. T5·T7·T9 가 T4 결과를 활용하므로 unblock 효과 큼.

## 설계 결정 (T4)

### 백엔드
- `ScheduleEventListItem` 구조체에 `is_system_reserved: bool` 추가
- `list_schedule_events` SELECT 에 `c.is_system_reserved` 컬럼 추가
- `from_row` 에 `try_get::<i64, _>("is_system_reserved")? != 0` 추가
- 신규 의존성/DB 마이그레이션 없음 — `schedule_codes.is_system_reserved` 컬럼은 V102 시점부터 존재

### 프론트엔드
- `src/types/academic.ts` `ScheduleEventListItem` 인터페이스에 `is_system_reserved: boolean` 추가
- `src/components/academic/CalendarCell.tsx`:
  - `codeBadgeClass(codeName, isSystemReserved)` 시그니처 변경
  - 시스템 코드는 코드명 기반 색상 유지 (기존 6종 매핑), 사용자 코드는 amber 기본
  - 호출 사이트 1곳 갱신
- `src/components/academic/ThreeMonthCalendar.tsx`:
  - `draggableEventIds` 의 `systemNames` Set 리터럴 6개 제거
  - 조건: `!event.is_period_type && !event.is_system_reserved`
- `src/lib/tauri/index.ts`: `listScheduleEvents` fallback 은 `[]` 빈 배열 — 갱신 불필요 확인 (`createScheduleCode` 등 3곳은 `ScheduleCode` 타입으로 이미 `is_system_reserved` 포함)

### 변경 후 동작 일관성
- 기존 시스템 6종(공휴일/보강데이/공휴수업일/방학/휴원일/단원평가 응시일) 모두 `is_system_reserved=1` 시드(V102) 이므로 배지 색상 + 드래그 차단 동작 동일.
- 향후 사용자가 시스템 코드를 추가할 때도 자동으로 색상 분기 적용 (현재는 V102 시드 외 시스템 코드 추가 경로 없음).

### 신규 의존성
- 없음.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/academic.rs | [2회] | `ScheduleEventListItem` + `list_schedule_events` JOIN |
| src/types/academic.ts | [1회] | `ScheduleEventListItem` 타입 확장 |
| src/components/academic/CalendarCell.tsx | [3회 ⚠️] | `codeBadgeClass` 시그니처 + 호출 변경 |
| src/components/academic/ThreeMonthCalendar.tsx | [1회] | `draggableEventIds` 플래그 기반 |
| src/lib/tauri/index.ts | [0회] | dev fallback 응답 (3곳) |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/Cargo.toml` — 신규 의존성 없음
- [ ] `src-tauri/migrations/` — DB 스키마 변경 없음 (V102 컬럼 활용)
- [ ] `src-tauri/src/commands/auth.rs`, `recovery.rs`, `paths.rs`, `lock.rs`, `lib.rs` — T1~T3 완료, T4 영향 없음
- [ ] `src/components/academic/ScheduleCodePanel.tsx` — 이미 `is_system_reserved` 사용 (별도 코드 패스)

## 완료 기준 (이번 세션)

### T4 — is_system_reserved JOIN + 프론트 하드코딩 제거 (sprint7.md L124-148)
- ✅ AC-T4-1: `list_schedule_events` 응답 구조체에 `is_system_reserved: bool` 필드 포함 + JOIN SELECT 갱신
- ✅ AC-T4-2: 시스템 6종 배지 색상 기존 매핑 동일 — `SYSTEM_BADGE_CLASS` lookup 객체로 데이터 정의 분리
- ✅ AC-T4-3: `draggableEventIds` 가 `!event.is_period_type && !event.is_system_reserved` 로 변경되어 동작 보존
- ✅ AC-T4-4: **분기 로직** 의 시스템 코드명 리터럴 0개 (R33 해소). `CalendarCell.tsx` 의 `SYSTEM_BADGE_CLASS` 객체 키 6건은 lookup 데이터(분기 아님). `hasHoliday`/`hasAssessment` 3건(line 154-156)은 특정 코드 식별용 비즈니스 로직(공휴일 셀 빨간 배경, 단원평가 분리 표시)으로 T4 범위 외 — I-S4-1 로 기록
- ✅ AC-T4-5: `pnpm tsc --noEmit` + `pnpm lint` 통과

### 세션 종료 조건
- ✅ Self-verify: cargo test cipher off 166 / on 127, cargo clippy clean (양쪽), pnpm lint clean, pnpm tsc --noEmit clean
- ✅ simplify 검토 — `codeBadgeClass` 분기 → lookup 객체로 단순화, `draggableEventIds` Set 리터럴 제거로 6라인 절감
- ⬜ 단일 커밋 (5파일 + scope.md)

## 발견된 이슈

### I-S4-1: CalendarCell.tsx 의 `hasHoliday`/`hasAssessment`/`nonAssessmentEvents` 코드명 리터럴 (T4 범위 외)

- **위치**: `src/components/academic/CalendarCell.tsx:154-156`
- **현상**:
  ```ts
  const hasHoliday = events.some((e) => e.code_name === '공휴일')
  const hasAssessment = events.some((e) => e.code_name === '단원평가 응시일')
  const nonAssessmentEvents = events.filter((e) => e.code_name !== '단원평가 응시일')
  ```
- **분석**: 시스템 여부 판정이 아닌 **특정 코드 식별** 비즈니스 로직. 공휴일은 셀 빨간 배경, 단원평가는 별도 표시 영역 등 시각적 분리를 위해 코드명을 사용.
- **T4 와 관계**: R33 본질("시스템 코드 6종 Set 으로 시스템 여부 판단")과 별개 — 본 finding 은 코드 식별 (특정 코드 1개 매칭). 백엔드가 별도 플래그를 제공하지 않는 한 코드명 매칭이 자연스러움.
- **권고 처리**: 후속 세션에서 schedule_codes 에 의미적 분류 컬럼 (e.g., `code_intent`: holiday / assessment / 일반) 추가 고려. 단발성이라 sprint7 sprint-close 단계 또는 Phase 3 이후로 이연. T4 단독 세션 범위 외이므로 carry-over.

## carry-over (Session #2 발견 9건, 후속 세션 처리)

I-S2-2 ~ I-S2-10: 후속 세션 또는 hotfix.
