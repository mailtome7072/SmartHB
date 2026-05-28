-- V108: makeup_attendances.status CHECK 제약 단순화 — 'makeup_absent' 제거 (Sprint 10 T1', PI-07)
--
-- 배경: Sprint 9 J5 사용자 결정으로 '보강 미등원' 개념 폐기됨. mark_makeup_absent IPC + audit
-- variant 는 Sprint 10 T1 (`dde74aa`) 에서 dead code 정리 완료. 그러나 V106 의 CHECK 제약은
-- 여전히 'makeup_absent' 를 허용 — 코드 상으로 도달 불가능하나 스키마 단계에서 명시적 차단.
--
-- 데이터 안전성: makeup_attendances 에 status='makeup_absent' 행 0건 보장 (Sprint 9 J5 폐기 후
-- 운용 데이터 미존재 + UI 호출 경로 제거). INSERT SELECT 시 데이터 누락 위험 없음.
--
-- 방법: SQLite 는 CHECK 제약 ALTER 불가 → 부모 테이블(makeup_attendances) 재생성.
--
-- ⚠️ FK 카운터 함정 (Sprint 10 T11 시각 검증에서 실데이터 code 787 발견):
--   regular_attendances.makeup_attendance_id → makeup_attendances.id 자식 FK 가 존재한다.
--   sqlx 는 마이그레이션을 트랜잭션으로 감싸고 앱 연결은 `PRAGMA foreign_keys = ON` 이다.
--   - `PRAGMA foreign_keys = OFF` 는 트랜잭션 내부에서 무시됨 (SQLite 공식 재구성 절차는 BEGIN
--     '밖'에서 OFF 를 요구 — sqlx 모델에선 불가).
--   - `PRAGMA defer_foreign_keys = ON` 도 실패: DROP 의 암묵적 DELETE 가 deferred 위반 카운터를
--     +1 하는데, 부모 행을 INSERT 한 시점엔 테이블 이름이 makeup_attendances_new 라 카운터가
--     감소하지 않고 RENAME 으로도 감소 안 됨 → COMMIT 시 카운터>0 → code 787.
--     (foreign_key_check 는 0건이지만 deferred 카운터가 남아 실패하는 SQLite 동작)
--
-- 해결: 재구성 동안 dangling 참조 자체를 없앤다.
--   1) 자식 FK 값(ra_id→mk_id)을 TEMP 테이블에 보존하고 NULL 로 비운다 (NULL FK 는 검사 제외)
--   2) 부모 테이블 재구성 (이제 참조하는 자식 없음 → DROP/RENAME 안전, 카운터 증가 없음)
--   3) 보존한 값으로 자식 FK 복원 (부모 행이 동일 id 로 존재 → 즉시 FK 만족)
--   foreign_keys ON + 트랜잭션 내부에서 전 구간 정합 유지. (Perl DBD::SQLite 로 실데이터 재현·검증)

-- 1) 자식 FK 보존 + NULL
CREATE TEMP TABLE _ra_makeup_map AS
    SELECT id AS ra_id, makeup_attendance_id AS mk_id
    FROM regular_attendances
    WHERE makeup_attendance_id IS NOT NULL;

UPDATE regular_attendances
SET makeup_attendance_id = NULL
WHERE makeup_attendance_id IS NOT NULL;

-- 2) 부모 테이블 재구성 — CHECK 단순화 (`status = 'makeup_attended'`)
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

-- 3) 자식 FK 복원
UPDATE regular_attendances
SET makeup_attendance_id = (
    SELECT mk_id FROM _ra_makeup_map WHERE ra_id = regular_attendances.id
)
WHERE id IN (SELECT ra_id FROM _ra_makeup_map);

DROP TABLE _ra_makeup_map;
