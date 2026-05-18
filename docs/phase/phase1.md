# Phase 1: 인프라 + 기반 도메인

## 개요
- **목표**: 후속 전 Phase가 의존하는 데이터 인프라(SQLCipher, app.lock, 4계층 백업, 무결성 검증)를 확립하고, 원생/스케줄/코드 테이블 기반 도메인을 완성하여 첫 사용 가능한 화면(원생 관리 + 초기 설정 마법사)을 제공한다.
- **예상 기간**: 3스프린트 / 6주 (Sprint 1~3)
- **관련 ROADMAP 항목**: `ROADMAP.md` > Phase 1 (인프라 + 기반 도메인)
- **PRD 기준**: v1.5.1

---

## 설계 결정 사항 (ADR 6건)

| # | 항목 | 결정 시점 | 후보 | 권장안 | 결정 이유 |
|---|------|----------|------|--------|----------|
| ADR-001 | SQLCipher 도입 방식 | Sprint 1 진입 시 | (A) `libsqlite3-sys` bundled-sqlcipher feature (B) 시스템 sqlcipher 라이브러리 | **(A) bundled-sqlcipher** | CI에서 시스템 라이브러리 설치 없이 양 OS 빌드 가능. 단일 바이너리 배포 단순화. Cargo feature flag로 개발/프로덕션 전환 용이 |
| ADR-002 | app.lock 라이브러리 선정 | Sprint 1 진입 시 | (A) `fs2` advisory locking (B) `file-lock` crate (C) 자체 구현 | **(A) fs2 + 자체 heartbeat** | fs2는 양 OS 지원 + advisory lock API 제공. heartbeat(60초)와 5분 만료는 자체 구현. file-lock은 기능 부족, 자체 구현은 비용 과다 |
| ADR-003 | 백업 구현 방식 | Sprint 1 중간 | (A) `rusqlite::backup` API 직접 사용 (B) sqlx raw connection에서 backup | **(A) rusqlite::backup** | SQLite Online Backup API를 직접 호출 가능. sqlx는 backup API를 노출하지 않음. rusqlite를 보조 의존성으로 추가하여 백업 전용으로 사용 |
| ADR-004 | Keychain crate 선정 | Sprint 1 진입 시 | (A) `keyring` crate (B) `security-framework` + `windows-credential` 개별 사용 | **(A) keyring** | 양 OS 통합 API 제공 (macOS Keychain + Windows Credential Manager). 추상화 수준 적절. v3.x 안정판 |
| ADR-005 | PI-05 일련번호 자동 채번 | Sprint 2 진입 전 사용자 결정 | (A) 수동 입력만 (B) 임시 규칙 `YY+0001` 자동 채번 | **사용자 결정 대기** | MVP 수동 입력만으로 충분할 수 있으나, 사용자 편의를 고려하면 자동 채번 옵션 제공이 바람직. Sprint 2 진입 전 결정 필수 |
| ADR-006 | Pretendard 폰트 임베드 방식 | Sprint 3 진입 시 | (A) `@fontsource/pretendard` npm 패키지 (B) `public/fonts/` self-host | **(B) self-host** | 오프라인 환경 100% 보장 필수 (인터넷 없이 동작). fontsource도 빌드 시 포함되지만, self-host가 static export + Tauri 번들링에서 경로 제어가 명확 |

ADR 문서 저장 위치: `docs/arch/adr-001-sqlcipher-integration.md` ~ `adr-006-pretendard-embed.md`
각 ADR은 해당 Sprint의 sprint-planner가 상세 작성한다.

---

## Phase 분할 (Sprint 1~3)

### Sprint 1: 데이터 인프라 (2주)

**목표**: SQLCipher 암호화 DB + 인증 + app.lock 동시성 제어 + 4계층 백업 + 무결성 검증의 전체 인프라를 구축하여, 이후 모든 Sprint가 안전한 데이터 기반 위에서 작업할 수 있게 한다.

**구현 범위**:

