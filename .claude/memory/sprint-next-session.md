---
name: sprint-next-session
description: "v0.3.1 배포 완료 (2026-05-23). 다음: /sprint-dev 8 → Sprint 8 (출결 관리 — Phase 2 나머지)"
metadata: 
  node_type: memory
  type: project
  originSessionId: deploy-prod-v0.3.1
---

v0.3.1 배포 완료 (2026-05-23). **develop → master 직접 머지 + v0.3.1 태그 push 완료**.

## v0.3.1 배포 현황

| 항목 | 내용 |
|------|------|
| 배포일 | 2026-05-23 |
| 버전 | v0.3.1 |
| 포함 스프린트 | Sprint 6 (학사 스케줄) + Sprint 7 (carry-over 해소 + 38건 fix) |
| master 머지 | 6bf7cd5 (--no-ff) |
| 태그 | v0.3.1 push 완료 → GitHub Actions CD 트리거 |
| cargo test | cipher off 187 passed / cipher on 127 passed |

## Sprint 8 진입 시 우선 액션

1. `/sprint-dev 8` 입력하여 Sprint 8 구현 시작
2. Sprint 8: 출결 관리 (Phase 2 나머지) — Phase 2 마일스톤(M3)
3. sprint-planner agent로 Sprint 8 계획 수립 먼저 (docs/sprint/sprint8.md)

## Sprint 8 핵심 작업 (참고)

- DB 마이그레이션: regular_attendances + makeup_attendances 테이블
- 출결 생성 로직 (`generate_attendances`, `get_attendance_grid`)
- 출결표 UI (행×원생, 열×일자, 50명×31일 렌더링 1초 이내)
- 캘린더 라이브러리 ADR (FullCalendar vs React Big Calendar) — `docs/arch/adr-006-calendar-library.md`
- 수업 관리 캘린더 뷰 기초 (§4.6.1)

## 후속 처리 필요 항목 (risk-register 등록됨)

- **R40~R44 (High)**: I-S2-2 ~ I-S2-6 — partial-NULL 손상, set_password 재진입 가드, CRED_CACHE static drop, check_auth_status 마이그레이션, test→Keychain 사이드이펙트
- **R45~R48 (Medium~High)**: concurrent race, mutex poison, migration audit, 잡다 low
- **R39**: StudyPeriodEditor create+confirm 원자성 (hotfix 후보)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **v0.3.1 인스톨러**: GitHub Release에서 Windows .msi / macOS .dmg 다운로드 가능 (Actions 완료 후)
