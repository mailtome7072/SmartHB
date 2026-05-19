# Sprint Plan sprint1

## 기간
2026-05-19 ~ 2026-06-01 (2주, 10일)

## 목표
SQLCipher 암호화 DB + 인증 + app.lock 동시성 제어 + 4계층 백업 + 무결성 검증의 전체 데이터 인프라를 구축하여, 이후 모든 Sprint가 안전한 데이터 기반 위에서 작업할 수 있게 한다. 앱 시작 ~ 메인 진입 3초 이내 성능 목표를 달성한다.

## ROADMAP 연계 기능
- Phase 1 > Sprint 1: 데이터 인프라
- M1 마일스톤: SQLCipher + app.lock + 4계층 백업 동작
- Phase 설계: `docs/phase/phase1.md` (ADR 4건, 전문가 4관점 검토 반영)

## 핵심 제약 사항
- **첫 2일 PoC 우선**: SQLCipher + `bundled-sqlcipher-vendored-openssl` 양 OS 빌드 검증을 Day 1~2에 완료. 실패 시 즉시 Sprint 재계획
- **메모리 zeroize**: 모든 비밀번호/키 메모리는 `zeroize` crate로 사용 후 즉시 폐기
- **PRAGMA quick_check 우선**: 일상 무결성 검증은 quick_check 사용 (~50ms). integrity_check는 일일 백업 시점 + 사용자 수동 요청 시만
- **성능 기준**: 앱 시작 ~ 메인 진입 < 3초 (동기화 대기 제외, PRD SS5.6)
- **Forbidden Area**: SETUP.sh / .github/workflows/ 수정 필요 시 사용자 허가 요청 필수

---

## 작업 목록

### Day 1~2: SQLCipher PoC (최우선 리스크 해소)

- ⬜ **T1. SQLCipher PoC + ADR-001 작성** · skill: brainstorming
  - `libsqlite3-sys` `bundled-sqlcipher-vendored-openssl` feature 로컬 빌드 검증 (Win/Mac)
  - sqlx + SQLCipher 통합 CRUD 테스트 (인메모리 + 파일 DB)
  - Cargo.toml feature flag 설정: `cipher` feature로 개발/프로덕션 분리
  - ADR 문서: `docs/arch/adr-001-sqlcipher-integration.md`
  - **PoC 실패 시**: 시스템 sqlcipher 라이브러리 대체안 시도 → 2차 실패 시 사용자 보고 + Sprint 재계획
  - 산출물: ADR-001, Cargo.toml 업데이트, PoC 테스트 코드

### Day 3~4: 키 관리 + 인증 + 에러 처리 기반

- ⬜ **T2. 에러 처리 기반 구축**
  - `src-tauri/src/error.rs`: `thiserror` 기반 커스텀 에러 타입 정의 (AppError enum)
  - 에러 카테고리: Auth, Db, Lock, Backup, Integrity, Io, Config
  - Tauri 커맨드용 `impl From<AppError> for String` 직렬화
  - 사용자 친화 한국어 메시지 매핑
  - 산출물: `src-tauri/src/error.rs`

- ⬜ **T3. OS Keychain 통합 + PBKDF2 키 유도 + ADR-004** · skill: brainstorming
  - `keyring` crate v3.x 도입 + 양 OS 테스트
  - PBKDF2 600,000 iterations + 32바이트 salt (OWASP 2024 권장)
  - `zeroize` crate 통합 — 키 바이트 DB 열기 직후 즉시 zeroize
  - 비밀번호/키/복구 코드 Debug trait 수동 구현 (로그 출력 방지)
  - ADR 문서: `docs/arch/adr-004-keychain-crate.md`
  - 산출물: ADR-004, `src-tauri/src/commands/auth.rs` (키 유도 함수)

- ⬜ **T4. 인증 IPC + 잠금 화면 UI** · skill: frontend-design
  - Tauri IPC: `set_password`, `unlock_db`, `check_auth_status`
  - 최초 실행 시 비밀번호 설정 화면: "앱 보호를 위해 비밀번호를 설정해주세요" + 확인 입력
  - 잠금 화면 UI: 입력 필드 56px, 비밀번호 표시 토글 (눈 아이콘 44x44px)
  - 오류 시 빨간 테두리 + 한국어 안내: "비밀번호가 올바르지 않습니다"
  - "비밀번호를 잊으셨나요?" 링크 → 복구 코드 입력 화면
  - 색상: 배경 `#F9F7F4`, 주요 버튼 `#2563EB`, 텍스트 `#1A1A1A`
  - 산출물: `src/app/lock/page.tsx`, `src/components/LockScreen.tsx`

