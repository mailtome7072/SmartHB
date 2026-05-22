---
Sprint: 7  |  Date: 2026-05-22  |  Session: #6
---

> Sprint 7 Session #6 — T6 단독 (교습기간 설정 UX 재설계).
> Issue 5 carry-over 해소. 토글 제거 + 자동 selection. 예상 4h.

## 이전 세션 결과

- Session #1 (`8eb1c92`): T1 — Keychain 통합 캐싱
- Session #2 (`4178324`): T2 — salt.bin 이전 + 보안 패치 6건
- Session #3 (`2fad4fb`): T3 — device_id 영속화
- Session #4 (`6b5f8de`): T4 — is_system_reserved JOIN
- Session #5 (`ba7ef09`): T5 — 코드 관리 /settings 이동 + Selector 분리

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T6** | 교습기간 설정 UX 재설계 — 토글 제거 + 미확정 월 자동 selection 모드 | 4h |

> 사용자 결정 (2026-05-22): Session #6 = T6 단독.

## 설계 결정 (T6)

### Before vs After
- **Before**: `mode = 'view' | 'editing'` 토글 버튼 → 모드 활성 → 셀 클릭 → "확정"
- **After**: 토글 제거. 미확정 월에서 셀 클릭 즉시 selection 모드 자동 활성. "확정" / "취소" 버튼만.

### 미확정 월 판정
- 현재 보고 있는 중앙 월 (`centerYearMonth`) 에 대해 `getStudyPeriod(yearMonth)` IPC 호출
- `null` 반환 → 미확정 (selection 모드 자동 활성)
- `StudyPeriod` 객체 반환 → 확정됨 (읽기 전용 표시)
- TanStack Query 캐시 키 `['study-period', yearMonth]` — staleTime 30s

### 모드 충돌 회피
- 일정 배치 모드 (`selectedCode !== null`) 가 활성이면 교습기간 selection 비활성 — 두 모드 동시 활성 차단
- `eventPlaceMode: boolean` prop 으로 StudyPeriodEditor 에 전달

### StudyPeriodEditor props 시그니처 변경
- **제거**: `mode: EditorMode`, `setMode: (m: EditorMode) => void`
- **추가**: `centerYearMonth: string` (현재 중앙 월), `eventPlaceMode: boolean` (배치 모드 활성 여부)
- **유지**: `selection: SelectionRange`, `setSelection: (s: SelectionRange) => void`
- `EditorMode` 타입 export 는 유지 — 다른 곳에서 import 가능성 대비 (실제 사용처 없으면 제거)

### `/academic` 페이지 변경
- `studyPeriodMode` state 제거
- selection 분기: `getStudyPeriod(centerYearMonth)` 결과에 따라 자동 결정
- `handleCellClick`: selectedCode 있으면 일정 배치, 없고 미확정 월이면 교습기간 selection
- `useEffect` 충돌 회피 로직 단순화 (mode → selectedCode 동기화 불필요)

### 캘린더 네비게이션 (AC-T6-5)
- ThreeMonthCalendar 가 이미 prev/next 버튼 보유 (`shift(delta)` + `nav` 영역)
- 추가 작업 불필요 — 기존 네비게이션이 "화면 최상단 캘린더 네비게이션" 요구사항 충족

### 신규 의존성
- 없음.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/academic/StudyPeriodEditor.tsx | [1회] | props 시그니처 변경 + 토글 제거 + getStudyPeriod 조회 |
| src/app/academic/page.tsx | [9회 ⚠️] | studyPeriodMode state 제거 + 분기 단순화 |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/**` — 본 세션 백엔드 변경 없음
- [ ] `src/components/academic/ThreeMonthCalendar.tsx` — 네비게이션 기존 활용
- [ ] `src/lib/tauri/index.ts` — getStudyPeriod 기존 IPC 활용 (인터페이스 변경 없음)
- [ ] `src/types/academic.ts` — StudyPeriod 타입 변경 없음

## 완료 기준 (이번 세션)

### T6 — 교습기간 설정 UX 재설계 (sprint7.md L176-200)
- ✅ AC-T6-1: 토글 버튼 제거. `studyPeriodMode` state 폐기, `getStudyPeriod` 결과로 자동 활성
- ✅ AC-T6-2: 안내 메시지 "캘린더에서 교습기간을 선택하세요" — `isSelectionActive` 시에만 표시
- ✅ AC-T6-3: 시작일/종료일 클릭 후 "확정" + "취소" 버튼 — 토글 없이 selection 으로만 분기
- ✅ AC-T6-4: 확정 월은 `confirmedPeriod` 표시 (읽기 전용) — 삭제 버튼은 T8 carry-over
- ✅ AC-T6-5: ThreeMonthCalendar 기존 prev/next nav 가 sprint7.md 요구사항 충족

### 세션 종료 조건
- ✅ Self-verify: pnpm lint clean / pnpm tsc --noEmit clean / cargo check clean
- ✅ simplify — `studyPeriodMode` state + 충돌 회피 useEffect 제거, `EditorMode` 타입 폐기, 호출부 단순화
- ⬜ 단일 커밋 (2파일 + scope.md)

## 발견된 이슈

(없음 — Step-back 트리거 발생 시 여기에 기록)

## carry-over

- Session #2 발견 9건 (I-S2-2 ~ I-S2-10) — 후속
- Session #4 발견 1건 (I-S4-1: CalendarCell hasHoliday/hasAssessment) — 후속
