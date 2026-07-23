# Sprint 23 코드 리뷰

> 대상: Sprint 23 (94171fa~9fdad65) — 2026-07-22 프로덕션 데이터 소실 사고 재발방지 (ADR-012 A안)
> 리뷰 일자: 2026-07-23
> 자동 검증 결과: cargo test 478 passed / clippy clean / pnpm tsc clean / pnpm lint clean / pnpm build 성공

---

## 발견 사항 (5건)

### 🟡 M-01 — pool() fast-path TOCTOU + 장기 작업 중 graceful close 경합 (Medium, 수용)

**위치**: `src-tauri/src/commands/db.rs:pool()`, `close_for_idle()`

**설명**: `pool()` 는 `RECONNECT_LOCK` 없이 `current_open_pool()`(fast-path)을 먼저 확인한다. 이후 쿼리 실행 전에 `close_for_idle()` 가 `RECONNECT_LOCK` 을 획득하여 pool 을 닫으면, 호출자는 이미 닫힌 `SqlitePool` 참조로 쿼리를 실행해 "pool is closed" 오류를 받는다.

두 번째 경로: Tauri 명령이 5분 이상 실행되는 동안 `pool().await?` 를 여러 번 호출하면, 5분 경과 후 유휴 감지 태스크가 graceful close 를 시도한다. `max_connections=1` 에서 커넥션이 쿼리 간 반환될 때 close 가 완료되어, 다음 쿼리에서 "pool is closed" 오류가 발생할 수 있다.

**실제 위험도**: 낮음. 5분 임계 + 1분 점검 주기 조합에서 정상 원장 작업(CRUD 수 초 이내)이 이 윈도우를 맞출 확률이 매우 낮다. 오류 발생 시에도 데이터 손상이 아닌 UI 오류로 나타나며 재시도로 해소된다.

**대응 계획**: R150으로 등록. 발생 빈도가 관찰되면 `LAST_ACTIVITY_SECS`를 쿼리 레벨(after_connect 또는 after_query hook)에서 갱신하거나, 장기 명령이 시작 시 타이머를 스스로 갱신하는 패턴 도입 검토.

---

### 🟡 M-02 — LockScreen.tsx 채택 오류 분기 한국어 문자열 매칭 (Medium, 수용)

**위치**: `src/components/LockScreen.tsx:120` (`adoptMsg.includes('이미 이 PC')`)

**설명**: `try_adopt_key` 에서 "이미 이 PC에 인증 키가 있습니다" 오류가 반환되면 채택 시도가 아닌 원래 잠금 오류를 표시한다. 이 분기 판정을 한국어 부분 문자열로 처리하므로, 백엔드의 오류 메시지가 바뀌면 분기가 조용히 오동작할 수 있다.

**실제 위험도**: 낮음. 동일 저장소라 백엔드·프론트 동시 변경이 이루어지며, 해당 오류 경로는 "같은 PC 재로그인" 케이스로 발생 빈도가 낮다.

**대응 계획**: R151로 등록. 다음 auth.rs 수정 스프린트에서 백엔드가 구조화된 오류 코드(`AlreadyHasKey` 등)를 반환하도록 개선 고려.

---

### 🔵 L-01 — eprintln! 프로덕션 디버그 로그 (Low, 기록)

**위치**: `db.rs`, `startup.rs`, `lock.rs`, `integrity.rs` 전반

**설명**: 유휴 close, 재연결, 락 획득, 복원 등 주요 이벤트를 `eprintln!`으로 기록한다. Tauri 프로덕션 빌드에서 stderr 가 캡처되지 않으면 이 로그가 관찰 불가능하다. Sprint 23 이 데이터 안전 강화 스프린트임을 감안하면, 관찰 가능성을 높이는 것이 중요하다.

**대응 계획**: 향후 `tauri-plugin-log` 또는 `tracing` 도입 시 전환 고려. 현재는 개발 시 유용하므로 제거하지 않음.

---

### 🔵 L-02 — warn_if_drastic_shrink 비차단 설계 (Low, 기록)

**위치**: `src-tauri/src/commands/backup.rs:warn_if_drastic_shrink()`

