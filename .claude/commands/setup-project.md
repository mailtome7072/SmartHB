# /setup-project

`ARCHITECTURE.md`의 변수 테이블을 읽어 프로젝트 파일 전체에 일괄 적용합니다.

## 실행 절차

### 1단계: ARCHITECTURE.md에서 변수 읽기

`ARCHITECTURE.md`의 **프로젝트 변수** 테이블에서 아래 5개 값을 읽는다.
값이 `(여기에 입력)` 상태이거나 비어 있으면 **즉시 중단**하고 사용자에게 알린다.

| 변수 | 읽을 열 |
|------|--------|
| `project_name` | 프로젝트 이름 |
| `project_description` | 프로젝트 한 줄 설명 |
| `github_org` | GitHub 조직 또는 계정명 |
| `github_repo` | GitHub 저장소명 |
| `decision_date` | PRD 작성 결정일 |

자동 조합 (치환 후 실제값이 채워진 예시):
- `repo_url` = `https://github.com/${github_org}/${github_repo}.git`
- `ghcr_prefix` = `ghcr.io/${github_org}/${github_repo}`

### 2단계: 파일별 치환 실행

아래 파일에 대해 `${변수명}` 형식의 플레이스홀더를 ARCHITECTURE.md에서 읽은 실제 값으로 대체한다.

**README.md**
- `${project_name}` → 실제 project_name 값
- `${project_description}` → 실제 project_description 값
- `${github_org}` → 실제 github_org 값
- `${github_repo}` → 실제 github_repo 값

**CLAUDE.md**
- `${github_org}` → 실제 github_org 값
- `${github_repo}` → 실제 github_repo 값

**PRD.md**
- `${decision_date}` → 실제 decision_date 값

**docs/ci-policy.md**
- `${github_org}` → 실제 github_org 값
- `${github_repo}` → 실제 github_repo 값

**docker-compose.prod.yml**
- `${github_org}` → 실제 github_org 값
- `${github_repo}` → 실제 github_repo 값

### 3단계: 결과 요약 출력

치환 완료 후 다음 형식으로 결과를 출력한다:

```
✅ /setup-project 완료

적용된 변수:
  project_name      = {실제값}
  project_description = {실제값}
  github_org        = {실제값}
  github_repo       = {실제값}
  decision_date     = {실제값}
  repo_url          = {조합값}
  ghcr_prefix       = {조합값}

수정된 파일:
  - README.md
  - CLAUDE.md
  - PRD.md
  - docs/ci-policy.md
  - docker-compose.prod.yml

참고: .github/workflows/deploy.yml은 github.repository 내장 변수를 사용하므로 치환 불필요합니다.

🔍 치환 결과 확인 (선택):
  각 파일에서 아래 명령으로 잔류 플레이스홀더를 검색하세요:
  grep -r '\${github_org}\|${github_repo}\|${project_name}\|${decision_date}' README.md CLAUDE.md PRD.md docs/ci-policy.md docker-compose.prod.yml
  결과가 없으면 치환이 정상 완료된 것입니다.

⚠️  다음 항목은 수동 설정이 필요합니다:
  - GitHub Secrets: LIGHTSAIL_HOST, LIGHTSAIL_USER, LIGHTSAIL_SSH_KEY (GHCR 인증은 GITHUB_TOKEN 자동 제공)
    앱 시크릿(POSTGRES_PASSWORD, JWT_SECRET 등) 전체 목록: docs/ci-policy.md 참조
  - .env 파일: .env.example 복사 후 실제 값 입력
  - docs/dev-process.md 섹션 6.3: SSH 접속 정보 직접 입력
    (SSH_KEY_PATH, USER, SERVER_IP, APP_PATH — deploy-prod 에이전트 실서버 검증 시 참조)
```

### 오류 처리

- 변수가 미입력 상태(`(여기에 입력)`)이면 해당 변수명을 나열하고 중단
- 대상 파일이 존재하지 않으면 경고만 출력하고 나머지 파일은 계속 처리
