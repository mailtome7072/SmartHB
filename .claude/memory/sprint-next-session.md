---
name: sprint-next-session
description: "Sprint 11 완료 (Phase 4 청구+수납). 다음: sprint-review 실행 → develop 머지 → Sprint 12 착수"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint11-close-completed
---

Sprint 11 완료 (2026-05-29). Phase 4 첫 마일스톤 (청구+수납 도메인) 완성.
sprint-close 완료. 다음 단계: sprint-review 에이전트 실행.

## 다음 액션

1. sprint-review 에이전트 실행 (코드 리뷰 + 자동 검증)
2. sprint-review 완료 후 sprint11 → develop 직접 머지
3. `pnpm tauri:dev` 로 스테이징 수동 검증 (DEPLOY.md 체크리스트)
4. Sprint 12 착수: `/sprint-dev 12`

## Sprint 11 완료 현황

- **기간**: 2026-05-28 ~ 2026-05-29
- **계획 문서**: `docs/sprint/sprint11.md`
- **Phase**: Phase 4 (청구+수납+공지문) — Sprint 11 완료, Sprint 12 착수 예정
- **Task**: T0~T9 전수 완료 (308 테스트 통과)
- **DB**: V109 마이그레이션 완료 (bills + payments + payment_methods.is_card_type)
- **carry-over**: F1~F7 7건 전수 해소

## Sprint 11 PI 결정 기록

| PI | 결정 |
|----|------|
| PI-10 마감 후 수정 사유 UX | 모달 다이얼로그 |
| PI-11 마감 해제(reopen) | 불가 (개별 건 수정만, 사유 필수) |
| PI-12 수납 테이블 | 별도 payments 테이블 |
| 카드 계열 식별 | `is_card_type` BOOLEAN 플래그 |
| F4 N+1 범위 | calendar.rs만 (attendance.rs carry-over 유지) |

## sprint-review carry-over 안내

- finding 식별: T2/T4 동적 SQL 빌드 패턴 (안전하지만 review에서 확인 권장)
- "청구 50명 3초 이내" 실측 사용자 시각 검증 미진행 — risk-register 또는 retrospective에 명시

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, develop → main 직접 머지 ([[workflow-no-pr]])
- **메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
