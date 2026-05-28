# ROADMAP.md

## 개요
- **프로젝트 목표**: 50대 교습소 원장 1인을 위한 원생/출결/보강/청구/단원평가 통합 데스크톱 관리 앱 (Windows + macOS)
- **전체 예상 기간**: 약 32주 (17 스프린트, 7 Phase) + UAT 2주 = 총 34주
- **현재 진행 단계**: Phase 0 준비 완료, Phase 1 착수 예정
- **PRD 버전**: v1.5 (2026-05-15)

---

## 진행 상태 범례

- ✅ 완료
- 🔄 진행 중
- 📋 예정
- ⏸️ 보류

---

## 📊 프로젝트 현황 대시보드

| 항목 | 내용 |
|------|------|
| 전체 진행률 | 65% (10/17 스프린트 완료) |
| 현재 Phase | Phase 3 완료 (2026-05-28) — Sprint 9+10 완료. Phase 4 (청구+수납+공지문) 착수 예정 |
| 다음 마일스톤 | 청구 생성 + 3단계 마감 + 수납 관리 (Sprint 11) |
| MVP 범위 | PRD §4.0~§4.14, §5.3~§5.5, §6.6 (Post-MVP §4.15 제외) |
| 팀 규모 가정 | AI 페어 프로그래밍 1인 개발 (2주 스프린트) |

---

## 🏗️ 기술 아키텍처 결정 사항

| 결정 항목 | 선택 | 사유 | 결정 시점 |
|-----------|------|------|-----------|
| 데스크톱 셸 | Tauri 2 (Rust) | 메모리 효율, 작은 번들, Rust 보안 모델 | PRD v1.0 |
| 프론트엔드 | Next.js 15 + React 19 | `output: 'export'` 정적 빌드, App Router | 프로젝트 착수 |
| DB | SQLite + sqlx 0.8 | 로컬 단일 파일, 오프라인 우선 | PRD v1.1 |
| DB 암호화 | SQLCipher AES-256 | 개인정보보호법 준수, OS Keychain 키 보관 | PRD v1.1 |
| 상태 관리 | Zustand + TanStack Query | 전역 상태 + IPC 응답 캐싱 | PRD §5.1 |
| UI 컴포넌트 | shadcn/ui + Tailwind CSS | 접근성 커스터마이징 용이 | PRD §5.1 |
| 캘린더 라이브러리 | FullCalendar (MIT) | ADR-006: React Big Calendar 대비 일/주/월 뷰 지원 + TypeScript + static export 호환성 우위 | Sprint 10 T8 |
| E2E 테스트 | Tauri WebDriver (tauri-driver) | Tauri 공식 권장, PI-11 해결 | PRD v1.5 |

---

## 🔗 의존성 맵

```
Phase 1 (인프라+기반)
  ├── Sprint 1: Tauri 셸 + SQLCipher + app.lock + 4계층 백업 + 무결성 검증
  ├── Sprint 2: 원생/스케줄/표준교습비/코드 테이블 (DB + 백엔드)
  └── Sprint 3: 원생 관리 프론트 + 초기 설정 마법사 + 글로벌 검색 + 접근성 기반

Phase 1.5 (품질 안정화)  ← Phase 1 완료 필수
  └── Sprint 4: Phase 1 스테이징 검증 14개 이슈 해소 + 교습소 설정 화면

Phase 1.5b (추가 안정화)  ← Phase 1.5 완료 필수
  └── Sprint 5: 환경 호환 + 다중 인스턴스 차단 + 시드 보정

Phase 2 (학사+출결)  ← Phase 1.5b 완료 필수
  ├── Sprint 6: 학사 스케줄 (3개월 캘린더 + 코드 3속성 + 교습기간)
  ├── Sprint 7: carry-over 해소 (Keychain 최적화 + 교습기간 UX + 배치 제약 + salt 이전)
  └── Sprint 8: 출결 생성 + 출결표 UI + 상태 토글 (캘린더 ADR)

Phase 3 (보강+소멸)  ← Phase 2 완료 필수
  ├── Sprint 9: 보강 등록(개별) + 보강데이 일괄 + 매칭 로직
  └── Sprint 10: 소멸 자동 전이 + 퇴교 보강 처리 + 캘린더 뷰

Phase 4 (청구·수납)  ← Phase 2 완료 필수 (Phase 3과 병행 불가 — 출결 의존)
  ├── Sprint 11: 청구 생성 + 3단계 마감 + 수납 관리
  └── Sprint 12: 공지문 이미지 일괄 생성 (HTML5 Canvas)

Phase 5 (단원평가+보고서)  ← Phase 2 완료 필수
  ├── Sprint 13: 단원평가 점수 입력 + 추이 조회
  └── Sprint 14: 분기 학습보고서 + A4 4분할 인쇄/PDF

Phase 6 (대시보드+유틸)  ← Phase 3~5 모두 완료 필수
  └── Sprint 15: 대시보드 5개 위젯 + 알림 + 가져오기/내보내기 + 자가 진단

Phase 7 (안정화+UAT)  ← Phase 6 완료 필수
  ├── Sprint 16: 양 OS 빌드 검증 + 성능 최적화 + 접근성 감사
  └── Sprint 17: UAT(원장 2주 파일럿) + 피드백 반영 + v1.0 릴리즈
```

---

## ⚠️ 리스크 및 완화 전략

| # | 리스크 | 영향도 | 발생 확률 | 완화 전략 |
|---|--------|--------|----------|-----------|
| R1 | SQLCipher + sqlx 통합 실패 | 높음 | 중간 | Sprint 1에서 ADR + PoC. `libsqlite3-sys` bundled-sqlcipher feature 우선 시도, 실패 시 시스템 sqlcipher 라이브러리 |
| R2 | app.lock 클라우드 동기화 지연 → 양 PC 동시 접근 | 높음 | 낮음 | PI-09 완화안 적용 (강제 점유 시 경고 + 무결성 검증), 5분 임계값 설정 가능 |
| R3 | MYBOX 30GB 한도 초과 (백업 누적) | 중간 | 낮음 | PI-08 — 원생 50명 기준 DB ~5MB, 백업 68개 ~340MB로 안전. 이상치 감지 로직 추가 |
| R4 | 50대 사용자 UI 적응 실패 | 높음 | 중간 | Phase 1 Sprint 3에서 접근성 기반 구축, Phase 7 UAT 2주로 실사용 검증 |
| R5 | 보강 매칭 로직 복잡도 (PI-02 미결정) | 중간 | 높음 | Sprint 8 진입 전 PI-02 사용자 결정 필수, 미결정 시 "시간 무관 일 단위 매칭" 보수적 채택 |
| R6 | 캘린더 라이브러리 선택 오류 | 중간 | 중간 | Sprint 7에서 ADR 작성 후 구현, PoC 비교 |
| R7 | PI-01 소멸 자동 전이 트리거 누락 | 높음 | 높음 | Sprint 9에서 앱 시작 시 batch 로직 구현 (별도 트리거) |
| R8 | PI-07 복구 코드 Feature 미정의 | 중간 | 높음 | Sprint 1 인증 구현 시 사용자에게 PI-07 결정 요청, 미결정 시 비밀번호만 MVP |

---

## Phase 1: 인프라 + 기반 도메인 (Sprint 1~3) ✅ 완료 (2026-05-21)

> **Phase 설계 완료**: `docs/phase/phase1.md` — 전문가 4관점 검토 반영 (보안/성능/UX/PO+인프라)

### 목표
데이터 영역의 모든 후속 작업이 의존하는 인프라(SQLCipher, app.lock, 백업, 무결성 검증)를 확립하고, 원생/스케줄/코드 테이블 기반 도메인을 완성하여 첫 사용 가능한 화면(원생 관리 + 초기 설정 마법사)을 제공한다.

### Sprint 1: 데이터 인프라 (2주) ✅ 완료 (2026-05-19)

> PR: (sprint1 → develop, 2026-05-19)
> ADR: adr-001 / adr-002 / adr-003 / adr-004 완료

#### 작업 목록

- ✅ **SQLCipher 통합 ADR 작성 및 PoC**: `libsqlite3-sys` bundled-sqlcipher feature vs 시스템 sqlcipher 비교
  - Cargo.toml 의존성 설정 (sqlx + sqlcipher)
  - Windows/macOS 양 플랫폼 빌드 검증
  - ADR 문서: `docs/arch/adr-001-sqlcipher-integration.md`
- ✅ **OS Keychain/Credential Manager 통합**: `keyring` crate 도입
  - 비밀번호 기반 PBKDF2 키 유도 구현 (600K iter + zeroize)
  - PI-07 결정 반영: Argon2id 12자리 31자 알파벳 복구 코드
  - ADR 문서: `docs/arch/adr-004-keychain-crate.md`
