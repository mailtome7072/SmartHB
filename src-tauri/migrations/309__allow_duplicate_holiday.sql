-- Sprint 18: '공휴일' 중복불가 해제
--
-- 실사용 결과 공휴일 + 보강데이, 공휴일 + 공휴수업일 등 복수 일정코드를
-- 같은 날에 배치해야 하는 경우가 발생함. V308(보강데이)과 동일하게 중복 허용으로 변경.
--
-- 적용 범위: 기존 PC(앱 업데이트 후 첫 실행 자동 적용), 신규 PC(V301 이후 순차 적용)

UPDATE schedule_codes
   SET is_duplicate_blocked = 0
 WHERE code_name = '공휴일'
   AND is_system_reserved = 1;
