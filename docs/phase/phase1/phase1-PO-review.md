# Phase 1 PO (Product Owner) + 인프라 검토

> 검토 대상: Phase 1 (인프라 + 기반 도메인, Sprint 1~3)
> 검토일: 2026-05-18
> 관점: 비즈니스 요구사항 충족도, 우선순위 적절성, CI/CD 인프라, 양 OS 빌드

---

## 1. 비즈니스 요구사항 충족도

### Phase 1 → PRD 매핑 검증

| PRD 섹션 | 요구사항 | Sprint | 상태 |
|----------|---------|--------|------|
| §4.0 | 초기 설정 마법사 9단계 | Sprint 3 | ✅ 반영 |
| §4.1 | 원생 관리 (등록/수정/조회/퇴교) | Sprint 2~3 | ✅ 반영 |
| §4.2 | 수업 스케줄 (요일별 + 이력) | Sprint 2~3 | ✅ 반영 |
| §4.3 | 표준 교습비 매핑 | Sprint 2 | ✅ 반영 |
| §4.12 | 코드 테이블 관리 | Sprint 1~3 | ✅ 반영 |
| §4.14 | 글로벌 검색 | Sprint 3 | ✅ 반영 |
| §5.3 | app.lock 동시성 제어 | Sprint 1 | ✅ 반영 |
| §5.4 | 4계층 자동 백업 | Sprint 1 | ✅ 반영 |
| §5.5 | SQLCipher + 인증 + 복구 코드 | Sprint 1 | ✅ 반영 |
| §5.6 | 앱 시작 < 3초 | Sprint 1 | ✅ 반영 |
| §5.7 | 접근성 기준선 | Sprint 3 | ✅ 반영 |
| §7.3 | 감사 로그 | Sprint 1 | ✅ 반영 |
| PI-07 | 복구 코드 발급/재발급 | Sprint 1 | ✅ 반영 (v1.5.1) |

**누락 항목**: 없음. Phase 1 범위의 모든 PRD 요구사항이 Sprint 1~3에 배치됨.

---

## 2. 우선순위 적절성

### Sprint 순서 합리성 평가

| 평가 항목 | 판정 | 사유 |
|----------|------|------|
| Sprint 1 → 인프라 선행 | ✅ 적절 | SQLCipher, app.lock, 백업은 후속 모든 Sprint의 전제 조건 |
| Sprint 2 → 백엔드 선행 | ✅ 적절 | Sprint 3 프론트엔드가 호출할 API를 먼저 완성 |
| Sprint 3 → 프론트엔드 후행 | ✅ 적절 | 안정된 백엔드 위에서 UI 구축 |
| V001~V008 마이그레이션 순서 | ✅ 적절 | 외래키 의존 순서 (코드 → 원생 → 학사 → 설정) |

### 볼륨 분배 평가

| Sprint | 예상 작업량 | 평가 |
|--------|-----------|------|
| Sprint 1 | ADR 4건 + PoC + 인프라 전체 | **높음** — 3초 목표 + SQLCipher PoC가 핵심 리스크. PoC 실패 시 일정 지연 가능 |
| Sprint 2 | IPC 10+ 커맨드 + 마이그레이션 3건 | **보통** — 비즈니스 로직은 단순 (CRUD + 이력), 패턴 반복 |
| Sprint 3 | UI 컴포넌트 10+ 개 + 마법사 9단계 | **높음** — 마법사 볼륨이 크지만, shadcn/ui 활용으로 완화 가능 |

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| **필수** | Sprint 1 리스크 버퍼 | SQLCipher PoC를 Sprint 1 첫 2일에 배치. 실패 시 즉시 대안 채택 (ADR-001에 fallback 명시) |
| 권고 | Sprint 3 우선순위 | 마법사가 볼륨 초과 시 "가져오기(단계 7)"를 이연. 핵심 단계(운영시간, 학교코드, 표준교습비, 백업폴더)만 Sprint 3, 나머지는 후속 |

---

## 3. CI/CD 인프라 (양 OS 빌드)

