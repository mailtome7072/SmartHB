# /sprint-dev

sprint{n}.md 계획 문서를 기반으로 구현 단계에 진입합니다. 브랜치 생성, 구현 현황 파악, 가이드라인 적용을 자동으로 처리합니다.

> **실행 주체**: 이 커맨드는 **사용자가 직접** Claude Code에 입력하는 슬래시 커맨드입니다. 에이전트가 대신 실행하지 않으며, sprint-planner 완료 후 사용자가 직접 `/sprint-dev {n}`을 입력해야 합니다.

## 사용법

```
/sprint-dev [n]
```

- `n`: 스프린트 번호 (생략 시 현재 브랜치명에서 자동 추출)

## 실행 절차

### 0단계: Scope 선언 (Plan First) — Harness Engineering 원칙 1

코드 수정 전 `docs/sprint/sprint{n}/scope.md`를 작성하거나 확인한다.

**신규 세션 (scope.md 없음)**: 아래 템플릿으로 파일을 생성한 후 1단계로 진행.

```markdown
---
Sprint: {n}  |  Date: YYYY-MM-DD  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/... | [0회] | (수정 목적) |
| src/... | [0회] | (수정 목적) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] (기타 scope 외 파일)

## 완료 기준 (이번 세션)
- [ ] (sprint{n}.md의 완료 기준 중 이번 세션 담당 항목)
```

**재진입 세션 (scope.md 있음)**: 파일을 읽어 이전 상태를 확인하고 세션 번호를 +1 한 뒤 이어서 진행.

**최근 세션 패턴 검토** (재진입 세션에서 권장):
`.claude/logs/session-summary.md`가 있으면 읽어 이전 세션의 반복 위반 패턴을 파악합니다.
- `BLOCK` 이벤트가 많은 파일 → scope 선언 또는 허가 토큰 사전 준비
- doc-checker 반복 경고 → 해당 문서를 scope.md 완료 기준에 명시
- `loop-3x` 이벤트 → 해당 파일은 신중히 접근, 사전 설계 강화

**Step-back 프로토콜**: 예상치 못한 구조적 충돌 발견 시 코드 수정 즉시 중단 → scope.md `## 발견된 이슈` 섹션에 기록 → 사용자에게 보고.

**Re-planning 트리거**: 실제 수정 파일이 scope.md 선언 대비 30% 초과 또는 새 의존성/DB 변경 필요 시 → scope.md 업데이트 후 사용자 확인.

---

### 1단계: 스프린트 번호 결정

`$ARGUMENTS`가 있으면 해당 번호를 사용합니다.
없으면 현재 브랜치명에서 `sprint` 뒤의 숫자를 추출합니다:
- `sprint3`, `sprint3-auth`, `sprint-3` → `n=3` ✅
- `develop`, `main`, `feature/...` 등 숫자 미발견 → `ROADMAP.md`의 `🔄 진행 중` 항목 확인
여전히 불명확하면 사용자에게 직접 확인합니다.

### 2단계: 컨텍스트 로드

다음 파일을 순서대로 읽습니다:
1. `docs/sprint/sprint{n}.md` — 작업 목록, 완료 기준, 기술적 접근 방법
2. `ROADMAP.md` — 해당 스프린트 목표 및 Phase 위치 확인
3. `git log develop..HEAD --oneline` — 이미 완료된 커밋 확인 (재진입 시)
4. `git status` — 현재 작업 상태

### 3단계: 브랜치 확인 및 생성

현재 브랜치가 `sprint{n}`인지 확인합니다:

- **브랜치 생성 전 `git pull origin develop` 반드시 실행** (fetch만으로는 부족; 로컬 develop에 반영 필요):
  ```bash
  git checkout develop && git pull origin develop
  ```
- `sprint{n}` 브랜치가 없으면 develop 기반으로 생성:
  ```bash
  git checkout -b sprint{n}
  ```
- `sprint{n}` 브랜치가 이미 있으면 전환:
  ```bash
  git checkout sprint{n}
  ```

### 4단계: 구현 현황 보고

사용자에게 다음 정보를 보고합니다:

```
Sprint {n} 구현 모드 진입

계획 문서: docs/sprint/sprint{n}.md
현재 브랜치: sprint{n}

작업 목록:
  ✅ (이미 완료된 커밋에서 추론한 항목)
  ⬜ (남은 작업 항목들)

완료 기준 (Definition of Done):
  - (sprint{n}.md의 완료 기준 항목들)
```

새 세션에서 재진입하는 경우 "마지막으로 완료된 작업" 정보를 명시합니다.

### 5단계: 구현 가이드라인 적용

