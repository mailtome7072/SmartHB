---
name: sprint-next-session
description: "Sprint 10 Session #3 완료 (T1' V108 마이그레이션). 다음: T3 — 소멸 자동 전이 백엔드 IPC (4h)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session3-t1prime
---

Sprint 10 — Phase 3 완결 sprint. Session #3 (T1' V108) 완료. 다음은 **T3 소멸 자동 전이 백엔드 IPC**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 결정 | ✅ | `b4565d4` |
| T1' | V108 — `makeup_attendances.status` CHECK 단순화 | ✅ | `1efd70f` |
| **T3** | **소멸 자동 전이 백엔드 IPC** (`expiration.rs` 신규) | ⬜ 다음 세션 | — |
| T4 | 소멸 전이 트리거 통합 (앱 시작/출결 생성/교습기간 등록) | ⬜ | — |
| T5 | 보강소멸 → 결석 수동 환원 IPC (AC-4.5-5) | ⬜ | — |
| T6 | 퇴교 시 미사용 보강 처리 IPC (PRD §4.5.9) | ⬜ | — |
| T7 (축소) | 선행 수업 — 출결 생성 충돌 방지 검증 only (PI-08) | ⬜ | — |
| T8 | 캘린더 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9~T11 | UI (소멸 환원 / 퇴교 보강 / 캘린더 뷰) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T1' 결과 요약

| 영역 | 변동 |
|------|------|
| 마이그레이션 | `108__cleanup_makeup_status_check.sql` 신규 (50 라인) — CHECK 제약에서 `'makeup_absent'` 제거 |
| 패턴 | SQLite CHECK ALTER 미지원 → table rename + INSERT SELECT (V107 동일) |
| 자동 검증 | cargo test 251 passed (T1 동일, 회귀 없음) / clippy clean |
| .sqlx | 영향 없음 — makeup.rs는 런타임 sqlx::query 사용 |

## T3 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T3 작업 계획 (예상 4h)

scope.md Session #2 의 핵심 설계를 그대로 구현:

1. **신규 모듈** `src-tauri/src/commands/expiration.rs`
   - `expire_overdue_absences_impl(pool, as_of: Option<NaiveDate>)` — `Local::now` 기본값 (PI-06)
   - `expire_overdue_absences()` IPC 핸들러
   - 응답 구조체 `ExpirationReport { transitioned_count, details }`
   - `ExpiredAbsenceDetail { student_name, event_date, makeup_deadline }`

2. **핵심 SQL**
   ```sql
   UPDATE regular_attendances
   SET status = 'makeup_expired'
   WHERE status = 'absent'
     AND makeup_attendance_id IS NULL
     AND makeup_deadline IN (
       SELECT year_month FROM study_periods WHERE end_date <= ?  -- as_of
     )
   RETURNING ...;
   ```

3. **audit variant 추가** — `MakeupExpired` → `"makeup-expired"`
   - `src-tauri/src/commands/audit.rs` Sprint 9 패턴 그대로

4. **단위 테스트 6건+** (sprint10.md T3 AC):
   - 소멸기한 도래 + 미보강 → 전이 성공
   - 소멸기한 미도래 → 전이 없음
   - 이미 `makeup_done` → 전이 대상 아님
   - 이미 `makeup_expired` → 중복 전이 없음
   - 교습기간 미등록 월 → 전이 보류 (대기) — PI-05 정책 검증
   - 복수 원생 batch 전이

5. **mod.rs / lib.rs 등록** — `pub mod expiration;` + invoke_handler

### T3 후속 (T4)
- 3개 트리거 통합 (PI-05 결정 — 앱 시작/출결 생성/교습기간 등록)
- TS 래퍼 `expireOverdueAbsences` 추가
- 응답 구조체에 `expiration_report` 옵션 필드 추가 (출결 생성/교습기간 등록 응답에 동봉)

## Sprint 10 Capacity 추적

- 계획: 38.5h 구현 + 6h 시각 검증 버퍼 = **44.5h**
- 실측 누적: T1 1.5h + T2 1h + T1' 0.5h = 3h
- 남은 capacity: 약 41.5h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **마이그레이션 정책** — V108 (Sprint 10 T1') 까지 적용. 다음은 V109+ (필요 시)
