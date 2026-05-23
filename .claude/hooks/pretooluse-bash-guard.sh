#!/usr/bin/env bash
# pretooluse-bash-guard.sh
# Claude Code PreToolUse Hook — Bash 위험 명령 6가지 패턴 차단
#
# 입력: stdin JSON {"tool_input": {"command": "..."}}
# 출력: Exit 0 (허용) / Exit 2 (차단 + 메시지)

set -uo pipefail

# log-helper 로드 (없으면 no-op 함수 정의)
if [ -f ".claude/hooks/lib/log-helper.sh" ]; then
  source ".claude/hooks/lib/log-helper.sh"
else
  log_event() { :; }
fi

# stdin 읽기
INPUT=$(cat)

# JSON 파서 선택 — python3 우선, jq 폴백. 둘 다 없으면 안전을 위해 차단.
if command -v python3 &>/dev/null; then
  COMMAND=$(python3 -c "
import sys, json
try:
    d = json.loads(sys.stdin.read())
    print(d.get('tool_input', {}).get('command', ''))
except:
    print('')
" <<< "$INPUT" 2>/dev/null || echo "")
elif command -v jq &>/dev/null; then
  COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""' 2>/dev/null || echo "")
else
  echo ""
  echo "🚫 [bash-guard] python3 또는 jq가 필요합니다. 둘 다 미설치 상태에서는 위험 명령 차단을 보장할 수 없어 안전을 위해 차단합니다."
  echo "   macOS: brew install jq  (또는 xcode-select --install 로 python3)"
  echo "   Windows: winget install jqlang.jq"
  exit 2
fi

# 명령어가 없으면 허용 (다른 도구 호출일 가능성)
[ -n "$COMMAND" ] || exit 0

# 차단 함수 — stdout에 메시지 출력 후 exit 2
block() {
  echo ""
  echo "🚫 [bash-guard] $1"
  echo ""
  echo "  차단된 명령어: $COMMAND"
  echo ""
  log_event "bash-guard" "BLOCK" "${RULE_ID:-unknown}" "${COMMAND:0:80}"
  exit 2
}

# ── 규칙 1: 디렉토리 체이닝 차단 ────────────────────────────────────
# cd /path && ... 형태의 접두사 금지 (CLAUDE.md Bash 명령 실행 규칙)
if echo "$COMMAND" | grep -qE '^\s*cd\s+[^\s&;]+\s*&&'; then
  RULE_ID="dangerous-cd"
  block "디렉토리 체이닝(cd /path && ...)은 금지됩니다.
  작업 디렉토리는 항상 프로젝트 루트로 설정되어 있습니다.
  → git 명령: 'git ...' 형태로 직접 실행하세요."
fi

# ── 규칙 2: main 브랜치 직접 push 차단 ─────────────────────────────
if echo "$COMMAND" | grep -qE 'git push(\s+[^\s]+)?\s+main(\s|$)'; then
  RULE_ID="push-main"
  block "main 브랜치 직접 push는 금지됩니다.
  → PR 워크플로우를 사용하세요: develop → main PR은 deploy-prod 에이전트가 담당합니다."
fi

# ── 규칙 4: force push 차단 ─────────────────────────────────────────
if echo "$COMMAND" | grep -qE 'git push.+(-f\b|--force\b|--force-with-lease\b)'; then
  RULE_ID="force-push"
  block "Force push는 공유 브랜치의 히스토리를 손상시킵니다.
  → 해결책: 충돌을 해소하거나 새 커밋을 생성하세요."
fi

# ── 규칙 5: hard reset 차단 ─────────────────────────────────────────
if echo "$COMMAND" | grep -qE 'git reset\s+--hard'; then
  RULE_ID="hard-reset"
  block "git reset --hard는 로컬 변경 사항을 영구적으로 삭제합니다.
  → 대안: 'git stash'로 임시 보관하거나 'git revert'를 사용하세요."
fi

# ── 규칙 6: 브랜치 명명 규칙 검증 ──────────────────────────────────
if echo "$COMMAND" | grep -qE 'git (checkout -b|switch -c)\s+'; then
  BRANCH=$(echo "$COMMAND" | grep -oE '(checkout -b|switch -c)\s+\S+' | awk '{print $NF}' | head -1)
  if [ -n "$BRANCH" ]; then
    # 허용 패턴: sprint{N}, sprint{N}-{설명}, hotfix/{설명}
    if ! echo "$BRANCH" | grep -qE '^(sprint[0-9]+(-[a-z0-9][a-z0-9-]*)?|hotfix/.+)$'; then
      RULE_ID="branch-naming"
      block "브랜치 명명 규칙 위반: '$BRANCH'
  허용 패턴:
    ✓ sprint{N}           예: sprint1, sprint12
    ✓ sprint{N}-{설명}    예: sprint3-auth, sprint5-payment
    ✓ hotfix/{설명}       예: hotfix/fix-login-bug
  허용되지 않는 패턴:
    ✗ feature/*, bugfix/*, fix/*, 기타 임의 이름"
    fi
  fi
fi

exit 0
