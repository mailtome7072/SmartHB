# CHANGELOG

이 파일은 프로젝트의 버전별 변경 이력을 기록합니다.
형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.0.0/)를 기반으로 하며,
[Semantic Versioning](https://semver.org/lang/ko/)을 준수합니다.

---

## 작성 규칙

### 카테고리

| 카테고리 | 설명 |
|----------|------|
| `Added` | 새로운 기능 추가 |
| `Changed` | 기존 기능 변경 |
| `Deprecated` | 곧 제거될 기능 예고 (하위 호환성 안내) |
| `Removed` | 기능 제거 |
| `Fixed` | 버그 수정 |
| `Security` | 보안 취약점 수정 |

### Semantic Versioning 올림 기준

| 버전 | 트리거 |
|------|--------|
| `MAJOR` | 하위 호환 불가 변경 — API 브레이킹 체인지, DB 구조 대규모 변경 |
| `MINOR` | 하위 호환 신규 기능 추가 — 새 엔드포인트, 새 UI 기능 |
| `PATCH` | 버그 수정, 핫픽스, 문서 수정 |

### [Unreleased] 운영 방법

- **채우는 시점**: PR merge 시마다 해당 카테고리에 항목 추가
- **버전 전환 시점**: `deploy-prod` agent가 main 배포 시 `[Unreleased]` → `[x.y.z] - YYYY-MM-DD`로 전환
- **새 버전은 항상 최상단에 추가**

---

## [Unreleased]

### Added
- Sprint 22: **보강 분 단위 부분 차감 스키마** — `makeup_allocations` 배분 링크 테이블(V311) 신규. 한 보강이 여러 결석에 얼마(분)를 충당했는지 명시적으로 저장. FK(makeup_attendances/regular_attendances) + UNIQUE(makeup_id, absence_id) + CHECK(allocated_minutes > 0) + 인덱스 2개. ADR-011 Accepted
- Sprint 22: **유실 결석 자동 백필** — V312 멱등 마이그레이션. 기존 일 단위 매칭 데이터를 분 단위 배분으로 이전하고, 보강 시간이 결석 시간보다 적었던 경우(부분 보강) 잔여 결석을 `absent`로 복원. 앱 첫 실행 시 자동·무알림 적용
- Sprint 22: **마이그레이션 직전 사전 스냅샷 백업** — V311/V312 적용 전 DB를 `backup/pre-migration/` 에 자동 보관하는 안전장치 (ADR-011 R140)

### Changed
- Sprint 22: **보강 등록·취소 분 단위 부분 차감 전환** — `makeup.rs` 등록 로직이 `makeup_allocations`에 분 단위로 배분하고 취소 시 해당 배분 레코드만 삭제하여 정확한 부분 환원 지원
- Sprint 22: **보강 관련 조회/집계/소멸/진단 쿼리 잔여분 기반 전환** — `calendar.rs`, `attendance.rs`, `expiration.rs`, `diagnosis.rs` 등 8개 파일에서 잔여분(`class_minutes - SUM(allocated_minutes)`)을 기준으로 통일

### Fixed
- Sprint 22: **보강 부분 차감 시 잔여 결석 유실 버그 수정** — 2시간 결석에 1시간만 보강 시 잔여 1시간이 보강 대상 목록에서 사라지던 실사용 버그 해소. 잔여분이 있는 결석은 `absent` 상태로 목록에 재노출
- Sprint 22: **출결 그리드 상하 스크롤 시 고정 헤더 깨짐 수정** — `AttendanceGrid` sticky 헤더·셀의 z-index 층위 충돌 해소. 스크롤 시 날짜 헤더·원생 셀이 콘텐츠 위에 올바르게 고정

---

## [1.3.0] - 2026-07-20

### Added
- Sprint 20: **청구 삭제 기능** — 미수납(draft/confirmed) 청구만 삭제 가능, 수납완료 청구는 삭제 거부(ADR-010 B안). `delete_bill` IPC(payments CASCADE, BillDeleted 감사 로그), 삭제 버튼(미수납만 활성) + 확인 다이얼로그. `DeleteBillDialog` 컴포넌트 신규. 단위 테스트 4건
- Sprint 20: **실 DB 보정 절차 문서** — 교습기간 기준 전환 후 기존 오류 청구 식별·삭제 절차 (`docs/sprint/sprint20/data-correction-procedure.md`). 퇴교취소 후 월별 데이터 재생성 운영 워크플로우 안내(T5-b) 포함

### Changed
- Sprint 21: **수업일 이동 다이얼로그 출발일 달력월 기준으로 수정** — `MoveAttendanceDialog`에서 달력월 범위 계산(`lastDay`, `dateStr`)을 출발일의 달력월 기준으로 일관화. 백엔드 `move_attendance`가 달력월 동월 한정이므로 정합 보장
- Sprint 20: **청구 대상 선정 규칙 교습기간 기준 전환** — `generate_bills_impl` 기간을 달력월(`year_month_range`) 대신 `study_periods.start_date/end_date`로 교체. 교습기간 종료일 이후 입교 원생 자동 제외. `effective_from` 필터 추가(스케줄 이력 정합). 교습기간 미등록 월 청구 생성 차단 + 안내 메시지. `get_billing_summary_impl` 동기화 — 유령 "추가 생성" 버튼 방지. 회귀 테스트 6건
- Sprint 20: **교습일정 인쇄 3개월+ 걸침 레이아웃 수정** — 기존 멀티페이지 달력 분할 방식에서 **주 월(主月) 달력 한 장** 표기로 변경. 앞뒤 이웃 달의 교습일은 해당 셀에 강조 표시. 그리드 밖 대규모 기간만 멀티페이지 폴백

### Fixed
- Sprint 21: **출결 다월 교습기간 그리드 표시/태깅 불일치 수정 (R136)** — `AttendanceGrid` 컬럼을 달력월 1~말일(`daysOfMonth`) 대신 교습기간 실제 날짜 범위(`start_date`~`end_date`)로 재설계. 전체 ISO 날짜 매핑으로 다월 걸침 시 DD 충돌 해소. 월 경계 헤더 월 표기 추가. 단일월 교습기간 폴백 유지. `sync_single_date` 태깅을 달력월(`&date[..7]`) → 교습기간 `year_month`로 통일하여 `generate_impl`과 일관성 확보. 단위 테스트 3건 추가
- Sprint 20: **출결 생성 버튼 숨김 버그 수정(버그 A)** — `count_ungenerated`가 단순 미생성 레코드 수가 아닌 기대 수업일 대비 판정으로 교체되어, 부분생성 상태에서도 생성 버튼이 올바르게 표시됨. 회귀 테스트 1건

---

## [1.2.0] - 2026-07-09

### Added
- Sprint 19: **수강생대장 출력** — 원생관리 화면에서 전체 원생 수강 현황을 A4 표 형식으로 인쇄. 매 페이지 반복 헤더 + 페이지 번호, 브라우저 기본 머리글/바닥글 제거. `src/lib/student-roster-print-html.ts` 신규, `src/app/students/roster-print` 신규 라우트
- Sprint 19: **인쇄 팝업 아키텍처 전환** — 교습일정 인쇄를 기존 같은 창 인쇄 방식에서 Tauri 네이티브 창(`WebviewWindow`) 기반 독립 팝업 인쇄 방식으로 전면 재설계. `src/lib/academic-print-html.ts` 신규, `src/app/academic/print` 신규 라우트, `capabilities/default.json` 권한 추가. 수강생대장 출력도 동일 아키텍처 재사용
- Sprint 19: **공통 정렬 훅** (`useTableSort`) — 모든 그리드에서 재사용 가능한 클릭 정렬 인프라. `withTiebreak`로 동일 값 시 이름 2차 정렬 일반화
- Sprint 19: **학년 자동 승급** — 매년 1월 이후 최초 실행 시 확인 다이얼로그 + 전체 원생 일괄 학년 UP. 2026년은 이미 수동 처리 완료로 제외
- Sprint 19: **학교급 기반 학교 선택 필터링** — 원생 등록/수정 폼에서 학교급 선택 시 해당 학교급의 학교만 드롭다운에 표시. 코드 관리 화면에 `school_type` 편집 UI 추가
- Sprint 19: **V310 마이그레이션** — `schools.school_type` 텍스트 패턴 기반 자동 보정(초등학교→elementary, 중학교→middle)
- Sprint 19: **사용 매뉴얼** — 화면별 사용법을 담은 HTML 매뉴얼을 앱 리소스로 번들, 사이드바 "설정"↔"종료" 사이 "매뉴얼" 메뉴 클릭 시 시스템 기본 브라우저로 열람. `src-tauri/resources/manual/`, `tauri.conf.json` bundle.resources 등록

### Changed
- Sprint 19: 모든 원생 관련 그리드 기본 정렬을 학년+이름순으로 통일 (원생 목록, 출결, 청구, 수납, 공지문)
- Sprint 19: 출결 그리드 이중 스크롤 컨테이너 해소 — 가로/세로 스크롤바가 뷰포트 내 항상 접근 가능한 위치에 고정
- Sprint 19: 청구·수납 관리 그리드 좌우+상하 스크롤 지원 추가
- Sprint 19: 원생관리 페이지네이션 제거 + 50명 제한 해제 (전체 목록 단일 뷰)
- Sprint 19: 청구관리 년월 선택을 이전/다음 네비게이터 방식으로 통일
- Sprint 19: 수업관리 캘린더 주보기를 월보기와 동일한 3열 이름 그리드 + 셀 높이 확장 방식으로 재설계 — 이름 칩 좌우 위치 고정, 동시 인원별 동적 열 확장
- Sprint 19: 수업관리 캘린더 기본 진입 뷰를 월보기로 변경
- Sprint 19: 교습일정 인쇄 캘린더 Red 테두리 + 기간성 학사일정 밴드(셀 배경색 직접 칠하기) + 폰트 확대 + 동적 행 수 계산 (빈 행 제거)
- Sprint 19: 대시보드 당일수업 + 생일 위젯 상하 배치 + 2:1 비율
- Sprint 19: 수업관리 캘린더 라인을 진하게 조정
- Sprint 19: 좌측 사이드바 메뉴 폰트 한 단계 확대 (text-sm → text-base)
- Sprint 19: 전체 메뉴 페이지(대시보드 제외) UI 밀도 조정 — 행간 1.25 통일, 패딩/여백 축소
- Sprint 19: 접근성 폰트 기준 개정 — 본문 18/16pt → 16/14pt로 하향 조정 (실사용 피드백 반영), CLAUDE.md 반영
- Sprint 19: h1~h6 전역 스타일을 `@layer base`로 감싸 유틸리티 클래스 무력화 버그 수정 (CSS Cascade Layers 오래된 버그 근본 수정)

### Fixed
- Sprint 19: 인쇄 창(팝업)을 닫으면 앱 전체 DB pool이 종료되던 심각한 버그 수정 — `src-tauri/src/lib.rs` `on_window_event` 핸들러에서 메인 창 이벤트만 pool close 트리거하도록 수정
- Sprint 19: FullCalendar 주보기 2xN/10xN 겹침 버그 근본 수정 — 시간대별 칩 좌우 위치가 실제 column과 어긋나던 문제 해결
- Sprint 19: 인쇄 대화상자 닫힘(취소 포함) 시 인쇄 팝업 창 자동 종료
- Sprint 19: 기간성 학사일정 밴드 배경색이 인쇄에서 빠지던 문제 수정 (셀 배경색 직접 칠하기 방식으로 전환)
- Sprint 19: 주보기 셀이 다음 시간대 위로 넘쳐 보이던 문제 근본 수정
- Sprint 19: 원생 상세/수정 폼 여백 과다 문제 개선

---

## [1.1.0] - 2026-07-02

### Added
- Sprint 18: **수업관리 캘린더 주보기 색상 4색 체계** — 수업 시간 기준 60분(파랑)/120분(초록)/180분(노랑)/240분(빨강). 원생별 개별 이벤트로 FullCalendar 균등 열 자동 배치. 2시간 이상 수업은 1시간 슬롯 단위 분할 칩(↑/↓ 연속 표시)
- Sprint 18: **수업관리 캘린더 월보기 원생 이름 표기** — 월 보기 셀에 원생 이름 2열 그리드 직접 표기, 수업 시간 기준 4색 배지, hover 시 상세 정보
- Sprint 18: **학사 일정 변경 시 출결 자동 동기화** — `sync_attendance_on_schedule_change` 함수 신규. 일정코드 `allows_regular_class` ON→OFF 전환 시 해당 날짜 정규 출결 삭제, OFF→ON 전환 시 해당 요일 정규 스케줄 원생에게 출결 자동 생성. `create_schedule_event`/`update_schedule_event`/`delete_schedule_event` IPC 3종 연동. 단위 테스트 2건 추가
- Sprint 18: **교습일정 인쇄 기능** — 확정 교습기간 선택 후 A4 세로 레이아웃 교습일정 인쇄. `AcademicSchedulePrint` 컴포넌트 신규, `@media print` CSS 추가, 일정관리 화면에 인쇄 버튼 연결

### Changed
- Sprint 18: 수업관리 캘린더 기본 뷰 월보기 → 주보기(timeGridWeek) 변경
- Sprint 18: 수업관리 캘린더 요일 시작 월요일(firstDay:1) → 일요일(firstDay:0) 변경
- Sprint 18: 결제선생 결제수단 카드사 선택 optional 활성화 — 기존 `is_card_type=1` 필수에서 항상 선택 가능으로 변경
- Sprint 18: `app.lock` stale 임계값 600초 → 86400초(24시간) 상향 — 1인 운영 실사용 패턴 반영 (A107)
- Sprint 18: `auto_restore_with_retry` rollback 파일명에 루프 인덱스 접미사 추가 — 동시 복원 시도 시 파일명 충돌 방지 (A108)
- Sprint 18: `cleanup_stale_tmp_backups`를 `tokio::task::spawn_blocking`으로 래핑 — 비동기 런타임 블로킹 방지 (A110)
- Sprint 18: WAL 체크포인트 실패 시 `pool.close()` 후 early return — 커넥션 풀 누수 방지 (A111)
- Sprint 18: V308 — 보강데이 `is_duplicate_blocked=0` (동일 날짜 복수 일정 코드 허용)
- Sprint 18: V309 — 공휴일 `is_duplicate_blocked=0` (공휴일 + 보강데이/공휴수업 복수 배치 허용)
- Sprint 18 후속: 수업관리 캘린더 주/일보기 겹침 열 배정을 하루 단위 interval greedy packing(`assignColumns`)으로 재구현 — 여러 시간대에 걸친 원생이 항상 동일한 열에 표시되도록 보장. 겹침 3명 이상은 주보기만 2열×N행 재배치(일보기는 폭이 넓어 한 행 유지)
- Sprint 18 후속: 수업관리 캘린더 월보기 원생 배지 2열→3열 그리드, 색상을 주보기와 동일한 `colorForDuration` 기준으로 통일
- Sprint 18 후속: 수업관리 캘린더 3시간(180분) 색상을 amber에서 violet 계열로 교체 — 교습일 셀배경(amber)과 가독성 구분
- Sprint 18 후속: 수업관리 캘린더 다중 슬롯 화살표(↑/↓) 위치를 원생 이름 뒤로 이동
- Sprint 18 후속: 일정관리(학사) `ThreeMonthCalendar` 요일 시작을 일요일로 변경 — 수업관리 캘린더만 적용되고 누락됐던 부분 보정
- Sprint 18 후속: 교습일정 인쇄 버튼을 교습기간 미등록 시에도 항상 표시(비활성화+안내 툴팁)로 변경 — 기존에는 조건 미충족 시 버튼 자체가 숨겨져 찾기 어려웠음
- Sprint 18 후속: 교습일정 인쇄 레이아웃 A4 가로 방향 + 여백 최소화(브라우저 기본 머리글/바닥글 억제) + 달력 셀 균등 배분으로 개선
- Sprint 18 후속: 결제선생 카드사 매칭에 라벨(`결제선생`) 폴백 추가 — 코드값 불일치 레거시 데이터 대비
- Sprint 18 후속: 수납 관리 · 월별 집계 탭의 청구년월/연도 선택 UI를 드롭다운에서 일정 관리와 동일한 "◀ 이전 / 년월 / 다음 ▶" 방식으로 통일

### Fixed
- Sprint 18: `auto_restore_with_retry` 단위 테스트 정리 — retry 성공/실패 시나리오 커버 (A109)
- Sprint 18 후속: 수업관리 캘린더 주보기에서 원생 칩 hover 시 `Cannot read properties of undefined (reading 'split')` 크래시 수정 — hover 강조용 background 이벤트가 `eventContent`에서 `extendedProps` 없이 처리되던 문제
- Sprint 18 후속: 교습일정 인쇄 클릭 시 미리보기가 빈 화면으로 뜨던 문제 수정 — Next.js App Router에는 Pages Router 전용 `#__next` 래퍼가 없어 인쇄 wrapper가 상위 트리와 함께 숨겨졌음. `createPortal`로 `document.body` 직속 렌더링으로 변경
- Sprint 18 후속: 학사 일정 변경 시 출결 자동 동기화(`sync_attendance_on_schedule_change`)가 공휴일(OFF)과 공휴수업일(ON) 이벤트가 같은 날짜에 공존할 때 무조건 OFF로 판정해 출결이 생성되지 않던 버그 수정 — ON 이벤트 존재 시 우선하도록 변경, 회귀 테스트 추가

### Changed
- Sprint 17: hourly 백업 생성 간격 3600초 → 7200초 (MYBOX 부하 절감)
- Sprint 17: `app.lock` heartbeat 완전 제거 — 1인 운영 정책으로 잠금 파일 단순화 (시작 시 생성 / 종료 시 삭제)
- Sprint 17: DB 폴더 변경(`change_db_folder`) WAL 체크포인트 실패 시 복사 중단 + 사용자 친화 오류 메시지 반환 (기존: 에러 묵살)
- Sprint 17: 자동 복원 로직 `auto_restore` → `auto_restore_with_retry` — quick_check 재검증 최대 3회 retry, 모두 실패 시 다음 백업으로 순차 시도

### Removed
- Sprint 17: `SyncStatus` 백엔드 IPC + 프론트엔드 polling + UI 상태바 표시 완전 삭제 (`sync.rs` 삭제, `app-shell.tsx`, `top-bar.tsx`, `types/index.ts`, `tauri/index.ts` 정리)

### Fixed
- Sprint 17: 백업 파일 저장 방식을 직접 쓰기에서 tmp 파일 후 rename (atomic write)으로 변경 — 클라우드 동기화 중 백업 파일 손상 방지. stale tmp 파일 자동 정리 함수 추가

---

## [1.0.0] - 2026-06-12

> **정식 출시 (v1.0)** — Sprint 15~16 작업을 포함한 첫 정식 릴리즈. v0.6.0 단독 배포를 폐기하고 v1.0으로 직행. 단원평가/학습보고서(Phase 5)는 개발 취소됨.

### Added
- Sprint 16: **수업일 변경 도메인** — ① 케이스1 1회성 이동(출결 행 `event_date` 이동 + 시작시간 시(時) 단위 입력 + 이동 메모, 동월·평일·미충돌·present만 허용) ② 케이스2 특정일 이후 영구 변경(`effective_from` 시계열 스케줄 + 미처리 출결만 재생성, 결석·보강·메모 행 보존). `move_attendance`/`apply_schedule_change` IPC, `generate` 날짜 인식 리팩토링, 출결표 이동 다이얼로그 + ScheduleEditor 변경일 선택. DB 마이그레이션 V306(`regular_attendances.note`)·V307(`regular_attendances.start_time`)
- Sprint 16: **수업 캘린더 개선** — 주/일/월 보기 원생별 색칩 + 수업 시간 표기(길이 구분), 주 보기 2열(2×N) 묶음·일 보기 원생별 개별 블록(실제 길이), 칩 hover 시 수업 시간 범위 테두리 강조, 월 헤더 요일 표기
- Sprint 16: **원생 CSV 가져오기** (`/settings/import`, T2) — `students` 테이블 일괄 등록. UTF-8/EUC-KR 자동 인코딩(BOM 처리), 학년("초3"·"중2") 파싱, 미리보기 드라이런(파싱·검증·중복판정) → 가져오기 전 자동 백업 1회 → 단일 트랜잭션 INSERT. 중복(일련번호 또는 이름+모연락처) skip. `csv`·`encoding_rs` 신규 의존성
- Sprint 16: **공지문 캔버스 요소 보강** — 교습소 로고·2D바코드 이미지 요소(체크박스 on/off, 드래그 + 비율 유지 리사이즈) + 임의 이미지 추가(`customImages`) + 텍스트박스 배경색 지정. 미등록 이미지 체크 시 안내 팝업
- Sprint 16: **공지문 교습일정 달력 이미지** — 청구년월 학사일정을 일요일 시작 달력 PNG로 렌더해 공지문 캔버스에 합성. 교습기간 빨간 외곽선(첫 평일 수업일~마지막 평일 수업일, 토·일 제외), 특이일 라벨(공휴일·보강데이), 기간 하이라이트. 신규 의존성·마이그레이션 없음(`src/lib/calendar-image.ts`)
- Sprint 16: **DB 폴더 변경** (`/settings/db-folder`, T3, ADR-009) — 클라우드 동기화 경로 재지정 + `smarthb/` 전체(DB·salt.bin·assets·output·backup) 동반 이전. copy-then-switch + 재시작 방식(WAL checkpoint → 재귀 복사·fsync → cipher 무결성 검증 → 원본 MOVED_TO 마커 → config.json 갱신). 실패 시 기존 폴더 유지. `tauri-plugin-process` 신규
- Sprint 16: **백업 복원 연결** — ① 시작 시 무결성 손상(`quick_check` Failed) 감지 시 최신 정상 exit 백업으로 자동 복원 + 손상본 보존 + 고지 배너 ② `/settings/backup`에 "이 백업으로 복원" 수동 버튼(확인 모달 + 재시작)
- Sprint 16: **daily/weekly 백업 스케줄러** — catch-up 방식(앱 시작 + hourly tick마다 최신 백업 24h/7d 경과 또는 0건 판정 시 생성). `is_due` 순수 함수 + 단위 테스트
- Sprint 16: **청구/수납 메뉴 분리** — '청구/수납 관리' → '청구 관리'(`/billing`) + '수납 관리'(`/payments` 신설, 수납 + 월별 집계 탭). 공통 로직 `useBillingShared` 훅 + `BillingSummaryBar`/`BillingSearchBar` 추출
- Sprint 16: **미저장 이탈 경고 공통 훅** `useUnsavedChanges` (T1) — 창 닫기(beforeunload) + Ctrl+S 저장 + 메뉴 이동 가드를 한 줄로 일괄 획득. 입력 필드 포커스 중 Ctrl+N 억제(A99)
- Sprint 16: **원생 폼 UX 개선** — 임시저장 입력 즉시 저장(이탈 손실 방지) + 재진입 시 '이어서 작성' 복원, 원생 목록·신규 버튼 임시저장 배지, 성별·학년·학교 필수 입력 검증(미지정 차단), 학교급↔학교명 정합성 검증
- Sprint 15: 교습소 정보 화면 신설 (`/settings/info`) — 교습소명·대표자·연락처·주소·사업자등록번호·최대 인원·면적·운영 시간 등 9필드 CRUD + 로고/2D바코드 이미지 2종 업로드·미리보기·삭제. `app_settings` JSON 저장(마이그레이션 없음), 기존 `notice_asset` IPC 재사용
- Sprint 15: 자가 진단 이력 수동 삭제 — `/settings/diagnosis` 이력 목록에 행 단위 삭제 버튼 + 전체 비우기 버튼 추가. `delete_diagnosis_history(id)` / `clear_diagnosis_history()` IPC 2종, 확인 다이얼로그(PRD §5.7), 단위 테스트 3건
- Sprint 15: 전역 `GlobalShortcuts` 컴포넌트 신설 — Ctrl+F(글로벌 검색 포커스), Ctrl+N(신규 원생) 단축키 전역 등록
- Sprint 15: 전역 `GlobalTooltip` 컴포넌트 신설 — 브라우저 기본 `title` 툴팁을 20px 커스텀 팝업으로 통일
- Sprint 15: 원생 상세 화면에 '원생 관리 메인으로' 버튼 추가

### Changed
- Sprint 16: 사이드바 UX 개선 — 활성 메뉴 강조(좌측 보더 + 배경 + 볼드, `aria-current`) + 메뉴 그룹 여백·구분선 + 너비 20% 축소
- Sprint 16: 확인/취소 버튼 높이 `h-8`→`h-11`(44px) + 본문 `text-base` 상향 (50대 사용자 접근성, P1)
- Sprint 16: 본문 보조 텍스트 `text-gray-500` → `text-muted-foreground` 72곳 일괄 교체 (WCAG AA 명도 대비, P1)
- Sprint 16: 백업 보관 개수 축소 — exit 10→5 / hourly 24→12 / daily 30→14 / weekly 4 유지 (합계 68→35, 1인 시스템 + 클라우드 동기화 폴더 점유 고려, PRD §5.4 v1.5.2)
- Sprint 16: Pretendard 폰트 subset → full woff2 교체 (공지문 캔버스 렌더 시 희귀 한자·특수문자 fallback 방지)
- Sprint 16: 자가진단 검사3(청구 미생성) 청구 대상(현행 스케줄 + 주당시간>0)만 검사 — 스케줄 없는 재원생 만성 오탐 제거 (P2)
- Sprint 16: 학사코드 색상 SSOT 통합 (`src/lib/schedule-code-colors.ts`) — 공지문 달력 색을 앱과 일치(보강데이 teal·공휴수업일 pink, P2)
- Sprint 15: 설정 허브 카드 순서 재배치 (PIN 카드 위치 조정) + '마법사 재실행' 카드 → 'DB 폴더 변경(예정)' 안내 카드 전환(disabled, Sprint 16 예정)
- Sprint 15: `dashboard.rs` `monthly_summary` GROUP BY 서브쿼리 패턴으로 리팩토링 (R99 해소 — 부분 수납 확장 대비)
- Sprint 15: 대시보드 위젯 타이틀 inline `fontSize` → Tailwind `text-2xl` 통일 (A97)
- Sprint 15: 청구 생성 `standard_fees` 조회 N+1 쿼리 → IN 쿼리 단일 배치로 전환

### Fixed
- Sprint 16 (코드리뷰 P0, 데이터 안전): 앱 종료 시 WAL checkpoint(TRUNCATE) + connection pool close — 양 PC 클라우드 동기화 간 torn-sync(미반영 WAL) 차단
- Sprint 16 (코드리뷰 P0): `config.json` 저장 시 fsync 적용 (전원 손실 시 손상 방지)
- Sprint 16 (코드리뷰 P0): `todayLocalISO()` 로컬 날짜 헬퍼 — 오전 시간대(UTC 변환 시 어제로 밀림) 날짜 입력 버그 3곳 수정
- Sprint 16 (코드리뷰 P0): 수납 입력 draft 보호(refetch 시 미저장분 보존 + 탭/월 변경 확인 모달), 출결 그리드 Ctrl+Z editable 가드, 요일 변경 원자 커맨드(`change_schedule_day`)
- Sprint 16: 수업일 이동(케이스1) 후 주간 캘린더 클릭 시 `padStart` 크래시 수정 — 이동 출결의 `start_time` 부재를 `COALESCE`(V307) + null 가드로 해소
- Sprint 16: 정규수업 가능 판정을 `allowsMakeup` → `regular_blocked`(allows_regular=0)로 교정 — 주말·보강데이 이동 차단 오판 수정
- Sprint 16 (코드리뷰 P2): academic create/update/confirm 커밋 후 소멸 전이 실패를 fail-soft로 통일("성공인데 실패 보고" UX 버그 제거), `/settings/codes` 저장 실패 표시 + 행 입력 props 동기화
- Sprint 15: WCAG AA 명도 대비 미달 `text-gray-400` → `text-gray-600` 17건 수정 (전체 화면)

### Build
- Sprint 16: Windows 설치 패키지를 `.exe`(NSIS) 전용으로 제한 (msi 제외, macOS는 dmg) — 파일명 `SmartHB_{ver}_x64-setup.exe`

## [0.6.0] - 2026-06-06

### Added
- Sprint 14: 대시보드 (`/`) — 교습소 현황 위젯(재원 총원·학년별/성별/학교별 비율·분기별 입퇴교 추이), 당일 수업 위젯(12시간제 am/pm·시간대별 인원/명단), 월 요약 위젯(이전/다음 월 전환·청구/입금/미납/당월 입퇴교), 교습소 월별 청구총액 추이 그래프(최근 12개월), 이달의 생일 위젯, 메모 3슬롯(포스트잇·드래그 리사이즈·1초 디바운스 자동저장), 알림 4종(보강 소멸 임박·미확정 청구·학사 미수립·자가 진단 이상)
- Sprint 14: 데이터 자가 진단 (`/settings/diagnosis`) — 검사 7종(보강필요시간 음수·재원중 출결 미생성·재원중 청구 미생성·스케줄-출결 불일치·결석 소멸기한 미설정·고아 보강 데이터·수납 정합성), 수동 실행 + 매월 1일 자동 실행(app_settings last_auto_diagnosis 분리), 12개월 이력 보관, 해결 항목 자동 재검증·정리
- Sprint 14: 데이터 내보내기 (`/settings/data`) — 원생 명단·출결·청구-수납 엑셀(.xlsx) 내보내기(rust_xlsxwriter 0.95, 단월/전체 기간 선택, OS 저장 다이얼로그, 금전 천단위 콤마+우측정렬, autofit 컬럼너비)
- Sprint 14: 복원 리허설 (`/settings/backup`) — 백업 파일 목록 표시 + "복원 리허설" 버튼(임시 복사→integrity_check→주요 6종 행수→사본 폐기, 운영 데이터 무영향)
- Sprint 14: 원생 생년월일 필드 추가 — 원생 등록/수정 폼, 원생 목록 컬럼, 엑셀 내보내기 컬럼 포함(선택 입력, V305 마이그레이션)
- Sprint 14: DB 마이그레이션 V303(`diagnosis_history` 테이블), V304(퇴교생 미보강 결석 일괄 소멸 백필), V305(`students.birth_date` 컬럼)
- Sprint 14: recharts 3.8.1 신규 의존성 (대시보드 차트, dynamic import ssr:false)
- Sprint 14: rust_xlsxwriter 0.95 신규 의존성 (엑셀 내보내기, 순수 Rust, Win/Mac 안전)
- Sprint 13: 실행 시 PIN 인증 옵션화 (ADR-008: 기기별 선택적 PIN 게이트) — `/settings` 화면에 '실행 시 PIN 인증 사용' 토글(Switch) 추가. OFF 설정 시 앱 재시작 시 키체인 자동 잠금해제 → PIN 입력 없이 메인 진입. 키체인 키 부재 시 안전 폴백(PIN 입력 요구)
- Sprint 13: `auto_unlock_with_keychain` IPC 신규 (`startup.rs`) — 키체인 유도키 직접 로드, `run_startup` 공통 시퀀스 추출 (락+무결성+DB초기화+heartbeat/backup+소멸전이 공통화)
- Sprint 13: `skip_pin_on_launch` config.json 플래그 get/set IPC — `get_pin_skip_setting` / `set_pin_skip_setting` (DB 밖, app_config_dir, PC별 독립, unlock 전 호출 가능)
- Sprint 13: TypeScript IPC 래퍼 3종 추가 — `getPinSkipSetting`, `setPinSkipSetting`, `autoUnlockWithKeychain`
- Sprint 13: ADR-008 신규 작성 (`docs/arch/adr-008-optional-pin-gate.md`) — PRD §5.5 인증 의무 완화 결정, 데이터 보호를 OS 계정+키체인 ACL로 위임하는 트레이드오프 명시

### Changed
- Sprint 14: 대시보드가 기본 진입 화면(`/`)으로 활성화 — 기존 리다이렉트 placeholder에서 실제 대시보드 컴포넌트로 교체
- Sprint 14: 내보내기 형식 CSV→엑셀(.xlsx) 전환 — 정렬·서식(천단위 콤마·우측정렬·autofit) 지원, BOM 불필요
- Sprint 14: `compute_summary` 보강필요시간 이월 누적으로 변경 — 소멸기한이 조회월 이상인 이월 결석 포함(퇴교생 제외), `earliest_pending`과 정합
- Sprint 14: `next.config.ts` dev 전용 webpack 캐시 비활성화 — Node 25 V8 webpack 캐시 직렬화 abort 크래시 회피(production 빌드 무영향)
- Sprint 13: `/lock` 진입 흐름 분기 — `skip_pin_on_launch=true` 시 `autoUnlockWithKeychain` → 성공 시 메인 진입, 실패 시 기존 LockScreen 폴백. 자동 잠금해제 중 로딩 스피너 표시

### Fixed
- Sprint 14: 자가진단 검사 1(보강필요시간 음수) 오탐 수정 — 결석 대상을 `absent`+`makeup_done`으로 변경(소멸 `makeup_expired` 면제)
- Sprint 14: 대시보드 출결 진행률 공휴일 오탐 수정 — `allows_regular_class=0` 학사일정 기간을 비수업일에서 제외하도록 보정
- Sprint 14: 보강 소멸 임박 알림 퇴교생 제외 — 퇴교한 원생의 미보강 결석이 알림에 노출되던 버그 수정
- Sprint 14: `update_student`·`withdraw_student` 경로 퇴교 시 미보강 결석 미처리 버그 — V304 마이그레이션으로 기존 데이터 백필 + 알림 쿼리 퇴교 필터 추가
- Sprint 13: 글로벌 검색 원생 클릭 404 수정 — `/students/{id}` 라우트 미존재로 404 발생하던 버그를 `/students/edit?id=` 경로로 교정
- Sprint 13: 글로벌 검색 드롭다운 방향키 탐색 + 한글 IME 처리 — `ArrowUp`/`ArrowDown`/`Enter`/`Escape` 키 바인딩 추가, 한글 조합 중(`isComposing`) 방향키 오작동 방지
- Sprint 13: `save_notice_preview` 경로 경계 검증 (R88) — 절대경로 + `.png` 확장자 검사 + path traversal 차단, `data_root()` 밖 폴더 자동생성 금지

### Removed
- Sprint 13: Phase 5 (단원평가/학습보고서) 메뉴 항목 완전 제거 — `menu-config.ts`에서 '단원 평가'(`/exams`) + '학습 보고서'(`/reports`) 항목 삭제. PRD §4.7/§4.8/§6.1 [CANCELLED] 표기

### Added
- Sprint 12: 공지문 편집 화면 (`/notices` 라우트) — 좌측 원생 리스트(체크박스 다중 선택/전체선택), 우측 배경서식 + 텍스트박스 3종(청구월/원생이름/청구액) 오버레이, react-rnd 드래그+리사이즈, 다중 선택(Shift+클릭), 방향키 미세 이동(1/10px), 빈 영역 클릭 선택 해제, 사이드바 "공지문" 메뉴 활성화
- Sprint 12: 공지문 이미지 일괄 생성 엔진 (`src/lib/notice-generator.ts`) — Canvas 2D 직접 렌더 (macOS WKWebView foreignObject+img 결함 회피), 원생별 PNG 일괄 생성, 청구액 천단위 콤마(AC-4.10-1), 재생성 덮어쓰기 확인(AC-4.10-2), 진행률 표시, 미리보기 팝업 + 파일 저장 다이얼로그
- Sprint 12: 공지문 백엔드 IPC (`src-tauri/src/commands/notice.rs` 신규) — `list_notice_assets`, `save_notice_asset`, `delete_notice_asset`, `save_notice_layout`, `get_notice_layout`, `save_notice_image`, `save_notice_images_batch`, `check_notice_output_exists`, `open_notice_output_folder` 9종 + 단위 테스트
- Sprint 12: 공지문 저장 경로 규칙 — `{data_root}/output/{공지문이름}/{YYMM}/{이름}_{YYMM}_{원생}.png` (공백 제거, 한글 NFC 정규화, 클라우드 동기화 폴더 공유)
- Sprint 12: 공지문 레이아웃 저장/로드 (AC-4.10-3) — 텍스트박스 위치/크기/폰트/글자별 색상(`TextboxConfig.charColors`)을 `app_settings` JSON으로 유지, 진입 시 자동 로드
- Sprint 12: 저장 위치 클릭 시 폴더 열기 — 출력 폴더가 없으면 자동 생성 후 OS 파일 탐색기로 오픈
- Sprint 12: 청구월 컨트롤 년도 없이 월만 표시 (UI 간결화)
- Sprint 12: `components/ui/pin-field.tsx` 6박스(OTP) PIN 공용 컴포넌트 신규 — LockScreen + 설정 PIN 변경 화면(`/settings/pin`) 동일 UI로 통일
- Sprint 12: `/settings/pin` 라우트 신설 — 현 PIN 검증 후 새 PIN 설정 흐름 + `change_pin` IPC 백엔드
- Sprint 12: 공지문 TypeScript IPC 래퍼 8종 + `src/types/notice.ts` 도메인 타입 (`NoticeAsset`, `NoticeLayout`, `TextboxConfig`, `SaveNoticeResult`)
- post-Sprint 11 (develop 보완): 앱 잠금 인증을 6자리 숫자 PIN 으로 전환 — `LockScreen` / `RecoveryCodeInput` 입력 전환, 백엔드 `validate_pin` (길이 6 + ascii digit) 진입점 재검증, dev autologin + `.env.example` 6자리 PIN 대응 (ADR-007: `docs/arch/adr-007-pin-authentication.md`)
- post-Sprint 11 (develop 보완): ADR-007 신규 작성 — 6자리 숫자 PIN 보안 트레이드오프 명시 수용, 복구코드 12자리 유지 결정
- post-Sprint 11 (develop 보완): 청구 관리 '월별 집계' 탭 — 년/월 토글(연도 `YYYY-%` 집계 / 월 집계), 요약 박스 + 결제수단별 수납총액(열 배치). 백엔드 `get_billing_period_stats(period)` IPC + `BillingPeriodStats`/`PaymentMethodSummary` 타입. 단위 테스트: `billing_period_stats_groups_by_method`
- post-Sprint 11 (develop 보완): 월별 집계 기간 선택을 실제 청구 생성된 년월로 한정 — `list_billed_months` IPC (`bills` distinct `bill_year_month` DESC), 집계 탭 드롭다운이 생성된 청구 없는 년월은 표시하지 않음. 단위 테스트: `list_billed_months_returns_distinct_desc`

### Changed
- Sprint 12: 사이드바 메뉴 순서 변경 — 출결관리/학사 순서 swap + '학사 스케줄' → '학사 관리' → '일정 관리' 표기 일괄 변경 (`src/lib/menu-config.ts`, sidebar, global-search)
- Sprint 12: 배경서식 미선택 시 텍스트박스 입력/체크박스 동작 정비 — 배경 없이도 텍스트 미리보기 가능
- post-Sprint 11 (develop 보완): 청구 탭 상태 필터에 '마감' 추가 + 옵션별 건수 표기(전체/확정/미확정/마감), '마감 완료' 배지를 상태 필터 앞쪽으로 이동
- post-Sprint 11 (develop 보완): 수납 탭 필터 건수 표기(전체/수납완료/미수납) 추가
- post-Sprint 11 (develop 보완): 마감 후 수정 사유 게이트 완화(10자 이상 → 비어있지 않음)
- post-Sprint 11 (develop 보완): 입금일 선택 시 달력 닫고 입금자 칸으로 포커스 이동 UX
- post-Sprint 11 (develop 보완): 월별 집계 탭 — 청구 데이터 0건 시 현재 년월을 디폴트로 표시하여 빈 화면 대신 "0건" 상태 노출

### Fixed
- Sprint 12: 빈 이미지 생성 버그 수정 — html-to-image toPng() 가 macOS WKWebView에서 foreignObject+img 렌더링 실패로 빈 PNG 반환하는 결함을 Canvas 2D 직접 렌더 방식으로 전환하여 해소
- Sprint 12: 공지문 화면 좌우 패널 위치 swap — 원생 리스트(우→좌), 미리보기(좌→우) 배치 변경 (사용자 UX 요청)
- Sprint 12 (`fix(schedules)`): 월보기 캘린더 일자별 인원수 초기 렌더링 누락 수정
- post-Sprint 11 (develop 보완): 확정 버튼 비활성 버그 수정 — 마감 후 수정 사유 게이트 10자 조건으로 인한 오작동 해소
- post-Sprint 11 (develop 보완): 수납완료 행 수납 취소 기능 추가 (`batch_update_payments` 재사용, 신규 IPC 없음) — 잘못 입력된 수납 정정 가능
- post-Sprint 11 (develop 보완): 입금 완료 시 결제수단 필수 검증 — 백엔드 `validate_payment_input` 2곳 + 프론트 가드/빨간 테두리. 신규 단위 테스트: `create_payment_rejects_paid_without_method`, `batch_cancel_payment_resets_is_paid`
- post-Sprint 11 (develop 보완): 수납완료된 청구는 수정 불가 — `update_bill_impl` 가 `is_paid` 기준으로 거부 + 프론트 금액 편집 비활성. 신규 단위 테스트: `update_bill_paid_rejected`

### Removed
- Sprint 12: **복구 코드 시스템 전면 제거** (사용자 결정 — cipher OFF 환경에서 불필요). 제거 항목: `src-tauri/src/commands/recovery.rs`, `RecoveryCodeInput.tsx`, `RecoveryCodeDisplay.tsx`, `src/lib/recovery-code.ts`, `argon2` Cargo 의존성, audit `RecoveryCodeIssued` variant, TypeScript `AuditEventType` 의 `'recovery-code-issued'`
- post-Sprint 11 (develop 보완): **청구 '마감(closed)' 개념 전면 폐기** (원장 결정, 2026-05-30). 청구 상태는 미확정→확정 2단계로 축소. 제거 항목: `close_billing_month` IPC, `CloseMonthDialog`/`CloseReasonDialog` 컴포넌트, "당월 청구 마감" 버튼·"마감 완료" 배지·'마감' 상태 필터, audit `BillMonthClosed`/`BillClosedModified`, `update_bill` 의 `close_reason` 파라미터. DB 마이그레이션 **V111** — `bills` 재구성으로 `status` CHECK(draft/confirmed) + `close_reason`/`closed_at` 컬럼 제거(기존 closed → confirmed 흡수). PRD §4.9.7 갱신, AC-4.9-7/8 폐기, AC-4.9-9 신설(수납완료 청구 수정 불가)

### Added
- Sprint 11: DB 마이그레이션 V109 — `bills` + `payments` 테이블 신규 (청구 3단계 상태 머신 draft/confirmed/closed, 수납 1:1 별도 테이블 PI-12 확정, UNIQUE: `(student_id, bill_year_month)` + `bill_id`, FK: `students(id)` / `bills(id)` / `payment_methods(id)` / `card_companies(id)`)
- Sprint 11: 청구 IPC 4종 (`src-tauri/src/commands/billing.rs` 신규) — `generate_bills` (재원 원생 일괄, 표준 교습비 매핑, 월중입퇴교 플래그 자동), `list_bills` (미확정+월중입퇴교 상단 우선), `get_bill`, `update_bill` (상태별 수정 제약), `get_default_bill_year_month` — 단위 테스트 17건
- Sprint 11: 청구 상태 머신 IPC 3종 — `confirm_bill` (단건), `confirm_all_bills` (일괄), `close_billing_month` (전체 confirmed 전제 조건 강제 AC-4.9-7), `update_closed_bill` (close_reason 필수 AC-4.9-8) — 단위 테스트 9건
- Sprint 11: 수납 IPC 5종 — `create_payment`, `update_payment` (카드 계열 card_company_id 필수 검증 AC-4.9-4), `list_unpaid_bills`, `batch_update_payments` (BEGIN IMMEDIATE 트랜잭션), `get_billing_summary` (총청구액/입금완료액/미납액) — 단위 테스트 9건
- Sprint 11: audit variants 3종 추가 — `BillConfirmed`, `BillMonthClosed`, `BillClosedModified`
- Sprint 11: 청구 마감 UX 다이얼로그 3종 — `CloseReasonDialog` (사유 입력 textarea ≥10자, shadcn/ui Dialog), `ConfirmBillUpdateDialog` (확정 후 수정 확인 AC-4.9-3), `CloseMonthDialog` (마감 확인 + 경고 문구)
- Sprint 11: TypeScript IPC 래퍼 13종 + `src/types/billing.ts` 도메인 타입 — `Bill`, `BillStatus`, `Payment`, `BillingSummary`, `BillListFilter` 등
- Sprint 11: `/billing` 라우트 신설 + `BillingGrid` 컴포넌트 — 년월 선택, 청구 생성/확정/마감 버튼, 미확정 상단 배너(AC-4.9-5), 월중입퇴교 amber-50 행 구분(AC-4.9-2), TanStack Query 캐싱. 사이드 메뉴 "청구 관리" 활성화
- Sprint 11: `PaymentsView` 컴포넌트 — [청구|수납] 2탭 통합, 입금 일괄 처리 모드 (max-h-[800px] overflow + sticky thead 최소 20행 AC-4.9-6), 월별 요약(총청구/입금/미납), 카드사 드롭다운 카드 계열 시에만 노출

### Changed
- Sprint 11: `payment_methods.is_card_type` 컬럼 추가 (V109 ALTER TABLE) — 카드 계열 결제수단 판별 (기존 시드 `code='card'` 1건 마킹)
- Sprint 11 (T0/F7): 사이드 메뉴 '보강 관리' (`/makeups`) `disabledHint` 제거 — Sprint 10 T11에서 `/schedules` 캘린더 뷰로 통합 완료 (`src/lib/menu-config.ts`)
- Sprint 11 (T0/F5): `ClassCalendar` viewType 비동기 상태 한 프레임 불일치 해소 (`src/components/schedules/ClassCalendar.tsx`)

### Fixed
- Sprint 11 (T0/F1): `build_day_schedules` `d.succ_opt().expect()` panic 가능성 해소 — `.ok_or_else()` 안전 전환 (`attendance.rs`)
- Sprint 11 (T0/F2): `generate_impl` expire 호출 실패 시 fail-soft 전환 — expire 실패해도 출결 생성 성공 반환, expire 에러는 warn 로그만 (`attendance.rs`)
- Sprint 11 (T0/F3): `calendar.rs` `_year_month` 미사용 파라미터 정리
- Sprint 11 (T0/F4): 보강관리 N+1 쿼리 → IN batch 1쿼리 (`calendar.rs` 한정) — 루프 내 개별 쿼리를 JOIN/IN 절로 batch 처리
- Sprint 11 (T0/F6): flaky 테스트 `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` `#[ignore]` 마킹 (동시성 설계 재검토 별도 backlog)

---

## [0.5.0] - 2026-05-28

### Added
- Sprint 10: 소멸 자동 전이 IPC (`src-tauri/src/commands/expiration.rs` 신규) — `expire_overdue_absences` + 3개 트리거 통합 (앱 시작 / 출결 생성 / 교습기간 등록). 소멸기한 도래 + 미보강 결석 → `makeup_expired` 자동 전이, 단위 테스트 7건
- Sprint 10: 소멸 전이 알림 UI — 앱 시작 / 출결 생성 / 교습기간 등록 후 전이 건수 토스트 (건수 > 0일 때만)
- Sprint 10: 퇴교 보강 처리 IPC 2종 — `get_pending_makeup_for_withdrawal` + `process_withdrawal_makeup` (즉시 소멸 / 보강 후 퇴교 / 외부 처리 후 소멸 3선택지), 단위 테스트 6건
- Sprint 10: 퇴교 보강 처리 UI — `WithdrawalMakeupDialog` (미사용 보강 보유 원생에게만 표시, 원생 관리 퇴교 흐름 통합)
- Sprint 10: 캘린더 집계 IPC 2종 (`src-tauri/src/commands/calendar.rs` 신규) — `get_calendar_data` (일별 수업 원생 목록) + `get_makeup_management_data` (보강 필요 원생, 소멸기한 임박 순), 단위 테스트 5건
- Sprint 10: 캘린더 뷰 UI (`/calendar` 라우트) — FullCalendar 일/주/월 뷰, 원생 상세 팝업 (출결/보강 상세 + 출결관리 이동), 보강 관리 전용 뷰 (소멸 임박 7일 강조). 수업 관리 메뉴 활성화. 7라운드 시각 검증 완료
- Sprint 10: ADR-006 캘린더 라이브러리 선택 (`docs/arch/adr-006-calendar-library.md`) — FullCalendar MIT 채택 (React Big Calendar 대비 일/주/월 뷰 + TypeScript + static export 호환성 우위)
- Sprint 9: 보강 IPC 백엔드 7종 (`src-tauri/src/commands/makeup.rs` 신규) — `get_pending_absences`, `get_makeup_eligible_dates`, `create_makeup_with_absences`, `cancel_makeup`, `mark_makeup_absent`, `batch_create_makeups`, `get_absence_history`
- Sprint 9: 보강 비즈니스 규칙 단위 테스트 28건 신규 (T2 9 + T3 9 + T4 7 + T8 3, PRD §6.5 100% 커버)
- Sprint 9: `audit::AuditEventType` — `MakeupCreated`, `MakeupCancelled`, `MakeupAbsent` 3 variant 추가
- Sprint 9: 보강 등록 UI — `MakeupRegisterDialog` (비수업일 셀 클릭, 충당 결석 다중 선택, 소멸기한 임박 순 정렬)
- Sprint 9: 보강 삭제 UI — `MakeupManageDialog` (보강일 emerald 셀 클릭 진입, 취소 시 결석 자동 환원)
- Sprint 9: 결석 이력 UI — `AbsenceHistoryDialog` (출결표 학생명 클릭, 미처리/보강완료/소멸 3종 시각 구분)
- Sprint 9: `src/lib/time.ts` 신규 — `minutesToHours` / `hoursToMinutes` / `formatHours` / `minutesToHoursText` (UI는 시간 단위, 백엔드는 분 단위 유지)
- Sprint 9: `src/types/makeup.ts` 도메인 타입 8종 — `PendingAbsence`, `EligibleDate`, `CreateMakeupPayload`, `MakeupResult`, `BatchMakeupEntry`, `BatchCreateMakeupsPayload`, `BatchFailure`, `BatchResult` + `AbsenceHistoryItem`
- Sprint 9: `src/lib/tauri/index.ts` 보강 IPC 래퍼 7종 추가

### Changed
- Sprint 9 (I3/T10): `get_makeup_eligible_dates` 보강 가능일 재정의 — 케이스 A (평일+보강불가코드없음) OR 케이스 B (`allows_makeup_class=1`). `study_periods` 범위 제약 제거 + T3 정규 수업 요일 차단 검증 3 폐기 (수업 후 추가 보강 허용)
- Sprint 9 (J4/J6): 보강일(emerald) 셀 신규 추가 — 보강 당일 그리드에 "보강" 라벨 emerald 배경으로 표시. 보강 삭제 진입점을 결석 셀에서 보강일 셀로 이동
- Sprint 9 (J7): 결석 셀 라벨 통일 — `absent`/`makeup_done` 모두 '결석' 표기 (`×` 제거), `makeup_done` 배경은 emerald (보강일 셀과 동일)
- Sprint 9 (J8/J9/J10): 출결 셀 양방향 tooltip — 결석 셀 hover 시 매칭 보강일자, 보강 셀 hover 시 충당 결석일자(다건 줄바꿈)
- Sprint 9 (A41/T7): 출결표 헤더 라벨 "결석" → "미처리\\n결석" (title 속성에 필터 조건 설명 추가)
- Sprint 9 (I2): 헤더 보강 필요 학생 수 표시 + 0명 시 disabled 처리
- Sprint 9 (I7): 출결표 일자 헤더 — `allowsMakeup=true` 일자 sky-100/sky-800 배경 강조 + "보강데이" title

### Changed
- Sprint 10: `audit::AuditEventType` — `MakeupExpired` variant 추가 (소멸 자동 전이 감사 로그)

### Removed
- Sprint 10 (T1): `mark_makeup_absent` IPC + `batch_create_makeups` IPC 완전 제거 (Sprint 9 J5/J7 사용자 결정 후 dead code 정리)
- Sprint 10 (T1): `audit::AuditEventType::MakeupAbsent` variant 제거
- Sprint 9 (J5): 보강 미등원 UI — `MakeupManageDialog`에서 "미등원" 옵션 제거 (사용자 결정 — 보강은 결과 기록 의미)
- Sprint 9 (J7): `BatchMakeupDialog` 컴포넌트 삭제 — 보강데이 일괄 기능 폐기 (사용자 결정)
- Sprint 9 (J7): 출결표 헤더 "보강데이 일괄" 버튼 제거
- Sprint 9 (K7): 출결표 헤더 'N / M 명' 별도 카운터 — 라벨 병기 형태로 통합

### Fixed
- Sprint 10: V108 마이그레이션 — `makeup_attendances.status` CHECK 제약 단순화 (`'makeup_absent'` 제거). FK 카운터 함정(SQLite code 787) TEMP 테이블 패턴으로 해소. 실데이터 앱 시작 불가 문제 예방
- Hotfix: 퇴교 번복(`reinstate_student`) 시 `process_withdrawal_makeup`으로 `makeup_expired` 전이된 결석 중 `makeup_deadline >= 현재 YYYY-MM` 항목만 `absent`로 환원 — 자연 만기 항목은 환원 대상 제외, 트랜잭션 내 원자적 처리, audit `student-reinstated.details`에 `revivedAbsenceIds` 추적
- Hotfix: 퇴교 처리 다이얼로그 AlertDialog controlled 변환 + 명시적 close — 3선택지 클릭 차단 해소. `WithdrawalMakeupDialog` z-50 → z-60 안전망. 퇴교일자 date input `onChange` blur 강제로 Tauri WebView native picker 자동 닫힘
- Hotfix: 퇴교 번복 다이얼로그 안내 문구 갱신 — "Phase 3 미래형 약속" 제거, 현재 동작(결석 환원 범위) 명시
- Hotfix: 퇴교 번복 시 `ExternalExpire`가 덮어쓴 `absence_memo` NULL 클리어

### Sprint 9 Session #12 — 4차 시각 검증 K1~K7 흡수 (2026-05-26)

`Added`:
- (K1') 그리드 응답에 `earliest_pending_absence_date: Option<String>` 추가 — 만기 미도래 미보강 결석 중 가장 이른 일자(이전 월 결석 포함). 백엔드 단위 테스트 3건 신규
- (K2/K2') 출결 관리 헤더 '재원중만' 체크박스 — 퇴교 원생 필터, 디폴트 ON
- (K3) 정규 수업 셀(present/makeup_done/makeup_expired) 우클릭 → 보강 등록 진입. 결석 셀 우클릭 메모 동작은 기존 유지
- (K4) 출결표 일자 헤더 보강데이 라벨 — 날짜 밑 작은 폰트 '보강데이' 표기 (셀 너비 변동 없음)
- (K6) '보강대상' 체크박스 — 만기 미도래 미보강 결석이 있는 원생만 필터, 디폴트 OFF
- (K7) 라벨 카운트 병기 — "재원중(N명)" / "보강대상(M명)". 보강대상 카운트는 재원중 필터 ON 시 재원중 원생 한정 (연계)

`Changed`:
- (K1') 비수업일 셀 '+' 표시 조건 정밀화 — `summary.makeupNeededMinutes > 0` → "셀 일자 이전에 만기 미도래 미보강 결석 존재". 이전 월 결석에 대한 보강 등록도 다음 월 그리드에서 진입 가능
- (K4) 단원평가 응시일 헤더 — sky 배경 제거 (일반 평일과 동일). 보강데이는 sky 배경 유지

---

## [0.4.0] - 2026-05-24

### Added
- Sprint 8: DB 마이그레이션 V106 — `regular_attendances` + `makeup_attendances` 테이블 신규 (보강필요시간, 소멸기한, 결석 사유 메모 포함)
- Sprint 8: 출결 IPC 6종 (`src-tauri/src/commands/attendances.rs` 신규) — `generate_attendances`, `check_attendance_exists`, `get_attendance_grid`, `toggle_attendance`, `update_absence_memo`, `get_attendance_summary`
- Sprint 8: 출결표 프론트엔드 (`/attendance` 라우트) — `AttendanceGrid` 컴포넌트(행=원생 × 열=일자 sticky 4컬럼 고정), `AbsenceMemoDialog`, 원생 이름 검색 필터, 요약 컬럼(출석/결석/보강필요시간)
- Sprint 8: 사이드바 "출결" 메뉴 활성화 + "보강 관리" disabled 항목 노출
- Sprint 8: `audit::SecurityEvent` + `AttendanceToggled` audit variants 추가
- Sprint 8: 보강필요시간/소멸기한 비즈니스 규칙 단위 테스트 10 시나리오 (100% 커버)

### Changed
- Sprint 8: I-S2-4 (R42) — `invalidate_credential_cache` pub 승격 + exit_hook 등록으로 앱 종료 시 Keychain 캐시 안전 폐기
- Sprint 8: I-S2-7 (R45) — `get_cached_or_load_key` + `verify_password` concurrent race 제거 (`LOAD_MUTEX` + `ensure_cache_loaded` 헬퍼 도입)
- Sprint 8: I-S2-8 (R46) — `cred_cache_lock` 헬퍼로 `Mutex` poison graceful 복구 (7곳 일괄 적용)
- Sprint 8: R39 (A28) — `create/update_study_period` overlap 검증에 `AND is_confirmed = 1` 추가 (미확정 기간과 중첩 허용)
- Sprint 8: R51 (A37) — `calendarEventClick` studyPeriodMode early return으로 교습기간 확인 모드 중 일정 클릭 시 의도치 않은 동작 차단

### Fixed
- Sprint 8: I-S2-2 (R40) — `is_salt_corrupted` partial-NULL 패턴 감지 강화 (null byte 포함 hex 스트링 처리)
- Sprint 8: I-S2-3 (R41) — `set_password` `AtomicBool` 재진입 가드 RAII panic-safe 보강
- Sprint 8: I-S2-5 (R43) — `salt_exists_at` legacy keyring fallback 검증 테스트 추가
- Sprint 8: I-S2-9 (R47) — `migrate_keyring_salt_to` `SecurityEvent` audit 기록 누락 수정
- Sprint 8: I-S2-10 (R48-a) — `device.id` 파일 권한 `0o600` 설정 (Unix)
- Sprint 8 review F2 — V107 마이그레이션 추가. `regular_attendances.makeup_attendance_id → makeup_attendances(id)` FK 제약 누락을 테이블 재생성 패턴으로 보강 (Phase 3 보강 매칭 도입 전 참조 무결성 확보)

### Security
- Sprint 8: I-S2-2~5 (R40~R43) — auth.rs Keychain 보안 4건 강화 (partial-NULL 감지, AtomicBool 재진입 가드, cache invalidation exit_hook, legacy fallback 검증)
- Sprint 8: I-S2-7 (R45) — 동시 요청 시 Keychain 직접 접근 race 조건 제거 (LOAD_MUTEX 도입)
- Sprint 8: I-S2-10 (R48-a) — device_id 파일 Unix 권한 0o600으로 제한

---

## [0.3.2] - 2026-05-23

### Fixed
- R50: `NEXT_PUBLIC_DEV_AUTOLOGIN` 환경 변수가 Next.js 빌드 타임에 클라이언트 번들에 인라인되는 보안 위험을 코드 주석 및 `.env.example`에 명시. release 빌드 전 3가지 안전 조치(제거/빈 값/unset) 안내 추가. 동일 변수를 사용하는 `LockWarning.tsx`에도 동일 경고 반영 (LockScreen.tsx, .env.example, LockWarning.tsx)
- I-S2-6: `auth.rs::load_salt_backs_up_corrupted_file` 테스트에 `#[ignore]` 가드 추가. 해당 테스트가 dev 환경의 실제 OS Keychain 항목을 읽고 삭제할 수 있어 개발자 SmartHB salt 손상 위험 방지. 수동 실행: `cargo test -- --ignored`

---

## [0.3.1] - 2026-05-23

### Added
- Sprint 7: `CredentialCache` 구조체 도입 (`OnceLock<Mutex<Option<CachedCredentials>>>`, ZeroizeOnDrop) — 앱 시작 시 1회 Keychain 로드 후 캐시 경유, macOS Keychain 다이얼로그 반복(3+ 회→최대 1회) 해소 (Issue 1, Critical UX)
- Sprint 7: salt.bin 클라우드 동기화 폴더 이전 (`smarthb/salt.bin`) — Keychain 의존도 감소, 양 PC 동일 salt 자동 동기화 보장 (A17/A27 3회 이월 최종 해소)
- Sprint 7: device_id 영속화 — `app_config_dir/device_id` 파일로 재시작 간 UUID 유지. stale lock 디바이스 식별 정확도 향상 (Issue 8, PRD §5.3)
- Sprint 7: `ScheduleCodeSelector` 컴팩트 컴포넌트 신규 (`src/components/academic/ScheduleCodeSelector.tsx`) — `/academic` 캘린더에서 코드 패널 제거 후 셀 배치 시 인라인 코드 선택 UX 제공
- Sprint 7: `/settings/schedule-codes` 라우트 신설 — 학사 일정 코드 관리 화면을 설정 메뉴 하위로 이동 (Issue 3)
- Sprint 7: 보안 패치 6건 (S-T2-1~6) — eprintln 키 누출 제거, set_password 원자성 보강, recovery 원자성 보강, NTFS power-loss fsync 강화, delete_key NoEntry idempotent, PC-B UX 개선
- Sprint 7 post-review: 확대 보기 모드 — 월별 캘린더 단독 확대 표시 (V15), prev/next 비활성 (V31), 창 가변 확장 (V33)
- Sprint 7 post-review: `tauri-plugin-window-state` 도입 — 윈도우 크기·위치 자동 저장·복원 (V18)
- Sprint 7 post-review: `schedule_events.is_seeded` 컬럼 추가 (V302 마이그레이션) — 시드 공휴일 vs 사용자 추가 공휴일 구분, 시드 공휴일만 삭제 차단 (V16/V21)
- Sprint 7 post-review: 비밀번호 입력 모드 배지 — 마지막 입력 문자 종류(한글/영문/숫자/특수) 실시간 표시 (V37b)
- Sprint 7 post-review: dev 빌드 자동 로그인 우회 (`NEXT_PUBLIC_DEV_AUTOLOGIN`) — 시각 검증 효율화 (V30)

### Changed
- Sprint 7: 교습기간 설정 UX 재설계 — 토글 버튼 제거, 셀 클릭 즉시 selection 모드 자동 진입 (Issue 5, PRD §4.4.2)
- Sprint 7: `schedule_events` 배치 제약 강화 — 교습기간 내 일자에만 배치 허용 + 중복불가 코드(`is_duplicate_blocked`) 간 동일 일자 중복 배치 상호 차단 (Issue 4, R34)
- Sprint 7: `ScheduleEventListItem` 응답에 `is_system_reserved` 플래그 추가 (백엔드 JOIN 확장) — 프론트엔드 `codeBadgeClass`/`draggableEventIds` 하드코딩 제거 (A23/R33)
- Sprint 7: `academic.rs` `delete_schedule_event` — 공휴일 이벤트(`is_holiday=true`) 삭제 차단 추가 (Issue 7)
- Sprint 7: 교습기간 삭제(`delete_study_period`) 시 cascade — 해당 기간 내 공휴일을 제외한 학사 일정 일괄 삭제 (Issue 6)
- Sprint 7: 사이드바 종료 메뉴 위치 최종 확정 — 설정 메뉴 다음, 메뉴 리스트 내 배치 + TopBar h-16 정렬 보정 (Sprint 6 후속 보강 3건)
- Sprint 7 post-review: 학사 컨트롤 바 통합 — 교습기간 + 코드 selector를 단일 컨트롤 바로 (V11), 외곽 박스 제거, 코드명만 chip 표기 (V10)
- Sprint 7 post-review: `ScheduleCodeSelector` — 시스템 예약 코드 포함 활성 전체 코드 노출 (V6)
- Sprint 7 post-review: 교습기간 셀 배경 강화 — 수업 가능(amber-100)/불가(gray-100) 색상 구분 (V22/V23), 다른 월 교습기간 블러 (V32/V35)
- Sprint 7 post-review: 기간성 코드 캘린더 표시 — 시작/종료 마커(S/E), 중간 날짜 배지 연속 표시 (V13/V20)
- Sprint 7 post-review: TopBar 시작 속도 텍스트를 "정상속도"/"속도저하" 레이블로 변경 (V34)
- Sprint 7 post-review: 비밀번호 입력 보기/숨김 버튼 텍스트화 ("보기"/"숨김") (V36)
- Sprint 7 post-review: `exit_hook` idempotent 가드 — 윈도우 닫기·앱 종료 이중 이벤트 시 1회만 실행 (V24)
- Sprint 7 post-review: 글로벌 단축키 훅(`use-keyboard-shortcuts`) 제거 (V19) — 혼동 유발 단축키 비활성화, 사이드바 shortcut 표기 제거

### Fixed
- Sprint 7: Issue 1 — macOS 앱 시작 시 Keychain 비밀번호 다이얼로그가 3회 이상 반복 표시되어 startup 31초 소요되던 Critical UX 이슈 해소
- Sprint 7: Issue 3 — 학사 일정 코드 관리가 `/academic` 화면 하단에 노출되어 UX 혼란 야기, `/settings/schedule-codes`로 분리
- Sprint 7: Issue 4/R34 — 교습기간 외 일자에 학사 일정 배치 가능하던 가드 부재 문제 해소
- Sprint 7: Issue 5 — 교습기간 선택 진입을 위한 토글 버튼을 찾기 어렵던 문제, 셀 클릭으로 자동 진입
- Sprint 7: Issue 6 — 교습기간 삭제 버튼 부재 + 삭제 시 학사 일정이 고아 데이터로 잔류하던 문제 해소
- Sprint 7: Issue 7 — 확정 교습기간 내 법정 공휴일이 삭제 가능하던 보안 부재 문제 해소
- Sprint 7: Issue 8 — `lock.rs` device_id가 매 프로세스 재생성되어 stale lock이 항상 "다른 디바이스" 로 오식별되던 문제 해소
- Sprint 7: A23/R33 — `codeBadgeClass`에 시스템 코드 ID 하드코딩으로 코드 추가 시 배지가 무채색 표시되던 문제 해소
- Sprint 7 post-review: V1 — 교습기간 `year_month`가 시작일 기준 월로 저장되어 cross-month 교습기간의 월 분류 오류 (시작일 5/29 → 6월 교습기간이 5월로 저장됨) 수정
- Sprint 7 post-review: V7 — 교습기간이 월 경계를 넘어가는 경우 이전/이후 그리드에서 in-study 셀이 표시되지 않던 문제 수정 (allStudyPeriods 전달로 cross-month 처리)
- Sprint 7 post-review: V9 — 공휴수업일 배치 가드 정상화 (공휴일 없는 날에 배치 차단, 공휴수업일+공휴일 외 조합 차단)
- Sprint 7 post-review: V12 — 교습기간 selection 시 다른 교습월의 기간 일자 포함 차단 (프론트엔드 가드 추가)
- Sprint 7 post-review: V14 — 단원평가 응시일 셀 상단 색 라인 제거, 일반 배지로 통일
- Sprint 7 post-review: V18 — 앱 종료 후 재시작 시 윈도우 크기·위치가 초기화되던 문제 (tauri-plugin-window-state)
- Sprint 7 post-review: V20 — 기간성 코드(방학 등)가 시작일 셀에만 배지 표시되고 중간/종료일 셀에서 보이지 않던 문제
- Sprint 7 post-review: V26 — 기간성 코드 배치 시 범위 겹침 충돌 검사 미적용으로 사이 일자 중복 가드 누락 수정
- Sprint 7 post-review: V29 — 보강데이를 운영일(수업 있는 날)에 배치 가능하던 가드 부재 문제 수정
- Sprint 7 post-review: V32/V35 — 다른 월 교습기간 셀이 현재 월 교습기간과 시각 구분이 안 되던 문제 수정 (블러 강화)
- Sprint 7 post-review: V37 — 한글 IME 활성 상태에서 비밀번호 입력 시 한글 자모가 비밀번호로 입력되던 UX 문제 수정

### Security
- Sprint 7: S-T2-1 — `eprintln!` 으로 DB 암호화 키 hex가 콘솔에 노출되던 문제 제거
- Sprint 7: S-T2-2 ~ S-T2-6 — set_password/recovery 원자성 보강, NTFS fsync, delete_key idempotent, PC-B UX 개선

### Added
- Sprint 6: Phase 2 학사 스케줄 관리 첫 기능 진입 — `/academic` 라우트 신설, 사이드바 "학사 스케줄" 메뉴 활성화
- Sprint 6: 3개월 학사 캘린더 컴포넌트 — Tailwind grid-cols-7 직접 구현 (shadcn/ui Calendar 미사용), 공휴일/교습기간/일정 배지 표시, 교습기간 셀 selection 통합
- Sprint 6: 교습기간 설정 UI (PRD §4.4.2) — 시작일/종료일 셀 클릭 → StudyPeriodEditor, 교습기간 확정/해제, 지난 달 읽기 전용 (AC-4.4-1)
- Sprint 6: 학사 일정 코드 + 배치 UI (PRD §4.4.3~4.4.6) — 시스템 예약 5종 활성 토글, 사용자 코드 CRUD, 날짜 셀 클릭 등록, @dnd-kit 드래그 이동 (단일 일자, 시스템 코드 제외)
- Sprint 6: 단원평가 응시일 자동 배치 (PRD §4.4.7) — 2주차/4주차 월~금 자동 배치 IPC (`auto_place_assessment_dates`)
- Sprint 6: 백엔드 IPC 15개 (`src-tauri/src/commands/academic.rs` 신규) — study_periods 6종 + schedule_codes 4종 + schedule_events 5종
- Sprint 6: TypeScript IPC 래퍼 15개 + 도메인 타입 10개 (`src/types/academic.ts`)
- Sprint 6: V301 마이그레이션 — schedule_codes 시드 3속성 보정 (보강데이/공휴수업일/단원평가 속성, PRD §4.4.4 정합) + 한국 법정 공휴일 2025~2027 64건
- Sprint 6: `pnpm holidays:fetch` 빌드 스크립트 (`scripts/fetch-holidays.ts`, tsx 기반) — 공공데이터포털 API 호출 + V301 SQL 생성 자동화
- Sprint 6: ADR-005 (`docs/arch/adr-005-holiday-api-selection.md`) — 공휴일 API 소스/저장 위치/갱신 주기 결정 (매년 1월 V401+ 마이그레이션)
- Sprint 6: 신규 devDependency — `tsx 4.22` (빌드 타임 TypeScript 실행)
- Sprint 6: 신규 환경변수 — `KOREA_HOLIDAY_API_KEY` (`.env.example` 추가)
- Sprint 6: A20 해소 — lock/page.tsx 에러 화면 재시도 버튼 + lockStatus 초기화
- Sprint 6: A21 해소 — paths.rs OnceLock 병렬 테스트 격리 (테스트 146건 안정화)
- Sprint 6: A22 해소 — 코드 DnD 필터링 sort_order 충돌 (방법 B 적용)

### Changed
- Sprint 6: V301 — V102 schedule_codes 시드 3속성 보정 (보강데이 `is_duplicate_blocked` false → true 외 PRD §4.4.4 정합 2건)
- Sprint 6: `.claude/hooks/posttooluse-code-validator.sh` — `.env` 차단 정규식 좁힘 (`.env.example` 허용, `.env.local`/`.env.*.local` 패턴으로 실제 시크릿 파일만 차단)

### Fixed
- Sprint 6: A20 — lock/page.tsx 에러 화면에서 재시도 버튼 누락으로 앱 재시작 없이 락 재점유가 불가능하던 문제 해소
- Sprint 6: A21 — paths.rs OnceLock 테스트 격리 부족으로 병렬 실행 시 flaky 발생하던 문제 해소
- Sprint 6: A22 — 코드 DnD 드래그 후 필터 변경 시 sort_order가 충돌하여 순서가 뒤섞이던 문제 해소

---

## [0.2.1] - 2026-05-21

### Added
- Sprint 5: `tauri-plugin-single-instance` 2.4.2 도입 — 동일 PC 다중 인스턴스 원천 차단. 두 번째 인스턴스 기동 시 기존 창 포커스 + 새 프로세스 즉시 종료 (PRD §5.3)
- Sprint 5: `cross-env` devDependency 추가 — `pnpm dev` 스크립트에 `NODE_OPTIONS=--no-experimental-webstorage` 적용 (Node 25/20 cross-OS 호환)
- Sprint 5: V201 마이그레이션 (`201__update_seed_data.sql`) — 표준교습비 시드 (3/4/5/6h: 16만/20만/23만/26만원) + 결제수단 시드 (현금 비활성 + 계좌이체/카드/결제선생/성남사랑 활성 5종) 운영 값으로 보정. 멱등성 보장 (V001/V104 baseline 일치 행만 변경, 사용자 수정 데이터 보존)

### Changed
- Sprint 5: LockPage 진입 시 락 상태 사전 체크 로직 추가 — stale 락(5분 미갱신) 자동 점유 후 LockWarning 라우팅 활성화. 이전에는 LockWarning 화면으로 진입하지 않던 문제 해소
- Sprint 5: 마법사 완료 redirect 경로 수정 — `/` → `/settings` (마법사 완료 후 교습소 설정 화면으로 직행)

### Fixed
- Sprint 5: Node 25 환경에서 Next.js Dev Overlay의 `localStorage.getItem` 호출이 SSR에서 실패하여 `/` 페이지 500 에러 발생하던 이슈 해소 (`--no-experimental-webstorage` 플래그)
- Sprint 5: 동일 PC 다중 인스턴스 기동 시 두 번째 인스턴스가 외부 디바이스로 오인 → "다른 PC 사용 중" 오표시 + 잠금해제 무반응 이슈 해소 (single-instance 플러그인)

### Added
- Sprint 4: 교습소 설정 메뉴 화면 신설 (PRD §4.12) — 운영 시간(요일별 시작/종료/수업 길이) 편집, `save_operating_hours`/`get_operating_hours` IPC
- Sprint 4: 수업 스케줄 시작시간 콤보박스 + 수정/삭제 기능 — 운영 시간 내 1시간 단위 선택, 운영시간 디폴트 자동 적용, 스케줄 카드 수정/삭제 UI
- Sprint 4: 코드 테이블 DnD 순서 변경 (`@dnd-kit/core`, `@dnd-kit/sortable`, `@dnd-kit/utilities`) + 활성 상태 필터 + 신규 항목 sort_order 자동 부여
- Sprint 4: 원생 목록 화면 — 주총 수업시간 + 수업 요일 컬럼 추가
- Sprint 4: 원생 등록/수정 폼 — 학교명 Select 연동(학교 코드 테이블), 연락처 자동 하이픈(`formatPhone`), 금액 천단위 콤마(`formatCurrency`), 일련번호(`serial_no`) readonly 보호, 퇴교일 필드 + 퇴교 번복 기능
- Sprint 4: 원생 등록 완료 후 수업 스케줄 등록 안내 UX (등록 직후 알림 + 스케줄 편집 페이지 이동 버튼)
- Sprint 4: `format.ts` 유틸 신규 (`src/lib/format.ts`) — `formatPhone`, `formatCurrency` 2종
- Sprint 4: `reinstate_student` IPC 커맨드 신규 — 퇴교 번복 기능 백엔드
- Sprint 4: shadcn/ui AlertDialog 컴포넌트 도입 — `window.confirm`/`window.alert` 전면 교체 (Tauri 2 CSP 차단 해소)
- Sprint 4: 단위 테스트 130건 (Sprint 3 109건 → +7건, post-sprint3 23건 포함 기준 +7)
- Sprint 4: 신규 의존성 — `@base-ui/react`, `class-variance-authority`, `clsx`, `lucide-react`, `tailwind-merge`, `tw-animate-css` (shadcn/ui init), `@dnd-kit/core`, `@dnd-kit/sortable`, `@dnd-kit/utilities`

### Changed
- Sprint 4: 상태바 — 점유/백업/동기화/시작시간 IPC 실연결 (AppShell 에서 `checkLockStatus`/`listBackups`/`checkSyncStatus` 60초 polling 호출, 시작시간은 `useSessionStore.lastStartup.elapsed_ms` 사용)
- Sprint 4: 수업 스케줄 편집 UI — 추가/변경 폼을 등록된 스케줄 그리드 **위**로 이동 (`ScheduleEditor` 내 영역 재배치), 1회 수업 시간 select 옵션을 1시간 단위(1/2/3/4)로 제한
- Sprint 4: 원생 목록 디폴트 정렬 — 번호순(`StudentSort::SerialAsc`) 으로 변경. 컬럼 헤더 클릭으로 번호/이름/학년/입교일 asc↔desc 토글

### Fixed
- Sprint 4: `window.confirm`/`window.alert` 차단 — Tauri 2 `dialog:allow-confirm` 미허용으로 퇴교 확인 다이얼로그가 작동하지 않던 Critical Runtime Error 해소 (shadcn AlertDialog로 교체)
- Sprint 4: 상태바 IPC 미연결 — 점유/백업/동기화/시작시간 표시가 항상 초기값으로 표시되던 이슈 해소
- Sprint 4: 퇴교일 필드 미표시 — 원생 등록/수정 폼에서 `withdraw_date`를 입력할 수 없던 이슈 해소
- Sprint 4: 일련번호 수정 허용 — `serial_no` 필드가 편집 가능하여 PI-05 자동 채번 정합성 위험. 프론트 readonly + 백엔드 `update_student` SQL 에서 `serial_no` 컬럼 제외 (defense in depth)
- Sprint 4: 학교명 텍스트 자유입력 — 코드 테이블과 연동 없이 자유입력만 가능하던 이슈 해소 (Select 컴포넌트로 교체)
- Sprint 4: 스케줄 시작시간 자유입력 — 운영 시간 범위 외 시간 입력이 가능하던 이슈 해소 (콤보박스 + 운영 시간 검증)
- Sprint 4: 코드 테이블 정렬 변경 불가 — sort_order 변경 UX가 없어 순서를 조정할 수 없던 이슈 해소 (DnD)

### Security
- Sprint 4: Next.js 15.3.2 CVE-2025-66478 — 현재 미적용, release 전 업그레이드 필수 (Sprint 5 또는 별도 hotfix)

### Added
- Sprint 3: Pretendard 폰트 self-host — `public/fonts/` woff2 배치, `@font-face` + Tailwind config 설정. 본문 18px, 헤더 24px+, 행간 1.5 기본값 확립 (ADR-006)
- Sprint 3: 앱 레이아웃 셸 — 사이드바(메뉴 9종 + 단축키 병기 + 비활성 툴팁), 상단 상태바(점유 디바이스/마지막 백업/동기화 상태), AppShell 조합 컴포넌트
- Sprint 3: 글로벌 검색바 (PRD §4.14) — 원생 이름(우선)/학교명/메뉴명 검색, 한글 자모 부분 일치, 200ms 디바운싱, 1클릭 이동, Ctrl+F 단축키
- Sprint 3: Zustand 스토어 2종 (`src/stores/session-store.ts`, `src/stores/app-store.ts`) — 세션 상태/락 점유/사이드바 상태/선택 교습기간월
- Sprint 3: TanStack Query Provider — IPC 응답 캐싱/무효화 패턴 확립 (`src/providers/query-provider.tsx`)
- Sprint 3: `tauri-plugin-dialog` 통합 — 폴더 선택 네이티브 다이얼로그 IPC + `capabilities/default.json` `dialog:default` 권한
- Sprint 3: 초기 설정 마법사 백엔드 (`src-tauri/src/commands/setup.rs`) — `save_cloud_folder`, `complete_setup`, `get_setup_status` IPC 3종 + `app_config_dir/config.json` 설정 분리 저장 (chicken-and-egg step-back 반영)
- Sprint 3: 초기 설정 마법사 프론트엔드 (`src/app/setup/page.tsx`) — 4단계(환영/클라우드 폴더 선택/비밀번호 설정/완료) + 단계별 독립 저장 + 뒤로가기 지원
- Sprint 3: 원생 목록 화면 (`src/app/students/page.tsx`) — TanStack Query 캐싱, 필터 7종(이름/학교급/학년/학교명/요일/성별/재원상태) + 정렬 3종 + 페이지네이션
- Sprint 3: 원생 등록/수정 폼 — `create_student`/`update_student`/`withdraw_student` IPC 연동, 3분 자동 임시저장(localStorage), 미저장 경고 다이얼로그, 퇴교 처리 확인 다이얼로그
- Sprint 3: 코드 테이블 관리 화면 (PRD §4.12) — 학교/표준교습비/결제수단/카드사 탭 CRUD, is_active 소프트 삭제, sort_order 변경
- Sprint 3: 수업 스케줄 편집 UI (PRD §4.2) — 요일별 시작 시간/수업 시간 입력, 운영 시간 내 제한, 주 총 수업시간 실시간 표시, 표준 교습비 자동 매칭 표시
- Sprint 3: 키보드 단축키 체계 (`src/hooks/use-keyboard-shortcuts.ts`) — F1/Ctrl+F/Ctrl+N/Ctrl+S/Ctrl+Z/ESC/Ctrl+P 7종 바인딩
- Sprint 3: `count_students(filter)` IPC 신규 — 페이지네이션 총 건수 반환
- Sprint 3: 단위 테스트 109건 (Sprint 2 97건 → +12건)

### Changed
- Sprint 3: `src/app/page.tsx` 라우팅 분기 업데이트 — `not-initialized` 상태 시 `/setup` 마법사로 이동
- Sprint 3: `StudentFilter` 구조체에 `limit: Option<u32>`, `offset: Option<u32>` 추가 — 기본 limit=50, 상한 1000
- Sprint 3: `list_students`/`list_codes` SQL에 `LIMIT ? OFFSET ?` 적용

### Fixed
- Sprint 3: R13 PII 마스킹 — `students.rs` `try_record` 3곳 `details=None` 적용하여 감사 로그에 원생 이름 미포함
- post-sprint3: `config.json` 손상 자동 복구 (`setup.rs`) — PC 강제 종료로 인한 NTFS power-loss 시 발생하는 NULL-바이트 파일/파싱 실패를 감지하여 `config.json.corrupted-{ts}` 로 백업 후 기본값 fallback. 사용자는 마법사를 다시 진행하면 자동 복구됨. 단위 테스트 6건 추가 (총 115건)
- post-sprint3: `app.lock` 손상 자동 복구 (`lock.rs`) — 동일한 NTFS power-loss 패턴이 락 파일에도 발생. `String::trim()` 이 NULL 을 공백으로 인식하지 않아 파싱 실패가 `AppError::Lock` 으로 wrap 되어 사용자에게 "다른 컴퓨터에서 사용 중" 으로 잘못 표시되던 회귀 해소. `parse_lock_info` 가 손상 감지 시 `Ok(None)` 반환 → `acquire_lock_atomic` 이 새 락 즉시 작성. 단위 테스트 5건 추가 (총 123건)
- post-sprint3: keyring v3 OS native backend 활성화 (`Cargo.toml`) — `keyring = "3"` default-features 만 켜진 상태에서 backend 미연결로 `set_password` 가 silent OK 반환 후 `get_password` 가 항상 `NoEntry` 반환하던 critical 회귀 해소. `features = ["apple-native", "windows-native"]` 명시. 마법사 비밀번호 설정 흐름 전체가 차단되던 증상 해결
- post-sprint3: stale 락 자동 점유 (`lock.rs`) — 이전 세션 비정상 종료로 잔존한 stale 락(5분 미갱신)을 `force` 옵션과 무관하게 자동 정리. PRD §5.3 "강제 점유 옵션" 은 fresh 락에만 적용 (단일 사용자 UX). 마법사 LockScreen 의 `force=false` 하드코딩이 stale 락에 막히던 회귀 해소

### Added
- post-sprint3: 에러 진단 인프라 (`error.rs`, `auth.rs`) — `From<AppError> for String` 변환 시점에 raw `Display` 메시지를 stderr 에 `[error] ...` 로 보존 (PRD §6.4 "사용자 화면엔 친화 메시지, 콘솔엔 기술 상세" 정책 준수). `set_password` / `verify_password` 단계별 진단 로그 (hex 첫 8byte 만 — 키 노출 방지). tracing crate 통합 시 `tracing::error!` 로 교체 예정

---

## [0.2.1] - 2026-05-22

### Security
- **CVE-2025-66478**: Next.js 15.3.2 → 15.3.6 업그레이드 (App Router + React Server Components RCE 취약점 패치). 본 앱은 `output: 'export'` 정적 빌드로 실 익스플로잇 노출은 없으나 예방적 차원으로 적용. `eslint-config-next`도 동반 업그레이드.

---

## [0.2.0] - 2026-05-20

### Added
- Sprint 2: 루트 라우팅 + 인증 게이트 — `src/app/page.tsx` 클라이언트 가드, `lock/page.tsx` onUnlocked → `app_startup_sequence` → 메인 redirect, `src/lib/auth-state.ts` 모듈 스코프 인증 상태 (Sprint 3 Zustand 도입 전)
- Sprint 2: DB 마이그레이션 V101 — students + student_schedules 테이블 (PI-05 자동 채번: `MAX+1` + `BEGIN IMMEDIATE` + override 허용)
- Sprint 2: DB 마이그레이션 V102 — study_periods + schedule_codes 테이블 (시스템 예약 코드 5종 시드 데이터 포함)
- Sprint 2: DB 마이그레이션 V103 — schedule_events 테이블
- Sprint 2: DB 마이그레이션 V104 — standard_fees 재설계 (weekly_minutes 기준 교습비 자동 매칭)
- Sprint 2: DB 마이그레이션 V105 — schools 보강 (school_type / region 컬럼)
- Sprint 2: 원생 CRUD IPC 커맨드 4종 (`create_student`, `update_student`, `list_students`, `get_student`) — 이름/학교급/학년/학교명/요일/성별 다중 필터, 이름순/입교일순/학년순 정렬, 재원 상태 필터
- Sprint 2: 수업 스케줄 IPC 커맨드 3종 (`set_schedule`, `get_schedules`, `get_weekly_hours`) — (원생, 요일) UNIQUE 검증, 변경 이력 자동 생성, 주 총 수업시간 자동 산정
- Sprint 2: 표준 교습비 IPC 커맨드 3종 (`list_fees`, `create_fee`, `update_fee`) + 주 수업시간 → 교습비 자동 매칭 함수
- Sprint 2: 코드 테이블 CRUD IPC 커맨드 — 학교/결제수단/카드사 공용 CRUD (is_active 소프트 삭제, sort_order 변경)
- Sprint 2: 도메인 타입 4종 — `src/types/{student,schedule,fee,code}.ts`
- Sprint 2: 프론트엔드 IPC 래퍼 18개 추가 (`src/lib/tauri/index.ts`) — dev mode fallback 포함, 총 22 신규 IPC
- Sprint 2: `AppError::UserFacing(String)` variant 신규 — 도메인 검증 메시지 사용자 친화적 노출
- Sprint 2: `AuditEventType` 확장 — StudentCreated / StudentUpdated / StudentWithdrawn 추가
- Sprint 2: 단위 테스트 97건 (Sprint 1 64건 → +33건)

### Changed
- Sprint 2: R6 salt 이전 이연 확정 — `{data_root}/salt.bin` 평문 파일 보관, Sprint 3 마법사 통합 시점에 Keychain 이전 (R12 신설 추적)
- Sprint 2: T8 `query!()`/`query_as!()` 매크로 전환 이연 — 동적 `query() + bind()` 패턴 유지, 별도 backlog 추가

> **이연 사유**: salt 이전은 마법사 DB 경로 설정과 coupled, `query!()` 매크로는 V101~V105 스키마 안정화 후 일괄 전환이 안전

### Fixed
- R15: `startup::exit_hook`에서 `release_lock_atomic()` 직접 호출로 교체 (6c85f5c)

---

## [0.1.0] - 2026-05-19

### Added
- Sprint 1: SQLCipher AES-256 암호화 DB 통합 (`libsqlite3-sys bundled-sqlcipher-vendored-openssl`, `cipher` feature flag로 개발/프로덕션 분리) — ADR-001
- Sprint 1: OS Keychain/Credential Manager 통합 (`keyring` crate) + PBKDF2 600K iter 키 유도 + `zeroize` 메모리 폐기 — ADR-004
- Sprint 1: PI-07 복구 코드 — Argon2id 해시, 12자리 31자 알파벳 포맷
- Sprint 1: 인증 IPC 커맨드 (`set_password`, `unlock_db`, `check_auth_status`) + 잠금 화면 UI (Pretendard 18pt, 44×44px 버튼)
- Sprint 1: `app.lock` 동시성 제어 — `fs2` advisory locking + 60초 heartbeat + 5분 강제 해제 — ADR-002
- Sprint 1: 4계층 자동 백업 — exit(10) / hourly(24) / daily(30) / weekly(4), SQLite Online Backup API, 암호화 상태 그대로 보관 — ADR-003
- Sprint 1: 무결성 검증 — 앱 시작 시 `PRAGMA quick_check / integrity_check`, 손상 감지 시 자동 복원 + `restore_rollback` 안전망 + 손상본 격리
- Sprint 1: 동기화 대기 로직 — DB/락 파일 최신 동기화 확인
- Sprint 1: 감사 로그 (`audit_logs` 테이블) + 주요 커맨드 7곳 통합
- Sprint 1: DB 마이그레이션 V001 (코드성 테이블: schools, payment_methods, card_companies, standard_fees) + V008 (app_settings, audit_logs)
- Sprint 1: 앱 시작 시퀀스 — `tokio::join!` 락+무결성 병렬 실행, PRD §5.6 < 3초 목표 구현
- Sprint 1: `commands/paths.rs` / `commands/runtime.rs` / `app_err!` 매크로 공통 헬퍼 모듈 분리
- Sprint 1: `thiserror` 기반 `AppError` 7종 변형 (`Auth`, `Db`, `Lock`, `Backup`, `Integrity`, `Io`, `Config`) — `src-tauri/src/error.rs`
- Sprint 1: 단위 테스트 74건 (`cargo test` 기준)
- Sprint 1: CI 매트릭스 — `ci.yml` + `deploy.yml` cipher feature on/off 양 OS 빌드 (Windows Strawberry Perl 포함)
- Sprint 1: ADR-001/002/003/004 문서 4건 (`docs/arch/`)

---

## [0.0.1] — 프로젝트 초기 템플릿 (보일러플레이트 + 계획 산출물)

### Added
- 프로젝트 초기 템플릿 설정
- Claude Code 에이전트 정의 (sprint-planner, sprint-close, sprint-review, hotfix-close, deploy-prod, phase-planner, prd-to-roadmap)
- CI/CD 파이프라인 (GitHub Actions — ci.yml + deploy.yml)
- 개발 프로세스 문서 (`docs/dev-process.md`)
- CI/CD 정책 문서 (`docs/ci-policy.md`)
- 전략 지침 문서 (`strategy/`)
- 하네스 엔지니어링 정책 5종 (`docs/harness-engineering/`)
- PRD.md v1.5 (MVP) — 분기 학습보고서 도메인 재설계 (작성 주기 월 1회 → 분기 1회, 키 `(분기, 원생)`, 단일 컬럼 `종합의견`, 단원평가 점수에 종속)
- PRD.md v1.4 (MVP) — 5건 Post-MVP 승격 통합 (초기 설정 마법사 §4.0, 글로벌 검색 §4.14, 청구 마감 워크플로우 §4.9.7, 데이터 자가 진단 §6.6, 키보드 단축키 §5.7)
- PRD 정합화 산출물: `docs/prd-issues.md` (논리 오류 11건), `docs/data-model.md` (도메인 → SQLite 스키마 1차 매핑)
- Tauri shell plugin (`tauri-plugin-shell`) — 외부 프로세스 실행 및 OS 기본 앱으로 파일/URL 열기
- `.gitattributes` — 셸 스크립트 LF 정규화 (macOS/Windows 양 OS 보장)
- `docs/setup-guide.md` Tauri 아이콘 생성 절차 (5-A 섹션)

### Changed
- 데이터 저장 모델 — Supabase에서 **로컬 SQLite + SQLCipher AES-256 + 클라우드 동기화 폴더(MYBOX 우선)** 로 전환 (PRD v1.1)
- 기술 스택 — FastAPI/Docker에서 **Tauri 2 + Next.js 15 + React 19** 로 전환 (커밋 f2fbb7c)
- 동시성 모델 — 양 PC 시점 분리 사용 + `app.lock` heartbeat 60s, 5분 미갱신 강제해제 (PRD §5.3)
- 백업 정책 — 4계층 자동 백업(exit/hourly/daily/weekly) + SQLite Online Backup API (PRD §5.4)
- 백업 복원 리허설 — 정기 수행 모드에서 **필요시 수행 모드**로 단순화 (PRD v1.4)
- 청구 데이터 상태 — 2단계(미확정/확정)에서 **3단계(미확정/확정/마감)** 로 확장 (PRD v1.4 §4.9.7)
- 학습보고서 E2E 도구 — Playwright에서 `Tauri WebDriver(tauri-driver)` 로 통일 (PRD §6.5)
- 학습보고서 출력 — `§4.8.4`의 "파일 저장 없음" 제거하여 인쇄 + PDF 저장 양쪽 허용 (`§4.13.2` 와 정합)
- 클라우드 동기화 폴더명 — `smarthm/` → **`smarthb/`** 로 통일 (프로젝트명과 일치)
- 배포 모델 — GitHub Releases 인스톨러(Windows `.msi`/`.exe`, macOS `.dmg`) (PRD)
- AI 협업 가이드 정합 — `CLAUDE.md`, `.claude/rules/backend.md`, `.claude/rules/frontend.md`, `ARCHITECTURE.md`에 SQLCipher / 락 / 무결성 / Pretendard / Zustand / TanStack Query / FullCalendar / 글로벌 검색바 / 분기 학습보고서 / 청구 마감 정책 반영
- 단일 사용자 모델 — CV 문서의 "팀 채널 모니터링" → "원장 직접 체감"으로 정합

### Fixed
- 셸 스크립트 실행권한 비트 부여 (`SETUP.sh`, `scripts/hooks/pre-commit`, `.claude/hooks/*.sh`) — macOS clone 시 `Permission denied` 방지
- `scripts/hooks/pre-commit` 옛 경로(`app/frontend/`) 제거 및 `scripts/pre-commit-lint.sh` 위임 wrapper로 단순화
- `.claude/hooks/pretooluse-bash-guard.sh` — python3 미설치 환경에서 jq 폴백 추가, 둘 다 없으면 안전을 위해 차단
- `SETUP.sh` macOS Xcode CLI 미설치 시 `exit 1` 로 강제 차단

---

## 참고

- 로드맵 연계: `ROADMAP.md` (Phase/Sprint 상태와 버전 연결)
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