| 카테고리 | 작업 | IPC 커맨드 | 마이그레이션 |
|----------|------|-----------|-------------|
| SQLCipher 통합 | ADR-001 PoC + Cargo.toml 설정 + 양 OS 빌드 검증 | - | - |
| 키 관리 | `keyring` crate 통합 + PBKDF2 키 유도 | - | - |
| 인증 | 비밀번호 입력 → DB 복호화 → 진입 + 잠금 화면 UI | `unlock_db`, `check_auth_status`, `set_password` | V008 (app_settings) |
| PI-07 복구 코드 | 설정 메뉴 발급/재발급 + 해시 저장 + 검증 흐름 | `generate_recovery_code`, `verify_recovery_code`, `reset_password_with_code` | V008 내 recovery_code_hash 컬럼 |
| app.lock | 락 생성/확인/heartbeat(60초)/강제 점유(5분) + 경고 화면 UI | `acquire_lock`, `release_lock`, `check_lock_status` | - (파일 기반) |
| 4계층 백업 | exit/hourly/daily/weekly + 순환 삭제 + rusqlite::backup | `create_backup`, `list_backups`, `restore_backup` | - |
| 무결성 검증 | 앱 시작 시 PRAGMA integrity_check + 손상 시 자동 복원 | `check_integrity`, `auto_restore` | - |
| 동기화 대기 | DB/락 파일 최신 동기화 확인 + 30초 타임아웃 | `check_sync_status` | - |
| 감사 로그 | audit_logs 테이블 + 로깅 미들웨어 | `get_audit_logs` | V008 (audit_logs) |
| 코드 테이블 | schools, payment_methods, card_companies, standard_fees | - | V001 |
| 에러 처리 기반 | `thiserror` 커스텀 에러 타입 + `src-tauri/src/error.rs` | - | - |
| 앱 시작 시퀀스 | 동기화 대기 → 락 확인 → 무결성 검증 → 인증 → 메인 진입 (< 3초) | `app_startup_sequence` | - |

**완료 기준**:
- ⬜ SQLCipher AES-256 암호화된 DB로 CRUD 동작 확인 (양 OS)
- ⬜ app.lock으로 양 PC 시점 분리 동작 검증
- ⬜ 4계층 백업이 지정 트리거에서 정상 생성/순환 삭제
- ⬜ PRAGMA integrity_check 통과 + 손상 시 자동 복원 동작
- ⬜ 앱 시작 시퀀스 전체 < 3초 (락 확인 + 무결성 검증 + 인증 포함)
- ⬜ 복구 코드 발급/검증/재발급 동작
- ⬜ `cargo test` + `cargo clippy -- -D warnings` 통과
- ⬜ ADR-001 (SQLCipher), ADR-002 (app.lock), ADR-003 (백업), ADR-004 (Keychain) 문서 완료

**산출물**:
- `docs/arch/adr-001-sqlcipher-integration.md`
- `docs/arch/adr-002-applock-library.md`
- `docs/arch/adr-003-backup-implementation.md`
- `docs/arch/adr-004-keychain-crate.md`
- `src-tauri/migrations/V001__create_schools_and_payment_codes.sql`
- `src-tauri/migrations/V008__create_app_settings_and_audit_logs.sql`
- `src-tauri/src/commands/auth.rs`
- `src-tauri/src/commands/lock.rs`
- `src-tauri/src/commands/backup.rs`
- `src-tauri/src/commands/integrity.rs`
- `src-tauri/src/error.rs`

---

### Sprint 2: 기반 도메인 백엔드 (2주)

**목표**: 원생/스케줄/표준교습비/코드 테이블의 DB 스키마와 IPC 커맨드를 구현하여, Sprint 3 프론트엔드가 호출할 수 있는 완전한 백엔드 API를 제공한다.

**사전 조건**: Sprint 1 완료 + PI-05 사용자 결정 완료

**구현 범위**:

| 카테고리 | 작업 | IPC 커맨드 | 마이그레이션 |
|----------|------|-----------|-------------|
| 원생 CRUD | 등록/수정/조회/필터링/퇴교 | `create_student`, `update_student`, `list_students`, `get_student`, `withdraw_student` | V002 |
| 수업 스케줄 | 설정/조회/이력관리/주 총 수업시간 산정 | `set_schedule`, `get_schedules`, `get_weekly_hours`, `get_schedule_history` | V002 |
| 표준 교습비 | CRUD + 주 수업시간 → 교습비 매칭 | `list_fees`, `create_fee`, `update_fee`, `match_fee_by_hours` | V001 (이미 생성) |
| 코드 테이블 | 학교/결제수단/카드사 공용 CRUD | `list_codes`, `create_code`, `update_code`, `reorder_codes` | V001 (이미 생성) |
| 학사 도메인 스키마 | study_periods + schedule_codes + schedule_events | - (Sprint 4에서 IPC 구현) | V003, V004 |
| 비즈니스 규칙 | 재원생 판정, (원생,요일) UNIQUE 검증, 스케줄 변경 이력 자동 생성, 주 총 수업시간 계산 | - | - |