- ✅ **인증 화면 (앱 시작 잠금)**: 비밀번호 입력 → DB 복호화 → 진입
  - Tauri IPC 커맨드: `set_password`, `unlock_db`, `check_auth_status`
  - 프론트엔드: 잠금 화면 UI (Pretendard 18pt, 44x44px 버튼)
- ✅ **app.lock 동시성 제어**: 락 생성/확인/heartbeat(60초)/강제 점유(5분)
  - `fs2` crate advisory locking + 자체 heartbeat 구현
  - ADR 문서: `docs/arch/adr-002-applock-library.md`
- ✅ **4계층 자동 백업**: SQLite Online Backup API 활용
  - exit(10) / hourly(24) / daily(30) / weekly(4) 구현
  - 파일명: `app_YYYYMMDD_HHMMSS.db`, 순환 삭제
  - ADR 문서: `docs/arch/adr-003-backup-implementation.md`
- ✅ **무결성 검증**: 앱 시작 시 `PRAGMA quick_check / integrity_check`
  - 손상 감지 시 최신 백업 자동 복원 + restore_rollback 안전망
  - 손상본 격리 보관 로직
- ✅ **동기화 완료 대기**: DB/락 파일 최신 동기화 확인 로직 구현
- ✅ **감사 로그 기반 구축**: `audit_logs` 테이블 + 7곳 audit 통합
- ✅ **DB 마이그레이션 V001**: 코드성 테이블 (schools, payment_methods, card_companies, standard_fees)
- ✅ **DB 마이그레이션 V008**: app_settings + audit_logs
- ✅ **앱 시작 시퀀스**: tokio::join! 락+무결성 병렬 + PRD §5.6 < 3초 구현
- ✅ **paths/runtime/app_err! 통합**: 공통 헬퍼 모듈 분리 + 단위 테스트 74건
- ✅ **CI 매트릭스**: ci.yml + deploy.yml cipher feature 양 OS 빌드 (Strawberry Perl)

