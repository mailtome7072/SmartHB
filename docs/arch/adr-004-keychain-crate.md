# ADR-004: OS Keychain crate 선정

- **상태**: Proposed
- **날짜**: 2026-05-19
- **결정자**: SmartHB 개발팀 (Sprint 1 T3)

## Context (배경)

PRD §5.5 는 SQLCipher 암호화 키를 **OS Keychain(macOS) / Credential Manager(Windows)** 에 보관할 것을 의무로 규정한다. 사용자 비밀번호로부터 PBKDF2 로 유도한 32바이트 키를 OS 보안 저장소에 저장하여, 앱 메모리에서는 사용 직후 즉시 zeroize 한다.

본 결정은 다음을 좌우한다:
- 양 OS (Windows 10/11 + macOS 12+) 통합 코드 경로
- 키 저장/조회 API 의 추상화 수준 (네이티브 직접 호출 vs 통합 추상화)
- 후속 T4(인증) / T5(복구 코드) / T7(백업 복원) 의 키 관리 의존성

### 제약 사항

- 양 OS 동시 지원 (PRD §5.1)
- 키 메모리 노출 최소화 (zeroize 별도 적용, 본 ADR 범위 밖)
- 단일 사용자 데스크톱 앱 — 멀티 사용자 시나리오 미고려
- 인터넷 없이 동작 — 외부 키 서비스(AWS KMS, HashiCorp Vault) 사용 불가

---

## 1단계: Weighted Decision Matrix

| 기준 | 가중치 | 선택지 A | A 점수 | 선택지 B | B 점수 |
|------|--------|---------|--------|---------|--------|
| 빌드 단순성 (양 OS 통합 코드) | 0.25 | 단일 `keyring::Entry::new(...)` API 호출만으로 양 OS 동작 | **5** | macOS: `security-framework`, Windows: `windows`/`keyring-rs` 별도 호출 + `cfg(target_os)` 분기 필요 | **2** |
| 양 OS 호환성 | 0.25 | Windows Credential Manager + macOS Keychain + Linux Secret Service 모두 지원 (Linux는 보너스) | **5** | macOS+Windows 명시적 처리, Linux 별도 추가 필요 | **4** |
| 추상화 수준 적절성 | 0.15 | 양 OS 공통 API 만 추상화, OS별 특수 기능(예: macOS Touch ID prompt)은 우회 필요 | **4** | 네이티브 API 직접 호출 — 모든 OS별 특수 기능 활용 가능 | **5** |
| 의존성 무게 | 0.10 | `keyring` v3.x 자체는 가벼움, 내부적으로 `security-framework`/`windows` crate 사용 | **4** | 두 crate 별도 도입 — 의존성 트리 증가 | **3** |
| 유지보수성 (단일 코드 경로) | 0.15 | 단일 코드 경로, 신규 개발자 학습 곡선 낮음 | **5** | OS별 분기 코드 + 두 API 학습 필요 | **2** |
| 보안성 (네이티브 보안 기능 접근) | 0.10 | 표준 키체인 항목 CRUD 만 지원. OS별 attribute(예: Touch ID 요구) 직접 설정 어려움 | **4** | 네이티브 API 로 모든 보안 attribute 설정 가능 | **5** |
| **총점** |  |  | **4.65** |  | **3.35** |

- A 총점 = 5×0.25 + 5×0.25 + 4×0.15 + 4×0.10 + 5×0.15 + 4×0.10 = 1.25 + 1.25 + 0.60 + 0.40 + 0.75 + 0.40 = **4.65**
- B 총점 = 2×0.25 + 4×0.25 + 5×0.15 + 3×0.10 + 2×0.15 + 5×0.10 = 0.50 + 1.00 + 0.75 + 0.30 + 0.30 + 0.50 = **3.35**
- 차이 1.30 → A 우세 명확 (임계치 0.3 상회)

---

## 2단계: SWOT + Trade-off

### 선택지 A: `keyring` crate v3.x

- **Strengths**
  1. 단일 API 로 macOS Keychain + Windows Credential Manager + Linux Secret Service 모두 지원 → 코드 분기 0건
  2. 활발히 유지 (2026 기준 v3.x 안정판), 양 OS native crate 의존을 자체적으로 격리
  3. 직관적 `Entry::new(service, user).get_password() / set_password()` API
- **Weaknesses**
  1. OS별 특수 기능(macOS Touch ID prompt 통합, Windows DPAPI 직접 사용)에 접근 어려움
  2. 키 값 전달이 `String` 기반 — 바이너리 키를 hex 인코딩으로 변환 필요 (사소함)
- **Opportunities**
  1. Tauri 데스크톱 앱 컨벤션과 일치 — community 사례 풍부
  2. 향후 Linux 지원이 필요해지면 추가 작업 없이 동작
- **Threats**
  1. crate 자체 deprecate 시 마이그레이션 부담 (현재는 활발, 위험 낮음)
  2. v2 → v3 메이저 변경 시 호환성 확인 필요

### 선택지 B: `security-framework` + `windows-credential` 개별 사용

- **Strengths**
  1. 네이티브 API 직접 호출 — 모든 OS 보안 기능 활용 가능 (Touch ID, DPAPI 등)
  2. 의존성 자체는 OS별로 격리되어 정밀 제어 가능
- **Weaknesses**
  1. `#[cfg(target_os = "macos")]` / `#[cfg(target_os = "windows")]` 분기 코드 필요 — 유지보수 부담
  2. 두 별개 API 학습 — 신규 개발자 온보딩 비용 증가
  3. macOS Linux 환경 미지원 (보조 개발 환경에서 빌드 실패)
