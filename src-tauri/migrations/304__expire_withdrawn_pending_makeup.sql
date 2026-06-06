-- V304: 퇴교 원생의 미보강 결석 일괄 소멸 (Sprint 14 버그픽스, PRD §4.5.9)
--
-- 배경: 퇴교 처리 정식 흐름(expiration.rs::process_withdrawal_makeup)은 미보강 결석을
-- makeup_expired 로 전이하지만, 원생 수정 폼(update_student)·withdraw_student 로 withdraw_date
-- 만 직접 설정하는 경로는 이 전이를 건너뛴다. 그 결과 퇴교생인데 status='absent' 인 결석이
-- 남아 대시보드 "보강 소멸 임박" 알림 등에 잘못 노출된다.
--
-- 불변식: 퇴교 원생은 미보강(absent·미매칭) 결석을 가지면 안 된다 (보강 대상 아님).
-- 본 마이그레이션은 이 불변식을 위반하는 기존 데이터를 일괄 소멸 전이한다.
--
-- 멱등: 재실행해도 영향 행이 없으면 변화 없음. 전 환경 안전.
-- 주의: 본 전이는 audit_logs 를 남기지 않는다 (일회성 데이터 정정). 이후 신규 발생분은
-- 집계 쿼리의 퇴교 제외(dashboard/attendance) + 정식 퇴교 흐름이 담당한다.

UPDATE regular_attendances
SET status = 'makeup_expired',
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE status = 'absent'
  AND makeup_attendance_id IS NULL
  AND student_id IN (SELECT id FROM students WHERE withdraw_date IS NOT NULL);
