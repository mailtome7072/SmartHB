#!/usr/bin/env bash
# posttooluse-code-validator.sh
# Claude Code PostToolUse Hook — Edit/Write 후 즉각 코드 검증
# Harness Engineering 원칙 2 (Strict Guardrails) 구현
#
# 입력: stdin JSON {"tool_name": "...", "tool_input": {"file_path": "...", ...}}
# 출력: Exit 0 (pass) / Exit 1 (warning, non-blocking) / Exit 2 (block + 메시지)

set -uo pipefail

# log-helper 로드 (없으면 no-op 함수 정의)
if [ -f ".claude/hooks/lib/log-helper.sh" ]; then
  source ".claude/hooks/lib/log-helper.sh"
else
  log_event() { :; }
fi

# stdin에서 도구 입력 추출 (jq 우선, fallback: grep/sed)
INPUT=$(cat)

if command -v jq &>/dev/null; then
  TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // ""' 2>/dev/null || echo "")
  FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // ""' 2>/dev/null || echo "")
else
  TOOL_NAME=$(echo "$INPUT" | grep -o '"tool_name":"[^"]*"' | sed 's/"tool_name":"//;s/"//' | head -1 || echo "")
  FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"//' | head -1 || echo "")
fi

# 도구명 또는 파일 경로가 없으면 pass
[ -n "$TOOL_NAME" ] || exit 0
[ -n "$FILE_PATH" ] || exit 0

# Edit/Write 도구만 검사
if [[ "$TOOL_NAME" != "Edit" && "$TOOL_NAME" != "Write" ]]; then
  exit 0
fi

# ── 규칙 1: .env 파일 수정 차단 ──────────────────────────────────────────
if echo "$FILE_PATH" | grep -qE '(^|/)\.(env)(\.[a-zA-Z0-9.]+)?$'; then
  echo ""
  echo "🚫 [posttooluse-validator] .env 파일 수정이 차단됩니다."
  echo ""
  echo "  파일: $FILE_PATH"
  echo ""
  echo "  이유: 환경변수 파일은 민감한 시크릿을 포함합니다."
  echo "  → 환경변수 추가 시 .env.example에 키 이름만 기재하세요."
  echo "  → 실제 값은 사람이 직접 .env에 입력합니다."
  echo ""
  log_event "code-validator" "BLOCK" "env-file" "$FILE_PATH"
  exit 2
fi

# ── 규칙 2: .claude/settings.json 수정 경고 ─────────────────────────────
if echo "$FILE_PATH" | grep -qE '\.claude/settings(\.local)?\.json$'; then
  echo ""
  echo "⚠️  [posttooluse-validator] Claude 설정 파일이 수정되었습니다."
  echo ""
  echo "  파일: $FILE_PATH"
  echo "  → Hook 또는 권한 변경이 적용됩니다. 의도한 변경인지 확인하세요."
  echo ""
  exit 0
fi

# ── 규칙 5: Forbidden Areas — 사용자 확인 후 허가 ───────────────────────
# 허가 플래그 경로 계산 (파일 경로 기반 MD5 해시)
_permit_flag() {
  if command -v python3 &>/dev/null; then
    python3 -c "
import hashlib
h = hashlib.md5('$FILE_PATH'.encode()).hexdigest()[:12]
print('.claude/tmp/claude-permit-' + h)
" 2>/dev/null
  elif command -v md5sum &>/dev/null; then
    local h
    h=$(echo -n "$FILE_PATH" | md5sum | cut -c1-12)
    echo ".claude/tmp/claude-permit-$h"
  elif command -v md5 &>/dev/null; then
    local h
    h=$(echo -n "$FILE_PATH" | md5 -q | cut -c1-12)
    echo ".claude/tmp/claude-permit-$h"
  elif command -v powershell &>/dev/null; then
    local h
    h=$(powershell -NoProfile -Command "\$md5=[System.Security.Cryptography.MD5]::Create(); \$h=[System.BitConverter]::ToString(\$md5.ComputeHash([System.Text.Encoding]::UTF8.GetBytes('$FILE_PATH'))).Replace('-','').ToLower().Substring(0,12); Write-Output \$h" 2>/dev/null)
    echo ".claude/tmp/claude-permit-$h"
  else
    echo ".claude/tmp/claude-permit-unknown"
  fi
}

_check_permit() {
  local flag
  flag=$(_permit_flag)
  mkdir -p "$(dirname "$flag")" 2>/dev/null || true
  if [ -f "$flag" ]; then
    rm -f "$flag"
    echo ""
    echo "✅ [posttooluse-validator] Forbidden Area 수정이 허가되었습니다 (1회 사용)."
    echo "  파일: $FILE_PATH"
    echo ""
    return 0
  fi
  return 1
}

