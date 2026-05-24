-- V107: regular_attendances.makeup_attendance_id → makeup_attendances(id) FK 제약 추가
--
-- 배경: V106 작성 시 makeup_attendances forward reference 회피 의도로 FK 절을 누락했으나,
-- 두 테이블 모두 CREATE 된 후라면 SQLite 도 FK 절 사용 가능. Phase 3 보강 매칭 도입 전에
-- 참조 무결성 보강 필요. Sprint 8 sprint-review 발견 F2 해소.
--
-- 방법: SQLite 는 ALTER TABLE 로 FK 추가 불가 → 테이블 재생성 패턴.
-- - 임시 테이블 (regular_attendances_new) 생성 — V106 정의 + FK 절 추가
-- - 데이터 복사 (INSERT INTO new SELECT FROM old)
-- - 원본 DROP + RENAME
-- - 인덱스 재생성 (V106 와 동일 5개)
--
-- PRAGMA foreign_keys: 본 마이그레이션 시점에는 OFF 가정 (sqlx 표준). 런타임에 ON 으로
-- 설정되므로 신규 INSERT/UPDATE 부터 FK 강제.

CREATE TABLE regular_attendances_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id),
    event_date TEXT NOT NULL,
    year_month TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'present'
        CHECK (status IN ('present', 'absent', 'makeup_done', 'makeup_expired')),
    class_minutes INTEGER NOT NULL CHECK (class_minutes > 0),
    absence_memo TEXT,
    makeup_deadline TEXT,
    makeup_attendance_id INTEGER REFERENCES makeup_attendances(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (student_id, event_date),
    CHECK (year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]'),
    CHECK (event_date GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]-[0-3][0-9]'),
    CHECK (makeup_deadline IS NULL OR makeup_deadline GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]')
);

INSERT INTO regular_attendances_new
    (id, student_id, event_date, year_month, status, class_minutes,
     absence_memo, makeup_deadline, makeup_attendance_id, created_at, updated_at)
SELECT
    id, student_id, event_date, year_month, status, class_minutes,
    absence_memo, makeup_deadline, makeup_attendance_id, created_at, updated_at
FROM regular_attendances;

DROP TABLE regular_attendances;
ALTER TABLE regular_attendances_new RENAME TO regular_attendances;

-- 인덱스 재생성 (V106 와 동일 — 테이블 재생성 시 인덱스도 함께 소실됨)
CREATE INDEX idx_regular_att_student ON regular_attendances(student_id);
CREATE INDEX idx_regular_att_yearmonth ON regular_attendances(year_month);
CREATE INDEX idx_regular_att_date ON regular_attendances(event_date);
CREATE INDEX idx_regular_att_makeup ON regular_attendances(makeup_attendance_id);
