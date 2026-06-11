---
Sprint: 16  |  Date: 2026-06-09  |  Session: #4 (공지문 이미지 요소 보강)
---

## 추가 보강 (Session #4 연장, 2026-06-09) — ✅ 시각검증 완료
시각검증 중 사용자 추가 요청 — 모두 구현·검증·시각검증 완료:
- 이미지 체크박스 위치를 청구액 아래로 이동
- 텍스트박스 **배경색** 지정(글자/배경 팔레트, '없음'=투명) — TextboxConfig.background_color
- 노랑 프리셋 → 밝은 노랑(#FFEC99) 통일
- 배경서식 글씨 깨짐: 생성 PNG를 **배경 원본 해상도**로 렌더(기존 bgDims 표시추정치→naturalWidth), 미리보기 image-rendering
- **임의 이미지 추가('이미지 추가' 버튼)**: NoticeLayout.customImages[] 신설(파일 업로드→assets 저장→배치). 로고/바코드(images)와 별개. z-order = 배경 위·다른 컨트롤 아래. 사이드 패널 custom 텍스트박스 아래 버튼+목록.

## 이번 세션 목표 (Session #4) — 공지문 캔버스 이미지 요소 (T3 진입 전 보강)
공지문 생성 화면에 교습소 로고·2D바코드를 캔버스 요소로 추가. 체크박스로 on/off, 드래그 이동 + **비율 유지 리사이즈**(react-rnd `lockAspectRatio`). 이미지 미등록 시 안내 팝업. 레이아웃에 저장(사용자 결정 2026-06-09).

### 발견(재사용)
- 에디터 = **react-rnd** 기반 텍스트박스(ratio 좌표, scale 미리보기). `lockAspectRatio` 내장 → 비율 유지 자동.
- 교습소 이미지 = `getAcademyInfo` + `readNoticeAsset`(settings/info 패턴). NoticeLayout = `app_settings.notice_layout` **JSON** → 필드 추가해도 **마이그레이션 불요**, `#[serde(default)]` 구버전 호환.

### 수정/생성 파일
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/notice.rs | [0회] | `NoticeImageConfig{kind,enabled,x/y/w/h_ratio}` + `NoticeLayout.images`(serde default) + default_images |
| src/types/notice.ts | [0회] | `NoticeImageConfig`/`NoticeImageKind` + `NoticeLayout.images` |
| src/app/notices/page.tsx | [0회] | 로고/2D바코드 체크박스 + 교습소 이미지 로드 + 이미지 Rnd(lockAspectRatio) 렌더 + 미등록 안내 팝업 + 생성/미리보기 포함 |

### 설계 결정
- 레이아웃 **저장 O** — NoticeLayout.images. 템플릿 저장/로드 시 이미지 배치 복원.
- 체크 시 이미지 dataUrl 없으면 안내 팝업("설정 > 교습소 정보 등록 필요") + 체크 취소.
- 초기 크기는 img naturalWidth/Height 비율로 h_ratio 보정. enabled만 토글(위치 유지).
- logo/barcode는 모든 원생 공통 — 생성 시 동일 배치.

### 완료 기준
- ✅ 백엔드 NoticeImageConfig + images (cargo test notice 5건 / clippy --all-targets clean)
- ✅ 체크박스 2종(교습소로고/2D바코드) + 미등록 안내 팝업
- ✅ 이미지 Rnd 드래그+비율유지(lockAspectRatio) 리사이즈, onLoad 비율 보정
- ✅ 미리보기/생성 포함 — notice-generator는 별도 canvas 렌더라 imageUrls 전달 + drawImage 추가
- ✅ 레이아웃 저장/로드(makeDefaultLayout·normalizeLayout images 보강, 구버전 serde default)
- ✅ Self-verify: cargo test / clippy / tsc / lint 통과
- ✅ 실 앱 시각검증 (사용자) 완료 (2026-06-09) — 이미지요소·비율버그·텍스트순서·체크박스위치·배경색·밝은노랑·배경해상도·이미지추가 전부

---

## (Session #3 기록 — 보존·완료) T2
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

---

## 추가 보강 (Session #5, 2026-06-10) — 공지문 교습일정 달력 이미지
공지문 캔버스에 **청구년월 교습일정을 달력 형식 이미지로 합성**. 2D바코드 체크박스 아래 "교습일정" 체크박스 신설. (사용자 요청 2026-06-10, 가능여부 확인 후 착수)

### 설계 결정 (사용자 확정 2026-06-10)
- 배치: **이동·크기조절 가능 요소** — `NoticeImageKind`에 `'calendar'` 추가 (logo/barcode와 동일 메커니즘, 비율 좌표·체크박스 on/off).
- 표시 항목: 교습기간 **빨간 외곽선**(교습기간 ∩ 수업일 영역) + 특이일 라벨(공휴일·보강데이) + 기간 하이라이트(단원평가 주간 등) + 전월/익월 회색.
- 단원평가: 점수 기능 취소됐으나 **학사일정 라벨은 그대로 렌더**(데이터 그대로).
- 달력 그리드: **일요일 시작**(공지문=대중 공개용, 예시 이미지 기준). 앱 학사캘린더는 월요일 시작이나 공지문은 별도.

### 발견(재사용)
- 데이터: `getStudyPeriod(yearMonth)`(빨간선 범위) + `listScheduleEvents(from,to)`(라벨/하이라이트) + `getOperatingHours()`(수업일 판정). **신규 IPC/마이그레이션 0**.
- 빨간 외곽선 = `inStudyPeriod && hasClassOnDate` 셀 영역의 경계 — `ThreeMonthCalendar.hasClassOnDate` 로직 포팅(휴원/공휴일 제외, 공휴수업일 포함).
- 캔버스 합성: `notice-generator.ts`의 이미지 요소 드로잉 재활용. 달력 PNG dataURL → `imageUrls['calendar']`.
- z-order: 배경 → customImages → **layout.images 배열순서(logo/barcode/calendar)** → 텍스트. 미리보기 DOM 순서와 1:1(WYSIWYG) — 별도 정렬 안 함(요소 비중첩 전제).
- 그리드 날짜계산: `buildMonthGrid`(라이브러리 무의존) 패턴 — 단 일요일 시작으로 변형.

### 수정/생성 파일
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/lib/calendar-image.ts | [신규] | 한 달 데이터 → 캔버스 PNG dataURL 렌더러(그리드·빨간외곽선·라벨·하이라이트·요일색) |
| src/types/notice.ts | [0회] | `NoticeImageKind`에 `'calendar'` 추가 |
| src/lib/notice-generator.ts | [0회] | 달력 이미지 합성 포함(z-order: 배경→추가이미지→달력→로고/바코드→텍스트) |
| src/app/notices/page.tsx | [0회] | "교습일정" 체크박스 + 청구년월 학사데이터 로드 → 달력 dataURL 생성 → 이미지 요소 렌더 |

### 완료 기준 (Session #5)
- ✅ 달력 이미지 렌더러 — 외곽선·라벨·하이라이트·회색 + 검정 셀선
- ✅ "교습일정" 체크박스 on/off + 드래그·리사이즈
- ✅ 생성·미리보기에 달력 합성 반영
- ✅ Self-verify: lint / tsc 통과 (프론트 전용 — Rust 변경 없음) + 실 앱 시각검증(사용자 검수 완료)

### 시각검증 반영(사용자 요청)
- 빨간 외곽선 규칙 확정: **첫 평일 수업일 ~ 마지막 평일 수업일** 구간을 감싸되, 경계 비수업일(공휴일 등)은 트림·사이의 평일 공휴일은 포함·**토·일요일은 항상 제외**.
- 셀 기본 테두리 검정(`GRID_LINE=#000000`).
- 보강데이 라벨: 볼드 + 150%(30px) + top 위치를 단원평가 주간 밴드 라벨과 동일(상수 공유).

---

## T3: DB 폴더 변경 + salt.bin 이전 (Session #6, 2026-06-10) — MUST · PI-16
클라우드 동기화 경로 재지정 + `smarthb/` 전체(DB·salt·assets·output·backup) 동반 이전. **copy-then-switch + 재시작**. 설계: `docs/arch/adr-009-db-folder-change.md`.

### 사용자 확정 (2026-06-10)
- 원본(구) 폴더: **유지 + 마커 파일**(MOVED_TO.txt). 자동 삭제 안 함.
- 대상에 기존 `smarthb/app.db` 존재 시: **차단 + 안내**(덮어쓰기 방지).
- 재시작: **tauri-plugin-process 추가 + 프론트 relaunch()**.

### 절차 (ADR-009)
대상검증(기존 DB 차단·동일/포함 차단·쓰기권한) → WAL checkpoint(TRUNCATE) → 재귀복사(app.lock·-wal·-shm 제외, fsync) → 검증(cipher PRAGMA key + integrity_check) → 마커파일(원본) → config.json 갱신(마지막) → 원본 락 해제 → 프론트 relaunch.
실패 시: config 미변경(앱 기존폴더 유지) + 부분복사 best-effort 제거. 원본 불삭제.

### 수정/생성 파일
| 파일 | 비고 |
|------|------|
| src-tauri/src/commands/setup.rs | `change_data_folder` IPC + impl + 복사/검증/마커 헬퍼 + 단위테스트 (config 헬퍼 재사용) |
| src-tauri/src/commands/paths.rs | `data_root_for(cloud)` 헬퍼 노출 |
| src-tauri/Cargo.toml + lib.rs + capabilities/default.json | tauri-plugin-process 추가·등록·권한 + change_data_folder 등록 |
| package.json | @tauri-apps/plugin-process |
| src/lib/tauri/index.ts + src/types | `changeDataFolder` 래퍼 + 타입 |
| src/app/settings/page.tsx | 'DB 폴더 변경' 카드 활성화(disabledHint 제거) |
| src/app/settings/db-folder/page.tsx (신규) | 폴더 선택(Dialog) + 안내/경고 + 실행 + 완료 후 relaunch |

### 완료 기준 (Session #6)
- ✅ `change_data_folder` IPC + 단위테스트 8건(empty/same/overlap/기존DB차단/fresh/copy+skip/marker)
- ✅ WAL checkpoint(TRUNCATE) + 재귀복사(app.lock·-wal·-shm 제외, fsync) + cipher 검증(sqlx integrity_check) + 마커 + config 갱신(마지막)
- ✅ tauri-plugin-process 추가·등록 + capabilities `process:allow-restart` + 프론트 `relaunchApp`
- ✅ /settings/db-folder UI + 카드 활성화 + 양PC 재지정 경고 안내 + 확인/완료 모달
- ✅ Self-verify: cargo test(403) / clippy --all-targets / cargo check --features cipher / lint / tsc 통과
- ✅ 실앱 시각검증 (사용자) — 실제 데이터로 이전 성공: 새 폴더에 app.db/salt/assets/backup/output 전부 복사 + WAL checkpoint + 검증 + MOVED_TO 마커 + config 갱신 + relaunch 전 과정 정상. 원복 후 원본 폴더 정상 기동·데이터 정상.

### 구현 메모
- `change_data_folder`(setup.rs): copy-then-switch. 실패 시 config 미변경 → 기존폴더 유지, 부분복사 best-effort 제거. 원본 불삭제(마커만).
- 검증은 sqlx 기반 `PRAGMA integrity_check`(feature 무관, cipher 시 PRAGMA key 적용).
- 원본 락 해제는 생략 — heartbeat가 재생성하고 재시작 후 원본 폴더는 abandoned. (ADR 명시한 best-effort 해제는 무의미하여 미구현)
- `paths::data_root_for(cloud)` 헬퍼 추가.
- **dev relaunch 가드**(시각검증 발견): dev 빌드는 화면을 localhost dev서버에서 로드 → `relaunch()` 시 dev서버 동반 종료로 "localhost 연결 거부". 완료 모달에서 `NODE_ENV!=='production'`이면 자동 재시작 대신 **수동 재시작 안내**. 프로덕션은 자동 relaunch 정상. (프로덕션 버그 아님 — dev 한정 한계)

---

## daily/weekly 백업 스케줄러 연결 (Session #8, 2026-06-11) — catch-up 방식
backlog 해소: daily(30)/weekly(4) 계층은 `backup.rs`에 정의·rotation만 있고 **생성 트리거가 없던** 공백. 실사용 직전 데이터 안전 보강 최우선 (사용자 확정 순서 1번).

### 설계 (메모리 권장안 채택)
- 순수 interval 타이머는 앱이 계속 떠 있어야만 fire → 간헐적 사용 패턴에 부적합.
- **catch-up 방식**: 앱 시작 시 1회 + hourly tick마다 `scan_layer(Daily/Weekly)` 최신 `created_at` 확인 → **24h/7d 경과 또는 백업 0건**이면 `try_create_backup(layer)` 호출.
- "is due" 판정은 순수 함수로 분리 — **feature 무관 단위테스트**. 실제 create_backup은 cipher 빌드만 동작(off는 기존 stub 안내, dev 무해).

### 수정 파일
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/backup.rs | [0회] | `BackupLayer::catchup_interval()`(Daily=24h/Weekly=7d/나머지 None) + `is_due()` 순수 함수 + `run_catchup_backups()` + 단위테스트 |
| src-tauri/src/startup.rs | [0회] | spawn_background_tasks: 백업 task 시작 직후 catch-up 1회 + hourly tick마다 catch-up. 모듈 doc 갱신 |

### 범위 외 (수정 안 함)
- 프론트엔드 — 변경 없음 (백업 목록 UI는 기존 list_backups로 daily/weekly 자동 표시)
- 마이그레이션·신규 의존성 — 없음

### 완료 기준
- ✅ catchup_interval/is_due 순수 함수 + 단위테스트 5건 (0건→due, 미경과→false, 경계 포함 경과→true, daily/weekly interval, cipher off fail-soft)
- ✅ run_catchup_backups: scan 실패 fail-soft(스킵), due 시 try_create_backup 위임
- ✅ startup 연결: 백업 task 시작 직후 1회 + hourly tick마다, OnceLock 중복 spawn 방지 유지
- ✅ Self-verify: cargo test(408) / clippy --all-targets clean / cargo check --features cipher 통과 (프론트 무변경 → lint/tsc 생략)

### 추가 결정 — 보관 개수 축소 (사용자 확정 2026-06-11, PRD v1.5.2)
1인 사용 시스템 + 백업 위치가 클라우드 동기화 폴더(업로드 트래픽·용량 점유)인 점을 들어 사용자가 축소 결정. 복구 시나리오(당일 실수=hourly, 손상=exit, 과거 시점=daily/weekly)는 유지.
- **exit 10→5 / hourly 24→12 / daily 30→14 / weekly 4 유지** — 합계 68→35.
- 갱신: `backup.rs::max_keep`+테스트+모듈doc / `src/types/index.ts` 주석 / **PRD §5.4** 표+Change Log v1.5.2(트리거도 catch-up 방식으로 명확화) / **ADR-003** 개정 노트(본문 보존) / `.claude/rules/backend.md` / `ARCHITECTURE.md`.
- ROADMAP·CHANGELOG·sprint1.md·phase1 리뷰 등 과거 기록 문서는 당시 수치 그대로 보존(미수정).
- 기존 백업이 새 상한 초과 시 다음 rotation에서 오래된 것부터 자동 정리(코드 변경 불요).

---

## 백업 복원 연결 (Session #7, 2026-06-10) — 자동 복원 + 수동 복원 UI
백업/복원 로직(ADR-003)은 있으나 사용자 흐름에 미연결이던 공백 해소. 자동(부팅 차단형 손상 대응) 우선 + 수동(정상 운영 중 롤백) 보조. 사용자 결정: **둘 다**.

### 배경(발견)
- `auto_restore`/`restore_backup` IPC는 구현됐으나 프론트에서 **호출하는 곳이 없었음**. 시작 시 무결성 손상 감지해도 감사 로그만 기록. 백업관리 화면은 목록+리허설만(복원 버튼 없음).
- daily/weekly 계층은 스케줄러 미연결(이번 범위 외, 별도 backlog).

### 자동 복원 (1순위)
- `startup.rs::run_startup`: 인증(키 캐시 충전) 후 → DB 초기화 **전에**, quick_check 가 `Failed`면 `integrity::auto_restore_sync()` 실행(최신 정상 exit 백업으로 교체, 현재 손상본 rollback 보존) → `StartupResult.auto_restored: Option<RestoreResult>` 반환 + 감사 로그. **cipher off 개발 빌드는 quick_check stub Ok → 미진입(정상/dev 무영향)**. 복원 실패는 fail-soft.
- 프론트: `StartupResult.auto_restored` 있으면 루트 페이지에 **"최근 정상 백업으로 자동 복원됨 + 이후 입력 누락 가능 + 손상본 보존" 고지 배너**(session-store `restoreNoticeDismissed`).

### 수동 복원 (2순위)
- `/settings/backup`: 선택 백업에 **"이 백업으로 복원" 버튼**(danger) + 확인 모달(시점·데이터 손실·rollback 보존 안내) → `restoreBackup(path)` → 완료 모달 → 재시작(dev 가드 동일).

### 수정 파일
| 파일 | 비고 |
|------|------|
| src-tauri/src/commands/integrity.rs | `auto_restore_sync` pub(crate) 노출 |
| src-tauri/src/startup.rs | run_startup 손상 자동복원 단계 + `StartupResult.auto_restored` |
| src/types/index.ts | `StartupResult.auto_restored: RestoreResult\|null` |
| src/stores/session-store.ts | `restoreNoticeDismissed` + `dismissRestoreNotice` |
| src/app/page.tsx | 자동복원 고지 배너 |
| src/app/settings/backup/page.tsx | 수동 복원 버튼 + 확인/완료 모달 |
| src/lib/tauri/index.ts | dev fallback `auto_restored: null` |

### 완료 기준 (Session #7)
- ✅ 시작 손상 자동복원 연결(StartupResult.auto_restored) + 고지 배너
- ✅ 수동 복원 UI(버튼+확인+완료/재시작)
- ✅ Self-verify: cargo test(403) / clippy --all-targets / cargo check --features cipher / lint / tsc 통과
- ⬜ 실앱 시각검증 — ⚠️ **dev 한계**: 백업/복원/무결성은 cipher 빌드에서만 실동작(dev는 stub·백업 0건). dev에선 페이지 렌더·회귀만 확인 가능. 실동작 검증은 cipher 빌드 + 손상 시뮬레이션 필요.
