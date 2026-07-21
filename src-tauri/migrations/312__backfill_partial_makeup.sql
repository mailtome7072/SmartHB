-- V312: 기존 일(日) 단위 매칭 데이터 → 분(分) 단위 배분(makeup_allocations) 이전 + 부분보강 잔여 복원
-- (Sprint 22 T5, ADR-011)
--
-- 배경:
-- - 기존 옵션 A(일 단위 매칭)에서는 결석이 보강에 매칭되면 시간과 무관하게 통째로
--   status='makeup_done' 이 되어, 보강 시간이 결석 시간보다 적어도(부분 보강) 잔여가 유실됐다.
--   (실사용 버그: 2시간 결석 → 1시간 보강 시 잔여 1시간이 목록에서 사라짐)
-- - 원본 데이터(결석 class_minutes, 보강 class_minutes, makeup_attendance_id 매칭)는 모두
--   보존돼 있어 손실 없이 재계산 가능.
--
-- 알고리즘 (등록 로직 create_makeup_with_absences_impl 과 동일한 배분 규칙):
-- 1. 각 보강(makeup)에 매칭된 결석들을 소멸기한 임박(오래된) 순으로 정렬.
-- 2. 보강 시간을 그 순서대로 결석에 배분: 각 결석에 min(결석분, 남은 보강분).
--    - 윈도우 함수로 "이전 결석분 누적(prev_sum)" 을 구해
--      alloc = min(결석분, max(0, 보강분 - prev_sum)) 로 계산.
-- 3. 배분(alloc>0)을 makeup_allocations 로 이전.
-- 4. 배분 후에도 잔여(class_minutes - 배분합)가 남는 makeup_done 결석을 absent 로 복원
--    → 보강 대상 목록에 잔여분과 함께 다시 노출된다.
-- - 초과 보강분(보강 > 매칭 결석 합)은 배분할 결석이 없으므로 버려진다(관대 처리).
-- - makeup_expired(소멸) 결석은 makeup_attendance_id IS NULL 이거나 status 가 달라 대상 아님
--   → 원장 판단 존중(보정 제외).
--
-- 멱등성:
-- - INSERT 는 NOT EXISTS 로 이미 이전된 배분을 건너뛴다((makeup_id,absence_id) UNIQUE).
-- - UPDATE 는 잔여>0 인 makeup_done 만 대상 → 이미 absent 로 복원된 행은 재대상 아님.
-- - 재실행해도 결과 불변.
--
-- 완전 자동·무알림 (사용자 확정): 앱 첫 실행 시 조용히 자동 보정. 원장 UI 안내 없음.
-- makeup_attendance_id 레거시 컬럼은 남긴다(ADR-011, 신규 로직 미참조).

-- 1. 기존 makeup_done 매칭을 배분으로 이전 (순차 배분).
INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes)
WITH matched AS (
    SELECT
        ra.id AS absence_id,
        ra.makeup_attendance_id AS makeup_id,
        ra.class_minutes AS absence_minutes,
        m.class_minutes AS makeup_minutes,
        COALESCE(SUM(ra.class_minutes) OVER (
            PARTITION BY ra.makeup_attendance_id
            ORDER BY (ra.makeup_deadline IS NULL), ra.makeup_deadline, ra.event_date, ra.id
            ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING
        ), 0) AS prev_sum
    FROM regular_attendances ra
    JOIN makeup_attendances m ON m.id = ra.makeup_attendance_id
    WHERE ra.makeup_attendance_id IS NOT NULL
      AND ra.status = 'makeup_done'
),
alloc AS (
    SELECT
        makeup_id,
        absence_id,
        MIN(absence_minutes, MAX(0, makeup_minutes - prev_sum)) AS allocated
    FROM matched
)
SELECT makeup_id, absence_id, allocated
FROM alloc
WHERE allocated > 0
  AND NOT EXISTS (
      SELECT 1 FROM makeup_allocations ma
      WHERE ma.makeup_id = alloc.makeup_id AND ma.absence_id = alloc.absence_id
  );

-- 2. 배분 후 잔여가 남은 makeup_done 결석을 absent 로 복원 (부분보강 잔여 노출).
UPDATE regular_attendances
SET status = 'absent',
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE status = 'makeup_done'
  AND makeup_attendance_id IS NOT NULL
  AND (class_minutes - COALESCE((
        SELECT SUM(allocated_minutes) FROM makeup_allocations
        WHERE absence_id = regular_attendances.id
      ), 0)) > 0;
