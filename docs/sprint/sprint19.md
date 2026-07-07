# Sprint Plan sprint19

## 기간
2026-07-07 ~ 2026-07-20 (2주)

## 목표
실사용 원장님의 UI/UX 피드백 8건 + 추가 요구 2건(학년 자동 승급, 학교급 기반 필터링)을 반영하여 그리드 정렬 통일, 스크롤 접근성 개선, 교습일정 인쇄 캘린더 고도화, 수업관리 캘린더 표시 규칙 정비, 대시보드 레이아웃 최적화, 연초 학년 자동 승급, 학교급 기반 학교 선택 필터링을 달성한다.

## ROADMAP 연계 기능
- Post-v1.1 안정화 (사용자 피드백 반영 + 운영 편의 기능)
- 신규 의존성: 없음
- DB 마이그레이션: V310 (schools.school_type 텍스트 기반 자동 보정) 1건

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint18-retrospective.md`

| 항목 ID | 항목 | 반영 방법 |
|---------|------|-----------|
| A113 | 백엔드-프론트엔드 상수 쌍 목록화 (STALE_THRESHOLD_SECONDS 등) | T0에서 CLAUDE.md 또는 scope.md에 상수 쌍 문서화 |
| A116 | AcademicSchedulePrint 동적 행 수 계산 (5행 달 빈 행 제거) | T4 교습일정 인쇄 개선에 흡수 |
| A114 | sync_single_date 이력 패턴 통일 | Low, Post-MVP 유지 (이번 스프린트 범위 외) |
| A115 | cipher 스모크 테스트 수행 | 배포 후 수동 검증 항목, 코드 변경 불필요 (이번 스프린트 범위 외) |

## 작업 목록

### T0: Sprint 18 회고 액션 반영 (A113, A116 선행) — 1.5h

- ✅ A113 — 백엔드-프론트엔드 상수 쌍 목록화. `STALE_THRESHOLD_SECONDS`(lock.rs↔LockWarning.tsx) + `MEMO_DEFAULT_HEIGHT`(dashboard.rs↔DashboardView.tsx) 2쌍을 `.claude/rules/harness-engineering.md`에 문서화 (3ff4c68)
- ✅ A116 — `AcademicSchedulePrint.tsx` 6행 고정 → `Math.ceil(cells.length / 7)` 동적 행 수 계산 (3ff4c68)

**관련 파일**: `src/components/academic/AcademicSchedulePrint.tsx`

### T1: 공통 정렬 인프라 + 원생 목록 정렬 통일 — 3h

사용자 요구 1, 2번의 기반. 모든 그리드에 재사용할 정렬 로직을 구축하고 원생 목록에 먼저 적용한다.

- ✅ 공통 정렬 훅/유틸리티 구현 (`src/hooks/useTableSort.ts` 신규) — 출결/청구(T2/T3)용, `withTiebreak`로 "동일 값 정렬 시 이름 2차 정렬" 일반화 (bfc8725)
- ✅ 원생 목록(`src/app/students/page.tsx`) 기본 정렬을 학년별+이름 가나다순(GradeAsc)으로 변경. 성별/수업시간 헤더 클릭 정렬 신규 추가. 학교급은 학년과 동일 기준이라 별도 버튼 없이 유지(중복 하이라이트 UX 방지) (bfc8725)
- ✅ 백엔드 `students.rs` StudentSort에 GenderAsc/Desc, WeeklyHoursAsc/Desc 추가, 디폴트를 GradeAsc로 변경, 단위 테스트 3건 추가 (bfc8725)

**관련 파일**:
- `src/hooks/useTableSort.ts` (신규)
- `src/app/students/page.tsx` (라인 60-77, 259-300)
- `src-tauri/src/commands/students.rs` (ORDER BY 절)

**난이도**: 중간 — 기존 부분 구현을 일반화하는 리팩토링 + 백엔드 정렬 보강

### T2: 출결 그리드 정렬 + 스크롤 개선 — 3.5h

사용자 요구 2번(출결 그리드 컬럼 정렬) + 3번(스크롤 접근성) 해결.

- ✅ `AttendanceGrid.tsx` 헤더 클릭 정렬 신규 구현 — T1 공통 훅(useTableSort) 적용 (10e485e)
  - 정렬 가능 컬럼: 원생(학년+이름), 출석, 미처리결석, 보강필요, 보강완료 — 실제 표시되는 좌측 고정 컬럼 기준(학교급/수업시간 컬럼 자체가 이 그리드엔 없어 계획 대비 조정)
  - 기본 정렬: 학년별+이름 가나다순 (백엔드 ORDER BY 변경 + 프론트 기본 comparator)
  - 날짜 셀은 정렬 대상 아님 (셀 데이터 특성상)
- ✅ 이중 스크롤 컨테이너 해소 (10e485e): `attendance/page.tsx` section을 `overflow-hidden`으로, `AttendanceGrid.tsx` 내부 div가 유일한 스크롤 컨테이너가 되도록 flex 재구성
  - `thead sticky top-0` + 좌측 4컬럼 sticky 유지
  - 가로 스크롤바가 뷰포트 내 고정 위치에서 항상 접근 가능

**관련 파일**:
- `src/components/attendance/AttendanceGrid.tsx` (라인 233-274)
- `src/app/attendance/page.tsx` (라인 350)

**난이도**: 중간~높음 — 이중 스크롤 구조 변경은 기존 sticky 레이아웃과의 상호작용 주의 필요

### T3: 청구 그리드 정렬 + 스크롤 개선 — 2.5h

사용자 요구 2번(청구 그리드 컬럼 정렬) + 3번(스크롤 접근성) 해결.

- ✅ `BillingGrid.tsx` 헤더 클릭 정렬 신규 구현 — T1 공통 훅 적용 (491e5a4)
  - 정렬 가능 컬럼: 원생명, 학년, 표준금액, 조정금액, 상태
  - **계획 대비 조정**: 기본 정렬은 학년+이름 단독이 아니라 기존 확정 워크플로우 그룹핑(미확정→확정, 월중입퇴교 우선)을 유지 — 백엔드 3차 정렬키만 이름→학년+이름으로 강화. 컬럼 클릭 시에는 전체 재정렬(그룹핑 무시)
- ✅ 스크롤: `thead sticky top-0` 적용. 이 그리드는 출결 그리드와 달리 이중 스크롤 컨테이너 구조가 없어(부모에 별도 overflow 래퍼 없음) T2 수준의 구조 변경은 불필요했음

**관련 파일**:
- `src/components/billing/BillingGrid.tsx`
- `src/app/billing/page.tsx`

**난이도**: 중간 — T2 패턴 재사용으로 T2 대비 난이도 낮음

### T4: 교습일정 인쇄 캘린더 개선 — 5h · skill: frontend-design

사용자 요구 4번. 인쇄용 `AcademicSchedulePrint.tsx`의 캘린더를 공지문 생성(`calendar-image.ts`)과 동등한 수준으로 시각 개선한다.

- ⬜ 교습일 Red 테두리 표시: 교습기간 내 수업 가능일(공휴일/보강불가 코드 미등록일)의 `<td>` 셀에 `border: 2px solid #E03131` 적용
  - 판별 로직: `calendar-image.ts:352-419` `drawTeachingOutline()` 참조 — regionStart~regionEnd 내 `allows_regular_class=true` 일자
  - 인쇄(`@media print`) 환경에서 테두리 렌더링 확인 필수