#### 완료 기준 (Definition of Done)
- ✅ SQLCipher AES-256 암호화된 DB 파일로 CRUD 동작 확인
- ✅ app.lock으로 양 PC 시점 분리 동작 검증
- ✅ 4계층 백업이 지정 트리거에서 정상 생성/순환 삭제
- ✅ `PRAGMA integrity_check` 통과 + 손상 시 자동 복원 동작
- ✅ `cargo test` 74 passed + `cargo clippy -- -D warnings` 통과
- ✅ ADR-001/002/003/004 문서 완료

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420 (Tauri dev)
2. browser_snapshot → 잠금 화면 렌더링 확인 (비밀번호 입력 필드)
3. browser_click → 비밀번호 입력 + "잠금 해제" 버튼 클릭
4. browser_snapshot → 메인 화면 진입 확인 (또는 에러 메시지)
5. browser_console_messages(level: "error") → 콘솔 에러 없음 확인
```

#### 기술 고려사항
- `libsqlite3-sys` bundled-sqlcipher feature가 양 OS에서 빌드 가능한지 PoC 필수
- `keyring` crate는 macOS/Windows 양 플랫폼 지원 확인
- 백업은 SQLCipher 암호화 상태 그대로 보관 (복호화 금지)
- **PI-07 결정 필요**: 복구 코드 발급/검증 Feature 범위 (Sprint 1 착수 전 사용자 확인)

---

### Sprint 2: 기반 도메인 백엔드 (2주) ✅ 완료 (2026-05-20)

#### 작업 목록

- ✅ **루트 라우팅 + 인증 게이트**: `src/app/page.tsx` 클라이언트 가드, `lock/page.tsx` onUnlocked 흐름
- ✅ **DB 마이그레이션 V101**: students + student_schedules 테이블 (PI-05 자동 채번 `MAX+1` + `BEGIN IMMEDIATE`)
- ✅ **DB 마이그레이션 V102**: study_periods + schedule_codes 테이블 (시스템 예약 5종 시드)
- ✅ **DB 마이그레이션 V103**: schedule_events 테이블
- ✅ **DB 마이그레이션 V104**: standard_fees 재설계 (weekly_minutes 기준 매칭)
- ✅ **DB 마이그레이션 V105**: schools 보강 (school_type / region 컬럼)
- ✅ **원생 CRUD IPC 커맨드**: `create_student`, `update_student`, `list_students`, `get_student`
  - 필터링: 이름/학교급/학년/학교명/요일/성별 다중 조합
  - 정렬: 이름순/입교일순/학년순, 재원 상태 필터
- ✅ **수업 스케줄 IPC 커맨드**: `set_schedule`, `get_schedules`, `get_weekly_hours`
  - (원생, 요일) UNIQUE 검증, 변경 이력 자동 생성, 주 총 수업시간 자동 산정
- ✅ **표준 교습비 IPC 커맨드**: `list_fees`, `create_fee`, `update_fee` + 자동 매칭 함수
- ✅ **코드 테이블 CRUD IPC 커맨드**: 학교/결제수단/카드사 공용 CRUD (is_active 소프트 삭제, sort_order)
- ✅ **프론트엔드 IPC 래퍼**: `src/lib/tauri/index.ts` Sprint 2 래퍼 18개 추가 (dev mode fallback 포함)
- ✅ **도메인 타입**: `src/types/{student,schedule,fee,code}.ts` 4종
- ✅ **비즈니스 규칙 단위 테스트**: 97 passed (Sprint 1 64 → +33)

> **이연**: T2 salt 이전(Sprint 3 마법사 통합 시점, R12), T8 `query!()` 매크로(별도 backlog)

#### 완료 기준 (Definition of Done)
- ✅ 모든 마이그레이션 정상 적용 + `.sqlx/` 오프라인 캐시 갱신/커밋
- ✅ IPC 커맨드별 단위 테스트 통과 (인메모리 DB)
- ✅ 원생 50명 기준 CRUD 응답 300ms 이내 (실측은 v0.2.0 배포 후 사용자 환경)
- ✅ `cargo test` 97 passed + `cargo clippy -- -D warnings` 통과

#### 🧪 Playwright MCP 검증 시나리오
```
(Sprint 2는 백엔드 중심 — Tauri IPC 단위 테스트로 검증)
1. cargo test 전체 통과 확인
2. sqlx prepare 오프라인 캐시 정상 생성 확인
```

#### 기술 고려사항
- `query!`/`query_as!` 매크로로 컴파일 타임 타입 검사 필수
- 에러 처리: `thiserror` + `Result<T, String>` (Tauri 커맨드 규약)
- PI-05 확정: 자동 채번 구현 (`MAX+1` + override 허용)

---

### Sprint 3: 원생 관리 프론트 + 초기 설정 마법사 (2주) ✅ 완료 (2026-05-21)

> 계획 문서: `docs/sprint/sprint3.md` / Task 15/15 완료

#### 작업 목록

- ✅ **T1: Pretendard 폰트 self-host** — `public/fonts/` woff2 배치, `@font-face` + Tailwind config, 18px 본문/24px+ 헤더/행간 1.5 기본값
- ✅ **T2: R13 PII 마스킹** — `students.rs` `try_record` 3곳 `details=None` 적용
- ✅ **T3: R14 페이지네이션** — `list_students`/`list_codes` LIMIT/OFFSET + `count_students` IPC 신규
- ✅ **T4: Zustand 스토어 + TanStack Query** — session-store/app-store + QueryProvider
- ✅ **T5: 앱 레이아웃 셸** — 사이드바(단축키 병기) + 상단 상태바 + AppShell
- ✅ **T6: 글로벌 검색바 (§4.14)** — 원생/학교명/메뉴명 검색, 200ms 디바운싱, Ctrl+F
- ✅ **T7: tauri-plugin-dialog** — 폴더 선택 IPC + capabilities 권한
- ✅ **T8: 마법사 백엔드** — `setup.rs` IPC(save_cloud_folder/complete_setup/get_setup_status) + config.json 분리 (chicken-and-egg step-back → `app_config_dir` 채택)
- ✅ **T9: 초기 설정 마법사 프론트** — `/setup` 4단계(환영/폴더/비밀번호/완료) + 라우팅 분기
- ✅ **T10: 원생 목록 화면** — TanStack Query 연동, 필터/정렬/페이지네이션
- ✅ **T11: 원생 등록/수정 폼** — CRUD 전체 흐름, 3분 자동 임시저장, 미저장 경고
- ✅ **T12: 코드 테이블 관리 화면 (§4.12)** — 학교/표준교습비/결제수단/카드사 CRUD
- ✅ **T13: 수업 스케줄 편집 UI (§4.2)** — 요일별 시간 입력, 주 총 시간/교습비 매칭 표시
- ✅ **T14: 키보드 단축키 체계** — F1/Ctrl+F/Ctrl+N/Ctrl+S/Ctrl+Z/ESC/Ctrl+P
- ✅ **T15: 통합 검증** — cargo test 109 passed (+12), clippy/lint/tsc/build 모두 통과

> **이연 항목** (Sprint 4 또는 hotfix sweep): R12 salt 이전 (`paths::data_root()` 동적화), T8 `query!()` 매크로 전환 (별도 backlog)

#### 완료 기준 (Definition of Done)
- ✅ 원생 등록/수정/조회/퇴교 전체 흐름 동작
- ✅ 초기 설정 마법사 4단계 완주 가능
- ✅ 글로벌 검색바에서 원생 이름 검색 + 1클릭 이동
- ✅ Pretendard 18pt, 44x44px 클릭 영역, WCAG AA 명도 대비 확인
- ✅ `cargo test` 109 passed, `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420
2. browser_snapshot → 초기 설정 마법사 환영 화면 확인
3. browser_click → 각 단계 "다음" 또는 "건너뛰기" 클릭
4. browser_snapshot → 마법사 완료 후 대시보드 진입 확인
5. browser_click → 사이드바 "원생 관리" 메뉴 클릭
6. browser_snapshot → 원생 목록 화면 렌더링 확인
7. browser_click → "신규 원생 등록" 버튼 클릭
8. browser_snapshot → 등록 폼 렌더링 확인
9. browser_console_messages(level: "error") → 콘솔 에러 없음
```

#### 기술 고려사항
- `src/lib/tauri/` 추상화 레이어 경유 필수 (invoke 직접 호출 금지)
- `'use client'`는 Tauri IPC 필요한 최소 컴포넌트에만 적용
- `typeof window !== 'undefined'` 가드 필수 (SSR 대응)
- `next/image` 사용 시 `unoptimized` 속성 필수

---

## Phase 1.5: 품질 안정화 (Sprint 4) ✅ 완료 (2026-05-21)

### Sprint 4: Phase 1 품질 안정화 — 스테이징 검증 14개 이슈 해소 (2주) ✅ 완료 (2026-05-21)

> 계획 문서: `docs/sprint/sprint4.md`
> Sprint 3 스테이징 검증에서 발견된 Critical Runtime Error 1건 + 사용자 보고 13건을 모두 해결.
> 교습소 설정 화면 신설, 원생 관리 UX 개선, 코드 테이블 완성도 향상, 수업 스케줄 편집 기능 보강.
> post-T11 추가 4건 (원생 목록 수업요일/시간 컬럼, 스케줄 폼 위치/1시간 단위/운영시간 디폴트/번호정렬) 포함.

#### 작업 목록

- ✅ **T1**: window.confirm 차단 해소 + shadcn/ui Dialog 도입 + capabilities 권한 정비
- ✅ **T2**: 교습소 설정 메뉴 화면 신설 (§4.12)
- ✅ **T3**: 상태바 점유/백업/동기화 + 시작 시간 표시 수정
- ✅ **T4**: 원생 등록/수정 — 학교명 선택란 추가 + 필터 연동
- ✅ **T5**: 연락처 자동 하이픈 + 금액 천단위 콤마 유틸리티
- ✅ **T6**: 원생 일련번호 수정 차단
- ✅ **T7**: 원생 등록 후 수업 스케줄 등록 UX 개선
- ✅ **T8**: 퇴교일 필드 추가 + 퇴교 번복 기능 (V101 withdraw_date 기존 컬럼 활용)
- ✅ **T9**: 수업 스케줄 시작시간 콤보박스 + 수정/삭제 기능
- ✅ **T10**: 코드 테이블 드래그 순서 변경 + 활성 상태 필터
- ✅ **T11**: 통합 검증 + 14개 이슈 전수 재검증

#### 완료 기준 (Definition of Done)
- ✅ 14개 사용자 보고 이슈 전수 재검증 통과 (사용자 시각 검증 "이상없음" 2026-05-21)
- ✅ cargo test 130 passed + clippy/lint/tsc/build 무오류
- ✅ `pnpm tauri:dev` 실행 후 마법사 → 메인 → 전체 흐름 검증 완료

---

## Phase 1.5b: 추가 안정화 (Sprint 5) ✅ 완료 (2026-05-22)

### Sprint 5: Phase 1.5b 안정화 — 환경 호환 + 다중 인스턴스 차단 + 시드 보정 (2주) ✅ 완료 (2026-05-22)

> 계획 문서: `docs/sprint/sprint5.md`
> 원래 Sprint 5(학사 스케줄)는 Sprint 6으로 이연. 스테이징 검증(2026-05-22)에서 발견된 5건 이슈 해소.
> develop 머지: `sprint5 → develop` (--no-ff, 2026-05-22)

#### 작업 목록

- ✅ **T0**: Node 25/20 cross-OS 환경 호환 (`cross-env` + `--no-experimental-webstorage`) + CVE-2025-66478 영향 분석 (`9a2349f`)
- ✅ **T1**: `tauri-plugin-single-instance` 도입 — 동일 PC 다중 인스턴스 원천 차단 (`78e6323`)
- ✅ **T1-sub**: LockPage 진입 시 락 상태 사전 체크 → LockWarning 라우팅 활성 / stale 락 자동 점유 동작 검증 (`8f4d64b`)
- ✅ **T2**: 마법사 완료 redirect 수정 (`/` → `/settings`) (`b93b887`)
- ✅ **T3+T4**: V201 마이그레이션 — 표준교습비 시드 (3/4/5/6h: 16/20/23/26만원) + 결제수단 시드 (현금 비활성 + 계좌이체/카드/결제선생/성남사랑 활성) (`09e67d7`)
- ✅ **T5**: 통합 검증 + DoD 갱신 (`2cf9b32`, `45d9d3f`)

#### 완료 기준 (Definition of Done)
- ✅ Node 25/20 양 환경 dev 서버 정상 기동
- ✅ 동일 PC 다중 인스턴스 차단 동작
- ✅ 마법사 완료 후 `/settings` 이동
- ✅ 표준교습비/결제수단 시드 운영값 반영
- ✅ cargo test 130 passed + clippy + lint + build 전체 통과

---

## Phase 2: 학사 + 출결 (Sprint 6~8) ✅ 완료 (2026-05-24)

> **스프린트 번호 이동**: Sprint 4(Phase 1.5 안정화) + Sprint 5(Phase 1.5b 안정화) 삽입으로 원래 Sprint 5(학사 스케줄) → **Sprint 6**, 원래 Sprint 6(출결 관리) → **Sprint 7 → Sprint 8**로 이연됨.
> **Sprint 7 삽입**: Sprint 6 시각 검증에서 발견된 carry-over 8건(Keychain/교습기간 UX/배치 제약/salt 이전/device_id 등) 해소를 위해 Sprint 7이 carry-over 전담 스프린트로 삽입됨. 출결 관리는 Sprint 8로 이연.

> **Phase 2 메모 (Sprint 6 발견)**: 공휴일 시드는 data.go.kr API 2028+ 미발표 제약으로 2025~2027 (64건)으로 범위 제한. 이후 매년 1월 신규 발표분을 V401(+) 마이그레이션으로 추가하는 갱신 정책 채택 (ADR-005).

### 목표
교습기간/학사일정 수립과 출결 생성/관리를 완성하여, 원장이 매월 학사 운영의 핵심 흐름(UC-2, UC-3)을 수행할 수 있게 한다.

### Sprint 6: 학사 스케줄 관리 (2주) ✅ 완료 (2026-05-22)

> 계획 문서: `docs/sprint/sprint6.md` / Task 12/12 완료 / 9 세션
> develop 머지: `sprint6 → develop` (--no-ff, 2026-05-22)

#### 작업 목록

- ✅ **T1 (A20)**: lock/page.tsx 에러 화면 재시도 버튼 추가 + lockStatus 초기화
- ✅ **T2**: V301 마이그레이션 — schedule_codes 시드 3속성 보정 + 한국 법정 공휴일 2025~2027 64건
- ✅ **T2-a**: `pnpm holidays:fetch` 빌드 스크립트 (`scripts/fetch-holidays.ts`) + tsx devDependency + KOREA_HOLIDAY_API_KEY 환경변수
- ✅ **T2-c**: ADR-005 공휴일 API 소스 + 저장 정책 결정 (`docs/arch/adr-005-holiday-api-selection.md`)
- ✅ **T3 (A21)**: paths.rs OnceLock 테스트 격리 리팩토링 — 병렬 실행 안정화
- ✅ **T4 (A22)**: 코드 DnD 필터링 시 sort_order 충돌 해소 (방법 B)
- ✅ **T5**: 교습기간 IPC 6종 (`academic.rs` 신규 — create/update/list/get/confirm/delete_study_period)
- ✅ **T6**: 학사 일정 코드 IPC 4종 (list/create/update_schedule_code, toggle_schedule_code_active)
- ✅ **T7**: 학사 일정 배치 IPC 5종 (create/update/delete/list_schedule_events, auto_place_assessment_dates)
- ✅ **T8**: TypeScript IPC 래퍼 15개 + 도메인 타입 10개 (`src/types/academic.ts`)
- ✅ **T9**: 3개월 학사 캘린더 컴포넌트 + `/academic` 라우트 (Tailwind grid-cols-7 직접 구현, R30 완화)
- ✅ **T10**: 교습기간 설정 UI — StudyPeriodEditor + 캘린더 selection 통합
- ✅ **T11**: 학사 일정 코드 + 배치 UI + @dnd-kit 드래그 이동 (단일 일자)
- ✅ **T12**: 통합 검증 — cargo test 146 passed, clippy/tsc/lint/build 모두 통과

#### 완료 기준 (Definition of Done)
- ✅ 3개월 캘린더에서 교습기간 설정/확정/읽기전용 동작
- ✅ 학사 일정 코드 5종 + 사용자 추가 코드 배치 동작
- ✅ 단원평가 응시일 자동 배치 + 수동 조정 동작
- ✅ 지난 달 데이터 수정 차단 확인
- ✅ `cargo test` 146 passed + clippy/lint/tsc/build 무오류

#### 기술 고려사항
- 3개월 캘린더: shadcn/ui Calendar 대신 Tailwind grid-cols-7 직접 구현 (R30 복잡도 회피)
- 법정 공휴일 데이터: schedule_events 테이블 통합 저장 + 매년 1월 갱신 마이그레이션 (ADR-005)
- HTML button 중첩 회피: CalendarCell 외부 button → div + role/tabIndex (배지 button 분리)
- SQLite VALUES...AS alias 미지원: column1/column2 자동 명명 우회 (V301 syntax)

---

### Sprint 7: Phase 2 carry-over 해소 + 인프라 안정화 (2주) ✅ 완료 (2026-05-22)

> 계획 문서: `docs/sprint/sprint7.md` / Task 10/10 완료 / 9 세션
> Sprint 6 시각 검증에서 발견된 carry-over 8건 전수 해소. 출결 도메인은 Sprint 8로 이연.
> develop 머지: `sprint7 → develop` (--no-ff, 2026-05-22)

#### 작업 목록

- ✅ **T1**: macOS Keychain 반복 다이얼로그 해소 — keyring 호출 통합 캐싱 (Critical UX, Issue 1) `8eb1c92`
- ✅ **T2**: salt.bin 이전 (Keychain → cloud/smarthb/salt.bin) + Critical 보안 패치 6건 (A17/A27 3회 이월 최종 해소) `4178324`
- ✅ **T3**: device_id 영속화 — stale lock 정확한 디바이스 식별 (Issue 8, PRD §5.3) `2fad4fb`
- ✅ **T4**: is_system_reserved JOIN 추가 + 프론트 하드코딩 제거 (A23, R33) `6b5f8de`
- ✅ **T5**: 학사 일정 코드 관리 → `/settings/schedule-codes` 이동 (Issue 3) `ba7ef09`
- ✅ **T6**: 교습기간 설정 UX 재설계 — 토글 제거, 자동 selection 모드 (Issue 5) `2405ca5`
- ✅ **T7**: 학사 일정 배치 제약 강화 — 중복불가 상호 차단 + 교습기간 내만 (Issue 4, R34) `84aa86f`
- ✅ **T8**: 교습기간 삭제 + cascade 삭제 (공휴일 보존) (Issue 6) `a521102`
- ✅ **T9**: 확정 교습기간 내 공휴일 삭제 차단 (Issue 7) `84aa86f`
- ✅ **T10**: 통합 검증 — 자동 검증 전수 통과 `c39dab2`

#### 완료 기준 (Definition of Done)
- ✅ Keychain 다이얼로그 최대 1회 + startup < 3초 (Issue 1) — 자동 검증 통과, 시각 검증은 sprint-review 단계
- ✅ salt.bin 클라우드 폴더 이전 완료 (A17/A27)
- ✅ device_id 영속화 + stale lock 정확한 식별 (Issue 8)
- ✅ 교습기간 UX 재설계 + 배치 제약 + cascade 삭제 + 공휴일 보호 (Issue 3~7)
- ✅ is_system_reserved JOIN + 프론트 하드코딩 제거 (R33)
- ✅ `cargo test` 177 passed (cipher off) / 127 passed (cipher on) + clippy/lint/tsc/build 무오류

#### 기술 고려사항
- Keychain 캐싱: ZeroizeOnDrop + Mutex<Option<>> 패턴, 앱 프로세스 종료 시 자동 폐기
- salt.bin 마이그레이션: Keychain → 파일 자동 1회 전환, 기존 Keychain 항목 정리
- device_id 저장 경로: OS 로컬(app_config_dir) vs 클라우드 폴더 결정 필요 (R37)

---

### Sprint 8: 출결 관리 + Sprint 7 carry-over 흡수 (2주) ✅ 완료 (2026-05-24)

> 계획 문서: `docs/sprint/sprint8.md` / Task T1~T9 + T9 follow-up 3건 완료 / 9 세션
> Phase 2 마지막 마일스톤. 출결 생성 + 출결표 UI + 상태 토글 + Sprint 7 carry-over High 4건 + Medium 6건 흡수.
> develop 머지: `sprint8 → develop` (--no-ff, 2026-05-24)

#### 작업 목록

- ✅ **T1**: DB 마이그레이션 V106 — regular_attendances + makeup_attendances 테이블
- ✅ **T2**: 출결 생성 IPC — `generate_attendances`, `check_attendance_exists`
  - "정규수업 진행 OFF" 일자 건너뜀, 입교일/퇴교일 범위 외 건너뜀, 중복 생성 방지 (AC-4.5-1)
- ✅ **T3**: 출결 조회 + 토글 IPC 6종 — `get_attendance_grid`, `toggle_attendance`, `update_absence_memo`, `get_attendance_summary` + `audit::AttendanceToggled`
- ✅ **T4**: 출결표 프론트엔드 UI — `/attendance` 라우트, `AttendanceGrid` 컴포넌트, `AbsenceMemoDialog`, 사이드바 "출결" 메뉴 활성화
- ✅ **T4 follow-up**: UX 보강 — 사이드바 너비, 요일 행, 시간 단위, 컬럼 재배치/배경색, sticky 4컬럼, 셀 너비 30% 감소, 원생 검색 필터
- ✅ **T5**: 보강필요시간/소멸기한 단위 테스트 100% (10 시나리오)
- ✅ **T6**: Sprint 7 carry-over High 4건 해소 (I-S2-2/3/4/5, R40~R43) — Keychain/auth 보안
- ✅ **T7**: Sprint 7 carry-over Medium-High 1건 해소 (I-S2-7, R45) — Keychain concurrent race
- ✅ **T8**: carry-over Medium 6항목 해소 (R46/R47/R48-a/R39/R51, A31)
- ✅ **T9**: 통합 검증 — 자동 7항목 전수 통과 + AC 일괄 마킹

#### 완료 기준 (Definition of Done)
- ✅ 출결 생성 → 출결표 표시 → 출석/결석 토글 전체 흐름 동작
- ✅ 50명 x 31일 렌더링 1초 이내
- ✅ 보강필요시간 정확 계산 (결석 토글 시 +-변경) — 단위 테스트 100% 커버
- ✅ `cargo test --lib` cipher off 221 passed / cipher on 133 passed + clippy/lint/tsc/build 무오류

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420/attendance
2. browser_click → "출결생성" 버튼 클릭
3. browser_snapshot → 출결표 렌더링 확인 (원생 x 일자 그리드)
4. browser_click → 출석 셀 클릭 (→ 결석 토글)
5. browser_snapshot → 셀 빨간색 변경 + 보강필요시간 업데이트 확인
6. browser_console_messages(level: "error") → 콘솔 에러 없음
7. browser_network_requests → IPC 호출 성공 확인
```

