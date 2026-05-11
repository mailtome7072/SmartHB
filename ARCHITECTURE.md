# Architecture.md

이 파일은 프로젝트의 **변수 레지스트리**이자 **아키텍처 개요**입니다.
템플릿을 clone한 후 아래 변수 테이블을 채우고, `/setup-project` 스킬을 실행하면 프로젝트 전체에 자동 적용됩니다.

---

## 프로젝트 변수

> **사용법**: 값을 채운 뒤 Claude Code에서 `/setup-project`를 실행하세요.
> `${repo_url}`과 `${ghcr_prefix}`는 아래 값으로 자동 조합됩니다.

| 변수 | 값 | 설명 |
|------|----|------|
| `project_name` | (여기에 입력) | 프로젝트 이름 (예: MyApp) |
| `project_description` | (여기에 입력) | 프로젝트 한 줄 설명 |
| `github_org` | (여기에 입력) | GitHub 조직 또는 계정명 (예: myorg) |
| `github_repo` | (여기에 입력) | GitHub 저장소명 (예: myapp) |
| `decision_date` | (여기에 입력) | PRD 작성 결정일 (예: 2026-03-24) |

### 자동 조합 변수 (직접 입력 불필요)

| 변수 | 조합 규칙 |
|------|----------|
| `repo_url` | `https://github.com/${github_org}/${github_repo}.git` |
| `ghcr_prefix` | `ghcr.io/${github_org}/${github_repo}` |

### 적용 대상 파일

| 파일 | 교체 항목 |
|------|----------|
| `README.md` | 제목(`project_name`), 설명(`project_description`), git remote URL(`github_org`, `github_repo`) |
| `CLAUDE.md` | 원격 저장소 URL(`repo_url`) |
| `PRD.md` | 작성일 메타데이터(`decision_date`) |
| `docs/ci-policy.md` | GHCR 이미지명(`github_org`, `github_repo`) |
| `docker-compose.prod.yml` | GHCR 이미지명(`github_org`, `github_repo`) |

> **`.github/workflows/deploy.yml`은 치환 불필요**: `github.repository` 내장 변수를 사용하므로 클론 직후 바로 동작합니다.

---

## 아키텍처 개요

```
project-root/
├── app/
│   ├── backend/         # FastAPI (Python) — app/backend/requirements.txt, app/backend/tests/
│   └── frontend/        # React (pnpm) — package.json은 루트 또는 app/frontend/에 위치
├── .claude/
│   ├── agents/          # Claude 에이전트 정의 (7개)
│   │   └── agent-memory/    # 세션 간 공유 에이전트 메모리 (버전 관리됨)
│   ├── commands/        # 슬래시 커맨드 (/setup-project, /sprint-dev, /restart)
│   ├── rules/           # 조건부 자동 로드 규칙 (sprint-workflow, backend, frontend, notion)
│   └── skills/          # Claude 스킬 정의 (명시적 호출용 — karpathy-guidelines, writing-plans 등)
├── strategy/            # 전략 지침 (브랜치, 테스트, 배포, 코드리뷰 등)
├── docs/                # 산출물 저장 (sprint/, phase/, deploy-history/, test-reports/ 등)
└── .github/workflows/   # ci.yml (PR 검증), deploy.yml (main → 프로덕션 배포)
```

**핵심 흐름**: `PRD.md` → `ROADMAP.md` → `sprint{n}` 브랜치 → `develop` PR → `main` 배포

**에이전트 역할** (Opus = 계획/설계, Sonnet = 실행/검증):
- `prd-to-roadmap` (Opus) — PRD 분석 → ROADMAP.md 자동 생성
- `phase-planner` (Opus) — 3스프린트+ 대규모 기능 Phase 설계 (선택적)
- `sprint-planner` (Opus) — ROADMAP 기반 스프린트 계획 수립
- `sprint-close` (Sonnet) — 스프린트 마무리: 문서화 + PR 생성
- `sprint-review` (Sonnet) — 스프린트 코드 리뷰 + 자동 검증 + 회고
- `deploy-prod` (Sonnet) — develop → main 프로덕션 배포
- `hotfix-close` (Sonnet) — 긴급 패치 마무리 (main 직접 배포)