- ⬜ 기간성 학사일정 밴드(막대) 표현:
  - `schedule_codes.is_period_type=1`인 코드가 등록된 경우, 시작일~종료일을 걸치는 오버레이 밴드 배치
  - 구현 방식: table 위에 `position: absolute` 밴드 요소 배치 (부모 `position: relative`), 셀 좌표 계산으로 시작/종료 위치 결정
  - 라벨은 밴드 중앙에 1회만 표시 (기존 각 셀 반복 렌더링 폐기)
  - 셀 구분선(border)은 밴드 아래에 그대로 유지
  - 밴드 높이/너비 내에서 라벨 폰트 자동 조절 (`overflow: hidden`, `text-overflow: ellipsis`)
  - `calendar-image.ts:265-277` bandEvents/inBand 패턴 참조하되, CSS/HTML로 이식
- ⬜ 캘린더 내 일정 정보 폰트 크기 확대: 셀 너비를 넘지 않는 범위에서 최대한 크게 (`font-size: clamp(...)` 또는 셀 크기 대비 상대 단위)
- ⬜ A116 반영: 동적 행 수 계산 (`Math.ceil(cells.length / 7)`) — 5행 달에서 빈 6번째 행 제거
- ⬜ 인쇄(`@media print`) 렌더링 호환성 테스트: Chrome/Edge WebView 기준 오버레이 밴드+테두리 정상 출력 확인

