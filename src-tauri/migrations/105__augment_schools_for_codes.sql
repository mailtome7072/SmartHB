-- V105: schools 테이블에 코드 관리용 컬럼 추가 (Sprint 2 T12, data-model §1.3)
--
-- 변경 사유:
-- - V001 schools 는 (id, name UNIQUE, school_type CHECK, region, created_at) 만 가짐
-- - data-model §1.3 SSOT 는 sort_order / is_active / updated_at 도 요구
-- - 코드 테이블 CRUD IPC (T12) 가 정렬 변경 + 소프트 삭제(is_active) 를 사용
--
-- SQLite ALTER TABLE ADD COLUMN 제약:
-- - NOT NULL 컬럼은 DEFAULT 값 필수 → 본 마이그레이션은 모두 DEFAULT 지정
-- - 기존 school_type / region 컬럼은 보존 (PRD §4.1.1 학교급 자동완성 등에 활용)

ALTER TABLE schools ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
ALTER TABLE schools ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1));
ALTER TABLE schools ADD COLUMN updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
