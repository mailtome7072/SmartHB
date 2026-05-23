---
Sprint: 6  |  Date: 2026-05-22  |  Session: #9
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — **마지막 세션** — T11 + T12 일괄 + 드래그 이동 포함.
> 사용자 결정(2026-05-22): T11 + T12 일괄, 드래그 이동 T11 에서 (@dnd-kit 활용).
> 예상 8h. R30: 드래그 이동은 단일 일자만 (기간성 제외).

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 / T3 / T4 (Sprint 5 부채) | `2c5b8a1` `c2be584` `83f19d1` |
| #2 | T5+T6 (academic.rs study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | T7 (academic.rs schedule_events 5) | `a4c380e` |
| #4 | T8 (TS IPC 래퍼 15 + 도메인 타입 10) | `5941d24` |
| #5 | T2-c (ADR-005) | `10a92d4` |
| #6 | T2-a / T2-b (스크립트 + V301 + 64건 공휴일) | `1d0ebe1` `f534706` |
| #7 | T9 (3개월 캘린더 + /academic) | `604027b` |
| #8 | T10 (StudyPeriodEditor + selection 통합) | `bb767d6` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T11** | ScheduleCodePanel + EventPlacer + 일정 삭제 + 드래그 이동 (단일 일자) | 6h |
| **T12** | 통합 검증 (cargo test 전체 + tsc/lint/build + 회귀 점검) | 2h |

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/academic/CalendarCell.tsx | [5회 ⚠️] | 외부 button → div + role/tabIndex 리팩토링 (배지 button 분리 가능하도록), 드래그 droppable 지원 |
| src/components/academic/ThreeMonthCalendar.tsx | [15회 ⚠️] | DndContext 래핑, onEventDelete/onEventDrop props 추가, 일정 배지 button 분리 |
| src/components/academic/ScheduleCodePanel.tsx | [1회] | 신규 — 코드 목록 + 시스템 토글 + 사용자 CRUD + 코드 선택 (radio-like) |
| src/components/academic/EventPlacer.tsx | [2회] | 신규 — 선택 코드 표시 + 셀 클릭 핸들러 + 자동 배치 버튼 + 한국어 에러 처리 |
| src/app/academic/page.tsx | [5회 ⚠️] | mode 확장 `'view'|'study-period'|'event-place'`, ScheduleCodePanel/EventPlacer 통합 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/` — 백엔드 IPC 그대로 (T5/T6/T7 + V301 모두 충족)
- [ ] `src/lib/tauri/index.ts` — T8 래퍼 그대로 사용
- [ ] `src/types/academic.ts` — T8 타입 그대로 사용
- [ ] `package.json` — @dnd-kit 이미 설치됨, 신규 의존성 없음
- [ ] StudyPeriodEditor.tsx — T10 그대로

## 완료 기준 (이번 세션)

### T11 — 학사 일정 코드 + 일정 배치 UI (PRD §4.4.3~4.4.7, sprint6.md L309-338)
- ✅ AC-T11-1: 시스템 코드 6종 🔒 표기 + 3속성 readonly + 백엔드 `update_schedule_code` 가드
- ✅ AC-T11-2: 신규 폼 디폴트 `DEFAULT_FORM = { regular: false, makeup: false, dup_blocked: true, period: false }`
- ✅ AC-T11-3: 단일 일자 코드 선택 → 셀 클릭 즉시 `createScheduleEvent` (useEventPlaceCellHandler)
- ✅ AC-T11-4: 기간성 코드 → selectionStart/End 두 셀 클릭 → "확정" 버튼 → INSERT
- ✅ AC-T11-5: 백엔드 중복불가 에러 ("동일 일자에 같은 코드의 일정이 이미 존재합니다") → AlertDialog
- ✅ AC-T11-6: "단원평가 응시일" 코드 선택 시 "{centerYearMonth} 자동 배치" 버튼 → `autoPlaceAssessmentDates` (No-op 시 결과 다이얼로그)
- ✅ AC-T11-7: 지난 달 셀 disabled (T9) + 백엔드 `update_schedule_event` / `delete_schedule_event` 가드

### 추가 (T11 부수)
- ✅ CalendarCell 외부 button → div + role/tabIndex/keyDown (HTML 표준 중첩 회피)
- ✅ 일정 배지 클릭 → 삭제 AlertDialog → `deleteScheduleEvent` + invalidateQueries + 백엔드 에러 처리
- ✅ @dnd-kit 드래그 이동 — EventBadge(useDraggable) + DroppableCell(useDroppable) + DndContext + PointerSensor 8px
- ✅ 드래그 제한: 단일 일자만, 시스템 코드 6종 제외 (실수 방지)
- ✅ 시스템 코드 활성 토글 (`toggleScheduleCodeActive`) — 3속성은 잠금

### T12 — 통합 검증 (sprint6.md L344-365)
- ✅ `cargo test --manifest-path src-tauri/Cargo.toml` **146 passed**, 0 failed (병렬 실행 안정)
- ✅ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- ✅ `pnpm tsc --noEmit` 통과 (strict)
- ✅ `pnpm lint` "No ESLint warnings or errors"
- ✅ `pnpm build` 모든 라우트 정상 (/academic 9.49kB / First Load 176kB)
- ✅ Sprint 6 전체 회귀 점검:
  - T1 (lock 재시도) `/lock` 빌드 OK
  - T2 (V301 시드) `v301_*` 5 테스트 통과
  - T3 (paths.rs OnceLock) 병렬 실행 안정
  - T4 (DnD sort_order) `/settings/codes` 빌드 OK
  - T5/T6/T7 (academic.rs IPC 15) `academic` 16 테스트 통과
  - T8 (TS IPC 래퍼) tsc 통과
  - T9/T10/T11 (`/academic` 캘린더 + 교습기간 + 배치 + 드래그) 빌드 OK

### 세션 종료 조건
- ✅ 단계별 커밋 4개: T11 코드+배치+삭제 `651d569` / T11 드래그 `2d4c95b` / T12 + scope 완료 (본 커밋)
- ✅ Self-verify 각 단계 통과
- ✅ simplify 검토 — EventBadge 추출이 React hooks 규칙 준수에 필수, 그 외 추상화 적절
- ✅ Sprint 6 Definition of Done 모든 필수 항목 ✅ (sprint-close 에이전트가 ROADMAP/CHANGELOG/DEPLOY 처리)

## 설계 결정

### Mode 확장
- `'view' | 'study-period' | 'event-place'`
- mode='study-period' 은 T10 의 'editing' 을 리네이밍 (의미 명확화)
- mode='event-place' 신규

### ScheduleCodePanel 구조
- 코드 목록 (시스템 5+1종 위, 사용자 추가 아래) — 라디오 button 으로 코드 선택
- 시스템 코드: `is_system_reserved=1` 표시(🔒) + 3속성 readonly + 활성 토글만 활성
- 사용자 코드: 신규 추가 폼 (code_name + 3속성 체크박스 + 단일/기간성 선택) + 편집/활성 토글
- 코드 선택 시 mode='event-place' 자동 활성

### EventPlacer 흐름
1. 선택된 코드 표시: "{코드명} 배치 중 (단일 일자 / 기간성)" + "취소" 버튼
2. 단원평가 코드 선택 시 "자동 배치" 버튼 표시 — `autoPlaceAssessmentDates(중앙 월)` 호출
3. 단일 일자 코드: 셀 1회 클릭 → `createScheduleEvent({ code_id, event_date, period_end_date: null })`
4. 기간성 코드: T10 의 selection 패턴 재사용 — 시작/종료일 두 셀 클릭 → `createScheduleEvent({ code_id, event_date, period_end_date })`
5. 백엔드 에러 (중복불가 등) → AlertDialog 한국어 메시지

### 일정 삭제 흐름
1. 셀의 일정 배지 button 클릭
2. AlertDialog 확인 ("'{display_name}' 일정을 삭제합니다")
3. `deleteScheduleEvent(id)` + invalidateQueries(['schedule-events'])
4. 백엔드가 지난 달 삭제 시도 시 한국어 에러 → AlertDialog

### 드래그 이동 (@dnd-kit)
- DndContext 가 ThreeMonthCalendar 를 감싸기
- 일정 배지 = `useDraggable` (단일 일자 + is_period_type=0 인 경우만)
- CalendarCell = `useDroppable` (지난 달 / 그리드 외 제외)
- onDragEnd: drop 셀 date 추출 → `updateScheduleEvent(id, { event_date: newDate, period_end_date: null, display_name: ... })` + invalidateQueries
- 교습기간 외 셀에 drop 시 사용자 친화 안내 (현재 모든 셀이 droppable — 백엔드 가드는 없으나 UI 제약은 우선 안 둠. 시간 부족 시 생략)
- 공휴일/단원평가는 드래그 불가 (시스템 코드 보호)

### CalendarCell 리팩토링 (button→div)
- 이유: button 안에 button 중첩 불가 (HTML 표준)
- 변경: 외부 `<button>` → `<div role="button" tabIndex={0} onClick={...} onKeyDown={Enter}>` 
- 배지를 `<button onClick={(e)=>{e.stopPropagation(); onEventClick(event)}}>` 로 분리
- 접근성 보존: aria-label / aria-pressed / focus ring 유지

### CalendarCell prop 추가
- `onEventDelete?: (event: ScheduleEventListItem) => void` — 배지 클릭
- `draggableEventIds: Set<number>` — 드래그 가능 일정 id (외부에서 계산)

### 신규 의존성
- @dnd-kit/core, @dnd-kit/sortable, @dnd-kit/utilities — package.json 이미 보유

## 코드 패턴 SSOT

- 컴포넌트: `'use client'` 명시
- IPC: @/lib/tauri 추상화 레이어 (createScheduleEvent / updateScheduleEvent / deleteScheduleEvent / autoPlaceAssessmentDates / listScheduleCodes / createScheduleCode / updateScheduleCode / toggleScheduleCodeActive)
- TanStack Query: useQuery (listScheduleCodes, listScheduleEvents) + useMutation (create/update/delete/toggle/autoPlace)
- 모든 mutation 의 onSuccess 에서 invalidateQueries(['schedule-events']) / (['schedule-codes'])

## 발견된 이슈

> 진행 중 새 제약 발견 시 여기에 기록.
