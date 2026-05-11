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

# stdin에서 도구 입력 추출
INPUT=$(cat)
TOOL_NAME=$(python3 -c "
import sys, json
try:
    d = json.loads(sys.stdin.read())
    print(d.get('tool_name', ''))
except:
    print('')
" <<< "$INPUT" 2>/dev/null || echo "")

FILE_PATH=$(python3 -c "
import sys, json
try:
    d = json.loads(sys.stdin.read())
    ti = d.get('tool_input', {})
    print(ti.get('file_path', ''))
except:
    print('')
" <<< "$INPUT" 2>/dev/null || echo "")

# 도구명 또는 파일 경로가 없으면 pass
[ -n "$TOOL_NAME" ] || exit 0
[ -n "$FILE_PATH" ] || exit 0

# Edit/Write 도구만 검사
if [[ "$TOOL_NAME" != "Edit" && "$TOOL_NAME" != "Write" ]]; then
  exit 0
fi

# ── 규칙 1: .env 파일 수정 차단 ──────────────────────────────────────────
# .env, .env.local, .env.production 등 수정 시 차단
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
# settings.json 수정 시 경고 출력 (차단하지 않음 — 의도적 변경 허용)
if echo "$FILE_PATH" | grep -qE '\.claude/settings(\.local)?\.json$'; then
  echo ""
  echo "⚠️  [posttooluse-validator] Claude 설정 파일이 수정되었습니다."
  echo ""
  echo "  파일: $FILE_PATH"
  echo "  → Hook 또는 권한 변경이 적용됩니다. 의도한 변경인지 확인하세요."
  echo ""
  # 차단하지 않음 (exit 0으로 계속)
  exit 0
fi

# ── 규칙 5: Forbidden Areas — 사용자 확인 후 허가 ───────────────────────
# Harness Engineering 원칙 2: Forbidden Areas
# 인프라·CI/CD·보안 핵심 파일은 사용자 명시적 허가 후에만 수정 가능
#
# [허가 흐름]
#   1. Edit/Write 시도 → 이 훅이 차단 + 허가 명령 안내
#   2. Claude가 사용자에게 허가 요청
#   3. 사용자 승인 → Claude가 아래 명령 실행: touch {PERMIT_FLAG}
#   4. Edit/Write 재시도 → 훅이 플래그 확인 후 허용 + 플래그 삭제 (1회용)

# 허가 플래그 경로 계산 (파일 경로 기반 해시 → .claude/tmp/claude-permit-{hash})
_permit_flag() {
  python3 -c "
import hashlib, sys
h = hashlib.md5('$FILE_PATH'.encode()).hexdigest()[:12]
print('.claude/tmp/claude-permit-' + h)
" 2>/dev/null
}

# 허가 플래그 존재 시 1회 허용 (플래그 즉시 삭제)
_check_permit() {
  local flag
  flag=$(_permit_flag)
  mkdir -p "$(dirname "$flag")"
  if [ -f "$flag" ]; then
    rm -f "$flag"
    echo ""
    echo "✅ [posttooluse-validator] Forbidden Area 수정이 허가되었습니다 (1회 사용)."
    echo "  파일: $FILE_PATH"
    echo ""
    return 0  # 허가됨
  fi
  return 1  # 허가 없음
}

# 허가 요청 메시지 출력 공통 함수
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
  _deny_with_permit "Harness Engineering 정책 변경은 팀 합의가 필요합니다. 정책 약화(guardrail 완화, 차단 조건 제거)는 특히 주의가 필요합니다."
fi

# 5-D: Docker/인프라 설정
if echo "$FILE_PATH" | grep -qE '(^|/)(docker-compose[^/]*\.ya?ml|docker/[^/])'; then
  _check_permit && exit 0
  _deny_with_permit "컨테이너 설정 변경은 스테이징·프로덕션 환경에 영향을 줍니다."
fi

# ── 규칙 3: Python 파일 syntax 검증 ─────────────────────────────────────
if echo "$FILE_PATH" | grep -qE '\.py$'; then
  if [ -f "$FILE_PATH" ]; then
    SYNTAX_OUTPUT=$(python3 -m py_compile "$FILE_PATH" 2>&1)
    SYNTAX_EXIT=$?
    if [ $SYNTAX_EXIT -ne 0 ]; then
      echo ""
      echo "🚨 [posttooluse-validator] Python syntax 오류 감지!"
      echo ""
      echo "  파일: $FILE_PATH"
      echo "  오류:"
      echo "$SYNTAX_OUTPUT" | sed 's/^/    /'
      echo ""
      echo "  → 즉시 수정이 필요합니다. 커밋 전 반드시 해결하세요."
      echo ""
      # syntax 오류는 경고만 출력 (exit 1, non-blocking)
      # 에이전트가 즉시 인지하고 수정할 수 있도록 함
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
    echo "  → 테스트용 더미 값이라면 무시해도 됩니다."
    echo ""
    log_event "code-validator" "WARN" "secret-pattern" "$FILE_PATH"
    # 경고만 출력, 차단하지 않음
    exit 0
  fi
fi

# ── 규칙 6: Planning First — sprint 브랜치 scope.md 존재 확인 (비차단 경고) ──
# sprint{N} 브랜치에서 코드 파일(.py, .ts, .tsx 등) 수정 시
# docs/sprint/sprint{N}/scope.md 없으면 Planning First 원칙 경고 출력
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
SPRINT_N=$(echo "$BRANCH" | grep -oE '(sprint|Sprint)([0-9]+)' | grep -oE '[0-9]+' | head -1 2>/dev/null || echo "")

if [ -n "$SPRINT_N" ]; then
  SCOPE_FILE="docs/sprint/sprint${SPRINT_N}/scope.md"
  # .md 파일, docs/ 내 파일, .claude/ 내 파일 수정은 제외 (코드 파일만 검사)
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
      # 비차단 — 경고만 출력
    fi
  fi
fi

exit 0
