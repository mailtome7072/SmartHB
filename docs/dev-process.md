# 개발 프로세스 가이드

> **프로세스 상세 가이드** — 검증 원칙, 개발 워크플로우, QA 기준의 상세 정의를 담습니다.
> CLAUDE.md가 최상위 AI 지침 문서이며, 이 문서는 CLAUDE.md에서 참조하는 프로세스 구현 세부사항을 제공합니다.
> 인프라 정책은 `docs/ci-policy.md`를 참조합니다.

---

## 1. Git 브랜치 전략

인프라 상세 정책은 [`docs/ci-policy.md`](ci-policy.md) 참조.

| 브랜치 | 역할 | 배포 환경 |
|--------|------|----------|
| `sprint{n}` | 스프린트 단위 개발 | 로컬 |
| `develop` | 스테이징 통합 브랜치 | 로컬 Docker |
| `main` | 프로덕션 브랜치 | 프로덕션 서버 |
| `hotfix/*` | 긴급 운영 패치 | main + develop 역머지 |

### Sprint 흐름

```
sprint{n}  →  PR to develop  →  로컬 Docker 스테이징 검증  →  PR to main  →  서버 자동 배포
```

### Hotfix 흐름

```
hotfix/*  →  PR to main  →  서버 자동 배포  →  main을 develop에 역머지
```

---

## 2. Hotfix vs Sprint 의사결정

### Hotfix 추천 기준 (모두 충족 시)

- 프로덕션 장애/버그
- 변경 범위: 파일 3개 이하 & 변경된 코드 50줄 이하 (`git diff main...HEAD` 추가(+)·삭제(-) 라인 합산 기준)
- DB 스키마 변경 없음
- 새 의존성(pip/npm) 추가 없음

### Sprint 추천 기준 (하나라도 해당 시)

- 새 기능 추가 또는 여러 모듈에 걸친 작업
- DB 스키마 변경 필요
- 새 의존성 추가 필요
- 파일 4개 이상 또는 코드 50줄 초과 변경

---

## 3. Sprint 프로세스

### 3.1 계획 (sprint-planner agent)

- ROADMAP.md를 참조하여 스프린트 번호와 목표를 확인
- `docs/sprint/sprint{n}.md` 계획 문서 생성
- karpathy-guidelines skill 및 writing-plans skill 준수

### 3.2 구현

- `/sprint-dev {n}` 커맨드로 구현 단계에 진입합니다 (브랜치 자동 생성 + 현황 파악).
- 브랜치는 반드시 `develop` 기반으로 생성합니다 (worktree 사용 금지):
  ```bash
  git checkout develop && git checkout -b sprint{n}
  ```

### 3.3 마무리 (sprint-close agent)

1. 현재 상태 파악 (브랜치, ROADMAP, DEPLOY.md 확인)
2. ROADMAP.md 상태 업데이트 (`🔄 진행 중` → `✅ 완료`)
3. sprint{n} → **develop** PR 생성 (main이 아닌 develop)
4. CHANGELOG.md 업데이트 (`[Unreleased]` 섹션에 변경사항 추가)
5. DEPLOY.md 업데이트: ⬜ 항목 초기 작성 (sprint-review 실행, docker compose up 포함) + 이전 기록 아카이빙
6. sprint-planner MEMORY.md 스프린트 현황 업데이트
7. 최종 보고 (PR URL, 다음 단계 안내)

> **다음 단계**: `sprint-review` 에이전트로 코드 리뷰·자동 검증·회고 작성을 실행합니다.
> **참고**: `develop` → `main` merge는 별도 QA 통과 후 deploy-prod agent를 사용합니다.

### 3.4 회고 (Sprint Retrospective)

상세 절차는 sprint-review agent를 참조합니다. sprint-review agent가 git 이력, 코드 리뷰 결과, 검증 결과를 종합하여 `docs/sprint-retrospectives/sprint{n}.md`를 자동 작성합니다.

---

## 4. Hotfix 프로세스

### 4.1 구현

- `main` 기반으로 `hotfix/{설명}` 브랜치 생성 (worktree 사용 금지)
- sprint-planner agent 사용하지 않음

### 4.2 마무리 (hotfix-close agent)

