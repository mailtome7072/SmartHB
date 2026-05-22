---
name: sprint-next-session
description: "Sprint 7 완료 (Task 10/10, 2026-05-22). 다음: /sprint-dev 8 → Sprint 8 (출결 관리 — Phase 2 나머지)"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint7-close
---

Sprint 7 완료. **브랜치 `sprint7` → `develop` 머지 완료** (--no-ff, 2026-05-22).
sprint-review 에이전트 실행 대기 중.

## Sprint 7 최종 현황

| 항목 | 내용 |
|------|------|
| 완료일 | 2026-05-22 |
| 세션 수 | 9 |
| Task | 10/10 완료 |
| cargo test | cipher off 177 passed / cipher on 127 passed |
| 버전 | v0.3.1 예정 (Sprint 6 + Sprint 7 통합, Unreleased) |

## Sprint 8 진입 시 우선 액션

1. sprint-review 에이전트 먼저 실행 (코드 리뷰 + 시각 검증 AC-T10-3 포함)
2. DEPLOY.md `⬜ sprint-review` + `⬜ tauri:dev 수동 확인` 완료 후 `/sprint-dev 8` 입력
3. Sprint 8: 출결 관리 (Phase 2 나머지) — Phase 2 마일스톤(M3)

## Sprint 8 핵심 작업 (참고)

- DB 마이그레이션 V005: regular_attendances + makeup_attendances 테이블
- 출결 생성 로직 (`generate_attendances`, `get_attendance_grid`)
- 출결표 UI (행×원생, 열×일자, 50명×31일 렌더링 1초 이내)
- 캘린더 라이브러리 ADR (FullCalendar vs React Big Calendar) — `docs/arch/adr-006-calendar-library.md`
- 수업 관리 캘린더 뷰 기초 (§4.6.1)

## Sprint 7 carry-over (sprint-review 또는 Sprint 8 이후)

- **I-S2-2 ~ I-S2-10** (9건): high-effort code review 잔여 — partial-NULL 손상, set_password 재진입 가드, CRED_CACHE static drop, check_auth_status 미-마이그레이션, test→Keychain 사이드이펙트, concurrent race, mutex poison, migration audit, 잡다 low
- **I-S4-1** (1건): CalendarCell hasHoliday/hasAssessment 비즈니스 식별
- **AC-T10-3 시각 검증**: `pnpm tauri:dev` UC-2 전체 흐름 — sprint-review 단계

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