코드 작성 시 다음을 자동 준수합니다:
- **karpathy-guidelines**: 파일 수정 전 반드시 읽기, git diff 확인, --no-verify 금지
- **백엔드 파일 접근 시**: `.claude/rules/backend.md` 자동 로드
- **프론트엔드 파일 접근 시**: `.claude/rules/frontend.md` 자동 로드
- **커밋**: 작업 단위별로 의미있는 커밋 메시지 (한국어)

### 6단계: Task 레벨 스킬 실행 (자동)

각 Task를 구현할 때 다음 순서로 스킬을 적용합니다:

**① Task 시작 전 — 선언 스킬 로드**

`sprint{n}.md`의 Task 항목에 `skill:` 이 명시된 경우, 구현 전에 해당 스킬을 읽고 적용합니다.

```
- ⬜ 로그인 API 버그 수정 · skill: systematic-debugging
```

| 스킬 이름 | 적합한 작업 유형 |
|----------|----------------|
| `systematic-debugging` | 버그 수정, 원인 불명 오류 추적 |
| `karpathy-guidelines` | 전체 구현 원칙 재확인이 필요한 복잡한 Task |
| `code-review` | 중요 로직 자기 검토 |
| `test-checklist` | 테스트 작성 Task |

`skill:` 이 없는 Task는 이 단계를 건너뜁니다.

**② Task 완료 후 — Self-verify + simplify (생략 불가)**

모든 Task 구현 완료 후 아래 순서를 반드시 따릅니다:

```
[Task 구현] → [Self-verify] → [simplify 실행] → [커밋]
```

**Self-verify** (커밋 전 의무 단계):
- 백엔드(Rust) 파일 변경 시: `cargo test` (src-tauri/ 기준)
- 백엔드 정적 분석: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- 프론트엔드 파일 변경 시: `pnpm lint` + `pnpm tsc --noEmit`
- Self-verify 실패 → 검증 실패 대응(3회 재시도 원칙) 적용

**simplify** (Self-verify 통과 후):
`.claude/skills/simplify.md` 스킬을 실행합니다.
- 불필요한 추상화, 중복 로직, 사용하지 않는 코드를 제거한다.
- 기능 변경 없이 코드 구조만 단순화한다.
- simplify 결과를 한 줄로 보고한 뒤 커밋한다.

### 검증 실패 대응 (3회 재시도 원칙)

Task 구현 중 테스트/린트 실패 발생 시 아래 절차를 따릅니다:

1. **1차 실패**: 오류 메시지를 분석하여 원인을 파악한 뒤 수정하고 재검증합니다.
2. **2차 실패**: 다른 접근 방법으로 재시도합니다. (1차와 동일한 수정 반복 금지)
3. **3차 실패**: 즉시 중단하고 사용자에게 다음 내용을 보고합니다.
   - 실패한 테스트·오류 내용
   - 시도한 수정 방법 (1차, 2차)
   - 막힌 이유

**루프 상태 전환 조건** — 아래 중 하나 충족 시 → `.claude/skills/loop-detection.md` 스킬 **즉시** 실행:
- 동일한 오류 메시지로 테스트가 **3회 이상 연속 실패**
- 동일한 파일을 **3회 이상 반복 수정** (scope.md에 `[3회 ⚠️]` 기록 시)

**CI 파이프라인 실패** (cargo test, cargo clippy, pnpm lint, pnpm build):
`docs/dev-process.md` 섹션 9.1을 참조하여 로컬에서 원인을 먼저 재현합니다.

**재시도 금지 패턴**: 동일한 코드 변경 반복 / `--no-verify` 우회 시도.

---

### 완료 신호

sprint-close를 실행하기 **전에** 아래 완료 조건을 먼저 확인합니다:

1. `git log develop..HEAD --oneline` — 모든 Task 커밋이 반영되었는지 확인
2. `docs/sprint/sprint{n}.md`의 완료 기준(Definition of Done) — 전 항목 `⬜ → ✅` 전환 여부 확인
3. `docs/sprint/sprint{n}/scope.md` — 수정 파일 목록 중 미완료 항목 없음 확인

모든 항목 확인 후:

> "sprint{n} 구현 완료했어. sprint-close 실행해줘."

sprint-close 완료 후:

> "sprint-review 실행해줘."

## 중간 재진입 (새 세션에서 이어서 구현)

새 세션에서 이어서 구현할 때:

> "/sprint-dev {n}"

sprint{n}.md가 SSOT이므로 이 문서 하나만 읽으면 어디서든 진행 상황을 파악하고 이어서 구현할 수 있습니다.
