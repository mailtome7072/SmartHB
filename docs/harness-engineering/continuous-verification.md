# 지속적 검증 (Continuous Verification)

> **용도**: Harness Engineering 원칙 5 — Continuous Verification  
> **사용 에이전트**: `deploy-prod`  
> **트리거**: `v*` 태그 push → GitHub Actions `deploy.yml` 완료 후

배포는 GitHub Actions 완료로 끝나지 않습니다. GitHub Release 아티팩트가 정상적으로 생성되었는지, 앱이 정상 동작하는지 확인하는 CV 단계가 완료되어야 스프린트/핫픽스가 진정으로 완료됩니다.

---

## CV 3단계

### 1단계: 아티팩트 검증 (Actions 완료 직후 — ~2분)

> deploy-prod 에이전트가 GitHub Actions 완료 후 즉시 실행

| # | 검증 항목 | 명령 | 성공 기준 |
|---|----------|------|---------|
| 1.1 | GitHub Release 생성 확인 | `gh release view v{version}` | Release 정상 출력 |
| 1.2 | Windows 인스톨러 아티팩트 확인 | `gh release view v{version} --json assets --jq '.assets[].name'` | `.msi` 또는 `.exe` 파일 존재 |
| 1.3 | macOS 인스톨러 아티팩트 확인 | 위와 동일 | `.dmg` 파일 존재 |
| 1.4 | Actions 빌드 로그 에러 확인 | `gh run view {run_id}` | 모든 job 성공 (✓ green) |

**1단계 실패 기준 (조치 필요)**:
- 1.1 Release 미생성 → Actions 로그 확인 후 재실행 또는 수동 Release 생성
- 1.2/1.3 아티팩트 누락 → Actions 빌드 실패 원인 확인 (`docs/dev-process.md` 섹션 9.2 참조)

---

### 2단계: 설치 및 기능 검증 (배포 후 — 선택적 수동)

> **원장(사용자)이 직접 인스톨러를 다운로드하여 신규 버전 적용 후 정상 동작을 확인하는 수동 단계.**
> SmartHB는 단일 사용자(원장 1인) 데스크톱 앱이므로 "팀원 검증"이 아닌 사용자 본인의 체감이 곧 검증이다.

| # | 검증 항목 | 방법 | 성공 기준 |
|---|----------|------|---------|
| 2.1 | 인스톨러 다운로드 | GitHub Release 페이지에서 다운로드 | 파일 정상 다운로드 |
| 2.2 | 설치 성공 | 인스톨러 실행 | 에러 없이 설치 완료 |
| 2.3 | 앱 실행 | 설치된 앱 실행 | 앱 정상 시작 (락 확인 + PRAGMA integrity_check 통과 포함) |
| 2.4 | 주요 기능 동작 | sprint{n}.md의 완료 기준 항목 기반 확인 | 모든 기준 통과 |
| 2.5 | 데이터 무결성 | 기존 클라우드 동기화 폴더의 DB가 정상 로드 + 백업 디렉토리 유지 | 데이터 유실 0건 |

> 초기 릴리즈에서는 2단계를 생략하고 DEPLOY.md에 "⬜ 설치 테스트: 수동 확인 필요"로 기록 가능.

---

### 3단계: 안정성 판단 (배포 후 30분)

> **원장(사용자)이 직접 체감하는 수동 단계.** 30분 이내 신규 버전을 일상 작업 흐름 1개 이상에 적용하여 즉시 불편/오류가 없는지 확인한다. 팀 채널 모니터링 같은 다인원 절차는 적용하지 않는다.

| # | 확인 항목 | 판단 기준 |
|---|----------|---------|
| 3.1 | 사용자 직접 체감 — 즉시 사용 가능 | 30분 이내 대시보드 진입 + 1개 주요 기능 동작 (예: 출결 토글, 청구 화면 진입, 글로벌 검색) |
| 3.2 | 크래시 리포트 없음 | Tauri 앱 비정상 종료 없음 |
| 3.3 | 자가 진단 이상 항목 없음 (참고) | 다음 자가 진단(매월 1일 자동 또는 수동 실행, PRD §6.6) 결과를 안정성 보조 신호로 활용 |

**3단계 완료 후**: DEPLOY.md `✅ CV 완료` 항목 체크

---

## 자동 조치 트리거 (deploy-prod 에이전트 기준)

다음 조건 발생 시 에이전트가 즉시 사용자에게 안내를 제시합니다:

| 트리거 | 대응 |
|--------|------|
| GitHub Release 미생성 | "⚠️ Release 미생성 — Actions 로그를 확인해주세요. `gh run list --branch main`" |
| Actions 빌드 실패 | "⚠️ 빌드 실패 — `docs/dev-process.md` 섹션 9.2 참조" |
| 아티팩트 누락 | "⚠️ 인스톨러 파일이 누락됨 — Actions 빌드 로그를 확인해주세요" |

**롤백 방법**: `docs/dev-process.md` 섹션 6.4 참조

---

## DEPLOY.md CV 기록 형식

deploy-prod 에이전트가 CV 결과를 DEPLOY.md에 기록하는 형식:

```markdown
### CV (Continuous Verification) — {날짜}

**1단계: 아티팩트 검증 (Actions 완료 직후)**
- ✅ GitHub Release v{version} 생성 확인
- ✅ Windows 인스톨러: SmartHB_{version}_x64-setup.exe
- ✅ macOS 인스톨러: SmartHB_{version}_aarch64.dmg
- ✅ Actions 모든 job 성공

**2단계: 설치 검증 (수동)**
- ⬜ 인스톨러 다운로드 및 설치 테스트 (원장 직접 확인 — 단일 사용자 모델)

**3단계: 안정성 (30분 후)**
- ⬜ 원장 직접 체감: 신규 버전에서 주요 기능 1개 이상 정상 동작 확인
- ⬜ 자가 진단(PRD §6.6) 이상 항목 없음 (참고용)
```

---

## CV 미수행 시

Actions 실패 등으로 CV를 수행할 수 없는 경우:
```markdown
- ⬜ CV 미수행: Actions 빌드 실패 — 원인 확인 후 재배포 필요
  → docs/harness-engineering/continuous-verification.md 참조
```
DEPLOY.md에 기록하고, 빌드 실패 원인을 파악 후 태그를 재생성하여 재배포합니다.
