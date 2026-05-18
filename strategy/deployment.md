# 배포 전략

> **역할**: 배포 흐름, 환경, 롤백 전략의 원칙을 정의한다.
> **SSOT**: 배포 프로세스 상세는 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 6, CI/CD 인프라는 [`docs/ci-policy.md`](../docs/ci-policy.md)를 참조한다.

---

## 배포 환경

| 환경 | 브랜치 | 방법 |
|------|--------|------|
| 로컬 스테이징 | `develop` | `pnpm tauri:dev` |
| 프로덕션 | `main` | `v*` 태그 push → GitHub Actions → GitHub Releases (Windows/macOS 인스톨러) |

## 배포 원칙

- develop → main PR은 QA 통과 후 deploy-prod agent를 통해서만 수행한다.
- Semantic Versioning(`MAJOR.MINOR.PATCH`) 태그를 생성한다.
- 배포 후 수동 작업은 `DEPLOY.md`에 기록하고 완료 후 `docs/deploy-history/`에 아카이브한다.

## 산출물

- 현재 배포 수동 작업: `DEPLOY.md`
- 배포 이력 아카이브: `docs/deploy-history/YYYY-MM-DD.md`

## 참조

- 배포 프로세스 상세: [`docs/dev-process.md`](../docs/dev-process.md) 섹션 6
- CI/CD 인프라 정책: [`docs/ci-policy.md`](../docs/ci-policy.md)
- deploy-prod agent: [`.claude/agents/deploy-prod.md`](../.claude/agents/deploy-prod.md)
