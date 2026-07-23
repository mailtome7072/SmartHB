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
| src-tauri/src/commands/db.rs | [2회] | T1 after_connect 훅 ✅, T2 create_if_missing 가드 ✅, T6 유휴 close/재연결 |
| src-tauri/src/commands/integrity.rs | [2회] | T2 빈 DB fail-hard ✅, T3 WAL 처리·다계층 폴백·소스 검증 ✅ |
| src-tauri/src/commands/backup.rs | [1회] | T4 소스 검증, 마지막 백업 축출 방지 ✅ |
| src-tauri/src/commands/paths.rs | [2회] | T2 setup_completed 캐시 ✅, T5 config 처리 통일 (무음 fallback 제거, salt.bin SSOT) ✅ |
| src-tauri/src/commands/setup.rs | [2회] | T2 complete_setup 캐시 갱신 ✅, T5 read_status_from_path 공유 ✅ |
| src-tauri/src/commands/auth.rs | [1회] | T5 set_password salt 가드 ✅, T7 try_adopt_key |
| src-tauri/src/commands/lock.rs | [0회] | T8 device.id 손상 처리, touch_lock, STALE 보정 |
| src-tauri/src/commands/startup.rs | [0회] | T3 auto_restore 확장, T6 유휴 감지 백그라운드 태스크 |
| src-tauri/src/lib.rs | [0회] | T7 신규 IPC 등록 (try_adopt_key) |
| src/lib/tauri/index.ts | [0회] | T7 IPC 래퍼 추가 |
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
- [ ] T6 A1 유휴 close + 활동 재연결
- [ ] T7 B1 신규 PC 키 유도 + 키체인 채택
- [ ] T8 B2 device.id 손상 + STALE 보정 (M3, M4)
- [ ] T9 통합 검증 + cipher 스모크 (자동 7항목)

## 작업 순서 (의존성 기반)
1. T1 (after_connect) → T6 의존
2. T2 (create_if_missing) → T3 의존
3. T4 (독립), T5 (config) → T7 의존
4. T8 (독립)
5. T6 (T1 의존), T7 (T5 의존)
6. T9 (전체 완료 후)

## 발견된 이슈
(없음)
