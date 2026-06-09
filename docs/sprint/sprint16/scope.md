---
Sprint: 16  |  Date: 2026-06-09  |  Session: #3 (T2)
---

## 이번 세션 목표 (Session #3)
**T2: CSV 가져오기 (6h, MUST)** — 원생 실데이터 이관. PRD §4.13.1.
백엔드(인코딩 디코딩 → CSV 파싱 → preview/import IPC → 테스트) → 프론트(`/settings/import` 미리보기·매핑·결과).

### T2 확정 범위 (단순·안전, 사용자 결정 2026-06-09)
- **대상 = `students` 한 테이블만**. `school_id`(학교 FK)·`student_schedules`(수업요일)는 CSV로 넣지 않음 → 앱에서 수동. school_id는 NULL로 INSERT.
- **컬럼**: 이름(필수) / 학년(필수, "초3"·"중2" 형식 → school_level+grade 파싱, grade 1~9 CHECK) / 입교일(필수, 비면 가져오기 실행일) / 성별·생년월일·모연락처·부연락처·학생연락처·일련번호(선택). gender 비면 'male' 기본 + 미리보기 경고(NOT NULL 제약, 사후 수정 가능). serial_no 비면 자동 채번(compute_next_serial 재사용).
- **중복 = skip만**(overwrite 제외): 일련번호 존재 OR (이름+모연락처) 존재 시 건너뜀.
- **안전장치**: ① 미리보기 우선(파싱·검증·중복판정만, INSERT 안 함) ② 가져오기 직전 백업 1회 ③ 인코딩 UTF-8/EUC-KR 자동(BOM 처리).
- **기존 로직 재사용**: `create_student`의 트랜잭션·채번·UNIQUE 매핑 패턴 따름(가능하면 행별 INSERT를 단일 트랜잭션으로).

### T2 수정/생성 파일
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/Cargo.toml | [0회] | 의존성 `csv = "1"`, `encoding_rs = "0.8"` 추가 (chardetng 제외 — UTF-8 실패 시 EUC-KR fallback 휴리스틱) |
| src-tauri/src/commands/import.rs | [0회] | **신규** — decode(BOM+UTF-8/EUC-KR) / parse_rows / `preview_students_csv(file_path)` / `import_students_csv(file_path)` + 단위 테스트 |
| src-tauri/src/commands/mod.rs | [0회] | `pub mod import;` 등록 |
| src-tauri/src/lib.rs | [0회] | invoke_handler에 preview/import 커맨드 등록 |
| src-tauri/src/commands/audit.rs | [0회] | `StudentsImported` audit variant(건수 details) 추가 가능 |
| src/types/import.ts | [0회] | **신규** — PreviewRow/PreviewResult/ImportResult 타입 |
| src/lib/tauri/index.ts | [0회] | `previewStudentsCsv`/`importStudentsCsv` 래퍼 + dev fallback |
| src/app/settings/import/page.tsx | [0회] | **신규** — 파일 선택(Dialog) → 미리보기 테이블(상태색) → 가져오기 → 결과 요약 |
| src/app/settings/page.tsx | [0회] | 'CSV 가져오기' 카드 추가 |

### T2 설계 결정
- **파일 전달**: 프론트는 Tauri Dialog로 **경로만** 얻고, 백엔드 IPC가 `std::fs::read(path)`로 읽음(notice_asset 패턴). bytes 직렬화 전송 회피, fs 권한은 Rust 직접이라 capability 무관.
- **preview/import 분리**: 둘 다 file_path 받아 동일 parse. preview는 DB 중복조회까지(드라이런), import는 백업+트랜잭션 INSERT.
- **인코딩**: UTF-8 BOM 제거 → from_utf8 시도 → 실패 시 encoding_rs EUC_KR 디코딩.

### T2 완료 기준
- ✅ csv/encoding_rs 의존성 추가 + 빌드
- ✅ decode/parse 단위 테스트 11건 (UTF-8, EUC-KR, BOM, 학년파싱 초/중·범위, 필수누락 거부, gender 기본, 행에러 격리, 입교일 기본, 중복판정)
- ✅ preview IPC + 중복판정(is_duplicate 순수함수 테스트)
- ✅ import IPC (백업 1회 → create_student 위임, 같은파일 중복 remember 누적)
- ✅ 프론트 /settings/import 미리보기·가져오기·결과 요약 + 설정 허브 카드
- ✅ Self-verify: cargo test(import 11) / clippy --all-targets clean / tsc / lint 통과
- ✅ 실 앱 시각검증 (사용자) "정상동작 확인, 검수완료"(2026-06-09). 개발모드(cipher off) 백업 stub 실패 메시지("디스크 여유 공간")는 정상 — import은 백업 실패해도 진행, 데이터 INSERT 확인.

