#!/bin/bash
# 개발 환경 초기화 스크립트
set -e

echo "=== Node.js 환경 확인 ==="

# Node.js 설치 여부 확인
if ! command -v node &> /dev/null; then
  echo "❌ Node.js가 설치되어 있지 않습니다. https://nodejs.org 에서 Node.js 20 이상을 설치하세요."
  exit 1
fi

# Node.js 메이저 버전 확인
NODE_MAJOR=$(node --version | sed 's/v//' | cut -d. -f1)
if [ "$NODE_MAJOR" -lt 20 ]; then
  echo "⚠️  Node.js 버전이 낮습니다: $(node --version) (권장: v20 이상)"
else
  echo "✅ Node.js: $(node --version)"
fi

echo ""
echo "=== 프론트엔드 환경 초기화 ==="

# npm 존재 여부 확인 후 pnpm 설치
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

# 프론트엔드 의존성 설치 (package.json이 있는 경우)
if [ -f "package.json" ]; then
  echo "pnpm install 실행 중..."
  pnpm install
  echo "✅ 프론트엔드 의존성 설치 완료"
  # TODO: 프론트엔드 소스가 있을 경우 아래 주석을 해제하세요
  # pnpm build
  # pnpm test
  # pnpm lint
else
  echo "package.json 없음 — 프론트엔드 의존성 설치 생략"
fi

echo ""
echo "=== 백엔드 환경 초기화 ==="

# Python 설치 여부 확인
if ! command -v python3 &> /dev/null; then
  echo "❌ python3가 설치되어 있지 않습니다. Python 3.12 이상을 설치하세요."
else
  echo "✅ Python: $(python3 --version)"

  # 가상환경 생성 (없을 경우만)
  if [ ! -d ".venv" ]; then
    echo "가상환경(.venv) 생성 중..."
    python3 -m venv .venv
    echo "✅ 가상환경 생성 완료"
  else
    echo "✅ 가상환경(.venv) 이미 존재"
  fi

  # 가상환경 활성화
  # shellcheck source=/dev/null
  source .venv/bin/activate
  echo "✅ 가상환경 활성화 완료"

  # 백엔드 의존성 설치 (requirements.txt가 있는 경우)
  if [ -f "app/backend/requirements.txt" ]; then
    echo "pip 의존성 설치 중 (app/backend/requirements.txt)..."
    python3 -m pip install -r app/backend/requirements.txt
    echo "✅ 백엔드 의존성 설치 완료"
  else
    echo "app/backend/requirements.txt 없음 — Python 의존성 설치 생략"
  fi
fi

echo ""
echo "=== 환경 변수 설정 ==="

# .env 파일 확인
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

echo ""
echo "=== Git Hooks 설치 (Harness Hook Compliance) ==="

if [ -d ".git" ]; then
  if [ -f "scripts/hooks/pre-commit" ]; then
    git config --local core.hooksPath scripts/hooks
    echo "✅ git hooks 경로 설정 완료 (scripts/hooks)"
    echo "   → 커밋 전 Python syntax + 프론트엔드 lint 자동 검사"
  else
    echo "⚠️  scripts/hooks/pre-commit 없음 — git hook 설치 생략"
  fi
else
  echo "⚠️  .git 디렉토리 없음 — git hook 설치 생략 (git init 후 SETUP.sh 재실행)"
fi

echo ""
echo "=== SETUP 완료 ==="
