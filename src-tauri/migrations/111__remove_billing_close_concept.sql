-- V111: 청구 '마감(closed)' 개념 제거 — bills 재구성 (post-Sprint 11, 원장 결정)
--
-- 배경: 월별 청구 마감(PRD §4.9.7) 워크플로우를 폐기한다. 청구 상태는 2단계
-- (미확정 draft → 확정 confirmed)만 남기고, 마감 전용 컬럼(close_reason, closed_at)과
-- 마감 정합 CHECK 를 제거한다. '수납완료된 청구 금액 수정 불가' 보호는 백엔드 IPC 가
-- is_paid 기준으로 계속 강제한다 (status 무관).
--
-- 방법: SQLite 는 CHECK 제약·컬럼 DROP(참조 CHECK 존재 시) ALTER 불가 → bills 재구성.
--
-- ⚠️ FK 함정 (V108 학습): payments.bill_id → bills(id) ON DELETE CASCADE 자식 FK 존재.
--   sqlx 는 마이그레이션을 트랜잭션으로 감싸고 앱 연결은 foreign_keys = ON.
--   bills 를 DROP 하면 CASCADE 로 payments 행이 함께 삭제된다(데이터 소실).
--   → payments 전체를 TEMP 에 백업하고 비운 뒤 bills 를 재구성하고 동일 id 로 복원한다.
--   payments.id / bills.id 를 모두 보존하므로 복원 시 FK 즉시 만족 (deferred 카운터 미발생).
--
-- 데이터 변환: 기존 status='closed' 행은 'confirmed' 로 흡수 (마감 = 확정의 잠금 상태였으므로).

-- 1) payments 백업 + 비우기 (CASCADE 데이터 소실 방지)
CREATE TEMP TABLE _payments_backup AS SELECT * FROM payments;
DELETE FROM payments;

-- 2) bills 재구성 — 마감 컬럼/CHECK 제거, status 2단계
CREATE TABLE bills_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id),
    bill_year_month TEXT NOT NULL,
    weekly_hours INTEGER NOT NULL CHECK (weekly_hours > 0),
    bill_amount INTEGER NOT NULL CHECK (bill_amount >= 0),
    adjusted_amount INTEGER NOT NULL CHECK (adjusted_amount >= 0),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'confirmed')),
    is_mid_month INTEGER NOT NULL DEFAULT 0 CHECK (is_mid_month IN (0, 1)),
    mid_month_type TEXT
        CHECK (mid_month_type IS NULL OR mid_month_type IN ('enrolled', 'withdrawn')),
    confirmed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (student_id, bill_year_month),
    CHECK (bill_year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]'),
    CHECK (
        (is_mid_month = 0 AND mid_month_type IS NULL) OR
        (is_mid_month = 1 AND mid_month_type IS NOT NULL)
    )
);

INSERT INTO bills_new
    (id, student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount,
     status, is_mid_month, mid_month_type, confirmed_at, created_at, updated_at)
SELECT
    id, student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount,
    CASE WHEN status = 'closed' THEN 'confirmed' ELSE status END,
    is_mid_month, mid_month_type, confirmed_at, created_at, updated_at
FROM bills;

DROP TABLE bills;
ALTER TABLE bills_new RENAME TO bills;

-- 인덱스 재생성 (V109 와 동일 — 테이블 재생성 시 인덱스 소실)
CREATE INDEX idx_bills_year_month ON bills(bill_year_month);
CREATE INDEX idx_bills_student ON bills(student_id);
CREATE INDEX idx_bills_status ON bills(status);

-- 3) payments 복원 (동일 id → FK 즉시 만족)
INSERT INTO payments SELECT * FROM _payments_backup;
DROP TABLE _payments_backup;
