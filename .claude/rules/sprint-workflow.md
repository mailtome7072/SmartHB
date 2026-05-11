---
description: 스프린트/핫픽스 워크플로우 핵심 규칙. 모든 대화에 자동 적용.
---

## 에이전트 사용 순서 (필수 준수)

스프린트 관련 작업은 반드시 아래 순서를 따른다:

1. **(선택) 대규모 기능 설계**: 3스프린트 이상의 기능은 `phase-planner` agent 먼저 사용
2. **계획**: `sprint-planner` agent → `docs/sprint/sprint{n}.md` 생성
3. **구현 진입**: `/sprint-dev {n}` 커맨드로 구현 단계 진입
4. **마무리 (두 단계로 분리)**:
   - `sprint-close` agent: ROADMAP 업데이트 + PR 생성 + 문서화
   - `sprint-review` agent: 코드 리뷰 + 자동 검증 + 회고 작성
5. **배포**: develop QA 통과 후 `deploy-prod` agent

> sprint-close와 sprint-review는 반드시 이 순서로 실행한다. sprint-review는 sprint-close 완료 후 단독 재실행도 가능하다.

## 브랜치 규칙 (자동 강제)

- `sprint{n}` 브랜치는 반드시 `develop` 기반으로 생성 (`git checkout develop && git checkout -b sprint{n}`)
- `main` 직접 push 금지 (PR + 리뷰 필수)
- worktree 사용 금지 — `git checkout -b` 방식만 허용

## Hotfix vs Sprint 판단 (plan 모드에서 필수)

수정 요청 수신 시 아래 기준으로 의사결정을 먼저 수행하고 사용자 확인을 받는다:

> **SSOT**: 상세 기준 및 경계 케이스는 `docs/dev-process.md` 섹션 2를 참조한다. 아래는 빠른 참조용 요약이다.

**Hotfix 조건** (모두 만족해야 함):
- 프로덕션 긴급 이슈
- 파일 3개 이하, 변경된 코드 50줄 이하 (`git diff main...HEAD` 추가(+)·삭제(-) 라인 합산 기준)
- DB 변경 없음, 새 의존성 없음

**Hotfix 브랜치 명명**: `hotfix/{설명}` — 예: `hotfix/fix-login-crash`, `hotfix/db-connection-timeout`, `hotfix/null-pointer-api`

**Hotfix 브랜치 생성** (반드시 `main` 기반):
```bash
git checkout main && git pull origin main
git checkout -b hotfix/{설명}
```
> ⚠️ `develop` 기반으로 hotfix 브랜치를 만들면 역머지 시 다음 스프린트와 충돌 발생

**Sprint 조건** (하나라도 해당 시):
- 새 기능 추가, 여러 모듈 수정, DB 변경, 새 의존성, 파일 4개 이상, 코드 50줄 초과

상세 기준: `docs/dev-process.md` 섹션 2

## 문서 동기화 규칙

코드 변경 시 관련 `docs/`를 함께 업데이트한다.
상세 원칙: `.claude/skills/karpathy-guidelines.md` 참조
