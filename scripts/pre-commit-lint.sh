#!/usr/bin/env bash
# pre-commit-lint.sh
# Harness Engineering 원칙 2: Hook Compliance
# git commit 실행 전 lint + syntax 검사를 수행하여 오류 있는 코드 커밋을 차단한다.
#
# 설치 방법: SETUP.sh 실행 시 자동 설치됨
#   cp scripts/pre-commit-lint.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
#
# 수동 설치:
#   bash scripts/pre-commit-lint.sh

set -uo pipefail

FAIL=0

echo "🔍 [pre-commit] Harness Hook Compliance 검사 시작..."

# ── Rust fmt/clippy 검사 ──────────────────────────────────────────────────
# staged Rust 파일이 있고 src-tauri/ 디렉토리가 존재하는 경우만 실행
RS_FILES=$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null | grep '\.rs$' || true)

if [ -n "$RS_FILES" ] && [ -d "src-tauri" ] && [ -f "src-tauri/Cargo.toml" ]; then
  echo ""
  echo "  🦀 Rust fmt 검사..."
  if command -v cargo &>/dev/null; then
    FMT_OUTPUT=$(cargo fmt --manifest-path src-tauri/Cargo.toml -- --check 2>&1)
    FMT_EXIT=$?
    if [ $FMT_EXIT -ne 0 ]; then
      echo "  ❌ fmt 오류: 포맷이 일치하지 않습니다"
      echo "$FMT_OUTPUT" | sed 's/^/     /'
      echo "     → cargo fmt --manifest-path src-tauri/Cargo.toml 실행 후 재커밋하세요."
      FAIL=1
    else
      echo "  ✅ cargo fmt 통과"
    fi

    echo ""
    echo "  🦀 Rust clippy 검사..."
    CLIPPY_OUTPUT=$(cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>&1)
    CLIPPY_EXIT=$?
    if [ $CLIPPY_EXIT -ne 0 ]; then
      echo "  ❌ clippy 경고/오류 발견"
      echo "$CLIPPY_OUTPUT" | tail -20 | sed 's/^/     /'
      FAIL=1
    else
      echo "  ✅ cargo clippy 통과"
    fi
  else
    echo "  ⚠️  cargo 없음 — Rust 검사 건너뜁니다."
  fi
fi

# ── 프론트엔드 lint 검사 ──────────────────────────────────────────────────
# staged TS/TSX 파일이 있고 루트 package.json이 존재하는 경우만 실행
FE_FILES=$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null | grep -E '^src/.*\.(ts|tsx|js|jsx)$' || true)

if [ -n "$FE_FILES" ] && [ -f "package.json" ]; then
  echo ""
  echo "  🎨 프론트엔드 lint 검사..."
  if command -v pnpm &>/dev/null; then
    LINT_OUTPUT=$(pnpm lint --max-warnings 0 2>&1)
    LINT_EXIT=$?
    if [ $LINT_EXIT -ne 0 ]; then
      echo "  ❌ lint 오류 발견"
      echo "$LINT_OUTPUT" | tail -20 | sed 's/^/     /'
      FAIL=1
    else
      echo "  ✅ lint 통과"
    fi
  else
    echo "  ⚠️  pnpm 없음 — 프론트엔드 lint 건너뜁니다."
  fi
fi

# ── 시크릿 패턴 검사 ─────────────────────────────────────────────────────
# staged 파일에서 하드코딩된 시크릿 패턴 감지 (경고, 차단하지 않음)
SECRET_MATCH=$(git diff --cached -- '*.rs' '*.ts' '*.tsx' '*.js' 2>/dev/null | \
  grep -E '^\+.*(password|secret|api_key|apikey|token|private_key)\s*=\s*["'"'"'][^${\s]{6,}["'"'"']' | \
  grep -v '\.example' | head -3 || true)

if [ -n "$SECRET_MATCH" ]; then
  echo ""
  echo "  ⚠️  [경고] 시크릿 패턴이 감지되었습니다 (차단하지 않음):"
  echo "$SECRET_MATCH" | sed 's/^/     /'
  echo "  → 실제 시크릿이라면 환경변수로 교체하세요."
fi

# ── 결과 ─────────────────────────────────────────────────────────────────
echo ""
if [ $FAIL -eq 0 ]; then
  echo "✅ [pre-commit] Hook Compliance 검사 통과 — 커밋을 진행합니다."
else
  echo "🚫 [pre-commit] 오류 발견 — 커밋이 차단됩니다."
  echo "   위의 오류를 수정한 후 다시 커밋하세요."
  echo "   (오류 수정 없이 강제 커밋: git commit --no-verify — Harness 원칙 위반)"
fi

exit $FAIL
