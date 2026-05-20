-- V104: standard_fees 재설계 — 학년별 → 주 수업시간별 (Sprint 2 T11, data-model §5.1)
--
-- 변경 사유:
-- - V001 의 standard_fees(grade_code, grade_label, monthly_fee) 학년별 모델은
--   PRD §4.9·data-model §5.1 SSOT (주 수업시간 → 교습비 매칭) 과 불일치
-- - bills.weekly_hours_snapshot 으로 청구 시점 주 수업시간을 보존하는 패턴이 SSOT
-- - 학년별 차등 교습비가 추후 필요해지면 별도 테이블(`grade_fees`) 신설
--
-- 마이그레이션 방식: DROP + CREATE (Sprint 1 시점에 사용자 데이터 없음 — 안전)

DROP TABLE IF EXISTS standard_fees;

CREATE TABLE standard_fees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    weekly_hours INTEGER NOT NULL UNIQUE CHECK (weekly_hours > 0),
    amount INTEGER NOT NULL CHECK (amount >= 0),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- 기본 시드 — 사용자가 설정 메뉴에서 자유롭게 수정/추가/삭제 가능.
-- 주 2~6시간 구간 + 표준 금액 예시.
INSERT INTO standard_fees (weekly_hours, amount, sort_order) VALUES
    (2, 150000, 1),
    (3, 200000, 2),
    (4, 250000, 3),
    (5, 300000, 4),
    (6, 350000, 5);
