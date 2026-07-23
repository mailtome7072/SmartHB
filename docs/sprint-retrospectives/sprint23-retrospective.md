# Sprint Retrospective Sprint 23

> 대상: Sprint 23 (94171fa~9fdad65) — 2026-07-22 프로덕션 데이터 소실 사고 재발방지 (ADR-012 A안)
> 리뷰 일자: 2026-07-23
> 코드 리뷰: Critical 0 / High 0 / Medium 2 / Low 3건
> 자동 검증: cargo test 478 passed / clippy clean / tsc clean / lint clean / build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint22-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A115 (High) | cipher 스모크 테스트 (Sprint 18~22 이월) | ✅ 완료 | T9 통합 검증에서 `cargo check --features cipher` 통과, `cargo test --features cipher` 140건 통과로 cipher-on 빌드 동작 확인 |
| A127 (Medium) | cancel_makeup_impl N+1 배치 쿼리 전환 | ⏸️ 이연 유지 | makeup.rs 미수정, 범위 밖 유지 |
| A128 (Low) | cancel_makeup docstring ADR-011 반영 갱신 | ⏸️ 이연 유지 | makeup.rs 미수정, 범위 밖 유지 |
| A129 (Low) | 보강 시간 입력 드롭다운 전환 | ⏸️ 이연 유지 | 출결 UI 미수정, 범위 밖 유지 |
| A114 (Low, 5회 이연) | sync_single_date 이력 패턴 통일 | ⏸️ 이연 유지 (6번째) | attendance.rs 로직 변경 없음, 다음 attendance.rs 수정 스프린트 포함 의무 |

---

## 잘한 점

### 사고 당일 RCA 완료 + 익일 재발방지 구현 착수

2026-07-22 프로덕션 데이터 소실 사고를 당일 `docs/incidents/2026-07-22-data-loss-rca.md` 로 문서화하고, 원인 분류(C1~C3 결함, H1~H5 설계 공백, M1~M4 미흡, B1~B2 불편)를 Weighted Matrix + SWOT 로 ADR-012 결정까지 마쳤다. 사고를 감지하고 RCA 를 완료하는 데까지 당일 안에 완결한 점이 특히 우수하다.

### 전역 Pool 전환: OnceCell → RwLock + async 재연결 (T6)

단순 참조 반환에서 완전 close + 재연결 패턴으로의 전환은 89개 이상의 호출부에 영향을 미치는 대규모 리팩터였다. `let pool = db::pool().await?; let pool = &pool;` 참조 섀도잉 패턴으로 다운스트림 변경을 최소화하면서 소유권 이전을 올바르게 처리했다. `RECONNECT_LOCK`(async Mutex)과 `POOL_SHUTDOWN` 래치의 조합으로 close-재연결 경쟁 조건과 구 경로 우발 재연결 두 가지 위험을 동시에 차단했다.

### 복원 체계 다계층 폴백 (T3)

exit-only 단층 복원에서 exit→daily→weekly 체인 폴백으로 확장하면서, 소스 파일 크기 사전 검증(512B 미만 거부) + quick_check + 원생 0명 빈 DB 거부를 조합해 "손상 백업으로 복원" 시나리오를 원천 차단했다. WAL 사이드카 원자 제거와 복원 후 fsync(R145 NTFS power-loss 대응)까지 포함해 복원 경로 전반을 강화했다.

### config 처리 통일 + salt SSOT 채택 (T2/T5)

`setup_completed` 캐시를 `paths.rs` 단일 진입점에 두고, config 부분 손상 시에도 `salt.bin` 존재만으로 셋업 완료를 추론하는 폴백(`done = status.setup_completed || salt_path().exists()`)이 M1 결함을 우아하게 해소했다. `set_password` 의 salt 하드 가드(M2)와 `try_adopt_key` 의 기존 salt 읽기 전용(T7)이 논리적으로 보완 관계를 형성한 설계가 인상적이다.

### A115 cipher 스모크 테스트 완료

Sprint 18 부터 5스프린트 이연됐던 A115(cipher 스모크 테스트)를 T9 통합 검증에서 자연스럽게 포함해 완료했다. `cargo check --features cipher` + `cargo test --features cipher` 140건 통과로 cipher-on 빌드의 정상 동작을 공식 확인했다.

