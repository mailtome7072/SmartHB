---
name: sprint-next-session
description: "Sprint 11 계획 확정 (Phase 4 청구+수납). 다음: /sprint-dev 11 진입"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint11-plan-confirmed
---

Sprint 11 계획 확정 (2026-05-28). Phase 4 (청구+수납) 첫 스프린트. PI 5건 전수 결정 완료.

## 다음 액션

> `/sprint-dev 11` 커맨드로 구현 단계 진입

## Sprint 11 개요

- **계획 문서**: `docs/sprint/sprint11.md`
- **Phase**: Phase 4 (청구+수납+공지문) -- Sprint 11~12
- **Task**: T0~T9 (10개, 30.5h / 34h capacity)
- **핵심 도메인**: 청구 3단계 상태 머신 (draft/confirmed/closed) + 수납 + 입금 일괄 처리
- **DB**: V109 마이그레이션 (bills + payments + codes.is_card_type)
- **carry-over**: T0에서 F1~F7 (7건) 일괄 해소

## PI 결정 완료 (2026-05-28)

| PI | 결정 |
|----|------|
| PI-10 마감 후 수정 사유 UX | 모달 다이얼로그 |
| PI-11 마감 해제(reopen) | 불가 (개별 건 수정만, 사유 필수) |
| PI-12 수납 테이블 | 별도 payments 테이블 |
| 카드 계열 식별 | `is_card_type` BOOLEAN 플래그 |
| F4 N+1 범위 | calendar.rs만 (attendance.rs carry-over 유지) |

## 직전 마일스톤

- Phase 3 완료 (2026-05-28), v0.5.0 GitHub Release 배포 완료
- Sprint 10 + hotfix 4건 develop 머지 완료

## 정책 (재확인)

- **PR 단계 생략** -- 단일 개발자, develop -> main 직접 머지 ([[workflow-no-pr]])
- **메모리 미러 동기화** -- 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