1. hotfix/* → **main** PR 생성
2. 변경 파일만 대상으로 경량 코드 리뷰
3. test-checklist skill의 "Hotfix" 컬럼 기준으로 타겟 검증 실행
4. `docs/deploy-history/YYYY-MM-DD.md`에 이전 기록 이동 후 DEPLOY.md 업데이트
5. develop 역머지 안내

---

## 5. 검증 매트릭스

> **SSOT**: `.claude/skills/test-checklist.md`
> Sprint / Hotfix / deploy-prod 컬럼별 검증 항목, 자동 검증 전제 조건, 결과 기록 규칙을 참조하세요.

---

## 6. 배포 프로세스

### 6.1 로컬 스테이징 (develop 브랜치 + Docker)

```bash
git pull origin develop
docker compose up --build
```

### 6.2 프로덕션 배포 (deploy-prod agent)

1. develop 브랜치 CI 통과 확인
2. develop → main PR 생성
3. GitHub Actions 자동 배포 (GHCR 이미지 빌드 → 서버 SSH 배포)
4. 실서버 자동 검증 (5단계: SSH 헬스체크, 컨테이너 상태, 로그, Playwright)

### 6.3 실서버 검증 (SSH 접속 정보)

> TODO: 프로젝트 배포 서버 정보를 이 섹션에 기입하세요.

- **키**: `{SSH_KEY_PATH}` (프로젝트 루트)
- **호스트**: `{USER}@{SERVER_IP}` (AWS Lightsail 또는 다른 서버)
- **앱 경로**: `{APP_PATH}`

```bash
# 헬스체크
curl -s http://{SERVER_IP}/api/v1/health

# 컨테이너 상태
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP} \
  "cd {APP_PATH} && sudo docker compose -f docker-compose.prod.yml ps"

# 백엔드 로그 오류 확인
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP} \
  "cd {APP_PATH} && sudo docker compose -f docker-compose.prod.yml logs backend --tail 30 2>&1 | grep -i 'error\|traceback\|critical' || echo 'No errors found'"
```

### 6.4 롤백 시나리오

#### A. 코드만 롤백 (이미지 태그 변경)

```bash
# 이전 버전 태그 확인
git log --oneline main -5

# 서버 SSH 접속 후
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP}
cd {APP_PATH}
sudo docker compose -f docker-compose.prod.yml down
sudo docker pull ghcr.io/{GITHUB_ORG}/{PROJECT}-backend:v{이전_버전}
sudo docker pull ghcr.io/{GITHUB_ORG}/{PROJECT}-frontend:v{이전_버전}
sudo docker compose -f docker-compose.prod.yml up -d
```

#### B. DB 포함 롤백 (주의: 데이터 손실 가능)

```bash
# 롤백 전 반드시 DB 백업
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP} \
  "cd {APP_PATH} && sudo docker compose -f docker-compose.prod.yml exec postgres pg_dump -U {DB_USER} {DB_NAME} > /tmp/backup_$(date +%Y%m%d).sql"

# Alembic 다운그레이드
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP} \
  "cd {APP_PATH} && sudo docker compose -f docker-compose.prod.yml exec backend alembic downgrade -1"
```

#### C. 긴급 서비스 중단

```bash
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP} \
  "cd {APP_PATH} && sudo docker compose -f docker-compose.prod.yml down"

# 원인 조사 후 서비스 복구
ssh -i {SSH_KEY_PATH} {USER}@{SERVER_IP} \
  "cd {APP_PATH} && sudo docker compose -f docker-compose.prod.yml up -d"
```

---

## 7. 코드 리뷰 체크리스트

> **SSOT**: `.claude/skills/code-review.md`
> 보안 / 성능 / 코드품질 / 테스트 / 패턴준수 체크리스트를 참조하세요.

---

## 8. 문서 관리 규칙

### 8.1 DEPLOY.md

- **목적**: 현재 배포 사이클의 검증 현황(자동 완료 ✅ + 수동 미완료 ⬜) 유지
- 다음 배포 시작 시 이전 배포 사이클 전체를 `docs/deploy-history/YYYY-MM-DD.md`로 이동
- 체크리스트는 GFM `[x]`/`[ ]` 대신 이모지(`✅`/`⬜`)를 사용합니다.

**편집 담당 (에이전트별 역할)**:
- `sprint-close`: ⬜ 항목 초기 작성 (sprint-review 실행, docker compose up 등) + 이전 기록 아카이빙
- `sprint-review`: 검증 결과를 ✅/❌로 업데이트 (자동 검증 완료 항목)
- `deploy-prod`: 배포 기록 추가 + 이전 배포 기록 아카이빙

### 8.2 docs/deploy-history/

- 날짜별 배포/검증 기록 아카이브
- 파일명: `YYYY-MM-DD.md` (해당 날짜의 모든 기록)

### 8.3 docs/setup-guide.md

- 초기 환경 설정 가이드 (외부 서비스 API, 개발 도구, 환경변수)
- 프로젝트 시작 시 1회 수행 항목

### 8.4 Sprint 문서

- 계획/완료 문서: `docs/sprint/sprint{n}.md`
- 첨부 파일 (스크린샷, 보고서): `docs/sprint/sprint{n}/`

### 8.5 Notion 업데이트 트리거

| 변경 유형 | 업데이트 페이지 |
|-----------|----------------|
| 새 버전 배포 | 릴리즈 노트 (최상단 추가) |
| DB 스키마 변경 | 데이터 모델 |
| API 변경/추가 | API 명세 |
| 새 기능 추가 | 기능 명세 |
| 아키텍처 변경 | 시스템 아키텍처 (Mermaid 다이어그램 포함) |

사용자가 지시할 때 업데이트합니다. sprint-review agent는 해당되는 Notion 페이지 업데이트 필요 여부를 최종 보고에서 안내합니다.

### 8.6 문서 최신화 트리거

| 변경 사항 | 업데이트 대상 | 담당 |
|-----------|--------------|------|
| 새 스프린트 완료 | `sprint-planner MEMORY.md`의 스프린트 현황 | sprint-close agent |
| 검증 매트릭스 변경 | `docs/dev-process.md` 섹션 5 | 직접 수정 |
| 환경변수/의존성 추가 | `docs/setup-guide.md` | 해당 스프린트 작업자 |
| 에이전트 워크플로우 변경 | `.claude/agents/*.md` 해당 파일 | 직접 수정 |
| 새 버전 배포 | Notion 릴리즈 노트 (섹션 8.5 참조) | deploy-prod agent |
| 스프린트 추가/완료 | `ROADMAP.md` 상태 업데이트 | sprint-close agent |
| DB/API/기능 변경 시 Notion | 섹션 8.5 트리거 참조 | sprint-review agent |

---

## 9. 문제 해결 가이드

### 9.1 CI 실패

**pytest 실패 시**
1. 로컬에서 `pytest app/backend/tests/ -v --tb=short` 실행하여 실패 케이스 확인
2. 실패 테스트를 개별 실행: `pytest app/backend/tests/test_foo.py::test_bar -v`
3. 수정 후 다시 push — CI가 자동 재실행됨

**pnpm test 실패 시**
1. 로컬에서 `pnpm test` 실행하여 실패 케이스 확인
2. 단일 파일 실행: `pnpm test -- --testPathPattern=<파일명>`
3. 수정 후 push

**pnpm lint 실패 시**
1. 로컬에서 `pnpm lint` 실행하여 오류 목록 확인
2. 대부분 자동 수정 가능: `pnpm lint --fix`
3. 수동 수정이 필요한 경우 오류 메시지의 파일:라인 위치 참조

### 9.2 Docker 빌드 실패

```bash
# 로컬에서 동일 환경 재현
docker compose up --build

# 이미지 캐시 완전 초기화 후 재시도
docker compose build --no-cache
```

- `requirements.txt` 또는 `package.json` 의존성 변경 후 빌드 실패 시 캐시 초기화 시도
- Dockerfile 경로 오류인 경우 `app/backend/Dockerfile` 또는 `app/frontend/Dockerfile` 확인

### 9.3 develop 브랜치 충돌

스프린트 병렬 진행 시 develop에 먼저 merge된 브랜치와 충돌이 발생할 수 있다.

```bash
# sprint{n} 브랜치에서 develop 최신 반영
git fetch origin
git merge origin/develop

# 충돌 파일 확인
git status

# 충돌 해결 후
git add <파일>
git commit
```

### 9.4 잘못된 브랜치에서 작업 시작한 경우

**sprint{n} 브랜치를 main 기반으로 생성한 경우** (develop 기반이어야 함):
```bash
# 현재 커밋을 스택에 보존
git stash

# 올바른 기반 브랜치로 재생성
git checkout develop
git pull origin develop
git checkout -b sprint{n}
git stash pop
```

**hotfix 브랜치를 develop 기반으로 생성한 경우** (main 기반이어야 함):
```bash
git stash
git checkout main
git pull origin main
git checkout -b hotfix/{설명}
git stash pop
```
