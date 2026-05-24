---
name: sprint-next-session
description: "Sprint 9 Session #3 완료 (T1+T2+T3, 3/9). 다음: T4 보강 취소 + 미등원 + 일괄"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session3-t3
---

Sprint 9 (Phase 3 보강 + 소멸) — T1+T2+T3 완료, T4~T9 다음 세션.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 확정 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 (karpathy-guidelines) | ✅ `e0e3659` |
| **T4** | **백엔드 IPC — 취소 + 미등원 + 일괄** | ⬜ 다음 세션 |
| T5 | TS IPC 래퍼 + 도메인 타입 | ⬜ |
| T6 | 보강 등록 (개별) UI | ⬜ |
| T7 | 보강데이 일괄 + 결석 라벨 (A41) | ⬜ |
| T8 | 결석 이력 조회 | ⬜ |
| T9 | 통합 검증 + A39/A40 프로세스 적용 | ⬜ |

검증 상태: `cargo test --lib` cipher off **240 passed** (T2 231 → +9) / cipher on **133 passed** / clippy --lib clean 양쪽.

## Session #3 (T3) 핵심 변경

- `create_makeup_with_absences` IPC — 트랜잭션 내 5종 검증 + INSERT makeup + UPDATE absences
  - 검증 5종: 이벤트 일자 보강 가능 / 학생 일관성 / 정규 수업 요일 차단 / 결석 유효성 (matched 우선) / PI-02 시간값 (옵션 A 일 단위로 생략)
  - race 차단: UPDATE WHERE 절에 `status='absent' AND makeup_attendance_id IS NULL` 재차 + `rows_affected=1` 검출
- `audit::AuditEventType` 신규 3 variants — `MakeupCreated/Cancelled/Absent`
- `CreateMakeupPayload` + `MakeupResult` 응답 구조체
- 테스트 9건 신규 (정상/empty/보강 불가/정규 요일/미존재/타 학생/매칭됨/롤백/입교일)

발견 사항:
- `seed_student` 의 `student_schedules.effective_from` NOT NULL 누락 → enroll_date 재사용
- 검증 4 의 matched/status 순서 — matched 우선으로 조정 (메시지 정확도)

## 다음 세션 (T4) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #4 진입 (T4)
2. T4 작업 (sprint9.md L122~141, 예상 5h):
   - `cancel_makeup(makeup_id)` — 결석 환원 트랜잭션 (`makeup_attendance_id=NULL`, `status='absent'` + `DELETE makeup_attendances`)
   - `mark_makeup_absent(makeup_id)` — 결석 상태 유지하며 보강만 'makeup_absent' 마킹
   - `batch_create_makeups(event_date, entries)` — 다중 원생 일괄 등록 + `BatchResult { succeeded, failed }` 부분 성공 처리
   - audit `MakeupCancelled` + `MakeupAbsent` 호출 (variant 는 T3 에서 이미 추가됨)
   - 단위 테스트: 취소 → 결석 환원 / 미등원 → 상태 유지 / 일괄 부분 성공/실패

## Sprint 9 잔여 마일스톤

- T4 백엔드 (5h) — 누적 19h / 38h (50%)
- T5~T8 프론트엔드 (16h)
- T9 통합 검증 (3h)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정은 scope.md 매트릭스에 명시 (sprint-close 통과 대비)
- **A40 sprint-review 산출물 강제** — T9 후 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