---

## (Session #2 기록 — 보존·완료) T1 목표
## 이번 세션 목표 (Session #2)
**T1: 회고 액션 + 코드 리뷰 carry-over (3h, MUST)** — A99(Ctrl+N 입력필드 방어) / A100+R105(미저장 이탈 경고 공통 훅) / Ctrl+S 전역 저장.
> T0(수업일 변경 도메인)은 Session #1에서 완료·시각검증 통과(아래 기록 보존). develop 미머지 상태로 sprint16 브랜치 누적 진행.

### T1 수정/생성 파일
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/lib/use-unsaved-changes.ts | [2회] | **신규** — `useUnsavedChanges(dirty, onSave?)`: beforeunload + `app:save`(Ctrl+S) + 메뉴이동 가드(unsavedGuard 등록). 2회차: window.confirm 차단 → unsavedNavTarget 세팅 방식으로 교체 |
| src/stores/app-store.ts | [1회] | `unsavedNavTarget` + `setUnsavedNavTarget` 추가 (공용 이동 확인 다이얼로그 트리거) |
| src/components/layout/UnsavedNavDialog.tsx | [0회] | **신규** — 공용 미저장 이동 확인 모달. AppShell 1회 마운트. plain fixed 오버레이(window.confirm 대체) |
| src/components/layout/app-shell.tsx | [1회] | UnsavedNavDialog 마운트 |
| src/components/layout/GlobalShortcuts.tsx | [0회] | A99: Ctrl+N 시 e.target tagName INPUT/TEXTAREA/SELECT·contentEditable 억제. Ctrl+S: preventDefault + `window.dispatchEvent(CustomEvent('app:save'))` |
| src/app/settings/info/page.tsx | [0회] | dirty 상태 추가 + useUnsavedChanges(dirty, handleSave) 적용. updateField/handleUpload/handleRemoveImage 시 dirty=true, 저장 성공 시 dirty=false |
| src/components/students/student-form.tsx | [0회] | (중복 제거) 인라인 beforeunload → useUnsavedChanges 훅으로 교체. R105 공통화 취지 |

### T1 설계 결정
- **이탈 가드 범위 = beforeunload + 내부 메뉴 이동 가드** (사용자 결정 2026-06-09, 2차 추가). 계획의 `routeChangeStart`는 App Router 미지원 → 채택 안 함. 대신 **기존 `unsavedGuard`(app-store, Sprint 12 공지문 편집용) 메커니즘 재사용** — 사이드바/글로벌검색이 이동 직전 가드 호출, dirty면 `window.confirm`으로 차단. app-store·sidebar·global-search는 변경 불필요(슬롯 재사용).
- **Ctrl+S 메커니즘**: GlobalShortcuts(상시 마운트)가 preventDefault + `app:save` CustomEvent dispatch → 활성 폼이 useUnsavedChanges(onSave)로 구독. 전역 click/store 결합 없이 느슨한 결합.
- 결과: `useUnsavedChanges(dirty, onSave?)` 한 줄로 3종 가드(창닫기·Ctrl+S·메뉴이동) 일괄 획득.

### T1 완료 기준
- ✅ A99: 입력 필드 포커스 중 Ctrl+N 억제 (Ctrl+F는 검색 이동이므로 유지) — `isEditableTarget` 가드
- ✅ useUnsavedChanges 훅 신규 + student-form 중복 제거 + settings/info 적용
- ✅ Ctrl+S: settings/info에서 저장 동작(`app:save` 이벤트), WebView 기본 저장 다이얼로그 억제
- ✅ 메뉴 이동 가드: dirty 시 사이드바/검색 이동 직전 확인 다이얼로그(`unsavedGuard` 재사용)
- ✅ Self-verify: tsc / lint 통과 (Rust 무변경 → cargo 생략)
- ✅ 실 앱 시각검증 (사용자) "이상 없음"(2026-06-09) — window.confirm 차단 발견 → 커스텀 UnsavedNavDialog로 교체 후 재검증 통과

---

