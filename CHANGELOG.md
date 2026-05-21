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
- Sprint 3: Pretendard 폰트 self-host — `public/fonts/` woff2 배치, `@font-face` + Tailwind config 설정. 본문 18px, 헤더 24px+, 행간 1.5 기본값 확립 (ADR-006)
- Sprint 3: 앱 레이아웃 셸 — 사이드바(메뉴 9종 + 단축키 병기 + 비활성 툴팁), 상단 상태바(점유 디바이스/마지막 백업/동기화 상태), AppShell 조합 컴포넌트
- Sprint 3: 글로벌 검색바 (PRD §4.14) — 원생 이름(우선)/학교명/메뉴명 검색, 한글 자모 부분 일치, 200ms 디바운싱, 1클릭 이동, Ctrl+F 단축키
- Sprint 3: Zustand 스토어 2종 (`src/stores/session-store.ts`, `src/stores/app-store.ts`) — 세션 상태/락 점유/사이드바 상태/선택 교습기간월
- Sprint 3: TanStack Query Provider — IPC 응답 캐싱/무효화 패턴 확립 (`src/providers/query-provider.tsx`)
- Sprint 3: `tauri-plugin-dialog` 통합 — 폴더 선택 네이티브 다이얼로그 IPC + `capabilities/default.json` `dialog:default` 권한
- Sprint 3: 초기 설정 마법사 백엔드 (`src-tauri/src/commands/setup.rs`) — `save_cloud_folder`, `complete_setup`, `get_setup_status` IPC 3종 + `app_config_dir/config.json` 설정 분리 저장 (chicken-and-egg step-back 반영)
- Sprint 3: 초기 설정 마법사 프론트엔드 (`src/app/setup/page.tsx`) — 4단계(환영/클라우드 폴더 선택/비밀번호 설정/완료) + 단계별 독립 저장 + 뒤로가기 지원
- Sprint 3: 원생 목록 화면 (`src/app/students/page.tsx`) — TanStack Query 캐싱, 필터 7종(이름/학교급/학년/학교명/요일/성별/재원상태) + 정렬 3종 + 페이지네이션
- Sprint 3: 원생 등록/수정 폼 — `create_student`/`update_student`/`withdraw_student` IPC 연동, 3분 자동 임시저장(localStorage), 미저장 경고 다이얼로그, 퇴교 처리 확인 다이얼로그
- Sprint 3: 코드 테이블 관리 화면 (PRD §4.12) — 학교/표준교습비/결제수단/카드사 탭 CRUD, is_active 소프트 삭제, sort_order 변경
- Sprint 3: 수업 스케줄 편집 UI (PRD §4.2) — 요일별 시작 시간/수업 시간 입력, 운영 시간 내 제한, 주 총 수업시간 실시간 표시, 표준 교습비 자동 매칭 표시
- Sprint 3: 키보드 단축키 체계 (`src/hooks/use-keyboard-shortcuts.ts`) — F1/Ctrl+F/Ctrl+N/Ctrl+S/Ctrl+Z/ESC/Ctrl+P 7종 바인딩
- Sprint 3: `count_students(filter)` IPC 신규 — 페이지네이션 총 건수 반환
- Sprint 3: 단위 테스트 109건 (Sprint 2 97건 → +12건)

### Changed
- Sprint 3: `src/app/page.tsx` 라우팅 분기 업데이트 — `not-initialized` 상태 시 `/setup` 마법사로 이동
- Sprint 3: `StudentFilter` 구조체에 `limit: Option<u32>`, `offset: Option<u32>` 추가 — 기본 limit=50, 상한 1000
- Sprint 3: `list_students`/`list_codes` SQL에 `LIMIT ? OFFSET ?` 적용

### Fixed
- Sprint 3: R13 PII 마스킹 — `students.rs` `try_record` 3곳 `details=None` 적용하여 감사 로그에 원생 이름 미포함
- post-sprint3: `config.json` 손상 자동 복구 (`setup.rs`) — PC 강제 종료로 인한 NTFS power-loss 시 발생하는 NULL-바이트 파일/파싱 실패를 감지하여 `config.json.corrupted-{ts}` 로 백업 후 기본값 fallback. 사용자는 마법사를 다시 진행하면 자동 복구됨. 단위 테스트 6건 추가 (총 115건)

---

## [0.2.0] - 2026-05-20

