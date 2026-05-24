---
name: capacity-pattern
description: 스프린트별 Capacity 대비 실제 소요 시간 패턴. 계획 수립 시 참조.
metadata:
  type: project
---

| Sprint | Capacity | 예상 소요 | 초과/여유 | 비고 |
|--------|----------|----------|----------|------|
| Sprint 6 | 40h | 45h | -12.5% 초과 | 학사 캘린더 복잡도 과소추정 |
| Sprint 7 | 40h | 33h | +17.5% 여유 | carry-over 집중, 기존 코드 수정 위주 |
| Sprint 8 | 40h | 41h | -2.5% 초과 | 본 작업(26h) + carry-over(12h) + 검증(3h) |

**Why:** 1인 개발자 기준 하루 4시간, 2주 10영업일 = 40시간이 표준 capacity.
**How to apply:** 40h를 초과하면 Low 이슈 이연 검토. 본 작업:carry-over 비율이 2:1 이상이면 적정.
