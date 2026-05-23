-- V302: schedule_events.is_seeded 플래그 추가 (V16, Sprint 7 post-review)
--
-- ## 배경
--
-- V301 이 한국 법정 공휴일 64건을 자동 시드한다 (data.go.kr). 사용자가 직접 추가하는 공휴일
-- (예: 임시 휴원 등 자체 운영 공휴일)과 시드 공휴일을 동일 row 로 취급하면 사용자가 시드
-- 공휴일을 실수로 삭제할 위험이 있고, 반대로 사용자 추가 공휴일까지 삭제 차단되면 운영
-- 자유도가 떨어진다.
--
-- ## 변경
--
-- 1. `schedule_events.is_seeded INTEGER NOT NULL DEFAULT 0` 컬럼 추가 (CHECK 0/1).
-- 2. V301 시드된 공휴일 row 전체를 is_seeded=1 로 업데이트.
--    (V301 시드는 V103 신규 row INSERT 형태라, 본 마이그레이션 시점에 schedule_events 에
--     존재하는 공휴일 코드 이벤트는 모두 V301 출처. 후속 사용자 입력은 is_seeded=0 으로 신규
--     생성됨.)
--
-- ## 후속
--
-- `delete_schedule_event` 가드: 공휴일 + is_seeded=1 만 삭제 차단. is_seeded=0 사용자 추가
-- 공휴일은 일반 삭제 흐름 허용.

ALTER TABLE schedule_events
ADD COLUMN is_seeded INTEGER NOT NULL DEFAULT 0
    CHECK (is_seeded IN (0, 1));

-- V301 시드 공휴일 plug — 본 마이그레이션 시점의 모든 공휴일 코드 이벤트를 시드로 마킹.
UPDATE schedule_events
SET is_seeded = 1
WHERE code_id IN (
    SELECT id FROM schedule_codes
    WHERE code_name = '공휴일' AND is_system_reserved = 1
);
