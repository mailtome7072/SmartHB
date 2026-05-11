# 하네스 엔지니어링 (Harness Engineering) 개요

이 문서는 ClaudeStarter 템플릿에 구현된 **하네스 엔지니어링 원칙**의 전체 구조를 설명합니다.

하네스 엔지니어링은 AI 코딩 에이전트의 자율성을 보장하면서도 명확한 가드레일로 안전성과 감사 가능성을 확보하는 접근법입니다. "말에 고삐를 채우되 달릴 수 있게 하는" 방식으로, 에이전트가 정해진 범위 안에서 최대한 효율적으로 작동하도록 설계합니다.

---

## 5대 원칙과 구현 위치

| 원칙 | 설명 | 구현 위치 |
|------|------|----------|
| **1. Planning First** | 코드 수정 전 scope.md 작성 | `sprint-dev` 0단계, `scope.md` 템플릿 |
| **2. Strict Guardrails** | 범위 외 변경 차단 | `posttooluse-code-validator.sh`, `harness-engineering.md` |
| **3. Verification Loops** | 3-retry 원칙, 동일 수정 반복 금지 | `sprint-dev` 검증 실패 대응, `harness-engineering.md` |
| **4. Policy Enforcement** | 배포 전 OPA 유사 정책 게이트 | `harness-ci-gate` 스킬, `deployment-policy.md` |
| **5. Continuous Verification** | 배포 후 자동 검증 및 롤백 트리거 | `deploy-prod` 에이전트, `continuous-verification.md` |

---

## 전체 흐름도

```
사용자 요청
    │
    ▼
[Plan First Gate]
sprint-dev 0단계: scope.md 작성
    │  ← 구조적 충돌 발견 시 → Step-back 프로토콜
    ▼
[코드 수정]
Edit/Write 도구
    │
    ▼ (자동)
[PostToolUse Hook]
posttooluse-code-validator.sh
  - .env 수정 차단
  - Python syntax 검증
  - 시크릿 패턴 감지
    │
    ▼
[Verification Loops]
테스트/린트 실행
  - 실패 시 3-retry (동일 수정 반복 금지)
  - 3회 실패 → 사용자 보고
    │
    ▼
sprint-close → sprint-review
    │
    ▼
[Pre-Deploy Policy Gate]
harness-ci-gate 스킬 실행
  - CI 통과 확인
  - CHANGELOG 업데이트 확인
  - 시크릿 grep 확인
  - risk-register 확인
    │  ← 미충족 항목 발견 → 배포 중단, 사용자 보고
    ▼
deploy-prod (PR to main)
    │
    ▼
GitHub Actions 자동 배포
    │
    ▼
[Continuous Verification]
CV 3단계 체크리스트
  - 기술 검증 (즉시)
  - 기능 검증 (5분 후)
  - 안정성 판단 (30분 후)
    │  ← 실패 시 → 즉시 롤백 안내
    ▼
배포 완료 ✅
```

---

## CI/CD에서의 Harness Engineering

`.github/workflows/ci.yml`에 추가된 Harness 관련 잡:

| 잡 이름 | 적용 원칙 | 실행 조건 |
|---------|----------|----------|
| `security-scan` | Policy Enforcement | 모든 PR |
| `validate-hotfix-scope` | Strict Guardrails | `hotfix/*` 브랜치 PR만 |

---

## 상세 문서

| 문서 | 설명 |
|------|------|
| `deployment-policy.md` | 배포 가능 조건 정의 (OPA 유사 정책) |
| `continuous-verification.md` | CV 3단계 체크리스트 및 롤백 트리거 |

---

## 규칙 및 훅 파일

| 파일 | 유형 | 역할 |
|------|------|------|
| `.claude/rules/harness-engineering.md` | Rule (always-loaded) | 5대 원칙 행동 지침 |
| `.claude/hooks/posttooluse-code-validator.sh` | PostToolUse Hook | 코드 수정 즉각 검증 |
| `.claude/skills/harness-ci-gate.md` | Skill | 배포 전 Policy Gate 체크리스트 |
