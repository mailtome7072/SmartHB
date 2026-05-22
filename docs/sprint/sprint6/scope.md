---
Sprint: 6  |  Date: 2026-05-22  |  Session: #7
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — T9 3개월 캘린더 + 데이터 통합.
> 사용자 결정(2026-05-22): T9 일괄 (구조+데이터) / Tailwind grid 직접 구현 (shadcn Calendar 미사용).
> 예상 6h. R30 리스크: 직접 구현 선택으로 완화 (props.modifiers 복잡도 회피).

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 / T3 / T4 (Sprint 5 부채) | `2c5b8a1` `c2be584` `83f19d1` |
| #2 | T5+T6 (academic.rs study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | T7 (academic.rs schedule_events 5) | `a4c380e` |
| #4 | T8 (TS IPC 래퍼 15 + 도메인 타입 10) | `5941d24` |
| #5 | T2-c (ADR-005) | `10a92d4` |
| #6 | T2-a / T2-b (스크립트 + V301 + 64건 공휴일) | `1d0ebe1` `f534706` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T9** | 3개월 캘린더 + 라우트 + 사이드바 메뉴 + 공휴일/교습기간/일정 IPC 통합 | 6h |

> skill: frontend-design (sprint6.md L275). 다만 .claude/skills/ 에 frontend-design 스킬이 없으므로 디자인 원칙을 .claude/rules/frontend.md (PRD §5.1 + §5.7) 와 PRD 본문(§4.4.1) 에서 직접 참조.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/lib/menu-config.ts | [1회] | "학사 스케줄" 활성 메뉴 추가 (Phase 2 disabledHint 제거) |
| src/app/academic/page.tsx | [1회] | 신규 — 학사 스케줄 페이지 라우트, AppShell 안에 ThreeMonthCalendar 배치 |
| src/components/academic/ThreeMonthCalendar.tsx | [2회] | 신규 — 3개월 가로 배치 컨테이너, 중앙 년월 화살표 네비, IPC 데이터 조회 (TanStack Query) |
| src/components/academic/CalendarCell.tsx | [1회] | 신규 — 1일 셀: 날짜 / 공휴일 배지 / 교습기간 배경 / 일정 배지 (단원평가 셀 상단 띠) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/` — 본 세션 백엔드 변경 없음 (T7 IPC + T2 시드 모두 충족)
- [ ] `src/lib/tauri/index.ts` — T8 래퍼 그대로 사용
- [ ] `src/types/academic.ts` — T8 타입 그대로 사용
- [ ] `package.json` / `Cargo.toml` — 신규 의존성 없음

## 완료 기준 (이번 세션)

### T9 — 3개월 캘린더 (PRD §4.4.1, sprint6.md L256-283)
- ✅ AC-T9-1: 3개월 가로 배치 — `grid grid-cols-1 md:grid-cols-3` 반응형
- ✅ AC-T9-2: 중앙 년월 ← → 화살표 → setCenter → useMemo 로 prev/next 연동 (월 단위)
- ✅ AC-T9-3: V301 공휴일 64건이 `code_name='공휴일'` 빨강 배지 + 일요일 텍스트 빨강
- ✅ AC-T9-4: 교습기간 셀 `bg-amber-50` 파스텔, 일정 셀에 코드별 색상 배지
- ✅ AC-T9-5: `isMonthPast = year_month < current_year_month` → opacity-60 + cursor-not-allowed + onClick disabled (백엔드 AC-4.4-1 가드와 일관)
- ✅ AC-T9-6: Pretendard 18pt 본문 / `min-h-[72px] min-w-[44px]` / `text-[var(--foreground)]` WCAG AA 토큰

### 추가 (T9 부수)
- ✅ 사이드바 menu-config "학사 스케줄" 활성 메뉴 (`/academic`)
- ✅ 단원평가 셀 상단 띠 `border-t-4 border-blue-400` (PRD §4.4.7)
- ✅ 코드별 배지 7종: 공휴일 red / 보강데이 teal / 공휴수업일 pink / 방학 purple / 휴원일 gray / 단원평가 blue / 사용자 amber
- ✅ 빈 데이터: `data ?? []` + eventsByDate Map graceful

### Session #7 발견 + 처리
- ⚠️ `Object.values(currentYearMonth())` spread 순서 미보장 위험 → useState 초기화 함수로 명시적 cur.year/cur.month 사용 (simplify 검토 시 발견·즉시 수정)
- ⓘ `frontend-design` 스킬은 `.claude/skills/` 에 미존재 — 디자인 원칙을 `.claude/rules/frontend.md` + PRD §5.7 에서 직접 참조

### 세션 종료 조건
- ✅ 단일 커밋 `604027b` (4 신규 + 1 수정, +501줄)
- ✅ Self-verify: tsc 통과 / lint clean / `pnpm build` 성공 (/academic 3.34kB, First Load 123kB)
- ✅ simplify 검토 — spread 위험 1건 발견·제거, 나머지 추상화 적절
- ⬜ 시각 확인 (`pnpm tauri:dev`) — 본 세션 범위 외, 사용자가 자발 검증 권장

## 설계 결정

### 라이브러리 선택
- **Tailwind `grid-cols-7` 직접 구현** (사용자 결정 2026-05-22) — shadcn/ui Calendar / react-day-picker 미설치
- 사유: 3개월 합성 + 교습기간 오버레이 + 다중 배지(공휴일/일정/단원평가)의 커스터마이징 자유. R30 리스크 완화.

### 데이터 흐름
- **TanStack Query** 로 IPC 응답 캐싱
  - `listScheduleEvents(fromDate, toDate)` — 3개월 범위 (이전월 1일 ~ 다음월 말일) 일정 + 공휴일 통합 조회
  - `listStudyPeriods(fromMonth, toMonth)` — 3개월 범위 교습기간
  - 네비게이션 시 queryKey 변경 → 자동 refetch
- 셀 렌더링: 각 셀이 자신의 날짜에 해당하는 schedule_events / study_period 를 필터링하여 표시

### 시각 디자인 토큰 (PRD §5.7)
- 본문: Pretendard 18pt (globals.css 기존 설정 유지)
- 셀 최소: `min-h-[60px] min-w-[44px]` (44×44 권장 + 캘린더 공간 확보)
- 교습기간 배경: `bg-amber-50` (저자극 파스텔)
- 단원평가 띠: `border-t-4 border-blue-400`
- 공휴일 배지: `bg-red-100 text-red-800` (한국 캘린더 관습)
- 보강데이 / 공휴수업일 / 방학 / 휴원일: 코드 색상 매핑

### 지난 달 셀 처리 (AC-T9-5)
- 컨테이너 props 로 `isPastMonth` 전달
- CalendarCell 이 `isPastMonth=true` 일 때 onClick 무시 + opacity-60 + cursor-not-allowed
- 시각적으로 회색조 처리 (배경 + 텍스트)

### 메뉴 위치
- 사이드바 `MENU_ITEMS` 에 "학사 스케줄" 항목 — "원생 관리" 다음 (Phase 2 첫 진입)
- shortcut: 미정 (Ctrl+L 또는 추후 결정, 이번 세션은 보류)

## 코드 패턴 SSOT

- 컴포넌트: `'use client'` 명시 (Tauri IPC 호출 + useQuery 사용)
- IPC 호출: `@/lib/tauri` 추상화 레이어만 통과 (frontend.md 규칙)
- TanStack Query: 기존 패턴 (`students` 화면) 답습
- 키보드 단축키: 이번 세션 범위 외 (T10/T11 또는 후속 sprint)
- 셀 클릭 핸들러: 본 세션은 기본 onClick (T10 교습기간 설정 모드에서 props.mode 추가 예정)

## 발견된 이슈

> 진행 중 새 제약·충돌 발견 시 여기에 기록.