### SQLCipher CI 빌드 전략

| OS | 방법 | CI 설정 |
|----|------|---------|
| **Windows** | `bundled-sqlcipher-vendored-openssl` feature | OpenSSL 별도 설치 불필요. Cargo feature flag만 추가 |
| **macOS** | `bundled-sqlcipher-vendored-openssl` feature | 동일. Homebrew openssl 불필요 |
| **Linux (CI)** | 동일 feature | Ubuntu runner에서도 작동 |

### 권고사항

| 등급 | 항목 | 설명 |
|------|------|------|
| **필수** | `bundled-sqlcipher-vendored-openssl` feature 확인 | `libsqlite3-sys` crate가 이 feature를 제공하는지 Sprint 1 PoC에서 확인. 없으면 CI에 OpenSSL 설치 스텝 추가 필요 |
| **필수** | CI 양 OS 빌드 검증 | Sprint 1 완료 시 GitHub Actions에서 `cargo build` + `cargo test`가 windows-latest, macos-latest에서 통과 확인 |
| 권고 | Cargo.toml feature 분리 | `[features]` 섹션에 `cipher = ["sqlx/sqlite", "libsqlite3-sys/bundled-sqlcipher"]` 정의하여 개발/CI/프로덕션 구분 |
| 권고 | 개발 환경 SQLCipher 비활성화 | 개발 편의를 위해 `SmartHB-dev.db`는 SQLCipher 미적용 모드 지원. feature flag로 전환 |

### CI 파이프라인 Sprint 1 최소 요구사항
```yaml
# .github/workflows/ci.yml (Sprint 1에서 구축)
jobs:
  build:
    strategy:
      matrix:
        os: [windows-latest, macos-latest]
    steps:
      - cargo build --manifest-path src-tauri/Cargo.toml
      - cargo test --manifest-path src-tauri/Cargo.toml
      - cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

---

## 4. Phase 1 완료 시 검증 항목

### M1 마일스톤 (Sprint 1): 인프라 확립
- ⬜ SQLCipher AES-256 암호화 DB CRUD 양 OS 동작
- ⬜ GitHub Actions CI 양 OS 빌드 통과
- ⬜ app.lock 시점 분리 동작
- ⬜ 4계층 백업 생성/순환 삭제
- ⬜ 무결성 검증 + 자동 복원
- ⬜ 복구 코드 발급/검증

### M2 마일스톤 (Sprint 3): 원생 관리 가능
- ⬜ 원생 등록/수정/조회/퇴교 E2E 동작
- ⬜ 초기 설정 마법사 9단계 완주
- ⬜ 글로벌 검색 원생 이름 검색 + 1클릭 이동
- ⬜ 접근성 기준선 달성 (Pretendard 18pt, WCAG AA, 44x44px)
- ⬜ 양 OS `pnpm tauri:dev` 실행 확인
- ⬜ `pnpm tauri:build` 인스톨러 생성 가능 확인 (Phase 7 사전 준비)

---

## 5. PI-05 결정 필요 알림

**일련번호 자동 채번 규칙** (PI-05)은 Sprint 2 진입 전 사용자 결정이 필수다.

- 선택지 A: 수동 입력만 (MVP 최소)
- 선택지 B: 임시 규칙 `YY+0001` 자동 채번

**Sprint 2 sprint-planner 호출 시 PI-05 결정 상태를 확인하고, 미결정이면 사용자에게 결정을 요청한다.**

---

## 6. 비용 효과성

| 항목 | 비용 | 비고 |
|------|------|------|
| 추가 Rust 의존성 | 0원 | keyring, fs2, rusqlite, argon2, zeroize — 모두 오픈소스 |
| CI 비용 | 0원 | GitHub Actions Free tier (2,000분/월, 양 OS 빌드 충분) |
| 폰트 | 0원 | Pretendard SIL Open Font License |
| shadcn/ui | 0원 | MIT License, npm 의존성 |
| DB 저장 | 0원 | 로컬 SQLite + MYBOX 무료 30GB |

**Phase 1 추가 비용: 0원**
