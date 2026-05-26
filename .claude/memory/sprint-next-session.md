---
name: sprint-next-session
description: "Sprint 10 Session #8 완료 (T7 선행 수업 검증). 다음: T8 — 캘린더 ADR (PI-03) + 백엔드 집계 IPC"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session8-t7
---

Sprint 10 — Phase 3 완결 sprint. Session #8 (T7 선행 수업) 완료. 다음은 **T8 캘린더 ADR + 집계 IPC** (PI-03 사용자 결정 필요).

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 | ✅ | `b4565d4` |
| T1' | V108 CHECK 단순화 | ✅ | `1efd70f` |
| T3 | 소멸 자동 전이 IPC | ✅ | `616021d` |
| T4 | 트리거 3개소 통합 | ✅ | `6b6cc47` |
| T5 | 환원 IPC | ❌ 폐기 | `1762e71` |
| T6 | 퇴교 보강 IPC | ✅ | `6209d00` |
| T7 | 선행 수업 검증 | ✅ | `840e9c7` |
| **T8** | **캘린더 ADR (PI-03) + 백엔드 집계 IPC** | ⬜ 다음 세션 | — |
| T9 (축소) | 소멸 알림 UI (앱 시작 토스트만 잔여) | ⬜ | — |
| T10 | 퇴교 보강 UI | ⬜ | — |
| T11 | 캘린더 뷰 UI | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T7 결과 요약

| 영역 | 변동 |
|------|------|
| 코드 분석 | 백엔드 `create_makeup_with_absences_impl` 가 PRD §4.2.3 시나리오 자연스럽게 지원 (보강일<결석일 순서 검증 없음, 의도) |
| 단위 테스트 | `create_makeup_supports_future_absence_for_advance_class` — 보강일(6/13) < 결석일(6/20) → makeup_done 전이 + makeup_attendance_id 연결 검증 |
| 운영 흐름 문서화 | scope.md Session #8 — generate → 셀 토글 → 선행 보강 등록 3-step |
| UI 제약 (이연) | MakeupRegisterDialog::filteredPending 은 보강일 이전 결석만 표시. 사용자 운영 시작 시 시각 검증 후 별도 task 에서 완화 검토 |
| 자동 검증 | cargo test 266 passed (T6 265 → +1) / clippy clean |

## T8 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T8 작업 계획 (예상 4h, sprint10.md L217-238)

1. **PI-03 ADR 작성** (사용자 결정 필요): FullCalendar vs React Big Calendar
   - 비교 기준: 라이선스 (MIT vs 상용), 번들 크기, 일/주/월 뷰 지원도, 커스텀 렌더러, TS 지원, Tauri static export 호환성
   - `docs/arch/adr-{NNN}-calendar-library.md` 저장
   - skill: brainstorming

2. **캘린더 데이터 집계 IPC** — `get_calendar_data(year_month)`
   - 일별 시간대별 수업 원생 목록 (원생명, 시작/종료 시간, 정규/보강 구분)
   - AC-4.6-1: 시간대별 인원 = 시작 원생 + 진행 중 원생 합산

3. **보강 관리 뷰 IPC** — `get_makeup_management_data(year_month)`
   - 보강 필요 원생 리스트 (소멸기한 임박 순)
   - 소멸 임박 판정: 교습기간 종료일 - 7일 이내

4. IPC 등록 + TS 래퍼

5. 단위 테스트 4건+

### PI-03 사용자 결정 필요

| 옵션 | 라이선스 | 번들 크기 | 커뮤니티/문서 | 비고 |
|------|---------|----------|---------------|------|
| FullCalendar | MIT (premium 기능은 상용) | 크다 (~150KB+) | 풍부 | 일/주/월 뷰 표준, 커스텀 렌더러 강력 |
| React Big Calendar | MIT | 중간 (~80KB) | 보통 | 가벼움, 커스터마이징 자유도 높음 |

→ Tauri static export 호환성 둘 다 확인 필요 (dynamic import + 'use client').

## Sprint 10 Capacity 추적

- 계획 40h (T5 폐기 반영)
- 실측 누적: T1 1.5 + T2 1 + T1' 0.5 + T3 2.5 + T4 3 + T5폐기 0.5 + T6 2 + T7 1 = **12h**
- 남은 capacity: 약 28h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **PI-03 사용자 결정 필요** — T8 진입 시 (FullCalendar vs React Big Calendar)
