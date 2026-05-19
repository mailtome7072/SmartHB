-- V008: app_settings (key/value) + audit_logs (T9, PRD §4.0·§6.6)
--
-- V002~V007 갭은 후속 sprint 의 도메인 테이블(원생·수업 스케줄·출결·보강·단원평가·청구·학사 일정·교습기간)
-- 예약. 본 마이그레이션은 인프라 테이블만 다룬다.
--
-- 변경 사유:
-- - app_settings: 초기 설정 마법사 결과(클라우드 폴더 경로 등), 단축키 커스텀, UI 환경설정
--   * key/value 형식으로 schema-less 유연성 확보 — 신규 설정 추가 시 마이그레이션 불필요
-- - audit_logs: 비밀번호 변경/복구 코드 발급/백업 복원/락 강제 점유 등 보안 이벤트 1년 보관
--   * 민감 데이터(비밀번호, 코드 해시, 키)는 미기록 — 호출자가 사전 마스킹
--   * (created_at DESC) INDEX 로 시간 역순 페이지네이션 O(log N)

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    event_type TEXT NOT NULL,
    event_subject TEXT,
    details TEXT
);

CREATE INDEX idx_audit_logs_created_at_desc ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
