# 프롬프트 가이드

Claude Code에서 어떤 프롬프트를 사용해야 할지 안내합니다.

---

## 작업 경로 선택 매트릭스

상황에 따라 아래 경로 중 하나를 선택하세요:

| 경로 | 상황 | 시작 | 마무리 | 모델 |
|------|------|------|--------|------|
| **A** | 새 프로젝트 시작 (PRD 완료) | `prd-to-roadmap` → `sprint-planner` → `/sprint-dev` | `sprint-close` → `sprint-review` | Opus→Sonnet |
| **B** | 대규모 기능 (3스프린트+) | `phase-planner` → `sprint-planner` → `/sprint-dev` | `sprint-close` → `sprint-review` | Opus→Sonnet |
| **C** | 일반 스프린트 (1~2스프린트) | `sprint-planner` → `/sprint-dev` | `sprint-close` → `sprint-review` | Opus→Sonnet |
| **D** | 긴급 버그 수정 (Hotfix) | `hotfix/{설명}` 브랜치 직접 생성 후 구현 | `hotfix-close` | Sonnet |
| **E** | 프로덕션 배포 (QA 완료 후) | — (develop QA 통과 후) | `deploy-prod` | Sonnet |

> 모델은 에이전트가 자동 선택합니다. 별도 설정 불필요.

---

## 경로별 핵심 프롬프트

### A. 새 프로젝트 시작

```
# 1. PRD 완성 후 ROADMAP 생성
"PRD.md 작성 완료했어. ROADMAP 만들어줘."

# 2. ROADMAP 검토 후 첫 스프린트 계획
"ROADMAP 검토했어. sprint 1 계획 세워줘."

# 3. 계획 확인 후 구현 시작
"/sprint-dev 1"
```

### B. 대규모 기능 설계

```
# 1. Phase 설계
"[기능명] 구현하려는데 Phase 설계해줘."

# 2. Phase 확인 후 스프린트 계획
"Phase 설계 확인했어. sprint {n} 계획 세워줘."

# 3. 구현 시작
"/sprint-dev {n}"
```

### C. 일반 스프린트

```
# 1. 스프린트 계획
"sprint {n} 계획 세워줘. [목표 설명]"

# 2. 구현 시작
"/sprint-dev {n}"

# 3. 구현 완료 후 마무리 (두 단계)
"sprint {n} 구현 완료했어. sprint-close 실행해줘."
"sprint-review 실행해줘."
```

### D. Hotfix

```
# 1. Hotfix 기준 확인 (파일 3개↓, 코드 50줄↓, DB 변경 없음)
# 2. main 기반 브랜치 생성
git checkout main && git checkout -b hotfix/{설명}

# 3. 구현 완료 후
"hotfix 구현 끝났어. 마무리해줘."

# 4. main merge 후 develop 역머지
"main merge 완료됐어. develop 역머지 해줘."
```

### E. 프로덕션 배포

```
# develop QA 완료 후
"수동 검증 완료했고 develop QA 통과했어. 프로덕션 배포 준비해줘."
```

---

## 새 세션에서 작업 이어가기

Claude Code 세션이 끊겨도 sprint{n}.md가 SSOT이므로 간단히 재진입할 수 있습니다:

```
# 구현 중 재진입
"/sprint-dev {n}"

# 특정 작업부터 이어서
"sprint{n}.md의 Task 4부터 이어서 구현해줘."

# sprint-review 재실행 (이슈 수정 후)
"이슈 수정 완료했어. sprint-review 다시 실행해줘."
```

---

## Notion 업데이트 (Notion MCP 연결 시)

스프린트 완료 또는 배포 후 Notion 문서를 최신화할 수 있습니다:

```
"이번 스프린트 결과 Notion에 반영해줘."
"API 명세 Notion에 업데이트해줘."
"배포 완료됐어. Notion 릴리즈 노트 작성해줘."
```

> Notion 업데이트 전에 `.claude/rules/notion.md`의 페이지 ID를 입력해 두세요.
