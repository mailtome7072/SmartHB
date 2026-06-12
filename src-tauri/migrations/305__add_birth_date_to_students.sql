-- 원생 생년월일 (선택 입력) — Sprint 14 검증 중 사용자 요청.
-- ISO 8601 (YYYY-MM-DD) 문자열. 기존 원생은 NULL (미입력 허용).
ALTER TABLE students ADD COLUMN birth_date TEXT;