---

## 개선할 점

### pool() fast-path TOCTOU (M-01 발견)

`pool()` 의 빠른 경로는 `RECONNECT_LOCK` 없이 pool 열림을 확인한다. 확인 후 쿼리 전에 `close_for_idle()` 가 개입하면 "pool is closed" 오류가 가능하다. 발생 확률은 낮지만, 5분 이상 실행되는 명령(예: 대량 임포트)에서는 쿼리 간 커넥션 반환 시점에 close 가 완료될 수 있다. 데이터 손상은 없으나 사용자가 오류를 경험할 수 있다.

### LockScreen 오류 분기 문자열 매칭 (M-02 발견)

`tryAdoptKey` 의 "이미 이 PC" 오류를 한국어 문자열로 구분하는 패턴은 백엔드 메시지 변경 시 조용히 분기가 오동작할 수 있다. 동일 저장소라 위험은 낮으나, 오류 코드(enum 변환) 기반 구분이 더 견고하다.

### eprintln! 프로덕션 로그 관찰 불가 (L-01 발견)

Sprint 23 에서 DB close/재연결, 락 이벤트, 복원 이벤트 등 안전 관련 이벤트가 `eprintln!` 으로 기록된다. Tauri 프로덕션에서 stderr 가 캡처되지 않으면 이 로그가 관찰 불가능하다. 데이터 안전 강화 스프린트인 만큼, 관찰 가능성을 높이는 구조화 로깅 도입이 다음 단계에서 필요하다.

### A114 sync_single_date 6스프린트 이연

Sprint 18→22→23 까지 6회 이연됐다. 기능 버그가 아닌 내부 일관성 문제이지만, 누적 이연 자체가 기술 부채로 작용한다. 다음 attendance.rs 수정 스프린트에서는 이 항목을 T0 에 강제 포함할 것을 권장한다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 | 상태 |
|----|------|----------|-----------|------|------|
| A130 | pool() TOCTOU 경합 모니터링 + 개선 검토 — LAST_ACTIVITY_SECS 를 쿼리 레벨에서 갱신하거나 장기 명령에 자체 타이머 리셋 패턴 도입 | Medium | `src-tauri/src/commands/db.rs` | 관찰 후 다음 db.rs 수정 스프린트 | 📋 예정 |
| A131 | LockScreen.tsx 채택 오류 분기를 구조화 오류 코드로 개선 — 백엔드 auth 오류 enum 정의 + 프론트 코드 분기로 전환 | Medium | `src-tauri/src/commands/auth.rs`, `src/components/LockScreen.tsx` | 다음 auth.rs 수정 스프린트 | 📋 예정 |
| A132 | 구조화 로깅 도입 검토 (`tauri-plugin-log` 또는 `tracing`) — 데이터 안전 이벤트(유휴 close, 복원, 락 획득)를 파일/알림으로 기록 | Low | `src-tauri/` 전반 | 다음 Phase 로드맵 검토 시 | 📋 예정 |
| A127 | cancel_makeup_impl N+1 배치 쿼리 전환 (Sprint 22→23 이월) | Medium | `src-tauri/src/commands/makeup.rs:cancel_makeup_impl` | 다음 makeup.rs 수정 스프린트 | ⏸️ 이연 |
| A128 | cancel_makeup docstring ADR-011 반영 갱신 (Sprint 22→23 이월) | Low | `src-tauri/src/commands/makeup.rs:575-581` | 다음 makeup.rs 수정 스프린트 | ⏸️ 이연 |
| A129 | 보강 시간 입력 드롭다운 전환 (Sprint 22→23 이월) | Low | `src/components/attendance/MakeupRegisterDialog.tsx:230-243` | 다음 출결 UI 수정 스프린트 | ⏸️ 이연 |
| A114 | sync_single_date 이력 패턴 통일 (Sprint 18~23 이월, 6번째) — 다음 attendance.rs 수정 스프린트에서 T0 강제 포함 | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | 다음 attendance.rs 수정 스프린트 (강제 포함) | ⏸️ 이연 |
