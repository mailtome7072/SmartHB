---
name: sprint22-context
description: "Sprint 22 계획 수립 컨텍스트 -- 보강 분 단위 부분 차감 전환 + 출결 그리드 z-index 수정. V311/V312 마이그레이션, ADR-011"
metadata:
  type: project
---

## Sprint 22 계획 (2026-07-21)

**목표**: 보강 매칭 모델을 일 단위(PI-02)에서 분 단위 부분 차감으로 전환. 유실 데이터 자동 백필. 출결 그리드 sticky z-index 수정.

**작업 규모**: T0~T9 10개 Task, 36h 예상. Capacity 40h 이내.

**핵심 변경**:
- ADR-011: 부분 소진 스키마 설계 결정 (A안 누적 컬럼 vs B안 배분 테이블)
- V311: 스키마 변경 마이그레이션
- V312: 유실 데이터 백필 마이그레이션 (멱등)
- makeup.rs: create/cancel 부분 차감 로직
- 8개 파일 쿼리 일괄 변경 (calendar/attendance/expiration/diagnosis/students)
- MakeupRegisterDialog: 1시간 단위 선택 UI
- AttendanceGrid: z-index 층위 재정렬

**회고 반영**: A126(yearMonth prop 명확화), A114(sync_single_date 이력 패턴 통일, 4스프린트 이연 최종 해소)

**리스크**: R139(8파일 쿼리 일괄 변경 누락 위험), R140(백필 실 DB 충돌), R141(상태 모델 호환성)

관련: [[sprint21-context]], [[velocity]]
