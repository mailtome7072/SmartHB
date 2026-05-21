---
name: sprint-next-session
description: "Sprint 4 (Phase 1.5 품질 안정화) 계획 확정 (2026-05-21) — 다음은 사용자가 `/sprint-dev 4` 입력"
metadata: 
  node_type: memory
  type: project
  originSessionId: 39e31ca0-8b92-444b-9a34-e09f9c9fb022
---

Sprint 3 + 5건의 post-sprint3 hotfix(`82eb1b2` `ddda06f` `15f7dc3` `ec7ffbf` `fb513b8` `08e9629` `951b074` `d9c8612` `275c55d`) 모두 develop 머지 + push 완료. 사용자 수동 스테이징 검증에서 14건 이슈(1 Critical Runtime + 13 기능) 발견 → Sprint 4 = Phase 1.5 품질 안정화로 계획 수립 완료 (`docs/sprint/sprint4.md`).

**다음 액션: 사용자가 `/sprint-dev 4` 입력하여 구현 단계 진입.**

**Why:** Sprint 4는 사용자 보고 14개 이슈 전수 해소 + 교습소 설정 화면(§4.0) 신설 + DB 마이그레이션 V201(students.withdrawn_at). 마법사·원생 관리·코드 테이블·수업 스케줄 전반의 품질 확립. 사용자 명시 의도 "놓치거나 임의로 누락시키면 안됨" — sprint4.md 의 이슈 매트릭스(14/11 매핑) 으로 검증.

**How to apply:** Sprint 4 관련 요청 받으면 먼저 `docs/sprint/sprint4.md` 의 Task 매트릭스(이슈 #0~#13 + Critical)를 참조해 현재 Task 위치 파악. CLAUDE.md 정책상 `/sprint-dev` 는 **사용자 직접 입력** — 에이전트가 대신 호출 금지.

## Sprint 4 핵심 (sprint4.md 요약)

- **명명**: Phase 1.5 품질 안정화 (원래 Sprint 4 학사 스케줄은 Sprint 5 로 이연)
- **Task**: T1~T11 (11개)
- **마이그레이션 번호 예약**: V201 (`students.withdrawn_at DATE NULL`)
- **신규 의존성 (사용자 허가 완료)**: `@dnd-kit/core` + `@dnd-kit/sortable` (T10 DnD)
- **조건부 도입**: shadcn/ui AlertDialog — `components.json` 부재 시 `npx shadcn@latest init` 선행 (T1)
- **회고 carry-over 통합**: A9(dialog:allow-open 좁히기) → T1, A11(window.confirm → shadcn) → T1
- **이연**: A7(이미 해소), A8(salt.bin), A10(Undo), A12(cipher 실측), A13(simplify 보완)

## 신규 리스크 (`docs/risk-register/2026-05-21.md`)

- R23: shadcn/ui 초기 설정 미완료 — T1 착수 시 `components.json` 확인
- R24: dnd-kit × React 19 호환성 미확인 — T10 착수 전 검증
- R25: 상태바 IPC 미연결 — systematic-debugging 으로 추적

## Sprint 3 종료 상태 (참고)

- 버전 `0.3.0` 미배포 (master = v0.2.0). Sprint 4 완료 후 v0.3.1 또는 v0.4.0 으로 배포 검토.
- cargo test 123 passed, dev 마법사 인증 통과 확인 (`[startup] total=5448ms` — dev unoptimized PBKDF2 비용, release 에서 해소 예정)
- 진단 인프라(`d9c8612`): `error.rs::From<AppError> for String` 가 stderr 에 raw 메시지 보존 + `auth.rs` 단계별 hex 진단 로그

## 정책 (재확인)

- **PR 단계 생략** ([[workflow-no-pr]])
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **Forbidden Area**: `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/`
- **새 의존성 추가 시 사용자 허가 필수** — 본 sprint 는 dnd-kit 2개 허가 완료

본 메모리는 Sprint 4 종료 시 다음 sprint-next-session 으로 슬러그 갱신.
