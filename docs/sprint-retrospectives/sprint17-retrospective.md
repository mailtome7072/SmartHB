# Sprint Retrospective Sprint 17

> 대상: Sprint 17 (82c385a) — DB 안전성 잔여 수정 + 정책 간소화
> 리뷰 일자: 2026-06-30
> 코드 리뷰: Critical 0 / High 2건(F1 stale lock, F2 rollback 충돌) / Medium 3건 / Low 2건
> 자동 검증: cargo test 411 passed (cipher off) / clippy --all-targets clean / cargo check --features cipher OK / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint16-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A101 | 배포 후 cipher 실동작 스모크 테스트 (.exe 설치) | ⏸️ 이연 | v1.0.1 hotfix 배포 후 수동 스모크테스트 미수행. Sprint 17 완료 후 재검토 필요 |
| A102 | release_lock_atomic 테스트 직렬화 검토 | ⏸️ 이연 | Sprint 17 범위 외. 차기 안정화 스프린트에서 처리 |
| A103 | cipher release 빌드 환경 기록 (PowerShell 전용) | ⏸️ 이연 | docs/setup-guide.md 미업데이트. 차기 스프린트 전 처리 권장 |
| A104 | notices/page.tsx useUnsavedChanges 훅 마이그레이션 | ⏸️ 이연 | P2 스프린트로 이연 유지 |
| A105 | MoveAttendanceDialog.handleSelect finally 블록 | ⏸️ 이연 | 차기 안정화 스프린트 대상 |
| A106 | P2 1차 안정화 스프린트 범위 확정 | ⏸️ 이연 | 실사용 피드백 수집 중. 시기 미도래 |

---

## 잘한 점

**Hotfix 후속 잔여 이슈 6건을 당일 완료 — 위험 구간을 빠르게 닫음**

v1.0.0 실사용 중 발견된 DB 저장 실패 버그를 Hotfix에서 긴급 6건 처리한 후, 남은 안전성 수정 3건(WAL 에러 처리·atomic write·복원 재검증)과 정책 간소화 3건(heartbeat 제거·hourly 간격 확대·SyncStatus 삭제)을 Sprint 17에서 당일 완료했다. 기술 부채를 분리하여 "긴급/비긴급" 두 배치로 처리한 전략이 적중했다.

**atomic write(T2) 구현이 NTFS power-loss 패턴을 올바르게 적용**

`.tmp` 파일 생성 → `quick_check` 검증 → `rename` 순서는 프로젝트 메모리(ntfs-power-loss-pattern.md)에 기록된 패턴을 충실히 따랐다. 미완성 백업 파일이 MYBOX에 즉시 동기화되는 문제를 구조적으로 차단했고, stale `.tmp` 파일 정리 함수도 함께 추가하여 비정상 종료 시나리오까지 대비했다.

**SyncStatus 완전 삭제(T6)가 코드베이스 전체에서 일관되게 처리됨**

`sync.rs` 삭제부터 `lib.rs` IPC 등록 제거, `index.ts` 래퍼, `app-shell.tsx` polling, `top-bar.tsx` UI, `types/index.ts` 타입까지 6개 파일에서 참조를 완전 제거했다. `pnpm tsc --noEmit`과 `pnpm lint`가 orphan 참조를 사전에 잡아냈고, 빌드가 클린하게 성공했다.

**simplify 적용이 함수 가독성을 실질적으로 높임**

`integrity.rs`의 `max_attempts` 변수를 `.take(3)` 직접 사용으로 단순화하고, `setup.rs`의 `map_err(|_|)`을 `map_err(|e|)` + `eprintln!`으로 교체하여 진단 정보를 보존했다. 코드 길이 감소보다 의도 명확성이 향상된 케이스.

---

## 개선할 점

**heartbeat 제거 후 stale lock 임계값 미조정 — 구조적 취약점 잔존**

