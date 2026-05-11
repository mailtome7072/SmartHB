#!/usr/bin/env bash
# posttooluse-scope-tracker.sh
# Edit/Write 도구 실행 후 scope.md의 파일 수정 횟수를 자동으로 증가시킨다.
# Harness Engineering 원칙 3 — Loop Detection 자동 카운팅 구현
#
# 동작 조건:
#   1. 현재 브랜치가 sprint{N} 형태일 것
#   2. docs/sprint/sprint{N}/scope.md 파일이 존재할 것
#   3. scope.md 테이블에 수정된 파일이 등록되어 있을 것
#
# 출력:
#   - 3회 도달 시: 루프 감지 경고 출력 (exit 1, non-blocking)
#   - 그 외: 조용히 종료

set -uo pipefail

# log-helper 로드 (없으면 no-op 함수 정의)
if [ -f ".claude/hooks/lib/log-helper.sh" ]; then
  source ".claude/hooks/lib/log-helper.sh"
else
  log_event() { :; }
fi

INPUT=$(cat)

# ── tool_name, file_path 추출 ─────────────────────────────────────────────
TOOL_NAME=$(python3 -c "
import sys, json
try:
    d = json.loads(sys.stdin.read())
    print(d.get('tool_name', ''))
except: print('')
" <<< "$INPUT" 2>/dev/null || echo "")

FILE_PATH=$(python3 -c "
import sys, json
try:
    d = json.loads(sys.stdin.read())
    ti = d.get('tool_input', {})
    print(ti.get('file_path', ''))
except: print('')
" <<< "$INPUT" 2>/dev/null || echo "")

# Edit / Write 도구만 처리
[[ "$TOOL_NAME" == "Edit" || "$TOOL_NAME" == "Write" ]] || exit 0
[ -n "$FILE_PATH" ] || exit 0

# scope.md 자체 수정은 무시 (카운터 업데이트가 재귀 호출되지 않도록)
echo "$FILE_PATH" | grep -qE 'scope\.md$' && exit 0

# ── sprint 번호 → scope.md 경로 결정 ─────────────────────────────────────
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
SPRINT_N=$(echo "$BRANCH" | grep -oE '(sprint|Sprint)([0-9]+)' | grep -oE '[0-9]+' | head -1)
[ -n "$SPRINT_N" ] || exit 0

SCOPE_MD="docs/sprint/sprint${SPRINT_N}/scope.md"
[ -f "$SCOPE_MD" ] || exit 0

# ── 절대 경로 → 상대 경로 변환 ──────────────────────────────────────────
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
RELATIVE_PATH="${FILE_PATH#$REPO_ROOT/}"

# ── scope.md 테이블에서 해당 파일 찾아 횟수 증가 ─────────────────────────
RESULT=$(python3 << PYEOF
import re, os, sys

scope_path = "$SCOPE_MD"
file_path  = "$RELATIVE_PATH"
basename   = os.path.basename(file_path)

with open(scope_path, 'r', encoding='utf-8') as f:
    lines = f.read().split('\n')

updated_count = None

for i, line in enumerate(lines):
    # 마크다운 테이블 행만 처리
    if not line.strip().startswith('|'):
        continue
    # 파일 경로 또는 파일명으로 매칭
    if file_path not in line and basename not in line:
        continue

    # [N회] 또는 [N회 ⚠️] 패턴 검색
    match = re.search(r'\[(\d+)회[^\]]*\]', line)
    if not match:
        continue

    current   = int(match.group(1))
    new_count = current + 1
    new_marker = f'[{new_count}회 ⚠️]' if new_count >= 3 else f'[{new_count}회]'

    lines[i]      = line[:match.start()] + new_marker + line[match.end():]
    updated_count = new_count
    break

if updated_count is not None:
    with open(scope_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines))
    print(updated_count)
else:
    print('not_found')
PYEOF
)

# ── 결과 처리 ─────────────────────────────────────────────────────────────
if [ "$RESULT" = "not_found" ] || [ -z "$RESULT" ]; then
  exit 0
fi

COUNT="$RESULT"

if [ "$COUNT" -ge 3 ] 2>/dev/null; then
    echo ""
    echo "🔁 [scope-tracker] 동일 파일 ${COUNT}회 수정 — 루프 감지 조건 충족"
    echo ""
    echo "  파일   : $RELATIVE_PATH"
    echo "  scope  : $SCOPE_MD"
    echo ""
    echo "  → loop-detection 스킬을 즉시 실행하세요."
    echo "  → 추가 수정 전 사용자 승인을 받아야 합니다."
    echo ""
    log_event "scope-tracker" "WARN" "loop-3x" "$RELATIVE_PATH"
    exit 1  # non-blocking 경고 (작업 차단하지 않음)
fi

# ── P3-A: Re-planning 트리거 감지 (30% 초과 경고) ──────────────────────────
# scope.md 선언 파일 수 vs 실제 수정된 파일 수를 비교
# 30% 이상 초과 시 Re-planning 트리거 조건 안내 (비차단)
DECLARED=$(python3 -c "
import re, sys
try:
    with open('$SCOPE_MD', 'r', encoding='utf-8') as f:
        lines = f.readlines()
    count = sum(1 for l in lines if l.strip().startswith('|') and re.search(r'\[\d+회', l))
    print(count)
except: print(0)
" 2>/dev/null || echo "0")

MODIFIED=$(python3 -c "
import re, sys
try:
    with open('$SCOPE_MD', 'r', encoding='utf-8') as f:
        lines = f.readlines()
    count = sum(1 for l in lines if l.strip().startswith('|') and re.search(r'\[[1-9]\d*회', l))
    print(count)
except: print(0)
" 2>/dev/null || echo "0")

if [ "$DECLARED" -gt 2 ] && [ "$MODIFIED" -gt 0 ]; then
  # 초과 비율 계산 (정수 산술)
  EXCESS=$(( (MODIFIED * 100 / DECLARED) ))
  if [ "$EXCESS" -ge 130 ]; then
    echo ""
    echo "⚠️  [scope-tracker] Re-planning 트리거 감지: 수정 파일(${MODIFIED})이 선언(${DECLARED}) 대비 ${EXCESS}%"
    echo "  → Harness 원칙 1의 Re-planning 트리거 조건(30% 초과)을 확인하세요."
    echo "  → scope.md를 업데이트하거나 사용자에게 범위 조정을 보고하세요."
    echo ""
    log_event "scope-tracker" "WARN" "replanning-trigger" "${MODIFIED}vs${DECLARED}"
  fi
fi

exit 0
