---
name: sprint-next-session
description: "Sprint 10 Session #1 완료 (T1 dead code 정리). 다음: T2 소멸 자동 전이 설계 + 사용자 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-session1-t1
---

Sprint 10 — Phase 3 완결 sprint. Session #1 (T1) 완료. 다음은 **T2 소멸 자동 전이 설계 + 사용자 확인**.

## Sprint 10 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| **T1** | Sprint 9 dead code 정리 (A49) | ✅ | `dde74aa` |
| T2 | 소멸 자동 전이 설계 + 사용자 확인 (A51) | ⬜ 다음 세션 | — |
| T3 | 소멸 자동 전이 백엔드 IPC | ⬜ | — |
| T4 | 소멸 전이 트리거 통합 (앱 시작/출결 생성/교습기간 등록) | ⬜ | — |
| T5 | 보강소멸 → 결석 수동 환원 IPC (AC-4.5-5) | ⬜ | — |
| T6 | 퇴교 시 미사용 보강 처리 IPC (PRD §4.5.9) | ⬜ | — |
| T7 | 선행 수업 처리 IPC (PRD §4.2.3) | ⬜ | — |
| T8 | 캘린더 라이브러리 ADR (PI-03) + 백엔드 집계 IPC | ⬜ | — |
| T9~T11 | UI (소멸 환원 / 퇴교 보강 / 캘린더 뷰) | ⬜ | — |
| T12 | 통합 검증 | ⬜ | — |

## T1 결과 요약

| 영역 | 변동 |
|------|------|
| 백엔드 | `mark_makeup_absent` + `batch_create_makeups` 함수/impl/IPC 핸들러 + payload struct 4종 + 단위 테스트 5건 삭제 |
| audit | `AuditEventType::MakeupAbsent` variant + 매핑 제거 |
| 자동 검증 | cargo test 251 passed (Sprint 9 256 → -5, 삭제 테스트 수와 일치) / clippy clean |
| TS 영향 | Sprint 9 T12에서 이미 정리됨 — 추가 수정 없음 |

**미수행 (T2 진입 시 판단)**: V108 마이그레이션 (`makeup_attendances.status` CHECK 제약에서 `makeup_absent` 값 제거).

## T2 (다음 세션) 진입 액션

새 대화 또는 같은 세션에서:

> "/sprint-dev 10"

T2 작업 (예상 2h):
1. 소멸 전이 트리거 3개소 설계서 작성 (scope.md Session #2)
   - 앱 시작 시 batch
   - 출결 생성 시 (`generate_attendances` 내부)
   - 교습기간 등록 직후
2. 소멸기한 판정 로직 확정: `makeup_deadline`(년월) + 해당 년월의 교습기간 종료일
3. 선행 수업(PRD §4.2.3) 운용 시나리오 확인
4. **사용자 확인 필요 항목** (A51 패턴):
   - 교습기간 미등록 월의 소멸 처리 방식 (대기 vs 즉시 소멸)
   - 선행 수업 등록 시 출결 생성 충돌 방지 정책 (R69)
   - V108 마이그레이션 필요 여부 (CHECK 제약 정리)
5. scope.md Session #2 작성 + 본 메모리 갱신

## Sprint 10 Capacity 추적

- 계획 38h 구현 + 6h 시각 검증 버퍼 = 44h
- 실측: T1 약 1.5h (계획 2h 대비 -0.5h)
- 남은 capacity: 약 42.5h

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **마이그레이션 정책** — V108부터 (V107이 Sprint 8 마지막). 도메인 100단위 블록 유지
