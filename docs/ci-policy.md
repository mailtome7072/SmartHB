# CI/CD 정책

---

## 환경 준비

### 런타임 버전
- **Node.js**: 20 (`actions/setup-node@v4`)
- **Rust**: stable (`dtolnay/rust-toolchain@stable`)
- **pnpm**: `npm install -g pnpm`으로 설치

### 빌드 파이프라인 흐름

```
sprint{n}  →  PR to develop  →  CI (cargo test + pnpm lint + pnpm build)
           →  pnpm tauri:dev 로컬 스테이징 검증
           →  PR to main  →  v* 태그 push  →  GitHub Actions deploy.yml
           →  Windows/macOS 인스톨러 자동 빌드 및 GitHub Release 첨부

hotfix/*  →  PR to main  →  CI 통과
          →  main merge  →  v* 태그 push  →  GitHub Actions deploy.yml
          →  main을 develop에 역머지
```

---

## Git 브랜치 전략

> 브랜치 구조와 배포 흐름 원칙은 [`strategy/branch-strategy.md`](../strategy/branch-strategy.md) 참조.
> Sprint/Hotfix 프로세스 상세는 [`docs/dev-process.md`](dev-process.md) 섹션 1 참조.

### 핵심 규칙

- `main` 직접 push 금지 — 반드시 PR + 리뷰 후 merge
- `develop` → `main` merge는 QA 통과 후 진행
- 긴급 패치는 **`main` 기반**으로 `hotfix/*` 브랜치를 생성하여 작업
- hotfix PR은 **`main`으로 직접** 생성 (develop 거치지 않음)
- main merge 후 반드시 `develop`에 역머지하여 동기화
- hotfix 범위 제한: 파일 3개 이하, 코드 50줄 이하, DB 변경 없음, 새 의존성 없음

---

## CI 파이프라인 (PR 체크)

PR이 `develop` 또는 `main`으로 올라오면 GitHub Actions `ci.yml`이 자동으로 실행됩니다.

### 필수 통과 조건

1. **Rust 검증** — `cargo fmt`, `cargo clippy`, `cargo test` 통과 필수
2. **프론트엔드 검증** — `pnpm tsc --noEmit`, `pnpm lint`, `pnpm build` 통과 필수
3. **Hotfix 범위 검증** (`hotfix/*` PR만) — 파일 3개 이하, 코드 50줄 이하

**GitHub Actions CI 예시 (Rust):**

```yaml
jobs:
  rust:
    runs-on: ubuntu-latest
    env:
      DATABASE_URL: sqlite::memory:
      SQLX_OFFLINE: "true"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - name: cargo fmt
        run: cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
      - name: cargo clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - name: cargo test
        run: cargo test --manifest-path src-tauri/Cargo.toml
```

**GitHub Actions CI 예시 (프론트엔드 pnpm):**

```yaml
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - name: Install pnpm
        run: npm install -g pnpm
      - name: Install dependencies
        run: pnpm install
      - name: TypeScript 타입 검사
        run: pnpm tsc --noEmit
      - name: Lint
        run: pnpm lint
      - name: Build (static export)
        run: pnpm build
```

---

## CD 파이프라인 (배포 흐름)

### develop merge 후 (스테이징 검증)

`develop` 브랜치는 별도 서버 없이 **로컬에서 앱을 직접 실행**하여 스테이징 검증합니다.

```bash
# 로컬에서 최신 코드 반영 후 검증
git pull origin develop
pnpm tauri:dev      # Tauri 앱 + Next.js dev server 동시 기동
```

### main merge 후 (프로덕션 배포)

`develop` → `main` merge 후 버전 태그를 push하면 GitHub Actions `deploy.yml`이 자동으로:

1. Windows/macOS 인스톨러 빌드
   - Windows: `.msi`, `.exe` (windows-latest)
   - macOS: `.dmg` (macos-latest, aarch64-apple-darwin)
2. GitHub Release 생성 및 아티팩트 첨부

```bash
# deploy-prod 에이전트가 수행 — 직접 실행 시 참고
git tag v{version}
git push origin v{version}
```

---

## 환경별 설정 관리