T5에서 heartbeat 루프를 제거했으나 `STALE_THRESHOLD_SECONDS = 300`(5분)은 변경하지 않았다. `LockInfo::new_for_self()`가 획득 시점에 `last_heartbeat`를 1회만 기록하고 이후 갱신이 없으므로, 앱을 5분 이상 사용하면 두 번째 PC가 "stale lock 자동 점유"(lock.rs:313)로 락을 획득할 수 있다. Sprint 17 계획 T5에서 "stale 판정 로직(5분 mtime 기반)은 유지 또는 config.json 기반으로 대체" 방향이 언급됐지만 구현이 빠졌다. 코드 리뷰에서 F1(High)으로 발견됐다.

**rollback 파일명 충돌 시나리오가 retry 설계 시 고려되지 않음**

`generate_rollback_filename`은 초(second) 단위 정밀도(`rollback_YYYYMMDD_HHMMSS.db`)를 사용한다. `auto_restore_with_retry` retry 루프에서 두 번의 복원이 1초 내에 실행되면 동일 rollback 경로로 `std::fs::rename`이 호출되어 iteration-1의 원본 DB rollback이 덮어쓰여진다. 새 retry 함수를 설계할 때 기존 파일명 생성 함수의 정밀도 한계를 고려하지 않은 설계 갭이다.

**새 핵심 함수에 단위 테스트가 빠짐**

`auto_restore_with_retry`는 startup 손상 복원 경로의 핵심 함수로 sprint17 계획 T3에 "단위 테스트 필수"가 명시됐으나 `#[cfg(test)]` 블록에 커버가 없다. `cleanup_stale_tmp_backups`와 WAL 체크포인트 실패 케이스도 동일. 함수 구현과 테스트 작성이 같은 커밋에서 완료되지 않아, 회귀 발생 시 자동 감지가 불가능한 상태.

**cleanup_stale_tmp_backups가 async 경로에서 spawn_blocking 없이 호출**

`run_startup`(async fn) 내에서 `backup::cleanup_stale_tmp_backups()`를 직접 호출한다. 내부가 `std::fs`(blocking) 이므로 tokio executor thread를 점유한다. 네트워크 드라이브 환경에서 startup 지연 가능성이 있다. Rust async 패턴에서 blocking I/O는 반드시 `spawn_blocking`으로 분리해야 한다는 원칙을 간과했다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A107 | STALE_THRESHOLD_SECONDS 상향(86400) 또는 config.json 기반 판정 전환 — heartbeat 제거와 함께 처리됐어야 할 임계값 조정. 현행 5분 임계값은 heartbeat 없는 환경에서 구조적 취약점 | High | `src-tauri/src/commands/lock.rs:39` | Sprint 18 T0 |
| A108 | rollback 파일명에 loop 인덱스 또는 나노초 추가 — `generate_rollback_filename` 또는 `auto_restore_with_retry` 호출부에서 파일명 고유성 보장. `rollback_YYYYMMDD_HHMMSS_{idx}.db` | High | `src-tauri/src/commands/integrity.rs` `generate_rollback_filename` | Sprint 18 T0 |
| A109 | auto_restore_with_retry 단위 테스트 작성 — 3회 retry 성공/실패, quick_check 실패 후 다음 후보 전환 시나리오 커버 | Medium | `src-tauri/src/commands/integrity.rs` `#[cfg(test)]` | Sprint 18 |
| A110 | cleanup_stale_tmp_backups spawn_blocking 래핑 — `tokio::task::spawn_blocking(|| backup::cleanup_stale_tmp_backups()).await` 또는 결과 무시 시 spawn만 | Medium | `src-tauri/src/startup.rs:207` | Sprint 18 |
| A111 | WAL 체크포인트 실패 시 pool.close() 보장 — early return 전 pool.close() 호출 추가 또는 SQLITE_BUSY 시 1-3회 retry | Medium | `src-tauri/src/commands/setup.rs:244-286` | Sprint 18 |
| A112 | cipher 스모크 테스트 수행 — v1.0.1(.exe) 설치 후 integrity_check, 백업/복원, DB 폴더 변경 실동작 확인. A101 이월 | High | 배포 후 수동 검증 | Sprint 17 develop QA 후 |
