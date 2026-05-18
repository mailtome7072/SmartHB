#!/usr/bin/env bash
# stop-doc-checker.sh
# Claude Code Stop Hook — 에이전트 완료 후 문서 누락 자동 감지
#
# 동작: git 변경 파일 패턴 분석 → 에이전트 유형 추론 → 필수 문서 검증 → 경고 출력
# 규칙 정의: .claude/hooks/lib/doc-rules.json 참조

set -uo pipefail

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
WARNINGS=()

# log-helper 로드 (없으면 no-op 함수 정의)
if [ -f ".claude/hooks/lib/log-helper.sh" ]; then
  source ".claude/hooks/lib/log-helper.sh"
else
  LOG_DIR=".claude/logs"
  log_event() { :; }
fi

# ── 변경 파일 목록 수집 ─────────────────────────────────────────────
# 스테이징 + 비스테이징 + 최근 커밋 변경분 모두 포함
STAGED=$(git diff --cached --name-only 2>/dev/null || echo "")
UNSTAGED=$(git diff --name-only 2>/dev/null || echo "")
RECENT=$(git diff HEAD~1 --name-only 2>/dev/null || echo "")
CHANGED_FILES=$(printf "%s\n%s\n%s" "$STAGED" "$UNSTAGED" "$RECENT" | sort -u | grep -v '^$' || echo "")

CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "")

# ── 에이전트 유형 감지 ──────────────────────────────────────────────
# 변경 파일 패턴으로 어떤 에이전트가 실행됐는지 추론

AGENT=""

# phase-planner: docs/phase/ 하위 파일 생성
if echo "$CHANGED_FILES" | grep -qE '^docs/phase/'; then
  AGENT="phase-planner"

# sprint-review: docs/test-reports/ 하위 파일 생성
elif echo "$CHANGED_FILES" | grep -qE '^docs/test-reports/'; then
  AGENT="sprint-review"

# sprint-planner: docs/sprint/sprint{N}.md 생성 (sprint-close보다 먼저 확인)
# — sprint-close는 ROADMAP+CHANGELOG+DEPLOY 3개를 동시 수정하므로 아래에서 구분됨
elif echo "$CHANGED_FILES" | grep -qE '^docs/sprint/sprint[0-9]+\.md$' \
  && ! echo "$CHANGED_FILES" | grep -q "^CHANGELOG\.md$"; then
  AGENT="sprint-planner"

# sprint-close: ROADMAP.md + CHANGELOG.md + DEPLOY.md 동시 수정
elif echo "$CHANGED_FILES" | grep -q "^ROADMAP\.md$" \
  && echo "$CHANGED_FILES" | grep -q "^CHANGELOG\.md$" \
  && echo "$CHANGED_FILES" | grep -q "^DEPLOY\.md$"; then
  AGENT="sprint-close"

# hotfix-close: hotfix/* 브랜치 + DEPLOY.md 수정
elif echo "$CURRENT_BRANCH" | grep -q "^hotfix/" && echo "$CHANGED_FILES" | grep -q "^DEPLOY\.md$"; then
  AGENT="hotfix-close"

# prd-to-roadmap: ROADMAP.md만 수정 (다른 패턴과 겹치지 않을 때)
elif echo "$CHANGED_FILES" | grep -q "^ROADMAP\.md$" && [ -z "$AGENT" ]; then
  AGENT="prd-to-roadmap"
fi

# 에이전트 미감지 시 조용히 종료
[ -n "$AGENT" ] || exit 0

# ── 검증 헬퍼 함수 ──────────────────────────────────────────────────

warn() {
  WARNINGS+=("$1")
}

# required: 파일이 이번 작업에서 수정되었는지 확인
check_required() {
  local file="$1"
  local msg="$2"
  if ! echo "$CHANGED_FILES" | grep -qF "$file"; then
    warn "📄 미수정: $file — $msg"
  fi
}

# grep_content: 파일 내 패턴 존재 여부 확인
check_grep_content() {
  local file="$1"
  local pattern="$2"
  local msg="$3"
  if [ -f "$PROJECT_ROOT/$file" ]; then
    if ! grep -qE "$pattern" "$PROJECT_ROOT/$file" 2>/dev/null; then
      warn "🔍 내용 누락: $file — $msg"
    fi
  fi
}

# glob_min: 패턴에 일치하는 파일 최소 N개 확인
check_glob_min() {
  local pattern="$1"
  local min="$2"
  local msg="$3"
  local count
  count=$(find "$PROJECT_ROOT" -path "$PROJECT_ROOT/$pattern" -type f 2>/dev/null | wc -l | tr -d ' ')
  if [ "$count" -lt "$min" ]; then
    warn "📁 파일 부족: '$pattern' — $msg (현재 ${count}개, 최소 ${min}개 필요)"
  fi
}

# checkbox_remaining: 파일에 ⬜ 미완료 체크박스가 있는지 확인
check_checkbox_remaining() {
  local file="$1"
  local msg="$2"
  if [ -f "$PROJECT_ROOT/$file" ] && grep -q "⬜" "$PROJECT_ROOT/$file" 2>/dev/null; then
    warn "☑️  미완료 항목: $file — $msg"
  fi
}

