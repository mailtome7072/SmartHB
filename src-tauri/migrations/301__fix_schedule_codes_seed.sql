-- V301: schedule_codes 시드 보정 + "공휴일" 시스템 코드 + 한국 법정 공휴일 시드
-- (Sprint 6 T2-b, PRD §4.4.4·§6.2, ADR-005)
--
-- 변경 사유:
-- 1) V102 schedule_codes 시드의 3속성이 PRD §4.4.4 와 불일치 — 보정 필요:
--    - 보강데이:        is_duplicate_blocked 0 → 1 (PRD ON)
--    - 공휴수업일:      allows_makeup_class  0 → 1 (PRD ON)
--    - 단원평가 응시일: is_duplicate_blocked 1 → 0, is_period_type 0 → 1 (PRD 기간성)
-- 2) ADR-005 결정 — "공휴일" 시스템 예약 코드 추가 (캘린더 통합 표시)
-- 3) 한국 법정 공휴일 2025~2027 (64건) 시드 — `pnpm holidays:fetch` 산출물 (data.go.kr)
--
-- 방어적 적용 (AC-T2-2):
-- - UPDATE: WHERE code_name + is_system_reserved=1 둘 다 일치 시에만 — 사용자가 코드명 변경한 경우 영향 없음
-- - INSERT OR IGNORE: 코드명 UNIQUE 위반 시 무시 (재실행/사용자 임의 추가 보호)
-- - 공휴일 INSERT: schedule_codes 에 "공휴일" 시스템 코드가 존재할 때만 (WHERE c.code_name='공휴일' AND c.is_system_reserved=1)
--
-- 갱신 (ADR-005):
-- - data.go.kr 특일 정보 API 는 1~2년치 미래 데이터만 사전 발표 — 2028+ 는 추후 발표
-- - 2026-12 이전 `pnpm holidays:fetch -- --years 2028-2030` 재실행 후 V401(+) 신규 마이그레이션 추가

-- ── 1단계: 시스템 예약 코드 3속성 보정 ────────────────────────────────────
UPDATE schedule_codes SET is_duplicate_blocked = 1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE code_name = '보강데이' AND is_system_reserved = 1 AND is_duplicate_blocked = 0;

UPDATE schedule_codes SET allows_makeup_class = 1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE code_name = '공휴수업일' AND is_system_reserved = 1 AND allows_makeup_class = 0;

UPDATE schedule_codes SET is_duplicate_blocked = 0, is_period_type = 1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE code_name = '단원평가 응시일' AND is_system_reserved = 1
      AND (is_duplicate_blocked != 0 OR is_period_type != 1);

-- ── 2단계: "공휴일" 시스템 코드 추가 (ADR-005) ─────────────────────────
-- 속성: 정규수업 OFF / 보강 OFF / 중복불가 ON / 단일 일자 — 사용자 3속성 수정 차단 (AC-4.4-5)
INSERT OR IGNORE INTO schedule_codes
    (code_name, is_system_reserved, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type)
VALUES
    ('공휴일', 1, 0, 0, 1, 0);

-- ── 3단계: 한국 법정 공휴일 2025~2027 시드 (64건, data.go.kr) ──────────
-- 생성: pnpm holidays:fetch -- --years 2025-2027
-- 동일 일자 다중 공휴일(예: 2025-05-05 어린이날·부처님오신날)은 별도 행으로 입력 —
-- V103 (code_id, event_date) UNIQUE 없음, 어플리케이션 IPC 가드만 적용.
-- SQLite VALUES 인라인 alias 미지원 — column1/column2 자동 명명 사용.
INSERT INTO schedule_events (code_id, event_date, period_end_date, display_name)
SELECT c.id, v.column1, NULL, v.column2
FROM schedule_codes c
CROSS JOIN (VALUES
  ('2025-01-01', '1월1일'),
  ('2025-01-27', '임시공휴일'),
  ('2025-01-28', '설날'),
  ('2025-01-29', '설날'),
  ('2025-01-30', '설날'),
  ('2025-03-01', '삼일절'),
  ('2025-03-03', '대체공휴일'),
  ('2025-05-05', '어린이날'),
  ('2025-05-05', '부처님오신날'),
  ('2025-05-06', '대체공휴일'),
  ('2025-06-03', '임시공휴일(제21대 대통령 선거)'),
  ('2025-06-06', '현충일'),
  ('2025-08-15', '광복절'),
  ('2025-10-03', '개천절'),
  ('2025-10-05', '추석'),
  ('2025-10-06', '추석'),
  ('2025-10-07', '추석'),
  ('2025-10-08', '대체공휴일'),
  ('2025-10-09', '한글날'),
  ('2025-12-25', '기독탄신일'),
  ('2026-01-01', '1월1일'),
  ('2026-02-16', '설날'),
  ('2026-02-17', '설날'),
  ('2026-02-18', '설날'),
  ('2026-03-01', '삼일절'),
  ('2026-03-02', '대체공휴일(삼일절)'),
  ('2026-05-01', '노동절'),
  ('2026-05-05', '어린이날'),
  ('2026-05-24', '부처님오신날'),
  ('2026-05-25', '대체공휴일(부처님오신날)'),
  ('2026-06-03', '전국동시지방선거'),
  ('2026-06-06', '현충일'),
  ('2026-07-17', '제헌절'),
  ('2026-08-15', '광복절'),
  ('2026-08-17', '대체공휴일(광복절)'),
  ('2026-09-24', '추석'),
  ('2026-09-25', '추석'),
  ('2026-09-26', '추석'),
  ('2026-10-03', '개천절'),
  ('2026-10-05', '대체공휴일(개천절)'),
  ('2026-10-09', '한글날'),
  ('2026-12-25', '기독탄신일'),
  ('2027-01-01', '1월1일'),
  ('2027-02-06', '설날'),
  ('2027-02-07', '설날'),
  ('2027-02-08', '설날'),
  ('2027-02-09', '대체공휴일(설날)'),
  ('2027-03-01', '삼일절'),
  ('2027-05-01', '노동절'),
  ('2027-05-05', '어린이날'),
  ('2027-05-13', '부처님오신날'),
  ('2027-06-06', '현충일'),
  ('2027-07-17', '제헌절'),
  ('2027-08-15', '광복절'),
  ('2027-08-16', '대체공휴일(광복절)'),
  ('2027-09-14', '추석'),
  ('2027-09-15', '추석'),
  ('2027-09-16', '추석'),
  ('2027-10-03', '개천절'),
  ('2027-10-04', '대체공휴일(개천절)'),
  ('2027-10-09', '한글날'),
  ('2027-10-11', '대체공휴일(한글날)'),
  ('2027-12-25', '기독탄신일'),
  ('2027-12-27', '대체공휴일(기독탄신일)')
) AS v
WHERE c.code_name = '공휴일' AND c.is_system_reserved = 1;
