# CI/CD 정책

---

## 환경 준비

### 런타임 버전
- **Node.js**: 20 (`actions/setup-node@v4`)
- **Python**: 3.12 (`actions/setup-python@v5`)
- **pnpm**: `npm install -g pnpm`으로 설치

### pnpm 설치 적용
- 프론트엔드 CI/CD 파이프라인은 pnpm 설치 단계를 포함해야 합니다.
- 설치 방식:

  ```yaml
  - name: Install pnpm
    run: npm install -g pnpm
  - name: Install dependencies
    run: pnpm install
  - name: Build
    run: pnpm build
  - name: Test
    run: pnpm test
  - name: Lint
    run: pnpm lint
  ```

**Sprint-Hotfix 흐름 요약:**

```
sprint{n}  →  PR to develop  →  로컬 Docker 스테이징 검증
           →  CI에서 pnpm install → build → test → lint
           →  PR to main  →  서버 자동 배포

hotfix/*  →  PR to main  →  CI에서 pnpm install → build → test → lint
          →  서버 자동 배포  →  main을 develop에 역머지
```

**GitHub Actions CI 예시 (백엔드 pytest):**

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Python 3.12 설정
        uses: actions/setup-python@v5
        with:
          python-version: "3.12"
          cache: "pip"
          cache-dependency-path: app/backend/requirements.txt
      - name: 의존성 설치
        run: pip install -r app/backend/requirements.txt
      - name: pytest 실행
        run: pytest app/backend/tests/ -v --tb=short
```

**GitHub Actions CI 예시 (프론트엔드 pnpm):**

```yaml
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Node.js 20 설정
        uses: actions/setup-node@v4
        with:
          node-version: 20
      - name: Install pnpm
        run: npm install -g pnpm
      - name: Install dependencies
        run: pnpm install
      - name: Build
        run: pnpm build
      - name: Test
        run: pnpm test
      - name: Lint
        run: pnpm lint
