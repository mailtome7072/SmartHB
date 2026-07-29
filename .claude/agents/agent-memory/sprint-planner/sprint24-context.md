---
name: sprint24-context
description: Sprint 24 계획 수립 컨텍스트 -- 다월 교습기간 year_month 오태깅 9건 일괄 수정 + V313 데이터 전수 보정 + 자가진단 검사 + 회귀 테스트. A114 7회 이연 최종 해소.
metadata:
  type: project
---

## Sprint 24 핵심 컨텍스트

**현재 스프린트**: Sprint 24 (Post-v1.5, v1.5.0 프로덕션 배포 완료 상태)
**근본 원인**: 코드가 "날짜의 달력월 = 교습기간 year_month"라고 가정하는 단일 안티패턴. Sprint 21 R136에서 generate_impl/sync_single_date만 수정, 나머지 잔존.

**정답 패턴**: `attendance.rs:1547-1555` sync_single_date의 `SELECT year_month FROM study_periods WHERE start_date <= date AND end_date >= date AND is_confirmed=1`

**수정 대상 9건**: A1(apply_schedule_change), A2(create_makeup), B1(move_attendance), B2(build_day_schedules), B3(notice), B4(dashboard), B5(makeup_eligible), B6(reinstate_student), B7(academic 가드 -- 설계 판단 필요), B8(프론트 MakeupRegisterDialog)

**데이터 보정**: V313 마이그레이션 (현재 최신 V312, 300대 보정 블록)

**회고 액션 반영**: A114 (sync_single_date 이력 패턴, 7회 이연) T0 강제 포함. A127/A128/A129/A130/A131/A132는 범위 밖 유지.

**Why:** 실사용 중 schedule-editor 저장 시 A1 버그가 발동하여 올바른 출결 행까지 오태깅으로 덮어쓰는 능동 오염 발생. 경계일에서만 어긋나므로 단일월 교습기간에서는 발현되지 않음.

**How to apply:** 모든 수정 대상에 동일한 study_periods 조회 패턴 적용. 청구/수납은 SAFE(이미 study_periods 직접 조회) -- 코드 변경 불필요, 회귀 테스트만.
