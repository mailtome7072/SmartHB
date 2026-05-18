# CHANGELOG

이 파일은 프로젝트의 버전별 변경 이력을 기록합니다.
형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.0.0/)를 기반으로 하며,
[Semantic Versioning](https://semver.org/lang/ko/)을 준수합니다.

---

## 작성 규칙

### 카테고리

| 카테고리 | 설명 |
|----------|------|
| `Added` | 새로운 기능 추가 |
| `Changed` | 기존 기능 변경 |
| `Deprecated` | 곧 제거될 기능 예고 (하위 호환성 안내) |
| `Removed` | 기능 제거 |
| `Fixed` | 버그 수정 |
| `Security` | 보안 취약점 수정 |

### Semantic Versioning 올림 기준

| 버전 | 트리거 |
|------|--------|
| `MAJOR` | 하위 호환 불가 변경 — API 브레이킹 체인지, DB 구조 대규모 변경 |
| `MINOR` | 하위 호환 신규 기능 추가 — 새 엔드포인트, 새 UI 기능 |
| `PATCH` | 버그 수정, 핫픽스, 문서 수정 |

### [Unreleased] 운영 방법

- **채우는 시점**: PR merge 시마다 해당 카테고리에 항목 추가
- **버전 전환 시점**: `deploy-prod` agent가 main 배포 시 `[Unreleased]` → `[x.y.z] - YYYY-MM-DD`로 전환
- **새 버전은 항상 최상단에 추가**

---

## [Unreleased]

### Added
- 프로젝트 초기 템플릿 설정
- Claude Code 에이전트 정의 (sprint-planner, sprint-close, sprint-review, hotfix-close, deploy-prod, phase-planner, prd-to-roadmap)
- CI/CD 파이프라인 (GitHub Actions — ci.yml + deploy.yml)
- 개발 프로세스 문서 (`docs/dev-process.md`)
- CI/CD 정책 문서 (`docs/ci-policy.md`)
- 전략 지침 문서 (`strategy/`)
- 하네스 엔지니어링 정책 5종 (`docs/harness-engineering/`)
- PRD.md v1.5 (MVP) — 분기 학습보고서 도메인 재설계 (작성 주기 월 1회 → 분기 1회, 키 `(분기, 원생)`, 단일 컬럼 `종합의견`, 단원평가 점수에 종속)
- PRD.md v1.4 (MVP) — 5건 Post-MVP 승격 통합 (초기 설정 마법사 §4.0, 글로벌 검색 §4.14, 청구 마감 워크플로우 §4.9.7, 데이터 자가 진단 §6.6, 키보드 단축키 §5.7)
- PRD 정합화 산출물: `docs/prd-issues.md` (논리 오류 11건), `docs/data-model.md` (도메인 → SQLite 스키마 1차 매핑)
- Tauri shell plugin (`tauri-plugin-shell`) — 외부 프로세스 실행 및 OS 기본 앱으로 파일/URL 열기
- `.gitattributes` — 셸 스크립트 LF 정규화 (macOS/Windows 양 OS 보장)
- `docs/setup-guide.md` Tauri 아이콘 생성 절차 (5-A 섹션)

### Changed
- 데이터 저장 모델 — Supabase에서 **로컬 SQLite + SQLCipher AES-256 + 클라우드 동기화 폴더(MYBOX 우선)** 로 전환 (PRD v1.1)
- 기술 스택 — FastAPI/Docker에서 **Tauri 2 + Next.js 15 + React 19** 로 전환 (커밋 f2fbb7c)
- 동시성 모델 — 양 PC 시점 분리 사용 + `app.lock` heartbeat 60s, 5분 미갱신 강제해제 (PRD §5.3)
- 백업 정책 — 4계층 자동 백업(exit/hourly/daily/weekly) + SQLite Online Backup API (PRD §5.4)
- 백업 복원 리허설 — 정기 수행 모드에서 **필요시 수행 모드**로 단순화 (PRD v1.4)
- 청구 데이터 상태 — 2단계(미확정/확정)에서 **3단계(미확정/확정/마감)** 로 확장 (PRD v1.4 §4.9.7)
- 학습보고서 E2E 도구 — Playwright에서 `Tauri WebDriver(tauri-driver)` 로 통일 (PRD §6.5)
- 학습보고서 출력 — `§4.8.4`의 "파일 저장 없음" 제거하여 인쇄 + PDF 저장 양쪽 허용 (`§4.13.2` 와 정합)
- 클라우드 동기화 폴더명 — `smarthm/` → **`smarthb/`** 로 통일 (프로젝트명과 일치)
- 배포 모델 — GitHub Releases 인스톨러(Windows `.msi`/`.exe`, macOS `.dmg`) (PRD)
- AI 협업 가이드 정합 — `CLAUDE.md`, `.claude/rules/backend.md`, `.claude/rules/frontend.md`, `ARCHITECTURE.md`에 SQLCipher / 락 / 무결성 / Pretendard / Zustand / TanStack Query / FullCalendar / 글로벌 검색바 / 분기 학습보고서 / 청구 마감 정책 반영
- 단일 사용자 모델 — CV 문서의 "팀 채널 모니터링" → "원장 직접 체감"으로 정합

### Fixed
- 셸 스크립트 실행권한 비트 부여 (`SETUP.sh`, `scripts/hooks/pre-commit`, `.claude/hooks/*.sh`) — macOS clone 시 `Permission denied` 방지
- `scripts/hooks/pre-commit` 옛 경로(`app/frontend/`) 제거 및 `scripts/pre-commit-lint.sh` 위임 wrapper로 단순화
- `.claude/hooks/pretooluse-bash-guard.sh` — python3 미설치 환경에서 jq 폴백 추가, 둘 다 없으면 안전을 위해 차단
- `SETUP.sh` macOS Xcode CLI 미설치 시 `exit 1` 로 강제 차단

---

## 참고

- 로드맵 연계: `ROADMAP.md` (Phase/Sprint 상태와 버전 연결)
- Notion 업데이트 트리거: `docs/dev-process.md` 섹션 8.5