### Day 5~6: 복구 코드 + app.lock

- ⬜ **T5. PI-07 복구 코드 발급/검증**
  - `argon2` crate 도입 — 복구 코드 해시는 Argon2id (PBKDF2보다 메모리-하드)
  - 12자리 영숫자 생성 (`OsRng`, 혼동 문자 0/O/1/l/I 제거)
  - 평문 1회 표시 후 즉시 zeroize
  - 재발급: 기존 코드 무효화 + 비밀번호 재입력 필수
  - 검증 흐름: 복구 코드 → Argon2id 해시 비교 → 새 비밀번호 설정 → SQLCipher rekey → Keychain 갱신
  - 분실 경고: "비밀번호와 복구 코드 모두 분실 시 데이터에 영구 접근 불가" 명시
  - IPC: `generate_recovery_code`, `verify_recovery_code`, `reset_password_with_code`
  - 산출물: `src-tauri/src/commands/recovery.rs`, 복구 코드 입력 UI 컴포넌트

- ⬜ **T6. app.lock 동시성 제어 + ADR-002 + 경고 화면** · skill: brainstorming
  - `fs2` crate advisory locking + 자체 heartbeat (60초 갱신)
  - 락 파일 구조: `{"device_id": "UUIDv4", "last_heartbeat": "ISO8601"}` (최소 정보만)
  - 디바이스 ID: 랜덤 UUID v4 (MAC 주소/하드웨어 시리얼 금지)
  - 5분 미갱신 시 강제 점유 옵션
  - 정상 종료 시 락 자동 해제
  - 경고 화면: "다른 컴퓨터에서 이 프로그램을 사용 중입니다" + 타이머 표시 + 강제 점유 버튼 44x44px
  - ADR 문서: `docs/arch/adr-002-applock-library.md`
  - IPC: `acquire_lock`, `release_lock`, `check_lock_status`
  - 산출물: ADR-002, `src-tauri/src/commands/lock.rs`, 경고 화면 UI 컴포넌트

### Day 7~8: 백업 + 무결성 검증

- ⬜ **T7. 4계층 자동 백업 + ADR-003** · skill: brainstorming
  - `rusqlite` crate 보조 의존성 추가 (백업 전용)
  - SQLite Online Backup API (`rusqlite::backup`) 사용
  - 4계층: exit(10) / hourly(24) / daily(30) / weekly(4) 순환 삭제
  - 파일명: `app_YYYYMMDD_HHMMSS.db`
  - 백업 파일 SQLCipher 암호화 상태 유지 (복호화 금지)
  - 백업 직후 `PRAGMA quick_check` 검증
  - hourly 백업: tokio 백그라운드 task (UI 비블로킹)
  - exit 백업: 동기 실행 (종료 전 완료 보장, ~50ms)
  - ADR 문서: `docs/arch/adr-003-backup-implementation.md`
  - IPC: `create_backup`, `list_backups`, `restore_backup`
  - 산출물: ADR-003, `src-tauri/src/commands/backup.rs`

- ⬜ **T8. 무결성 검증 + 자동 복원**
  - 앱 시작 시 `PRAGMA quick_check` 실행 (~50ms)
  - 일일 백업 시점에 `PRAGMA integrity_check` 실행 (full 검증)
  - 손상 감지 시: `backup/exit/` 최신본 자동 복원 + 사용자 알림 + 손상본 `corrupted/` 격리
  - 복원 전 대상 백업 파일 `PRAGMA quick_check` 통과 확인
  - 복원 직전 현재 DB를 `restore_rollback/`에 보존
  - IPC: `check_integrity`, `auto_restore`
  - 산출물: `src-tauri/src/commands/integrity.rs`

### Day 9: 동기화 대기 + 감사 로그 + 코드 테이블 + 시작 시퀀스

