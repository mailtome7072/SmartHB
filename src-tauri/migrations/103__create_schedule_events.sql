-- V103: schedule_events (Sprint 2 T7, PRD §4.4·§6.2, data-model §2.3)
--
-- 변경 사유:
-- - schedule_events: 학사 캘린더에 배치된 일정 — 단일 일자 또는 기간성 (방학 등)
--
-- 무결성:
-- - code_id FK → schedule_codes(id)
-- - period_end_date NULL = 단일 일자, NOT NULL = 기간성 (event_date ~ period_end_date)
-- - period_end_date >= event_date CHECK
-- - PRD §6.2 중복불가 코드 (일자, 코드) UNIQUE — Sprint 4 IPC 어플리케이션 레벨 검증
--   (부분 인덱스로 표현하기에는 is_duplicate_blocked 값이 schedule_codes 에 있어 JOIN 필요)

CREATE TABLE schedule_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code_id INTEGER NOT NULL REFERENCES schedule_codes(id),
    event_date TEXT NOT NULL,
    period_end_date TEXT,
    display_name TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (period_end_date IS NULL OR period_end_date >= event_date)
);

-- 캘린더 일자별 조회 가속 (월 단위 조회 빈번).
CREATE INDEX idx_schedule_events_date ON schedule_events(event_date);
CREATE INDEX idx_schedule_events_code ON schedule_events(code_id);
