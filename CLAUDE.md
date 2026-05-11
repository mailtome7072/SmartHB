# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 목적
이 문서는 AI 협업 도구(Claude 등)가 프로젝트 문서를 작성·검수할 때 따라야 할 지침을 정의한다.
사람 팀원은 README.md, PRD.md, ROADMAP.md, CHANGELOG.md를 참고하고, AI는 CLAUDE.md를 참고하여 일관된 산출물을 생성한다.

## 아키텍처 개요

> 디렉토리 구조 상세는 `ARCHITECTURE.md` 참조. 아래에는 AI 협업에 필요한 핵심 흐름만 기술.

이 저장소는 **Claude Code 협업 스타터 템플릿**이다. 실제 앱 코드는 스프린트 진행 중에 추가된다.

**핵심 흐름**: PRD.md → ROADMAP.md → sprint{n} 브랜치 → develop PR → main 배포
**에이전트 역할** (상세는 `ARCHITECTURE.md` 참조):
- Opus 계열: `prd-to-roadmap`, `phase-planner`, `sprint-planner` — 계획/설계
- Sonnet 계열: `sprint-close`, `sprint-review`, `deploy-prod`, `hotfix-close` — 실행/검증
- 슬래시 커맨드: `/sprint-dev [n]` — 구현 단계 오케스트레이터

**에이전트 공유 메모리**: `.claude/agents/agent-memory/` — 세션 간 유지되는 에이전트별 메모리 파일 (버전 관리됨). 현재 메모리 보유 에이전트: `sprint-planner`, `prd-to-roadmap`, `phase-planner`, `deploy-prod`

## 하네스 엔지니어링 원칙

이 템플릿은 AI 에이전트의 자율성을 보장하면서도 가드레일을 강제하는 **하네스(Harness) 엔지니어링** 원칙을 따른다.

| 원칙 | 설명 | 구현 위치 |
|------|------|----------|
| **1. Planning First** | 코드 수정 전 scope.md 작성 | `sprint-dev` 0단계, `docs/sprint/sprint{n}/scope.md` |
| **2. Strict Guardrails** | 범위 외 파일/라이브러리/구조 변경 금지 | `posttooluse-code-validator.sh`, `harness-engineering.md` |
| **3. Verification Loops** | 3-retry 원칙, 동일 수정 반복 금지 | `sprint-dev` 검증 실패 대응, `harness-engineering.md` |
| **4. Policy Enforcement** | 배포 전 OPA 유사 게이트 통과 필수 | `harness-ci-gate` 스킬, `deployment-policy.md` |
| **5. Continuous Verification** | 배포 후 자동 검증 및 롤백 트리거 | `deploy-prod` 에이전트, `continuous-verification.md` |

상세: `docs/harness-engineering/README.md`

## 신규 클론 후 시작 순서

1. `ARCHITECTURE.md`의 5개 변수(`project_name`, `github_org` 등) 채우기
2. Claude Code에서 `/setup-project` 실행 → `README.md`, `CLAUDE.md`, `PRD.md`, `docs/ci-policy.md` 플레이스홀더 일괄 치환
3. `./SETUP.sh` 실행 — Node.js 확인, pnpm 설치, **Rust toolchain 확인/설치**, jq 설치, SQLx CLI/Tauri CLI 설치, `.env` 복사
4. `git checkout -b develop` (최초 1회 — 스프린트 기반 브랜치)

> `deploy.yml`: `v*` 태그 push 시 GitHub Actions가 Windows/macOS 인스톨러 자동 빌드·Release 첨부
> 상세 온보딩: `docs/setup-guide.md` 참조

## 빌드 및 테스트 명령어

