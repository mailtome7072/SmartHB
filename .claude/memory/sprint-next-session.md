---
name: sprint-next-session
description: "Sprint 10 구현 완료 (T1~T12, T5 폐기) — Phase 3 완결. 다음: sprint-close → sprint-review"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session13-t12
---

Sprint 10 (Phase 3 보강+소멸 완결 sprint) **구현 전부 완료**. 다음 단계는 마무리 — `sprint-close` → `sprint-review`.

## Sprint 10 현황 — 전 Task 완료

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1~T8 (백엔드) | dead code 정리 / V108 / 소멸 IPC+트리거 / 퇴교 IPC / 선행수업 / 캘린더 집계 IPC + ADR-006 | ✅ | — |
| T5 | 소멸 환원 IPC | ❌ 폐기 (사용자 정책) | — |
| T9 | 앱 시작 소멸 토스트 | ✅ | `b7b6fcb` |
| T10 | 퇴교 보강 UI | ✅ | `7de4dbb` |
| **T11** | **캘린더 뷰 UI (FullCalendar)** | ✅ | `2d8fdb3` |
| **T12** | **통합 검증** | ✅ | `8550966` |

## T11 결과 요약 (FullCalendar 캘린더 뷰)

| 영역 | 변동 |
|------|------|
| 의존성 | FullCalendar 6.1.20 5종 (core/react/daygrid/timegrid/interaction) — ADR-006 사전 승인 |
| 메뉴 | '수업 관리'(`/schedules`) 활성화 (사용자 결정 — 신규 /calendar 아님) |
| 타입 | `src/types/calendar.ts` (CalendarMonth/Day/Session, MakeupManagementStudent) |
| 래퍼 | `getCalendarData` + `getMakeupManagementData` |
| 페이지 | `src/app/schedules/page.tsx` — 캘린더/보강관리 2탭 |
| 컴포넌트 | `ClassCalendar`(dynamic ssr:false) / `StudentDetailPopup` / `MakeupManagementView` |
| PI-04 | 보강 일괄 진입점 없음 — 보강관리 뷰는 소멸 임박 순 목록 + '출결관리 이동' 버튼만 |

## T12 자동 검증 결과

- `cargo test --lib` cipher off **272 passed / 0 failed**
- `cargo clippy --lib -- -D warnings` cipher off clean
- `pnpm lint` / `pnpm tsc --noEmit` / `pnpm build`(static export 16/16) clean
- 마이그레이션 self-check (A39): V108 1:1 일치
- ✅ **cipher on 로컬 검증 완료** (Session #14) — Strawberry Perl 설치(`winget`) + 테스트 게이트 정합 후: `cargo build --features cipher` Finished, `cargo test --features cipher` 116 passed, clippy clean. 유일 실패는 기존 flaky 동시성 테스트(단독 통과)

## 다음 단계 진입 액션

> "sprint10 구현 완료했어. sprint-close 실행해줘."

sprint-close 완료 후:

> "sprint-review 실행해줘."

### sprint-close 인계 사항
1. ROADMAP.md **Phase 3 완료 표기** (Sprint 9~10 → ✅), 대시보드 진행률 갱신 (전체 17 스프린트 기준 — 현재 헤더가 15로 오기재됨, 정정 권장)
2. CHANGELOG.md 0.4.x 항목 추가 — 소멸 자동 전이, 퇴교 보강 처리, 수업 관리 캘린더 뷰
3. PR 단계 생략 — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])

### sprint-review 인계 사항
1. **cipher on**: 로컬 검증 완료 (Strawberry Perl 설치됨). cargo test --features cipher 컴파일 가능하도록 8개 모듈 테스트 게이트 정합 (`a3b4915`). cipher-off 동작 불변
2. **사용자 시각 검증 대기**: 캘린더 일/주/월 전환 + 원생 팝업 + 보강관리 강조 + 보강완료(emerald)/소멸(gray) 색 구분
3. **carry-over flaky**: `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` 병렬 시 가끔 실패 (이번 run 통과)
4. 산출물 경로: `docs/test-reports/sprint10-*.md`, `docs/sprint-retrospectives/sprint10-retrospective.md`

## Sprint 10 Capacity 실측

- 계획 40h
- 실측 누적: T1 1.5 + T2 1 + T1' 0.5 + T3 2.5 + T4 3 + T5폐기 0 + T6 2 + T7 1 + T8 3 + T9 1 + T10 2 + T11 ~4 + T12 ~1 = **약 22.5h** (버퍼 대폭 여유)

## 정책 (재확인)
- **PR 단계 생략** — sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