#### 기술 고려사항
- 출결표 대량 셀 렌더링: 가상화(virtualization) 적용 검토
- TanStack Query로 출결 데이터 캐싱 + 토글 시 낙관적 업데이트
- 캘린더 뷰 라이브러리는 ADR 결과에 따라 Sprint 8 후반에서 완성

#### 🔁 차기 sprint 이연 후보 (Sprint 8 시각 검수 + sprint-review 에서 정리)
- ⬜ **R48-b — salt buffer ZeroizeOnDrop 시그니처 변경**: `load_salt_from`/`migrate_keyring_salt_to`/`generate_salt`/`store_salt_to` 가 모두 `[u8; SALT_LEN]` raw array 반환. `Zeroizing<[u8; SALT_LEN]>` 또는 신규 wrapper struct 도입 시 호출 사이트 광범위 영향이라 Sprint 8 범위에서 분리. 캐시 진입 후엔 `CachedCredentials` ZeroizeOnDrop 으로 보호되므로 잔존 위험은 stack 임시 변수 한정.
- ⬜ **반응형 폰트/셀 너비 — 모니터 해상도 비례 조정**: 현재 `--font-size-body: 18px` 와 h1~h6, `AttendanceGrid` 셀 너비(140/62/84px)가 모두 px 고정. 1024px ↔ 2560px 모니터에서 동일 픽셀. PRD §5.7 "18pt 권장(16pt 하한)" 유지하면서 큰 모니터 확대 — `clamp()` viewport 패턴 또는 html font-size 미디어쿼리 + rem 일괄 전환. 폰트 변경 시 셀 너비도 동기 필요 (텍스트 흘러나옴 방지).
- ⬜ **원생 검색 한글 자모 부분 일치**: Sprint 8 T9 follow-up 에서 출결관리에 원생 이름 substring 검색 도입 (`/attendance` 헤더). PRD §4.14 "한글 자모 부분 일치 + 영문 대소문자 무관" 중 자모 일치는 미적용 — `ㅈ`(자) 입력으로 "장수민" 매칭 같은 자모 분해 라이브러리(예: `hangul-js`) 또는 직접 분해 알고리즘 도입 필요. 글로벌 검색바(`src/components/layout/global-search.tsx`)도 동일 패턴 점검 대상.
- ⬜ **F1 (review) — 결석 컬럼 라벨 의미 명확화**: sprint-review 발견. 출결표 헤더 "결석(일)" 이 `status='absent' AND makeup_attendance_id IS NULL` 만 카운트 (보강완료/소멸 제외). 사용자가 "총 결석"으로 오해 가능. 라벨을 "미처리 결석(일)" 로 변경하거나 툴팁/도움말 추가. `AttendanceGrid` + `compute_summary` 둘 다 점검.
- ⬜ **F3 (review) — get_attendance_grid N+1 쿼리 패턴**: sprint-review 발견. `get_grid_impl` 학생 루프 안에 4쿼리 (day_rows, cell_rows, compute_summary 2건) — 50명 기준 200쿼리. 현재 PRD §5.7 "50명×31일 < 1초" 통과지만 데이터 누적/느린 HDD 환경에서 잠재 위험. JOIN 또는 단일 IN 쿼리로 batch 처리 검토.
- ⬜ **F5 (review) — validate_year_month 월 범위 검증**: sprint-review 발견. `attendance.rs::validate_year_month` 가 `2026-00` / `2026-13` 같은 의미론적 무효 입력 통과. `next_month_str` 단계에서 `NaiveDate::parse_from_str` 실패로 비친화적 에러 메시지 노출. 정규식/범위 검증 강화로 사용자 친화 메시지 즉시 반환.