### 사전 요구사항
- Node.js v20 이상
- Rust stable (rustup: https://rustup.rs)
- pnpm (`npm install -g pnpm`)
- jq (`winget install jqlang.jq` / `brew install jq`)
- SQLx CLI (`cargo install sqlx-cli --no-default-features --features sqlite`)
- Tauri CLI (`cargo install tauri-cli` 또는 devDependency로 설치)
- Windows: WebView2 런타임 (Windows 11 기본 포함)
- macOS: Xcode Command Line Tools (`xcode-select --install`)

### 개발 서버

```bash
pnpm tauri:dev       # Tauri 앱 + Next.js dev server 동시 기동 (권장)
pnpm dev             # Next.js dev server만 (브라우저 테스트용)
```

### 프론트엔드 (Next.js 15)

```bash
pnpm install         # 의존성 설치
pnpm build           # static export (out/ 디렉토리 생성)
pnpm lint            # ESLint
pnpm tsc --noEmit    # TypeScript 타입 검사
```

### 백엔드 (Rust/Tauri)

```bash
cargo build                                      # Rust 컴파일
cargo test                                       # 전체 Rust 단위 테스트 (src-tauri/ 기준)
cargo fmt --manifest-path src-tauri/Cargo.toml   # 코드 포맷
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings  # 정적 분석
```

### DB 마이그레이션 (SQLx, src-tauri/ 기준)

```bash
sqlx migrate run                 # 마이그레이션 적용
sqlx migrate add {설명}          # 새 마이그레이션 파일 생성 (src-tauri/migrations/)
sqlx migrate revert              # 마지막 마이그레이션 롤백
sqlx prepare                     # .sqlx/ 오프라인 캐시 갱신 (CI 용, 커밋 필수)
```

### 프로덕션 빌드 (인스톨러 생성)

```bash
pnpm tauri:build     # Windows .msi/.exe, macOS .dmg 생성
                     # 결과물: src-tauri/target/release/bundle/
```

## 저장소
- **원격 저장소**: https://github.com/mailtome7072/SmartHB.git
- **브랜치 전략**: strategy/branch-strategy.md 참고
- **문서 저장 위치**: docs/ 하위 폴더 (sprint, test-reports, deploy-history 등) — 구조 상세는 `strategy/documentation.md` 참조
- **배포 방식**: GitHub Releases — `v*` 태그 push 시 GitHub Actions가 Windows/macOS 인스톨러 자동 빌드·첨부

## 슬래시 커맨드

| 커맨드 | 구분 | 설명 | 정의 파일 |
|--------|------|------|----------|
| `/setup-project` | 프로젝트 커스텀 | `ARCHITECTURE.md` 변수 → `README.md`, `CLAUDE.md`, `PRD.md`, `docs/ci-policy.md`, `docker-compose.prod.yml` 플레이스홀더 일괄 치환 (`deploy.yml`은 `github.repository` 내장 변수 사용으로 치환 불필요) | `.claude/commands/setup-project.md` |
| `/sprint-dev [n]` | 프로젝트 커스텀 | `sprint{n}.md` 기반 구현 오케스트레이터 — 브랜치 생성, 현황 파악, 가이드라인 주입 (**사용자가 직접 입력하는 커맨드** — 에이전트가 대신 호출하지 않음) | `.claude/commands/sprint-dev.md` |
| `/restart` | 프로젝트 커스텀 | Docker Compose 서비스 재시작 | `.claude/commands/restart.md` |

## Hooks 시스템 (`.claude/hooks/`)

Claude Code가 도구 실행 전후로 자동으로 실행하는 검증 스크립트입니다.

| Hook | 파일 | 트리거 | 역할 |
|------|------|--------|------|
| PreToolUse | `pretooluse-bash-guard.sh` | Bash 도구 실행 전 | 위험 명령 6가지 패턴 차단 |
| PostToolUse | `posttooluse-code-validator.sh` | Edit/Write 도구 실행 후 | Python syntax 검증, `.env` 수정 차단, 시크릿 패턴 감지 |
| PostToolUse | `posttooluse-scope-tracker.sh` | Edit/Write 도구 실행 후 | scope.md 파일 수정 횟수 자동 증가, 3회 도달 시 loop-detection 경고 |
| Stop | `stop-doc-checker.sh` | 에이전트 응답 종료 후 | 에이전트별 문서 누락 자동 감지 |

**bash-guard 차단 규칙**: 디렉토리 체이닝(`cd /path &&`) / main·develop 직접 push / force push / `git reset --hard` / 허용되지 않는 브랜치 명명
**허용 브랜치 패턴**: `sprint{N}`, `sprint{N}-{설명}`, `hotfix/{설명}`

**doc-checker 감지 에이전트**: sprint-planner / sprint-close / sprint-review / hotfix-close / phase-planner / prd-to-roadmap
규칙 상세: `.claude/hooks/lib/doc-rules.json`

## 조건부 자동 로드 규칙 (`.claude/rules/`)

rules/ 파일은 조건에 따라 자동 로드됩니다. skills/는 에이전트/사용자가 명시적으로 참조할 때 로드됩니다.

| 파일 | 로드 조건 | 역할 |
|------|----------|------|
| `sprint-workflow.md` | 전체 대화 | 에이전트 사용 순서, 브랜치 규칙, Hotfix vs Sprint 판단 |
| `harness-engineering.md` | 전체 대화 | 5대 하네스 원칙, scope 선언 의무, step-back 프로토콜, 3-retry |
| `backend.md` | `src-tauri/**/*.rs` 등 접근 시 | Rust/Tauri/SQLx 개발 제약 (커맨드 구조, 마이그레이션, 보안) |
| `frontend.md` | `src/**/*.ts,tsx` 등 접근 시 | 프론트엔드 개발 제약 (TypeScript, Tauri IPC 추상화, XSS) |
| `notion.md` | "Notion/노션" 언급 또는 Notion MCP 사용 시 | Notion MCP 사용 규칙, 페이지 ID 매핑 |

> **MCP 서버**: `.mcp.json`에 Notion HTTP MCP(`https://mcp.notion.com/mcp`)가 설정되어 있다. Notion 연동 규칙은 `.claude/rules/notion.md` 참조.

### 내장 스킬 (`.claude/skills/`)

| 스킬 | 용도 |
|------|------|
| `karpathy-guidelines` | 코드 작성·수정 시 적용 원칙 |
| `simplify` | Task 완료 후 자동 실행 — 불필요한 추상화·중복·미사용 코드 제거 (`/sprint-dev` 내 모든 Task 완료 후 1회 호출) |
| `writing-plans` | 계획 문서 작성 형식·INVEST 기준 정의 (sprint-planner agent가 주로 참조하며, 직접 호출도 가능) |
| `code-review` | PR 코드 리뷰 체크리스트 |
| `test-checklist` | 테스트 보고서 작성 형식 |
| `retrospective` | 스프린트 회고 진행 형식 |
| `systematic-debugging` | 버그 근본 원인 파악 5단계 절차 (`/sprint-dev` 내 버그 Task에 자동 배정) |
| `brainstorming` | 설계 대안 비교(Weighted Matrix + SWOT) 및 ADR 작성 (`/sprint-dev` 내 설계 결정 Task에 자동 배정) — ADR 저장: `docs/arch/adr-{NNN}-{주제}.md` |
| `loop-detection` | 루프 상태 감지·분석·보고 프로토콜 (`/sprint-dev` 루프 감지 시 자동 배정 — 동일 테스트 3회 연속 실패 또는 동일 파일 3회 이상 수정 시) |
| `harness-ci-gate` | 배포 전 Policy Gate 체크리스트 — BLOCK/CONFIRM 조건 검증 (`deploy-prod`, `sprint-review` 에이전트 사용) |

## 환경 변수 관리 지시
- `.env` 파일은 프로젝트 루트에 위치하며, 각자 환경에서 작성한다.
- `.env` 파일은 절대 Git에 커밋하지 않는다 (`.gitignore`에 포함).
- `.env.example` 파일을 제공하여 필요한 변수 이름과 형식을 안내한다.
- 민감한 값(API 키, DB 접속 정보 등)은 사람이 직접 채운다.

## AI 협업 개발 원칙 (Karpathy)

> 상세 원칙은 `.claude/skills/karpathy-guidelines.md` 참조.

- 파일 수정 전 반드시 읽고 현재 상태를 파악한다.
- AI 생성 코드도 커밋 전 `git diff`로 의도치 않은 변경을 직접 확인한다.
- CI 실패 시 원인을 파악하고 수정한다 — `--no-verify` 우회 금지.
- 복잡한 작업은 추상화 계층 단위로 분해하여 요청한다 (DB 설계 + API + 프론트를 한 번에 요청 금지).
- AI 생성 코드도 `code-review` 스킬의 "AI 생성 코드 리뷰 추가 체크" 항목을 통과해야 한다.

## 언어 및 커뮤니케이션 규칙
- 기본 응답 언어: 한국어
- 코드 주석: 한국어로 작성
- 커밋 메시지: 한국어로 작성
- 문서화: 한국어로 작성
- 변수명/함수명: 영어 (코드 표준 준수)

## CI/CD 정책

> 파이프라인 기술 상세 (명령어, YAML 예시, 이미지 태그 규칙 등)는 [`docs/ci-policy.md`](docs/ci-policy.md) 참조.

모든 브랜치 전략은 `karpathy-guidelines` 스킬을 준수한다.

### Main 브랜치
- `develop` → `main` merge 후 `v{version}` 태그 push → GitHub Actions가 자동으로 Windows/macOS 인스톨러 빌드 및 GitHub Release 첨부
- deploy-prod agent가 태그 push 및 릴리즈 결과 확인을 담당한다.
- 📎 배포 절차: `docs/dev-process.md` 섹션 6.2 / Notion 업데이트: 섹션 8.5

**GitHub Secrets** (선택적):
- `TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD`: Tauri 자동 업데이트 서명용 (도입 시 설정)

> `GITHUB_TOKEN`은 자동 제공됨 — GitHub Release 생성에 별도 설정 불필요.

### Develop 브랜치
- `sprint{n}` → `develop` PR은 sprint-close agent가 생성한다.
- `develop` merge 후 로컬 Docker로 스테이징 검증한다. (`docker compose up --build`)
- GHCR push는 하지 않으며, 프로덕션 배포는 `main` merge 시에만 수행한다.
- 📎 검증 매트릭스: `docs/dev-process.md` 섹션 5 (Sprint 컬럼) / 스테이징 절차: 섹션 6.1

### Hotfix 브랜치
> Hotfix 추천 기준 SSOT: [`docs/dev-process.md`](docs/dev-process.md) 섹션 2
> 요건: 파일 3개 이하, 변경된 코드 50줄 이하 (diff 기준), DB 변경 없음, 새 의존성 없음

- `main` 기반으로 `hotfix/{설명}` 브랜치를 생성한다.
- sprint-planner agent는 사용하지 않는다.
- 구현 완료 후 hotfix-close agent를 사용하여 마무리한다 (PR to main, 경량 검증, `DEPLOY.md` 업데이트, develop 역머지 안내).
  > `DEPLOY.md`: 배포마다 리셋되는 수동 검증 체크리스트 — 완료 기록은 `docs/deploy-history/`에 아카이브.
- 프로덕션 배포는 main merge 시 GitHub Actions가 자동 수행한다.
- main merge 완료 후 `develop`에 역머지 필수 (hotfix 코드가 develop에 반영되지 않으면 다음 스프린트에서 충돌 발생):
  > "main merge 완료됐어. develop 역머지 해줘."
- 📎 검증 매트릭스: `docs/dev-process.md` 섹션 5 (Hotfix 컬럼) / 롤백: 섹션 6.4

## Bash 명령 실행 규칙

- Bash 명령 실행 시 `cd /path &&` 접두사를 사용하지 마세요. 작업 디렉토리가 이미 프로젝트 루트로 설정되어 있습니다.
- 특히 git 명령은 반드시 `git ...` 형태로 직접 실행하세요. (`cd ... && git ...` 금지)
- `.claude/settings.json`의 기본 허용 명령: `git *`, `pytest *`, `pnpm *`, `docker *`, `curl *`, `gh *`, `ssh -i *`. 기본 목록에 없는 명령이 필요하면 `.claude/settings.json`의 `permissions.allow`에 직접 추가하세요.

## 개발시 유의해야할 사항

1. **plan 모드에서 수정사항을 받으면 반드시 Hotfix vs Sprint 의사결정을 먼저 수행한다.**
  - 판단 기준: `docs/dev-process.md` 섹션 2 (SSOT) / `.claude/rules/sprint-workflow.md` (요약)
  - 사용자의 최종 결정을 받은 후 해당 프로세스를 따른다.

2. sprint 관련 문서 구조:
  - 스프린트 계획/완료 문서: `docs/sprint/sprint{n}.md`
  - 스프린트 첨부 파일 (스크린샷, 보고서 등): `docs/sprint/sprint{n}/`

3. sprint 개발이 plan 모드로 진행될 때는 다음을 꼭 준수합니다.
  - karpathy-guidelines skill을 준수한다.
  - 3스프린트 이상의 대규모 기능은 sprint-planner 이전에 phase-planner agent로 Phase 설계를 먼저 수행한다.
  - sprint-planner agent가 계획 수립 작업을 수행하도록 한다.
  - 계획 확인 후 `/sprint-dev {n}` 커맨드로 구현 단계에 진입한다. (브랜치 자동 생성)
  - 스프린트 구현 완료 후 **두 단계로** 마무리한다:
    1. sprint-close agent: 문서화 + PR 생성
    2. sprint-review agent: 코드 리뷰 + 자동 검증 + 회고
  - CI/배포 상세 절차는 위 CI/CD 정책을 참조한다.

4. hotfix 개발이 plan 모드로 진행될 때는 다음을 꼭 준수한다.
  - karpathy-guidelines skill을 준수한다.
  - `main` 기반으로 `hotfix/{설명}` 브랜치를 생성한다.
  - CI/배포 상세 절차는 위 CI/CD 정책 > Hotfix 브랜치를 참조한다.
  - 배포 후 실서버 검증이 필요하면 deploy-prod agent의 5단계(실서버 자동 검증)를 참조한다.

5. 검증 매트릭스 상세: `docs/dev-process.md` 섹션 5 참조
6. 배포 후 수동 작업: `DEPLOY.md` 참조 — 배포마다 리셋되는 일회성 체크리스트. 완료 기록은 `docs/deploy-history/`에 아카이브.
7. 체크리스트 작성 형식:
  - 완료 항목: `- ✅ 항목 내용`
  - 미완료 항목: `- ⬜ 항목 내용`
  - GFM `[x]`/`[ ]` 대신 이모지를 사용하여 마크다운 미리보기에서 시각적 구분을 보장한다.
  - 진행 상태 (ROADMAP.md 등): `📋 예정` → `🔄 진행 중` → `✅ 완료` / `⏸️ 보류`

## 문제 해결 참조

- **CI 실패** (cargo test, pnpm lint, pnpm build): `docs/dev-process.md` 섹션 9.1
- **Tauri 빌드 실패** (cargo error, WebView2 누락, Xcode CLI 미설치): `docs/dev-process.md` 섹션 9.2
- **develop 브랜치 충돌**: `docs/dev-process.md` 섹션 9.3
- **잘못된 브랜치에서 작업 시작** (sprint → develop 기반 재생성 등): `docs/dev-process.md` 섹션 9.4

## 워크플로우 지침

> **어떤 에이전트/커맨드를 써야 할지 모르겠다면** `docs/prompt-guide.md`를 참조한다. 작업 유형별(새 기능, 긴급 패치, 배포 등) 경로와 핵심 프롬프트 예시가 정리되어 있다.

각 워크플로우별 상세 포맷은 `strategy/` 하위 문서를 참조한다.

| 워크플로우 | 입력 | 출력 위치 | 전략 문서 |
|-----------|------|-----------|-----------|
| PRD → ROADMAP | PRD.md | ROADMAP.md | strategy/planning.md |
| Phase Planning | ROADMAP.md | docs/phase/phase{n}.md | phase-planner agent |
| Sprint Planning | ROADMAP.md | docs/sprint/sprint{n}.md | strategy/planning.md |
| Sprint Review | sprint{n}.md + git log | docs/test-reports/, docs/sprint-retrospectives/ | .claude/skills/code-review.md, test-checklist.md, retrospective.md |
| CHANGELOG | - | CHANGELOG.md | - |

> 기타 산출물(Test Report, Risk Register, Deployment Log, Code Review) 포맷은 `docs/prompt-guide.md` 참조.

**CHANGELOG 버전 표기**: `## [x.y.z] - YYYY-MM-DD` / 카테고리: Added / Changed / Fixed / Removed / 최신 버전은 최상단에 추가

모든 산출물은 Markdown 형식, 한국어로 작성하며 문서 간 연결 관계(PRD → ROADMAP → Sprint → Retrospective → Deployment → CHANGELOG)를 유지한다.
