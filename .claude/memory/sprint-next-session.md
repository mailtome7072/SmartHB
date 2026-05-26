---
name: sprint-next-session
description: "Sprint 10 Session #5 완료 (T4 트리거 3개소 통합). 다음: T5 — 보강소멸 → 결석 수동 환원 IPC"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session5-t4
---

Sprint 10 — Phase 3 완결 sprint. Session #5 (T4 트리거 통합) 완료. 다음은 **T5 보강소멸 → 결석 수동 환원 IPC (AC-4.5-5)**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 결정 | ✅ | `b4565d4` |
| T1' | V108 — `makeup_attendances.status` CHECK 단순화 | ✅ | `1efd70f` |
| T3 | 소멸 자동 전이 백엔드 IPC (expiration.rs) | ✅ | `616021d` |
| T4 | 소멸 전이 트리거 3개소 통합 | ✅ | `6b6cc47` |
| **T5** | **보강소멸 → 결석 수동 환원 IPC** (AC-4.5-5) | ⬜ 다음 세션 | — |
| T6 | 퇴교 시 미사용 보강 처리 IPC (PRD §4.5.9) | ⬜ | — |
| T7 (축소) | 선행 수업 — 출결 생성 충돌 방지 검증 only (PI-08) | ⬜ | — |
| T8 | 캘린더 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9~T11 | UI (소멸 환원 / 퇴교 보강 / 캘린더 뷰) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T4 결과 요약

| 영역 | 변동 |
|------|------|
| 백엔드 트리거 | attendance::generate_impl (트랜잭션 커밋 후), academic::create/update/confirm_study_period, startup::app_startup_sequence (fail-soft) |
| 응답 wrapper 신규 | academic::StudyPeriodResult { study_period, expiration_report } — camelCase serde rename |
| 응답 필드 추가 | GenerateResult.expiration_report, StartupResult.expiration_report |
| TS 타입 신규 | src/types/expiration.ts (ExpirationReport, ExpiredAbsenceDetail) |
| TS 래퍼 변경 | createStudyPeriod/updateStudyPeriod/confirmStudyPeriod 반환 타입 StudyPeriodResult + expireOverdueAbsences 신규 |
| UI 토스트 | attendance/page.tsx (amber-50, 닫기 버튼) + StudyPeriodEditor.tsx (AlertDialog 재활용) |
| 단위 테스트 | generate_includes_expiration_report_when_deadline_reached (응답 필드 검증) |
| 자동 검증 | cargo test 259 passed / clippy / lint / tsc / build all clean |

## T5 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T5 작업 계획 (예상 3h, sprint10.md T5 AC)

1. **신규 함수** `expiration.rs::revert_expired_to_absent_impl(pool, attendance_id) -> Result<()>`
   - 검증: 현재 status='makeup_expired' 인지 확인 (다른 상태면 거부)
   - UPDATE: `status='absent'`, `makeup_deadline` 재설정 (현재 월 + 1 또는 원래 값 복원)
   - audit: `MakeupExpiredReverted` variant 추가

2. **IPC** `revert_expired_to_absent(attendance_id)` 등록 + lib.rs invoke_handler

3. **TS 래퍼** `revertExpiredToAbsent` 추가 (src/lib/tauri/index.ts)

4. **단위 테스트 4건+** (sprint10.md T5 AC):
   - makeup_expired → absent 환원 성공
   - absent/present/makeup_done 상태에서 호출 → 거부
   - 환원 후 makeup_deadline 재설정 확인
   - audit 로그 기록 확인

### T5 설계 결정 필요 항목
- **새 deadline 계산**: (a) 환원 시점 = 오늘 → 다음 월 / (b) 원래 deadline 값 복원
- 원래 deadline 복원이 자연스러움 (사용자가 보강 등록을 위해 환원하는 흐름이므로 원래 기한 유지). 그러나 deadline 도래 후 환원이라면 원래값을 복원해도 다음 트리거에서 즉시 다시 소멸됨 — 일종의 무한 루프 가능. (a) 가 더 안전 — 환원 시 deadline 을 한 달 더 연장하여 사용자가 보강할 시간 부여.
- T5 진입 시 사용자 확인 권장 (PI-10 신규)

## Sprint 10 Capacity 추적

- 계획: 38.5h 구현 + 6h 시각 검증 버퍼 = **44.5h**
- 실측 누적: T1 1.5h + T2 1h + T1' 0.5h + T3 2.5h + T4 3h = **8.5h**
- 남은 capacity: 약 36h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **PI-06 패턴 유지** — `expire_overdue_absences_impl(pool, as_of: Option<NaiveDate>)` 시그니처가 트리거 3개소에서 잘 동작 확인됨. T5도 동일한 패턴 적용