---

## Phase 3: 보강 + 소멸 (Sprint 9~10) ✅ 완료 (2026-05-28)

### 목표
출결/보강의 가장 복잡한 도메인(보강 매칭, 소멸 자동 전이, 퇴교 처리)을 완성하여 UC-4를 달성한다.

### Sprint 9: 보강 등록 + 매칭 (2주) ✅ 완료 (2026-05-26)

> 계획 문서: `docs/sprint/sprint9.md` / Task T1~T12 완료 / 시각 검증 3라운드 18건 흡수 (I1~I8 + J1~J10)
> develop 머지: `sprint9 → develop` (--no-ff, 진행 중)

#### 주요 도메인 결정 사항

| 결정 | 내용 |
|------|------|
| PI-02 보강-결석 매칭 | 일 단위 매칭 확정 (옵션 A — 시간값 검증 없이 일 기준) |
| 보강 가능일 (I3) | 케이스 A (평일 + 보강불가 코드 없음) OR 케이스 B (`allows_makeup_class=1` 명시). `study_periods` 제약 제거 |
| 정규 수업 요일 보강 허용 | T3 검증 3 폐기 — 수업 후 추가 보강 진행 가능 |
| 시간 표시 단위 | UI 입력/표시는 시간(h) 단위, 백엔드 `class_minutes`(분) 유지 |
| 결석 셀 라벨 통일 | `absent`/`makeup_done` 모두 '결석' 표기, `makeup_done` 배경은 emerald |
| 보강 삭제 진입점 | 보강일(emerald) 셀 클릭 (기존 결석 셀에서 이동 — J6) |
| 보강 미등원 폐기 | 사용자 결정 — `markMakeupAbsent` UI 호출 제거 (J5) |
| 보강데이 일괄 기능 폐기 | `BatchMakeupDialog` 삭제 (J7) |

#### 작업 목록

- ✅ **T1**: PI-02 결정 반영 + 보강 도메인 설계 검토 — V108 신규 마이그레이션 불필요 확정
- ✅ **T2**: 보강 IPC 백엔드 — `get_pending_absences` + `get_makeup_eligible_dates` + `validate_year_month` 강화 (A43) + 단위 테스트 9건
- ✅ **T3**: 보강 등록 + 매칭 트랜잭션 IPC — `create_makeup_with_absences` (BEGIN IMMEDIATE, 검증 5종) + `MakeupCreated` audit variant + 단위 테스트 9건
- ✅ **T4**: 보강 취소 + 미등원 + 일괄 IPC — `cancel_makeup` + `mark_makeup_absent` + `batch_create_makeups` + 단위 테스트 7건
- ✅ **T5**: TypeScript IPC 래퍼 7종 + `src/types/makeup.ts` 도메인 타입 8종
- ✅ **T6**: 보강 등록(개별) UI — `MakeupRegisterDialog` + `AttendanceGrid` 비수업일 셀 클릭 + TanStack Query 무효화
- ✅ **T7**: 보강데이 일괄 UI + `MakeupManageDialog` + (A41) 헤더 라벨 "미처리\\n결석" 변경
- ✅ **T8**: 결석 이력 조회 — `get_absence_history` IPC + `AbsenceHistoryDialog` (3종 상태 시각 구분)
- ✅ **T9**: 통합 검증 — 자동 7항목 전수 통과 + 마이그레이션 self-check (A39) + sprint-review 산출물 경로 명시 (A40)
- ✅ **T10**: 보강 가능일 정의 확장 + T3 검증 3 폐기 (I3 시각 검증 반영)
- ✅ **T11**: 프론트엔드 시간 단위 + UX 보강 — `src/lib/time.ts` 신규 + I1/I2/I4~I8 흡수
- ✅ **T12**: 2/3차 시각 검증 J1~J10 흡수 — 보강일 emerald 셀 + 미등원 폐기 + 일괄 기능 폐기 + 양방향 tooltip

#### 완료 기준 (Definition of Done)
- ✅ 보강 등록(개별) → 결석 "보강완료" 전이 동작 (AC-4.5-3)
- ✅ 보강 취소 → 결석 환원 정합성 유지 (AC-4.5-4)
- ✅ "보강 진행 가능 OFF" 일자 보강 등록 차단 (AC-4.4-3)
- ✅ 결석 이력에서 보강완료/보강소멸 시각 구분 (AC-4.5-7)
- ✅ 보강 비즈니스 규칙 단위 테스트 신규 28건 (T2 9 + T3 9 + T4 7 + T8 3)
- ✅ `cargo test --lib` cipher off 254 passed / cipher on 133 passed
- ✅ `cargo clippy --lib -- -D warnings` cipher off/on clean
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- ✅ 마이그레이션 self-check (A39): V108 불필요 결정과 실제 migrations 1:1 일치
- ✅ 사용자 시각 검증 7라운드 "검증완료" (2026-05-26)

#### 기술 고려사항
- 보강-결석 매칭은 트랜잭션 내에서 원자적 실행 (정합성 필수)
- PI-02 확정: 일 단위 매칭 (분 단위 전환은 T3 검증 3 활성화만으로 가능 — R58)

---

### Sprint 10: 소멸 자동 전이 + 캘린더 뷰 완성 (2주) ✅ 완료 (2026-05-28)

> 계획 문서: `docs/sprint/sprint10.md` / Task T1~T12 완료 (T5 환원 IPC 폐기 — 사용자 정책) / 7라운드 시각 검증
> develop 머지: `sprint10 → develop` (--no-ff, 예정)

#### 주요 도메인 결정 사항

| 결정 | 내용 |
|------|------|
| T5 환원 IPC 폐기 | 사용자 정책 — "보강기한 소멸되면 끝", 환원 기능 불필요 |
| PI-03 캘린더 라이브러리 | FullCalendar (MIT) 채택 — ADR-006 작성 완료 |
| PI-04 보강데이 일괄 | 캘린더 보강관리 뷰에서 진입점 제공, 구체적 UI는 Phase 4+ 결정 |
| 선행 수업 (§4.2.3) | 기존 토글+보강 흐름 활용 (PI-08 결정), 별도 IPC 불필요 |
| V108 FK 카운터 함정 | TEMP 테이블 패턴으로 자식 FK 보존/복원 — 실데이터 code 787 해소 |

