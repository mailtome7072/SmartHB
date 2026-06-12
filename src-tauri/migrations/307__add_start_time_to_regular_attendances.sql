-- V307: regular_attendances 에 start_time 컬럼 추가 (Sprint 16 T0, 케이스1 시작시간 입력)
--
-- 변경 사유:
-- - 케이스1(1회성 수업일 이동, PI-28)에서 사용자가 도착일의 수업 시작시간을 직접 입력한다.
-- - 정규 출결은 시간 컬럼이 없어 수업 캘린더가 "출결일자 요일의 현행 스케줄 start_time" 을
--   JOIN 해 표시했는데, 이동 출결은 스케줄 없는 요일이라 시간을 못 찾아 표시 불가(크래시 유발).
-- - 이동/시간지정 출결은 본 컬럼에 시작시간을 저장하고, 일반 생성 출결은 NULL 로 두어
--   캘린더가 COALESCE(ra.start_time, ss.start_time) 로 시간을 해석한다.
--
-- 형식: "HH:MM:SS" (student_schedules.start_time 과 동일). 일반 출결은 NULL.
-- 방법: SQLite ADD COLUMN — 테이블 재구성 불필요.

ALTER TABLE regular_attendances ADD COLUMN start_time TEXT;
