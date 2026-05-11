# 배포 정책 (Deployment Policy)

> **용도**: Harness Engineering 원칙 4 — Policy Enforcement  
> **참조 스킬**: `.claude/skills/harness-ci-gate.md`  
> **사용 에이전트**: `deploy-prod`, `sprint-review`

이 문서는 배포 가능 조건을 정의합니다. 모든 조건이 충족되어야 배포를 진행합니다.
OPA(Open Policy Agent) 개념을 Claude Code 워크플로우에 맞게 적용한 형태입니다.

---

## 배포 가능 조건 (Deploy Gate)

### Category 1: Code Quality

| # | 조건 | 검증 방법 | 미충족 시 |
|---|------|----------|---------|
| 1.1 | CI(GitHub Actions) 통과 | GitHub PR 상태 확인 | **배포 차단** |
| 1.2 | pytest 전체 통과 | CI 결과 또는 로컬 재실행 | **배포 차단** |
| 1.3 | Docker 빌드 성공 | CI 결과 확인 | **배포 차단** |

### Category 2: Security

| # | 조건 | 검증 방법 | 미충족 시 |
|---|------|----------|---------|
| 2.1 | 코드에 하드코딩된 시크릿 없음 | `grep -rn "password\|secret\|api_key" --include="*.py" app/backend/` | **배포 차단** |
| 2.2 | `.env` 파일이 Git에 포함되지 않음 | `git status --short \| grep .env` | **배포 차단** |
| 2.3 | bandit high severity 없음 (선택) | CI security-scan 잡 결과 | 사용자 확인 후 결정 |

### Category 3: Documentation

| # | 조건 | 검증 방법 | 미충족 시 |
|---|------|----------|---------|
| 3.1 | CHANGELOG.md `[Unreleased]` 섹션 업데이트됨 | `grep "\[Unreleased\]" CHANGELOG.md` | **배포 차단** |
| 3.2 | DEPLOY.md sprint-review 완료 항목 체크됨 | DEPLOY.md `✅ sprint-review` 항목 확인 | **배포 차단** |
| 3.3 | ROADMAP.md 해당 스프린트 `✅ 완료` 상태 | ROADMAP.md 확인 | 사용자 확인 후 결정 |

### Category 4: Process

| # | 조건 | 검증 방법 | 미충족 시 |
|---|------|----------|---------|
| 4.1 | sprint-review 에이전트 완료됨 | DEPLOY.md 또는 docs/test-reports/ 파일 확인 | **배포 차단** |
| 4.2 | risk-register Medium+ 이슈 인지 및 승인됨 | docs/risk-register/ 최근 파일 확인 | 사용자 확인 후 결정 |
| 4.3 | 현재 브랜치가 `develop`임 | `git branch --show-current` | **배포 차단** |

---

## 정책 위반 처리 기준

| 등급 | 미충족 항목 | 처리 |
|------|-----------|------|
| **BLOCK** | 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 3.2, 4.1, 4.3 | 배포 즉시 차단, 미충족 항목 목록 보고 |
| **CONFIRM** | 2.3, 3.3, 4.2 | 사용자에게 확인 요청, 승인 시 배포 진행 |

---

## Hotfix 배포 시 적용 정책

Hotfix는 속도가 중요하므로 일부 조건을 경량화합니다:

| 조건 | Sprint 배포 | Hotfix 배포 |
|------|------------|------------|
| CI 통과 | BLOCK | BLOCK |
| pytest 통과 | BLOCK | BLOCK (타겟 테스트만) |
| CHANGELOG 업데이트 | BLOCK | CONFIRM |
| sprint-review 완료 | BLOCK | 해당 없음 (hotfix-close가 대신) |
| risk-register 확인 | CONFIRM | CONFIRM |

---

## 정책 변경 이력

| 날짜 | 변경 내용 | 변경 사유 |
|------|----------|---------|
| 최초 작성 | 기본 배포 정책 정의 | Harness Engineering 원칙 4 도입 |

> 정책 변경 시 이 테이블에 이력을 추가하세요.
