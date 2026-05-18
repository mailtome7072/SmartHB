# harness-ci-gate 스킬

> **용도**: Harness Engineering 원칙 4 — Policy Enforcement  
> **사용 에이전트**: `deploy-prod`, `sprint-review`  
> **정책 상세**: `docs/harness-engineering/deployment-policy.md`

이 스킬은 배포 전 Policy Gate 체크리스트를 실행합니다. 모든 BLOCK 조건을 통과해야 배포를 진행합니다.

---

## 실행 절차

### 선행 체크: PR 존재 확인

```bash
gh pr view --json number --jq '.number' 2>/dev/null || echo "NO_PR"
```
결과가 `NO_PR`이면: "현재 브랜치(`{브랜치명}`)에 해당하는 PR이 없습니다. PR을 먼저 생성하거나 올바른 브랜치에서 실행해주세요." **BLOCK** 처리.

---

### Step 1: BLOCK 조건 검증 (자동 실행)

아래 항목을 순서대로 확인합니다. 하나라도 미충족 시 즉시 중단하고 결과를 보고합니다.

#### 1.1 CI 통과 확인
```bash
gh pr view --json statusCheckRollup --jq '.statusCheckRollup[] | select(.status != "COMPLETED" or .conclusion != "SUCCESS") | .name'
```
- 결과 없음(빈 출력) → ✅ 통과
- 결과 있음 → ❌ 차단: 미통과 잡 이름 보고

#### 1.2 현재 브랜치 확인
```bash
git branch --show-current
```
- `develop` → ✅ 통과
- 그 외 → ❌ 차단: "develop 브랜치에서 실행해야 합니다"

#### 1.3 .env 파일 Git 추적 확인
```bash
git ls-files .env .env.* 2>/dev/null
```
- 출력 없음 → ✅ 통과
- 출력 있음 → ❌ 차단: ".env 파일이 Git에 포함되어 있습니다"

#### 1.4 하드코딩된 시크릿 패턴 스캔
```bash
git diff develop...HEAD -- '*.py' '*.ts' '*.tsx' | grep -E '^\+.*(password|passwd|secret|api_key|apikey|token|private_key)\s*=\s*["'"'"'][^${\s]{6,}["'"'"']' | grep -v '\.example' | head -5
```
- 결과 없음 → ✅ 통과
- 결과 있음 → ❌ 차단: 발견된 라인 보고

#### 1.5 CHANGELOG.md [Unreleased] 섹션 업데이트 확인
```bash
git diff develop...HEAD -- CHANGELOG.md | grep '^\+' | head -3
```
- 변경 있음 → ✅ 통과
- 변경 없음 → ❌ 차단: "CHANGELOG.md [Unreleased] 섹션을 업데이트하세요"

#### 1.6 DEPLOY.md sprint-review 완료 확인
`DEPLOY.md`를 읽어 `✅ sprint-review` 항목이 체크되었는지 확인합니다.
- `✅ sprint-review` 존재 → ✅ 통과
- `⬜ sprint-review` 또는 없음 → ❌ 차단: "sprint-review 에이전트를 먼저 실행하세요"

#### 1.7 Self-verify (cargo test) 완료 확인
- `docs/sprint/sprint{n}/scope.md`에서 수정된 Rust/TypeScript 파일 목록을 확인합니다.
- 해당 파일에 대한 Self-verify 실행 기록 여부를 확인합니다:
  - scope.md `## 완료 기준` 중 테스트 관련 항목이 `✅` 상태인지 확인
  - 또는 에이전트에게 "마지막으로 cargo test를 실행한 시점과 결과"를 직접 확인
- Rust 파일 변경이 없는 경우 해당 없음 (src-tauri/ 미수정 시 스킵)
- 미확인 시: "Self-verify(cargo test) 실행 기록이 없습니다. `cargo test`를 실행하고 결과를 확인하세요." → **CONFIRM** 조건 (차단하지 않음)

---

### Step 2: CONFIRM 조건 확인 (사용자 확인 필요)

BLOCK 조건을 모두 통과한 후, 아래 항목을 확인합니다.

#### 2.1 bandit 보안 스캔 결과
CI security-scan 잡에서 high severity 발견 여부를 확인합니다.
발견 시 사용자에게 내용을 보고하고 계속 진행 여부를 확인합니다.

#### 2.2 risk-register 미해결 이슈
`docs/risk-register/` 디렉토리에서 최근 파일을 읽어 Medium/High 미해결 이슈가 있으면 내용을 보고합니다.
사용자가 "인지 완료" 확인 후 진행합니다.

#### 2.3 ROADMAP.md 상태 확인
해당 스프린트가 `✅ 완료` 상태인지 확인합니다. 미완료 시 확인을 요청합니다.

---

### Step 3: Gate 결과 보고

체크 완료 후 결과를 요약하여 보고합니다:

```
## Policy Gate 결과 — {날짜}

### ✅ BLOCK 조건 (모두 통과)
- CI 통과: ✅
- 브랜치: develop ✅
- .env 미포함: ✅
- 시크릿 없음: ✅
- CHANGELOG 업데이트: ✅
- sprint-review 완료: ✅

### ⚠️ CONFIRM 조건
- bandit 결과: (결과 요약)
- risk-register: (미해결 이슈 목록 또는 "없음")
- ROADMAP 상태: (확인 결과)

→ 배포 진행 가능 / 배포 차단 (이유)
```

---

## 빠른 실행 (Hotfix 경량 게이트)

Hotfix 배포 시 경량화된 체크만 실행합니다:

| 항목 | Sprint | Hotfix |
|------|--------|--------|
| CI 통과 | ✅ 필수 | ✅ 필수 |
| .env 미포함 | ✅ 필수 | ✅ 필수 |
| 시크릿 없음 | ✅ 필수 | ✅ 필수 |
| CHANGELOG | ✅ 필수 | ⚠️ 확인 권장 |
| sprint-review | ✅ 필수 | 해당 없음 |
| risk-register | ⚠️ 확인 | ⚠️ 확인 |
