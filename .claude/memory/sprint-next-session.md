---
name: sprint-next-session
description: "Sprint 8 Session #5 완료 (T1~T5, 5/9). 다음 세션: T6 (Sprint 7 carry-over High 4건 — Keychain/auth 보안)"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint8-session5-t5
---

Sprint 8 출결 관리 — T1~T5 완료, T6~T9 다음 세션 이연.

## Sprint 8 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | V106 마이그레이션 (regular_attendances + makeup_attendances) | ✅ `f72778b` |
| T2 | 출결 생성 IPC (generate / check_exists) | ✅ `366f880` |
| T3 | 출결 조회 + 토글 IPC (grid / toggle / memo / summary) | ✅ `4efc570` |
| T4 | 출결표 프론트엔드 UI (/attendance + AttendanceGrid + AbsenceMemoDialog) | ✅ `0a20c18` |
| T4 follow-up | UX 보강 (AppShell 누락, 사이드바 너비, 요일 행, 시간 단위, 컬럼 재배치/배경색, 보강관리 메뉴) | ✅ `516758c` |
| T5 | 보강필요시간/소멸기한 단위 테스트 100% (시나리오 2/5/9/10 추가) | ✅ `5f2f0fd` |
| **T6** | **Sprint 7 carry-over High 4건** (I-S2-2/3/4/5, R40~R43) | ⬜ 다음 세션 |
| T7 | carry-over Medium-High (I-S2-7, R45) | ⬜ |
| T8 | carry-over Medium + R51/R52 (I-S2-8/9/10, R39, A31) | ⬜ |
| T9 | 통합 검증 | ⬜ |

검증 상태: `cargo test` cipher off **213 passed** (attendance 22) / clippy --lib clean / `pnpm lint` clean.

## 다음 세션 우선 액션

1. 새 대화에서 `/sprint-dev 8` → Session #6 진입 (T6)
2. **T6는 보안 경로 변경** — `systematic-debugging` 스킬 자동 배정 대상
3. 변경 대상 모듈:
   - `auth.rs` (verify_password / set_password / check_auth_status)
   - `keyring/` (CRED_CACHE static drop)
   - 관련 테스트의 Keychain 사이드이펙트 방지 (`#[ignore]` 또는 mock)

## T6 세부 (sprint8.md L274-294 참조)

- **I-S2-2 (R40)**: `verify_password`에서 partial-NULL 손상 감지·복구 강화 — atomic write fallback 패턴 적용
- **I-S2-3 (R41)**: `set_password` 재진입 가드 — Mutex flag, 중복 호출 거부
- **I-S2-4 (R42)**: `CRED_CACHE` static drop 시점 정리 — shutdown hook에서 명시적 무효화
- **I-S2-5 (R43)**: `check_auth_status`의 legacy keyring fallback 마이그레이션 — `salt_exists_at` 경로 검증

## 후속 처리 필요 항목 (T7~T8)

- **R45 (Medium-High)**: I-S2-7 — concurrent race (verify_password ↔ retrieve_key_from_keyring 순서)
- **R39**: StudyPeriodEditor create+confirm 원자성 (T8 또는 hotfix 후보)
- **R51/R52**: 잡다 low

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지 (`gh pr create` 금지)
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
- **사용자 메모리 미러 동기화 필수** — `~/.claude/projects/-Users-skyang-Projects-SmartHB/memory/sprint-next-session.md` 동시 갱신
