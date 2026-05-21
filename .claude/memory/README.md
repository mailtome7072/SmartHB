# 사용자 메모리 동기화 (릴레이 개발용)

본 디렉토리는 Claude Code **사용자 메모리(user-scope memory)** 의 저장소 미러입니다.
다른 환경(Mac, 다른 PC 등)에서 clone 후 한 줄 명령으로 메모리를 적용하여 **세션 컨텍스트
없이도 같은 협업 패턴을 즉시 이어갈 수 있도록** 하는 것이 목적입니다.

## 왜 이 패턴이 필요한가

Claude Code 의 사용자 메모리(`~/.claude/projects/{hash}/memory/`) 는 **로컬 디스크에만**
저장됩니다. 다른 환경에서 같은 저장소를 clone 해도 이 메모리는 따라오지 않아 — Claude 가
이미 알고 있던 컨텍스트(PR 정책, 알려진 트랩, sprint 진행 카드)를 매번 재학습해야 합니다.

본 디렉토리에 메모리를 commit 해두면 clone + 한 번의 sync 명령으로 모든 환경에서 동일한
컨텍스트를 보유할 수 있습니다.

## 디렉토리 위치

| 환경 | Claude Code 사용자 메모리 경로 |
|------|--------------------------------|
| Windows | `%USERPROFILE%\.claude\projects\{hash}\memory\` (예: `C--Projects-SmartHB`) |
| macOS | `~/.claude/projects/{hash}/memory/` |
| Linux | `~/.claude/projects/{hash}/memory/` |

`{hash}` 는 프로젝트 절대경로를 Claude Code 가 변환한 슬러그 — 처음 Claude Code 를 그 환경에서
실행한 직후 `~/.claude/projects/` 아래 어떤 디렉토리가 생성되는지 확인하면 됩니다.

## ⚡ 다른 환경에서 첫 Claude 세션 — 권장 문구 (옵션 B)

clone 후 별도 명령 없이 Claude 에게 다음 한 줄만 입력하면 됩니다 (옵션 B). Claude 가
**현재 브랜치 확인 → develop 전환 → 사용자 메모리 sync → 컨텍스트 파악 → 다음 작업
진입**을 자동 수행합니다.

```
develop 브랜치로 가서 .claude/memory/ 의 사용자 메모리를 ~/.claude/projects/ 의 본 프로젝트 디렉토리로 복사한 후, sprint-next-session.md 카드대로 다음 작업을 이어줘.
```

또는 더 짧게:

```
develop 으로 가서 .claude/memory/ sync 후 다음 작업 이어줘.
```

Claude 가 수행하는 흐름:
1. `git status` → 현재 브랜치 확인 (clone 직후는 master = v0.2.0 시점)
2. `git checkout develop && git pull` → 최신 작업 브랜치로 전환
3. `~/.claude/projects/` 에서 본 프로젝트 hash 디렉토리 식별
4. `.claude/memory/*.md` → 사용자 메모리 디렉토리 복사 (cp 또는 Copy-Item)
5. `MEMORY.md` 인덱스 + `sprint-next-session.md` 카드 확인
6. CLAUDE.md / ROADMAP / agent-memory 참조하여 다음 sprint 진입 계획 제안

> Claude Code 가 첫 실행 시 자동 생성하는 `~/.claude/projects/{hash}/` 디렉토리가 아직 없다면
> `mkdir -p` 로 사전 생성 가능 — Claude 가 자체적으로 처리합니다.

---

## 동기화 절차

### 새 환경에서 첫 적용 (clone 직후)

```bash
# 1. 한 번 Claude Code 를 SmartHB 디렉토리에서 실행 — 사용자 메모리 디렉토리 자동 생성
cd /path/to/SmartHB
claude  # 또는 ide 실행 후 종료

# 2. 생성된 디렉토리 확인 (macOS/Linux 예시)
ls ~/.claude/projects/   # SmartHB 관련 hash 디렉토리 식별

# 3. 저장소의 메모리 파일 → 사용자 메모리 디렉토리에 복사
cp .claude/memory/*.md ~/.claude/projects/{hash}/memory/

# Windows PowerShell
Copy-Item .claude\memory\*.md $env:USERPROFILE\.claude\projects\{hash}\memory\
```

### 메모리 변경 시 — 저장소 ↔ 사용자 메모리 양방향 동기화

**Claude 가 메모리를 추가/수정한 경우** (사용자 메모리에 먼저 반영됨):

```bash
# 사용자 메모리 → 저장소 (commit 전)
cp ~/.claude/projects/{hash}/memory/*.md .claude/memory/
git add .claude/memory/ && git commit -m "chore(memory): 사용자 메모리 동기화"
```

**다른 환경에서 pull 후** (저장소에 새 메모리가 들어온 경우):

```bash
git pull
cp .claude/memory/*.md ~/.claude/projects/{hash}/memory/
```

## 메모리 파일 목록 (현 시점)

| 파일 | 한 줄 설명 |
|------|----------|
| `MEMORY.md` | 인덱스 — 다른 4개 파일의 한 줄 hook 목록 (Claude 가 항상 컨텍스트에 로드) |
| `workflow-no-pr.md` | 단일 개발자 PR 단계 생략 정책 (`gh pr create` 금지, 직접 머지) |
| `sprint-next-session.md` | 다음 sprint 진입 카드 (현재 Sprint 5 — Phase 2 학사 스케줄) |
| `ntfs-power-loss-pattern.md` | NTFS power-loss 시 fs::write+rename NULL 손상 패턴 + 자동 복구 |
| `keyring-v3-features-trap.md` | `keyring = "3"` default features 만 쓰면 silent set fail — `apple-native`/`windows-native` 명시 필수 |

## 메모리 vs `.claude/agents/agent-memory/`

| 디렉토리 | 범위 | Git 추적 | 자동 로드 |
|---------|------|---------|----------|
| `.claude/agents/agent-memory/{agent}/` | 에이전트별 메모리 (sprint-planner 등) | ✅ | ✅ 해당 에이전트 호출 시 |
| `.claude/memory/` (본 디렉토리) | 사용자 메모리 미러 (수동 sync) | ✅ | ❌ Claude 가 직접 로드하지 않음 — 위 절차로 사용자 메모리 디렉토리에 복사해야 함 |
| `~/.claude/projects/{hash}/memory/` | Claude Code 사용자 메모리 (실제 로드 대상) | ❌ (로컬 only) | ✅ `MEMORY.md` 가 항상 컨텍스트에 |

## CLAUDE.md 협업 규칙

새 메모리를 추가하거나 수정한 후에는 **두 곳 모두 갱신**해야 다음 sprint/세션에서 일관성을
유지할 수 있습니다. 이 규칙은 CLAUDE.md "에이전트 공유 메모리" 다음 줄의 "사용자 메모리 미러
(릴레이 개발용)" 항목에 명시되어 있습니다.
