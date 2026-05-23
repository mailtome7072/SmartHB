# V102 schedule_codes 시드 불일치

Sprint 6 계획 수립 시 발견. V102 시드와 PRD §4.4.4 비교 결과:

| 코드 | 속성 | V102 값 | PRD 정의 | 보정 필요 |
|------|------|---------|----------|-----------|
| 보강데이 | is_duplicate_blocked | 0 | ON (1) | V301에서 수정 |
| 공휴수업일 | allows_makeup_class | 0 | ON (1) | V301에서 수정 |
| 단원평가 응시일 | is_duplicate_blocked | 1 | OFF (0) | V301에서 수정 |
| 단원평가 응시일 | is_period_type | 0 | 기간성(1) | V301에서 수정 |

**교훈**: 시드 데이터 작성 시 PRD 속성 표를 1:1 대조하여 검증해야 한다.
V301 마이그레이션에서 방어적 UPDATE (WHERE code_name + is_system_reserved = 1)로 보정.
