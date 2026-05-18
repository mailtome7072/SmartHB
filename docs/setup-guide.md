# 환경 설정 가이드

> 프로젝트 최초 시작 시 1회 수행하는 환경 설정 가이드입니다.

---

## 1. 사전 요구사항

- ⬜ Git
- ⬜ Node.js **v20 이상** (https://nodejs.org)
- ⬜ Rust stable (`https://rustup.rs`) — `src-tauri/Cargo.toml`은 `edition = "2021"` 사용
- ⬜ pnpm (`npm install -g pnpm` 또는 `corepack enable && corepack prepare pnpm@latest --activate`)
- ⬜ jq (`winget install jqlang.jq` / `brew install jq`)
- ⬜ python3 (Claude Code hooks 의존 — `macOS: xcode-select --install` 또는 `brew install python3`, `Windows: winget install Python.Python.3`)
- ⬜ SQLx CLI (`cargo install sqlx-cli --no-default-features --features sqlite`)
- ⬜ **Windows**: WebView2 런타임 (Windows 11 기본 포함 / Windows 10은 수동 설치)
- ⬜ **macOS**: Xcode Command Line Tools (`xcode-select --install`)

> **Tauri CLI 별도 설치 불필요**: `package.json`의 devDependency로 포함되어 `pnpm install` 시 자동 설치됩니다. CLI 호출은 `pnpm tauri ...` 또는 `pnpm exec tauri ...` 사용.

---

## 2. 저장소 클론

```bash
git clone https://github.com/mailtome7072/SmartHB.git
cd SmartHB
```

---

## 3. 환경변수 설정

```bash
# .env.example을 복사하여 .env 파일 생성
cp .env.example .env
```

`.env` 파일을 열고 필요한 값을 입력합니다. 기본값:

```env
APP_NAME=SmartHB
DEBUG=false
DATABASE_URL=sqlite:./SmartHB-dev.db
```

---

## 4. 개발 환경 초기화 (SETUP.sh)

```bash
# 실행 권한 부여 (최초 1회, macOS/Linux)
chmod +x SETUP.sh

# 개발 환경 초기화 실행
./SETUP.sh
```

SETUP.sh는 다음을 자동으로 수행합니다:
- Node.js v20 이상 버전 확인
- pnpm 설치 및 프론트엔드 의존성 설치 (`pnpm install`)
- jq 설치 안내 (Claude Code hooks 의존성)
- Rust/rustup 설치 확인
- SQLx CLI 설치 (`cargo install sqlx-cli --no-default-features --features sqlite`)
- macOS: Xcode CLI 설치 여부 확인
- Windows: WebView2 런타임 안내
- `.env.example` → `.env` 복사

---

## 5. 개발 서버 실행

```bash
# Tauri 앱 + Next.js dev server 동시 기동 (권장)
pnpm tauri:dev

# Next.js만 (브라우저 테스트용)
pnpm dev
```

---

## 5-A. Tauri 아이콘 생성 (최초 1회)

`src-tauri/tauri.conf.json`이 참조하는 아이콘 파일들(`icons/32x32.png`, `icons/icon.icns`, `icons/icon.ico` 등)은 저장소에 포함되어 있지 않다. 첫 빌드 전에 1024×1024px 이상의 정사각형 PNG 로고를 준비한 뒤 다음 명령을 실행한다.

```bash
# 로고 파일 경로를 인자로 전달 — Tauri CLI가 .icns/.ico/PNG 세트를 일괄 생성
pnpm tauri icon ./path/to/logo.png
```

생성 결과는 `src-tauri/icons/` 디렉토리에 저장되며 커밋 대상이다. 아이콘이 없으면 `pnpm tauri:build` 실행 시 macOS `.dmg` 번들링이 실패한다.

---

## 6. 외부 서비스 설정

> TODO: 프로젝트에서 사용하는 외부 서비스 설정 방법을 작성하세요.

---

## 7. 개발 도구 설정

### VS Code 권장 익스텐션

> TODO: 프로젝트에 맞는 권장 익스텐션 목록을 작성하세요.
> 권장 예시: rust-analyzer, Tauri, ESLint, Tailwind CSS IntelliSense

---

## 8. GitHub Secrets 설정 (CI/CD)

GitHub Actions 배포 파이프라인이 동작하려면 리포지토리에 아래 Secrets를 등록해야 합니다.

**설정 경로:** GitHub 리포지토리 → Settings → Secrets and variables → Actions → New repository secret

### 선택적 Secrets (자동 업데이트 서명용)

| Secret 이름 | 설명 | 획득 방법 |
|------------|------|----------|
| `TAURI_PRIVATE_KEY` | Tauri 자동 업데이트 서명 프라이빗 키 | `tauri signer generate` 명령으로 생성 |
| `TAURI_KEY_PASSWORD` | 위 키의 비밀번호 | 키 생성 시 설정한 값 |

> **참고**: `GITHUB_TOKEN`은 GitHub Actions에서 자동 제공됩니다 — GitHub Release 생성에 별도 설정 불필요.
> Tauri 자동 업데이트를 도입하지 않는 경우 위 Secrets는 불필요합니다.

---

## 9. Claude Code 설정

이 프로젝트는 Claude Code와 함께 사용하도록 설계되었습니다.

### 전제 조건

- Claude Code 설치: https://claude.ai/claude-code
- MCP 서버 설정 (선택사항): Notion 등

### 에이전트 활용

- `sprint-planner`: 스프린트 계획 수립
- `sprint-close`: 스프린트 마무리 (PR, 문서화)
- `sprint-review`: 코드 리뷰 + 자동 검증 + 회고
- `hotfix-close`: 핫픽스 마무리
- `deploy-prod`: 프로덕션 배포 (GitHub Releases)
- `prd-to-roadmap`: PRD → ROADMAP.md 변환

자세한 내용은 `README.md` 및 `CLAUDE.md` 참조.
