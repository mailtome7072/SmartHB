---
name: sqlite-migration-fk-rebuild
description: "sqlx 트랜잭션 내 SQLite 테이블 재구성 + 자식 FK 처리법, WAL DB 검사 주의 (V108 code 787 교훈)"
metadata: 
  node_type: memory
  type: project
  originSessionId: f546a0a4-7177-4af3-8279-8f6a1cc98c91
---

SQLite 테이블 재구성 마이그레이션(CHECK 변경 등)에서 **자식 FK 가 있으면** sqlx 트랜잭션 안에서 code 787 로 실패한다 (V108 사례).

**Why:** sqlx 는 마이그레이션을 트랜잭션으로 감싸고 앱 연결은 `PRAGMA foreign_keys = ON`(db.rs). 부모 테이블 DROP→RENAME 재구성 시 자식 FK 가 깨진다.
- `PRAGMA foreign_keys = OFF` 는 **트랜잭션 내부에서 무시**됨 (SQLite 공식 재구성 절차는 BEGIN '밖'에서 OFF 요구 — sqlx 모델 불가).
- `PRAGMA defer_foreign_keys = ON` 도 **실패**: DROP 암묵적 DELETE 가 deferred 위반 카운터 +1, 부모 행을 `_new`(다른 이름)에 INSERT 한 시점엔 감소 안 되고 RENAME 으로도 감소 안 됨 → COMMIT 시 카운터>0. (`foreign_key_check` 는 0건이어도 카운터 잔존)

**How to apply:**
1. **NULL-복원 패턴**으로 재구성: ① 자식 FK 값을 TEMP 테이블 보존 + NULL → ② 부모 재구성(dangling 없음 → DROP/RENAME 안전) → ③ 보존값으로 복원. foreign_keys ON + 트랜잭션 내부에서 전 구간 정합 유지.
2. **빈 인메모리 테스트(`test_pool_in_memory`)는 FK 데이터 경로를 못 잡는다** — 자식 행이 없어 통과. 테이블 재구성 마이그레이션은 **시드 데이터 있는 실DB 에서 `pnpm tauri:dev` 시각 검증** 필수.
3. **SQLite WAL DB 를 복사·검사할 땐 `-wal`/`-shm` 동반 또는 앱 종료(체크포인트) 후 복사.** `app.db` 만 복사하면 미커밋 WAL 내용이 빠진 낡은 스냅샷 → `_sqlx_migrations` 오판 (V108 재번호 헛수고의 원인).
4. sqlx 0.8 은 **순서 역행 pending 마이그레이션도 적용**한다 (번호 < 적용된 max 여도 OK). 재번호 불필요. (단, 신규는 가급적 max 번호 뒤에 — [[migration numbering 정책]])

검사 도구: 로컬에 sqlite3 CLI 없음. **Strawberry Perl 의 DBD::SQLite** 로 `PRAGMA foreign_key_check` / `_sqlx_migrations` 조회 가능 ([[cipher-test-gate-trap]] 에서 설치). 한글 경로는 Perl 이 못 여니 ASCII 임시 경로로 복사 후 검사.
