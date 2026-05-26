---
name: sprint-next-session
description: "Sprint 10 Session #2 완료 (T2 설계 결정 PI-05~PI-09). 다음: T1' (V108 마이그레이션) → T3 (소멸 자동 전이 백엔드 IPC)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session2-t2
---

Sprint 10 — Phase 3 완결 sprint. Session #2 (T2 설계) 완료. 다음은 **T1' (V108) → T3 (소멸 자동 전이 IPC)**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + 사용자 확인 (PI-05~PI-09) | ✅ | (현재 커밋) |
| **T1'** | **V108 마이그레이션 — `makeup_attendances.status` CHECK 정리 (PI-07)** | ⬜ 다음 세션 | — |
| T3 | 소멸 자동 전이 백엔드 IPC | ⬜ | — |
| T4 | 소멸 전이 트리거 통합 (앱 시작/출결 생성/교습기간 등록) | ⬜ | — |
| T5 | 보강소멸 → 결석 수동 환원 IPC (AC-4.5-5) | ⬜ | — |
| T6 | 퇴교 시 미사용 보강 처리 IPC (PRD §4.5.9) | ⬜ | — |
| T7 (축소) | 선행 수업 — 출결 생성 충돌 방지 검증 only (PI-08) | ⬜ | — |
| T8 | 캘린더 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9~T11 | UI (소멸 환원 / 퇴교 보강 / 캘린더 뷰) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T2 사용자 결정 사항 (2026-05-26)

| ID | 결정 | 영향 |
|----|------|------|
| **PI-05** | 트리거 3개소 (앱 시작 + 출결 생성 + 교습기간 등록) | T4 통합 작업 그대로 진행 |
| **PI-06** | 소멸 판정 기준일 = `chrono::Local::now()`. 단위 테스트는 `Option<NaiveDate>` 주입 시그니처 | T3 함수 시그니처에 반영 |
| **PI-07** | V108 마이그레이션 진행 | **T1' 신규 task** (0.5h, T3 이전 진행) |
| **PI-08** | 선행 수업 = 기존 상태 토글 흐름 활용 (별도 IPC 불필요) | **T7 범위 축소** — 출결 생성 충돌 방지 검증만 |
| **PI-09** | 자동 전이 알림 = 토스트 (건수 > 0일 때만) | T9 UI 작업 반영 |

## T2 핵심 설계 (scope.md Session #2 참조)

신규 모듈: `src-tauri/src/commands/expiration.rs`

핵심 SQL (T3에서 구현):
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

- `study_periods` 미등록 월은 서브쿼리에 매칭 안 되어 자연스럽게 제외 → PRD §4.5.7 "소멸기한 미확정" 정책 일치
- 응답 구조체 `ExpirationReport { transitioned_count, details: Vec<ExpiredAbsenceDetail> }`
- audit `MakeupExpired` variant 신규 (T3에서 추가)

## T1' (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

T1' 작업 (0.5h):
1. `src-tauri/migrations/108__cleanup_makeup_status_check.sql` 신규
2. SQLite CHECK ALTER 미지원 → table rename + 재생성 + INSERT SELECT 패턴
3. CHECK 제약 단순화: `status = 'makeup_attended'`
4. `.sqlx/` 캐시 갱신
5. cargo test 통과 (데이터 0건이므로 안전)

T1' 완료 후 T3 (소멸 자동 전이 IPC, 4h) 진입.

## Sprint 10 Capacity 추적

- 계획: 38.5h 구현 (T1' +0.5h) + 6h 시각 검증 버퍼 = **44.5h**
- 실측 누적: T1 1.5h + T2 1h = 2.5h
- 남은 capacity: 약 42h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **마이그레이션 정책** — V108부터 (V107이 Sprint 8 마지막). 도메인 100단위 블록 유지. T1' 가 V108 사용
