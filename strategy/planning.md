# 계획 수립 지침

> **역할**: 스프린트 계획 수립 기준과 ROADMAP → Sprint 변환 절차를 정의한다.
> **SSOT**: 프로세스 상세는 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 3을 참조한다.

---

## 스프린트 계획 원칙

- ROADMAP.md의 Phase/Sprint 목표를 기반으로 스프린트 번호와 작업 범위를 확정한다.
- sprint-planner agent가 이 파일과 ROADMAP.md를 입력으로 받아 `docs/sprint/sprint{n}.md`를 생성한다.
- karpathy-guidelines skill 및 writing-plans skill을 준수한다.

## 산출물

- **입력**: `ROADMAP.md`, `PRD.md`
- **출력**: `docs/sprint/sprint{n}.md`

## 참조

- 스프린트 프로세스 상세: [`docs/dev-process.md`](../docs/dev-process.md) 섹션 3
- Hotfix vs Sprint 의사결정: [`docs/dev-process.md`](../docs/dev-process.md) 섹션 2
- sprint-planner agent: [`.claude/agents/sprint-planner.md`](../.claude/agents/sprint-planner.md)
