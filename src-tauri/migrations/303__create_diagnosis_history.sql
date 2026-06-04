-- V303: diagnosis_history — 데이터 자가 진단 이력 (Sprint 14 T1, PRD §6.6)
--
-- 매월 1일 첫 실행 시 자동 + 사용자 수동 실행되는 자가 진단 결과를 보관한다.
-- 최근 12개월 이력만 유지하며, 초과분 정리는 run_diagnosis IPC 가
-- `DELETE WHERE run_date < date('now', '-12 months')` 로 수행한다 (AC-6.6-4).
--
-- details: 검사별 발견 항목 JSON 배열
--   [{check_id, severity, message, target_table, target_id}]
--
-- 번호 정책: 300번대 도메인 확장 블록 연속 (301 schedule_codes 보정, 302 is_seeded, 303 진단 이력).

CREATE TABLE diagnosis_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_date TEXT NOT NULL,            -- ISO 8601 (YYYY-MM-DD)
    run_type TEXT NOT NULL CHECK (run_type IN ('auto', 'manual')),
    total_checks INTEGER NOT NULL CHECK (total_checks >= 0),
    issues_found INTEGER NOT NULL CHECK (issues_found >= 0),
    details TEXT NOT NULL,             -- JSON 배열
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (run_date GLOB '[0-9][0-9][0-9][0-9]-[0-1][0-9]-[0-3][0-9]')
);

-- 이력 조회(최신순) + 12개월 초과 정리 WHERE 가속.
CREATE INDEX idx_diagnosis_history_run_date ON diagnosis_history(run_date);
