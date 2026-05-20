-- V102: study_periods + schedule_codes (Sprint 2 T6, PRD §4.4·§6.2, data-model §2.1·§2.2)
--
-- 변경 사유:
-- - study_periods: 교습기간 (월 단위) — 확정/마감 상태 관리 (AC-4.4-1)
-- - schedule_codes: 학사 일정 3속성 모델 (정규수업/보강/중복불가) + 시스템 예약 5종
--
-- 무결성:
-- - study_periods.year_month UNIQUE "YYYY-MM"
-- - PRD §6.2 교습기간 일자 중첩 금지 — 어플리케이션 레벨 검증 (Sprint 후속)
-- - schedule_codes.code_name UNIQUE
-- - AC-4.4-5: is_system_reserved = 1 행의 3속성은 어플리케이션에서 변경 차단

CREATE TABLE study_periods (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    year_month TEXT NOT NULL UNIQUE,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    is_confirmed INTEGER NOT NULL DEFAULT 0 CHECK (is_confirmed IN (0, 1)),
    is_closed INTEGER NOT NULL DEFAULT 0 CHECK (is_closed IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (end_date >= start_date),
    CHECK (year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]')
);

CREATE TABLE schedule_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code_name TEXT NOT NULL UNIQUE,
    is_system_reserved INTEGER NOT NULL DEFAULT 0 CHECK (is_system_reserved IN (0, 1)),
    allows_regular_class INTEGER NOT NULL CHECK (allows_regular_class IN (0, 1)),
    allows_makeup_class INTEGER NOT NULL CHECK (allows_makeup_class IN (0, 1)),
    is_duplicate_blocked INTEGER NOT NULL CHECK (is_duplicate_blocked IN (0, 1)),
    is_period_type INTEGER NOT NULL DEFAULT 0 CHECK (is_period_type IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- 시스템 예약 5종 시드 (PRD §4.4.4) — 사용자가 삭제 불가, 3속성 변경 차단 (AC-4.4-5)
INSERT INTO schedule_codes (code_name, is_system_reserved, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type) VALUES
    ('보강데이',         1, 0, 1, 0, 0),
    ('공휴수업일',       1, 1, 0, 1, 0),
    ('방학',             1, 0, 0, 1, 1),
    ('단원평가 응시일',  1, 1, 1, 1, 0),
    ('휴원일',           1, 0, 0, 1, 0);
