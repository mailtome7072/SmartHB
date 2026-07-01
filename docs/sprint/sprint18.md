# Sprint Plan sprint18

## 기간
2026-07-01 ~ 2026-07-14 (2주)

## 목표
v1.0.0 실사용 중 수집된 사용자 피드백 10건(이슈 1~10)을 반영하여 수업관리 캘린더 UX를 전면 개선하고, 일정코드 변경 시 출결 자동 동기화 백엔드 로직을 구현하며, 교습일정 인쇄 기능을 신규 추가한다. Sprint 17 회고 액션 아이템(A107~A111)을 T0에서 선행 처리한다.

## ROADMAP 연계 기능
- Post-v1.0 유지보수: 사용자 피드백 10건 반영
- Sprint 17 회고 액션 아이템 5건 (A107~A111)
- 수업관리 캘린더 뷰 재정비 (주/월 보기 개선)
- 일정코드-출결 자동 동기화 (백엔드 핵심 로직)
- 교습일정 인쇄 기능 신규

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint17-retrospective.md`

| 액션 ID | 항목 | 우선순위 | 반영 방법 |
|---------|------|----------|-----------|
| A107 | STALE_THRESHOLD_SECONDS 86400 상향 또는 config.json 기반 전환 | High | T0에서 처리 — 86400(24h) 상향 |
| A108 | rollback 파일명 고유성 보장 (loop 인덱스 추가) | High | T0에서 처리 — `_{idx}` 접미사 추가 |
| A109 | auto_restore_with_retry 단위 테스트 | Medium | T0에서 처리 |
| A110 | cleanup_stale_tmp_backups spawn_blocking 래핑 | Medium | T0에서 처리 |
| A111 | WAL 체크포인트 실패 시 pool.close() 보장 | Medium | T0에서 처리 |
| A101 | cipher 스모크 테스트 (.exe 설치) | High | 배포 후 수동 검증으로 이연 유지 (Sprint 18 범위 외) |
| A102~A106 | Sprint 16 이월 항목 | Low~Medium | Post-MVP backlog 유지 |

## 리스크 레지스터 반영

| 리스크 ID | 설명 | 반영 |
|-----------|------|------|
| R120 | stale lock 취약점 (5분 임계값) | T0 A107에서 해소 |
| R121 | rollback 파일명 충돌 | T0 A108에서 해소 |
| R122 | WAL 체크포인트 pool.close() 누락 | T0 A111에서 해소 |
| R123 | cleanup_stale_tmp_backups spawn_blocking 누락 | T0 A110에서 해소 |

## 작업 목록

### T0: Sprint 17 회고 액션 아이템 해소 (백엔드) — 2h

Sprint 17 코드 리뷰에서 발견된 High/Medium 5건을 선행 처리한다.

- ⬜ A107: `lock.rs` STALE_THRESHOLD_SECONDS 300 -> 86400 (24h) 상향
- ⬜ A108: `integrity.rs` `generate_rollback_filename`에 loop 인덱스 파라미터 추가 (`rollback_YYYYMMDD_HHMMSS_{idx}.db`)
- ⬜ A109: `integrity.rs` `auto_restore_with_retry` 단위 테스트 (3회 retry 성공/실패 시나리오)
- ⬜ A110: `startup.rs` `cleanup_stale_tmp_backups` 호출을 `tokio::task::spawn_blocking`으로 래핑
- ⬜ A111: `setup.rs` WAL 체크포인트 실패 시 early return 전 `pool.close()` 호출 추가

**파일**: `src-tauri/src/commands/lock.rs`, `integrity.rs`, `startup.rs`, `setup.rs`

### T1: 이슈 6 — 보강데이 is_duplicate_blocked=0 (V308) — 0.5h ✅ 완료

V308 마이그레이션: `schedule_codes` 테이블의 보강데이(makeup_day) 코드에 `is_duplicate_blocked=0` 설정. 동일 날짜에 보강데이와 다른 일정 코드 중복 배치 허용.

**파일**: `src-tauri/migrations/308__makeup_day_allow_duplicate.sql`

### T2: 이슈 6 — 공휴일 is_duplicate_blocked=0 (V309) — 0.5h ✅ 완료

V309 마이그레이션: 공휴일(holiday) 코드에 `is_duplicate_blocked=0` 설정. 공휴일과 다른 일정 코드(공휴수업 등) 중복 배치 허용.

**파일**: `src-tauri/migrations/309__holiday_allow_duplicate.sql`

### T3: 이슈 8 — '결제선생' 카드사 선택 가능 (프론트엔드) — 1h · skill: frontend-design

현재 `is_card_type=0`인 결제수단(결제선생 등) 선택 시 카드사 드롭다운이 비활성화된다. 카드사 선택을 optional로 허용하도록 변경한다.

- ⬜ 수납 관련 컴포넌트에서 카드사 드롭다운 활성화 조건 수정: `is_card_type=1` 필수 -> 항상 활성화(선택 사항)
- ⬜ DB 변경 없음 (`payments.card_company_id`는 이미 nullable)
- ⬜ 백엔드 검증 수정 불필요 (`is_card_type=0`이면 `card_company_id` optional 허용 기존 로직 유지)

**파일**: `src/app/fees/page.tsx` 또는 수납 관련 컴포넌트

### T4: 이슈 1 — 수업관리 기본 뷰 '주'로 변경 (프론트엔드) — 0.5h

수업관리 캘린더 진입 시 기본 뷰를 월(dayGridMonth)에서 주(timeGridWeek)로 변경한다.

- ⬜ `useState('dayGridMonth')` -> `useState('timeGridWeek')` 변경

**파일**: `src/components/schedules/ClassCalendar.tsx:113`

### T5: 이슈 9 — 달력 요일 순서 '일월화수목금토'로 변경 (프론트엔드) — 1h

FullCalendar와 공지문 이미지 달력의 요일 시작을 월요일(1)에서 일요일(0)로 변경한다.

- ⬜ FullCalendar `firstDay: 1` -> `firstDay: 0` 변경
- ⬜ `src/lib/calendar-image.ts` 요일 배열 순서 동기화 (일월화수목금토)
- ⬜ 두 달력의 요일 순서 일치 확인

**파일**: `src/components/schedules/ClassCalendar.tsx`, `src/lib/calendar-image.ts`

### T6: 이슈 2+3+4 — 주 보기 색상/레이아웃/다중슬롯 재정비 (프론트엔드) — 3h · skill: frontend-design

주 보기(timeGridWeek) 캘린더의 색상 체계, 레이아웃, 다중 시간 슬롯 칩 생성을 재정비한다.

- ⬜ **이슈 2 — 색상 체계**: STUDENT_PALETTE(원생별 색상) -> 수업 시간 기준 4색 체계로 전환
  - 1h(60분): 색상 A
  - 2h(120분): 색상 B
  - 3h(180분): 색상 C
  - 4h(240분): 색상 D
- ⬜ **이슈 3 — 2열 너비 균등**: 동일 시간대 2명 이상일 때 열 너비가 균등하게 배분되도록 수정
- ⬜ **이슈 4 — 다중 슬롯 칩 생성**: 2h 이상 수업은 각 시간 슬롯마다 별도 칩 생성
  - 예: 3시 시작 2h 수업 -> 3시 슬롯 칩 + 4시 슬롯 칩 (동일 색상으로 시각적 연결)
  - FullCalendar 이벤트 데이터 변환 로직 수정 필요

**파일**: `src/components/schedules/ClassCalendar.tsx`

### T7: 이슈 5 — 월 보기 셀 원생 이름 직접 표기 (프론트엔드) — 2h · skill: frontend-design

월 보기(dayGridMonth)에서 현재 `{N}명` 텍스트 + hover 툴팁 방식을 원생 이름 직접 표기로 변경한다.

- ⬜ 셀 내부에 원생 이름을 Nx2 그리드로 직접 표기
- ⬜ 이름 hover 시 상세 정보 툴팁 표시 (이름 + 수업 시간)
- ⬜ 셀 너비 좁을 때 이름 truncate + overflow 처리
- ⬜ 이름이 많을 때 셀 높이 조절 또는 스크롤 처리

**파일**: `src/components/schedules/ClassCalendar.tsx`

### T8: 이슈 7 — 일정코드 변경 시 출결 자동 동기화 (백엔드 핵심) — 3h

학사 일정 코드의 `allows_regular_class` 속성이 변경될 때 해당 날짜의 출결 데이터를 자동으로 동기화한다.

- ⬜ `attendance.rs`에 `sync_attendance_on_schedule_change()` 신규 함수 추가
  - **OFF->ON 전환** (예: 공휴일에 공휴수업 추가): 해당 요일 정규 스케줄 있는 원생에게 출결 INSERT OR IGNORE
  - **ON->OFF 전환** (예: 공휴수업 삭제, 휴원일 추가): 해당 날짜 정규 출결 전체 DELETE
  - **상태 변화 없음**: 무시 (no-op)
- ⬜ `academic.rs` 3개 IPC 함수 수정:
  - `create_schedule_event`: 생성 후 동기화 호출
  - `update_schedule_event`: period_end_date 조회 추가 + 동기화 호출
  - `delete_schedule_event`: period_end_date 조회 추가 + 동기화 호출
- ⬜ 단위 테스트 최소 3건: OFF->ON, ON->OFF, 변화없음
- ⬜ fail-soft 정책: 동기화 실패 시 `eprintln!` 로그만 남기고 IPC 흐름 차단하지 않음

**파일**: `src-tauri/src/commands/attendance.rs`, `src-tauri/src/commands/academic.rs`

### T9: 이슈 10 — 교습일정 인쇄 기능 신규 추가 (프론트엔드) — 3h · skill: frontend-design

확정된 교습기간 선택 후 교습일정을 A4 세로 레이아웃으로 인쇄하는 기능을 추가한다.

- ⬜ 일정관리 화면에 "교습일정 인쇄" 버튼 추가 (확정 교습기간 선택 시 활성화)
- ⬜ 인쇄 전용 컴포넌트 신규 작성
  - 제목: "mm월 교습일정 (mm.dd~mm.dd)"
  - 본문: 월간 달력 (해당 교습기간 날짜 표시, 일정코드 색상 구분)
  - 요일 순서: T5에서 변경되는 '일월화수목금토' 기준
- ⬜ `window.print()` 기반 브라우저 인쇄 다이얼로그 호출
- ⬜ CSS `@media print` 스타일: A4 세로 레이아웃, 화면 전용 요소 숨김
- ⬜ 참고: `src/app/notices/page.tsx`의 `renderCalendarImageDataUrl` 달력 구조 재활용 검토

**파일**: `src/app/academic/` 하위 (일정관리 화면), 신규 인쇄 컴포넌트

### T10: 통합 검증 — 1h

- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전수 통과
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ⬜ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ⬜ `pnpm lint` clean
- ⬜ `pnpm tsc --noEmit` clean
- ⬜ `pnpm build` (static export) 성공

## 완료 기준 (Definition of Done)

**필수**
- ⬜ T0 회고 액션 아이템 5건(A107~A111) 전수 해소
- ⬜ 수업관리 기본 뷰가 주(timeGridWeek)로 진입
- ⬜ 달력 요일 순서가 일월화수목금토 (FullCalendar + 공지문 달력 양쪽)
- ⬜ 주 보기 색상이 수업 시간 기준 4색 체계로 동작
- ⬜ 2h 이상 수업이 각 시간 슬롯마다 칩 생성
- ⬜ 월 보기 셀에 원생 이름 직접 표기
- ⬜ 결제선생 결제수단에서 카드사 optional 선택 가능
- ⬜ 일정코드 변경 시 출결 자동 동기화 (OFF->ON, ON->OFF 모두 동작)
- ⬜ 교습일정 인쇄 A4 레이아웃 정상 출력
- ⬜ cargo test 전수 통과 (411+ expected)
- ⬜ cargo clippy --all-targets -- -D warnings clean
- ⬜ cargo check --features cipher 통과
- ⬜ pnpm lint + tsc --noEmit + build 전수 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

## 기술 제약 및 참고 사항

- **단일 개발자 정책**: `gh pr create` 금지, 직접 머지
- **worktree 사용 금지**: `git checkout -b` 방식만 허용
- **`--no-verify` 금지**: pre-commit hook 필수 통과
- **Tauri IPC 패턴**: 컴포넌트에서 `invoke()` 직접 호출 금지, `src/lib/tauri/index.ts` 래퍼 경유
- **DB 마이그레이션**: V308, V309 이미 완료. T8은 스키마 변경 없음 (기존 테이블 활용)
- **T5 -> T6/T7/T9 의존성**: 요일 순서 변경(T5)이 T6 주 보기, T7 월 보기, T9 인쇄 달력에 영향. T5를 먼저 처리할 것
- **T6 FullCalendar 이벤트 모델**: 다중 슬롯 칩(이슈 4)은 FullCalendar의 단일 이벤트를 복수 이벤트로 분할하는 데이터 변환 필요. `eventContent` 또는 이벤트 생성 로직 수정

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 총 Task 수 | 11 (T0~T10) |
| 완료 Task | 2 (T1, T2) |
| 남은 Task | 9 (T0, T3~T10) |
| 예상 총 소요 | 17h (T0: 2h + T3: 1h + T4: 0.5h + T5: 1h + T6: 3h + T7: 2h + T8: 3h + T9: 3h + T10: 1h + 완료 T1+T2: 1h) |
| 참고 Velocity | Sprint 17: 계획 16h / 당일 완료 |
| 판정 | 17h -- 1일 내 완료 가능 (Velocity 기준 충분) |