- ⬜ **T9. 동기화 대기 + 감사 로그 + 코드 테이블**
  - **동기화 대기**: DB/락 파일 mtime 확인으로 최신 동기화 판단 + 30초 타임아웃 + 사용자 새로고침 옵션
  - 동기화 대기는 3초 예산 외 — 별도 대기 화면 표시
  - IPC: `check_sync_status`
  - **감사 로그**: audit_logs 테이블 + 로깅 미들웨어 (우선 기록: 비밀번호 변경, 복구 코드 발급, 백업 복원, 락 강제 점유)
  - 민감 데이터 마스킹 (비밀번호, 복구 코드 해시 필터링)
  - 1년 롤링 보관 (앱 시작 시 오래된 로그 삭제)
  - IPC: `get_audit_logs`
  - **코드 테이블**: schools, payment_methods, card_companies, standard_fees 시드 데이터
  - DB 마이그레이션: `V001__create_code_tables.sql`, `V008__create_app_settings_and_audit_logs.sql`
  - 산출물: `src-tauri/src/commands/sync.rs`, `src-tauri/src/commands/audit.rs`, 마이그레이션 파일 2개

- ⬜ **T10. 앱 시작 시퀀스 통합 + 성능 검증**
  - 시퀀스: 동기화 대기(별도) → [병렬: 락 확인 + 무결성 검증(quick_check)] → 인증 → 메인 진입
  - `tokio::join!`으로 락 확인 + 무결성 검증 병렬화
  - PRAGMA 설정: `journal_mode=WAL`, `cache_size=-8000`
  - 성능 측정: 시작 ~ 메인 진입 < 3초 (동기화 대기 제외)
  - IPC: `app_startup_sequence`
  - `src-tauri/src/lib.rs` invoke_handler에 모든 커맨드 등록
  - `src/lib/tauri/index.ts` IPC 래퍼 함수 추가
  - 산출물: `src-tauri/src/startup.rs`, `src/lib/tauri/index.ts` 업데이트

### Day 10: 테스트 + CI 검증 + 마무리

