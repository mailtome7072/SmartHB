-- V313: 다월 교습기간 year_month 오태깅 데이터 전수 보정 (Sprint 24 E1)
--
-- 배경:
-- 다월 교습기간(예: "2026-08" = 2026-07-30 ~ 2026-09-02)의 경계일에서, 과거
-- apply_schedule_change(A1) / create_makeup(A2) 이 출결·보강의 year_month 를 달력월로
-- 오태깅했다 (예: 7/30 → "2026-07"). 이로 인해 해당 행이 소속 교습기간(8월) 그리드/집계에서
-- 누락됐다 (R136 동일 계열). 코드는 Sprint 24 에서 교습기간 기준 태깅으로 수정됐고, 여기서는
-- 이미 저장된 오태깅 데이터를 전수 교정한다.
--
-- 규칙:
-- event_date 가 속한 확정 교습기간(start_date <= event_date <= end_date, is_confirmed=1)의
-- year_month 로 교정한다. 교습기간 일자 중첩은 academic.rs IPC 레벨에서 금지되므로 날짜당
-- 확정 교습기간은 최대 1개다 (서브쿼리는 최대 1행 반환).
--
-- 안전성/멱등성:
-- - 확정 교습기간에 속하지 않는 날짜(서브쿼리 결과 NULL)는 UPDATE 대상에서 자동 제외된다
--   (SQLite 에서 `year_month != (NULL)` 은 NULL → WHERE 거짓). 기존 값 유지, 데이터 손실 없음.
--   (이런 행은 교습기간이 (재)등록되면 이후 재보정/재생성 경로에서 자연 정리된다.)
-- - "현재 값 != 올바른 값" 인 행만 갱신하므로 반복 실행해도 안전(멱등) — 2회차 실행 시 0건.
-- - UNIQUE(student_id, event_date) 제약은 event_date 를 변경하지 않으므로 충돌 없음.
-- - sqlx 는 각 마이그레이션을 트랜잭션으로 실행하므로 부분 적용이 남지 않는다.

-- 정규 출결 보정
UPDATE regular_attendances
SET year_month = (
    SELECT sp.year_month FROM study_periods sp
    WHERE sp.start_date <= regular_attendances.event_date
      AND sp.end_date >= regular_attendances.event_date
      AND sp.is_confirmed = 1
)
WHERE year_month != (
    SELECT sp.year_month FROM study_periods sp
    WHERE sp.start_date <= regular_attendances.event_date
      AND sp.end_date >= regular_attendances.event_date
      AND sp.is_confirmed = 1
);

-- 보강 출결 보정
UPDATE makeup_attendances
SET year_month = (
    SELECT sp.year_month FROM study_periods sp
    WHERE sp.start_date <= makeup_attendances.event_date
      AND sp.end_date >= makeup_attendances.event_date
      AND sp.is_confirmed = 1
)
WHERE year_month != (
    SELECT sp.year_month FROM study_periods sp
    WHERE sp.start_date <= makeup_attendances.event_date
      AND sp.end_date >= makeup_attendances.event_date
      AND sp.is_confirmed = 1
);