**완료 기준**:
- ⬜ V002~V004 마이그레이션 정상 적용 + `.sqlx/` 오프라인 캐시 갱신/커밋
- ⬜ IPC 커맨드별 단위 테스트 통과 (인메모리 DB, SQLCipher 미적용 모드)
- ⬜ 원생 50명 기준 CRUD 응답 300ms 이내
- ⬜ 비즈니스 규칙 단위 테스트 100% 커버
- ⬜ `cargo test` + `cargo clippy -- -D warnings` 통과

**산출물**:
- `src-tauri/migrations/V002__create_students_and_schedules.sql`
- `src-tauri/migrations/V003__create_study_periods_and_schedule_codes.sql`
- `src-tauri/migrations/V004__create_schedule_events.sql`
- `src-tauri/src/commands/students.rs`
- `src-tauri/src/commands/schedules.rs`
- `src-tauri/src/commands/fees.rs`
- `src-tauri/src/commands/codes.rs`

---

### Sprint 3: 원생 관리 프론트 + 초기 설정 마법사 (2주)

**목표**: 앱 레이아웃 셸, 초기 설정 마법사, 원생 관리 UI, 글로벌 검색, 접근성 기반을 구축하여 첫 사용 가능한 화면을 제공한다.

**사전 조건**: Sprint 2 완료

**구현 범위**:

| 카테고리 | 작업 | 프론트엔드 컴포넌트 |
|----------|------|-------------------|
| 앱 레이아웃 셸 | 사이드바 네비게이션 + 상단 상태바 + 글로벌 검색바 | `src/app/layout.tsx`, `src/components/Sidebar.tsx`, `src/components/StatusBar.tsx` |
| 글로벌 검색 | 한글 자모 부분 일치 + 200ms 디바운싱 + 300ms 결과 + Ctrl+F | `src/components/GlobalSearch.tsx`, IPC: `search_global` |
| 원생 관리 | 목록 + 등록/수정 폼 + 필터/정렬 + 퇴교 + Ctrl+N | `src/app/students/page.tsx`, `src/components/StudentForm.tsx` |
| 수업 스케줄 편집 | 요일별 시작 시간 + 1회 수업 시간 + 운영 시간 제약 + 교습비 매칭 | `src/components/ScheduleEditor.tsx` |
| 초기 설정 마법사 | 9단계 + 건너뛰기 + 독립 저장 + 미완료 시 재진입 | `src/app/wizard/page.tsx`, `src/components/wizard/Step{1-9}.tsx` |
| 코드 테이블 관리 | 학교/표준교습비/결제수단/카드사/운영시간 | `src/app/settings/codes/page.tsx` |
| 상태 관리 설정 | Zustand 스토어 + TanStack Query 패턴 확립 | `src/stores/`, `src/hooks/queries/` |
| 접근성 기반 | Pretendard 18pt + WCAG AA + 44x44px + 저자극 톤 | `src/app/globals.css`, Tailwind config |
| 단축키 체계 | F1/Ctrl+F/Ctrl+N/Ctrl+S/Ctrl+Z/ESC/Ctrl+P | `src/hooks/useKeyboardShortcuts.ts` |
| Tauri IPC 추상화 | Sprint 2 IPC 커맨드 래퍼 | `src/lib/tauri/index.ts` |
| 실수 복구 | 3분 자동 임시저장 + 미저장 경고 + 1단계 Undo | `src/hooks/useAutoSave.ts`, `src/hooks/useUnsavedGuard.ts` |