- **Opportunities**
  1. 향후 macOS Touch ID 또는 Windows Hello 통합 시 직접 호출 가능
- **Threats**
  1. 양 OS 동작 차이로 인한 미묘한 버그 (예: macOS Keychain Access Group, Windows Credential persistence flag)
  2. macOS `security-framework` 와 `windows-credential` 의 메이저 버전 분기 시 동시 마이그레이션 필요

### Trade-off 명시

| 선택 시 | 개선 (↑) | 저하 (↓) |
|---------|----------|----------|
| **A 선택** | 단일 코드 경로, 양 OS 빌드 자동화, 신규 개발자 온보딩 | OS별 특수 보안 기능(Touch ID 등) 직접 활용 어려움 |
| B 선택 | 네이티브 보안 기능 풀 액세스 | 분기 코드 복잡도, 양 OS 유지 비용, 빌드 시간 |

### Risk 식별

| 리스크 | 관련 선택지 | 영향도 | 완화 방법 |
|--------|------------|--------|----------|
| `keyring` crate deprecate | A | 낮음 | 현재 v3.x 활발 유지. 발생 시 B 안으로 마이그레이션 (분기 코드 일회 작성) |
| OS별 미묘한 동작 차이 (락 충돌 등) | A | 낮음 | T11 단위 테스트 + 양 OS CI 빌드 검증 |
| 키 저장/조회 실패 시 데이터 영구 손실 | 공통 | 높음 | PI-07 복구 코드로 fallback (Sprint 1 T5 구현) |
| 키 메모리 노출 | 공통 | 높음 | `zeroize` crate 로 별도 처리 (본 ADR 범위 밖, T3 본 작업에 포함) |
| Touch ID/Windows Hello 향후 도입 필요 | A | 중간 | A로 시작 → 필요 시 추가 native crate 보강 (단계적 도입) |

---

## 3단계: Decision

**선택지 A — `keyring` crate v3.x** 채택.

> 1단계 총점: A = 4.65, B = 3.35 → 차이 1.30 (임계치 0.3 큰 폭 상회)
> 핵심 Trade-off: A 채택으로 OS별 특수 보안 기능 직접 활용을 일부 포기하는 대신, 단일 코드 경로 / 양 OS 빌드 자동화 / 신규 개발자 온보딩 / 유지보수성을 얻는다. PRD 단일 사용자 + 비밀번호 인증 모델에서는 키체인 표준 CRUD 만으로 충분하며, Touch ID/Windows Hello 가 향후 요구되면 단계적으로 native crate 를 보강할 수 있다.

### 구체 적용 방안

1. **Cargo.toml 추가**:
   ```toml
   keyring = "3"
   pbkdf2 = "0.12"
   sha2 = "0.10"
   hmac = "0.12"
   zeroize = { version = "1.7", features = ["zeroize_derive"] }
   rand = "0.8"
   hex = "0.4"
   ```

2. **키체인 항목 구조**:
   ```rust
   const KEYRING_SERVICE: &str = "SmartHB";
   const KEYRING_USER_KEY: &str = "db_encryption_key";  // SQLCipher AES-256 키
   ```
   복구 코드 해시는 별도 항목으로 저장 (T5 에서 결정 — Keychain 또는 DB).

3. **키 유도 함수** (`src-tauri/src/commands/auth.rs`):
   ```rust
   const PBKDF2_ITERATIONS: u32 = 600_000;  // OWASP 2024
   const SALT_LEN: usize = 32;
   const KEY_LEN: usize = 32;  // AES-256

   pub fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> DerivedKey { ... }
   ```

4. **zeroize 통합**:
   ```rust
   #[derive(ZeroizeOnDrop)]
   pub struct DerivedKey([u8; KEY_LEN]);

   impl std::fmt::Debug for DerivedKey {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           f.write_str("DerivedKey([REDACTED])")
       }
   }
   ```

---

## Consequences

### 긍정적 영향

- 양 OS 빌드 자동화 — `cargo build` 한 번으로 동작
- 신규 개발자 (또는 macOS 자택 환경) 온보딩 시 키체인 분기 학습 불필요
- 후속 T4/T5/T7 에서 동일 API 사용 — 일관성↑
- Linux 빌드 환경에서도 동작 (CI 보조 검증 가능)

### 부정적 영향 / 주의사항

- OS별 특수 보안 기능(Touch ID, Windows Hello, Touch ID Watch unlock 등) 통합 시 추가 native crate 의존성 필요
- `keyring` v3.x 가 메이저 변경되면 API 마이그레이션 부담 — `Cargo.lock` 으로 버전 고정 + `cargo audit` 모니터링
- 키 값이 hex 인코딩 String 으로 전달됨 — 메모리 노출 시점이 일시적이지만 존재. `zeroize` 로 즉시 폐기 필수

### 후속 액션

- **T4 (인증 IPC)**: 본 ADR 의 `derive_key` + `store_key_in_keyring` / `retrieve_key_from_keyring` 사용
- **T5 (복구 코드)**: 복구 코드 해시 저장 위치 결정 (Keychain 별도 항목 vs DB) — T5 진입 시 결정
- **T11 (CI 검증)**: 양 OS GitHub Actions 매트릭스에서 Keychain 동작 확인 (단위 테스트는 OS Keychain 미실행 환경에서 skip 처리 필요)
- **Touch ID 향후 통합**: PRD §5.5 가 "OS 생체인증" 을 옵션으로 명시 — Sprint 1 범위 밖, 별도 ADR 작성 필요 시점에 진행