#### 작업 목록

- ✅ **T1**: Sprint 9 dead code 정리 — `mark_makeup_absent` / `batch_create_makeups` / `MakeupAbsent` variant 완전 제거 (`dde74aa`)
- ✅ **T2**: 소멸 자동 전이 설계 + 사용자 확인 — PI-05~PI-09 결정 (트리거 3개소, 오늘 기준, V108, 토스트 알림)
- ✅ **T1'**: V108 마이그레이션 — `makeup_attendances.status` CHECK 단순화 (FK 카운터 함정 TEMP 패턴 적용)
- ✅ **T3**: 소멸 자동 전이 IPC — `expiration.rs` 신규 모듈 + `expire_overdue_absences` + 단위 테스트 7건
- ✅ **T4**: 소멸 전이 트리거 통합 — 앱 시작 / 출결 생성 / 교습기간 등록 3개 트리거 연결
- ❌ **T5**: 보강소멸 → 결석 수동 환원 IPC — 사용자 정책으로 폐기 (2026-05-26)
- ✅ **T6**: 퇴교 시 미사용 보강 처리 IPC — `get_pending_makeup_for_withdrawal` + `process_withdrawal_makeup` + 단위 테스트 6건
- ✅ **T7**: 선행 수업 검증 — 기존 흐름으로 미래 결석 + 보강 매칭 단위 테스트 확인
- ✅ **T8**: 캘린더 라이브러리 ADR (ADR-006: FullCalendar) + 집계 IPC (`calendar.rs` 신규) + 단위 테스트 5건
- ✅ **T9**: 소멸 알림 UI — 앱 시작 시 토스트 (건수 > 0일 때만)
- ✅ **T10**: 퇴교 보강 처리 UI — `WithdrawalMakeupDialog` + 원생 관리 퇴교 흐름 통합
- ✅ **T11**: 캘린더 뷰 UI — FullCalendar 일/주/월 + 원생 상세 팝업 + 보강 관리 뷰. 수업 관리 메뉴 활성화. 7라운드 시각 검증 완료
- ✅ **T12**: 통합 검증 — 자동 7항목 전수 통과 + 마이그레이션 self-check (A39) 1:1 일치

#### 완료 기준 (Definition of Done)
- ✅ 소멸 자동 전이가 앱 시작 / 출결 생성 / 교습기간 등록 3개 트리거에서 정상 발동
- ✅ 퇴교 처리 다이얼로그 3개 선택지 모두 동작
- ✅ 캘린더 뷰 일/주/월 전환 + 원생 팝업 동작 (7라운드 시각 검증 완료)
- ✅ `cargo test --lib` cipher off 272 passed + cipher on 116 passed (Strawberry Perl 설치 완료)
- ✅ `cargo clippy --lib -- -D warnings` cipher off/on clean
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` (static export 16/16) 통과
- ✅ 마이그레이션 self-check (A39): V108 1:1 일치 (FK 카운터 함정 TEMP 패턴 포함)

---

## Phase 4: 청구 + 수납 + 공지문 (Sprint 11~12) 🔄 진행 중

### 목표
교습비 청구/수납 전체 흐름(UC-5)과 카카오톡 공지문 이미지 생성을 완성한다.

### Sprint 11: 청구 + 수납 관리 (2주) 🔄 진행 중

#### 작업 목록

- ⬜ **DB 마이그레이션 V007**: bills + payments 테이블
- ⬜ **청구 데이터 생성 (§4.9.1)**: 재원 원생 일괄 생성
  - 청구액 = 표준 교습비 매핑 (주 수업시간)
  - 중복 생성 차단
  - IPC 커맨드: `generate_bills`, `list_bills`, `update_bill`
- ⬜ **월 중 입퇴교 처리 (§4.9.2)**: 시각 구분 표시 + 수동 조정
- ⬜ **청구 3단계 마감 (§4.9.7)**: 미확정 → 확정 → 마감
  - 확정 후 수정 시 확인 다이얼로그 (AC-4.9-3)
  - 마감 액션: 모든 청구 "확정" 시에만 활성화 (AC-4.9-7)
  - 마감 후 수정 시 사유 입력 필수 (AC-4.9-8)
- ⬜ **청구 화면 정렬 (§4.9.4)**: 미확정 + 월 중 입퇴교 상단 우선
- ⬜ **수납 관리 (§4.9.5)**: 입금 여부/일/입금자명/결제수단/카드사
  - 카드 계열 시 카드사 필수 (AC-4.9-4)
- ⬜ **입금 일괄 처리 모드 (§4.9.6)**: 미입금 리스트 + 행별 빠른 입력
  - 최소 20행 표시 (AC-4.9-6)
  - 미납 원생 자동 추출
- ⬜ **청구/수납 비즈니스 규칙 테스트**: 매핑, 상태 전이, 마감 제약

#### 완료 기준 (Definition of Done)
- ⬜ 청구 생성 → 확정 → 마감 전체 흐름 동작
- ⬜ 수납 입력 + 입금 일괄 처리 동작
- ⬜ 마감 후 수정 시 사유 입력 강제 확인
- ⬜ 청구 50명 생성 3초 이내

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420/billing
2. browser_click → "청구 데이터 생성" 버튼
3. browser_snapshot → 청구 목록 생성 확인 (미확정 상태)
4. browser_click → 개별 청구 "확정" 처리
5. browser_click → "당월 청구 마감" 버튼
6. browser_snapshot → 마감 상태 확인
7. browser_click → 마감 후 금액 수정 시도
8. browser_snapshot → 사유 입력 다이얼로그 확인
9. browser_console_messages(level: "error") → 콘솔 에러 없음
```

#### 기술 고려사항
- 청구 상태 전이는 백엔드에서 엄격하게 제어 (프론트 우회 방지)
- 입금 일괄 처리: 낙관적 업데이트 + 실패 시 롤백

---

### Sprint 12: 공지문 이미지 생성 (2주) 📋 예정

#### 작업 목록

- ⬜ **공지문 편집 화면 (§4.10.1)**: 좌(원생 리스트) + 우(배경서식 + 텍스트박스)
  - 텍스트박스 3종: 청구월/원생이름/청구액
  - 드래그 위치 조정, 크기 조절, 폰트 속성 변경
  - 위치/속성 저장 + 자동 로드
- ⬜ **일괄 이미지 생성 (§4.10.2)**: HTML5 Canvas + `html-to-image` 라이브러리
  - 저장 경로: `[설정폴더]/[YYYYMM]/[YYYYMM]_[원생이름].png`
  - 천단위 콤마 표기 (AC-4.10-1)
  - 재생성 시 덮어쓰기 확인 (AC-4.10-2)
  - 50장 30초 이내 (성능 요구사항)
- ⬜ **배경서식 관리**: 이미지 업로드/선택/미리보기
  - 저장 위치: `smarthb/assets/` (클라우드 동기화 폴더)
- ⬜ **데이터 가져오기 기초 (§4.13.1)**: CSV/Excel 원생 명단 가져오기
  - 표준 템플릿 파일 다운로드
  - 검증 + 미리보기 + 확정 흐름

#### 완료 기준 (Definition of Done)
- ⬜ 배경서식 위에 텍스트박스 배치 → 원생별 PNG 일괄 생성 동작
- ⬜ 50장 생성 30초 이내
- ⬜ 레이아웃 설정 저장/복원 동작
- ⬜ CSV 원생 가져오기 전체 흐름 동작

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420/notice
2. browser_snapshot → 공지문 편집 화면 (좌: 원생 리스트, 우: 서식 미리보기)
3. browser_click → 배경서식 선택
4. browser_click → 텍스트박스 드래그/크기 조절
5. browser_click → "발송용 공지문 생성" 버튼
6. browser_snapshot → 생성 완료 메시지 확인
7. browser_console_messages(level: "error") → 콘솔 에러 없음
```

#### 기술 고려사항
- `html-to-image` 라이브러리: DOM → Canvas → PNG 변환
- 대량 생성 시 Web Worker 또는 순차 처리로 UI 블로킹 방지
- Tauri 파일 시스템 접근 권한 (`capabilities/` 설정)

---

## Phase 5: 단원평가 + 학습보고서 (Sprint 13~14) 📋 예정

### 목표
단원평가 점수 관리와 분기 학습보고서 작성/출력(UC-6)을 완성한다.

### Sprint 13: 단원평가 점수 입력 (2주) 📋 예정

#### 작업 목록

- ⬜ **DB 마이그레이션 V006**: assessment_events + assessment_scores + learning_reports 테이블
- ⬜ **단원평가 회차 등록 (§4.7.1)**: 회차명, 응시 기간, 학년별 단원 설명
  - 월 2회 기본, 자유 추가/수정/삭제
  - IPC 커맨드: `create_assessment_event`, `list_assessment_events`
- ⬜ **원생별 점수 입력 (§4.7.2)**: 1차/2차 점수 (0~100)
  - 1차 미입력 시 2차 차단 (AC-4.7-2)
  - 응시일 입력 (기간 외 시 "기간 외 응시" 표시)
  - 미응시 원생 식별
  - IPC 커맨드: `save_score`, `get_scores_by_event`
- ⬜ **학습 성과 추이 조회 (§4.7.3)**: 학년별/원생별 라인 차트
  - 회차별 1차/2차 점수 + 분기 평균 라인
- ⬜ **분기 계산 헬퍼**: 학사력 기준 분기 산출 함수
  - 3~5월(1Q) / 6~8월(2Q) / 9~11월(3Q) / 12~2월(4Q)

#### 완료 기준 (Definition of Done)
- ⬜ 단원평가 회차 등록 → 점수 입력 → 추이 조회 전체 흐름
- ⬜ 1차 미입력 시 2차 차단 동작
- ⬜ 학습 성과 추이 차트 렌더링

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420/assessment
2. browser_click → "새 단원평가" 등록
3. browser_snapshot → 회차 등록 폼 확인
4. browser_click → 원생별 1차 점수 입력 → 저장
5. browser_snapshot → 점수 저장 확인
6. browser_click → "성적 추이" 탭
7. browser_snapshot → 라인 차트 렌더링 확인
8. browser_console_messages(level: "error") → 콘솔 에러 없음
```

