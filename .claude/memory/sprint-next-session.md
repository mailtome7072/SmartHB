---
name: sprint-next-session
description: "Sprint 9 Session #2 완료 (T1+T2, 2/9). 다음: T3 보강 등록 + 매칭 트랜잭션"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint9-session2-t2
---

Sprint 9 (Phase 3 보강 + 소멸) — T1+T2 완료, T3~T9 다음 세션.

## Sprint 9 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | PI-02 확정 + 스키마 검증 + scope.md | ✅ `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 validate 강화 | ✅ `14f583e` |
| **T3** | **백엔드 IPC — 보강 등록 + 매칭 (트랜잭션)** | ⬜ 다음 세션 |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ⬜ |
| T5 | TS IPC 래퍼 + 도메인 타입 | ⬜ |
| T6 | 보강 등록 (개별) UI | ⬜ |
| T7 | 보강데이 일괄 + 결석 라벨 (A41) | ⬜ |
| T8 | 결석 이력 조회 | ⬜ |
| T9 | 통합 검증 + A39/A40 프로세스 적용 | ⬜ |

검증 상태: `cargo test --lib` cipher off **231 passed** (T1 222 → +9) / cipher on **133 passed** / clippy --lib clean 양쪽.

## Session #2 (T2) 핵심 변경

- `src-tauri/src/commands/makeup.rs` 신규 모듈 — IPC 2종 + 응답 구조체 2종 + 테스트 8건
- `get_pending_absences(student_id)` — 미처리 결석 임박순 정렬
- `get_makeup_eligible_dates(student_id, year_month)` — `allows_makeup_class=1` 일자 + 학생 입퇴교 범위 필터 (정규 수업 요일은 T3 검증에서)
- `validate_year_month` 월 범위(01-12) 강화 + `pub(crate)` 노출 (A43)
- audit::AuditEventType 변경 없음 — T3/T4 에서 MakeupCreated/Cancelled/Absent 추가 예정

## 다음 세션 (T3) 우선 액션

1. 새 대화에서 `/sprint-dev 9` → Session #3 진입 (T3)
2. **T3 skill: karpathy-guidelines** — 보강 등록의 트랜잭션 원자성 핵심
3. T3 작업:
   - `create_makeup_with_absences(student_id, event_date, class_minutes, absence_ids: Vec<i64>) -> MakeupResult` IPC
   - 트랜잭션 내 5종 검증: 보강 가능 일자 / 결석 유효성 / 학생 일관성 / 시간값(PI-02 일 단위 — 생략) / 학생 정규 수업 요일 검사
   - audit::AuditEventType::MakeupCreated variant 추가 + `try_record`
   - 단위 테스트: 정상 매칭 / 무효 id 거부 / 이미 매칭된 결석 거부 / 트랜잭션 롤백 / 보강 불가 일자 차단

## Sprint 9 잔여 마일스톤

- T3~T4 백엔드 (11h 예상)
- T5~T8 프론트엔드 (16h 예상)
- T9 통합 검증 (3h)
- 누적: 13h / 38h 진행 (34%)

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지
- **A39 sprint-close 마이그레이션 self-check** — V108 불필요 결정을 scope.md L24-37 매트릭스에 명시 (sprint-close 통과 대비)
- **A40 sprint-review 산출물 강제** — T9 종료 후 sprint-review 가 4종 산출물 self-check
- **사용자 메모리 미러 동기화 필수**
