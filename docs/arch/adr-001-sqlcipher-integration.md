# ADR-001: SQLCipher 도입 방식

- **상태**: Proposed
- **날짜**: 2026-05-19
- **결정자**: SmartHB 개발팀 (AI 페어 프로그래밍, sprint1 T1)

## Context (배경)

PRD §5.1, §5.5는 프로덕션 DB(`app.db`)에 SQLCipher AES-256 암호화 적용을 의무로 규정한다. 현재 `Cargo.toml`은 `sqlx 0.8 (sqlite feature)`만 의존하며, SQLCipher 통합 방식이 미정 상태다. Sprint 1 Day 1~2 PoC를 통해 도입 방식을 확정하고 양 OS(Windows + macOS) 빌드 가능성을 검증해야 한다.

본 결정은 후속 모든 데이터 작업(인증 키 관리, 백업, 무결성 검증)의 전제다. 의존성 트리 조사 결과 `sqlx-sqlite 0.8.6 → libsqlite3-sys 0.30.1` 이며, `libsqlite3-sys`가 SQLCipher feature를 제공한다.

### 제약 사항

- **양 OS 지원**: Windows 10/11 + macOS 12+ 동시 작동 (PRD §5.1)
- **단일 사용자 환경**: 50대 원장 1인 — 신규 개발자 온보딩 부담 최소화
- **CI 양 OS 빌드**: GitHub Actions `windows-latest` + `macos-latest`에서 자동 빌드
- **인터넷 없이 동작**: 빌드 결과물에 SQLCipher 코드가 포함되어야 오프라인 사용 가능 (PRD §8.3)
- **GitHub Releases 인스톨러 배포**: `.msi`/`.dmg`로 사용자 PC에 설치되므로 시스템 라이브러리 의존 시 추가 설치 안내가 필요

---

## 1단계: Weighted Decision Matrix (정량 비교)

비교 기준 가중치는 본 프로젝트 특성(데스크톱 단독 사용자 + 양 OS + 인스톨러 배포)에 맞춰 조정했다.

| 기준 | 가중치 | 선택지 A | A 점수 | 선택지 B | B 점수 |
|------|--------|---------|--------|---------|--------|
| 빌드 단순성 (CI 양 OS 통과 용이성) | 0.30 | `cargo build` 한 번에 완결, OS별 사전 설치 0개 | **5** | CI에 `brew install` / `vcpkg install` 사전 단계 필요, 환경변수(`SQLCIPHER_LIB_DIR`) 설정 분기 | **2** |
| 보안성 (라이브러리 신뢰성·유지보수) | 0.20 | SQLCipher 공식 vendored 소스, OpenSSL 정적 link → CVE 발생 시 crate 업데이트 대기 | **4** | OS 패키지 매니저가 OpenSSL 자동 패치, 시스템 SQLCipher 버전 분기 가능성 존재 | **4** |
| 번들 크기 영향 | 0.10 | vendored OpenSSL 5~8MB 추가, 정적 link로 바이너리 ~10MB↑ | **3** | 시스템 라이브러리 동적 link → 바이너리 작음 | **5** |
| 개발자 경험 (로컬 설정 부담) | 0.20 | 신규 개발자가 git clone → `cargo build`만으로 즉시 빌드 | **5** | macOS: `brew install sqlcipher`, Windows: `vcpkg install sqlcipher:x64-windows-static` 학습 필요 | **2** |
| 라이선스 호환성 | 0.20 | SQLCipher BSD/Apache + OpenSSL Apache 2.0 — MIT/Apache 호환, 상업 사용 OK | **4** | 동일 | **4** |
| **총점** |  |  | **4.4** |  | **3.1** |

- 선택지 A (`bundled-sqlcipher-vendored-openssl`) = 5×0.30 + 4×0.20 + 3×0.10 + 5×0.20 + 4×0.20 = **4.40**
- 선택지 B (시스템 sqlcipher) = 2×0.30 + 4×0.20 + 5×0.10 + 2×0.20 + 4×0.20 = **3.10**
- 차이 1.30 → A 우세 명확 (0.3 임계치 큰 폭 상회)

---

## 2단계: SWOT + Trade-off 분석 (정성 비교)

### 선택지 A: `libsqlite3-sys` + `bundled-sqlcipher-vendored-openssl` feature

- **Strengths**
  1. `cargo build` 한 번으로 양 OS 빌드 완결 — Tauri 데스크톱 앱 컨벤션과 일치
  2. CI 매트릭스 단순 (사전 설치 단계 없음) — `ubuntu`/`macos`/`windows` 러너 모두 동일 워크플로우
  3. 단일 정적 바이너리 → `.msi`/`.dmg` 인스톨러에 시스템 라이브러리 의존 없음
- **Weaknesses**
  1. OpenSSL 정적 link로 인해 CVE 발생 시 crate 업데이트 대기 필요 (보안 패치 지연 가능성)
  2. 바이너리 크기 5~10MB 증가
