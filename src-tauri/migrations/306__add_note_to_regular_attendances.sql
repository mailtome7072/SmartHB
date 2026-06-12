-- V306: regular_attendances 에 note 컬럼 추가 (Sprint 16 T0, 케이스1 1회성 수업일 이동)
--
-- 변경 사유:
-- - 케이스1(특정일 1회성 수업일 이동, PI-20)에서 이동 내역을 셀에 표시하기 위한 메모.
--   예: "6/8(월)→6/10(수) 이동". 이동된 출결 행의 event_date 가 변경되므로 원래 일자
--   추적을 위해 텍스트 메모를 남긴다.
-- - absence_memo(결석 사유) 와는 의미가 다르므로 별도 컬럼으로 분리한다 — 이동된 행은
--   status='present'(출석) 상태를 유지하며, absence_memo 는 결석 다이얼로그 전용이다.
--
-- 방법: SQLite ADD COLUMN — regular_attendances 는 테이블 재구성 없이 컬럼만 추가.
--   (FK/제약 변경이 아니므로 V107 같은 테이블 재생성 패턴 불필요)

ALTER TABLE regular_attendances ADD COLUMN note TEXT;
