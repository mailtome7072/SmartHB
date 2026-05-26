-- V108: makeup_attendances.status CHECK 제약 단순화 — 'makeup_absent' 제거 (Sprint 10 T1', PI-07)
--
-- 배경: Sprint 9 J5 사용자 결정으로 '보강 미등원' 개념 폐기됨. mark_makeup_absent IPC + audit
-- variant 는 Sprint 10 T1 (`dde74aa`) 에서 dead code 정리 완료. 그러나 V106 의 CHECK 제약은
-- 여전히 'makeup_absent' 를 허용 — 코드 상으로 도달 불가능하나 스키마 단계에서 명시적 차단.
--
-- 데이터 안전성: makeup_attendances 에 status='makeup_absent' 행 0건 보장 (Sprint 9 J5 폐기 후
-- 운용 데이터 미존재 + UI 호출 경로 제거). INSERT SELECT 시 데이터 누락 위험 없음.
--
-- 방법: SQLite 는 CHECK 제약 ALTER 불가 → 테이블 재생성 패턴 (V107 동일).
-- - 임시 테이블 (makeup_attendances_new) 생성 — CHECK 단순화 (`status = 'makeup_attended'`)
-- - 데이터 복사 (INSERT INTO new SELECT FROM old)
-- - 원본 DROP + RENAME
-- - 인덱스 재생성 (V106 와 동일 2개)
--
-- PRAGMA foreign_keys: 본 마이그레이션 시점에는 OFF 가정 (sqlx 표준).
-- 본 작업으로 regular_attendances.makeup_attendance_id → makeup_attendances.id FK 가 일시
-- 분리되지만, RENAME 으로 동일 이름 복원 → FK 자동 재연결 (SQLite 기본 동작).

CREATE TABLE makeup_attendances_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id),
    event_date TEXT NOT NULL,
    year_month TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'makeup_attended'
        CHECK (status = 'makeup_attended'),
    class_minutes INTEGER NOT NULL CHECK (class_minutes > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- PRD §6.2: 보강 출결은 동일 일자 다중 허용 — UNIQUE 제약 없음
    CHECK (year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]'),
    CHECK (event_date GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]-[0-3][0-9]')
);

INSERT INTO makeup_attendances_new
    (id, student_id, event_date, year_month, status, class_minutes, created_at, updated_at)
SELECT
    id, student_id, event_date, year_month, status, class_minutes, created_at, updated_at
FROM makeup_attendances;

DROP TABLE makeup_attendances;
ALTER TABLE makeup_attendances_new RENAME TO makeup_attendances;

-- 인덱스 재생성 (V106 와 동일 — 테이블 재생성 시 인덱스도 함께 소실됨)
CREATE INDEX idx_makeup_att_student ON makeup_attendances(student_id);
CREATE INDEX idx_makeup_att_yearmonth ON makeup_attendances(year_month);
