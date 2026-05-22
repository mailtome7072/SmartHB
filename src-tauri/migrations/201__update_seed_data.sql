-- V201: 표준교습비 + 결제수단 시드 데이터 보정 (Sprint 5 T3+T4)
--
-- 배경: V104 의 standard_fees(2~6시간 5종, 15~35만원) 및 V001 의 payment_methods
-- (cash/card/transfer/other) 기본 시드를 실제 교습소 운영 값으로 변경한다.
--
-- 멱등성: 사용자가 이미 수정했을 수 있으므로, V104/V001 의 정확한 baseline 값과
-- 일치하는 행만 변경한다. pre-release 시점(실 사용자 데이터 없음)이지만 안전 우선.
--
-- 변경 후 목표 상태:
--   standard_fees: (3, 160000, 1), (4, 200000, 2), (5, 230000, 3), (6, 260000, 4)  -- 2시간 삭제
--   payment_methods: cash(비활성), transfer(2), card(3), pay_teacher(4), seongnam_love(5)  -- other 삭제

-- ── T3: standard_fees 시드 보정 ────────────────────────────────────────────────

-- V104 baseline (weekly_hours=2, amount=150000, sort_order=1) 일치 시에만 삭제
DELETE FROM standard_fees
 WHERE weekly_hours = 2 AND amount = 150000 AND sort_order = 1;

-- 나머지 4행: V104 baseline 금액·정렬 일치 시에만 신규 값으로 갱신
UPDATE standard_fees
   SET amount = 160000, sort_order = 1
 WHERE weekly_hours = 3 AND amount = 200000 AND sort_order = 2;

UPDATE standard_fees
   SET amount = 200000, sort_order = 2
 WHERE weekly_hours = 4 AND amount = 250000 AND sort_order = 3;

UPDATE standard_fees
   SET amount = 230000, sort_order = 3
 WHERE weekly_hours = 5 AND amount = 300000 AND sort_order = 4;

UPDATE standard_fees
   SET amount = 260000, sort_order = 4
 WHERE weekly_hours = 6 AND amount = 350000 AND sort_order = 5;

-- ── T4: payment_methods 시드 보정 ─────────────────────────────────────────────

-- 'cash' V001 baseline 일치 시 비활성화
UPDATE payment_methods
   SET is_active = 0
 WHERE code = 'cash' AND label = '현금' AND display_order = 1 AND is_active = 1;

-- 'other' V001 baseline (display_order=9) 일치 시 삭제
DELETE FROM payment_methods
 WHERE code = 'other' AND label = '기타' AND display_order = 9 AND is_active = 1;

-- 'transfer'/'card' display_order 재정렬 (V001: card=2, transfer=3 → 목표: transfer=2, card=3)
UPDATE payment_methods
   SET display_order = 2
 WHERE code = 'transfer' AND label = '계좌이체' AND display_order = 3 AND is_active = 1;

UPDATE payment_methods
   SET display_order = 3
 WHERE code = 'card' AND label = '카드' AND display_order = 2 AND is_active = 1;

-- 신규 결제 수단 2종 추가 (UNIQUE code 제약 → OR IGNORE 로 멱등성 보장)
INSERT OR IGNORE INTO payment_methods (code, label, display_order, is_active) VALUES
    ('pay_teacher', '결제선생', 4, 1),
    ('seongnam_love', '성남사랑', 5, 1);