## (Session #1 기록 — 보존) T0 목표
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
- ✅ **UI 통합 완료·커밋(919b16e)**:
  1. AttendanceGrid present 셀 우클릭 → `onPresentCellAction` (makeup_done/expired는 기존 보강등록 유지)
  2. `MoveAttendanceDialog` 신규 — 단일 월 grid-cols-7 달력, 출발일/기존출결일/휴일(isBlock) 비활성, moveAttendance 호출
  3. `schedule-editor.tsx` — 적용 시작일 date 입력(과거/미래) + setSchedule→applyScheduleChange 연계 + 결과 안내 배너(재생성/보존/청구)
  4. `attendance/page.tsx` — present 우클릭 [수업일 이동/보강 등록] 액션 모달 + MoveAttendanceDialog 연결

## T0 완료 — Self-verify 전수 통과 + 시각 검증 완료(2026-06-08)
- cargo test 384 passed / clippy --all-targets clean / tsc / lint / build 통과
- 핵심 커밋: a8edc6a(백엔드) → ee33e2d(타입·래퍼) → 919b16e(UI) + 시각검증 수정 다수(아래)
- ✅ **실 앱 시각 검증 완료** — `pnpm tauri:dev`로 케이스1 이동/시간입력, 케이스2 스케줄 변경, 캘린더 표시 전수 확인, "이상 없음"(사용자).

## 발견된 이슈 (시각 검증 Session #1)
- **I1 (해결)**: 출결표 출석 셀 hover 툴팁 '날짜 ○' → '날짜 (출석)' 표기 변경 요청. `cellTooltip` present 분기 + note(이동 메모) 툴팁 누락 보완. (커밋 예정)
- **I2 (크래시·즉시 방어 완료)**: 수업일 이동(케이스1) 후 수업 캘린더 **주간 보기 클릭 시 `Cannot read properties of undefined (reading 'padStart')` 크래시**.
  - 원인: `calendar.rs`가 정규 수업 시작시간을 **출결일자 요일의 현행 스케줄 JOIN**으로 가져옴 → 이동 출결(스케줄 없는 요일)은 `start_time` 부재 → `ClassCalendar.toIsoTime`이 빈/null 값에 `padStart` 호출.
  - 방어: `toIsoTime` null/빈값 가드, 주/일 뷰 `if (!startTime || !includes(':')) continue`, 월 뷰 `|| '시간미정'`. → 크래시 제거(이동 수업은 월 뷰 '시간미정', 주/일 뷰 시간슬롯 미표시).
  - **근본 해결(PI-28)**: 사용자 결정 — 1회성 이동 시 **수업 시작시간 입력**. V307 `regular_attendances.start_time` 추가, `move_attendance(start_time)` 저장, calendar `COALESCE(ra.start_time, ss.start_time)`, MoveAttendanceDialog 시작시간(시 단위) 입력. 단위 테스트(normalize_time, start_time 저장) 추가.

## 시각 검증 중 추가 수정·결정 (2026-06-08, PI-29~30 포함)
- **PI-29**: 케이스1 같은 날 추가 수업 → 빈 날 전용 이동 유지, 같은 날은 보강 등록. 시작시간 **시(時) 단위만** 선택.
- **PI-30**: 정규수업 불가일(주말/공휴일/보강데이) 이동 차단. `DaySchedule.regular_blocked`(allows_regular=0) 추가 — `allowsMakeup`(공휴수업일 등 정규 가능) 오판 교정.
- **출결표**: 출석 셀 hover 툴팁 '(출석)', 이동 메모 표기.
- **수업 캘린더(ClassCalendar)**: 원생별 색칩 + 수업시간 표기 / 월 헤더 요일 / 주 보기 2열 묶음·일 보기 개별 블록(실제 길이) / 칩 중앙정렬 / 칩 hover 시 수업 시간범위 테두리(월 보기 셀 hover 스타일) / 컨테이너 회색 테두리 제거 / 이전·다음 이모티콘+년월 18px / 토·일 컬럼 폭 절반 / 시간행 14:00 시작 / 주 행높이 5rem(6명 2×3 수용).
- **스케줄 편집(schedule-editor)**: 수정 시 요일 변경(원래 요일 종료) / 요일 선택 평일·미등록만 / 삭제 시 출결 정리(applyScheduleChange) / 추가·변경·삭제 확인 다이얼로그.
- 시각검증 수정 커밋: c5b79db ~ ee83b72 (약 20개). 자동검증(cargo test 384 / clippy / tsc / lint) 매 커밋 통과.
- 추가 마이그레이션: V306(note) + **V307(start_time)** — 둘 다 ALTER ADD COLUMN.