**완료 기준**:
- ⬜ 원생 등록/수정/조회/퇴교 전체 흐름 동작
- ⬜ 초기 설정 마법사 9단계 완주 가능
- ⬜ 글로벌 검색바에서 원생 이름 검색 + 1클릭 이동
- ⬜ Pretendard 18pt, 44x44px 클릭 영역, WCAG AA 명도 대비 확인
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` 통과
- ⬜ 양 OS(Win+Mac)에서 `pnpm tauri:dev` 실행 확인
- ⬜ ADR-006 (Pretendard 임베드) 문서 완료

**산출물**:
- `docs/arch/adr-006-pretendard-embed.md`
- `src/app/layout.tsx` (레이아웃 셸)
- `src/app/wizard/page.tsx` (초기 설정 마법사)
- `src/app/students/page.tsx` (원생 관리)
- `src/components/GlobalSearch.tsx`
- `src/lib/tauri/index.ts` (IPC 추상화 레이어)
- `src/stores/` (Zustand 스토어)
- `src/hooks/` (TanStack Query 훅, 단축키, 자동저장)

---

## 재사용 가능한 기존 코드

| 파일 경로 | 재사용 방법 |
|----------|------------|
| `src-tauri/src/commands/mod.rs` | 기존 greet 커맨드 구조를 패턴으로 활용 |
| `src-tauri/src/lib.rs` | invoke_handler 등록 패턴 확장 |
| `src/lib/tauri/index.ts` | 기존 invoke 추상화 패턴 확장 |
| `src-tauri/Cargo.toml` | sqlx, tokio, thiserror 이미 설정됨 |
| `docs/data-model.md` | V001~V008 마이그레이션 스키마 참조 가이드 |

---

## 리스크 및 완화 전략

| # | 리스크 | 영향도 | 완화 방법 |
|---|--------|--------|----------|
| R1 | SQLCipher + sqlx 통합 빌드 실패 (양 OS) | 높음 | Sprint 1 첫 2일에 PoC 실행. bundled-sqlcipher 실패 시 시스템 sqlcipher 대체안 ADR에 명시 |
| R2 | `rusqlite` + `sqlx` 동시 사용 시 DB 커넥션 충돌 | 중간 | 백업 전용 rusqlite 커넥션은 읽기 전용으로 제한. sqlx pool과 격리된 별도 경로로 open |
| R3 | 앱 시작 시퀀스 3초 초과 (동기화 대기 + 락 + 무결성 + 인증) | 높음 | 동기화 대기는 비동기 + 타임아웃 30초 별도. 락 확인 + 무결성(소규모 DB ~5MB) + 인증은 500ms 이내 목표. 무결성 검증은 `PRAGMA quick_check` 대체 검토 |
| R4 | PI-05 미결정으로 Sprint 2 착수 지연 | 중간 | 미결정 시 수동 입력만 구현 (자동 채번은 Post-Sprint 2로 이연). sprint-planner가 착수 전 사용자 확인 |
| R5 | 초기 설정 마법사 9단계 구현 볼륨 과다 (Sprint 3) | 중간 | 각 단계를 독립 컴포넌트로 설계. 저장 로직은 기존 IPC 재사용. UI는 shadcn/ui wizard 패턴 활용 |
| R6 | 한글 자모 검색 구현 복잡도 | 중간 | 한글 자모 분해 라이브러리(`hangul-js` 등) 활용. 프론트엔드에서 자모 분해 후 필터링 — 원생 50명 규모라 클라이언트 사이드 충분 |
| R7 | keyring crate 양 OS 호환성 이슈 | 중간 | Sprint 1 PoC에서 양 OS 테스트. 실패 시 ADR-004 대체안(platform-specific crate) 채택 |

---

## 전문가 검토 요약

### 보안 전문가 (상세: `docs/phase/phase1/phase1-보안전문가-review.md`)
- **[필수]** SQLCipher 키는 메모리에 최소 시간만 보유 — DB 열기 직후 키 바이트 zeroize 필수. `zeroize` crate 도입 권장
- **[필수]** 복구 코드 해시는 Argon2id 사용 (PBKDF2보다 메모리 하드, 무차별 대입 저항). 평문은 발급 1회 표시 후 즉시 메모리에서 zeroize
- **[필수]** 백업 파일도 SQLCipher 암호화 상태 유지 — rusqlite::backup은 암호화된 상태로 복사되므로 별도 처리 불필요 (확인만 필요)
- **[권고]** app.lock에 디바이스 ID는 랜덤 UUID 사용 — MAC 주소나 하드웨어 시리얼 사용 금지
- **[권고]** 비밀번호 분실 + 복구 코드 분실 시 데이터 영구 접근 불가 경고를 마법사와 설정 화면에 명시적으로 표시

### 성능 엔지니어 (상세: `docs/phase/phase1/phase1-성능엔지니어-review.md`)
- **[필수]** 앱 시작 3초 보장을 위해 무결성 검증은 `PRAGMA quick_check` 사용 (integrity_check 대비 10~100배 빠름). DB 5MB 기준 quick_check ~50ms
- **[필수]** 앱 시작 시퀀스 병렬화: 동기화 상태 확인과 락 파일 읽기를 병렬 실행. 무결성 검증은 DB 열기 직후 비동기 실행
- **[권고]** 글로벌 검색(원생 50명)은 프론트엔드 메모리 캐시로 충분 — TanStack Query로 원생 목록 캐싱 후 클라이언트 사이드 필터링. IPC 왕복 불필요
- **[권고]** 4계층 백업 중 hourly 백업은 DB 5MB 기준 ~50ms — 성능 영향 무시 가능. 단, 트랜잭션 중 백업은 WAL 모드에서만 안전

### UX 전문가 (상세: `docs/phase/phase1/phase1-UX전문가-review.md`)
- **[필수]** 마법사 9단계는 50대 사용자에게 과다 — **5~6단계로 축소** 권장. 환영(1) + 데이터 저장 위치(2) + 운영시간+표준교습비(3, 통합) + 학교코드+결제수단(4, 통합) + 샘플 원생(5) + 완료(6). 가져오기는 설정 메뉴로 이관
- **[보수적 채택]** PRD §4.0 원문 9단계를 유지하되, 각 단계에 "건너뛰기" 버튼 크기를 44x44px 이상으로 확보하고, 진행률 표시(1/9, 2/9...)를 상단에 명확히 표시. 단계 통합은 사용자 UAT 피드백 후 Phase 7에서 검토
- **[필수]** 잠금 화면의 비밀번호 입력 필드: 큰 입력창(높이 56px+), "비밀번호 표시" 토글, 입력 오류 시 빨간 테두리 + 명확한 한국어 안내 메시지
- **[권고]** 글로벌 검색바: 검색 결과 드롭다운에 원생 프로필 아이콘 + 학교/학년 서브텍스트로 동명이인 구분

### PO / 인프라 관점 (상세: `docs/phase/phase1/phase1-PO-review.md`)
- **[필수]** CI에서 SQLCipher 빌드: `bundled-sqlcipher` feature는 OpenSSL/LibreSSL 의존 — GitHub Actions에서 `libssl-dev`(Ubuntu) 또는 Homebrew `openssl`(macOS) 설치 필요. Windows는 `vcpkg`로 OpenSSL 설정 또는 `bundled-sqlcipher-vendored-openssl` feature 사용
- **[필수]** `bundled-sqlcipher-vendored-openssl` feature 존재 확인 — 이 feature가 있으면 CI에 별도 OpenSSL 설치 불필요 (가장 단순한 경로)
- **[권고]** Sprint 1 첫 2일은 CI 파이프라인에서 양 OS 빌드 성공 확인에 집중 — ADR-001 PoC는 로컬 + CI 양쪽에서 검증
- **[권고]** Phase 1 완료 시점에 `pnpm tauri:build`로 양 OS 인스톨러 생성 가능 상태 확인 (Phase 7 빌드 검증 사전 준비)

---

## 의존성 맵 (Sprint 간)

```
Sprint 1 (인프라)
  └── SQLCipher + 인증 + app.lock + 백업 + 무결성
  └── V001 (코드 테이블) + V008 (app_settings, audit_logs)
       │
       ▼
