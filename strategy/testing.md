# 테스트 전략

> **역할**: 테스트 레벨, 도구, 통과 기준을 정의한다.
> **SSOT**: 검증 항목 및 자동화 기준은 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 5(검증 매트릭스)가 단일 소스(SSOT)다.

---

## 테스트 레벨

| 레벨 | 도구 | 경로 | 실행 시점 |
|------|------|------|----------|
| 백엔드 단위/통합 테스트 | `pytest` | `backend/tests/` | CI (PR 체크) |
| 프론트엔드 테스트 | `pnpm test` | (프로젝트별 설정) | CI (PR 체크) |
| API 검증 | `curl` / `httpx` | — | sprint-review / hotfix-close |
| UI E2E 테스트 | Playwright | — | sprint-review / hotfix-close / deploy-prod |

## 통과 기준

- pytest 전체 통과 필수 (PR merge 조건)
- pnpm test 전체 통과 필수 (PR merge 조건)
- Docker 이미지 빌드 성공 (PR merge 조건)

## 테스트 결과 기록

- 산출물: `docs/test-reports/YYYY-MM-DD.md`

## 참조

- 검증 매트릭스 (SSOT): [`docs/dev-process.md`](../docs/dev-process.md) 섹션 5
- CI 파이프라인: [`docs/ci-policy.md`](../docs/ci-policy.md) CI 파이프라인 섹션
- CI 워크플로우: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)
