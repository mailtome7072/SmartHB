# 브랜치 운영 규칙

> **역할**: 브랜치 전략의 의도와 원칙을 정의한다.
> **SSOT**: 프로세스 상세는 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 1, 인프라 정책은 [`docs/ci-policy.md`](../docs/ci-policy.md)를 참조한다.

---

## 브랜치 구조 요약

| 브랜치 | 역할 | 배포 환경 |
|--------|------|----------|
| `sprint{n}` | 스프린트 단위 개발 | 로컬 |
| `develop` | 스테이징 통합 브랜치 | 로컬 Docker |
| `main` | 프로덕션 브랜치 | 프로덕션 서버 |
| `hotfix/*` | 긴급 운영 패치 | main + develop 역머지 |

## 핵심 원칙

- `main` 직접 push 금지 — 반드시 PR + 리뷰 후 merge
- worktree 사용 금지 — `git checkout -b` 방식만 사용
- 모든 브랜치 전략은 karpathy-guidelines skill 준수

## 참조

- Git 브랜치 전략 상세: [`docs/dev-process.md`](../docs/dev-process.md) 섹션 1
- CI/CD 인프라 정책: [`docs/ci-policy.md`](../docs/ci-policy.md)
- CLAUDE.md CI/CD 정책: [`CLAUDE.md`](../CLAUDE.md#cicd-정책)
