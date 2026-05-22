---
Sprint: 6  |  Date: 2026-05-22  |  Session: #8
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — T10 교습기간 설정 UI.
> Session #7 의 ThreeMonthCalendar 에 selectionRange 프리뷰 추가 + StudyPeriodEditor 신규.
> 예상 4h. T11 (일정 코드 + 배치) 직전 마지막 교습기간 도메인.

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 / T3 / T4 (Sprint 5 부채) | `2c5b8a1` `c2be584` `83f19d1` |
| #2 | T5+T6 (academic.rs study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | T7 (academic.rs schedule_events 5) | `a4c380e` |
| #4 | T8 (TS IPC 래퍼 15 + 도메인 타입 10) | `5941d24` |
| #5 | T2-c (ADR-005) | `10a92d4` |
| #6 | T2-a / T2-b (스크립트 + V301 + 64건 공휴일) | `1d0ebe1` `f534706` |
| #7 | T9 (3개월 캘린더 + /academic 라우트) | `604027b` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T10** | StudyPeriodEditor + 캘린더 selectionRange 확장 + 지난 달 차단 시각화 | 4h |

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/academic/CalendarCell.tsx | [2회] | isInSelection / isSelectionStart / isSelectionEnd prop 추가, 하이라이트 색상 |
| src/components/academic/ThreeMonthCalendar.tsx | [4회 ⚠️] | selectionRange prop 추가 → 각 셀 prop 매핑. 지난 달 교습기간 헤더에 "🔒" 표기 |
| src/components/academic/StudyPeriodEditor.tsx | [1회] | 신규 — 모드 토글, 선택 상태 관리, 확정 AlertDialog, TanStack Query mutation, 한국어 에러 메시지 |
| src/app/academic/page.tsx | [1회] | StudyPeriodEditor 통합 — mode/selectionRange state 끌어올리기, 캘린더와 연결 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/` — 백엔드 IPC 그대로 (T5 createStudyPeriod / confirmStudyPeriod 재사용)
- [ ] `src/lib/tauri/index.ts` — T8 래퍼 그대로 사용
- [ ] `src/types/academic.ts` — T8 타입 그대로 사용
- [ ] `package.json` / `Cargo.toml` — 신규 의존성 없음

## 완료 기준 (이번 세션)

### T10 — 교습기간 설정 UI (PRD §4.4.2, sprint6.md L286-305)
- ✅ AC-T10-1: 시작일→종료일 순차 클릭, `normalizeRange` 가 start>end 자동 swap
- ✅ AC-T10-2: useMutation onError → AlertDialog 로 백엔드 한국어 에러 (예: "다른 교습기간과 일자가 중첩됩니다") 노출
- ✅ AC-T10-3: 확정 후 `invalidateQueries(['study-periods'])` → 자동 refetch → `bg-amber-50` 배경 (T9 그대로)
- ✅ AC-T10-4: 지난 달 교습기간 헤더 "🔒 수정 불가" + 백엔드 `update_study_period` 가드 (year_month < current_year_month)

### 추가 (T10 부수)
- ✅ 모드 토글 — "교습기간 설정" ↔ "취소"
- ✅ 선택 진행 상태 텍스트 — "시작일 클릭" / "시작일 X — 종료일 클릭" / "X ~ Y 선택됨"
- ✅ 선택 범위 시각 하이라이트 — bg-blue-100 + ring-2 ring-blue-500 (시작·종료)
- ✅ "취소" 버튼 — selection 초기화 + mode='view' 복귀
- ✅ "확정" 버튼 + controlled AlertDialog 확인 → createStudyPeriod + confirmStudyPeriod 순차 호출
- ✅ 성공 후 mode 종료 + selection 초기화 + 캐시 무효화로 자동 refetch

### 세션 종료 조건
- ✅ 단일 커밋 `bb767d6` (3 수정 + 1 신규, +278줄)
- ✅ Self-verify: tsc 통과 / lint clean / build 성공 (/academic 6.03kB / First Load 159kB)
- ✅ simplify 검토 — SelectionRange interface + normalizeRange/statusText 헬퍼 적절, 추상화 과잉 없음

## 설계 결정

### Mode 관리
- /academic page 가 `mode: 'view' | 'study-period-edit'` state 보유
- ThreeMonthCalendar 는 mode 자체를 알 필요 없음 — `selectionRange` prop 만 받음
- onCellClick 콜백을 부모가 mode 별로 다르게 주입 (view 모드는 핸들러 미주입 → 셀 클릭 무동작)

### 선택 상태 관리 (StudyPeriodEditor 내부)
- `selectionStart: string | null` / `selectionEnd: string | null`
- 첫 클릭: start 설정
- 두 번째 클릭: end 설정, start > end 면 swap
- "취소" → 둘 다 null + mode 'view' 로 복귀

### 확정 흐름
1. start + end 모두 있을 때 "확정" 버튼 활성화
2. AlertDialog 확인 ("YYYY-MM-DD ~ YYYY-MM-DD 교습기간을 등록합니다")
3. mutation:
   - createStudyPeriod({ year_month: start 의 'YYYY-MM', start_date, end_date })
   - 성공 후 confirmStudyPeriod(생성된 id) — PRD §4.4.2 확정 흐름
4. 성공 시: invalidateQueries(['study-periods']) + 모드 종료 + 토스트 (선택)
5. 실패 시: AlertDialog 로 백엔드 에러 메시지 노출 (한국어 그대로)

### year_month 결정
- 사용자가 시작일을 선택한 월 (start.slice(0,7)) 를 year_month 로 사용
- 즉, "2026-06-01 ~ 2026-06-30" → year_month = "2026-06"
- start 와 end 가 다른 월에 걸쳐도 (월 수업일수 20일 충족 위해 전후월 포함 허용) year_month 는 start 기준

### 지난 달 차단 시각화 (AC-T10-4)
- MonthGrid 헤더의 교습기간 표시 옆에 `isPastMonth ? '🔒 수정 불가' : ''` 추가
- 셀은 이미 T9 에서 `cursor-not-allowed + onClick disabled` — 모드 활성 시 시작일 클릭도 막힘
- 백엔드는 `update_study_period` / `delete_study_period` 가 IPC 레벨 차단 — 본 세션 UI는 시각화만

### 신규 의존성 없음
- AlertDialog: `src/components/ui/alert-dialog.tsx` 기존 사용 (@base-ui/react)
- TanStack Query: 기존
- 토스트: 본 세션은 AlertDialog 만 사용 (간소화, 필요 시 후속)

## 코드 패턴 SSOT

- 컴포넌트: `'use client'` 명시
- IPC: @/lib/tauri 추상화 레이어 (createStudyPeriod / confirmStudyPeriod / deleteStudyPeriod)
- TanStack Query mutation 패턴: useMutation + onSuccess(invalidate) + onError(에러 dialog)
- AlertDialog 구조: AlertDialog > AlertDialogContent > Header(Title+Description) + Footer(Cancel + Action)

## 발견된 이슈

> 진행 중 새 제약 발견 시 여기에 기록.
