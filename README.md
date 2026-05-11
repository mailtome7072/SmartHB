# SmartHB
정쌤의 스마트해법수학

---

## 이 템플릿 사용 방법

```
0단계   clone → GitHub 저장소 연결
1단계   ARCHITECTURE.md 작성 → /setup-project → SETUP.sh   (프로젝트 변수 설정 + 개발환경 초기화)
2~3단계 착수 전 체크리스트 (필수)                            (기획 → 인프라)
4단계   개발 환경 가이드 완성 (선택 — 스프린트 진행 중 작성 가능)
완료 후  Claude Code 실행 → /init → develop 브랜치 생성 → 스프린트 반복 → 프로덕션 배포
```

자세한 절차는 아래 **[프로젝트 착수 전 체크리스트]** 및 **[착수 완료 후: 첫 스프린트 시작]** 섹션을 참조하세요.

---

## 템플릿 저장소 구조는 다음과 같습니다.
```
project-root/                    # 프로젝트 루트(Root) 폴더

├── .gitignore                   # 특정 파일이나 디렉터리를 추적하지 않도록 지정하는 설정 파일
├── .env                         # 실제 환경 변수 파일 (개발자 개인 환경에서 작성)
├── .env.example                 # 샘플 환경 변수 파일 (협업용, 민감 값은 비워둠)
├── SETUP.sh                     # 루트에 추가된 개발 환경 초기화 스크립트
├── README.md                    # 프로젝트 개요 및 기본 안내
├── ARCHITECTURE.md              # 프로젝트 변수 레지스트리 및 아키텍처 개요
├── PRD.md                       # 제품 요구사항 정의 (로드맵의 기반)
├── ROADMAP.md                   # 프로젝트 로드맵 (루트에 위치)
├── CLAUDE.md                    # AI 협업 지시 문서
├── DEPLOY.md                    # 배포 후 수동 작업 목록
├── CHANGELOG.md                 # 버전별 변경 이력 관리
├── docker-compose.prod.yml      # 프로덕션 Docker Compose (/setup-project로 이미지명 자동 치환)
├── .claude/
│   ├── agents/                  # Claude 에이전트 정의
│   │   ├── prd-to-roadmap.md    # PRD → ROADMAP 변환 에이전트 (Opus)
│   │   ├── phase-planner.md     # 대규모 기능 Phase 설계 에이전트 (Opus)
│   │   ├── sprint-planner.md    # 스프린트 계획 수립 에이전트 (Opus)
│   │   ├── sprint-close.md      # 스프린트 마무리 에이전트 (Sonnet)
│   │   ├── sprint-review.md     # 코드 리뷰·검증·회고 에이전트 (Sonnet)
│   │   ├── deploy-prod.md       # 프로덕션 배포 에이전트 (Sonnet)
│   │   ├── hotfix-close.md      # 핫픽스 마무리 에이전트 (Sonnet)
│   │   └── agent-memory/        # 에이전트 영구 메모리 (버전 관리됨, 팀 공유)
│   │       ├── sprint-planner/
│   │       ├── prd-to-roadmap/
│   │       ├── phase-planner/
│   │       └── deploy-prod/
│   ├── commands/
│   │   ├── sprint-dev.md        # /sprint-dev 슬래시 커맨드 (구현 진입)
│   │   ├── restart.md           # /restart 슬래시 커맨드
│   │   └── setup-project.md     # /setup-project 슬래시 커맨드
│   ├── hooks/                   # Claude Code 훅 (도구 실행 전후 자동 실행)
│   │   ├── pretooluse-bash-guard.sh       # Bash 실행 전 위험 명령 6가지 차단
│   │   ├── posttooluse-code-validator.sh  # Edit/Write 후 .env 차단, syntax 검증, 시크릿 감지
│   │   ├── posttooluse-scope-tracker.sh   # 파일 수정 횟수 자동 집계 (3회 시 loop 경고)
│   │   ├── stop-doc-checker.sh            # 에이전트 종료 후 문서 누락 감지 + session-summary 생성
│   │   └── lib/
│   │       ├── doc-rules.json   # 에이전트별 필수 문서 검증 규칙
│   │       └── log-helper.sh    # 훅 이벤트 로그 기록 공통 함수
│   ├── rules/                   # 조건부 자동 로드 규칙
│   │   ├── sprint-workflow.md      # 모든 대화에 자동 적용 — 에이전트 순서, 브랜치 규칙
│   │   ├── harness-engineering.md  # 모든 대화에 자동 적용 — 5대 하네스 원칙
│   │   ├── backend.md              # app/backend/**/*.py 접근 시
│   │   ├── frontend.md             # app/frontend/**/*.ts,tsx 접근 시
│   │   └── notion.md               # "Notion/노션" 언급 또는 Notion MCP 사용 시
│   ├── skills/                  # Claude 스킬 정의
│   │   ├── karpathy-guidelines.md  # 개발 원칙 지침
│   │   ├── writing-plans.md        # 계획 작성 지침
│   │   ├── code-review.md          # 코드 리뷰 체크리스트
│   │   ├── test-checklist.md       # 검증 매트릭스 (Sprint/Hotfix/deploy-prod)
│   │   ├── retrospective.md        # 스프린트 회고 작성 지침
│   │   ├── simplify.md             # 모든 Task 완료 후 코드 단순화 1회 실행 (sprint-dev 자동 호출)
│   │   ├── systematic-debugging.md # 버그·오류 Task 자동 배정 — 5단계 근본 원인 분석
│   │   ├── brainstorming.md        # 설계 대안 비교 Task 자동 배정 — Weighted Matrix → SWOT → ADR
│   │   ├── loop-detection.md       # 루프 상태 감지·분석·보고 (3회 실패/수정 시 자동 배정)
│   │   └── harness-ci-gate.md      # 배포 전 Policy Gate 체크리스트 (deploy-prod, sprint-review 사용)
│   ├── settings.json            # Claude 권한·훅 설정
│   ├── logs/                    # 훅 이벤트 로그 (gitignored) — YYYY-MM-DD.log + session-summary.md
│   └── tmp/                     # 허가 플래그 임시 파일 (gitignored) — Forbidden Area 1회 허용
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml               # PR 체크 (pytest, Docker 빌드)
│   │   └── deploy.yml           # main merge 시 프로덕션 배포
│   └── PULL_REQUEST_TEMPLATE.md # PR 코드리뷰 템플릿

├── strategy/                    # 전략 지침 모음 폴더
│   ├── planning.md              # 계획 수립 지침
│   ├── branch-strategy.md       # 브랜치 운영 규칙
│   ├── code-review.md           # 코드 리뷰 지침
│   ├── testing.md               # 테스트 전략
│   ├── deployment.md            # 배포 전략
│   ├── risk-management.md       # 리스크 관리 지침
│   ├── documentation.md         # 문서화 규칙
│   └── retrospectives.md        # 회고 기본 지침

├── docs/                        # 프로젝트 실행 기록 및 산출물 폴더
│   ├── ci-policy.md             # CI/CD 정책 문서 (pnpm 설치 및 실행 규칙 포함)
│   ├── prompt-guide.md          # 작업 경로 선택 가이드 (경로별 프롬프트 예시)
│   ├── EXAMPLE-prd.md           # PRD 작성 완성형 예시 (TaskFlow 가상 프로젝트)
│   ├── code-review-checklist.md # 코드 리뷰 체크리스트
│   ├── arch/                    # ADR(Architecture Decision Records) — brainstorming 스킬이 생성
│   ├── harness-engineering/     # 하네스 엔지니어링 정책 문서 (deployment-policy, continuous-verification)
│   ├── risk-register/           # 프로젝트별 리스크 이력 저장 (폴더)
│   ├── phase/                   # Phase 설계 기록 — phase-planner agent가 생성 (폴더)
│   ├── sprint/                  # 스프린트 계획 기록 (폴더)
│   ├── sprint-retrospectives/   # 스프린트 회고 기록 — sprint-review agent가 생성 (폴더)
│   ├── test-reports/            # 테스트 실행 결과 저장 (폴더)
│   ├── deploy-history/          # 배포 이력 및 장애/롤백 기록 (폴더)
│   └── retrospectives/          # 장기/팀 회고 기록 — 분기별 또는 필요 시 수동 작성 (폴더)

├── app/                         # 애플리케이션 소스 코드 (개발 코드 전체)
│   ├── frontend/                # 프론트엔드 소스 코드 (React 등, 프로젝트 시작 시 생성)
│   └── backend/                 # 백엔드 소스 코드 (FastAPI 등, 프로젝트 시작 시 생성)
│       ├── ...                  # 메인 애플리케이션 코드
│       ├── tests/               # pytest 테스트 (ci.yml: app/backend/tests/)
│       └── requirements.txt     # Python 의존성 (ruff 포함 — pip install -r로 설치)

├── scripts/                     # 유틸리티 스크립트
│   └── hooks/
│       └── pre-commit           # Git pre-commit 훅 — Python syntax + ruff + 프론트엔드 lint
```

