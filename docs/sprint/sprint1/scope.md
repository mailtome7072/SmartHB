---
Sprint: 1  |  Date: 2026-05-19  |  Session: #3 (T3 진입)
---

## 세션 진행 기록

- **Session #1** (T1 SQLCipher PoC + ADR-001): ✅ 완료. commit `10bf105`.
- **Session #2** (T2 에러 처리 기반 + AppError 첫 마이그레이션): ✅ 완료. commit `9ba7f6a`.
- **Session #3** (T3 OS Keychain + PBKDF2 + ADR-004): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T3 — Day 3 두 번째 작업)

**OS Keychain 통합 + PBKDF2 키 유도 + ADR-004** · skill: brainstorming

- `keyring` crate v3.x 도입 (양 OS Keychain/Credential Manager 통합 API)
- PBKDF2-HMAC-SHA256 + **600,000 iterations** (OWASP 2024 권장) + 32바이트 salt + 32바이트 출력 (AES-256 키 크기)
- `zeroize` crate 통합 — `DerivedKey` 가 Drop 시 자동으로 메모리 폐기
- `Debug` trait 수동 구현 — 키 바이트 로그 출력 차단 (`"DerivedKey([REDACTED])"`)
- `docs/arch/adr-004-keychain-crate.md` 작성 — brainstorming 스킬 적용 (Weighted Matrix + SWOT)
- `src-tauri/src/commands/auth.rs` 신규 — 키 유도/저장/조회/삭제 함수 + 단위 테스트
- T4 (인증 IPC + 잠금 화면 UI) 의 기반 모듈

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/Cargo.toml | [0회] | keyring/pbkdf2/sha2/hmac/zeroize/rand/hex crate 추가 |
| src-tauri/src/commands/auth.rs | [0회] | 신규 — 키 유도/Keychain 통합 함수 |
| src-tauri/src/commands/mod.rs | [0회] | mod auth; 추가 |
| docs/arch/adr-004-keychain-crate.md | [0회] | 신규 — brainstorming 스킬 적용 |
| docs/sprint/sprint1/scope.md | [0회] | 본 파일 — Session #3 갱신 |

### 이전 세션 변경 파일 (참고)

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/Cargo.toml | [1회] | T1: cipher feature + libsqlite3-sys optional |
| src-tauri/src/error.rs | [1회] | T2: AppError 7종 |
| src-tauri/src/lib.rs | [2회] | T1 invoke_handler, T2 mod error |
| src-tauri/src/commands/mod.rs | [2회] | T1 diagnose_sqlcipher, T2 AppError 마이그레이션 |
| docs/arch/adr-001-sqlcipher-integration.md | [1회] | T1 ADR |
| docs/setup-guide.md | [1회] | T1 Strawberry Perl 안내 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단). SQLCipher CI 빌드 매트릭스 변경은 T11 단계에서 사용자 허가 후 진행
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단). SQLCipher 빌드 도구 안내 추가는 사용자 허가 후 진행
- ⬜ `docs/harness-engineering/` — 정책 문서 (수정 불필요)
- ⬜ `.claude/` — 에이전트/룰/스킬/훅 (수정 불필요)
- ⬜ `PRD.md`, `ROADMAP.md`, `docs/phase/`, `docs/sprint/sprint1.md` — 계획·사양 SSOT (구현 단계에서 수정 금지)
- ⬜ `src/` (프론트엔드) — Sprint 1 후반 T4(잠금 화면)부터 진입, T1 PoC에서는 변경 없음

## 이번 세션의 완료 기준

