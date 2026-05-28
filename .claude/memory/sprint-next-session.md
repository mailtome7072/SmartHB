---
name: sprint-next-session
description: "Sprint 10 + hotfix 4건 develop 머지 + 수동 검증 완료. 다음: deploy-prod (develop → main + v0.4.x 태그)"
metadata:
  node_type: memory
  type: project
  originSessionId: post-sprint10-hotfix-ready-to-deploy
---

Sprint 10 + 본 세션 hotfix 모두 develop 에 머지·푸시 완료 (2026-05-28). 수동 검증 통과. 다음 단계는 **deploy-prod** — develop → main 머지 + `v0.4.x` 태그 push → GitHub Actions 자동 빌드.

## develop 누적 상태

| 커밋 | 내용 |
|------|------|
| `4e5ab15` | hotfix: 퇴교 번복 시 absence_memo 도 클리어 |
| `bfd9552` | hotfix(docs): 퇴교 번복 다이얼로그 안내 갱신 |
| `e921e6c` | hotfix: 퇴교 번복 결석 환원 + UX 보완 (AlertDialog controlled / date picker blur / z-index) |
| `2b5f482` | Sprint 10 완료 — Phase 3 완결 (소멸 자동 전이 + 퇴교 보강 + 캘린더 뷰) |

## 본 세션 hotfix 요약

### 1. reinstate_student 결석 환원 (`e921e6c`)
- 퇴교 시 `process_withdrawal_makeup` 으로 `makeup_expired` 전이된 결석 중 `makeup_deadline >= 현재 YYYY-MM` 인 항목만 `absent` 로 환원.
- 자연 만기 소멸은 T5 폐기 정책에 따라 환원 대상 외.
- 트랜잭션 안에서 결석 환원 + `withdraw_date NULL` 원자적 적용.
- audit `student-reinstated` details 에 `revivedAbsenceIds:[...]` 추적.

### 2. 다이얼로그 UX (`e921e6c`)
- 퇴교 확인 AlertDialog → controlled 변환. `handleWithdrawConfirmed` 진입 시 `setWithdrawAlertOpen(false)` 명시.
- `WithdrawalMakeupDialog` z-index `50 → 60` (잔존 backdrop 대비 안전망).
- 퇴교일자 date input `onChange` 안에서 `e.target.blur()` — Tauri WebView native picker 강제 닫힘.

### 3. 안내 메시지 정합 (`bfd9552`)
- 번복 다이얼로그의 "보강 잔여 처리는 Phase 3 에서 별도 제공됩니다." → 현재 동작 명시로 교체.

### 4. absence_memo 클리어 (`4e5ab15`)
- `ExternalExpire` 가 일괄 덮어쓴 외부 처리 메모를 환원 시 NULL 클리어. `attendance.rs::toggle_attendance` 의 결석→출석 전환 패턴과 동일.
- 단위 테스트 강화: memo 클리어 검증.

## 검증 결과 (집 PC)
- `cargo test --lib` (cipher off): 274 passed (Sprint 10 273 → hotfix +1)
- `cargo clippy --lib` clean / `pnpm lint` clean / `pnpm tsc` clean
- 사용자 수동 검증: 퇴교 → 다이얼로그 → 3선택지 클릭 / 번복 → 결석 환원 / 외부 메모 클리어 모두 통과
- DB 상태 확인: 홍길동 5/22·5/27 모두 정상 (absent + memo null)

## 다음 액션 — deploy-prod

새 대화 또는 같은 세션에서:

> "deploy-prod 실행해줘."

deploy-prod 에이전트가 처리:
1. `harness-ci-gate` 통과 점검 (CHANGELOG / sprint-review 산출물 / 시크릿 등)
2. `develop → main` 머지 (단일 개발자 정책, PR 생략 가능)
3. `v0.4.x` 태그 push → GitHub Actions: Windows/macOS 인스톨러 자동 빌드 + Release 첨부
4. 배포 후 CV 체크리스트 (헬스체크 / 에러 로그)

배포 후 사용자 작업:
- `DEPLOY.md` 의 ⬜ 항목 처리 → 완료본 `docs/deploy-history/2026-05-28.md` 로 아카이브
- 다음 스프린트(Sprint 11) 계획 수립은 별도 진입

## Sprint 11 carry-over 메모

> Sprint 10 sprint-review 에서 발견된 finding 5건은 모두 이연. 발견 사항 상세는 `docs/code-reviews/sprint10.md`.

- F1 (Medium): `build_day_schedules` succ_opt().expect() 패닉 가능 (`src-tauri/src/commands/attendance.rs:655`)
- F2 (Medium): `generate_impl` expire fail-hard vs startup fail-soft 정책 불일치 (`attendance.rs:155`)
- F3 (Low): `_year_month` 파라미터 미사용 (`calendar.rs:188`)
- F4 (Low): 보강관리 N+1 쿼리 (`calendar.rs:215`)
- F5 (Low): `ClassCalendar` viewType 비동기 업데이트 한 프레임 불일치 (`ClassCalendar.tsx:164`)
- 사이드 메뉴 '보강 관리' (`/makeups`) disabledHint 정리 — Sprint 10 T11 에서 `/schedules` 탭으로 통합됐으나 메뉴 정의는 옛 가정 그대로 (`src/lib/menu-config.ts:21`)
- flaky: `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` (병렬 시 간헐 실패)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, develop → main 직접 머지 ([[workflow-no-pr]])
- **메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
