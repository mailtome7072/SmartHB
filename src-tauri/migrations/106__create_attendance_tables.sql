-- V106: 출결 도메인 — 정규 출결 + 보강 출결 (Sprint 8 T1, PRD §4.5·§6.2, data-model §2.4)
--
-- 변경 사유:
-- - regular_attendances: 월별 일괄 생성되는 정규 수업 출결 (원생 × 수업 요일 일자)
-- - makeup_attendances: 결석에 대한 보강 출결 — 동일 일자 다중 보강 허용
-- - 보강필요시간 = sum(absence.class_minutes) - sum(makeup_attended.class_minutes) (PRD §4.5)
--
-- 무결성:
-- - regular_attendances UNIQUE (student_id, event_date) — PRD §6.2 정규 출결 중복 방지
-- - makeup_attendances UNIQUE 없음 — PRD §6.2 보강 출결 동일 일자 다중 허용
-- - regular_attendances.status: present/absent/makeup_done/makeup_expired (4상태)
-- - makeup_attendances.status: makeup_attended/makeup_absent (2상태)
-- - regular_attendances.makeup_attendance_id → makeup_attendances.id (보강 매칭 후 연결)
-- - 소멸기한: 결석 발생 월 + 1개월 (makeup_deadline 컬럼, YYYY-MM 형식, PRD §4.5.5)
--
-- 인덱스:
-- - year_month + student_id: 출결 그리드 조회 (50명 × 31일 < 1초, PRD §5.7)

-- ────────────────────── 정규 출결 ──────────────────────
CREATE TABLE regular_attendances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id),
    event_date TEXT NOT NULL,
    year_month TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'present'
        CHECK (status IN ('present', 'absent', 'makeup_done', 'makeup_expired')),
    class_minutes INTEGER NOT NULL CHECK (class_minutes > 0),
    absence_memo TEXT,
    makeup_deadline TEXT,
    makeup_attendance_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (student_id, event_date),
    CHECK (year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]'),
    CHECK (event_date GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]-[0-3][0-9]'),
    CHECK (makeup_deadline IS NULL OR makeup_deadline GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]')
);

-- ────────────────────── 보강 출결 ──────────────────────
CREATE TABLE makeup_attendances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id),
    event_date TEXT NOT NULL,
    year_month TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'makeup_attended'
        CHECK (status IN ('makeup_attended', 'makeup_absent')),
    class_minutes INTEGER NOT NULL CHECK (class_minutes > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- PRD §6.2: 보강 출결은 동일 일자 다중 허용 — UNIQUE 제약 없음
    CHECK (year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]'),
    CHECK (event_date GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]-[0-3][0-9]')
);

-- ────────────────────── 인덱스 ──────────────────────
-- 출결 그리드 조회 (월 단위 + 원생별) — PRD §5.7 50명×31일 < 1초 보장.
CREATE INDEX idx_regular_att_student ON regular_attendances(student_id);
CREATE INDEX idx_regular_att_yearmonth ON regular_attendances(year_month);
CREATE INDEX idx_regular_att_date ON regular_attendances(event_date);
CREATE INDEX idx_makeup_att_student ON makeup_attendances(student_id);
CREATE INDEX idx_makeup_att_yearmonth ON makeup_attendances(year_month);
-- 보강 매칭용 — 보강필요시간 누적 계산 (status='absent' AND makeup_attendance_id IS NULL).
CREATE INDEX idx_regular_att_makeup ON regular_attendances(makeup_attendance_id);
