---
name: data-loss-recovery-method
description: "SmartHB 암호화 DB 오프라인 복구법 — salt.bin+PIN으로 PBKDF2 키 재현, rusqlite bundled-sqlcipher로 복호화·행수검사·원자적 교체. 데이터 소실 재발 시 참조"
metadata:
  node_type: memory
  type: reference
---

2026-07-22 데이터 소실 사고에서 검증된 복구 절차. 상세: `docs/incidents/2026-07-22-data-loss-rca.md`.

## 키 재현 (앱과 동일)
- DB 키 = `PBKDF2-HMAC-SHA256(PIN, salt.bin, 600000회) → 32바이트`. 결정적·크로스OS 동일(재현성 테스트 보장).
- salt.bin: 클라우드폴더 `smarthb/salt.bin`(32바이트, 동기화됨). PIN: 6자리 (**원장 확인 — 메모리에 저장 금지**).
- SQLCipher 적용: `PRAGMA key = "x'<64hex>'"`. 앱은 rusqlite 0.32 / libsqlite3-sys 0.30 `bundled-sqlcipher-vendored-openssl`(SQLCipher 4.x 기본). 오프라인 도구도 **동일 crate 버전**으로 빌드해야 cipher 설정 일치.

## 절차
1. **복사본에서만 작업** — 라이브 클라우드 폴더 직접 작업 금지(도구가 -wal 생성 시 torn-sync). 원본 보존 아카이브 + 작업본 2부.
2. 오프라인 도구(scratchpad에 crate 버전 맞춰 빌드)로 salt.bin+PIN → 키 재현 → 폴더 내 전체 `.db` 복호화 → 6개 테이블(students / student_schedules / regular_attendances / makeup_attendances / bills / payments) 행수 카운트 → 온전본 식별. 판별: 빈 DB=229KB(시드만, 도메인 0행) vs 정상=499KB급. `quick_check ok`만으로 판단 금지(빈 DB도 ok).
3. 복원: MYBOX(클라우드) 동기화 일시중지 → 빈 app.db 보존 이동 → `app.db-wal`/`-shm` 제거 → 온전본을 `app.db`로 원자적 rename → 복호화 재검증 → 동기화 재개 → 키체인에 키 있는 원 PC에서 오픈.

## 절대 금지
- 사고 중 **"최초 설정 / 비밀번호 재설정"** 금지 → `set_password`가 새 salt 생성해 클라우드 salt.bin 덮어씀 = 기존 DB 영구 복호화 불가.
- 2번째 PC는 로컬 키체인에 키가 없어 로그인 불가(설계 한계 — Sprint 23 B에서 키 채택 경로로 해소 예정). 복구는 키 보유한 원 PC 또는 오프라인 도구로.

관련: [[sprint-next-session]], [[ntfs-power-loss-pattern]], [[keyring-v3-features-trap]]