---

## 에이전트 설명
이 프로젝트는 7개의 특화된 Claude 에이전트를 포함합니다.

**핵심 흐름**: `prd-to-roadmap` → (`phase-planner`) → `sprint-planner` → `/sprint-dev` → `sprint-close` → `sprint-review` → `deploy-prod` / 긴급 수정: `hotfix-close`

> 괄호는 선택적 단계 (3스프린트 이상 또는 여러 모듈에 걸친 아키텍처 결정이 필요한 경우에만 사용)

### 1. prd-to-roadmap (Opus)
**트리거**: PRD 문서가 있을 때 ROADMAP.md 생성 시
PRD(제품 요구사항 문서)를 분석하여 Agile/스크럼 방법론에 기반한 ROADMAP.md를 자동 생성합니다.

### 2. phase-planner (Opus)
**트리거**: 3스프린트 이상의 대규모 기능 또는 여러 모듈에 걸친 아키텍처 결정이 필요한 경우 (sprint-planner 이전에 사용)
대규모 기능을 독립 배포 가능한 Phase 단위로 분할하고, 보안·성능·UX·인프라 관점에서 설계를 검토합니다. `docs/phase/phase{n}.md` 생성 후 sprint-planner로 핸드오프합니다.

> **판단 기준**: 단일 기능이 3스프린트 이상이거나, DB·API·UI를 동시에 새로 설계해야 하는 경우. 1~2스프린트 규모는 sprint-planner를 바로 사용.

