---
name: cipher-test-gate-trap
description: cargo test --features cipher 가 컴파일 실패하는 잠재 트랩 — test_pool_in_memory 게이트 + Strawberry Perl
metadata: 
  node_type: memory
  type: project
  originSessionId: f546a0a4-7177-4af3-8279-8f6a1cc98c91
---

`cargo test --features cipher` 는 두 가지 함정으로 막힌다. 둘 다 CI 가 cipher 로 **빌드만**(`tauri build --features cipher`) 하고 테스트는 cipher-off 라서 오래 미발견됐다.

**Why:** cipher 로 도메인 테스트를 로컬 검증하려 할 때 둘 다 걸린다.

**How to apply:**
1. **Strawberry Perl 필수** — `--features cipher` 는 vendored OpenSSL 을 소스 빌드한다. 없으면 `Locale::Maketext::Simple` 누락으로 실패. `winget install StrawberryPerl.StrawberryPerl` 후 PowerShell 에서 `$env:PATH = "C:\Strawberry\c\bin;C:\Strawberry\perl\bin;" + $env:PATH` 로 cargo 실행. Bash(git-bash) 는 msys perl 을 먼저 잡으니 PowerShell 사용. ([[keyring-v3-features-trap]] 처럼 Windows 빌드 트랩)
2. **테스트 모듈 cipher 게이트** — `db::test_pool_in_memory()` 는 `#[cfg(all(test, not(feature = "cipher")))]` (인메모리 DB 는 SQLCipher 적용 불가). 이를 쓰는 `mod tests` 는 **반드시 동일 게이트**여야 cipher-test 가 컴파일된다. 신규 도메인 모듈에 테스트 추가 시 `#[cfg(test)]` 가 아니라 `#[cfg(all(test, not(feature = "cipher")))]` 로 선언할 것. (2026-05-27 기준 11개 모듈 전부 정합 완료)

검증 순서: `cargo build --features cipher`(=CI 경로) → `cargo test --lib --features cipher` → `cargo clippy --lib --features cipher -- -D warnings`.