#### 기술 고려사항
- 차트 라이브러리: Recharts 또는 Chart.js (가벼운 쪽 선택)
- 분기 계산 헬퍼는 Rust 백엔드에 구현 (프론트에서도 동일 로직 필요 시 TypeScript로 미러링)

---

### Sprint 14: 분기 학습보고서 (2주) 📋 예정

#### 작업 목록

- ⬜ **보고서 메뉴 진입 (§4.8.1)**: 분기 선택, 작성 가능 분기 판정
  - 분기 마지막 월 2차 점수 입력 완료 시에만 활성화 (AC-4.8-6)
  - 대상 원생 리스트 표시
- ⬜ **보고서 입력 화면 (§4.8.2)**
  - (가) 참고 정보: 분기 6회 점수 + 당해 연도 과거 분기 점수 + 추이 차트
  - (나) 입력: 종합의견 (멀티라인)
  - 6회 미만 시행 시 실제 회차만 동적 표시 (AC-4.8-7)
  - 점수 종속: 저장 시 복사 보관 안 함, 조회 시 동적 참조 (AC-4.8-5)
- ⬜ **보고서 저장 (§4.8.3)**: (분기, 원생) UNIQUE
  - IPC 커맨드: `save_learning_report`, `get_learning_report`, `get_quarter_scores`
- ⬜ **보고서 출력 (§4.8.4)**: A4 1장 4분할 박스
  - 상단: 원생 성명 + 분기 점수표
  - 중단: 점수 추이 그래프
  - 하단: 종합의견 (줄바꿈 반영)
  - 인쇄 직접 출력 + PDF 저장
- ⬜ **데이터 내보내기 보강 (§4.13.2)**: 학습보고서 PDF 분기별 일괄
  - 비밀번호 보호 옵션 (AC-4.13-4)