### 3. sprint-planner (Opus)
**트리거**: 새 스프린트 계획 수립 시
ROADMAP.md를 분석하고 writing-plans 스킬을 참조하여 실행 가능한 스프린트 계획을 수립합니다. 리스크 식별 시 `docs/risk-register/`에 기록합니다.

### 4. sprint-close (Sonnet)
**트리거**: 스프린트 구현 완료 후
문서화 + PR 생성에 집중합니다:
1. ROADMAP.md 상태 업데이트 (`🔄 진행 중` → `✅ 완료`)
2. `develop` 브랜치로 PR 생성
3. CHANGELOG.md 업데이트
4. DEPLOY.md 업데이트 (⬜ 항목 초기 작성) + 기록 아카이빙
5. sprint-planner 메모리 업데이트

### 5. sprint-review (Sonnet)
**트리거**: sprint-close 완료 후 (이슈 수정 후 독립 재실행 가능)
코드 품질 검토 및 검증을 담당합니다:
1. 코드 리뷰 (보안/성능/품질 체크리스트)
2. 자동 검증 실행 (pytest, API curl, Playwright UI)
3. 테스트 결과 기록 (`docs/test-reports/`)
4. 리스크 기록 (`docs/risk-register/` — Medium/High 이슈 발견 시)
5. Sprint 회고 작성 (`docs/sprint-retrospectives/`)