**설명**: 소스 DB 가 최신 백업의 50% 미만으로 급격히 줄었을 때 `eprintln!` 경고만 출력하고 백업을 계속 진행한다. 의도적 설계(대량 삭제 등 정상 케이스 가능)이나, L-01 과 동일하게 프로덕션에서 이 경고가 관찰되지 않을 수 있다.

**대응 계획**: 수용. 원생 0명 백업 거부(H2)와 결합하면 실질적 위험은 낮다.

---

### 🔵 L-03 — LockWarning.tsx A113 상수 쌍 동기화 확인됨 (Low, 이상 없음)

**위치**: `lock.rs:STALE_THRESHOLD_SECONDS = 86400` / `LockWarning.tsx:STALE_THRESHOLD_SECONDS = 86400`

**설명**: T8에서 STALE 값을 변경하지 않았으므로 A113 프론트 동기화 불필요라는 진행 중 판단을 검증했다. 양쪽 모두 86400 으로 일치함. 이상 없음.

---

## 영역별 추가 점검

### 보안 (backend.md Critical)

- SQL 인젝션: 모든 쿼리가 `bind()` 파라미터 또는 고정 SQL 조각 사용. `format!` 보간은 `pragma_key_sql(hex_key)` 패턴이나 hex 키는 `to_hex()` 출력으로 알파뉴메릭 제한됨. 이상 없음.
- 하드코딩 시크릿: 없음. 키는 OS Keychain / 메모리 캐시 경유.
- Tauri 권한 과다: `capabilities/default.json` 변경 없음. 이상 없음.
- SQLCipher 키 Keychain 외부 저장: `try_adopt_key` 에서 키체인 저장 전 `verify_key_opens_db` 검증 후 저장. 이상 없음.

### 보안 (backend.md High)

- `unwrap()` 남용: 프로덕션 코드에서 `expect("POOL rwlock poisoned")` 패턴이 3곳 있으나, RwLock poison 은 선행 패닉을 의미하는 치명적 상태로 `expect` 가 적절한 패턴이다. `mark_shutdown_for_restart` 는 `if let Ok(...)` 로 내성적으로 처리. 이상 없음.
- 마이그레이션 없는 스키마 변경: DB 마이그레이션 없음(V312 유지). 이상 없음.
- 새 쿼리 단위 테스트 누락: `configure_connection`, `close_for_idle` + 재연결, `create_if_missing` 가드, set_password salt 가드, `try_adopt_key` PIN 검증, 백업 rotation 보존 등 주요 경로 모두 단위 테스트 커버. 이상 없음.

### 프론트엔드 (frontend.md Critical/High)

- XSS: `dangerouslySetInnerHTML` 사용 없음. 이상 없음.
- `invoke()` 직접 호출: `tryAdoptKey` 래퍼가 `src/lib/tauri/index.ts` 에 추가됨. LockScreen.tsx 는 래퍼 사용. 이상 없음.
- TypeScript `any` 남용: 없음. `AuthStatus`, `StartupResult` 타입 명시.
- `'use client'` 과다: 기존 LockScreen.tsx 는 이미 클라이언트 컴포넌트.
- 인증 토큰 localStorage 저장: 없음. 이상 없음.

### AI 생성 코드 추가 체크

- T6 db.rs: `RECONNECT_LOCK` double-check 패턴(lock 전 확인 → lock 후 재확인)이 올바르게 구현됨. `POOL_SHUTDOWN` 래치가 재연결 봉쇄를 일관되게 적용.
- T3 integrity.rs: `precheck_restore_candidate`의 cipher-off 빌드 행동(`run_pragma_check` → 에러 반환 → fail-soft)이 주석에 명시됨.
- T8 lock.rs: `device_id_was_lost()` 가 첫 실행 NotFound 를 유실 아님으로 올바르게 구분. 2-PC 상호 배제를 보존하는 중요 로직.

---

## 결론

Critical/High 이슈 없음. Medium 2건은 발생 확률이 낮고 데이터 손상 위험이 없어 수용 결정. Sprint 23 의 핵심 재발방지 구현(after_connect 훅, create_if_missing 가드, 유휴 close + 재연결, 복원 다계층 폴백, 백업 소스 검증, salt 하드 가드, device.id 유실 처리)은 모두 올바르게 구현되었으며 단위 테스트로 커버됨.