| 환경 | 설정 방법 | 비고 |
|------|----------|------|
| 로컬 개발 | `.env` 파일 | Git 미추적 (`.gitignore`) |
| GitHub Actions | GitHub Secrets | Actions에서 자동 주입 |

### GitHub Secrets 목록

| Secret 이름 | 설명 | 필수 여부 |
|------------|------|---------|
| `TAURI_PRIVATE_KEY` | Tauri 자동 업데이트 서명 프라이빗 키 | 선택 (자동 업데이트 도입 시) |
| `TAURI_KEY_PASSWORD` | 위 키의 비밀번호 | 선택 |

> `GITHUB_TOKEN`은 GitHub Actions에서 자동 제공됩니다 — GitHub Release 생성에 별도 설정 불필요.

---

## 롤백 절차

> 아래는 CI/CD 관점의 롤백 요약입니다.
> 상세 롤백 절차: `docs/dev-process.md` 섹션 6.4

### GitHub Release 롤백

이전 버전 인스톨러는 GitHub Releases 페이지에서 직접 다운로드 가능합니다.

```bash
# 이전 버전 Release 확인
gh release list

# 특정 버전 아티팩트 다운로드
gh release download v{이전_버전}
```

### Git 태그 롤백

```bash
# 잘못된 태그 삭제 (배포 전 취소 시)
git tag -d v{version}
git push origin :refs/tags/v{version}
```

> ⚠️ 이미 GitHub Release가 생성된 후에는 Actions 아티팩트가 이미 배포되었을 수 있습니다.
> 사용자에게 이전 버전 다운로드 링크를 안내하는 것이 가장 빠른 롤백 방법입니다.

---

## SQLx 오프라인 캐시

Rust CI에서 DB 없이 컴파일하려면 SQLx 오프라인 캐시가 필요합니다.

```bash
# 로컬에서 캐시 갱신 후 커밋
sqlx prepare --manifest-path src-tauri/Cargo.toml
git add src-tauri/.sqlx/
git commit -m "chore: SQLx 오프라인 캐시 갱신"
```

CI 환경변수: `SQLX_OFFLINE=true` (ci.yml에 설정됨)

---

## SQLCipher 빌드 의존성 (PRD §5.1, §5.5)

프로덕션 DB는 SQLCipher AES-256으로 암호화되며, CI에서도 동일한 빌드 구성을 요구합니다.

### 빌드 도입 방식 (Sprint 시점 ADR로 결정 — `docs/arch/`)

| 방식 | 장점 | 단점 |
|------|------|------|
| `libsqlite3-sys` `bundled-sqlcipher` feature | 시스템 의존성 없음, 모든 플랫폼 동일 | 빌드 시간 증가, OpenSSL 의존 |
| 시스템 SQLCipher 라이브러리 사용 | 빌드 빠름, 보안 패치 OS 따름 | 플랫폼별 설치 명령 필요 |

### CI 환경 시스템 의존성 (방식 2 채택 시)

```yaml
# ci.yml — Rust job 사전 단계 예시
- name: Install SQLCipher (Ubuntu)
  if: runner.os == 'Linux'
  run: sudo apt-get install -y libsqlcipher-dev

- name: Install SQLCipher (macOS)
  if: runner.os == 'macOS'
  run: brew install sqlcipher

# Windows: vcpkg 또는 bundled-sqlcipher 권장
```

### bundled-sqlcipher 방식 (방식 1 채택 시)

```toml
# src-tauri/Cargo.toml
[dependencies]
libsqlite3-sys = { version = "*", features = ["bundled-sqlcipher"] }
# OpenSSL은 자동 처리되나, Windows에서 vcpkg 환경변수 설정 필요할 수 있음
```

> CI에서 SQLCipher 빌드 실패 시 즉시 BLOCK — `harness-ci-gate` 스킬의 Data Integrity 게이트(5.1)와 연동.

### 무결성 검증 CI 단계 (선택)

데이터 보안 Phase 완료 후 CI에 다음 단계 추가 권장:

```yaml
- name: 무결성 검증 단위 테스트
  run: |
    cargo test --manifest-path src-tauri/Cargo.toml integrity_check
    cargo test --manifest-path src-tauri/Cargo.toml backup
    cargo test --manifest-path src-tauri/Cargo.toml lock
    cargo test --manifest-path src-tauri/Cargo.toml diagnosis
```