### 6. deploy-prod (Sonnet)
**트리거**: develop 브랜치 QA 완료 후 프로덕션 배포 시
`develop` → `main` PR 생성, 사전 점검, 배포 후 실서버 검증을 수행합니다.

### 7. hotfix-close (Sonnet)
**트리거**: 핫픽스 구현 완료 후
sprint-close의 경량 버전. ROADMAP 업데이트 없이 `main` 브랜치로 PR을 생성하고, 머지 후 develop 역머지를 안내합니다. Medium/High 이슈 발견 시 `docs/risk-register/`에 기록합니다.

---

### Sprint 흐름은 아래와 같습니다.
```
1. sprint-planner → docs/sprint/sprint{n}.md 생성
2. /sprint-dev {n} → sprint{n} 브랜치 생성 + 구현 진입
3. 구현 작업...
4. sprint-close → develop PR + DEPLOY.md 초기화
5. sprint-review → 코드 리뷰 + 검증 + 회고 작성
6. QA 통과 후 deploy-prod → main 배포
```

### Hotfix 흐름은 아래와 같습니다.
```
1. `main` 기반으로 `hotfix/{설명}` 브랜치를 생성한다. (worktree 사용 금지)
2. 구현 작업
3. hotfix-close 에이전트 실행:
   > "hotfix 구현 끝났어. 마무리해줘."
4. GitHub Actions 자동 배포 → 배포 후 main을 develop에 역머지한다.
   > "main merge 완료됐어. develop 역머지 해줘."
```
자세한 내용은 `docs/dev-process.md` 참조.

> **에이전트 메모리**: `.claude/agents/agent-memory/` 디렉토리의 `MEMORY.md` 파일들은 에이전트가 세션 간 지식을 축적하는 데 사용됩니다. 초기에는 비어있으며, 각 에이전트 첫 실행 후 자동으로 채워집니다. 이 파일들은 버전 관리되므로 팀 전체가 공유합니다.

---

## 하네스 엔지니어링 원칙

AI 에이전트의 자율성을 보장하면서도 가드레일을 강제하는 **하네스(Harness) 엔지니어링** 원칙이 전체 워크플로우에 적용됩니다.

| 원칙 | 핵심 행동 | 구현 위치 |
|------|---------|---------|
| **1. Planning First** | 코드 수정 전 `scope.md` 작성 의무 | `sprint-dev` 0단계, `posttooluse-code-validator.sh` 경고 |
| **2. Strict Guardrails** | scope 외 파일·패키지·구조 변경 차단 | `posttooluse-code-validator.sh`, Forbidden Areas |
| **3. Verification Loops** | 3-retry, 동일 수정 반복 금지 | `posttooluse-scope-tracker.sh`, `loop-detection` 스킬 |
| **4. Policy Enforcement** | 배포 전 Policy Gate 통과 필수 | `harness-ci-gate` 스킬, `deploy-prod` 에이전트 |
| **5. Continuous Verification** | 배포 후 자동 검증 및 롤백 트리거 | `deploy-prod` 에이전트, `continuous-verification.md` |

**Forbidden Areas** (명시적 사용자 허가 없이 수정 차단):
- `.github/workflows/` — CI/CD 파이프라인
- `SETUP.sh` — 프로젝트 초기화 스크립트
- `docker/`, `docker-compose*.yml` — 컨테이너 인프라 설정
- `docs/harness-engineering/` — Harness 정책 문서

상세: `docs/harness-engineering/README.md` / `.claude/rules/harness-engineering.md`

---

## 슬래시 커맨드