- ⬜ **T11. 단위 테스트 + CI 양 OS 빌드 검증**
  - 각 커맨드 모듈 `#[cfg(test)]` 블록 작성
  - SQLCipher CRUD 테스트 (인메모리 DB)
  - app.lock 생성/해제/heartbeat/강제점유 테스트
  - 백업 생성/순환삭제/복원 테스트
  - 무결성 검증 + 손상 감지 테스트
  - 복구 코드 생성/검증/재발급 테스트
  - 비즈니스 규칙: PBKDF2 키 유도, Argon2id 해시, zeroize 동작
  - `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
  - `pnpm lint` + `pnpm tsc --noEmit` 통과
  - CI (.github/workflows/) 양 OS 빌드 검증 — **Forbidden Area이므로 수정 필요 시 사용자 허가 요청**
  - 산출물: 테스트 코드, CI 결과 확인

---

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 팀 규모 | AI 페어 프로그래밍 1인 |
| 스프린트 일수 | 10일 (2주) |
| 실작업 가능 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |
| Task 수 | 11개 |
| Task당 평균 예상 | 3~4시간 |
| 총 예상 작업량 | ~38시간 |
| 여유율 | ~5% (PoC 실패 시 재계획으로 대응) |

> SQLCipher PoC(T1)에 첫 2일(8시간)을 배정하여 최대 리스크를 조기에 해소한다. PoC 실패 시 Sprint 재계획이므로 후속 Task 작업량은 실질적으로 PoC 성공을 전제한다.

---

## 의존성 및 리스크

### 의존성

| Task | 선행 의존 | 비고 |
|------|----------|------|
| T2 (에러 처리) | 없음 | T1과 병렬 가능하나 T1 결과에 따라 에러 타입 조정 |
| T3 (키 관리) | T1 (SQLCipher PoC) | SQLCipher 통합 방식 확정 후 키 관리 구현 |
| T4 (인증 UI) | T2, T3 | 에러 타입 + 키 유도 함수 필요 |
| T5 (복구 코드) | T3, T4 | 인증 흐름 위에 복구 코드 구현 |
| T6 (app.lock) | T2 | 에러 타입만 필요, T1과 독립적 |
| T7 (백업) | T1 | rusqlite 의존, SQLCipher DB 필요 |
| T8 (무결성) | T7 | 백업 파일 경로 + 복원 로직 필요 |
| T9 (동기화+감사+코드) | T2 | 에러 타입 필요 |
| T10 (시작 시퀀스) | T3~T9 전체 | 모든 인프라 통합 |
| T11 (테스트+CI) | T1~T10 전체 | 전체 통합 테스트 |

### 리스크

| ID | 리스크 | 영향도 | 발생 확률 | 대응 방안 |
|----|--------|--------|----------|----------|
| R1 | SQLCipher `bundled-sqlcipher-vendored-openssl` 양 OS 빌드 실패 | 높음 | 중간 | 첫 2일 PoC에서 검증. 실패 시 시스템 sqlcipher → 2차 실패 시 Sprint 재계획 |
| R2 | `rusqlite` + `sqlx` 동시 사용 시 DB 커넥션 충돌 | 중간 | 낮음 | rusqlite 커넥션은 백업 전용 읽기 전용으로 제한, sqlx pool과 격리 |
| R3 | 앱 시작 시퀀스 3초 초과 | 높음 | 낮음 | quick_check (~50ms) + 병렬화로 ~1초 목표. 측정 후 병목 제거 |
| R4 | `keyring` crate 양 OS 호환성 이슈 | 중간 | 낮음 | PoC에서 양 OS 테스트. 실패 시 platform-specific crate 대체 |
| R5 | Argon2id 연산 시간 200ms 초과 (앱 시작 시) | 중간 | 낮음 | 양 OS에서 파라미터 튜닝, 200ms 이내 달성 확인 |
| R6 | CI (.github/workflows/) 수정 필요 — Forbidden Area | 중간 | 높음 | SQLCipher 빌드를 위한 CI 수정은 사용자 허가 후 진행 |

---

## 기술적 접근 방법

### 새 의존성 (Cargo.toml 추가 예정)
- `libsqlite3-sys` — `bundled-sqlcipher-vendored-openssl` feature (SQLCipher 통합)
- `keyring` ^3 — OS Keychain/Credential Manager 통합
- `argon2` — 복구 코드 Argon2id 해시
- `zeroize` — 민감 데이터 메모리 폐기
- `rusqlite` — SQLite Online Backup API (백업 전용)
- `fs2` — 파일 시스템 advisory locking
- `uuid` — 디바이스 ID UUIDv4 생성
- `rand` + `rand_core` — 복구 코드 생성 (OsRng)
- `chrono` — 타임스탬프 처리

### DB 마이그레이션 (Sprint 1 범위)
- `V001__create_code_tables.sql` — schools, payment_methods, card_companies, standard_fees
- `V008__create_app_settings_and_audit_logs.sql` — app_settings (recovery_code_hash 포함), audit_logs

### PRAGMA 설정 (앱 시작 시)
```sql
PRAGMA journal_mode=WAL;
PRAGMA cache_size=-8000;
PRAGMA foreign_keys=ON;
```

### 파일 구조 (신규 생성 예정)
```
src-tauri/
  src/
    error.rs              -- 커스텀 에러 타입
    startup.rs            -- 앱 시작 시퀀스 오케스트레이터
    commands/
      mod.rs              -- 기존 (확장)
      auth.rs             -- 인증 + 키 관리
      recovery.rs         -- 복구 코드 발급/검증
      lock.rs             -- app.lock 동시성 제어
      backup.rs           -- 4계층 백업
      integrity.rs        -- 무결성 검증
      sync.rs             -- 동기화 대기
      audit.rs            -- 감사 로그
  migrations/
    V001__create_code_tables.sql
    V008__create_app_settings_and_audit_logs.sql

docs/arch/
  adr-001-sqlcipher-integration.md
  adr-002-applock-library.md
  adr-003-backup-implementation.md
  adr-004-keychain-crate.md

src/
  app/lock/page.tsx       -- 잠금 화면
  components/
    LockScreen.tsx        -- 잠금 화면 컴포넌트
    LockWarning.tsx       -- app.lock 경고 화면
    SyncWaiting.tsx       -- 동기화 대기 화면
  lib/tauri/index.ts      -- IPC 래퍼 (확장)
