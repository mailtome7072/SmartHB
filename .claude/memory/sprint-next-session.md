---
name: sprint-next-session
description: "Sprint 10 Session #4 완료 (T3 소멸 자동 전이 IPC). 다음: T4 — 트리거 3개소 통합 (앱 시작/출결 생성/교습기간 등록)"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session4-t3
---

Sprint 10 — Phase 3 완결 sprint. Session #4 (T3 expiration.rs) 완료. 다음은 **T4 트리거 3개소 통합**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | Sprint 9 dead code 정리 | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + PI-05~PI-09 결정 | ✅ | `b4565d4` |
| T1' | V108 — `makeup_attendances.status` CHECK 단순화 | ✅ | `1efd70f` |
| T3 | 소멸 자동 전이 백엔드 IPC (expiration.rs) | ✅ | `616021d` |
| **T4** | **소멸 전이 트리거 통합** (앱 시작/출결 생성/교습기간 등록) | ⬜ 다음 세션 | — |
| T5 | 보강소멸 → 결석 수동 환원 IPC (AC-4.5-5) | ⬜ | — |
| T6 | 퇴교 시 미사용 보강 처리 IPC (PRD §4.5.9) | ⬜ | — |
| T7 (축소) | 선행 수업 — 출결 생성 충돌 방지 검증 only (PI-08) | ⬜ | — |
| T8 | 캘린더 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9~T11 | UI (소멸 환원 / 퇴교 보강 / 캘린더 뷰) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T3 결과 요약

| 영역 | 변동 |
|------|------|
| 신규 모듈 | `src-tauri/src/commands/expiration.rs` (~310 라인) |
| IPC | `expire_overdue_absences()` — 응답 `ExpirationReport { transitioned_count, details }` |
| 구현 함수 | `expire_overdue_absences_impl(pool, as_of: Option<NaiveDate>)` — `Local::now()` 기본값 (PI-06) |
| 핵심 SQL | UPDATE ... WHERE makeup_deadline IN (SELECT year_month FROM study_periods WHERE end_date <= ?) RETURNING ... |
| audit | `MakeupExpired` variant 추가 + 전이된 결석마다 1건 기록 |
| 단위 테스트 | 7건 (계획 6건+ 충족) |
| 자동 검증 | cargo test 258 passed (T1' 251 → +7) / clippy clean |
| 회귀 1건 | attendance.rs `summary_aggregates_completed_makeup_minutes` — V108 영향, 시드 데이터 정리로 해결 |

## T4 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

### T4 작업 계획 (예상 3h)

PI-05 결정: 트리거 3개소(앱 시작 / 출결 생성 / 교습기간 등록).

1. **앱 시작 트리거**
   - `src-tauri/src/startup.rs` 또는 `lib.rs` setup() 단계
   - DB 풀 생성 직후 `expire_overdue_absences_impl(pool, None).await` 호출
   - 결과는 startup 로그에 기록 (sync; UI는 별도 IPC로 첫 로드 시 조회)

2. **출결 생성 트리거** (`attendance.rs::generate_attendances`)
   - `generate_attendances` 종료 직전에 소멸 체크 호출
   - `GenerateResult` 응답에 `expiration_report` 필드 추가
   - 프론트엔드에서 토스트 표시 (PI-09)

3. **교습기간 등록 트리거** (`academic.rs` 또는 `schedules.rs`)
   - 교습기간 생성/수정 커맨드 종료 직전 호출
   - 응답에 동일하게 `expiration_report` 포함

4. **TS 래퍼** — `src/lib/tauri/index.ts` 에 `expireOverdueAbsences` 추가
   - 응답 타입: `ExpirationReport`
   - `src/types/expiration.ts` 신규 또는 `src/types/attendance.ts` 확장

5. **프론트엔드 초기화 시점** — layout mount 또는 첫 화면 진입 시 별도 IPC 호출 (PI-09 토스트)

### T4 단위 테스트
- 출결 생성 통합 — 동일 월에 소멸 도래 결석 있으면 expiration_report 동봉
- 교습기간 등록 통합 — 새로 등록된 월의 deadline 확정 즉시 반영

## Sprint 10 Capacity 추적

- 계획: 38.5h 구현 + 6h 시각 검증 버퍼 = **44.5h**
- 실측 누적: T1 1.5h + T2 1h + T1' 0.5h + T3 2.5h = 5.5h
- 남은 capacity: 약 39h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **PI-06 패턴** — `expire_overdue_absences_impl(pool, as_of: Option<NaiveDate>)` 시그니처를 T4 단위 테스트에서도 활용