| 커맨드 | 구분 | 설명 |
|--------|------|------|
| `/init` | Claude Code 내장 | 코드베이스 분석 후 CLAUDE.md 검토·갱신 — 첫 실행 시 및 프로젝트 구조 변경 후 사용 |
| `/setup-project` | 프로젝트 커스텀 | ARCHITECTURE.md 변수 → README.md, CLAUDE.md, PRD.md, docs/ci-policy.md, docker-compose.prod.yml 일괄 치환 |
| `/sprint-dev [n]` | 프로젝트 커스텀 | sprint{n}.md 기반 구현 오케스트레이터 — 브랜치 생성, 현황 파악, 가이드라인 주입 (**사용자가 직접 입력하는 커맨드** — 에이전트가 대신 호출하지 않음) |
| `/restart` | 프로젝트 커스텀 | Docker Compose 서비스 재시작 |

---

## 훅 시스템 및 로깅

### 자동 실행 훅 (`.claude/hooks/`)

| Hook 이벤트 | 파일 | 동작 |
|------------|------|------|
| **PreToolUse** (Bash) | `pretooluse-bash-guard.sh` | 디렉토리 체이닝, force push, main/develop 직접 push, hard reset, 브랜치 명명 위반 차단 |
| **PostToolUse** (Edit/Write) | `posttooluse-code-validator.sh` | `.env` 수정 차단, Forbidden Area 허가 검증, Python syntax, 시크릿 패턴 감지, Planning First 경고 |
| **PostToolUse** (Edit/Write) | `posttooluse-scope-tracker.sh` | `scope.md` 수정 횟수 자동 증가 — 3회 시 loop-detection 경고, 30% 초과 시 Re-planning 트리거 |
| **Stop** | `stop-doc-checker.sh` | 에이전트 유형별 필수 문서 누락 감지, `.claude/logs/session-summary.md` 갱신 |

### 로깅 시스템 (`.claude/logs/`, gitignored)

훅 이벤트는 자동으로 날짜별 로그 파일에 기록됩니다:

```
.claude/logs/
├── YYYY-MM-DD.log       # 훅 이벤트 로그 (타임스탬프|훅명|BLOCK/WARN|규칙ID|대상)
└── session-summary.md   # 세션 종료 시 자동 생성 — 다음 세션 재진입 시 참조
```

**다음 세션 재활용 흐름**: 세션 종료 → `session-summary.md` 자동 갱신 → 다음 `/sprint-dev` 0단계에서 읽어 반복 위반 패턴 파악 → `sprint-planner` 메모리에 기록 → 다음 스프린트 계획에 반영

## 문서 참고 체계
```
ARCHITECTURE.md (프로젝트 변수 레지스트리 — /setup-project 스킬의 입력)
CLAUDE.md (AI 협업 지시 — 빌드/테스트 명령어, 워크플로우 지침 포함)
  └→ strategy/*.md (전략 지침 계층)
  └→ .claude/rules/harness-engineering.md (하네스 원칙 — 전체 대화 자동 적용)
  └→ .claude/agents/*.md (Claude 에이전트 실행 로직)
  └→ .claude/skills/*.md (Claude 스킬 — 에이전트가 참조)
  └→ docs/dev-process.md (프로세스 상세)
      └→ docs/ci-policy.md (CI/CD 정책)
      └→ DEPLOY.md (배포 후 수동 작업)
```

**strategy/ 파일별 주요 참조 에이전트**

| strategy 파일 | 주로 참조하는 에이전트/스킬 |
|--------------|--------------------------|
| `planning.md` | sprint-planner, writing-plans 스킬 |
| `branch-strategy.md` | sprint-workflow 규칙 (전체 에이전트 공통) |
| `code-review.md` | sprint-review, hotfix-close (code-review 스킬 기준) |
| `testing.md` | sprint-review, hotfix-close (test-checklist 스킬 기준) |
| `deployment.md` | deploy-prod |
| `risk-management.md` | sprint-planner (risk-register), deploy-prod (버전 전략) |
| `documentation.md` | sprint-close (문서화 기준) |
| `retrospectives.md` | sprint-review (retrospective 스킬 기준) |

