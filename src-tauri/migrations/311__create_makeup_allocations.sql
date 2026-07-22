-- V311: 결석-보강 배분 링크 테이블 makeup_allocations (Sprint 22 T1, ADR-011)
--
-- 변경 사유:
-- - 보강 모델을 일(日) 단위 매칭 -> 분(分) 단위 부분 차감으로 전환 (ADR-011).
-- - 기존: regular_attendances.makeup_attendance_id (단일 FK) 로 결석->보강 1:N 표현.
--   부분 보강 도입 시 1결석:N보강 도 발생하므로 N:M + 배분량(분) 표현이 필요하다.
-- - makeup_allocations 는 "어느 보강(makeup_id)이 어느 결석(absence_id)을
--   몇 분(allocated_minutes) 충당했는가"를 명시적으로 저장한다.
--   -> 잔여 = regular_attendances.class_minutes - SUM(allocated_minutes)
--   -> 취소 시 해당 보강의 배분 레코드만 제거하여 정확한 부분 환원 가능 (T3).
--
-- 배포 안전성 (ADR-011, R142):
-- - regular_attendances.makeup_attendance_id 컬럼은 DROP 하지 않고 레거시로 남긴다.
--   -> 부모 테이블 재구성이 불필요 -> deferred FK 카운터 함정(V108 code 787) 원천 회피.
--   신규 로직은 makeup_allocations 만 사용하고 레거시 컬럼은 참조하지 않는다.
-- - 본 마이그레이션은 순수 CREATE TABLE + CREATE INDEX (신규) 만 수행 -> 기존 데이터 무손상.
-- - 기존 매칭(makeup_attendance_id)의 allocation 이전 + 부분 보강 잔여 복원은 V312 백필에서 수행.
--
-- 무결성:
-- - allocated_minutes > 0 (0분 배분 금지)
-- - UNIQUE (makeup_id, absence_id): 한 보강이 같은 결석에 중복 배분 금지 (한 등록당 결석별 1행)
-- - FK: makeup_id -> makeup_attendances(id), absence_id -> regular_attendances(id)
--   (sqlx 는 마이그레이션을 트랜잭션으로 감싸고 앱 연결은 PRAGMA foreign_keys = ON.
--    본 파일은 신규 테이블 INSERT 가 없어 FK 위반 여지 없음.)

CREATE TABLE makeup_allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    makeup_id INTEGER NOT NULL REFERENCES makeup_attendances(id),
    absence_id INTEGER NOT NULL REFERENCES regular_attendances(id),
    allocated_minutes INTEGER NOT NULL CHECK (allocated_minutes > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (makeup_id, absence_id)
);

-- 잔여 계산(absence_id 기준 집계) + 취소(makeup_id 기준 조회) 인덱스.
CREATE INDEX idx_makeup_alloc_makeup ON makeup_allocations(makeup_id);
CREATE INDEX idx_makeup_alloc_absence ON makeup_allocations(absence_id);
