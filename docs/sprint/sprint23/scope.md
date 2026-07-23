---
Sprint: 23  |  Date: 2026-07-23  |  Session: #1
---

## 스프린트 개요
프로덕션 데이터 소실 사고(2026-07-22) 재발방지. ADR-012 A안(클라우드 유지 + 접근 강화).
DB 마이그레이션·새 의존성 없음 — Rust 로직 변경 중심. v1.4.0 → v1.5.0.
T0(ADR-012) ✅ 완료 → T1부터 진행.

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/db.rs | [3회] | T1 after_connect 훅 ✅, T2 create_if_missing 가드 ✅, T6 유휴 close/재연결 ✅ |
| src-tauri/src/startup.rs | [1회] | T6 유휴 감지 백그라운드 태스크 ✅ (+ pool_if_open 전환) |
| (그 외 pool() 호출부 15개 파일) | [1회] | T6 pool() async 전환 — 바인딩 참조 섀도잉(downstream 무변경) |
| src-tauri/src/commands/integrity.rs | [2회] | T2 빈 DB fail-hard ✅, T3 WAL 처리·다계층 폴백·소스 검증 ✅ |
| src-tauri/src/commands/backup.rs | [1회] | T4 소스 검증, 마지막 백업 축출 방지 ✅ |
| src-tauri/src/commands/paths.rs | [2회] | T2 setup_completed 캐시 ✅, T5 config 처리 통일 (무음 fallback 제거, salt.bin SSOT) ✅ |
| src-tauri/src/commands/setup.rs | [2회] | T2 complete_setup 캐시 갱신 ✅, T5 read_status_from_path 공유 ✅ |
| src-tauri/src/commands/auth.rs | [2회] | T5 set_password salt 가드 ✅, T7 try_adopt_key ✅ |
| src-tauri/src/commands/lock.rs | [1회] | T8 device.id 유실 처리, touch_lock, 활동기준 STALE ✅ |
| src-tauri/src/lib.rs | [1회] | T7 try_adopt_key IPC 등록 ✅ |
| src/lib/tauri/index.ts | [1회] | T7 tryAdoptKey 래퍼 ✅ |
| src/components/LockScreen.tsx | [1회] | T7 잠금 실패 시 키 채택 폴백 ✅ (계획 예상목록 외 — 기능 도달성 위해 추가) |
| src/components/LockWarning.tsx | [0회] | T8 STALE_THRESHOLD_SECONDS 동기 (A113 상수 쌍) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/migrations/ — 신규 마이그레이션 없음 (V312 유지)
- [ ] attendance.rs — A114 이연 (범위 밖)
- [ ] makeup.rs — A127 이연 (범위 밖)

## 완료 기준 (스프린트 전체 — sprint23.md Definition of Done)
- ✅ T0 ADR-012 (A안 확정)
- ✅ T1 A3 after_connect PRAGMA 재적용 (C3, H5)
- ✅ T2 A2 create_if_missing 가드 + 빈 DB fail-hard (C1, C2)
- ✅ T3 A4 자동 복원 체계 강화 (H1, H3, H4)
- ✅ T4 A5 백업 소스 검증 + 축출 방지 (H2)
- ✅ T5 A6 config 통일 + set_password salt 가드 (M1, M2)
- ✅ T6 A1 유휴 close + 활동 재연결 (A안 채택 — 사용자 결정: 처음부터 강력한 조치)
- ✅ T7 B1 신규 PC 키 유도 + 키체인 채택
- ✅ T8 B2 device.id 손상 + STALE 보정 (M3, M4) — STALE 값(86400) 유지로 A113 프론트 동기화 불필요
- [ ] T9 통합 검증 + cipher 스모크 (자동 7항목)

## 작업 순서 (의존성 기반)
1. T1 (after_connect) → T6 의존
2. T2 (create_if_missing) → T3 의존
3. T4 (독립), T5 (config) → T7 의존
4. T8 (독립)
5. T6 (T1 의존), T7 (T5 의존)
6. T9 (전체 완료 후)

## 발견된 이슈

### T6 설계 갈림길 (2026-07-23, Session #1) — 유휴 close + 재연결 구조 충돌
- 전역 `POOL`이 `OnceCell<SqlitePool>`(교체 불가), 동기 `pool() -> &'static SqlitePool`가 89곳/17파일에서 직접 사용.
- 계획의 "유휴 시 pool.close() + 활동 시 build_pool 재호출"을 그대로 구현하려면:
  (1) POOL을 교체 가능 구조로 변경 → 반환타입/호출부 광범위 영향, (2) 동기 pool() vs 비동기 재연결 불일치 → IPC 진입 게이트웨이 필요.
- **대안 검토**:
  - A. 완전 close+재연결: 계획 충실, 그러나 코어 대규모 리팩터(고위험).
  - B. leaked-'static swap로 close 지원(호출부 무변경) + 재연결 트리거: 세션당 소량 메모리 leak, 재연결 시점 트리거 여전히 필요.
  - C. 유휴 시 wal_checkpoint(TRUNCATE)만 수행(close 안 함): 저위험. WAL을 본체 병합→클라우드 단일 파일 동기화 보장(주 안전 이득 확보). 파일 핸들 해제는 미달성(계획 대비 축소).
- **사용자 결정: A안(완전 close+재연결) 채택** — "처음부터 강력한 조치".
- **구현 방식(회귀 최소화)**: `pool()`을 `async fn -> SqlitePool`(owned, 유휴 시 자동 재연결)로 전환.
  호출부 89+곳은 바인딩 줄만 `let pool = db::pool().await...?; let pool = &pool;`(참조 섀도잉)으로 바꿔
  downstream 쿼리 호출은 전부 무변경. 컴파일러가 누락을 전수 검출 → 조용한 회귀 없음.
  전역 POOL을 `RwLock<Option<SqlitePool>>`로, 재연결 경로 기억 `POOL_DB_PATH`, 직렬화 `RECONNECT_LOCK`,
  활동 추적 `LAST_ACTIVITY_SECS`, 재시작 봉쇄 `POOL_SHUTDOWN`(change_data_folder). 백그라운드/exit는 `pool_if_open`(재연결 안 함).
- **구현 중 수정**: num_idle 가드 제거 — sqlx `num_idle()`가 쿼리 직후에도 0을 반환(size=1)해 오판.
  대신 `pool.close()`의 graceful 대기(사용 중 커넥션 반환까지) + 5분 무활동 임계로 안전 확보.
- **미구현(계획상 optional)**: 재연결 시 mtime/크기 비교(선택적 방어) — 계획서가 "복잡도에 따라 조정 가능"으로 명시, 생략.