> **Notion MCP**: `.mcp.json`에 Notion HTTP MCP가 설정되어 있습니다. 사용 규칙: `.claude/rules/notion.md`

---

## 프로젝트 착수 전 수행할 항목 및 체크리스트

> **체크 방법**: 항목 완료 시 이 파일을 직접 편집하여 `⬜`를 `✅`로 변경하세요.

---

### 0단계: GitHub 저장소 연결

> 개발 착수할 프로젝트 폴더를 생성하신 후 생성한 프로젝트 루트 경로로 이동 하여 터미널에서 아래 프롬프트로 Clone 합니다. (최초 1회)
> `git clone https://github.com/mailtome7072/CLAUDESTARTER.git .`
> 템플릿 깃허브 주소 뒤 '.' 은  생성한 프로젝트 루트 경로에 파일이나 폴더가 없어야 오류가 안납니다. (지금 폴더로 클론 됨을 뜻함)
> GitHub 저장소 연결 : 아래 절차는 클론한 템플릿을 **새 프로젝트 GitHub 저장소에 연결하고 초기 커밋을 푸시**하는 단계입니다.

1. 수동으로 GitHub에서 새 저장소를 생성합니다 (빈 저장소, README 초기화 없이).
2. 터미널에서 프로젝트 루트로 이동합니다.

```bash
# 기존 Git 정보 완전 삭제 (clone 하기위해 연결한 템플릿 저장소 URL 연결 해제)
rm -rf .git

# 새 Git 저장소 초기화
git init

# 파일 추적 시작
git add .

# 첫 커밋 생성
git commit -m "Initial commit from template"

# 방금 위(1.)에서 만든 새 프로젝트 저장소로 Git 연결 (실제 org와 repo명으로 교체)
git remote add origin https://github.com/당신의깃허브아이디/새저장소이름.git

# 내용 올리기
git push -u origin main
```

---

### 1단계: 프로젝트 변수 설정 (/setup-project)

> GitHub 저장소 연결이 완료되면, 프로젝트 식별 정보를 한 번에 설정합니다.
> `ARCHITECTURE.md`를 열어 프로젝트 변수를 채운 뒤 `/setup-project`를 실행하면 `README.md`, `CLAUDE.md`, `PRD.md`, `docs/ci-policy.md`, `docker-compose.prod.yml`의 플레이스홀더가 일괄 치환됩니다. (`deploy.yml`은 `github.repository` 내장 변수를 사용하므로 치환 불필요)

- ⬜ `ARCHITECTURE.md` — **프로젝트 변수** 테이블의 5개 값 입력
  - `project_name`: 프로젝트 이름
  - `project_description`: 프로젝트 한 줄 설명
  - `github_org`: GitHub 조직 또는 계정명
  - `github_repo`: GitHub 저장소명
  - `decision_date`: PRD 작성 결정일 (예: 2026-03-24)
- ⬜ Claude Code 실행 → `/setup-project` → `README.md`, `CLAUDE.md`, `PRD.md`, `docs/ci-policy.md` 플레이스홀더 일괄 치환 확인
- ⬜ `./SETUP.sh` 실행 — 개발 환경 초기화 (pnpm, Python venv, .env 생성)

---

### 2단계: 기획 문서 작성 (PRD → ROADMAP)
2~4단계 체크리스트를 **순서대로 완료한 뒤** 첫 스프린트를 시작하세요.
각 단계의 ⬜ 항목은 착수 전 반드시 처리해야 할 작업입니다.

> 이 단계를 완료해야 `prd-to-roadmap` 에이전트로 로드맵을 생성할 수 있습니다.
> PRD.md 문서를 열고 아래 체크리스트의 내용을 개발하실 프로젝트 내용으로 작성해 주세요.
> 올바를 PRD가 작성되어야 올바른 ROADMAP을 만드실 수 있습니다.
> 작성 예시는 [`docs/EXAMPLE-prd.md`](docs/EXAMPLE-prd.md)를 참고하세요.

