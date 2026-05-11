# 지속적 검증 (Continuous Verification)

> **용도**: Harness Engineering 원칙 5 — Continuous Verification  
> **사용 에이전트**: `deploy-prod`  
> **트리거**: main 브랜치 merge 완료 후 GitHub Actions 배포 시작 시

배포는 GitHub Actions 완료로 끝나지 않습니다. 실서버에서 실제로 정상 동작하는지 확인하는 CV 단계가 완료되어야 스프린트/핫픽스가 진정으로 완료됩니다.

---

## CV 3단계

### 1단계: 기술 검증 (배포 후 즉시 — ~2분)

> deploy-prod 에이전트가 GitHub Actions 배포 완료 후 즉시 실행

| # | 검증 항목 | 명령 | 성공 기준 |
|---|----------|------|---------|
| 1.1 | 헬스체크 엔드포인트 | `curl -s -o /dev/null -w "%{http_code}" http://{SERVER_IP}/api/v1/health` | HTTP 200 |
| 1.2 | 컨테이너 상태 확인 | `ssh {USER}@{SERVER_IP} "docker compose -f /opt/app/docker-compose.prod.yml ps"` | 모든 서비스 `running` |
| 1.3 | 백엔드 에러 로그 확인 | `ssh {USER}@{SERVER_IP} "docker compose -f /opt/app/docker-compose.prod.yml logs backend --tail 30"` | `ERROR`/`TRACEBACK`/`CRITICAL` 없음 |
| 1.4 | DB 마이그레이션 상태 | `ssh {USER}@{SERVER_IP} "docker compose -f /opt/app/docker-compose.prod.yml exec backend alembic current"` | `(head)` 상태 |

**1단계 실패 기준 (즉시 롤백 안내)**:
- 1.1 HTTP 200 아님
- 1.2 컨테이너 1개라도 `Exited` 또는 `Restarting`
- 1.3 `CRITICAL` 레벨 에러 발견

---

### 2단계: 기능 검증 (배포 후 5분 — ~5분)

> deploy-prod 에이전트가 1단계 통과 후 실행 (Playwright 설치 여부에 따라 자동/수동 전환)

| # | 검증 항목 | 방법 | 성공 기준 |
|---|----------|------|---------|
| 2.1 | 핵심 API 엔드포인트 응답 | curl + 기대 응답 확인 | HTTP 200/201 |
| 2.2 | 로그인 플로우 (UI) | Playwright 핵심 시나리오 | 에러 없이 완료 |
| 2.3 | 주요 기능 동작 확인 | sprint{n}.md의 완료 기준 항목 기반 | 모든 기준 통과 |

**Playwright 미설치 시 자동→수동 전환**:
```
⬜ [수동] 브라우저에서 {SERVER_URL}에 접속하여 로그인 및 주요 기능 동작 확인
```

---

### 3단계: 안정성 판단 (배포 후 30분)

> 사용자가 직접 확인하는 수동 단계 (에이전트가 자동화하기 어려운 영역)

| # | 확인 항목 | 판단 기준 |
|---|----------|---------|
| 3.1 | 에러율 증가 없음 | 배포 전 대비 에러 로그 빈도 동일 수준 |
| 3.2 | 응답 속도 정상 | 주요 API 응답 시간 배포 전 대비 2배 이하 |
| 3.3 | 사용자 불편 신고 없음 | 30분 이내 사용자/팀 채널 모니터링 |

**3단계 완료 후**: DEPLOY.md `✅ CV 완료` 항목 체크

---

## 자동 롤백 트리거 (deploy-prod 에이전트 기준)

다음 조건 발생 시 에이전트가 즉시 사용자에게 롤백 안내를 제시합니다:

| 트리거 | 대응 |
|--------|------|
| 헬스체크 HTTP 200 아님 | "⚠️ 헬스체크 실패 — 즉시 롤백을 권장합니다" |
| 컨테이너 Exited 상태 | "⚠️ 컨테이너 비정상 — 즉시 롤백을 권장합니다" |
| CRITICAL 에러 로그 발견 | "⚠️ 심각한 에러 감지 — 롤백 여부를 결정해주세요" |

**롤백 방법**: `docs/dev-process.md` 섹션 6.4 참조

---

## DEPLOY.md CV 기록 형식

deploy-prod 에이전트가 CV 결과를 DEPLOY.md에 기록하는 형식:

```markdown
### CV (Continuous Verification) — {날짜}

**1단계: 기술 검증 (배포 직후)**
- ✅ 헬스체크: HTTP 200
- ✅ 컨테이너 상태: 전체 running
- ✅ 에러 로그: 없음
- ✅ DB 마이그레이션: head 상태

**2단계: 기능 검증 (5분 후)**
- ✅ 핵심 API: 정상
- ⬜ Playwright 검증: 수동 필요 (Playwright 미설치)

**3단계: 안정성 판단 (30분 후)**
- ⬜ 에러율 모니터링 (사용자 직접 확인)
- ⬜ 응답 속도 확인 (사용자 직접 확인)
```

---

## CV 미수행 시

네트워크/SSH 접속 불가 등으로 CV를 수행할 수 없는 경우:
```markdown
- ⬜ CV 미수행: SSH 접속 불가 — 수동 검증 필요
  → docs/harness-engineering/continuous-verification.md 참조
```
DEPLOY.md에 기록하고, 팀원이 직접 서버에서 확인하도록 안내합니다.