```

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ SQLCipher AES-256 암호화된 DB로 CRUD 동작 확인 (양 OS)
- ⬜ PBKDF2 600K iterations 키 유도 + zeroize 동작 확인
- ⬜ 복구 코드 Argon2id 해시 발급/검증/재발급 동작
- ⬜ app.lock으로 양 PC 시점 분리 동작 검증 (heartbeat 60초, 강제 점유 5분)
- ⬜ 4계층 백업 생성/순환 삭제 동작 (exit/hourly/daily/weekly)
- ⬜ PRAGMA quick_check 통과 + 손상 시 자동 복원 동작
- ⬜ 앱 시작 시퀀스: 동기화 대기(별도) → 락 + 무결성(병렬) → 인증 → 메인 진입 < 3초
- ⬜ 잠금 화면 UI: 56px 입력 필드, 비밀번호 표시 토글, 한국어 오류 메시지
- ⬜ 코드 테이블 (schools, payment_methods, card_companies, standard_fees) 마이그레이션 적용
- ⬜ 감사 로그 기록 동작 (비밀번호 변경, 복구 코드 발급, 백업 복원, 락 강제 점유)
- ⬜ cargo test 전체 통과
- ⬜ cargo clippy -- -D warnings 통과
- ⬜ pnpm lint + pnpm tsc --noEmit 통과
- ⬜ ADR-001 (SQLCipher), ADR-002 (app.lock), ADR-003 (백업), ADR-004 (Keychain) 문서 완료

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ DEPLOY.md 업데이트
- ⬜ CI 양 OS 빌드 통과 확인

**보안 체크리스트 (보안 전문가 리뷰 기준)**
- ⬜ SQLCipher 키 zeroize 구현 확인
- ⬜ 비밀번호/키/복구 코드 로그 출력 없음 확인
- ⬜ 백업 파일 암호화 상태 유지 확인
- ⬜ audit_logs 민감 데이터 마스킹 확인
- ⬜ 디바이스 ID는 랜덤 UUIDv4 (개인정보 포함 금지)

---

## 예상 산출물

### 코드
- `src-tauri/src/error.rs` — 커스텀 에러 타입
- `src-tauri/src/startup.rs` — 앱 시작 시퀀스
- `src-tauri/src/commands/auth.rs` — 인증 + 키 관리
- `src-tauri/src/commands/recovery.rs` — 복구 코드
- `src-tauri/src/commands/lock.rs` — app.lock
- `src-tauri/src/commands/backup.rs` — 4계층 백업
- `src-tauri/src/commands/integrity.rs` — 무결성 검증
- `src-tauri/src/commands/sync.rs` — 동기화 대기
- `src-tauri/src/commands/audit.rs` — 감사 로그
- `src-tauri/migrations/V001__create_code_tables.sql`
- `src-tauri/migrations/V008__create_app_settings_and_audit_logs.sql`
- `src/app/lock/page.tsx` — 잠금 화면
- `src/components/LockScreen.tsx`
- `src/components/LockWarning.tsx`
- `src/components/SyncWaiting.tsx`
- `src/lib/tauri/index.ts` — IPC 래퍼 확장

### 문서
- `docs/arch/adr-001-sqlcipher-integration.md`
- `docs/arch/adr-002-applock-library.md`
- `docs/arch/adr-003-backup-implementation.md`
- `docs/arch/adr-004-keychain-crate.md`
- `docs/sprint/sprint1/scope.md` — sprint-dev 진입 시 작성

---

## 참고 사항

### 전문가 검토 반영 요약

| 관점 | 핵심 반영 사항 |
|------|---------------|
| 보안 | PBKDF2 600K iter, 복구 코드 Argon2id, zeroize crate, 민감 데이터 로그 차단 |
| 성능 | PRAGMA quick_check (integrity_check 대비 10~100배), 시작 시퀀스 병렬화, WAL 모드 |
| UX | 잠금 입력 56px, 비밀번호 표시 토글, 색상 팔레트 `#F9F7F4`/`#2563EB`, 분실 경고 문구 |
| PO/인프라 | `bundled-sqlcipher-vendored-openssl`로 CI 양 OS 단순화, PoC 첫 2일 집중 |

### PI-07 결정 상태
- **결정 완료** (PRD v1.5.1): 설정 메뉴에서 복구 코드 발급/재발급. Argon2id 해시 저장. Sprint 1에서 구현.

### SETUP.sh / CI 수정 가능성
- SQLCipher 빌드를 위해 `SETUP.sh`(SQLCipher 빌드 도구 확인)와 `.github/workflows/ci.yml`(양 OS 빌드 매트릭스)에 변경이 필요할 수 있다.
- 두 파일 모두 Forbidden Area이므로, 수정이 필요하다고 판단되면 사용자에게 허가를 요청한 후 진행한다.

### Phase 1 설계 문서 참조
- `docs/phase/phase1.md` — Sprint 1~3 전체 설계, ADR 6건, 의존성 맵
- `docs/phase/phase1/phase1-보안전문가-review.md`
- `docs/phase/phase1/phase1-성능엔지니어-review.md`
- `docs/phase/phase1/phase1-UX전문가-review.md`
- `docs/phase/phase1/phase1-PO-review.md`