#### 완료 기준 (Definition of Done)
- ⬜ 분기 보고서 작성 → 저장 → 인쇄/PDF 전체 흐름
- ⬜ A4 4분할 레이아웃 균등 + 줄바꿈 반영
- ⬜ 점수 수정 시 보고서 표시 자동 반영 확인
- ⬜ 6회 미만 시행 시 동적 표시 확인

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420/report
2. browser_click → 분기 선택 → 원생 선택
3. browser_snapshot → 참고 정보(점수표+차트) + 종합의견 입력 화면 확인
4. browser_click → 종합의견 입력 → "저장"
5. browser_snapshot → 저장 완료 확인
6. browser_click → "인쇄 미리보기"
7. browser_snapshot → A4 4분할 레이아웃 확인
8. browser_console_messages(level: "error") → 콘솔 에러 없음
```

#### 기술 고려사항
- A4 4분할 인쇄: CSS `@media print` + 고정 비율 박스
- 차트 이미지를 인쇄용 Canvas로 변환하는 로직 필요
- PDF 생성: `@react-pdf/renderer` 또는 브라우저 print-to-PDF (ADR 검토)

---

## Phase 6: 대시보드 + 유틸리티 (Sprint 15) 📋 예정

### 목표
모든 도메인 데이터를 대시보드에 집계하고, 자가 진단/가져오기-내보내기 등 유틸리티를 완성한다.

### Sprint 15: 대시보드 + 자가 진단 + 내보내기 (2주) 📋 예정

#### 작업 목록

- ⬜ **대시보드 (§4.11)**: 5개 위젯 + 알림
  - 교습소 현황 위젯 (§4.11.1): 재원 총원, 분기별 입퇴교, 학년/성별/학교별
  - 당일 수업 정보 (§4.11.2): 시간대별 수업 인원/명단
  - 해당 월 핵심 요약 (§4.11.3): 출결 진행률, 청구/입금/미납, 신규 입퇴교
  - 업무 리마인더/알림 (§4.11.4): 5종 알림 + 클릭 시 1클릭 이동
  - 출결 입력 진행률 상세 (§4.11.5): 미입력 일자 클릭 → 해당 출결 화면
  - 메모 위젯 (§4.11.6): 포스트잇 스타일
  - 모든 위젯 5초 이내 로드 (AC-4.11-1)
- ⬜ **데이터 자가 진단 (§6.6)**
  - DB 마이그레이션 V009: diagnosis_history 테이블
  - 검사 항목 7종 구현 (보강필요시간 음수, 출결/청구 미생성, 스케줄 불일치, 소멸기한 누락, 고아 데이터, 수납 정합성)
  - 자동: 매월 1일 첫 실행 시 수행
  - 수동: 설정 메뉴 → "데이터 자가 진단" 버튼
  - 진단 이력 12개월 보관
  - 이상 항목 → 대시보드 알림 + 해결 가이드
- ⬜ **데이터 내보내기 완성 (§4.13.2)**: 원생/출결/청구-수납 CSV/Excel
  - 기간 선택 + OS 표준 저장 다이얼로그
  - 비밀번호 보호 옵션
- ⬜ **복원 리허설 모드 (§5.4)**: 설정 → 백업 관리 → 리허설 버튼
  - 임시 환경 복원 + 무결성 검증 + 결과 표시 + 자동 삭제

#### 완료 기준 (Definition of Done)
- ⬜ 대시보드 5개 위젯 + 5종 알림 모두 동작
- ⬜ 자가 진단 7종 검사 + 결과 표시 + 대시보드 연동
- ⬜ 내보내기 CSV/Excel/PDF 동작
- ⬜ 복원 리허설 격리 환경 검증

#### 🧪 Playwright MCP 검증 시나리오
```
1. browser_navigate → http://localhost:1420 (대시보드)
2. browser_snapshot → 5개 위젯 렌더링 확인
3. browser_snapshot → 알림 영역 확인 (출결 미입력 등)
4. browser_click → 알림 클릭 → 해당 화면 이동 확인
5. browser_navigate → http://localhost:1420/settings
6. browser_click → "데이터 자가 진단" 버튼
7. browser_snapshot → 진단 결과 화면 확인
8. browser_console_messages(level: "error") → 콘솔 에러 없음
```

#### 기술 고려사항
- 대시보드 집계 쿼리: 복잡 쿼리는 백엔드 View 또는 전용 IPC로 분리
- 내보내기 Excel: `xlsx` 또는 `exceljs` 라이브러리 (프론트에서 생성)
- 복원 리허설: 임시 디렉토리에 DB 복사 → PRAGMA 검증 → 삭제

---

## Phase 7: 안정화 + UAT (Sprint 16~17) 📋 예정

### 목표
양 OS 빌드 검증, 성능 최적화, 접근성 감사를 완료하고, 원장 2주 UAT를 통해 v1.0을 릴리즈한다.

### Sprint 16: 양 OS 빌드 검증 + 최적화 (2주) 📋 예정

#### 작업 목록

- ⬜ **양 OS 빌드 산출물 검증**
  - Windows: `.msi` + `.exe` 인스톨러 → 설치/실행/언인스톨 테스트
  - macOS: `.dmg` → 설치/실행/삭제 테스트
  - WebView2 런타임 확인 (Windows)
  - Apple Silicon + Intel 유니버설 바이너리 확인 (macOS)
- ⬜ **성능 최적화**
  - 화면 전환 300ms 이내 확인
  - 출결표 50명 x 31일 렌더링 1초 이내
  - 청구 50명 생성 3초 이내
  - 공지문 50장 생성 30초 이내
  - 앱 시작 ~ 메인 화면 진입 3초 이내
  - 병목 지점 프로파일링 + 개선
- ⬜ **접근성 감사**
  - Pretendard 18pt 본문 / 24pt+ 헤더 / 행간 1.5 전체 화면 확인
  - WCAG AA 명도 대비 4.5:1 전체 검증
  - 44x44px 최소 클릭 영역 전체 검증
  - 키보드 단축키 7종 동작 확인
  - 저자극 톤 색상 확인
- ⬜ **E2E 테스트 (UC-1~UC-6)**: Tauri WebDriver 기반
  - UC-1: 신규 원생 등록
  - UC-2: 월말 학사 일정 수립
  - UC-3: 일일 출결 입력
  - UC-4: 결석 원생 보강 처리
  - UC-5: 월 교습비 청구 + 공지문
  - UC-6: 단원평가 점수 입력
- ⬜ **양 PC 동기화 시나리오 테스트**: Windows → 종료 → Mac 시작 → 데이터 확인
- ⬜ **기술 부채 정리**: 코드 리뷰 + 리팩토링 + 미사용 코드 제거

#### 완료 기준 (Definition of Done)
- ⬜ 양 OS 인스톨러 설치/실행/삭제 정상
- ⬜ 모든 성능 요구사항 충족
- ⬜ 접근성 기준 전체 통과
- ⬜ E2E UC-1~UC-6 모두 통과
- ⬜ 양 PC 동기화 시나리오 통과

#### 🧪 Playwright MCP 검증 시나리오
```
(이 스프린트는 Tauri WebDriver E2E로 검증 — UC-1~UC-6 전체)
1. tauri-driver로 앱 실행
2. UC-1: 원생 등록 흐름
3. UC-2: 학사 일정 수립 흐름
4. UC-3: 출결 입력 흐름
5. UC-4: 보강 처리 흐름
6. UC-5: 청구 + 공지문 흐름
7. UC-6: 단원평가 점수 입력 흐름
```

#### 기술 고려사항
- Tauri WebDriver 설정: `tauri-driver` + WebDriver 클라이언트
- 양 OS CI: GitHub Actions matrix (windows-latest, macos-latest)
- 성능 프로파일링: Rust → `flamegraph`, Frontend → Chrome DevTools

---

### Sprint 17: UAT + v1.0 릴리즈 (2주) 📋 예정

#### 작업 목록

- ⬜ **UAT 환경 준비**: 원장 실데이터 일부 마이그레이션
  - CSV 가져오기로 원생 데이터 이관
  - 교습소 PC(Windows) + 자택 Mac에 설치
- ⬜ **UAT 실행 (2주)**: 원장 실사용 + 피드백 수집
  - 일일 피드백 기록 (화면별 사용성, 글씨 크기, 동선)
  - 주간 리뷰 미팅 (2회)
  - 사용자 학습 기간 1주 이내 목표 확인
- ⬜ **피드백 반영**: 우선순위별 수정
  - Critical: 기능 오류 수정
  - High: UX 개선 (글씨 크기, 버튼 위치, 동선)
  - Medium: 미세 조정 (색상, 간격, 문구)
- ⬜ **CHANGELOG.md 최종 정리**: v1.0 릴리즈 노트
- ⬜ **v1.0 태그 + GitHub Release**: `v1.0.0` 태그 push → CI가 인스톨러 빌드/첨부
- ⬜ **배포 후 검증**: deploy-prod agent CV 체크리스트

#### 완료 기준 (Definition of Done)
- ⬜ 원장 UAT 2주 완료 + 주요 피드백 반영
- ⬜ 사용자 만족도 4.5/5 이상 (자가 평가)
- ⬜ 사용자 학습 기간 1주 이내 확인
- ⬜ v1.0.0 GitHub Release 생성 + 양 OS 인스톨러 첨부
- ⬜ 배포 후 CV 체크리스트 통과

#### 🧪 Playwright MCP 검증 시나리오
```
(UAT는 원장 수동 테스트 — 자동 검증은 Sprint 16에서 완료)
1. 배포 후 양 OS 인스톨러로 설치 확인
2. 앱 시작 → 잠금 해제 → 대시보드 진입 확인
3. 양 PC 전환 (Windows → Mac) 데이터 동기화 확인
```

#### 기술 고려사항
- UAT 중 심각한 이슈 발견 시 Sprint 16 연장 가능 (최대 1주)
- GitHub Release: `deploy.yml` 워크플로우 자동 실행

---

## 📈 마일스톤

| 마일스톤 | Phase | Sprint | 예상 시점 | 핵심 산출물 |
|----------|-------|--------|----------|------------|
| M1: 인프라 확립 | Phase 1 | Sprint 1 | +2주 | SQLCipher + app.lock + 4계층 백업 동작 |
| M2: 원생 관리 가능 | Phase 1 | Sprint 3 | +6주 | 원생 등록/조회 + 마법사 + 글로벌 검색 |
| M3: 학사+출결 동작 | Phase 2 | Sprint 8 | +14주 | 학사 일정 수립 + 출결 생성/관리 |
| M4: 보강 완성 | Phase 3 | Sprint 10 | +18주 | 보강 매칭 + 소멸 + 캘린더 뷰 |
| M5: 청구 완성 | Phase 4 | Sprint 12 | +22주 | 청구/수납/공지문 |
| M6: 평가+보고서 | Phase 5 | Sprint 14 | +26주 | 단원평가 + 분기 학습보고서 |
| M7: 대시보드 | Phase 6 | Sprint 15 | +28주 | 대시보드 + 자가 진단 + 내보내기 |
| M8: v1.0 릴리즈 | Phase 7 | Sprint 17 | +32주 | 양 OS 빌드 + UAT + v1.0.0 |

---

## 🔮 향후 계획 (Backlog / Post-MVP)

PRD §4.15에 명시된 Post-MVP 항목:

### 원장 결정에 의한 제외
- ⬜ 회계 관리 (수입/지출, 손익, 세무 자료) — Q10
- ⬜ 관리 서식 (출결부 출력, 학부모 보고서 PDF 등) — Q10
- ⬜ 학부모 커뮤니케이션 도구 (카톡 메시지 템플릿)

### 외부 의존성/기술 범위 확장
- ⬜ 카카오톡 알림톡 API 자동 발송 — Q2
- ⬜ 학부모용 모바일 읽기 전용 뷰 — Q4

### 비즈니스 룰 추가 결정 필요
- ⬜ 월 중 입퇴교 일할 자동 계산 — Q3
- ⬜ 퇴교 시 미사용 보강 환불 금액 자동 계산 — Q8

### 복잡도 대비 가치 재검토 필요
- ⬜ 출결 재생성 시 변경분만 반영 — Q9

### PRD 미해결 항목 (스프린트 진행 중 결정 필요)
- ✅ **PI-01** (High): 소멸 자동 전이 트리거 시점 → Sprint 10 완료 (앱 시작/출결 생성/교습기간 등록 3개 트리거)
- ✅ **PI-02** (High): 보강-결석 시간값 매칭 규칙 → Sprint 9 완료 (일 단위 매칭 확정)
- ✅ **PI-05** (Medium): 일련번호 자동 채번 규칙 → **확정**: `MAX+1` + `BEGIN IMMEDIATE` + override 허용 (2026-05-20)
- ⬜ **PI-07** (High): 복구 코드 발급/검증 Feature → Sprint 1 착수 전 결정 필요

### Sprint 3 이연 항목
- ✅ **R12: paths 동적화** — `paths::data_root()` 동적화 완료 (`82eb1b2`, 2026-05-21)
- ⬜ **R12 나머지: salt.bin 이전** — Keychain -> `{cloud}/smarthb/salt.bin` 이전. paths 동적화 완료 후 별도 sprint에서 처리.
- ⬜ **query!() 매크로 전환** — 동적 `query() + bind()` 패턴 유지 중. V101~V105 스키마 안정화 후 일괄 전환 예정. 별도 backlog 추적.

---

## 📋 품질 검증 체크리스트

- ✅ PRD의 모든 핵심 기능(§4.0~§4.14, §5.3~§5.5, §6.6)이 로드맵에 반영됨
- ✅ 각 Phase의 의존성이 올바르게 설정됨 (인프라 → 기반 도메인 → 학사/출결 → 보강 → 청구 → 평가 → 대시보드 → 안정화)
- ✅ MVP 범위가 PRD v1.0 기준으로 명확히 정의됨 (Post-MVP §4.15 제외)
- ✅ 각 태스크가 IPC 커맨드명, 마이그레이션 번호, UI 컴포넌트 수준으로 구체적
- ✅ 완료 기준이 측정 가능 (성능 수치, 테스트 통과, 양 OS 검증)
- ✅ 각 Phase에 Playwright MCP 검증 시나리오 포함
- ✅ PI-01/PI-02/PI-05/PI-07 미해결 항목의 결정 시점이 명시됨