- ⬜ `PRD.md` — 상단 메타데이터 블록의 프로젝트명·버전·날짜·담당자 입력
- ⬜ `PRD.md` — 문제 정의, 목표 및 성공 지표 작성
- ⬜ `PRD.md` — 타겟 사용자 및 사용자 스토리 작성
- ⬜ `PRD.md` — 기술 스택 확정 및 기재 (프론트엔드/백엔드/DB/인프라)
- ⬜ `PRD.md` — 비기능 요구사항 (성능·보안·확장성) 작성
- ⬜ `PRD.md` — 기능별 인수 조건 (Acceptance Criteria) 정의
- ⬜ `PRD.md` — 도메인 용어 사전 작성
- ⬜ `PRD.md` 작성 완료 후 Claude Code에서 아래 프롬프트 입력 → `ROADMAP.md` 자동 생성
  > "PRD.md 작성 완료했어. ROADMAP 생성해줘."
- ⬜ 생성된 `ROADMAP.md` — 스프린트 목표·일정 검토 및 확정

---

### 3단계: 인프라 및 CI/CD 설정

> **항목 순서를 지켜주세요.** Docker 파일 생성 전에 ci.yml 빌드 스텝을 활성화하면 CI가 즉시 실패합니다.

- ⬜ `.env` — SETUP.sh가 생성한 `.env` 파일에 실제 값 입력 (DB 비밀번호, API 키 등)
- ⬜ GitHub Secrets 설정: `LIGHTSAIL_HOST`, `LIGHTSAIL_USER`, `LIGHTSAIL_SSH_KEY` (GHCR 인증은 `GITHUB_TOKEN` 자동 제공 — 별도 PAT 불필요)
  > 앱 레벨 시크릿(`POSTGRES_PASSWORD`, `JWT_SECRET`, `SECRET_KEY`, `NEXT_PUBLIC_API_URL`) 전체 목록: `docs/ci-policy.md` 참조
- ⬜ `docs/ci-policy.md` — 프로젝트 환경에 맞는 CI 정책 세부 내용 (브랜치명, 테스트 범위 등) 기입
- ⬜ `docs/dev-process.md` 섹션 6.3 — 실서버 SSH 접속 정보 기입 (호스팅 미정이면 생략)
- ⬜ Docker 파일 생성 (CI/CD 실행에 필수 — 아래 ci.yml 활성화의 전제조건):
  - `docker/backend/Dockerfile.prod` — 백엔드 프로덕션 이미지
  - `docker/frontend/Dockerfile.prod` — 프론트엔드 프로덕션 이미지
  - `docker/nginx/Dockerfile` — Nginx 리버스 프록시 이미지
  - `docker-compose.yml` — 로컬 개발 환경 (**첫 스프린트에서 앱 코드와 함께 작성** — sprint-planner가 Sprint 1 시 자동 태스크로 포함)
  - `docker-compose.prod.yml` — 프로덕션 환경 (**템플릿에 포함됨** — `/setup-project` 실행 시 이미지명 자동 치환)
- ⬜ `.github/workflows/ci.yml` — Docker 빌드 스텝 경로 확인 후 주석 해제 (Docker 파일 생성 후 진행)
- ⬜ `.github/workflows/ci.yml` — 프론트엔드 빌드·테스트 스텝 주석 해제 (**첫 스프린트에서 프론트엔드 코드 생성 후 진행**)

---

### 4단계: 개발 환경 가이드 완성 (선택)

> 팀원 온보딩을 위한 문서입니다. 첫 스프린트 시작의 전제조건이 아니므로 스프린트 진행 중에 작성해도 무방합니다.
> `docs/setup-guide.md`를 프로젝트 환경에 맞게 작성합니다.

