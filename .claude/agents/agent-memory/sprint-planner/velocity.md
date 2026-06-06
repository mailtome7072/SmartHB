---
name: velocity
description: 과거 스프린트 Velocity 데이터 -- Capacity 추정 시 참조
metadata:
  type: project
---

## Velocity 관찰 (1인 AI 페어 프로그래밍)

| Sprint | Task 수 | 핵심 특성 | 비고 |
|--------|---------|----------|------|
| 11 | T0-T9 (10) | 청구+수납 백엔드+프론트 + carry-over 7건 | 대규모, post-develop 보완 7커밋 추가 |
| 12 | T0-T9 (10) | 공지문 이미지 Canvas + PIN UI 통일 + scope 외 3건 | 중간, scope 초과 발생 |
| 13 | T0-T6+ 검수 추가 (8) | PIN 옵션화 + Phase 5 취소 + carry-over | 소형, stale carry-over 3건 발견 |

## Capacity 기준
- 1인 x 10일 x 4h/일 = 40시간
- Task 8~10개가 적정 (하루 1~1.5 Task)
- carry-over + 통합 검증은 항상 2~3h 예산 확보

## 교훈
- carry-over 항목은 계획 전 코드 현황 확인 필수 (A90, Sprint 13 회고)
- post-develop 보완 커밋이 매 스프린트 발생 -- 시각 검증 후 UX 보강 예산 2~3h 별도 확보 권장
