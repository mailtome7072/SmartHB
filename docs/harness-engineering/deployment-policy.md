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
| 1.2 | cargo test 전체 통과 | CI 결과 또는 로컬 재실행 | **배포 차단** |
| 1.3 | pnpm build 성공 (Next.js static export) | CI 결과 확인 | **배포 차단** |

### Category 2: Security

| # | 조건 | 검증 방법 | 미충족 시 |
|---|------|----------|---------|
| 2.1 | 코드에 하드코딩된 시크릿 없음 | `grep -rn "password\|secret\|api_key" --include="*.rs" --include="*.ts" src-tauri/ src/` | **배포 차단** |
| 2.2 | `.env` 파일이 Git에 포함되지 않음 | `git ls-files .env .env.*` | **배포 차단** |

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

### Category 5: Data Integrity (PRD §5.3~§5.5, §6.6)

> SmartHB는 단일 사용자 데스크톱 앱 + 로컬 SQLite + 클라우드 동기화 폴더 모델이다. 데이터 손상이나 키 유출은 사용자 1인에게 즉시 전파되므로 별도 게이트로 관리한다.
> Sprint 1~2 단계에서는 아직 해당 기능이 미구현이므로 게이트 등급을 **CONFIRM** 으로 두고, 데이터 보안 Phase 완료 시 **BLOCK** 으로 승격한다 (정책 변경 이력 표 하단 참조).

| # | 조건 | 검증 방법 | 미충족 시 |
|---|------|----------|---------|
| 5.1 | SQLCipher 키가 OS Keychain에서만 로드 (코드 하드코딩·로깅 0건) | `grep -rn "pbkdf2\|SQLCipher.*key\|sqlcipher_key" src-tauri/` 결과 검토 + 로그 출력 확인 | CONFIRM (Phase 완료 후 BLOCK) |
| 5.2 | `app.lock` heartbeat 로직 단위 테스트 통과 | `cargo test --manifest-path src-tauri/Cargo.toml lock` | CONFIRM (Phase 완료 후 BLOCK) |
| 5.3 | `PRAGMA integrity_check` 스타트업 훅 존재 | `grep -rn "integrity_check" src-tauri/src/` | CONFIRM (Phase 완료 후 BLOCK) |
| 5.4 | 백업 4계층 보관 정책 단위 테스트 통과 | `cargo test --manifest-path src-tauri/Cargo.toml backup` | CONFIRM (Phase 완료 후 BLOCK) |
| 5.5 | 자가 진단(PRD §6.6) 검사 항목 단위 테스트 통과 | `cargo test --manifest-path src-tauri/Cargo.toml diagnosis` | CONFIRM (Phase 완료 후 BLOCK) |

---

## 정책 위반 처리 기준

| 등급 | 미충족 항목 | 처리 |
|------|-----------|------|
| **BLOCK** | 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 3.2, 4.1, 4.3 | 배포 즉시 차단, 미충족 항목 목록 보고 |
| **CONFIRM** | 3.3, 4.2, 5.1, 5.2, 5.3, 5.4, 5.5 | 사용자에게 확인 요청, 승인 시 배포 진행 |

---

## Hotfix 배포 시 적용 정책

Hotfix는 속도가 중요하므로 일부 조건을 경량화합니다:

| 조건 | Sprint 배포 | Hotfix 배포 |
|------|------------|------------|
| CI 통과 | BLOCK | BLOCK |
| cargo test 통과 | BLOCK | BLOCK (변경 모듈 대상) |
| CHANGELOG 업데이트 | BLOCK | CONFIRM |
| sprint-review 완료 | BLOCK | 해당 없음 (hotfix-close가 대신) |
| risk-register 확인 | CONFIRM | CONFIRM |

---

## 정책 변경 이력

| 날짜 | 변경 내용 | 변경 사유 |
|------|----------|---------|
| 2026-05-18 | 분기 학습보고서 도메인 적용 — 청구 마감 3단계(미확정/확정/마감), 백업 복원 리허설 단순화, E2E 도구 `tauri-driver` 통일 반영 | PRD v1.5 정합화 (학습보고서 분기 단위 재설계, §4.8/§4.9.7/§5.4/§6.5) |
| 2026-05-15 | Category 5 (Data Integrity) 신설 — SQLCipher / 락 / 무결성 / 백업 / 자가 진단 게이트 5종 추가, 초기에는 CONFIRM 등급 | PRD v1.4 §5.3~§5.5, §6.6 정합화 |
| 2026-05-11 | pytest → cargo test, Docker → Tauri 빌드 기준으로 변경 | FastAPI → Tauri 스택 전환 |
| 최초 작성 | 기본 배포 정책 정의 | Harness Engineering 원칙 4 도입 |

> 정책 변경 시 이 테이블에 이력을 추가하세요.