### Added
- Sprint 2: 루트 라우팅 + 인증 게이트 — `src/app/page.tsx` 클라이언트 가드, `lock/page.tsx` onUnlocked → `app_startup_sequence` → 메인 redirect, `src/lib/auth-state.ts` 모듈 스코프 인증 상태 (Sprint 3 Zustand 도입 전)
- Sprint 2: DB 마이그레이션 V101 — students + student_schedules 테이블 (PI-05 자동 채번: `MAX+1` + `BEGIN IMMEDIATE` + override 허용)
- Sprint 2: DB 마이그레이션 V102 — study_periods + schedule_codes 테이블 (시스템 예약 코드 5종 시드 데이터 포함)
- Sprint 2: DB 마이그레이션 V103 — schedule_events 테이블
- Sprint 2: DB 마이그레이션 V104 — standard_fees 재설계 (weekly_minutes 기준 교습비 자동 매칭)
- Sprint 2: DB 마이그레이션 V105 — schools 보강 (school_type / region 컬럼)
- Sprint 2: 원생 CRUD IPC 커맨드 4종 (`create_student`, `update_student`, `list_students`, `get_student`) — 이름/학교급/학년/학교명/요일/성별 다중 필터, 이름순/입교일순/학년순 정렬, 재원 상태 필터
- Sprint 2: 수업 스케줄 IPC 커맨드 3종 (`set_schedule`, `get_schedules`, `get_weekly_hours`) — (원생, 요일) UNIQUE 검증, 변경 이력 자동 생성, 주 총 수업시간 자동 산정
- Sprint 2: 표준 교습비 IPC 커맨드 3종 (`list_fees`, `create_fee`, `update_fee`) + 주 수업시간 → 교습비 자동 매칭 함수
- Sprint 2: 코드 테이블 CRUD IPC 커맨드 — 학교/결제수단/카드사 공용 CRUD (is_active 소프트 삭제, sort_order 변경)
- Sprint 2: 도메인 타입 4종 — `src/types/{student,schedule,fee,code}.ts`
- Sprint 2: 프론트엔드 IPC 래퍼 18개 추가 (`src/lib/tauri/index.ts`) — dev mode fallback 포함, 총 22 신규 IPC
- Sprint 2: `AppError::UserFacing(String)` variant 신규 — 도메인 검증 메시지 사용자 친화적 노출
- Sprint 2: `AuditEventType` 확장 — StudentCreated / StudentUpdated / StudentWithdrawn 추가
- Sprint 2: 단위 테스트 97건 (Sprint 1 64건 → +33건)

### Changed
- Sprint 2: R6 salt 이전 이연 확정 — `{data_root}/salt.bin` 평문 파일 보관, Sprint 3 마법사 통합 시점에 Keychain 이전 (R12 신설 추적)
- Sprint 2: T8 `query!()`/`query_as!()` 매크로 전환 이연 — 동적 `query() + bind()` 패턴 유지, 별도 backlog 추가

> **이연 사유**: salt 이전은 마법사 DB 경로 설정과 coupled, `query!()` 매크로는 V101~V105 스키마 안정화 후 일괄 전환이 안전

### Fixed
- R15: `startup::exit_hook`에서 `release_lock_atomic()` 직접 호출로 교체 (6c85f5c)

---

## [0.1.0] - 2026-05-19

### Added
- Sprint 1: SQLCipher AES-256 암호화 DB 통합 (`libsqlite3-sys bundled-sqlcipher-vendored-openssl`, `cipher` feature flag로 개발/프로덕션 분리) — ADR-001
- Sprint 1: OS Keychain/Credential Manager 통합 (`keyring` crate) + PBKDF2 600K iter 키 유도 + `zeroize` 메모리 폐기 — ADR-004
- Sprint 1: PI-07 복구 코드 — Argon2id 해시, 12자리 31자 알파벳 포맷
- Sprint 1: 인증 IPC 커맨드 (`set_password`, `unlock_db`, `check_auth_status`) + 잠금 화면 UI (Pretendard 18pt, 44×44px 버튼)
- Sprint 1: `app.lock` 동시성 제어 — `fs2` advisory locking + 60초 heartbeat + 5분 강제 해제 — ADR-002
- Sprint 1: 4계층 자동 백업 — exit(10) / hourly(24) / daily(30) / weekly(4), SQLite Online Backup API, 암호화 상태 그대로 보관 — ADR-003
- Sprint 1: 무결성 검증 — 앱 시작 시 `PRAGMA quick_check / integrity_check`, 손상 감지 시 자동 복원 + `restore_rollback` 안전망 + 손상본 격리
- Sprint 1: 동기화 대기 로직 — DB/락 파일 최신 동기화 확인
- Sprint 1: 감사 로그 (`audit_logs` 테이블) + 주요 커맨드 7곳 통합
- Sprint 1: DB 마이그레이션 V001 (코드성 테이블: schools, payment_methods, card_companies, standard_fees) + V008 (app_settings, audit_logs)
- Sprint 1: 앱 시작 시퀀스 — `tokio::join!` 락+무결성 병렬 실행, PRD §5.6 < 3초 목표 구현
- Sprint 1: `commands/paths.rs` / `commands/runtime.rs` / `app_err!` 매크로 공통 헬퍼 모듈 분리
- Sprint 1: `thiserror` 기반 `AppError` 7종 변형 (`Auth`, `Db`, `Lock`, `Backup`, `Integrity`, `Io`, `Config`) — `src-tauri/src/error.rs`
- Sprint 1: 단위 테스트 74건 (`cargo test` 기준)
- Sprint 1: CI 매트릭스 — `ci.yml` + `deploy.yml` cipher feature on/off 양 OS 빌드 (Windows Strawberry Perl 포함)
- Sprint 1: ADR-001/002/003/004 문서 4건 (`docs/arch/`)

---

## [0.0.1] — 프로젝트 초기 템플릿 (보일러플레이트 + 계획 산출물)

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