_deny_with_permit() {
  local reason="$1"
  local flag
  flag=$(_permit_flag)
  echo ""
  echo "🚫 [posttooluse-validator] Forbidden Area — 사용자 허가가 필요합니다."
  echo ""
  echo "  파일: $FILE_PATH"
  echo "  이유: $reason"
  echo ""
  echo "  → 사용자에게 허가를 요청하세요."
  echo "  → 허가 확인 후 아래 명령을 실행하면 다음 1회 수정이 허용됩니다:"
  echo ""
  echo "     touch $flag"
  echo ""
  log_event "code-validator" "BLOCK" "forbidden-area" "$FILE_PATH"
  exit 2
}

# 5-A: CI/CD 파이프라인 파일
if echo "$FILE_PATH" | grep -qE '\.github/workflows/.*\.ya?ml$'; then
  _check_permit && exit 0
  _deny_with_permit "CI/CD 워크플로우는 전체 배포 파이프라인에 영향을 미칩니다."
fi

# 5-B: SETUP.sh — 프로젝트 초기화 스크립트
if echo "$FILE_PATH" | grep -qE '(^|/)SETUP\.sh$'; then
  _check_permit && exit 0
  _deny_with_permit "SETUP.sh는 모든 개발 환경 초기화에 사용되는 핵심 스크립트입니다."
fi

# 5-C: Harness 정책 문서 (정책 임의 약화 방지)
if echo "$FILE_PATH" | grep -qE 'docs/harness-engineering/'; then
  _check_permit && exit 0
  _deny_with_permit "Harness Engineering 정책 변경은 팀 합의가 필요합니다."
fi

# 5-D: Tauri 앱 설정 (배포 번들에 직접 영향)
if echo "$FILE_PATH" | grep -qE 'src-tauri/tauri\.conf\.json$'; then
  _check_permit && exit 0
  _deny_with_permit "tauri.conf.json은 앱 번들 ID, 권한, 배포 설정에 직접 영향을 줍니다."
fi

# ── 규칙 3: Rust 파일 syntax 검증 ─────────────────────────────────────
if echo "$FILE_PATH" | grep -qE '\.rs$'; then
  if [ -f "src-tauri/Cargo.toml" ]; then
    SYNTAX_OUTPUT=$(cargo check --manifest-path src-tauri/Cargo.toml 2>&1)
    SYNTAX_EXIT=$?
    if [ $SYNTAX_EXIT -ne 0 ]; then
      echo ""
      echo "🚨 [posttooluse-validator] Rust 컴파일 오류 감지!"
      echo ""
      echo "  파일: $FILE_PATH"
      echo "  오류 (최근 20줄):"
      echo "$SYNTAX_OUTPUT" | tail -20 | sed 's/^/    /'
      echo ""
      echo "  → 즉시 수정이 필요합니다. 커밋 전 반드시 해결하세요."
      echo ""
      exit 1
    fi
  fi
fi

# ── 규칙 4: 시크릿 패턴 감지 (경고) ─────────────────────────────────────
if [ -f "$FILE_PATH" ]; then
  SECRET_MATCH=$(grep -nE \
    '(password|passwd|secret|api_key|apikey|token|private_key)\s*=\s*["'"'"'][^${\s]{6,}["'"'"']' \
    "$FILE_PATH" 2>/dev/null | grep -v '\.example' | head -3 || true)

  if [ -n "$SECRET_MATCH" ]; then
    echo ""
    echo "⚠️  [posttooluse-validator] 코드에 하드코딩된 시크릿 패턴이 감지됩니다."
    echo ""
    echo "  파일: $FILE_PATH"
    echo "  의심 라인:"
    echo "$SECRET_MATCH" | sed 's/^/    /'
    echo ""
    echo "  → 실제 시크릿이라면 즉시 제거하고 환경변수로 교체하세요."
    echo ""
    log_event "code-validator" "WARN" "secret-pattern" "$FILE_PATH"
    exit 0
  fi
fi

# ── 규칙 6: Planning First — sprint 브랜치 scope.md 존재 확인 (비차단 경고) ──
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
SPRINT_N=$(echo "$BRANCH" | grep -oE '(sprint|Sprint)([0-9]+)' | grep -oE '[0-9]+' | head -1 2>/dev/null || echo "")

if [ -n "$SPRINT_N" ]; then
  SCOPE_FILE="docs/sprint/sprint${SPRINT_N}/scope.md"
  if ! echo "$FILE_PATH" | grep -qE '(\.md$|/docs/|\.claude/)'; then
    if [ ! -f "$SCOPE_FILE" ]; then
      echo ""
      echo "⚠️  [posttooluse-validator] Planning First 경고: scope.md 없음"
      echo ""
      echo "  파일  : $FILE_PATH"
      echo "  브랜치: $BRANCH"
      echo "  → $SCOPE_FILE 을 먼저 작성하세요 (Harness 원칙 1)."
      echo ""
      log_event "code-validator" "WARN" "planning-first" "$FILE_PATH"
    fi
  fi
fi

exit 0
