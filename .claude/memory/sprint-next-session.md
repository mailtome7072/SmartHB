---
name: sprint-next-session
description: "Sprint 8 Session #8 완료 (T1~T8, 8/9). 다음 세션: T9 통합 검증 (자동 + 수동 시각 검증)"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint8-session8-t8
---

Sprint 8 출결 관리 — T1~T8 완료, T9 (마지막) 다음 세션.

## Sprint 8 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | V106 마이그레이션 | ✅ `f72778b` |
| T2 | 출결 생성 IPC | ✅ `366f880` |
| T3 | 출결 조회 + 토글 IPC | ✅ `4efc570` |
| T4 | 출결표 프론트엔드 UI | ✅ `0a20c18` |
| T4 follow-up | UX 보강 | ✅ `516758c` |
| T5 | 보강필요시간/소멸기한 단위 테스트 100% | ✅ `5f2f0fd` |
| T6 | Sprint 7 carry-over High 4건 (I-S2-2/3/4/5, R40~R43) | ✅ `14b9bfb` |
| T7 | Sprint 7 carry-over Medium-High (I-S2-7, R45) — Keychain race | ✅ `e89c3a8` |
| T8 | carry-over Medium 6항목 (R46/R47/R48-a/R39/R51, A31) | ✅ `069f435` |
| **T9** | **통합 검증** (자동 + 수동 시각 검증 1h) | ⬜ 다음 세션 |

검증 상태: `cargo test --lib` cipher off **221 passed** / cipher on **133 passed** / clippy --lib clean 양쪽 / `pnpm lint` clean / `pnpm tsc --noEmit` clean.

## Session #8 (T8) 핵심 변경

- **R46**: `cred_cache_lock()` 헬퍼로 7곳 `.expect("cred_cache poisoned")` 일괄 정리 + LOAD_MUTEX 인라인. `PoisonError::into_inner()` 로 graceful 복구
- **R47**: `AuditEventType::SecurityEvent` variant 추가. `migrate_keyring_salt_to` 성공 시 tokio runtime 검출 후 fire-and-forget spawn
- **R48-a**: `write_device_id_atomic` Unix 권한 0o600. `device_id_file_has_owner_only_permissions` 테스트
- **R39**: `create/update_study_period` overlap 쿼리에 `AND is_confirmed = 1`. 미확정 교습기간이 신규 등록 차단 안 함. `overlap_skips_unconfirmed_periods` SQL 단위 테스트
- **R51**: `calendarEventClick` 에 `if (studyPeriodMode) return` 가드
- **A31**: 외부 점유 skip 가드 이미 보유 → 추가 변경 불필요로 확인
- **R48-b (skip)**: salt buffer ZeroizeOnDrop 시그니처 광범위 변경 필요 — 후속 task 이연

## 다음 세션 우선 액션 (T9: 통합 검증)

1. 새 대화에서 `/sprint-dev 8` → Session #9 진입 (T9)
2. T9 작업 (sprint8.md L343-367):
   - **자동 검증**: cargo test (cipher off+on) / cargo clippy / pnpm lint / pnpm tsc / pnpm build
   - **수동 검증 (사용자 시각 검증 1h)**:
     - T1: sqlx migrate run 후 테이블 생성 확인
     - T2: 월 선택 → "출결 생성" → 출결 레코드 생성 확인
     - T3: 출결표 그리드 + 셀 클릭 토글
     - T4: 출결표 UI 전체 흐름 (출석/결석/요약/Undo)
     - T5: 보강필요시간이 결석 토글에 정확 반응
     - T6: Keychain 다이얼로그 1회 이하 + startup < 3초
     - T7: startup 순서 정상 동작
     - T8: 교습기간 미확정 overlap 해소 + selection 모드 배지 클릭 무동작
     - UC-3 일일 출결 입력 전체 흐름 완주
3. T9 완료 후 → `sprint-close` agent → `sprint-review` agent

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지 (`gh pr create` 금지)
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지
- **사용자 메모리 미러 동기화 필수** — `.claude/memory/sprint-next-session.md` 동시 갱신
