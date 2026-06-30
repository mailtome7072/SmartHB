# Sprint 17 계획 컨텍스트

## 배경
- v1.0.0 실사용 중 장시간 작업 후 DB 오류로 저장 실패 + 재시작 시 데이터 손실 발견
- 코드 전수 조사로 버그 9건 확인, 긴급 6건은 Hotfix(`hotfix/db-lock-and-backup-fix`)로 선행 처리
- Sprint 17은 남은 3건 안전성 수정 + 3건 정책 간소화

## Task 구성
- T1~T3: 그룹 A (안전성) — WAL 체크포인트 에러 처리, 백업 atomic write, 자동 복원 재검증
- T4~T6: 그룹 B (간소화) — hourly 간격 2h, heartbeat 제거, SyncStatus 삭제
- T7: 통합 검증

## 핵심 수치
- 예상 총 시간: 16h (가용 40h 중)
- DB 마이그레이션: 없음
- 새 의존성: 없음
- 수정 파일 10개 (삭제 1개: sync.rs)

## 전제 조건
- 집<->교습소 동시 사용 없음 (heartbeat 제거, SyncStatus 삭제의 전제)
- 사용자가 동기화 상태 제거에 동의