- **Opportunities**
  1. Tauri 2 + SQLCipher 조합의 공식 권장 패턴 — 향후 community 지원 풍부
  2. `cargo features` flag로 개발(`cipher` off) / 프로덕션(`cipher` on) 빠른 전환
- **Threats**
  1. `libsqlite3-sys` crate 자체가 deprecated되면 sqlx 호환성 깨질 위험 (낮음, 활발히 유지 중)
  2. OpenSSL 라이선스 변경 가능성 (낮음, 현재 Apache 2.0)

### 선택지 B: 시스템 sqlcipher 라이브러리

- **Strengths**
  1. 시스템 OpenSSL 보안 패치가 자동 적용됨
  2. 바이너리 크기 작음
- **Weaknesses**
  1. macOS: `brew install sqlcipher` + Windows: `vcpkg install sqlcipher` 사전 단계 필수
  2. 인스톨러 배포 시 사용자 PC에 SQLCipher 사전 설치 불가능 → **사실상 채택 불가** (PRD 인스톨러 모델과 충돌)
  3. CI 빌드에서 환경변수 분기 필요 (`SQLCIPHER_LIB_DIR`, `SQLCIPHER_INCLUDE_DIR`)
  4. 양 OS에서 SQLCipher 버전 일관성 보장 어려움
- **Opportunities**
  1. 사용자가 OS 패키지 매니저로 자체 업데이트 가능 (단, 본 앱 사용자는 50대 원장 1인 — 비현실적)
- **Threats**
  1. SQLCipher 메이저 버전 분기 시 양 OS 호환성 문제
  2. CI 매트릭스 복잡도 증가 → Sprint 1 일정 영향

### Trade-off 명시

| 선택 시 | 개선 (↑) | 저하 (↓) |
|---------|----------|----------|
| **A 선택** | 빌드 자동화, CI 단순성, 개발자 온보딩, 인스톨러 배포 자연스러움 | 번들 크기 +5~10MB, OpenSSL CVE 패치 lag |
| B 선택 | 바이너리 크기, OpenSSL 자동 패치 | CI 복잡도, 개발자 학습, **인스톨러 배포 모델 충돌** |

### Risk 식별

| 리스크 | 관련 선택지 | 영향도 | 완화 방법 |
|--------|------------|--------|----------|
| OpenSSL CVE 발생 시 패치 lag | A | 중간 | `cargo audit` CI 통합 + `libsqlite3-sys` 업데이트 모니터링. 단일 사용자 데스크톱 앱이라 공격 표면 제한적 |
| 인스톨러 사용자에게 SQLCipher 사전 설치 요구 | B | 높음 | (A 채택으로 회피) |
| 양 OS SQLCipher 버전 불일치 | B | 중간 | (A 채택으로 회피 — vendored 단일 버전 보장) |
| SQLCipher 키 메모리 노출 | 공통 | 높음 | `zeroize` crate로 사용 후 즉시 메모리 폐기 (T3에서 구현) |
| bundled 빌드 시간 증가 (~3분) | A | 낮음 | CI sccache/cargo cache로 cold build 2~3분 → warm 30초 |

---

## 3단계: Decision (결정)

**선택지 A — `libsqlite3-sys` + `bundled-sqlcipher-vendored-openssl` feature** 채택.

> 1단계 총점: A = 4.4, B = 3.1 → 차이 1.3 (임계치 0.3 상회)
> 핵심 Trade-off: A 선택으로 번들 크기 +5~10MB와 OpenSSL 패치 lag을 감수하는 대신, CI 양 OS 단순화 / 인스톨러 배포 자연스러움 / 개발자 온보딩 향상을 얻는다. 본 프로젝트는 50대 원장 1인 데스크톱 앱이라 공격 표면이 제한적이고 인스톨러 배포 모델 호환성이 우선 가치다.

### 구체 적용 방안

1. **Cargo.toml 추가**:
   ```toml
   libsqlite3-sys = { version = "0.30", features = ["bundled-sqlcipher-vendored-openssl"] }
   ```
   sqlx는 기존 `sqlite` feature를 유지하되, `libsqlite3-sys`를 명시적으로 의존시켜 SQLCipher가 통합되도록 한다.