Sprint 2 (기반 도메인 백엔드) ← Sprint 1 인프라 필수
  └── V002 (원생/스케줄) + V003 (교습기간/학사코드) + V004 (학사일정)
  └── 원생/스케줄/교습비/코드 IPC 커맨드
       │
       ▼
Sprint 3 (프론트엔드) ← Sprint 2 백엔드 API 필수
  └── 앱 레이아웃 + 마법사 + 원생 관리 UI + 글로벌 검색
  └── 접근성 기반 + 상태 관리 패턴 확립
```

---

## Phase 1 완료 후 상태

Phase 1 완료 시 시스템은 다음 상태가 된다:
- SQLCipher 암호화 DB가 클라우드 동기화 폴더에서 안전하게 동작
- 양 PC 시점 분리 사용이 app.lock으로 보장됨
- 4계층 백업이 자동으로 생성/순환 삭제됨
- 50대 사용자가 마법사를 통해 초기 설정을 완료하고, 원생을 등록/관리할 수 있음
- 글로벌 검색으로 원생을 빠르게 찾을 수 있음
- 접근성 기준(Pretendard 18pt, WCAG AA, 44x44px)이 확립되어 후속 Phase에서 일관되게 적용됨

**Phase 2 진입 조건**: Phase 1 Sprint 3의 모든 완료 기준 충족 + 양 OS 동작 확인
