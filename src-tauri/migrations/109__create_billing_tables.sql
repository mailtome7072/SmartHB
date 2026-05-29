-- Sprint 11 T1: 청구·수납 도메인 진입 — bills, payments 테이블 + payment_methods.is_card_type.
--
-- PRD §4.9 (청구 관리) + §4.9.5~§4.9.6 (수납) + §6.2 UNIQUE 제약 준수.
-- PI-12 확정: 별도 payments 테이블 (분할 납부/환불 확장 여지, 감사 로그 분리).
-- 카드 계열 식별: payment_methods.is_card_type BOOLEAN 플래그 (T4 카드사 필수 검증에서 사용).
--
-- 핵심 비즈니스 규칙:
-- - 청구 UNIQUE: (student_id, bill_year_month) — PRD §6.2
-- - 청구 상태 머신: draft → confirmed → closed (T3 IPC 가 백엔드에서 강제, 마이그레이션은 CHECK 만)
-- - 마감(closed) 시 closed_at 기록. close_reason 은 마감 후 수정 시 백엔드 IPC 가 강제 (NULL 허용)
-- - is_mid_month + mid_month_type 정합: 월 중 입퇴교 플래그가 1 이면 type ('enrolled'/'withdrawn') 필수
-- - payments 1:1 관계: bill_id UNIQUE + ON DELETE CASCADE
-- - is_paid=1 일 때 paid_date 필수 (CHECK 로 보장)

-- ─── 1. payment_methods 에 is_card_type 컬럼 추가 ───
-- SQLite 는 ALTER TABLE ADD COLUMN 지원. CHECK 제약은 컬럼 정의에 포함.
ALTER TABLE payment_methods
    ADD COLUMN is_card_type INTEGER NOT NULL DEFAULT 0
    CHECK (is_card_type IN (0, 1));

-- 기존 시드 중 '카드' (code='card') 만 카드 계열로 마킹.
-- 추후 '신용카드' / '체크카드' 등 분리 시드 추가 시 마이그레이션에서 함께 마킹 권장.
UPDATE payment_methods SET is_card_type = 1 WHERE code = 'card';

-- ─── 2. bills 테이블 ───
CREATE TABLE bills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES students(id),
    bill_year_month TEXT NOT NULL,
    weekly_hours INTEGER NOT NULL CHECK (weekly_hours > 0),
    bill_amount INTEGER NOT NULL CHECK (bill_amount >= 0),
    adjusted_amount INTEGER NOT NULL CHECK (adjusted_amount >= 0),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'confirmed', 'closed')),
    is_mid_month INTEGER NOT NULL DEFAULT 0 CHECK (is_mid_month IN (0, 1)),
    mid_month_type TEXT
        CHECK (mid_month_type IS NULL OR mid_month_type IN ('enrolled', 'withdrawn')),
    close_reason TEXT,
    closed_at TEXT,
    confirmed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (student_id, bill_year_month),
    CHECK (bill_year_month GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]'),
    -- 마감 정합: status='closed' ↔ closed_at NOT NULL
    CHECK ((status = 'closed') = (closed_at IS NOT NULL)),
    -- 월 중 입퇴교 플래그/타입 정합
    CHECK (
        (is_mid_month = 0 AND mid_month_type IS NULL) OR
        (is_mid_month = 1 AND mid_month_type IS NOT NULL)
    )
);
CREATE INDEX idx_bills_year_month ON bills(bill_year_month);
CREATE INDEX idx_bills_student ON bills(student_id);
CREATE INDEX idx_bills_status ON bills(status);

-- ─── 3. payments 테이블 (PI-12: 별도 테이블, 1:1 with bills) ───
CREATE TABLE payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bill_id INTEGER NOT NULL UNIQUE REFERENCES bills(id) ON DELETE CASCADE,
    is_paid INTEGER NOT NULL DEFAULT 0 CHECK (is_paid IN (0, 1)),
    paid_date TEXT,
    payer_name TEXT,
    payment_method_id INTEGER REFERENCES payment_methods(id),
    card_company_id INTEGER REFERENCES card_companies(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (paid_date IS NULL OR paid_date GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]-[0-3][0-9]'),
    -- is_paid=1 일 때 paid_date 필수
    CHECK (is_paid = 0 OR paid_date IS NOT NULL)
    -- 카드 계열 시 card_company_id 필수는 백엔드 IPC 책임 (payment_methods.is_card_type=1 일 때 검증)
);
CREATE INDEX idx_payments_bill ON payments(bill_id);
CREATE INDEX idx_payments_paid_date ON payments(paid_date);
