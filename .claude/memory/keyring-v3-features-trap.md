---
name: keyring-v3-features-trap
description: keyring crate v3 는 OS native backend 가 features 로 분리됨 — 명시 활성 없으면 silent set fail + 항상 NoEntry 반환
metadata: 
  node_type: memory
  type: project
  originSessionId: 39e31ca0-8b92-444b-9a34-e09f9c9fb022
---

Rust `keyring` crate **v3.x 부터 OS native backend (windows-native, apple-native, linux-native-sync-persistent) 가 features 로 분리**됐다. `keyring = "3"` 으로만 적으면 default-features 만 활성 = **backend 미연결 상태**.

이 상태에서:
- `Entry::set_password()` 는 silent `Ok(())` 반환 (아무 데도 저장 안 함)
- `Entry::get_password()` 는 항상 `Err(NoEntry)` 반환
- 컴파일/clippy 모두 통과, 런타임 에러도 명시적이지 않음 — **매우 비자명한 silent fail**

**진단 방법**: `Cargo.lock` 의 keyring deps 확인. `log` + `zeroize` 2개뿐이면 backend 미연결. 정상이면 `windows-sys` (Windows) 또는 `security-framework` (macOS) 가 추가로 보여야 한다.

**Why:** 본 프로젝트 Sprint 1 T3 (`8e17324`, 2026-05-04 추정) 에서 도입 후, cargo test 가 `derive_key` / `generate_salt` 순수 함수만 검증해서 keyring round-trip 누락이 표면화되지 않음. 2026-05-21 마법사 첫 통합 검증에서 사용자가 발견 — 비밀번호 설정 → 곧바로 verify 가 "Salt 항목이 존재하지 않습니다" 로 실패.

**How to apply:**
- 본 프로젝트 cross-platform (Windows + macOS) 정책상 features 명시 필수:
  ```toml
  keyring = { version = "3", features = ["apple-native", "windows-native"] }
  ```
- 신규 keyring 도입 또는 버전 업 시 항상 `Cargo.lock` 의 deps 를 확인할 것
- keyring 관련 통합 테스트 추가 가치 있음 (현재 누락) — 단 OS keychain 접근이라 CI 에서 까다로움. 대안: 통합 e2e 테스트에서 set→get round-trip 검증

## 해결 커밋
`08e9629` (2026-05-21) — `Cargo.toml` 에 features 명시. `Cargo.lock` 에 `windows-sys 0.60.2` + `security-framework` 추가 확인됨.

## 관련
- [[ntfs-power-loss-pattern]] — 같은 진단 세션에서 발견된 별개 사고 (config.json + app.lock NULL 손상)
- [[workflow-no-pr]] — 단일 개발자 정책상 develop 직접 커밋
