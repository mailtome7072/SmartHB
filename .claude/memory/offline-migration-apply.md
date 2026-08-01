---
name: offline-migration-apply
description: "앱이 못 여는(마이그레이션 hang 등) 상황에서 대기 마이그레이션을 오프라인으로 선적용+재암호화하는 절차. [[data-loss-recovery-method]] 자매편"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 2de8b3ba-b8a7-4d3d-bec8-87adf8a9081b
  modified: 2026-08-01T11:42:37.070Z
---

암호화 프로덕션 DB에 **대기 중 마이그레이션을 오프라인으로 대신 적용**하는 방법 (앱이 그 마이그레이션에서 hang 할 때 우회용). 복사본에서만 작업, 원본은 안전본 확보 후 원자적 교체.

**키 유도** (모든 오프라인 접근 공통): `키 = PBKDF2-HMAC-SHA256(PIN, salt.bin(32B), 600000회) → 32B` → SQLCipher `PRAGMA key = "x'<hex>'"`. (PIN=000000, salt=클라우드폴더 salt.bin. auth.rs `derive_key` 참조)

**절차** (Mac 스크래치패드에서 검증됨):
1. 실 `app.db`+`salt.bin` 복사(원본 무수정).
2. 별도 Rust 프로젝트: `sqlx`(sqlite,migrate,macros) + `libsqlite3-sys{bundled-sqlcipher-vendored-openssl}` 동시 의존 → sqlx가 SQLCipher 링크. after_connect 훅에서 `PRAGMA key` 먼저 적용 후 WAL 등. `sqlx::migrate!("./migrations")`(앱 migrations 복사, 상대경로 필수 — 절대경로는 매크로가 거부).run → 대기분만 적용. **sqlx 마이그레이터라 _sqlx_migrations 체크섬이 정확**(수동 INSERT 금지 — 체크섬 불일치 시 다음 실행 에러).
3. `PRAGMA wal_checkpoint(TRUNCATE)` + pool.close → 단일 파일화, 빈 -wal/-shm 제거.
4. 검증: 키로 열림 + `SELECT MAX(version)` = 목표 + `PRAGMA quick_check`=ok + 행수 무손실.
5. 교체: 안전본 확보 → MyBox의 `app.db-wal/-shm/app.lock` 제거 → 새 파일을 임시명 복사 후 `mv -f`로 원자적 교체 → 해시 재확인.
6. 양 PC 모두 MyBox 동기화 완료 후 실행(둘 다 새 version). 앱 종료 상태에서 교체.

관련: [[data-loss-recovery-method]](복구), [[v151-migration-hang]](이 절차를 쓴 사고), [[deploy-version-three-files]].
