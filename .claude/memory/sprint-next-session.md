---
name: sprint-next-session
description: "Sprint 8 Session #7 완료 (T1~T7, 7/9). 다음 세션: T8 (Medium 잔여 + R39/R51/R52, A31)"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint8-session7-t7
---

Sprint 8 출결 관리 — T1~T7 완료, T8~T9 다음 세션 이연.

## Sprint 8 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | V106 마이그레이션 (regular_attendances + makeup_attendances) | ✅ `f72778b` |
| T2 | 출결 생성 IPC (generate / check_exists) | ✅ `366f880` |
| T3 | 출결 조회 + 토글 IPC (grid / toggle / memo / summary) | ✅ `4efc570` |
| T4 | 출결표 프론트엔드 UI (/attendance + AttendanceGrid + AbsenceMemoDialog) | ✅ `0a20c18` |
| T4 follow-up | UX 보강 | ✅ `516758c` |
| T5 | 보강필요시간/소멸기한 단위 테스트 100% | ✅ `5f2f0fd` |
| T6 | Sprint 7 carry-over High 4건 (I-S2-2/3/4/5, R40~R43) — Keychain/auth 보안 | ✅ `14b9bfb` |
| T7 | Sprint 7 carry-over Medium-High (I-S2-7, R45) — Keychain concurrent race | ✅ `e89c3a8` |
| **T8** | **carry-over Medium + R51/R52** (I-S2-8/9/10, R39, A31) | ⬜ 다음 세션 |
| T9 | 통합 검증 (수동 시각 검증 포함) | ⬜ |

검증 상태: `cargo test --lib` cipher off **219 passed** / cipher on **132 passed** / clippy --lib clean 양쪽.

## Session #7 (T7) 핵심 변경

- **R45 race 확인**: `get_cached_or_load_key` + `verify_password` 가 double-checked locking 누락 — 캐시 미스 시 `cred_cache` lock 해제 후 `load_credentials_to_cache` 호출 사이에 race
- **해결**: `static LOAD_MUTEX: Mutex<()>` + `ensure_cache_loaded()` 헬퍼. fast path 캐시 hit + slow path LOAD_MUTEX 직렬화 + double-check 패턴. keyring 첫 진입자 1회만 호출
- **호출자 통합**: 두 함수 모두 `ensure_cache_loaded()?` 호출로 단순화 (verify_password 캐시 미스 분기 5줄 → 1줄)
- **테스트**: 16 스레드 fast-path 동시 진입 검증, slow-path 직렬화는 OS keychain 의존이라 `#[ignore]`

## 다음 세션 우선 액션

1. 새 대화에서 `/sprint-dev 8` → Session #8 진입 (T8)
2. T8 작업 (sprint8.md L314-339):
   - **I-S2-8 (R46)**: `auth.rs` Mutex poison 복구 — `.lock().expect("cred_cache poisoned")` → `lock().unwrap_or_else(|e| e.into_inner())`
   - **I-S2-9 (R47)**: `migrate_keyring_salt_to` 에 `try_record(AuditEventType::SecurityEvent, ...)` 추가
   - **I-S2-10 (R48)**: `device.id` 권한 0o600 / salt buffer ZeroizeOnDrop / stale doc comment 정리
   - **R39 (A28)**: `academic.rs` `create_study_period` overlap 검사에 `AND is_confirmed = 1` 추가
   - **A31**: lock 테스트 flaky 해소
   - **R51 (A37)**: `academic/page.tsx` selection 모드 중 배지 클릭 비활성화

## T9 통합 검증 (마지막)

- 자동: cargo test/clippy + pnpm lint/tsc/build 전체 통과
- 수동 시각 검증 (1h): UC-3 출결 전체 흐름 + T1~T8 항목별 확인

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지 (`gh pr create` 금지)
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지
- **사용자 메모리 미러 동기화 필수** — `.claude/memory/sprint-next-session.md` 동시 갱신
