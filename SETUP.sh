#!/bin/bash
# 개발 환경 초기화 스크립트 (Tauri 2 + Next.js 15 + SQLite)
set -e

echo "=== SmartHB 개발 환경 초기화 ==="
echo ""

# ── Node.js 확인 ────────────────────────────────────────────────────────────
echo "=== Node.js 환경 확인 ==="

if ! command -v node &> /dev/null; then
  echo "❌ Node.js가 설치되어 있지 않습니다. https://nodejs.org 에서 Node.js 20 이상을 설치하세요."
  exit 1
fi

NODE_MAJOR=$(node --version | sed 's/v//' | cut -d. -f1)
if [ "$NODE_MAJOR" -lt 20 ]; then
  echo "⚠️  Node.js 버전이 낮습니다: $(node --version) (권장: v20 이상)"
else
  echo "✅ Node.js: $(node --version)"
fi

# ── pnpm 설치 ────────────────────────────────────────────────────────────────
if ! command -v npm &> /dev/null; then
  echo "❌ npm이 설치되어 있지 않습니다. Node.js를 먼저 설치하세요."
  exit 1
fi

if ! command -v pnpm &> /dev/null; then
  echo "pnpm이 없습니다. 설치 중..."
  npm install -g pnpm
  echo "✅ pnpm 설치 완료"
else
  echo "✅ pnpm: $(pnpm --version)"
fi

# ── jq 설치 확인 ─────────────────────────────────────────────────────────────
echo ""
echo "=== jq 확인 (Claude Code hooks 의존성) ==="

if ! command -v jq &> /dev/null; then
  echo "⚠️  jq가 설치되어 있지 않습니다."
  echo "   Claude Code hooks가 jq를 사용합니다."
  if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "   설치: brew install jq"
  else
    echo "   Windows: winget install jqlang.jq  또는  scoop install jq"
    echo "   Linux:   sudo apt install jq  또는  sudo yum install jq"
  fi
  echo "   ⚠️  jq 없이도 개발은 가능하나 hooks 일부 기능이 제한됩니다."
else
  echo "✅ jq: $(jq --version)"
fi

# ── python3 설치 확인 (Claude Code hooks 의존성) ───────────────────────────
echo ""
echo "=== python3 확인 (Claude Code hooks 의존성) ==="

if ! command -v python3 &> /dev/null; then
  echo "⚠️  python3가 설치되어 있지 않습니다."
  echo "   Claude Code의 일부 hooks(특히 posttooluse-scope-tracker)가 python3에 의존합니다."
  if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "   설치: xcode-select --install  또는  brew install python3"
  else
    echo "   Windows: winget install Python.Python.3  또는  https://python.org"
    echo "   Linux:   sudo apt install python3"
  fi
  echo "   ⚠️  python3 없이는 일부 hooks가 비활성화됩니다(jq 폴백 적용)."
else
  echo "✅ python3: $(python3 --version 2>&1)"
fi

# ── Rust/Cargo 확인 ──────────────────────────────────────────────────────────
echo ""
echo "=== Rust toolchain 확인 ==="

if ! command -v rustup &> /dev/null; then
  echo "⚠️  Rust(rustup)가 설치되어 있지 않습니다."
  echo "   설치: https://rustup.rs"
  echo "   또는: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo ""
  echo "   ⚠️  Rust 없이는 Tauri 빌드가 불가능합니다. 설치 후 SETUP.sh를 다시 실행하세요."
else
  echo "✅ rustup: $(rustup --version 2>/dev/null | head -1)"
  rustup toolchain install stable --no-self-update 2>/dev/null || true
  echo "✅ Rust stable: $(rustc --version)"

  # ── SQLx CLI ────────────────────────────────────────────────────────────
  echo ""
  echo "=== SQLx CLI 확인 ==="
  if ! command -v sqlx &> /dev/null; then
    echo "sqlx-cli가 없습니다. 설치 중... (잠시 시간이 걸립니다)"
    cargo install sqlx-cli --no-default-features --features sqlite
    echo "✅ sqlx-cli 설치 완료"
  else
    echo "✅ sqlx: $(sqlx --version)"
  fi
fi

# ── OS별 시스템 의존성 안내 ────────────────────────────────────────────────
echo ""
echo "=== Tauri 시스템 의존성 안내 ==="
if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "macOS: Xcode Command Line Tools 필요"
  if ! xcode-select -p &> /dev/null; then
    echo "❌ Xcode Command Line Tools 미설치"
    echo "   설치 명령: xcode-select --install"
    echo "   설치 후 SETUP.sh 를 다시 실행하세요."
    echo "   ⚠️  Xcode CLI 없이는 Rust 컴파일 및 Tauri 빌드가 모두 실패합니다."
    exit 1
  else
    echo "✅ Xcode CLI: $(xcode-select -p)"
  fi
else
  echo "Windows: WebView2 런타임은 Windows 11에 기본 포함됩니다."
  echo "Windows 10의 경우: https://developer.microsoft.com/microsoft-edge/webview2/ 에서 설치"
fi

# ── 프론트엔드 의존성 설치 ────────────────────────────────────────────────────
echo ""
echo "=== 프론트엔드 의존성 설치 ==="

if [ -f "package.json" ]; then
  echo "pnpm install 실행 중..."
  pnpm install
  echo "✅ 프론트엔드 의존성 설치 완료"
  echo "   (Tauri CLI는 pnpm devDependency로 포함됨 — pnpm exec tauri --version 으로 확인 가능)"
else
  echo "package.json 없음 — 프론트엔드 의존성 설치 생략"
fi

# ── 환경 변수 설정 ────────────────────────────────────────────────────────────
echo ""
echo "=== 환경 변수 설정 ==="

if [ ! -f ".env" ]; then
  if [ -f ".env.example" ]; then
    cp .env.example .env
    echo "✅ .env.example → .env 복사 완료. 실제 값을 .env에 입력하세요."
  else
    echo ".env.example 없음 — .env를 직접 생성하세요."
  fi
else
  echo "✅ .env 파일이 이미 존재합니다."
fi

# ── Git Hooks 설치 ────────────────────────────────────────────────────────────
echo ""
echo "=== Git Hooks 설치 (Harness Hook Compliance) ==="

if [ -d ".git" ]; then
  if [ -f "scripts/hooks/pre-commit" ]; then
    git config --local core.hooksPath scripts/hooks
    echo "✅ git hooks 경로 설정 완료 (scripts/hooks)"
    echo "   → 커밋 전 cargo fmt/clippy + 프론트엔드 lint 자동 검사"
  else
    echo "ℹ️  scripts/hooks/ 디렉토리가 비어있어 git hook 설치를 생략합니다."
    echo "   스프린트 개발 중 pre-commit hook이 필요하면 scripts/hooks/pre-commit 파일을 추가하세요."
  fi
else
  echo "⚠️  .git 디렉토리 없음 — git init 후 SETUP.sh 재실행"
fi

echo ""
echo "=== SETUP 완료 ==="
echo ""
echo "다음 명령으로 개발을 시작하세요:"
echo "  pnpm tauri:dev   # Tauri 앱 + Next.js 동시 기동"
echo "  pnpm dev         # Next.js만 (브라우저 테스트)"
