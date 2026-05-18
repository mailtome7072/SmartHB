# 하네스 엔지니어링 원칙

> **로드 조건**: 전체 대화 (always-loaded)
> AI 코딩 에이전트는 이 원칙을 모든 작업에서 준수한다.

---

## 원칙 1: Planning First (계획 우선)

코드를 수정하기 전에 반드시 **scope 선언**을 완료한다.

- `/sprint-dev {n}` 진입 시 `docs/sprint/sprint{n}/scope.md`를 작성한 후 코드를 건드린다.
- scope.md 없이 Edit/Write 도구를 코드 파일에 사용하지 않는다.
- 재진입(새 세션) 시 기존 scope.md를 읽고 이전 상태를 확인한 후 이어서 진행한다.

### Step-back 프로토콜

예상치 못한 구조적 충돌이나 설계 오류 발견 시:
1. 코드 수정을 **즉시 중단**한다.
2. 발견한 충돌/이슈를 scope.md `## 발견된 이슈` 섹션에 기록한다.
3. 사용자에게 보고하고 재계획 요청을 구체적으로 제시한다.
   > 예: "인증 미들웨어가 scope에 없는 파일 3개에 의존하고 있습니다. 계획을 조정해야 합니다."

### Re-planning 트리거

아래 중 하나라도 해당하면 즉시 사용자에게 보고하고 scope.md를 업데이트한다:
- 실제 수정 파일이 scope.md에 선언한 파일과 **30% 이상** 차이
- 새 의존성(pip/npm 패키지) 추가 필요 발생
- DB 스키마 변경 필요 발생 (scope에 없었던 경우)

---

## 원칙 2: Strict Guardrails (엄격한 범위 제한)

scope.md에 명시되지 않은 변경은 사전 승인 없이 수행하지 않는다.

**금지 행위 (사용자 확인 없이)**:
- scope 외 파일 수정
- `requirements.txt` / `package.json`에 새 패키지 추가
- 디렉토리 구조 변경 (새 디렉토리 생성 포함)
- `.claude/settings.json` 수정

**Forbidden Areas (명시적 사용자 허가 없이 수정 완전 금지)**:
- `.github/workflows/` — CI/CD 파이프라인 (posttooluse hook이 자동 차단)
- `SETUP.sh` — 프로젝트 초기화 스크립트 (posttooluse hook이 자동 차단)
- `docker/`, `docker-compose*.yml` — 컨테이너 인프라 설정 (경고 출력)
- `docs/harness-engineering/` — Harness 정책 문서 (정책 약화 방지, 경고 출력)

**허용 예외**: 사용자가 명시적으로 요청하거나 sprint 계획 문서(`sprint{n}.md`)에 명시된 경우.

### Hook Compliance (커밋 품질 강제)

모든 커밋은 `pre-commit hook`을 통과해야 한다:
- Python syntax 오류 → 커밋 차단
- 프론트엔드 lint 오류 (`pnpm lint --max-warnings 0`) → 커밋 차단

`git commit --no-verify` 사용 금지 — Harness 원칙 위반.

---

## 원칙 3: Verification Loops (검증 루프)

### 3-Retry 원칙

테스트/린트 실패 시:
1. **1차 실패**: 오류 메시지 분석 → 원인 파악 → 수정 → 재검증
2. **2차 실패**: 다른 접근 방법으로 재시도 (1차와 동일한 수정 반복 금지)
3. **3차 실패**: 즉시 중단 후 사용자 보고 (시도한 방법 2가지 명시)

**절대 금지**:
- `--no-verify`, `--skip-tests` 플래그 사용
- 실패 원인 파악 없이 동일 수정 반복
- 테스트 파일 삭제/비활성화로 CI 통과 시도

### Loop Detection (루프 감지)

**루프 상태 기준** — 아래 중 하나 충족 시:
- 동일한 오류 메시지로 테스트가 **3회 이상 연속 실패**
- 동일한 파일을 **3회 이상 반복 수정** (동일 Task 내)

**루프 감지 시**: `.claude/skills/loop-detection.md` 스킬을 **즉시** 실행한다.
추가 수정 없이 현황을 분석하고 사용자에게 보고한다. 사용자 승인 없이 재시도 금지.

파일 수정 횟수는 `scope.md`의 수정 파일 목록에 `[1회]`, `[2회]`, `[3회 ⚠️]` 형식으로 기록한다.

### Self-verify (자기 검증)

코드 작성 완료 후 `git commit` 전에 **반드시** 아래를 수행한다:

1. **백엔드 테스트**: `cargo test` (Rust 파일 변경 시, src-tauri/ 기준)
2. **정적 분석**: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
3. **Linter**: `pnpm lint` (프론트엔드 파일 변경 시)
4. **Syntax 확인**: posttooluse hook이 자동으로 검증 (Rust syntax, cargo check)

Self-verify 없이 `git commit` 금지 — pre-commit hook이 강제한다.
Self-verify 실패 → Verification Loop(3-retry) 적용.

### 검증 순서

코드 변경 후 아래 순서로 검증을 수행한다:
```
Edit/Write → posttooluse-code-validator (자동)
    → Self-verify (단위 테스트 → 통합 테스트 → lint)
    → simplify 실행
    → git commit → pre-commit hook (lint 재확인, 자동)
```

---

## 원칙 4: Policy Enforcement (정책 강제)

배포 전에 반드시 Policy Gate를 통과해야 한다.

deploy-prod 에이전트가 `.claude/skills/harness-ci-gate.md` 스킬을 실행하여 배포 정책을 검증한다. 정책 상세는 `docs/harness-engineering/deployment-policy.md` 참조.

> sprint-review 에이전트 완료(`DEPLOY.md ✅ sprint-review`)는 harness-ci-gate의 BLOCK 조건 중 하나다. sprint-review는 gate를 실행하는 것이 아니라 gate 통과 조건을 충족시키는 역할을 한다.

**자동 차단 조건**:
- CI(GitHub Actions) 미통과
- 코드에 하드코딩된 시크릿 발견
- CHANGELOG.md 미업데이트

---

## 원칙 5: Continuous Verification (지속적 검증)

배포는 merge 시 끝나지 않는다. 배포 후 검증이 완료되어야 스프린트가 진정으로 완료된다.

deploy-prod 에이전트는 배포 후 `docs/harness-engineering/continuous-verification.md`의 CV 체크리스트를 실행한다.

**CV 실패 시**:
- 헬스체크 실패 → 즉시 롤백 안내 (`docs/dev-process.md` 섹션 6.4)
- 에러 로그 Critical 발견 → 즉시 사용자 보고

---

## 요약 참조 테이블

| 원칙 | 의무 행동 | 위반 시 |
|------|----------|---------|
| Planning First | scope.md 작성 후 코드 수정 | 코드 수정 중단, scope.md 먼저 작성 |
| Strict Guardrails | scope 외 변경 금지 | 사용자 확인 요청 후 진행 |
| Verification Loops | 3-retry, 동일 수정 반복 금지 | 3회 실패 시 사용자 보고 |
| Policy Enforcement | 배포 전 harness-ci-gate 스킬 실행 | 미충족 항목 보고, 배포 중단 |
| Continuous Verification | 배포 후 CV 체크리스트 실행 | CV 실패 시 롤백 안내 |