- ⬜ Cargo.toml에 `libsqlite3-sys` 의존성 추가 + `bundled-sqlcipher-vendored-openssl` feature 활성화
- ⬜ Windows 로컬 빌드 (`cargo build --manifest-path src-tauri/Cargo.toml`) 성공
- ⬜ macOS 로컬 빌드는 사용자 환경(자택 Mac)에서 검증 필요 — Windows 측 검증 후 사용자에게 macOS 검증 요청
- ⬜ sqlx pool로 SQLCipher 암호화 DB 열기 + 간단한 CREATE/INSERT/SELECT 테스트
- ⬜ PRAGMA key 적용 시점·방법 코드 검증 (Connection 생성 직후)
- ⬜ Cargo feature flag 분리 설계 (`cipher` feature on/off로 SQLCipher 적용 여부 제어)
- ⬜ ADR-001 작성: 도입 방식 (bundled-sqlcipher-vendored-openssl) + 빌드 영향 + 보안 영향 + 대체안

## 발견된 이슈

### Issue #1 (2026-05-19 Session #1): vendored-openssl 빌드 시 Windows Perl 모듈 누락

- **현상**: `cargo build --manifest-path src-tauri/Cargo.toml` 실행 시 `openssl-sys` 의 vendored 빌드 단계에서 `Locale::Maketext::Simple.pm` 모듈을 찾지 못해 실패
- **원인**: Windows의 git bash/MSYS2 Perl 환경에 OpenSSL 빌드 스크립트가 요구하는 표준 Perl 모듈이 누락. vendored-openssl 사용 시 OpenSSL 소스에서 직접 빌드하므로 풀 Perl 환경 필요
- **영향**: ADR-001의 1차 가정("`cargo build` 한 번이면 끝") 부분 실패. 단, macOS는 system Perl 충분, Windows만 영향
- **2차 시도 방안** (1차 실패 후 다른 접근):
  - `libsqlite3-sys` 를 **optional dependency** 로 전환하고 `cipher` feature 와 연결 → `cargo build` (default) 는 SQLCipher 빌드 발동 안 함 → Windows 빌드 즉시 성공
  - `cargo build --features cipher` (프로덕션·CI 빌드) 는 Strawberry Perl 또는 ActivePerl 설치 후 진행
  - 개발자 안내는 `docs/setup-guide.md` 에 추가 (Forbidden Area 아니므로 즉시 수정 가능)
- **사용자 보고 후 진행**: 본 결정은 ADR-001 핵심 가정에 영향을 미치므로 ADR-001 본문에 `Consequences > 부정적 영향` 항목 보강 + 사용자 명시 확인 후 진행

## brainstorming 스킬 적용 — ADR-001 작성 방법

T1은 sprint1.md에서 `skill: brainstorming` 으로 명시되었다. ADR 작성 시 다음 절차:

1. **후보 옵션 정의** (이미 사전 정의됨):
   - (A) `libsqlite3-sys` `bundled-sqlcipher-vendored-openssl` feature
   - (B) 시스템 sqlcipher 라이브러리 (`brew install sqlcipher` / `vcpkg install sqlcipher`)
2. **Weighted Matrix** 평가축:
   - 빌드 단순성 (양 OS CI 통과 용이성)
   - 보안성 (라이브러리 신뢰성·유지보수)
   - 번들 크기 영향
   - 개발자 경험 (로컬 설정 부담)
   - 라이선스 호환성
3. **SWOT** 분석 (각 옵션별 Strength/Weakness/Opportunity/Threat)
4. ADR 본문에 두 분석 결과 + 권장안 + 결정 + Consequence 명시
5. 저장 경로: `docs/arch/adr-001-sqlcipher-integration.md`

## 추후 세션 계획 (참고)

- Session #2: T2 에러 처리 기반 + T3 Keychain + ADR-004
- Session #3: T4 인증 + 잠금 화면 UI
- Session #4: T5 복구 코드 + T6 app.lock + ADR-002
- Session #5: T7 백업 + ADR-003 + T8 무결성
- Session #6: T9 동기화/감사/코드 + T10 시작 시퀀스 통합
- Session #7: T11 단위 테스트 + CI 검증

본 scope.md는 각 세션 시작 시 갱신한다.