- ⬜ `docs/setup-guide.md` — Node.js·Python 권장 버전 명시 (예: Node.js 20, Python 3.12)
- ⬜ `docs/setup-guide.md` — OS별 (macOS/Linux/Windows) 의존성 설치 명령어 작성
- ⬜ `docs/setup-guide.md` — 프로젝트 전용 환경 변수 항목 및 설명 추가
- ⬜ `docs/setup-guide.md` — IDE 추천 설정 및 확장 플러그인 가이드 작성
- ⬜ `docs/setup-guide.md` — 로컬 실행 검증 절차 (`docker compose up` 후 확인 방법) 기재

#### 주요 개발 명령어
```bash
# 프론트엔드 (pnpm)
pnpm install && pnpm build
pnpm test                                          # 전체 테스트
pnpm lint

# 백엔드 (Python/pytest + ruff)
source .venv/bin/activate
pytest app/backend/tests/                          # 전체 테스트
pytest app/backend/tests/test_foo.py::test_bar     # 단일 테스트 함수
ruff check app/backend/                            # Python 린트 (pre-commit 자동 실행)

# 로컬 스테이징 (Docker)
docker compose up --build
```

---

## 프로젝트 착수 전 수행할 항목 및 체크리스트 완료 후 : 첫 스프린트 시작

아래 순서로 개발을 시작합니다:

> **Claude Code에서 `/init`을 실행하세요.**
> 1단계의 `/setup-project`와는 별개로, 첫 스프린트 시작 직전에 프로젝트 구조와 빌드 명령어를 분석하여 CLAUDE.md를 최신 상태로 갱신합니다.

> **어떤 에이전트/커맨드를 써야 할지 모르겠다면** `docs/prompt-guide.md`를 참조하세요. 작업 유형별(새 기능, 긴급 패치, 배포 등) 경로와 핵심 프롬프트 예시가 정리되어 있습니다.

```
# 최초 1회
0. Claude Code 실행 → /init → CLAUDE.md 검토·갱신
1. git checkout -b develop && git push -u origin develop

# 스프린트마다 반복 (sprint-planner가 ROADMAP에서 다음 번호 자동 결정)
2. 아래 프롬프트 입력 → sprint-planner 에이전트 → docs/sprint/sprint{n}.md 생성
   > "ROADMAP 검토했어. sprint {n} 계획 세워줘."
3. 계획 확인 후 /sprint-dev {n} 입력 → sprint{n} 브랜치 자동 생성 + 구현 진입
4. 구현 작업
5. sprint-close 에이전트 → develop PR + DEPLOY.md 초기화
   > "sprint{n} 구현 완료했어. 마무리 작업 해줘."
6. sprint-review 에이전트 → 코드 리뷰 + 검증 + 회고 작성
   > "sprint-review 실행해줘."

# QA 통과 후 (복수 스프린트 묶어서 배포 가능)
7. deploy-prod 에이전트 → main 배포
   > "수동 검증 완료했고 develop QA 통과했어. 프로덕션 배포 준비해줘."
```

---

## 참고 문서
- `ARCHITECTURE.md` — 프로젝트 변수 레지스트리 및 아키텍처 개요
- `CLAUDE.md` — AI 협업 지침, 빌드/테스트 명령어, 워크플로우 지침
- `docs/prompt-guide.md` — 작업 경로 선택 가이드 (어떤 상황에 어떤 프롬프트를 쓸지)
- `docs/dev-process.md` — 개발 프로세스 전체 가이드
- `docs/ci-policy.md` — CI/CD 정책 상세
- `docs/setup-guide.md` — 환경 설정 가이드
- `ROADMAP.md` — 프로젝트 로드맵
- `PRD.md` — 제품 요구사항 정의
- `DEPLOY.md` — 배포 후 수동 작업 목록
- `CHANGELOG.md` — 버전별 변경 이력
- `docs/harness-engineering/README.md` — 하네스 엔지니어링 정책 개요 (5대 원칙 상세)
