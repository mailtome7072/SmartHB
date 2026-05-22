---
name: sprint-next-session
description: "Sprint 5 완료(close+review) + Sprint 6 계획 수립까지 완료 (2026-05-22). 다음 액션: (선택) deploy-prod → 사용자가 /sprint-dev 6 입력"
metadata: 
  node_type: memory
  type: project
  originSessionId: ec4dbd04-fb9a-48c2-82eb-4dab55bbfa1a
---

Sprint 5(Phase 1.5b 안정화) **sprint-close + sprint-review 완료**, Sprint 6(Phase 2 학사 스케줄) **계획 수립까지 완료**. 마지막 develop 머지: `e717a78`, sprint-close 커밋: `c51c22d`, sprint-review 커밋: `e5b7480`. ROADMAP/CHANGELOG/DEPLOY 갱신 완료, develop = origin/develop 동기화 상태. Sprint 5 코드 리뷰: Critical 0 / High 0 / Medium 1 / Low 1.

**다음 액션 (두 가지)**
1. **(선택) deploy-prod 에이전트** — 누적 변경 배포. Sprint 3 종료 시점 `0.3.0` 미배포 상태였음. 현재 `package.json` 0.2.1 → 배포 시 버전 결정(v0.3.0 또는 v0.4.0) 필요.
2. **`/sprint-dev 6`** — **사용자가 직접 입력**해야 한다 (CLAUDE.md 정책: `/sprint-dev`는 에이전트 호출 금지). 입력 시 `sprint6` 브랜치 자동 생성 후 12개 Task 구현 진입.

**Why:** sprint-review까지 끝났고 sprint6.md (469줄, 12 Task) 도 sprint-planner agent가 이미 산출. 즉 계획 단계는 전부 종료 — 이제 배포 또는 구현 진입만 남았다.

**How to apply:** "다음 뭐하지?" 류 질문에는 1번(배포 의향 확인) → 2번(`/sprint-dev 6` 안내) 순서로 응대. 사용자가 sprint-planner 다시 실행하려 하면 막고 sprint6.md가 이미 있음을 알린다.

## Sprint 5 완료 산출물 (검증됨)

- `docs/sprint/sprint5.md` — DoD 전수 통과 기록
- `docs/sprint-retrospectives/sprint5-retrospective.md` — 회고 (이전 액션 아이템 A14~A18 이행 결과 포함)
- `docs/test-reports/sprint5-test-report.md` — 자동 검증 보고서
- `CHANGELOG.md` `[0.2.1]` 항목 — 환경 호환 + 다중 인스턴스 차단 + 시드 보정

## Sprint 6 계획 산출물 (sprint-planner 완료)

- `docs/sprint/sprint6.md` (469줄, 12 Task) — T1(A20 lock 재시도) / T2(V301 시드 보정 + 공휴일 시드) / T3(A21 flaky 테스트) / T4(A22 DnD) / T5~T7(교습기간/일정코드/배치 IPC) / T8(래퍼·타입) / T9~T11(캘린더·교습기간·배치 UI) / T12(통합검증)
- `docs/risk-register/2026-05-22.md` — R30(캘린더 복잡도) / R31(공휴일 API) / R32(OnceLock 리팩토링 부작용) 추가
- `.claude/agents/agent-memory/sprint-planner/MEMORY.md` — "다음 스프린트 번호: 7" 갱신
- `.claude/agents/agent-memory/sprint-planner/v102-seed-mismatch.md` — V102 시드와 PRD §4.4.4 불일치 발견 메모 (T2에서 V301로 보정)

## Sprint 5 결과 요약 (참고)

- **명명**: Phase 1.5b 안정화 (원래 Sprint 5 학사 스케줄 → Sprint 6 이연)
- **Task**: T0~T5 (6개) — 전수 통과
- **마이그레이션**: V201 (students.withdrawn_at + 시드 보정)
- **신규 의존성**: `tauri-plugin-single-instance`
- **핵심 변경**: Node 25/20 cross-OS 환경 호환 (T0), 다중 인스턴스 차단 (T1), LockPage 락 사전 체크 (T1-sub), 마법사 redirect `/` → `/settings` (T2), V201 시드 보정 (T3+T4), 검증 통과 (T5)

## Sprint 7 미리보기 (ROADMAP 기준)

- **출결 생성 + 출결표 UI + 상태 토글** — Sprint 6의 학사 스케줄 도메인에 의존

## 정책 (재확인)

- **PR 단계 생략** ([[workflow-no-pr]]) — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **Forbidden Area**: `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/`
- **새 의존성 추가 시 사용자 허가 필수**

본 메모리는 Sprint 6 sprint-close 시점에 다음 sprint-next-session으로 슬러그 갱신.
