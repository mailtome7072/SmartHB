# Sprint 17 코드 리뷰

> 대상: Sprint 17 (82c385a) — DB 안전성 잔여 수정 + 정책 간소화
> 리뷰 일자: 2026-06-30
> 자동 검증 결과: cargo test 411 passed / clippy clean / cargo check --features cipher OK / pnpm lint clean / pnpm tsc clean / pnpm build 성공

## 발견 사항 (7건)

### F1 — Stale lock 5분 후 auto-acquire (High, 미조치 — 검토 필요)

- 위치: `src-tauri/src/commands/lock.rs:39, 155-168`
- 실패 시나리오: `STALE_THRESHOLD_SECONDS = 300`. `LockInfo::new_for_self()`는 lock 획득 시 `last_heartbeat = Utc::now()`를 1회만 기록한다. heartbeat 제거(T5) 이후 이 값은 갱신되지 않는다. 앱을 5분 이상 사용하면 `is_stale()` → `seconds_since_heartbeat() >= 300`이 true가 된다. 다른 PC에서 앱 실행 시 `acquire_lock_atomic`이 "stale lock 자동 점유"(lock.rs:313)로 락을 획득 → 두 PC가 동시에 같은 SQLite 파일에 쓰게 되어 WAL 손상 위험.
- 근거: `lock.rs` 직접 확인. `is_stale()`, `new_for_self()`, stale 자동 점유 로직(313행) 모두 코드에 존재. heartbeat 제거 후 갱신 메커니즘 없음.
- 조치 방안: (a) `STALE_THRESHOLD_SECONDS`를 충분히 큰 값(예: 86400, 24h)으로 상향 또는 무효화, (b) 락 파일을 mtime 기반에서 config.json 기반 "마지막 사용 PC" 판정으로 전환 (sprint17 계획 T5에 언급된 대안), (c) heartbeat 재도입

### F2 — rollback 파일명 초 단위 충돌 (High, 미조치 — 검토 필요)

- 위치: `src-tauri/src/commands/integrity.rs:217-218` (`generate_rollback_filename`)
- 실패 시나리오: `generate_rollback_filename(Utc::now())` 결과는 `rollback_YYYYMMDD_HHMMSS.db` (초 단위 정밀도, 테스트 `rollback_20260519_153045.db` 확인). `auto_restore_with_retry` retry 루프에서 두 번의 `restore_from_path_sync` 호출이 1초 이내에 실행되면 동일한 rollback 경로를 생성한다. Windows `std::fs::rename`은 대상이 존재해도 덮어쓰므로 iteration-1 rollback(원본 app.db)이 iteration-2의 실패 복원본으로 무음 덮어쓰여진다. 원본 DB가 영구 소실.
- 조치: rollback 파일명에 loop 인덱스 또는 나노초 컴포넌트 추가.

### F3 — WAL 체크포인트 실패 시 pool.close() 미실행 (Medium, 미조치)

- 위치: `src-tauri/src/commands/setup.rs:244-286`
- 실패 시나리오: WAL checkpoint `?` 전파로 조기 반환 시 step 6 `pool.close().await`(281행)에 도달하지 못한다. Pool이 구 경로를 계속 참조하는 상태에서 프론트엔드가 IPC 호출을 계속하면 사용자가 "폴더 변경 실패"로 인식했음에도 구 DB에 데이터가 기록될 수 있다. 더불어 WAL checkpoint는 active reader 존재 시 `SQLITE_BUSY`로 실패할 수 있어 정상 운영 중 hourly backup task와 경쟁 시 발생 가능.
- 조치: early return 전 pool.close() 호출, 또는 WAL checkpoint를 1-3회 retry 후 실패 처리.

### F4 — cleanup_stale_tmp_backups() async 경로에서 blocking I/O (Medium, 미조치)

- 위치: `src-tauri/src/startup.rs:207`, `src-tauri/src/commands/backup.rs:458-471`
- 실패 시나리오: `run_startup`은 `async fn`. `cleanup_stale_tmp_backups()`는 4개 백업 레이어에 대해 `std::fs::read_dir` + `std::fs::remove_file`을 동기로 실행한다. MYBOX 폴더가 느린 네트워크 경로에 있을 경우 tokio executor thread를 수백 ms 동안 점유. `spawn_blocking` 없이 호출.
- 조치: `tokio::task::spawn_blocking(backup::cleanup_stale_tmp_backups).await` 또는 `let _ = tokio::task::spawn_blocking(...)` (결과 불필요 시).

### F5 — auto_restore_with_retry 단위 테스트 누락 (Medium, 미조치)

- 위치: `src-tauri/src/commands/integrity.rs:256-293`
- 실패 시나리오: `auto_restore_with_retry`는 startup 손상 복원 경로의 핵심 함수(3회 retry, quick_check 재검증)로, sprint17 계획 T3에 "단위 테스트 필수"가 명시됐으나 `#[cfg(test)]` 블록에 해당 테스트가 없다. 회귀 발생 시 자동 감지 불가.
- 규칙: `backend.md` — "락 메커니즘, 백업, 무결성 검증, 자가 진단 로직은 독립 모듈로 분리하여 테스트"

### F6 — 최종 에러 메시지 혼용 (Low, ROADMAP 이연)

- 위치: `src-tauri/src/commands/integrity.rs:289-292`
- 내용: 루프 소진 시 "복원할 수 있는 백업이 없습니다" 메시지는 `candidates.is_empty()` 분기(261행)와 동일 문구. 두 상황(백업 없음 vs. 후보 3개 모두 손상)을 같은 문자열로 노출하여 사용자 혼란 및 지원 진단 어려움.

### F7 — cleanup_stale_tmp_backups .tmp 확장자 필터 과도 범위 (Low, ROADMAP 이연)

- 위치: `src-tauri/src/commands/backup.rs:466`
- 내용: `path.extension() == Some("tmp")` 필터가 backup 레이어 디렉토리 안의 모든 `.tmp` 파일을 삭제한다. MYBOX가 동기화 도중 `.tmp` 파일을 목적지 디렉토리에 staging하는 경우 해당 파일이 삭제될 수 있다 (PLAUSIBLE — MYBOX의 실제 staging 경로에 따라 다름). 방어적으로 `app_*.db.tmp` 패턴 매칭 추가 권장.

## 영역별 추가 점검

- 보안 (backend.md Critical): SQL 인젝션 없음, 하드코딩 시크릿 없음, Tauri 권한 변경 없음
- 보안 (backend.md High): `unwrap()`/`expect()` 프로덕션 코드 없음 (unwrap_or만 사용)
- 프론트엔드 (frontend.md Critical/High): `dangerouslySetInnerHTML` 없음, `invoke()` 직접 호출 없음, TypeScript any 없음
- DB 마이그레이션: 없음 (schema 변경 없는 스프린트)
- AI 생성 코드 추가 체크: 새 함수 3개 (`auto_restore_with_retry`, `cleanup_stale_tmp_backups`, WAL 에러 처리) — 로직은 정확하나 테스트 커버리지 부족(F5)

## 결론

Critical 0건. High 2건(F1 stale lock, F2 rollback 충돌)은 데이터 안전성 관련이지만 둘 다 집<->교습소 동시 사용 없음 전제 하에 실제 발생 가능성이 낮음. F1은 다음 스프린트에서 STALE_THRESHOLD 상향 또는 config.json 기반 판정 전환으로 해결 권장. F2는 rollback 파일명에 카운터 추가로 단순 수정 가능. Medium 3건은 ROADMAP 등록 후 이연.