**관련 파일**:
- `src/components/academic/AcademicSchedulePrint.tsx` (라인 65-79, 105, 117, 124-131)
- `src/lib/calendar-image.ts` (참조용: 라인 204-235, 265-277, 332-350, 352-419)
- `src/components/academic/CalendarCell.tsx` (참조용: 라인 141-242)

**난이도**: 높음 — absolute 포지셔닝 밴드 오버레이 + 인쇄 호환성은 까다로움. 밴드 좌표 계산(셀 DOM 위치 기반)이 핵심 난관

### T5: 수업관리 주보기 개선 — 화살표 제거 + 2xN 버그 수정 — 4.5h · skill: systematic-debugging

사용자 요구 5번. 원인 미확정 상태의 2xN 규칙 위반 버그 포함.

- ⬜ 화살표(↓/↑) 완전 제거: `ClassCalendar.tsx` 라인 582-587의 `multiSlot && !isLast` / `!isFirst` 조건부 아이콘 렌더링 삭제. 이름만 표시
- ⬜ 2xN 규칙 위반 버그 디버깅 + 수정:
  - **현상**: 4명 초과 시 한 셀에 2xN 규칙이 지켜지지 않음 (2026-06 데이터에서 대부분 재현)
  - **조사 대상**: `assignColumns()` (라인 113-136, greedy interval packing) + 2열 재배치 로직 (라인 226-279)
  - `needSplit` 트리거 조건(라인 247: `overlapTotal > 2`, 즉 3명 이상) + `rowGroup = Math.floor(column / 2)` (라인 248) 30분 단위 행 분할 로직 정밀 분석
  - 개발 PC 앱 데이터 DB(`C:\개인폴더\스마트해법\smarthb\app.db`)의 2026-06 월 데이터에서 다수 원생 겹치는 요일/시간대로 재현 후 근본 원인 규명
  - 수정 후 2명, 4명, 6명, 8명 등 다양한 겹침 시나리오에서 2xN 규칙 준수 확인
- ⬜ 주보기에서 원생 이름만 표시 (시간 정보 등 부가 정보 표시 여부 점검)

**관련 파일**:
- `src/components/schedules/ClassCalendar.tsx` (라인 113-136, 226-279, 247-248, 582-587)

**난이도**: 높음 — 2xN 버그의 근본 원인이 미확정. `assignColumns()` interval packing + 행 분할 로직의 edge case 분석 필요. systematic-debugging 스킬로 5단계 절차 적용

### T6: 수업관리 일보기 10xN 규칙 + 캘린더 라인 진하게 — 2.5h

사용자 요구 6번(일보기 10xN) + 7번(캘린더 라인 진하게) 통합.

- ⬜ 일보기 10xN 규칙 구현:
  - 현재: `isDay`일 때 `needSplit=false` 강제(라인 247) → FullCalendar 자동 폭 분배만 사용
  - 변경: 한 시간대에 10명 초과 시, 주보기의 2xN과 동일 원리로 행 분할 적용 (10명 단위)
  - `assignColumns()` 결과의 `overlapTotal > 10` 시 `needSplit=true`, `rowGroup = Math.floor(column / 10)` 패턴
  - 10명 이하는 기존 FullCalendar 자동 배치 유지
- ⬜ 월/주/일보기 캘린더 grid border 진하게:
  - FullCalendar CSS 커스터마이징: `.fc .fc-scrollgrid`, `.fc td`, `.fc th` 등의 `border-color` / `border-width` 조정
  - 현재 기본 `border-color`(연한 회색) → 진한 회색(`#6b7280` 또는 `border-gray-400` 수준)으로 변경
  - `ClassCalendar.tsx` 내 `<style>` 블록 또는 `globals.css` FullCalendar override 섹션에 추가

**관련 파일**:
- `src/components/schedules/ClassCalendar.tsx` (라인 247, 432, 446 + CSS 스타일 블록)
- `src/app/globals.css` (FullCalendar CSS override)

**난이도**: 중간 — 10xN 로직은 주보기 2xN 패턴을 확장하는 것이므로 T5 완료 후 진행하면 수월. CSS 변경은 단순

### T7: 대시보드 레이아웃 변경 — 1.5h

사용자 요구 8번. 당일 수업 + 이달의 생일 위젯을 좌우 배치에서 상하 배치로 변경.

- ⬜ `DashboardView.tsx`(라인 125-171) 레이아웃 변경:
  - 현재: `flex-row` (sm 이상에서 좌우) + 각 위젯 `sm:w-1/2`
  - 변경: `flex-col` (항상 상하) + 당일 수업 `flex-[2]` / 이달의 생일 `flex-[1]` 비율
  - `sm:flex-row` 제거, `sm:w-1/2` 제거
