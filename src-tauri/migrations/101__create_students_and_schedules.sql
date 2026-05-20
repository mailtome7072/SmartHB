-- V101: students + student_schedules (Sprint 2 T5, PRD §4.1·§4.2·§6.2, data-model §1.1·§1.2)
--
-- 변경 사유:
-- - students: 원생 마스터 — 일련번호(자동 채번 PI-05), 학교급/학년, 입퇴교일, 학교 FK
-- - student_schedules: 요일별 수업 스케줄 + 변경 이력 (effective_to NULL = 현행)
--
-- 무결성:
-- - students.serial_no UNIQUE (AC-4.1.1-3, PI-05 자동 채번 기본)
-- - students CHECK withdraw_date >= enroll_date (AC-4.1.1-4)
-- - student_schedules 부분 인덱스 UNIQUE (student_id, day_of_week) WHERE effective_to IS NULL
--   * R11 — SQLite 3.8.0+ 부분 인덱스 지원. Tauri 번들 SQLite 3.39+ (2022) 라 호환 OK
-- - student_schedules.duration_hours CHECK > 0 (PRD §4.2.1)
--
-- SSOT: data-model.md §1.1·§1.2 (sprint2.md 의 serial_no INTEGER / duration_hours REAL 표기는
-- 미스 — data-model.md 의 TEXT / INTEGER 따름)

CREATE TABLE students (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    serial_no TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    gender TEXT NOT NULL CHECK (gender IN ('male', 'female')),
    school_level TEXT NOT NULL CHECK (school_level IN ('elementary', 'middle')),
    grade INTEGER NOT NULL CHECK (grade BETWEEN 1 AND 9),
    school_id INTEGER REFERENCES schools(id),
    phone_student TEXT,
    phone_mother TEXT,
    phone_father TEXT,
    enroll_date TEXT NOT NULL,
    withdraw_date TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (withdraw_date IS NULL OR withdraw_date >= enroll_date)
);

-- 재원생 필터(withdraw_date IS NULL) 가 매우 빈번 — 부분 인덱스로 최적화.
CREATE INDEX idx_students_active ON students(withdraw_date) WHERE withdraw_date IS NULL;

CREATE TABLE student_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 1 AND 7),
    start_time TEXT NOT NULL,
    duration_hours INTEGER NOT NULL CHECK (duration_hours > 0),
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- (원생, 요일) UNIQUE — 현행 스케줄(effective_to IS NULL) 에만 적용 (PRD §6.2 정합).
CREATE UNIQUE INDEX uq_student_day_current
    ON student_schedules(student_id, day_of_week)
    WHERE effective_to IS NULL;

-- 원생별 스케줄 조회 + JOIN 가속.
CREATE INDEX idx_student_schedules_student_id ON student_schedules(student_id);
