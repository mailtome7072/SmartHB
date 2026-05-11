# 코드 리뷰 지침

> **역할**: 코드 리뷰의 원칙과 기준을 정의한다.
> **SSOT**: 코드 리뷰 체크리스트는 [`.claude/skills/code-review.md`](../.claude/skills/code-review.md)가 단일 소스(SSOT)다.

---

## 코드 리뷰 원칙

- sprint-review agent의 2단계, hotfix-close agent의 3단계에서 체크리스트를 실행한다.
- 보안 → 성능 → 코드 품질 → 테스트 → 패턴 준수 순서로 검토한다.
- 체크리스트 완료 후 결과를 `docs/test-reports/YYYY-MM-DD.md`에 기록하고, DEPLOY.md의 해당 항목을 ✅로 업데이트한다.

## 참조

- 코드 리뷰 체크리스트 (SSOT): [`.claude/skills/code-review.md`](../.claude/skills/code-review.md)
- sprint-review agent: [`.claude/agents/sprint-review.md`](../.claude/agents/sprint-review.md)
- hotfix-close agent: [`.claude/agents/hotfix-close.md`](../.claude/agents/hotfix-close.md)