- ⬜ 당일 수업 위젯 높이가 이달의 생일 위젯의 2배가 되도록 `flex-grow` 비율 조정
- ⬜ 위젯 내부 콘텐츠가 늘어난 높이에 자연스럽게 채워지는지 확인 (overflow 처리)

**관련 파일**:
- `src/components/dashboard/DashboardView.tsx` (라인 125-171)

**난이도**: 낮음 — Tailwind 클래스 변경으로 해결 가능

### T8: 학년 자동 승급 (매년 1월 이후 최초 실행) — 3h

연초 학년 자동 승급 기능. `diagnosis.rs`의 "매월 1일 자동 진단" 패턴과 동일한 트리거 구조.

- ⬜ 백엔드 IPC 커맨드 신규: `students.rs`에 `check_grade_promotion` + `promote_grades` 2종
  - `check_grade_promotion`: `app_settings`에서 `last_grade_promotion_year` 조회 → 현재 연도와 비교 → 대상 존재 여부 + 인원수 반환
    - 대상: `withdraw_date IS NULL` (재원생만) AND `school_level IN ('elementary', 'middle')` AND 최대 학년 미만 (초등 grade < 6, 중등 grade < 3)
    - 최대 학년(초6/중3) 도달 시 승급 대상에서 제외
  - `promote_grades`: 대상 일괄 `UPDATE students SET grade = grade + 1` + `app_settings` `last_grade_promotion_year` 현재 연도로 갱신 + audit 로그
  - 단위 테스트: 정상 승급 / 최대 학년 미승급 / 퇴교생 제외 / 이미 처리된 연도 스킵 — 4건
- ⬜ 프론트엔드 확인 다이얼로그: `app-shell.tsx`의 세션당 1회 체크 플래그 패턴 재사용
  - 앱 시작 시 `check_grade_promotion` 호출 → 대상 N명 존재 시 "올해 N명의 학년이 자동 상향됩니다. 진행하시겠습니까?" 확인 다이얼로그 표시
  - 사용자 승인 시에만 `promote_grades` 호출, 거부 시 다음 세션까지 재확인 없음 (세션당 1회)
  - "자동 조용히 적용 금지" 원칙 준수 (frontend.md 위험 동작 확인 다이얼로그)
- ⬜ TypeScript IPC 래퍼 2종 (`src/lib/tauri/index.ts`)

**관련 파일**:
- `src-tauri/src/commands/students.rs` (IPC 신규)
- `src-tauri/src/commands/mod.rs` (invoke_handler 등록)
- `src-tauri/src/lib.rs` (invoke_handler 등록)
- `src/lib/tauri/index.ts` (래퍼 2종)
- `src/components/layout/app-shell.tsx` (세션당 1회 체크 패턴)
- `src-tauri/src/commands/diagnosis.rs` (참조: `last_auto_diagnosis` 패턴)

**난이도**: 중간 — 기존 diagnosis.rs 패턴 재사용으로 설계 난이도 낮음. 단위 테스트 포함

### T9: 학교급 기반 학교 선택 필터링 — 3.5h

학교 등록 시 `school_type` 입력 UI 추가 + 원생 폼 학교 드롭다운을 `school_type` 기반 필터로 교체 + 기존 데이터 자동 보정 마이그레이션.

- ⬜ DB 마이그레이션 V310: 기존 학교 데이터 `school_type` 자동 보정
  - `name LIKE '%초등학교%'` → `school_type = 'elementary'`
  - `name LIKE '%중학교%'` → `school_type = 'middle'`
  - 나머지는 `etc` 유지 (현재 전부 `etc`)
  - 파일명: `310__auto_correct_school_type.sql`
- ⬜ 설정 > 코드 테이블 > 학교 등록/수정 화면에 `school_type` 선택 UI 추가
  - `src/app/settings/codes/page.tsx`에 라디오 또는 셀렉트 (elementary/middle/high/etc)
  - 백엔드 `codes.rs`의 `NewCode.extra` 필드는 이미 학교 확장 속성 지원 → 프론트에서 `school_type` 값을 `extra`에 포함하여 전송
  - 기존 학교 수정 시에도 `school_type` 변경 가능
