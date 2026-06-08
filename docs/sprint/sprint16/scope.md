---
Sprint: 16  |  Date: 2026-06-08  |  Session: #1
---

## 이번 세션 목표
**T0: 수업일 변경 도메인 (케이스1 1회성 이동 + 케이스2 특정일 이후 영구 변경)** — `/sprint-dev 16` 최우선 Task.
백엔드(마이그레이션 → IPC → 테스트) 먼저 완성 후 프론트엔드 UI.

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

### 백엔드 (Rust)
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/306__add_note_to_regular_attendances.sql | [0회] | 신규 — `note TEXT` 컬럼 (케이스1 이동 메모) |
| src-tauri/src/commands/attendance.rs | [0회] | `move_attendance`(케이스1) + `apply_schedule_change`(케이스2) + `generate_impl` 날짜 인식 리팩토링 |
| src-tauri/src/commands/audit.rs | [0회] | `AttendanceRescheduled` / `ScheduleChangedWithRegen` variant 추가 |
| src-tauri/src/lib.rs | [0회] | 신규 커맨드 invoke_handler 등록 |
| src-tauri/.sqlx/ | [0회] | 오프라인 캐시 갱신 (query! 사용 시) |

### 프론트엔드 (TypeScript/React)
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/lib/tauri/index.ts | [0회] | `moveAttendance` / `applyScheduleChange` IPC 래퍼 + dev fallback |
| src/types/attendance.ts (또는 schedule.ts) | [0회] | 이동/재생성 요청·응답 타입 |
| src/components/attendance/* | [0회] | 출결표 셀 액션 "수업일 이동" + 도착일 달력 다이얼로그 (ThreeMonthCalendar 패턴 재활용) |
| src/components/schedules/* | [0회] | ScheduleEditor 변경일 날짜 선택 + 재생성 확인 다이얼로그 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/src/commands/billing.rs — 청구 로직 자체 불변 (PI-22: 토스트 안내만, 금액 수동 조정). 변경 전/후 주당시간은 apply_schedule_change 응답으로 전달
- [ ] src-tauri/src/commands/makeup.rs — 케이스2는 처리행(보강 포함) 보존 정책이라 보강 로직 변경 불필요

## 핵심 설계 (확정 — sprint16.md PI-20~27)
- **케이스1** = 출결 행 이동: `event_date` UPDATE + `note` 기록. 동월 한정 / present만 / 도착일 OFF·공휴일·충돌 차단(PI-25). UI는 present 셀 클릭→달력 다이얼로그(PI-26/27)
- **케이스2** = 시계열 + 미처리만 재생성: `set_schedule(effective_from=D)` + `apply_schedule_change(D)`. 변경일 양방향(사전/사후, PI-24). present만 DELETE 후 날짜 인식 재생성, 결석/보강/메모 행 보존(PI-21). effective_to exclusive
- **generate_impl 날짜 인식**: 각 날짜 d에 `effective_from ≤ d AND (effective_to IS NULL OR d < effective_to)` 매칭 (근본 개선)

## 완료 기준 (이번 세션)
- [ ] V306 마이그레이션 작성 + `sqlx migrate run` 적용 + self-check
- [ ] `move_attendance` IPC + 단위 테스트 (정상/OFF차단/월경계/충돌/present외 거부)
- [ ] `generate_impl` 날짜 인식 리팩토링 + 기존 테스트 회귀 통과
- [ ] `apply_schedule_change` IPC + 단위 테스트 (사후 소급/사전/결석·보강 보존/변경일 이전 불변/하한 검증)
- [ ] audit variant 2종 추가
- [ ] lib.rs 커맨드 등록 + IPC 래퍼/타입
- [ ] 프론트 UI (출결표 이동 다이얼로그 + ScheduleEditor 변경일) — 세션 여유에 따라 분리 가능
- [ ] Self-verify: cargo test / clippy --all-targets / lint / tsc 통과

## 진행 현황 (Session #1)
- ✅ **백엔드 완료·커밋(a8edc6a)**: V306(note) + move_attendance + apply_schedule_change + generate 날짜 인식 리팩토링. 단위 테스트 9건, 전체 383 passed, clippy --all-targets clean.
- ✅ **프론트 타입+래퍼 커밋(ee33e2d)**: AttendanceCell.note, MoveAttendanceResult/ScheduleChangeResult, moveAttendance/applyScheduleChange 래퍼. tsc/lint 통과.
- ⬜ **UI 통합 잔여**:
  1. AttendanceGrid present 셀 우클릭 → [수업일 이동 / 보강 등록] 선택 (기존 `onClassDayMakeupRegister` 단일 동작을 메뉴로 확장, PI-26 확정)
  2. 신규 `MoveAttendanceDialog` — 도착일 월 달력(ThreeMonthCalendar/CalendarCell 패턴 재활용), OFF일/공휴일/충돌/타월 비활성, moveAttendance 호출
  3. `students/schedule-editor.tsx` — 변경일 날짜 선택 + 재생성 확인 다이얼로그(setSchedule + applyScheduleChange 연계, 보존 N건 안내)
  4. 부모 `app/attendance/page.tsx` 콜백 연결

## 발견된 이슈
(없음 — 발견 시 기록)
