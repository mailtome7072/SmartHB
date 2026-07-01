# ROADMAP.md

## 개요
- **프로젝트 목표**: 50대 교습소 원장 1인을 위한 원생/출결/보강/청구/단원평가 통합 데스크톱 관리 앱 (Windows + macOS)
- **전체 예상 기간**: 약 30주 (16 스프린트, 6 Phase + Sprint 13 독립) + UAT 2주 = 총 32주 (Phase 5 취소로 2스프린트 단축)
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
| 전체 진행률 | v1.0.0 릴리즈 완료 + Post-v1.0 유지보수 진행 중 |
| 현재 Phase | **Post-v1.0 안정화** — Sprint 18 진행 중 (사용자 피드백 10건 + 캘린더 UX 개선) |
| 다음 마일스톤 | v1.0.2 패치 (Sprint 18 완료 후) |
| MVP 범위 | PRD §4.0~§4.6, §4.9~§4.14, §5.3~§5.5, §6.6 (§4.7~§4.8 단원평가+학습보고서 취소, §4.15 Post-MVP 제외) |
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

Phase 5 (단원평가+보고서) ❌ 전면 취소 (2026-05-31)

Sprint 13: PIN 인증 옵션화 + Phase 5 취소 반영 + carry-over  ← Phase 4 완료 필수

Phase 5 (대시보드+유틸, 구 Phase 6)  ← Phase 3~4 + Sprint 13 완료 필수
  └── Sprint 14: 대시보드 5개 위젯 + 알림 + 가져오기/내보내기 + 자가 진단

Phase 6 (안정화+UAT, 구 Phase 7)  ← Phase 5 완료 필수
  ├── Sprint 15: 양 OS 빌드 검증 + 성능 최적화 + 접근성 감사
  └── Sprint 16: UAT(원장 2주 파일럿) + 피드백 반영 + v1.0 릴리즈
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

## Phase 4: 청구 + 수납 + 공지문 (Sprint 11~12) ✅ 완료 (2026-06-02)

### 목표
교습비 청구/수납 전체 흐름(UC-5)과 카카오톡 공지문 이미지 생성을 완성한다.

### Sprint 11: 청구 + 수납 관리 (2주) ✅ 완료 (2026-05-29)

> 계획 문서: `docs/sprint/sprint11.md` / Task T0~T9 완료 / Phase 3 carry-over 7건 흡수
> develop 머지: `sprint11 → develop` 직접 머지 예정 (단일 개발자 정책)

#### 주요 도메인 결정 사항

| 결정 | 내용 |
|------|------|
| PI-10 마감 후 수정 사유 UX | 모달 다이얼로그 — 의도적 행위 강조, 실수 방지 |
| PI-11 마감 해제(reopen) | 불가 — PRD 미언급, 개별 건 수정(사유 필수)만 허용 |
| PI-12 수납 테이블 분리 | 별도 payments 테이블 — 분할 납부/환불 확장 여지, 감사 로그 분리 |
| 카드 계열 식별 기준 | `is_card_type` BOOLEAN 플래그 (V109에 포함) |
| F4 N+1 쿼리 범위 | `calendar.rs` N+1만 — `attendance.rs`는 PRD 성능 충족, carry-over 유지 |

#### 작업 목록

