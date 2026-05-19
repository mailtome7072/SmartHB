-- V001: 코드 테이블 4종 + 시드 데이터 (T9, PRD §4.1·§4.9·§6.2)
--
-- 변경 사유:
-- - schools: 원생 등록 시 학교명 자동완성, 학년 매핑
-- - payment_methods: 청구 수납 시 결제 수단 선택
-- - card_companies: 카드 결제 시 카드사 선택
-- - standard_fees: 표준 교습비 (학년별 기본 금액, 조정 시 사용자가 수정)
--
-- 무결성:
-- - schools.name UNIQUE (학교명 중복 등록 방지)
-- - payment_methods.code UNIQUE
-- - card_companies.code UNIQUE
-- - standard_fees.(grade_code) UNIQUE (학년당 표준 교습비 1건)

-- schools.name 은 UNIQUE — SQLite 가 자동으로 인덱스를 생성하므로 별도 INDEX 선언 불필요.
CREATE TABLE schools (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    school_type TEXT NOT NULL CHECK (school_type IN ('elementary', 'middle', 'high', 'etc')),
    region TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE payment_methods (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

CREATE TABLE card_companies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
);

CREATE TABLE standard_fees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    grade_code TEXT NOT NULL UNIQUE,
    grade_label TEXT NOT NULL,
    monthly_fee INTEGER NOT NULL CHECK (monthly_fee >= 0),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- 시드: 결제 수단
INSERT INTO payment_methods (code, label, display_order) VALUES
    ('cash', '현금', 1),
    ('card', '카드', 2),
    ('transfer', '계좌이체', 3),
    ('other', '기타', 9);

-- 시드: 주요 카드사 (PRD §4.9 카드 수납)
INSERT INTO card_companies (code, label, display_order) VALUES
    ('shinhan', '신한카드', 1),
    ('kookmin', '국민카드', 2),
    ('samsung', '삼성카드', 3),
    ('hyundai', '현대카드', 4),
    ('lotte', '롯데카드', 5),
    ('bc', 'BC카드', 6),
    ('hana', '하나카드', 7),
    ('woori', '우리카드', 8),
    ('nh', '농협카드', 9),
    ('citi', '씨티카드', 10),
    ('kakao', '카카오뱅크', 11),
    ('toss', '토스뱅크', 12);

-- 시드: 표준 교습비 (학년별 기본값, 사용자가 설정 메뉴에서 수정 가능)
-- 초등 ~ 고등 학년 코드: elem-1 ~ elem-6 / mid-1 ~ mid-3 / high-1 ~ high-3
INSERT INTO standard_fees (grade_code, grade_label, monthly_fee) VALUES
    ('elem-1', '초등 1학년', 150000),
    ('elem-2', '초등 2학년', 150000),
    ('elem-3', '초등 3학년', 160000),
    ('elem-4', '초등 4학년', 170000),
    ('elem-5', '초등 5학년', 180000),
    ('elem-6', '초등 6학년', 190000),
    ('mid-1', '중등 1학년', 220000),
    ('mid-2', '중등 2학년', 230000),
    ('mid-3', '중등 3학년', 240000),
    ('high-1', '고등 1학년', 260000),
    ('high-2', '고등 2학년', 270000),
    ('high-3', '고등 3학년', 280000);
