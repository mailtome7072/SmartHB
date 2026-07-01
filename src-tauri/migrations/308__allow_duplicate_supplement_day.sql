-- Sprint 18: 이슈 6 — '보강데이' 중복불가 해제
--
-- PRD §4.4.4에서 ON으로 설정(V301)했으나, 실사용 결과 같은 날 보강데이+공휴일 등
-- 복수 일정코드 배치가 필요한 경우가 발생함. 중복 허용으로 변경.
--
-- 적용 범위:
--   - 기존 PC: 앱 업데이트 후 첫 실행 시 자동 적용
--   - 신규 PC: V102(생성) → V301(0→1) → V308(1→0) 순서로 적용되어 최종값 0
--   - 개발/테스트: academic.rs 시드 검증 테스트 어서션도 함께 수정됨

UPDATE schedule_codes
   SET is_duplicate_blocked = 0
 WHERE code_name = '보강데이'
   AND is_system_reserved = 1;