- ⬜ 원생 등록/수정 폼 학교 드롭다운 필터링 교체
  - `src/components/students/student-form.tsx`(라인 180-193)의 `.includes('중학교')` 텍스트 매칭 → 실제 `school_type` 컬럼 기반 필터링으로 교체
  - 학생의 `school_level` 선택값(`elementary`/`middle`)과 일치하는 `school_type`의 학교만 드롭다운에 노출
  - `school_level` 변경 시 드롭다운 목록 자동 갱신 + 기존 선택값 초기화
  - `school_type='etc'`는 모든 학교급에서 표시 (기타 학교)

**관련 파일**:
- `src-tauri/migrations/310__auto_correct_school_type.sql` (신규)
- `src/app/settings/codes/page.tsx` (학교 등록/수정 UI)
- `src/components/students/student-form.tsx` (라인 180-193)
- `src-tauri/src/commands/codes.rs` (참조: `NewCode.extra`)

**난이도**: 중간 — V310 단순 UPDATE, UI는 기존 구조에 셀렉트 추가. 학교 드롭다운 필터링 로직 교체가 핵심

### T10: 통합 검증 — 2.5h

- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (T8 신규 단위 테스트 4건 포함)
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ⬜ `pnpm lint` clean
- ⬜ `pnpm tsc --noEmit` clean
- ⬜ `pnpm build` (static export) 성공
- ⬜ `sqlx migrate run` 정상 (V310 적용 확인)
- ⬜ 시각 검증: 각 요구사항별 Before/After 확인
  - 원생 목록 기본 정렬 학년+이름순 확인
  - 출결/청구 그리드 컬럼 클릭 정렬 동작
  - 출결 그리드 가로 스크롤 즉시 접근 가능
  - 교습일정 인쇄: Red 테두리 + 밴드 + 폰트 확대 + 동적 행 수
  - 주보기: 이름만 표시, 2xN 규칙 준수
  - 일보기: 10명 초과 시 행 분할
  - 캘린더 라인 진한 테두리
  - 대시보드 상하 배치 + 2:1 비율
  - 학년 승급: 1월 이후 첫 실행 시 확인 다이얼로그 → 승인 시 일괄 승급 → 재실행 시 미표시
  - 학교 필터: 설정 > 학교 등록에 school_type 선택 → 원생 폼에서 학교급에 맞는 학교만 드롭다운 노출

## 완료 기준 (Definition of Done)

**필수**
- ⬜ 모든 원생 그리드 기본 정렬: school_level ASC → grade ASC → name 가나다순
- ⬜ 원생 목록 / 출결 / 청구 그리드에서 컬럼 헤더 클릭 정렬 동작 (동일 학년 내 이름 2차 정렬 자동)
- ⬜ 출결/청구 그리드 가로 스크롤이 페이지 어디서든 접근 가능 (이중 스크롤 컨테이너 해소)
- ⬜ 교습일정 인쇄 캘린더: 교습일 Red 테두리 + 기간성 일정 밴드 오버레이 + 폰트 확대
- ⬜ 수업관리 주보기: 화살표 제거, 이름만 표시, 4명 초과 시 2xN 규칙 준수
- ⬜ 수업관리 일보기: 10명 초과 시 10xN 규칙으로 행 분할
- ⬜ 수업관리 월/주/일보기 캘린더 grid border 진하게
- ⬜ 대시보드 당일수업+생일 위젯 상하 배치, 높이 비율 2:1
- ⬜ 학년 자동 승급: 1월 이후 최초 실행 시 확인 다이얼로그 → 승인 시 일괄 승급, 최대학년(초6/중3) 미승급, 퇴교생 제외
- ⬜ 학교급 기반 필터: 설정 > 학교에 school_type 선택 UI, 원생 폼에서 school_level에 맞는 학교만 드롭다운 노출
- ⬜ V310 마이그레이션 정상 적용 (기존 학교 school_type 자동 보정)
- ⬜ cargo test 전체 통과 (Sprint 18 기준 418건 + T8 신규 4건 이상)
- ⬜ cargo clippy --all-targets -D warnings clean
- ⬜ pnpm lint + pnpm tsc --noEmit + pnpm build 전수 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

## Capacity 확인

| 항목 | 값 |
|------|---|
| 팀 규모 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10일 |
| 일 실작업 시간 | 4h |
| 총 Capacity | 40h |
| 계획 작업량 | ~33h (T0 1.5h + T1 3h + T2 3.5h + T3 2.5h + T4 5h + T5 4.5h + T6 2.5h + T7 1.5h + T8 3h + T9 3.5h + T10 2.5h) |
| 여유 버퍼 | ~7h (18%) — T5 디버깅 불확실성 + 시각 검증 후 UX 보강 예산 |

