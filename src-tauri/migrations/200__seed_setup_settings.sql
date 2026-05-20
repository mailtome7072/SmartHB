-- V200: 초기 설정 마법사 (PRD §4.0, Sprint 3 T8)
--
-- app_settings 에 마법사 관련 키를 시드한다. 실제 값은 마법사 진행 중 setup IPC 가 갱신.
--
-- ## chicken-and-egg 회피
--
-- 클라우드 폴더 경로는 OS app_config_dir 의 config.json 에 우선 저장된다 (DB 열기 전 필요).
-- 본 테이블의 cloud_folder_path 는 unlock 이후 보조 메타데이터로만 유지 — 양 PC 가 같은
-- 클라우드 폴더를 공유하므로 디버그·점검용으로 동기화된 값을 확인할 수 있다.

INSERT INTO app_settings (key, value) VALUES
    ('setup_completed', 'false'),
    ('cloud_folder_path', '');