# hotfix_scope: 변경 파일/줄 수 범위 초과 확인
check_hotfix_scope() {
  local max_files="$1"
  local max_lines="$2"
  local msg="$3"

  local file_count
  file_count=$(echo "$CHANGED_FILES" | grep -v '^$' | grep -vE '\.md$' | wc -l | tr -d ' ')

  local line_count=0
  if git diff HEAD~1 --stat &>/dev/null; then
    line_count=$(git diff HEAD~1 --shortstat 2>/dev/null | grep -oE '[0-9]+ insertion' | grep -oE '^[0-9]+' || echo 0)
  fi

  if [ "$file_count" -gt "$max_files" ]; then
    warn "📏 범위 초과: 변경 파일 ${file_count}개 (최대 ${max_files}개) — $msg"
  fi
  if [ "$line_count" -gt "$max_lines" ]; then
    warn "📏 범위 초과: 변경 코드 ${line_count}줄 (최대 ${max_lines}줄) — $msg"
  fi
}

# ── 에이전트별 검증 규칙 ────────────────────────────────────────────
case "$AGENT" in

  "sprint-planner")
    check_required "ROADMAP.md" \
      "스프린트 상태를 📋 예정 → 🔄 진행 중으로 업데이트하세요."
    check_required ".claude/agents/agent-memory/sprint-planner/MEMORY.md" \
      "에이전트 메모리를 갱신하세요."
    ;;

  "sprint-close")
    check_required "DEPLOY.md" \
      "새 배포 검증 항목을 추가하세요."
    check_required "CHANGELOG.md" \
      "이번 스프린트 변경 사항을 기록하세요."
    check_checkbox_remaining "ROADMAP.md" \
      "ROADMAP.md에 완료되지 않은 항목이 있습니다. 스프린트 완료 상태를 확인하세요."
    ;;

  "sprint-review")
    check_required "DEPLOY.md" \
      "sprint-review 완료 항목(⬜ → ✅)을 업데이트하세요."
    check_grep_content "DEPLOY.md" "(pytest|Playwright|curl|✅ sprint-review)" \
      "DEPLOY.md에 자동 검증 결과(pytest/Playwright)가 기록되지 않았습니다."
    ;;

  "hotfix-close")
    check_hotfix_scope 3 50 \
      "핫픽스 기준(파일 3개·50줄 이하)을 초과했습니다. Sprint 프로세스를 고려하세요."
    check_required "DEPLOY.md" \
      "핫픽스 검증 항목을 추가하세요."
    ;;

  "phase-planner")
    check_required "ROADMAP.md" \
      "Phase 항목 상태를 업데이트하세요."
    check_glob_min "docs/phase/*-review.md" 1 \
      "전문가 검토 결과 파일을 최소 1개 생성하세요. (예: docs/phase/phase1/phase1-PO-review.md)"
    ;;

  "prd-to-roadmap")
    check_required "ROADMAP.md" \
      "ROADMAP.md를 생성하거나 업데이트하세요."
    ;;
esac

# ── 결과 출력 ────────────────────────────────────────────────────────
if [ ${#WARNINGS[@]} -gt 0 ]; then
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  📋 문서 누락 감지 — 에이전트: $AGENT"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  for w in "${WARNINGS[@]}"; do
    echo "  $w"
  done
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  규칙 상세: .claude/hooks/lib/doc-rules.json"
  echo ""
  # 문서 누락 이벤트를 로그에 기록
  for w in "${WARNINGS[@]}"; do
    log_event "doc-checker" "WARN" "$AGENT" "$w"
  done
fi

# ── session-summary.md 업데이트 ────────────────────────────────────────────
# 세션 종료 시 오늘의 로그를 분석하여 다음 세션 재활용 가능한 패턴 요약 작성
_update_session_summary() {
  local summary_file="$LOG_DIR/session-summary.md"
  local today
  today=$(date +%Y-%m-%d)
  local log_file="$LOG_DIR/$today.log"

  mkdir -p "$LOG_DIR" 2>/dev/null || return 0

  local block_count=0 warn_count=0
  if [ -f "$log_file" ]; then
    block_count=$(grep -c "|BLOCK|" "$log_file" 2>/dev/null || echo "0")
    warn_count=$(grep -c  "|WARN|"  "$log_file" 2>/dev/null || echo "0")
  fi

  {
    echo "# 세션 로그 요약 — $today"
    echo ""
    echo "## 이벤트 통계"
    echo "- BLOCK: ${block_count}건 | WARN: ${warn_count}건"
    echo ""
    echo "## 문서 누락 패턴 (doc-checker)"
    if [ ${#WARNINGS[@]} -gt 0 ]; then
      for w in "${WARNINGS[@]}"; do echo "- $w"; done
    else
      echo "- 없음"
    fi
    echo ""
    echo "## 코드 위반 패턴 (code-validator / bash-guard)"
    if [ -f "$log_file" ] && grep -q "|BLOCK|" "$log_file" 2>/dev/null; then
      grep "|BLOCK|" "$log_file" | awk -F'|' '{print $4": "$5}' | sort | uniq -c | sort -rn | head -5 | sed 's/^/- /'
    else
      echo "- 없음"
    fi
    echo ""
    echo "## 루프 감지 이벤트 (scope-tracker)"
    if [ -f "$log_file" ] && grep -q "loop-3x" "$log_file" 2>/dev/null; then
      grep "loop-3x" "$log_file" | awk -F'|' '{print $5}' | sort | uniq -c | sed 's/^/- /'
    else
      echo "- 없음"
    fi
    echo ""
    echo "## 다음 세션 참고사항"
    echo "> sprint-dev 0단계 재진입 시 이 파일을 읽어 반복 위반 패턴을 확인하세요."
    echo "> 3회 이상 반복된 패턴은 sprint-planner MEMORY.md에 기록하세요."
  } > "$summary_file" 2>/dev/null || true
}

_update_session_summary

exit 0
