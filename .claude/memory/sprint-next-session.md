---
name: sprint-next-session
description: "Sprint 10 계획 수립 완료 (2026-05-26). 다음: /sprint-dev 10 으로 구현 진입"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint10-planning
---

Sprint 10 계획 수립 완료 (2026-05-26). Phase 3 마지막 스프린트 — 소멸 자동 전이 + 퇴교 보강 + 선행 수업 + 캘린더 뷰.

## Sprint 10 핵심 정보

- **계획 문서**: `docs/sprint/sprint10.md`
- **Task**: T1~T12 (12개, 44h = 38h 구현 + 6h 시각 검증 버퍼)
- **Phase 위치**: Phase 3 (보강 + 소멸) 두 번째이자 마지막 sprint
- **브랜치**: `sprint10` (develop 기반, 아직 미생성 — `/sprint-dev 10` 진입 시 생성)

## Task 요약

| Task | 내용 | 시간 |
|------|------|------|
| T1 | Sprint 9 dead code 정리 (A49) | 2h |
| T2 | 소멸 자동 전이 설계 + 사용자 확인 (A51) | 2h |
| T3 | 소멸 자동 전이 백엔드 IPC — expiration.rs 신규 | 4h |
| T4 | 소멸 트리거 통합 (앱 시작/출결 생성/교습기간 등록) | 3h |
| T5 | 보강소멸 → 결석 환원 IPC (AC-4.5-5) | 3h |
| T6 | 퇴교 보강 처리 IPC (§4.5.9) | 3h |
| T7 | 선행 수업 IPC (§4.2.3) | 2h |
| T8 | 캘린더 ADR + 집계 IPC (PI-03) | 4h |
| T9 | 소멸 환원/알림 UI | 3h |
| T10 | 퇴교 보강 처리 UI | 3h |
| T11 | 캘린더 뷰 UI 일/주/월 (§4.6) | 6h |
| T12 | 통합 검증 + 자동 검증 | 3h |

## 미결정 항목

- PI-03: 캘린더 라이브러리 선택 (T8에서 ADR)
- PI-04: 보강데이 일괄 등록 버튼 범위 (T11에서 사용자 확인)

## 다음 액션

```
/sprint-dev 10
```

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint10 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- **마이그레이션**: V108부터 (V107 완료)
- **시각 검증 버퍼**: 6h 별도 예약 (A50)