```

> **참고**: 실제 `ci.yml`의 프론트엔드 스텝은 기본적으로 주석 처리되어 있습니다. 프론트엔드 소스 경로가 확정된 후 활성화하세요.

---

## Git 브랜치 전략

> 브랜치 구조와 배포 흐름 원칙은 [`strategy/branch-strategy.md`](../strategy/branch-strategy.md) 참조.
> Sprint/Hotfix 프로세스 상세는 [`docs/dev-process.md`](dev-process.md) 섹션 1 참조.

---

### Docker 이미지 태깅 규칙

| 이미지 | Registry |
|--------|---------|
| 백엔드 | `ghcr.io/${{ github.repository }}-backend` |
| 프론트엔드 | `ghcr.io/${{ github.repository }}-frontend` |
| nginx | `ghcr.io/${{ github.repository }}-nginx` |

> `github.repository`는 GitHub Actions에서 `owner/repo` 형식으로 자동 제공됩니다. `/setup-project` 치환 불필요.

| 브랜치 | Image Tag |
|--------|-----------|
| `develop` merge | 이미지 빌드 없음 — 로컬 Docker로만 검증 |
| `main` merge | `backend:latest`, `backend:{commit SHA}`, `backend:v{MAJOR.MINOR.PATCH}`, `frontend:latest`, `frontend:{commit SHA}`, `frontend:v{MAJOR.MINOR.PATCH}` |
| `hotfix` | `backend:v{MAJOR.MINOR.PATCH}`, `frontend:v{MAJOR.MINOR.PATCH}` |
> 버전은 Semantic Versioning (`MAJOR.MINOR.PATCH`) 기준

---

### 핵심 규칙

- `main` 직접 push 금지 — 반드시 PR + 리뷰 후 merge
- `develop` → `main` merge는 QA 통과 후 진행
- 긴급 패치는 **`main` 기반**으로 `hotfix/*` 브랜치를 생성하여 작업
- hotfix PR은 **`main`으로 직접** 생성 (develop 거치지 않음)
- main merge 후 반드시 `develop`에 역머지하여 동기화
- hotfix 범위 제한: 파일 3개 이하, 코드 50줄 이하, DB 변경 없음, 새 의존성 없음
- 스프린트 병렬 진행 시 `develop` merge 충돌 주의

---

## CI 파이프라인 (PR 체크)

PR이 `develop` 또는 `main`으로 올라오면 GitHub Actions가 자동으로 실행됩니다.

### 필수 통과 조건

1. 백엔드 테스트 — `app/backend/tests/` 전체 테스트는 `pytest`로 통과 필수
2. 프론트엔드 테스트 — `pnpm test` 통과 필수 (프론트엔드 소스 경로 확정 후 `ci.yml`에서 활성화)
3. Docker 이미지 빌드 성공 — 백엔드/프론트엔드 이미지 빌드 확인 (Dockerfile 생성 후 활성화)

> **프로젝트 적용 시 주의**: 템플릿 기본 상태에서 `ci.yml`의 프론트엔드 스텝과 Docker 빌드 스텝은 TODO 주석으로 비활성화되어 있습니다. 프로젝트 디렉터리 구조 확정 후 해당 스텝을 활성화하세요.

PR merge는 활성화된 조건이 모두 통과된 후에만 가능합니다 (Branch Protection Rule).

---

## CD 파이프라인 (배포 흐름)

### develop merge 후 (스테이징 검증)

`develop` 브랜치는 별도 서버 없이 **로컬 Docker**로 스테이징 검증합니다.

```bash
# 로컬에서 최신 코드 반영 후 검증
git pull origin develop
docker compose up --build
```

### main merge 후 (프로덕션 배포)

`main` 브랜치에 merge되면 GitHub Actions가 자동으로:

1. Docker 이미지 빌드 (backend + frontend + nginx)
2. GHCR에 이미지 push (`latest`, `{commit SHA}`, `v{MAJOR.MINOR.PATCH}`)
3. 프로덕션 서버에 SSH 접속
4. `docker compose pull && docker compose up -d` 실행

---

## 환경별 설정 관리

| 환경 | 설정 방법 | 비고 |
|------|----------|------|
| 로컬 개발 | `.env` 파일 | Git 미추적 (`.gitignore`) |
| 프로덕션 | GitHub Secrets | Actions에서 주입 |

> **프로덕션 .env 파일 관리:** 서버의 `{APP_PATH}/.env`는 서버에 수동으로 생성합니다. GitHub Secrets와 별도로 관리되며, 배포 시 자동으로 덮어쓰지 않습니다. 최초 서버 설정 시 `.env.example`을 복사하여 작성하세요.

### GitHub Secrets 목록 (프로덕션 필수)

| Secret 이름 | 설명 |
|------------|------|
| `LIGHTSAIL_SSH_KEY` | 서버 인스턴스 SSH 프라이빗 키 |
| `LIGHTSAIL_HOST` | 서버 IP 또는 도메인 |
| `LIGHTSAIL_USER` | SSH 사용자명 (예: `ubuntu`) |
| `POSTGRES_PASSWORD` | DB 비밀번호 |
| `JWT_SECRET` | JWT 서명 키 |
| `SECRET_KEY` | 앱 시크릿 키 |
| `NEXT_PUBLIC_API_URL` | 프론트엔드에서 사용하는 백엔드 API URL |

---

## 롤백 절차

> 아래는 CI/CD 관점의 롤백 요약입니다.

### 빠른 롤백 (Docker 이미지)

```bash
# 서버 SSH 접속 후
cd {APP_PATH}
docker compose -f docker-compose.prod.yml down
docker pull ghcr.io/{GITHUB_ORG}/{PROJECT}-backend:v{이전_버전}
docker pull ghcr.io/{GITHUB_ORG}/{PROJECT}-frontend:v{이전_버전}
docker compose -f docker-compose.prod.yml up -d
```

### DB 마이그레이션 롤백

```bash
# Alembic 다운그레이드 (주의: 데이터 손실 가능)
docker compose exec backend alembic downgrade -1
```

> ⚠️ DB 마이그레이션 롤백은 데이터 손실이 발생할 수 있습니다.
> 롤백 전 반드시 DB 백업을 수행하세요.

---

## HTTPS/TLS

### 방법 1: Let's Encrypt + certbot (권장)

```bash
# 서버 인스턴스에서
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d yourdomain.com
```

Nginx 설정에서 certbot이 자동으로 SSL 블록을 추가합니다. 90일마다 자동 갱신됩니다.

### 방법 2: 로드밸런서 SSL

AWS Lightsail 또는 다른 클라우드 콘솔에서 로드밸런서 생성 후 SSL 인증서를 연결합니다.
추가 비용이 발생하지만 관리가 단순합니다.