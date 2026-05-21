---
name: workflow-no-pr
description: SmartHB 는 단일 개발자 단일 프로젝트라 PR 생성 단계 전부 생략 — sprint/hotfix 모두 직접 머지
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 47b6f808-becf-4e4e-aec5-9afbd7b80b18
---

SmartHB 프로젝트는 PR 생성·리뷰 단계를 **모두 생략**한다. sprint, hotfix, deploy 어느 단계에서도 `gh pr create` 호출 금지.

**Why:** 사용자가 2026-05-19 Sprint 1 마무리 시점에 "단일프로젝트이므로 이후 PR 단계는 모두 생략해줘" 라고 명시. 단일 개발자(원장 1인) 단일 저장소 운영이라 PR 게이트가 오버헤드.

**How to apply:**

1. **sprint-close / hotfix-close 에이전트**: PR 생성 단계 건너뛰고 push 명령만 안내. 또는 사용자가 직접 머지하도록 가이드.
2. **sprint-review 에이전트**: PR 없이 `sprint{n}` 브랜치 자체에서 코드 리뷰 + 자동 검증 + 회고 작성.
3. **deploy-prod 에이전트**: `develop → main` PR 생성 건너뛰고 직접 fast-forward 머지 + 태그 push.
4. **브랜치 머지 흐름** (PR 대체):
   - `sprint{n} → develop`: `git checkout develop && git merge --no-ff sprint{n} && git push origin develop`
   - `develop → main`: deploy-prod 가 동일 패턴으로 직접 머지 후 `v{version}` 태그 push
   - `hotfix/* → main`: hotfix-close 가 직접 머지, develop 역머지도 직접
5. **CI 게이트**: `.github/workflows/ci.yml` 의 PR 트리거(`on.pull_request`)는 의미 없어지므로 sprint-review 시 로컬 self-verify 결과(cargo test / clippy / pnpm lint / tsc)가 사실상의 게이트.
6. **`docs/dev-process.md` / `CLAUDE.md` 의 PR 절차 문서는 보존**: 향후 협업자 합류 시 활성화 가능한 옵션으로 남겨둠. 단, 에이전트 실행 흐름에서는 본 메모리 우선.

머지를 `--no-ff` 로 하는 이유: sprint 단위 커밋 그룹을 머지 커밋으로 묶어 `git log --first-parent` 조회 시 sprint 경계가 보이도록 함 (PR 머지 커밋의 효과를 직접 머지로 재현).
