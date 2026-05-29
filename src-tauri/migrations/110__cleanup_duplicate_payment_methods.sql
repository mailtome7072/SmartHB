-- Sprint 11 post-merge hotfix: payment_methods 중복 시드 정리.
--
-- V001 + V201 누적으로 같은 label 의 row 가 2건씩 존재:
--   id=4 code='other'         label='결제선생'  ← V201 에서 label 변경 (옛 'other'/'기타' 잔존)
--   id=5 code='locCash'       label='성남사랑'  ← V201 에서 추가
--   id=6 code='pay_teacher'   label='결제선생'  ← V201 에서 추가 (id=4 와 label 중복)
--   id=7 code='seongnam_love' label='성남사랑'  ← V201 에서 추가 (id=5 와 label 중복)
--
-- 정합: 새로 추가된 의미 명확한 code (pay_teacher, seongnam_love) 만 유지.
-- 옛 code (other, locCash) 는 삭제 — 사용자 결정 (2026-05-29 post-Sprint 11).
--
-- 안전성: payments 테이블이 비어있는 시점 (Sprint 11 출시 직전) 에 적용 → FK 영향 없음.
-- 이미 payments 가 참조 중인 환경에서는 본 마이그레이션이 FK 에러로 실패 — 운영 환경 식별 후
-- 별도 데이터 마이그레이션 (UPDATE payments SET payment_method_id = ...) 선행 필요.

DELETE FROM payment_methods WHERE code IN ('other', 'locCash');