- ✅ **T0**: Phase 3 carry-over 7건 정리 (F1 panic 해소 / F2 fail-soft / F3 미사용 파라미터 / F4 N+1 batch / F5 viewType 프레임 / F6 flaky #[ignore] / F7 보강관리 메뉴 제거)
- ✅ **T1**: DB 마이그레이션 V109 — bills + payments + payment_methods.is_card_type
- ✅ **T2**: 청구 생성 IPC 4종 (`generate_bills`, `list_bills`, `get_bill`, `update_bill`, `default_year_month`) + 단위 테스트 17건
- ✅ **T3**: 청구 상태 머신 IPC 3종 (`confirm_bill`, `confirm_all_bills`, `close_billing_month`, `update_closed_bill`) + audit 3 variants + 단위 테스트 9건
- ✅ **T4**: 수납 IPC 5종 (`create_payment`, `update_payment`, `list_unpaid_bills`, `batch_update_payments`, `get_billing_summary`) + 단위 테스트 9건
- ✅ **T5**: 청구 마감 UX 다이얼로그 3종 (`CloseReasonDialog` / `ConfirmBillUpdateDialog` / `CloseMonth`)
- ✅ **T6**: TypeScript IPC 래퍼 13종 + `src/types/billing.ts` 도메인 타입
- ✅ **T7**: 청구 관리 UI — `/billing` 라우트 + `BillingGrid` + 사이드 메뉴 활성화
- ✅ **T8**: 수납 관리 UI — `[청구|수납]` 탭 통합 + `PaymentsView` (입금 일괄 처리)
- ✅ **T9**: 통합 검증 + AC 전수 마킹 (sprint11.md DoD/AC)

#### 완료 기준 (Definition of Done)
- ✅ 청구 생성 → 확정 → 마감 전체 흐름 동작
- ✅ 수납 입력 + 입금 일괄 처리 동작
- ✅ 마감 후 수정 시 사유 입력 강제 확인 (AC-4.9-8)
- ✅ 마감 전제조건: 모든 청구 confirmed 시에만 마감 활성화 (AC-4.9-7)
- ✅ 카드 계열 시 카드사 필수 (AC-4.9-4)
- ✅ 입금 일괄 처리 한 화면 최소 20행 (AC-4.9-6)
- ✅ Phase 3 carry-over F1~F7 전수 해소
- ✅ `cargo test --lib` 308 passed / `cargo clippy -- -D warnings` clean
- ✅ `pnpm lint` / `pnpm tsc --noEmit` / `pnpm build` static export 17 routes 통과
- ⚠️ 청구 50명 생성 3초 이내 — 인메모리 단위 테스트 0.21s/35 tests. 실측은 사용자 시각 검증으로 이연 (sprint-review risk-register 기록)

#### post-Sprint 11 develop 보완 (2026-05-30)

> 정식 sprint 브랜치 없이 develop 에 직접 커밋 (단일 개발자 PR 생략 정책).
> 검수 후 발견된 8건 UX/보안 보강 + PIN 인증 전환(ADR-007) 2커밋.

커밋 `945e4a7` — 청구/수납 검수 후속 보완 8건:
- 청구 탭 상태 필터 '마감' 추가 + 옵션별 건수 표기 / '마감 완료' 배지 위치 이동
- 수납 탭 필터 건수 표기 / 마감 후 수정 사유 게이트 완화 / 입금일 달력 닫힘 + 포커스 이동
- 수납완료 행 수납 취소 기능 (`batch_update_payments` 재사용)
- 입금 시 결제수단 필수 검증 (백엔드 2곳 + 프론트 가드)
- 수납완료된 마감 청구 수정 불가 (`update_bill_impl` 거부 + 프론트 비활성)
- 신규 단위 테스트 3건: `create_payment_rejects_paid_without_method`, `batch_cancel_payment_resets_is_paid`, `update_bill_closed_paid_rejected`
- 자동 검증: cargo test 315건 통과 / clippy clean / pnpm lint clean / tsc clean

커밋 `c93399e` — 앱 잠금 인증 6자리 숫자 PIN 전환 (ADR-007):
- `LockScreen` / `RecoveryCodeInput` 입력을 6자리 숫자 PIN 으로 전환
- 백엔드 `validate_pin` (len 6 + ascii digit) — `set_password` / `reset_password_with_code` 재검증
- dev autologin + `.env.example` 6자리 PIN 대응
- ADR-007 신규 작성 (`docs/arch/adr-007-pin-authentication.md`) — 보안 트레이드오프 명시, 복구코드 12자리 유지

커밋 `70c59a1` — 청구 관리 '월별 집계' 탭 추가:
- 3번째 탭 '월별 집계' 신설. 년/월 토글(연도 `YYYY-%` 집계 / 월 집계), 요약 박스 + 결제수단별 수납총액(열 배치)
- 백엔드 `get_billing_period_stats(period)` IPC + `BillingPeriodStats`/`PaymentMethodSummary` 타입 신규
- 신규 단위 테스트: `billing_period_stats_groups_by_method`

커밋 `c1ae063` — 청구 '마감(closed)' 개념 전면 폐기 (원장 결정):
- 청구 상태 2단계(미확정→확정)로 축소. V111 마이그레이션 — `bills` 재구성 (`status` CHECK(draft/confirmed), `close_reason`/`closed_at` 제거, closed→confirmed 흡수, payments CASCADE 백업/복원)
- 제거: `close_billing_month` IPC, `CloseMonthDialog`/`CloseReasonDialog`, 마감 버튼/배지/필터, audit `BillMonthClosed`/`BillClosedModified`, `update_bill` `close_reason` 파라미터
- 수납완료(is_paid) 청구 금액 수정 거부로 전환. PRD §4.9.7 폐기 반영, AC-4.9-9 신설
- 신규 단위 테스트: `update_bill_paid_rejected`
- 자동 검증: cargo test 326건 통과 / clippy clean / pnpm lint + tsc clean / V111 실DB 시각검증(closed→confirmed 변환 + payment 보존 + FK 0위반)

커밋 `2a964b0` — 월별 집계 기간 선택을 청구 생성된 년월로 한정:
- `list_billed_months` IPC 신규 (`bills` distinct `bill_year_month` DESC) — 청구가 없는 년월은 드롭다운에 표시하지 않음
- 신규 단위 테스트: `list_billed_months_returns_distinct_desc`

커밋 `29fbe93` — 월별 집계 빈 데이터 시 디폴트 년월 현재 년월로:
- 청구 데이터 0건일 때 현재 년월을 디폴트로 사용하여 "0건" 상태 정상 노출 (빈 화면 방지)

> post-Sprint 11 4커밋 자동 검증 (2026-05-30): cargo test 326건 통과 / clippy -D warnings clean / pnpm lint clean / pnpm tsc clean

---

### Sprint 12: 공지문 이미지 생성 (2주) ✅ 완료 (2026-06-02)

> 계획 문서: `docs/sprint/sprint12.md` / Task T0~T9 10개 Task 완료 + 사용자 검증 완료
> Phase 4 마지막 마일스톤. CSV 가져오기는 Sprint 15로 이연.
> 신규 의존성: `html-to-image` ^1.11.13, `react-rnd` ^10.x (PI-14 확정)
> develop 머지: `sprint12 → develop` 직접 머지 예정 (단일 개발자 정책)

#### 주요 도메인 결정 사항

| 결정 | 내용 |
|------|------|
| PI-13 이미지 생성 | html-to-image (frontend.md 명시) 시도 후 macOS WKWebView foreignObject+img 결함으로 Canvas 2D 직접 렌더로 전환 |
| PI-14 드래그 라이브러리 | react-rnd 확정 (사용자 결정 2026-05-30) |
| 저장 경로 개편 | `output/{공지문이름}/{YYMM}/{이름}_{YYMM}_{원생}.png` (공백 제거, 한글 NFC 정규화) |
| 복구 코드 시스템 | 사용자 결정으로 전면 제거 — cipher OFF 환경에서 불필요 |
| PIN UI 통일 | 6박스(OTP) 공용 컴포넌트(`components/ui/pin-field.tsx`) — LockScreen + 설정 PIN 변경 동일 UI |

#### 작업 목록

- ✅ **T0: Sprint 11 carry-over 정리** — A70/A73/A82/A85/A86 해소. A71 수동 검증 완료
- ✅ **T1: 백엔드 경로 헬퍼** — `paths.rs` `assets_dir()`, `notice_output_dir()` 추가 + 단위 테스트
- ✅ **T2: 배경서식 관리 IPC** — `notice.rs` 신규 (`list_notice_assets`, `save_notice_asset`, `delete_notice_asset`) + 단위 테스트
- ✅ **T3: 레이아웃 설정 IPC** — `save_notice_layout`, `get_notice_layout` (`app_settings` JSON 저장) + 단위 테스트
- ✅ **T4: 이미지 저장 IPC** — `save_notice_image`, `save_notice_images_batch`, `check_notice_output_exists`, `open_notice_output_folder` + 단위 테스트
- ✅ **T5: TypeScript IPC 래퍼** — 8종 + `src/types/notice.ts` 도메인 타입
- ✅ **T6: 공지문 편집 화면 UI** — `/notices` 라우트, 좌(원생 리스트+체크박스) + 우(배경서식+텍스트박스 오버레이), react-rnd 드래그+리사이즈, 다중 선택(Shift+클릭), 방향키 미세 이동, 빈 영역 클릭 선택 해제, 글자별 폰트색(charColors), 미저장 변경 전역 네비게이션 가드, 공지문 저장/닫기/초기화 흐름, 사이드바 메뉴 활성화, 저장 경로 클릭 시 폴더 열기
- ✅ **T7: 일괄 이미지 생성 엔진** — `notice-generator.ts` Canvas 2D 직접 렌더 (macOS WKWebView foreignObject 결함 회피), 천단위 콤마(AC-4.10-1), 재생성 확인 다이얼로그(AC-4.10-2), 진행률 표시, 미리보기 팝업 + 파일 저장 다이얼로그
- ✅ **T8: Tauri capabilities** — `fs:allow-write-text-file`, `fs:allow-read-dir`, `fs:allow-open-url` 최소 권한 추가
- ✅ **T9: 통합 검증** — 자동 검증 전수 통과. 사용자 수동 검증(공지문 기능 + PIN UI) 완료
- ✅ **scope 외: PIN UI 통일** — `components/ui/pin-field.tsx` 6박스 OTP 공용 컴포넌트 + LockScreen + `/settings/pin` 통일
- ✅ **scope 외: 복구 코드 시스템 제거** — `commands/recovery.rs` 삭제, argon2 의존성 제거, 관련 UI 3종 제거
- ✅ **scope 외: 메뉴 정비** — '학사 스케줄' → '학사 관리' → '일정 관리' 변경, 출결관리/학사 순서 swap

#### 완료 기준 (Definition of Done)
- ✅ 배경서식 업로드 → 선택 → 미리보기 동작
- ✅ 텍스트박스 3종 드래그 + 크기 조절 + 폰트 속성 변경 동작
- ✅ 위치/속성 저장 → 재진입 시 자동 로드 (AC-4.10-3)
- ✅ 원생별 PNG 일괄 생성 → 파일 시스템 저장 동작
- ✅ 저장 경로: `{data_root}/output/{공지문이름}/{YYMM}/{이름}_{YYMM}_{원생}.png`
- ✅ 청구액 천단위 콤마 표기 (AC-4.10-1)
- ✅ 동일 월 재생성 시 덮어쓰기 확인 다이얼로그 (AC-4.10-2)
- ✅ cargo test --lib 전수 통과 / cargo clippy clean / pnpm lint + tsc + build 통과
- ✅ 사용자 수동 검증 완료 (2026-06-01 ~ 2026-06-02)

#### 기술 고려사항
- html-to-image foreignObject+img macOS WKWebView 결함으로 Canvas 2D 직접 렌더 채택
- 저장 경로 공백 제거 + 한글 NFC 정규화 (파일 시스템 호환성)
- Tauri `shell:allow-open` 재사용으로 폴더 열기 구현 (fs 플러그인 미추가)

---

## Phase 5: 단원평가 + 학습보고서 ~~(Sprint 13~14)~~ ❌ 전면 취소 (2026-05-31 원장 결정)

> **취소 사유**: 운영 상 불필요하다고 원장이 판단. 단원평가(`/exams`) + 학습보고서(`/reports`) 개발 전면 취소.
> 관련: `.claude/memory/exam-feature-cancelled.md`

~~### 목표~~
~~단원평가 점수 관리와 분기 학습보고서 작성/출력(UC-6)을 완성한다.~~

*(이하 Sprint 13~14 상세 계획 삭제 — 미구현 상태로 코드 영향 없음)*

---

## Sprint 13: PIN 인증 옵션화 + Phase 5 취소 반영 + carry-over (2주) ✅ 완료 (2026-06-02)

> 계획 문서: `docs/sprint/sprint13.md`
> Phase 5 취소로 기존 Sprint 13(단원평가)/14(학습보고서) 대체. PIN 인증 옵션화(C안: 키체인 자동 스킵) + 기술 부채 해소.
> develop 머지: `sprint13 → develop` 직접 머지 예정 (단일 개발자 정책)

#### 작업 목록

- ✅ **T0-c**: R88 — `save_notice_preview` 경로 경계 검증 (절대경로+.png+traversal 차단, data_root 밖 폴더 자동생성 금지)
- ✅ **T1**: Phase 5 전면 취소 반영 — 메뉴 항목 제거 + PRD §4.7/§4.8/§6.1 [CANCELLED] 표기
- ✅ **T2**: ADR-008 작성 — 기기별 선택적 PIN 게이트 설계 결정
- ✅ **T3**: 백엔드 config.json `skip_pin_on_launch` 플래그 get/set IPC
- ✅ **T4**: 백엔드 키체인 자동 잠금해제 IPC + startup 공통 로직 추출 (`run_startup`)
- ✅ **T5/T6**: 프론트엔드 IPC 래퍼 3종 + 설정 화면 PIN 스킵 토글 + /lock 진입 자동 잠금해제 분기/폴백
- ✅ **검수 중 추가**: 글로벌 검색 원생 클릭 404 수정 (`/students/{id}` → `/students/edit?id=`)
- ✅ **검수 중 추가**: 글로벌 검색 드롭다운 방향키 탐색 + 한글 IME 처리

> **T0-a/b/d (A87/A85/A70) carry-over**: 계획 수립 시점에 기재됐으나 이전 스프린트에서 이미 해소됨. 실제 신규 작업은 T0-c(R88)만 수행.

#### 완료 기준 (Definition of Done)
- ✅ PIN 스킵 토글 OFF: 앱 재시작 시 PIN 입력 없이 메인 진입
- ✅ PIN 스킵 토글 ON: 기존 PIN 입력 흐름 정상 (회귀 없음)
- ✅ 키체인 키 부재 시: 토글 OFF여도 PIN 입력 요구 (안전 폴백)
- ✅ Phase 5 메뉴 항목 완전 제거 확인
- ✅ cargo test 315 passed / clippy clean / cargo check --features cipher 통과 / pnpm lint + tsc + build 전수 통과
- ✅ 사용자 수동 검수 완료 (PIN 토글 OFF→자동 로그인 / ON 회귀 / Phase 5 메뉴 제거 / 검색 등)

---

## Phase 5: 대시보드 + 유틸리티 (Sprint 14) ✅ 완료 (2026-06-06)

> Phase 번호 재정렬: 기존 Phase 6 → Phase 5 (Phase 5 단원평가+학습보고서 전면 취소로 이동)

### 목표
모든 도메인 데이터를 대시보드에 집계하고, 자가 진단/가져오기-내보내기 등 유틸리티를 완성한다.

### Sprint 14: 대시보드 + 자가 진단 + 내보내기 (2주) ✅ 완료 (2026-06-06)

> 계획 문서: `docs/sprint/sprint14.md` / Task T0~T8 완료 + 검증-phase 보강 다수
> 신규 의존성: recharts 3.8.1 (대시보드 차트), rust_xlsxwriter 0.95 (엑셀 내보내기)
> 신규 마이그레이션: V303(diagnosis_history), V304(퇴교생 미보강 결석 백필), V305(students.birth_date)
> develop 머지: `sprint14 → develop` 직접 머지 예정 (단일 개발자 정책)

#### 작업 목록

- ✅ **T0 carry-over (A91/A93)**: `startup.rs` cipher-off 동작 명시 주석 + ADR-008 정정, `/lock` SplashScreen 단일 로딩 통합
- ✅ **T1/T2 데이터 자가 진단 (§6.6)**: V303 마이그레이션(diagnosis_history) + `diagnosis.rs` IPC 4종 + 검사 7종 + 단위 테스트 20건 + 프론트엔드 `/settings/diagnosis` 라우트 + AppShell 자동 트리거
  - 검사 항목: ①보강필요시간 음수 ②재원중 당월 출결 미생성 ③재원중 당월 청구 미생성 ④스케줄-출결 요일 불일치 ⑤결석 소멸기한(`makeup_deadline`) 미설정 ⑥고아 보강 데이터 ⑦수납 정합성(결제수단/카드사 누락)
  - 자동: 매월 1일 첫 실행 시(`last_auto_diagnosis` app_settings 분리), 수동 실행 모두 지원
  - 이력 중복 방지(동일 결과 재기록 스킵) + 해결 항목 자동 재검증·정리 + 완전 0건 전환
- ✅ **T3/T4 대시보드 (§4.11)**: `dashboard.rs` IPC 7종 + 알림 4종 + 위젯 UI
  - 교습소 현황 위젯 (재원 총원, 학년/성별/학교별 비율, 분기별 입퇴교 추이)
  - 당일 수업 위젯 (12시간제 am/pm + 시간대별 인원/명단)
  - 월 요약 위젯 (이전/다음 월 전환, 청구/입금/미납/당월 입퇴교)
  - 교습소 월별 청구총액 추이 그래프 (최근 12개월 라인차트)
  - 이달의 생일 위젯, 메모 3슬롯(포스트잇, 드래그 리사이즈, 1초 디바운스 자동저장)
  - 알림 4종: 보강 소멸 임박(퇴교생 제외) / 미확정 청구 / 학사 미수립 / 자가 진단 이상
  - 차트 라이브러리: recharts 3.8.1 (dynamic import ssr:false)
- ✅ **T5/T6 데이터 내보내기 (§4.13.2)**: `export.rs` IPC 3종 + 엑셀(.xlsx) 내보내기
  - rust_xlsxwriter 0.95 신규 의존성(순수 Rust, Win/Mac 안전)
  - 원생 명단(일련번호 오름차순, 생년월일 포함) / 출결(정규+보강 UNION, 구분 컬럼) / 청구-수납(청구상태 컬럼 포함)
  - 금전 컬럼 천단위 콤마 + 우측정렬, autofit 컬럼너비, 수업시간 시간 단위 통일
  - 기간: 단월/전체(Option<String>) + OS 저장 다이얼로그
- ✅ **T7 복원 리허설 (§5.4)**: `backup.rs` 확장 — `run_backup_rehearsal` IPC(임시복사→integrity_check→주요 6종 행수→사본 폐기, cipher off 평문 게이트) + `/settings/backup` 라우트
- ✅ **T8 통합 검증**: cargo test 368 passed / clippy clean / `cargo check --features cipher` clean / pnpm lint·tsc·build 전수 통과
- ✅ **검증-phase 보강**: 자가진단 음수 오탐 수정, 이력 중복 방지, 해결 항목 자동 재검증, 완전 0건 전환 / 출결 진행률 위젯·알림 제거(항상 100% 무의미) / 퇴교생 미보강 결석 V304 백필 / 이월 누적 보강필요시간 / 원생 생년월일 추가(V305 + 폼/목록/엑셀) / 대시보드 월 요약 이전/다음 월 전환 + 이달의 생일 위젯 / Node 25 dev webpack 캐시 크래시 회피

#### 완료 기준 (Definition of Done)
- ✅ 대시보드 위젯 + 알림 모두 동작 (5초 이내 로드 AC-4.11-1)
- ✅ 자가 진단 7종 검사 + 결과 표시 + 대시보드 연동
- ✅ 내보내기 엑셀(.xlsx) 3종 동작 + OS 저장 다이얼로그
- ✅ 복원 리허설 격리 환경 검증 (운영 데이터 무영향)
- ✅ `cargo test --lib` 368 passed / `cargo clippy -- -D warnings` clean
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ✅ 사용자 수동 검수 완료 (2026-06-06)

---

## Phase 6: 안정화 + UAT (Sprint 15~16) 🔄 진행 중

> Phase 번호 재정렬: 기존 Phase 7 → Phase 6 (Phase 5 취소로 이동)

### 목표
양 OS 빌드 검증, 성능 최적화, 접근성 감사를 완료하고, 원장 2주 UAT를 통해 v1.0을 릴리즈한다.

### Sprint 15: 양 OS 빌드 검증 + 최적화 + 접근성 감사 (2주) ✅ 완료 (2026-06-07)

> 계획 문서: `docs/sprint/sprint15.md` / Task T0~T6(+T5) 완료, T7~T9 Sprint 16 이연
> 신규 의존성 없음 / DB 마이그레이션 없음

#### 실제 완료 범위 (T0~T6+T5)

- ✅ **T0**: `dashboard.rs` `monthly_summary` GROUP BY 서브쿼리 리팩토링 (R99 해소) + 대시보드 위젯 타이틀 inline fontSize → Tailwind `text-2xl` 통일 (A97)
- ✅ **T1**: 교습소 정보 화면 신설 (`/settings/info`) — AcademyInfo 텍스트 9필드 + 로고/2D바코드 이미지 2종 업로드/미리보기/삭제. `get_academy_info`/`save_academy_info` IPC + `app_settings` JSON 저장 + `notice_asset` IPC 재사용. 설정 허브 카드 활성화
- ✅ **T5** (마이너 UI 개선, 사용자 시각 검증 완료):
  - 설정 허브 카드 순서 변경 (PIN 위치 조정)
  - '마법사 재실행' 카드 → 'DB 폴더 변경(예정)' 안내 카드 (disabled, Sprint 16 이연 명시)
  - 원생 상세 화면에 '원생 관리 메인으로' 버튼 추가
  - 전역 `GlobalTooltip` 컴포넌트 도입 (브라우저 `title` 속성 툴팁 20px 통일)
  - 대시보드 위젯 폰트 미세 조정
- ✅ **T2**: 자가 진단 이력 수동 삭제 — `delete_diagnosis_history(id)` / `clear_diagnosis_history()` IPC 2종 + UI 삭제 버튼 + 확인 모달, 단위 테스트 3건
- ✅ **T3**: 접근성 감사 — `text-gray-400` → `text-gray-600` 17건 수정(WCAG AA 명도 기준 통과), `GlobalShortcuts` 컴포넌트 신설(Ctrl+F / Ctrl+N 단축키 전역 등록). 감사 보고서: `docs/sprint/sprint15/accessibility-audit.md`
- ✅ **T4**: 기술 부채 정리 — `cargo clippy --all-targets` 부채 6건 해소. A89(`/notices/page.tsx` 분리)는 로직이 이미 별도 모듈로 분리된 상태 확인 → UI 구획화만 Sprint 16 이연
- ✅ **T6**: 청구 생성 `standard_fees` N+1 쿼리 제거 (IN 쿼리 단일 배치로 전환). 성능 보고서: `docs/sprint/sprint15/performance-report.md`

#### Sprint 16으로 이연 (T7~T9 + 기타)

- **T7 양 OS 빌드 검증** — 물리 환경 의존(Windows PC), Sprint 16 UAT와 통합
- **T8 양 PC 동기화 시나리오** — 물리 환경 의존(양 PC), Sprint 16 UAT와 통합
- **T9 통합검증(빌드 산출물·양OS 실검증)** — 코드 검증(cargo test 375 passed / clippy --all-targets / lint / tsc)은 이번 스프린트 통과. 빌드 산출물·양 OS 실검증만 Sprint 16
- 출결표 N+1 재설계·셀 memo·`makeup_attendances` 인덱스 (실측 후 결정)
- 공지문 I/O 병렬화
- 접근성 밀집UI 44px·gray-500·F1·Ctrl+S 미달 항목 (Medium/Low)
- A89 `/notices` UI 구획화 (로직 분리 완료, UI 3분할만 잔여)
- DB 폴더 변경(경로 재지정) — 고위험, R12 salt.bin 이전과 함께 Sprint 16 설계·ADR
- CSV 가져오기 (Sprint 16 UAT 환경 준비 첫 번째 작업)

#### 완료 기준 (Definition of Done)

- ✅ 교습소 정보 저장/조회 정상 — 텍스트 9필드 + 이미지 2종 업로드/미리보기/삭제
- ✅ 자가 진단 이력 행 단위 삭제 + 전체 비우기 정상
- ✅ monthly_summary GROUP BY 리팩토링 완료 (R99 해소)
- ✅ 접근성 Critical 수정 — WCAG AA 명도 대비 17건 수정, 전역 단축키 Ctrl+F/Ctrl+N 등록
- ✅ `cargo test --lib` 375 passed
- ✅ `cargo clippy --all-targets -- -D warnings` clean
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ⬜ macOS .dmg / Windows .msi 빌드 설치 검증 → Sprint 16
- ⬜ 양 PC 동기화 시나리오 → Sprint 16
- ⬜ E2E UC-1~UC-5 자동화 → Post-MVP backlog

#### 마이그레이션 self-check

V305 최신 유지 (Sprint 15 신규 마이그레이션 없음 — DB 변경 없음 확인).

#### 기술 고려사항
- Tauri WebDriver E2E 자동화 설정: Sprint 16 이연 (별도 인프라 세팅 8~12h, Post-MVP backlog)
- 양 OS CI: GitHub Actions matrix (windows-latest, macos-latest) → Sprint 16에서 실행

---

### Sprint 16: 수업일 변경 도메인 + 양 OS 빌드 검증 + DB 폴더 변경 + 실사용 개시 + v1.0 릴리즈 (2주) ✅ 완료 (2026-06-12)

> 사용자 이슈(2026-06-08): **수업일 변경 2종(1회성 이동 + 특정일 이후 영구 변경)을 T0 최우선**으로 추가.
> Sprint 15에서 이연된 T7(양 OS 빌드) · T8(양 PC 동기화) · T9(통합검증 빌드 부분)를 흡수.
> PI-16 확정(2026-06-08): DB 폴더 변경 + salt.bin 이전 Sprint 16 포함.
> PI-18 확정(2026-06-08): 격식 2주 UAT 폐기 → 바로 실사용 개시, 피드백은 실사용 중 수집.
> PI-20~23 확정(2026-06-08): 케이스1=출결행 이동 / 케이스2=미처리만 재생성·처리행 보존 / 청구=현행유지+안내 / 우선순위=T0.

#### 작업 목록

- ✅ **T0: 수업일 변경 도메인** (최우선) — 케이스1(1회성 출결 행 이동, 동월·메모) + 케이스2(특정일 이후 영구 변경: 날짜 인식 generate + 변경일 이후 미처리만 재생성·결석/보강 보존). V306(`note` 컬럼). IPC `move_attendance`/`apply_schedule_change`. 청구 현행 유지+안내
- ✅ **T1: 회고 액션** — A99(Ctrl+N 방어) + A100(미저장 이탈 경고 공통 훅) + Ctrl+S 저장 단축키
- ✅ **T2: CSV 가져오기** (PRD §4.13.1) — 실사용 개시 첫 번째 작업. 원생 데이터 이관
- ✅ **T3: DB 폴더 변경 + salt.bin 이전** — copy-then-switch + salt.bin/WAL/backup 동반 이전 + 앱 재시작. ADR 작성. cipher ON + 양 PC 시나리오 검증 필수. R12 salt.bin 이전 최종 해소
- ✅ **T4 [Sprint 15 이연] 양 OS 빌드 검증** — macOS `.dmg` + Windows `.msi` 설치/실행/삭제
- ✅ **T5 [Sprint 15 이연] 양 PC 동기화 시나리오 테스트** — Win→Mac, Mac→Win, 비정상 종료 강제 점유 3종
- ✅ **T6: 실사용 개시 준비** — 양 OS 설치 + 데이터 이관 확인 + 기동 검증
- ✅ **T7: 초기 실사용 피드백 대응** — Critical/High 즉시 수정 (버퍼), P0 7건/P1 11건/P2 7건 선별 반영
- ✅ **T8: 접근성 잔여 개선** — 원생 폼 UX 개선(임시저장·필수입력·정합성), 미저장 이탈 경고, Ctrl+S
- ⏸️ **T9: 공지문 I/O 병렬화** (Post-MVP backlog 이연) — 50장 일괄 생성 성능
- ✅ **T10: v1.0 릴리즈 준비** — CHANGELOG v1.0.0 + 버전 1.0.0 업데이트 완료
- ✅ **T11: 통합 검증** — cargo test 415 passed / clippy --all-targets clean / cargo check --features cipher OK / lint + tsc + build 전수 통과 (2026-06-12)
- ⬜ **v1.0 태그 + GitHub Release**: `v1.0.0` 태그 push → CI가 인스톨러 빌드/첨부 (**deploy-prod agent — 사용자 명시 지시 후**)
- ⬜ **배포 후 검증**: deploy-prod agent CV 체크리스트

#### 완료 기준 (Definition of Done)
- ✅ 수업일 변경 케이스1(1회성 이동) + 케이스2(특정일 이후 변경·미처리 재생성·처리행 보존) 동작 + 날짜 인식 generate + V306/V307 적용
- ✅ 양 OS 인스톨러 설치/실행/삭제 정상
- ✅ 양 PC 동기화 시나리오 최소 2종 통과 (Win→Mac, Mac→Win)
- ✅ DB 폴더 변경 정상 동작 + 양 PC 경로 인식 정합 확인
- ✅ CSV 가져오기로 원생 실데이터 이관 성공
- ✅ 실사용 개시 완료 + 초기 Critical/High 피드백 반영 (P0 7건/P1 11건/P2 7건)
- ✅ cargo test 415 passed / clippy --all-targets clean / cipher OK / lint+tsc+build 전수 통과
- ⬜ v1.0.0 GitHub Release 생성 + 양 OS 인스톨러 첨부 (deploy-prod agent 대기)
- ⬜ 배포 후 CV 체크리스트 통과

#### 기술 고려사항
- 수업일 변경: `student_schedules` effective_from/to 시계열 활용. `generate` 날짜 인식(effective_to exclusive) 리팩토링. 케이스2는 변경일 이후 present만 DELETE 후 재생성(처리행 보존 → 보강 고아 회피)
- DB 폴더 변경: copy-then-switch + WAL checkpoint + fsync. salt.bin 손상 감지 적용
- GitHub Release: `deploy.yml` 워크플로우 자동 실행
- 배포(deploy-prod): 사용자 명시 지시 전까지 절대 진행하지 않음

---

## Post-v1.0: 유지보수 + 안정화

### Sprint 17: DB 안전성 잔여 수정 + 클라우드 동기화 정책 간소화 (2주) ✅ 완료 (2026-06-30)

> 계획 문서: `docs/sprint/sprint17.md`
> v1.0.0 실사용 중 발견된 DB 오류 버그 9건 중 긴급 6건은 Hotfix(`hotfix/db-lock-and-backup-fix`)로 선행 처리.
> Sprint 17은 남은 안전성 수정 3건 + 정책 간소화 3건을 담당.
> develop 머지: `sprint17 → develop` 직접 머지 (단일 개발자 정책, 2026-06-30)

#### 작업 목록

**그룹 A — 남은 안전성 수정**
- ✅ **T1**: DB 폴더 변경 WAL 체크포인트 에러 처리 (`setup.rs`)
- ✅ **T2**: 백업 파일 임시 파일 후 이동 방식 — atomic write + stale tmp 정리 (`backup.rs`)
- ✅ **T3**: 자동 복원 후 재검증 — quick_check 최대 3회 retry, `auto_restore_with_retry` (`startup.rs`, `integrity.rs`)

**그룹 B — 정책 간소화**
- ✅ **T4**: Hourly 백업 간격 3600 → 7200초 (MYBOX 부하 절감) (`startup.rs`)
- ✅ **T5**: Heartbeat 완전 제거 — 1인 운영 최적화 (`startup.rs`, `lock.rs`)
- ✅ **T6**: SyncStatus 백엔드+프론트 완전 삭제 — `sync.rs` 삭제, `app-shell.tsx`, `top-bar.tsx`, `types/index.ts`, `tauri/index.ts`

**통합**
- ✅ **T7**: 통합 검증 — cargo test 411건 / clippy --all-targets clean / cipher check / pnpm lint+tsc+build 전수 통과

#### 완료 기준 (Definition of Done)
- ✅ WAL 체크포인트 실패 시 복사 중단 + 사용자 오류 메시지
- ✅ 백업 파일 atomic write (tmp -> rename)
- ✅ 자동 복원 후 재검증, 실패 시 rollback
- ✅ hourly 간격 2시간, heartbeat 제거, SyncStatus 삭제
- ✅ cargo test 411건 + clippy --all-targets + cipher check + lint + tsc + build 전수 통과

---

### Sprint 18: 사용자 피드백 10건 반영 + 캘린더 UX 개선 + 출결 동기화 (2주) 🔄 진행 중

> 계획 문서: `docs/sprint/sprint18.md`
> v1.0.0 실사용 중 수집된 사용자 피드백 이슈 10건 반영. Sprint 17 회고 액션 아이템 5건 선행 처리.
> T1(V308 보강데이 중복허용) + T2(V309 공휴일 중복허용) 이미 커밋 완료.
> develop 머지: `sprint18 → develop` 직접 머지 예정 (단일 개발자 정책)

#### 작업 목록

**회고 액션 아이템 해소**
- ⬜ **T0**: A107~A111 — stale lock 임계값 상향(86400) + rollback 파일명 고유성 + auto_restore_with_retry 테스트 + spawn_blocking + WAL pool.close()

**이미 완료 (V308/V309)**
- ✅ **T1**: 이슈 6 — 보강데이 is_duplicate_blocked=0 (V308 마이그레이션)
- ✅ **T2**: 이슈 6 — 공휴일 is_duplicate_blocked=0 (V309 마이그레이션)

**프론트엔드 — 수업관리 캘린더 UX**
- ⬜ **T3**: 이슈 8 — '결제선생' 카드사 선택 가능 (optional)
- ⬜ **T4**: 이슈 1 — 수업관리 기본 뷰 '주'로 변경
- ⬜ **T5**: 이슈 9 — 달력 요일 순서 '일월화수목금토'로 변경 (FullCalendar + 공지문 달력)
- ⬜ **T6**: 이슈 2+3+4 — 주 보기 색상(시간기준 4색)/레이아웃(2열 균등)/다중슬롯 칩 재정비
- ⬜ **T7**: 이슈 5 — 월 보기 셀 원생 이름 직접 표기 (Nx2 그리드 + hover 상세)

**백엔드 — 출결 자동 동기화**
- ⬜ **T8**: 이슈 7 — 일정코드 변경 시 출결 자동 동기화 (`sync_attendance_on_schedule_change` + academic.rs 3개 IPC 수정 + 단위 테스트 3건)

**프론트엔드 — 교습일정 인쇄**
- ⬜ **T9**: 이슈 10 — 교습일정 인쇄 기능 (A4 세로 + window.print() + @media print)

**통합**
- ⬜ **T10**: 통합 검증 — cargo test + clippy --all-targets + cipher check + pnpm lint+tsc+build 전수 통과

#### 완료 기준 (Definition of Done)
- ⬜ Sprint 17 회고 액션 A107~A111 전수 해소
- ⬜ 수업관리 기본 뷰 '주' + 요일 '일월화수목금토' + 시간 기준 4색 + 다중 슬롯 칩
- ⬜ 월 보기 셀 원생 이름 직접 표기
- ⬜ 결제선생 카드사 optional 선택
- ⬜ 일정코드 변경 시 출결 자동 동기화 (OFF->ON, ON->OFF)
- ⬜ 교습일정 인쇄 A4 레이아웃 정상 출력
- ⬜ cargo test 전수 통과 + clippy --all-targets + cipher check + lint + tsc + build

---

## 📈 마일스톤

| 마일스톤 | Phase | Sprint | 예상 시점 | 핵심 산출물 |
|----------|-------|--------|----------|------------|
| M1: 인프라 확립 | Phase 1 | Sprint 1 | +2주 | SQLCipher + app.lock + 4계층 백업 동작 |
| M2: 원생 관리 가능 | Phase 1 | Sprint 3 | +6주 | 원생 등록/조회 + 마법사 + 글로벌 검색 |
| M3: 학사+출결 동작 | Phase 2 | Sprint 8 | +14주 | 학사 일정 수립 + 출결 생성/관리 |
| M4: 보강 완성 | Phase 3 | Sprint 10 | +18주 | 보강 매칭 + 소멸 + 캘린더 뷰 |
| M5: 청구 완성 | Phase 4 | Sprint 12 | +22주 | 청구/수납/공지문 |
| ~~M6: 평가+보고서~~ | ~~Phase 5~~ | — | — | ❌ 전면 취소 (2026-05-31) |
| M6: PIN 옵션화 | — | Sprint 13 | +24주 ✅ | PIN 스킵 토글 + Phase 5 취소 반영 |
| M7: 대시보드 | Phase 5 | Sprint 14 | +26주 ✅ | 대시보드 + 자가 진단 + 내보내기 |
| M7.5: 안정화 | Phase 6 | Sprint 15 | +28주 ✅ | 교습소 정보 + 접근성 감사 + 성능 최적화 (T7~T9 Sprint 16 이연) |
| M8: v1.0 릴리즈 | Phase 6 | Sprint 16 | +32주 ✅ | 양 OS 빌드 검증 + UAT + v1.0.0 |
| M9: v1.0.1 안정화 | Post-v1.0 | Sprint 17 | +34주 ✅ | DB 안전성 잔여 수정 + 정책 간소화 |
| M10: 사용자 피드백 반영 | Post-v1.0 | Sprint 18 | +36주 | 캘린더 UX 개선 + 출결 동기화 + 교습일정 인쇄 |

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
- ⬜ **R12 나머지: salt.bin 이전** — Keychain -> `{cloud}/smarthb/salt.bin` 이전. paths 동적화 완료 후 별도 sprint에서 처리. → **Sprint 15 'DB 폴더 변경(경로 재지정)' 작업과 상호 의존** (경로 재지정 시 salt.bin 동반 이전 필수) — 함께 설계.
- ⬜ **query!() 매크로 전환** — 동적 `query() + bind()` 패턴 유지 중. V101~V105 스키마 안정화 후 일괄 전환 예정. 별도 backlog 추적.

---

## 📋 품질 검증 체크리스트

- ✅ PRD의 핵심 기능(§4.0~§4.6, §4.9~§4.14, §5.3~§5.5, §6.6)이 로드맵에 반영됨 (§4.7~§4.8 취소)
- ✅ 각 Phase의 의존성이 올바르게 설정됨 (인프라 → 기반 도메인 → 학사/출결 → 보강 → 청구 → PIN 옵션화 → 대시보드 → 안정화)
- ✅ MVP 범위가 PRD v1.0 기준으로 명확히 정의됨 (§4.7~§4.8 취소, Post-MVP §4.15 제외)
- ✅ 각 태스크가 IPC 커맨드명, 마이그레이션 번호, UI 컴포넌트 수준으로 구체적
- ✅ 완료 기준이 측정 가능 (성능 수치, 테스트 통과, 양 OS 검증)
- ✅ 각 Phase에 Playwright MCP 검증 시나리오 포함
- ✅ PI-01/PI-02/PI-05/PI-07 미해결 항목의 결정 시점이 명시됨