> Sprint 17(16h 예상) / Sprint 18(17h 예상) 대비 작업량이 많으나, UI/UX 변경 8건은 백엔드 IPC 없이 프론트엔드 전용. T8(학년 승급)은 기존 diagnosis.rs 패턴 재사용, T9(학교 필터)는 V310 단순 UPDATE + UI 셀렉트 추가로 복잡도 낮음. 전체 33h는 40h Capacity 내(18% 버퍼). 이연 불필요.

## 참고 사항

### 작업 의존성
```
T0 (회고 액션)
 ↓
T1 (공통 정렬 인프라) ──→ T2 (출결 정렬+스크롤) ──→ T3 (청구 정렬+스크롤)
                                                      
T4 (인쇄 캘린더) ← 독립
T5 (주보기 2xN) ──→ T6 (일보기 10xN + 라인) ← T5의 행 분할 패턴 확장
T7 (대시보드) ← 독립
T8 (학년 자동승급) ← 독립 (students.rs + app-shell.tsx)
T9 (학교급 필터링) ← 독립 (V310 + codes/page + student-form)
T10 (통합 검증) ← 모든 Task 완료 후
```

### 기술적 주의사항
- **정렬 인프라**: 프론트엔드 클라이언트 사이드 정렬로 통일. 원생 목록만 서버사이드 정렬(페이지네이션 때문) — `list_students` ORDER BY 보강 필요할 수 있음
- **스크롤 개선**: 이중 스크롤 컨테이너 해소 시 기존 sticky 컬럼(`left-0/140/202/264/348`)과의 상호작용 반드시 테스트. 한쪽 스크롤 제거 시 sticky가 풀리는 경우가 있음
- **인쇄 밴드 오버레이**: `position: absolute` 밴드의 좌표는 셀 DOM의 `offsetLeft`/`offsetWidth` 기반으로 계산. `@media print`에서 CSS box model이 화면과 다를 수 있으므로 인쇄 미리보기 필수 확인
- **2xN 버그**: 원인 미확정 상태. `assignColumns()` greedy interval packing의 column 할당과 `Math.floor(column / 2)` rowGroup 매핑 사이의 불일치가 유력 후보. 개발 PC의 2026-06 데이터(36명, 다수 겹침)로 재현 후 분석
- **10xN 규칙**: T5에서 수정한 2xN 패턴을 10명 단위로 확장. T5 완료 후 진행이 효율적
- **FullCalendar CSS**: 커스텀 CSS 우선순위 확보 필요 (`:where()` 또는 specificity 조정)
- **학년 승급 트리거**: `diagnosis.rs`의 `LAST_AUTO_DIAGNOSIS_KEY` + `app_settings` 패턴을 `last_grade_promotion_year`로 복제. 값은 연도 문자열(예: `"2026"`). 현재 연도와 다르면 대상 조회 → 다이얼로그 → 승인 시 UPDATE + 연도 갱신
- **학교 school_type**: V310 마이그레이션은 `UPDATE schools SET school_type = 'elementary' WHERE name LIKE '%초등학교%'` + middle 패턴. 번호 310은 301~(보정/확장) 블록 내 다음 번호(V309 최신)
- **원생 폼 학교 드롭다운**: 현재 `student-form.tsx:180-193`의 `.includes('중학교')` 경고 로직 → `school_type` 기반 필터링으로 교체 시 경고 로직도 불필요해짐(목록 자체가 필터링되므로)

### 리스크
| ID | 설명 | 영향도 | 대응 |
|----|------|--------|------|
| R126 | T5 2xN 버그 원인 불명 — 디버깅이 예상보다 길어질 수 있음 | 중간 | 4.5h + 버퍼 7h 확보. 3-retry 후에도 미해결 시 사용자 보고 |
| R127 | T4 인쇄 밴드 오버레이가 `@media print`에서 정상 렌더링되지 않을 가능성 | 중간 | 밴드 미동작 시 대안: 셀 배경색+좌측 테두리 조합으로 시각적 연속성 표현 (degraded fallback) |
| R128 | T2 이중 스크롤 해소 시 기존 sticky 4컬럼 레이아웃 파손 | 중간 | 변경 전 현재 sticky 동작 스크린샷 보존. 문제 발생 시 부모 overflow 제거 대신 `position: sticky; bottom: 0` 스크롤바 고정 대안 검토 |
