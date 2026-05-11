# 환경 설정 가이드

> 프로젝트 최초 시작 시 1회 수행하는 환경 설정 가이드입니다.

---

## 1. 사전 요구사항

- ⬜ Git
- ⬜ Docker Desktop
- ⬜ Node.js **v20 이상** (https://nodejs.org)
- ⬜ Python **3.12 이상** (https://www.python.org)
- ⬜ 기타 도구 (프로젝트에 맞게 추가)

---

## 2. 저장소 클론

```bash
git clone https://github.com/your-org/your-project.git
cd your-project
```

---

## 3. 환경변수 설정

```bash
# .env.example을 복사하여 .env 파일 생성
cp .env.example .env
```

`.env` 파일을 열고 필요한 값을 입력합니다:

> TODO: 각 환경변수에 대한 설명과 획득 방법을 작성하세요.

---

## 4. 로컬 개발 환경 실행

### 네이티브 개발 환경 (SETUP.sh)

```bash
# 실행 권한 부여 (최초 1회)
chmod +x SETUP.sh

# 개발 환경 초기화 실행
./SETUP.sh
```

SETUP.sh는 다음을 자동으로 수행합니다:
- Node.js v20 이상 버전 확인
- pnpm 설치 및 프론트엔드 의존성 설치
- Python 가상환경(`.venv`) 생성 및 백엔드 의존성 설치
- `.env.example` → `.env` 복사

### Docker Compose 환경 (전체 스택)

> **사전 조건**: `docker-compose.yml`은 이 템플릿에 포함되지 않습니다. 첫 스프린트에서 백엔드·프론트엔드 앱 코드가 생성될 때 함께 작성하세요. (`app/backend/Dockerfile`, `app/frontend/Dockerfile` 생성 후 서비스 정의 추가)

```bash
# Docker Compose로 전체 스택 실행
docker compose up --build

# 백엔드 DB 마이그레이션 (최초 1회)
docker compose exec backend alembic upgrade head

# 초기 데이터 시드 (필요한 경우)
docker compose exec backend python app/backend/scripts/seed.py
```

서비스 접속:
- 프론트엔드: http://localhost:3000
- 백엔드 API: http://localhost:8000
- API 문서: http://localhost:8000/docs

---

## 5. 외부 서비스 설정

> TODO: 프로젝트에서 사용하는 외부 서비스 설정 방법을 작성하세요.

### 5.1 {외부 서비스 1}

> TODO

### 5.2 {외부 서비스 2}

> TODO

---

## 6. 개발 도구 설정

### VS Code 권장 익스텐션

> TODO: 프로젝트에 맞는 권장 익스텐션 목록을 작성하세요.

---

## 7. GitHub Secrets 설정 (CI/CD 필수)

GitHub Actions 배포 파이프라인이 동작하려면 리포지토리에 아래 Secrets를 등록해야 합니다.

**설정 경로:** GitHub 리포지토리 → Settings → Secrets and variables → Actions → New repository secret

### 필수 Secrets

| Secret 이름 | 설명 | 획득 방법 |
|------------|------|----------|
| `LIGHTSAIL_SSH_KEY` | 서버 인스턴스 SSH 프라이빗 키 전체 내용 | AWS Lightsail 키 페어 다운로드 후 내용 복사 |
| `LIGHTSAIL_HOST` | 서버 IP 또는 도메인 | AWS Lightsail 콘솔에서 확인 |
| `LIGHTSAIL_USER` | SSH 사용자명 | 기본값: `ubuntu` |
| `POSTGRES_PASSWORD` | DB 비밀번호 | 직접 설정 (충분한 길이의 랜덤 문자열 권장) |
| `JWT_SECRET` | JWT 서명 키 | 직접 설정 (32바이트 이상 랜덤 문자열 권장) |
| `SECRET_KEY` | 앱 시크릿 키 | 직접 설정 (32바이트 이상 랜덤 문자열 권장) |
| `NEXT_PUBLIC_API_URL` | 프론트엔드에서 사용하는 백엔드 API URL | 예: `https://api.yourdomain.com` |

> **참고**: Secrets 목록 전체는 `docs/ci-policy.md` → "GitHub Secrets 목록 (프로덕션 필수)" 섹션을 참조합니다.

---

## 8. Claude Code 설정

이 프로젝트는 Claude Code와 함께 사용하도록 설계되었습니다.

### 전제 조건

- Claude Code 설치: https://claude.ai/claude-code
- MCP 서버 설정 (선택사항): Playwright, Notion 등

### 에이전트 활용

- `sprint-planner`: 스프린트 계획 수립
- `sprint-close`: 스프린트 마무리 (PR, 코드 리뷰, 검증)
- `hotfix-close`: 핫픽스 마무리
- `deploy-prod`: 프로덕션 배포
- `prd-to-roadmap`: PRD → ROADMAP.md 변환

자세한 내용은 `README.md` 참조.