2. **Cargo feature flag 분리** (개발/프로덕션 전환 — Windows Perl 의존 회피 효과):
   ```toml
   [features]
   default = []
   cipher = ["dep:libsqlite3-sys"]   # SQLCipher 활성화 (프로덕션)

   [dependencies]
   libsqlite3-sys = { version = "0.30", features = ["bundled-sqlcipher-vendored-openssl"], optional = true }
   ```
   - 개발 환경 (`./SmartHB-dev.db`): `cargo build` (cipher off) → 평문 SQLite, **OpenSSL 빌드 발동 안 함 → Windows에서도 즉시 성공**
   - 프로덕션 빌드: `cargo build --features cipher` → SQLCipher 활성화, vendored OpenSSL 빌드
   - **2026-05-19 PoC 검증**: cipher off 빌드 33초 성공 (Windows). cipher on 빌드는 Strawberry Perl 설치 후 사용자 환경(macOS 자택 + Windows 교습소)에서 검증 필요

   > `libsqlite3-sys` 의 `bundled-sqlcipher-vendored-openssl` 은 빌드 시점에 결정되므로 동일 바이너리에서 런타임 분기는 불가. cipher off 빌드는 평문 SQLite, cipher on 빌드는 SQLCipher 라는 **두 개의 별도 바이너리**가 산출된다 (CI matrix에서 cipher on 빌드만 배포).

3. **DB 열기 시 키 적용** (T3에서 구현):
   ```rust
   // 의사 코드 (T3 키 관리 구현 시 실제 작성)
   let pool = SqlitePoolOptions::new()
       .after_connect(|conn, _| Box::pin(async move {
           sqlx::query(&format!("PRAGMA key = '{}'", hex_key)).execute(conn).await?;
           sqlx::query("PRAGMA journal_mode=WAL").execute(conn).await?;
           Ok(())
       }))
       .connect(db_url).await?;
   ```

---

## Consequences (영향)

### 긍정적 영향

- CI 워크플로우 (`.github/workflows/ci.yml`) 양 OS 매트릭스에서 추가 설치 단계 없이 동작
- 신규 개발자 (또는 macOS 자택 환경) 온보딩 시 `git clone → cargo build` 외 추가 학습 불필요
- 인스톨러(`.msi`/`.dmg`) 배포 시 사용자 PC에 SQLCipher 사전 설치 요구 없음
- Tauri 데스크톱 앱 컨벤션과 일치 — 커뮤니티 사례 풍부

### 부정적 영향 / 주의사항

- 바이너리 크기 5~10MB 증가 (Tauri 데스크톱 앱 평균 25~35MB 기준 ~20% 증가)
- OpenSSL 정적 link로 인해 OpenSSL CVE 발생 시 `libsqlite3-sys` crate 업데이트를 기다려야 함
- Bundled 빌드 시간 cold start 2~3분 (CI sccache로 warm 30초 이내 최적화)
- **Windows + cipher feature 빌드 시 Strawberry Perl 필수** — vendored OpenSSL 이 OpenSSL 소스를 Perl 스크립트로 빌드하는데, Windows MSYS2/Git Bash 기본 Perl 환경에는 `Locale::Maketext::Simple.pm` 등 OpenSSL 빌드 필수 모듈이 누락되어 있다. **2026-05-19 PoC 1차 시도에서 실제 발생한 문제**.
  - macOS 는 system Perl 로 충분 (영향 없음)
  - Windows 개발자: `https://strawberryperl.com/` 에서 64-bit MSI 1회 설치 (~10분)
  - CI Windows runner: `windows-latest` 에 Strawberry Perl 사전 설치되어 있음 (별도 step 불필요)
  - `docs/setup-guide.md` 5-B 섹션에 안내 추가

### 후속 액션

- **T3 키 관리** 구현 시: 사용자 비밀번호 PBKDF2 600K iter 키 유도 → `PRAGMA key = X'{hex_key}'` 적용 → `zeroize`로 메모리 즉시 폐기
- **CI 양 OS 빌드 검증** (T11): bundled 빌드 시간 측정 후 sccache 캐싱 설정 검토 (`.github/workflows/ci.yml` 변경 시 Forbidden Area 허가 필요)
- **`cargo audit`** CI 단계 추가 검토: OpenSSL CVE 자동 감지 (T11 또는 후속 sprint)
- **개발 환경 cipher feature**: T11 또는 후속 sprint에서 dev/prod 빌드 시간 비교 후 default cipher on/off 결정

### 검증 결과 (PoC, 2026-05-19)

- ✅ Windows 로컬 `cargo build` (cipher off) 성공 — 33초 (warm cache)
- ⬜ Windows 로컬 `cargo build --features cipher` — Strawberry Perl 설치 후 사용자 환경에서 검증 예정
- ⬜ macOS 로컬 `cargo build --features cipher` — 사용자 자택 Mac에서 검증 예정
- ⬜ sqlx + SQLCipher CRUD 동작 확인 — T3 (키 관리) 구현 후 검증
- ⬜ `PRAGMA cipher_version` 응답 확인 (`4.x.x` 형식) — T3 구현 후 검증

### 변경 이력

- 2026-05-19 PoC 1차 시도 (cipher feature 무관 항상 활성화) → Windows Perl 모듈 누락으로 실패 → optional dependency 전환으로 해결
- 2026-05-19 cipher feature off 빌드 검증 완료 → ADR 본문 Decision 항목 보강 (feature flag 패턴 수정)
