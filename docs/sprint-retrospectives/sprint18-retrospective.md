# Sprint Retrospective Sprint 18

> 대상: Sprint 18 (d86c25b) — 사용자 피드백 10건 + 캘린더 UX 개선 + 출결 동기화
> 리뷰 일자: 2026-07-01
> 코드 리뷰: Critical 0 / High 0 / Medium 3건 / Low 2건 (수정 후 재검증 완료)
> 자동 검증: cargo test 414 passed / clippy --all-targets clean / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint17-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A107 | STALE_THRESHOLD_SECONDS 86400 상향 | ✅ 완료 | lock.rs 86400으로 변경 (T0) |
| A108 | rollback 파일명 고유성 보장 (loop 인덱스 추가) | ✅ 완료 | generate_rollback_filename에 idx 파라미터 추가 (T0) |
| A109 | auto_restore_with_retry 단위 테스트 | ✅ 완료 | 3회 retry 성공/실패 시나리오 커버 (T0) |
| A110 | cleanup_stale_tmp_backups spawn_blocking 래핑 | ✅ 완료 | tokio::task::spawn_blocking 적용 (T0) |
| A111 | WAL 체크포인트 실패 시 pool.close() 보장 | ✅ 완료 | early return 전 pool.close() 호출 추가 (T0) |
| A112 | cipher 스모크 테스트 수행 | ⏸️ 이연 | Sprint 18 범위 외. 수동 검증 항목으로 유지 |

---

## 잘한 점

**Sprint 17 회고 액션 아이템 5건(A107~A111) T0 선행 처리 — 기술 부채 청산 패턴의 정착**

이전 회고에서 발견된 High/Medium 5건을 스프린트 첫 번째 태스크(T0)로 묶어 선행 처리했다. lock.rs STALE_THRESHOLD_SECONDS 상향, rollback 파일명 충돌 방지, auto_restore_with_retry 단위 테스트, spawn_blocking 래핑, pool.close() 보장이 모두 당일 완료됐다. 기술 부채를 이월하지 않고 다음 스프린트 시작 시 즉시 처리하는 패턴이 두 스프린트 연속으로 작동했다.

**T8 출결 자동 동기화 — 핵심 비즈니스 로직을 단위 테스트로 커버**

학사 일정 변경 시 출결 자동 동기화(`sync_attendance_on_schedule_change`)는 복잡한 비즈니스 규칙(OFF→ON INSERT, ON→OFF DELETE, 교습기간 확인)을 포함한다. 함수 구현과 함께 OFF→ON / ON→OFF / 변화없음 3가지 시나리오를 단위 테스트로 커버했다. 이전 스프린트에서 "함수 구현과 테스트 작성이 같은 커밋에서 완료되지 않아 회귀 감지 불가능"을 개선할 점으로 기록했는데, 이번에 그 원칙을 실천했다.

**캘린더 UX 개선(T4~T7) 당일 완료 — Velocity 예측 신뢰도 향상**

수업관리 캘린더의 기본 뷰 변경(T4), 요일 시작 변경(T5), 주보기 4색 체계 + 다중슬롯 칩(T6), 월보기 원생 이름 표기(T7)는 FullCalendar 이벤트 모델 변환이 포함된 복잡한 작업이었다. 계획 예상 시간(6.5h)과 실제 소요가 근접했다. Sprint 17 Velocity(16h/당일)를 기준으로 산정한 Sprint 18 Capacity(17h) 예측이 유효함을 확인했다.

**sprint-review 코드 리뷰 → 즉시 수정 → 재검증 루프 정착**

sprint-review에서 발견된 5건(LockWarning 임계값 불일치, T8 fail-soft 위반, DELETE 조건 누락, 인쇄 race condition, 색상 중복)을 모두 당일 수정 후 재검증했다. Critical/High 없이 Medium 3 + Low 2 수준에서 마무리된 것은 구현 품질이 전반적으로 양호했음을 보여준다.

---

## 개선할 점

**LockWarning.tsx 프론트엔드 상수가 백엔드 Rust 상수와 동기화되지 않음**

lock.rs의 `STALE_THRESHOLD_SECONDS`를 300→86400으로 변경(T0)했지만, 프론트엔드 `LockWarning.tsx`의 동명 상수는 업데이트하지 않았다. sprint-review 코드 리뷰에서 발견됐지만, T0 구현 당시 "백엔드 수정 → 프론트엔드 동기화 필요" 체크리스트가 없었던 것이 원인이다. 백엔드-프론트엔드 상수 쌍이 존재하는 파일 목록을 scope.md에 명시하면 재발을 예방할 수 있다.

**T8 fail-soft 정책이 `?` 연산자로 구현됨 — 스프린트 계획과 구현 불일치**

sprint18.md T8 설계에 "동기화 실패 시 eprintln! 로그만 남기고 IPC 흐름 차단하지 않음"이 명시됐지만, 실제 구현에서는 `.await?`를 사용해 에러를 전파했다. 계획 문서의 fail-soft 정책이 구현 단계에서 누락된 사례다. 핵심 설계 결정(fail-soft vs fail-hard)은 함수 docstring 또는 inline 주석에 명시해 구현자(AI 포함)가 의도를 확인할 수 있게 해야 한다.

**OFF→ON 복원 시 스케줄 이력 반영 불완전 — generate_impl과 sync_single_date 패턴 불일치**

`sync_single_date`의 INSERT 쿼리는 현행 스케줄(`effective_to IS NULL OR effective_to > date`)만 참조하는 반면, `generate_impl`은 전체 이력을 순회한다. 스케줄 변경 이력이 있는 원생의 과거 날짜에서 OFF→ON 복원 시 이력 기반 스케줄이 반영되지 않는다(R124). 설계 시 두 함수의 접근 방식 차이를 인지하고 의도적으로 결정했어야 할 사항이다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A113 | 백엔드-프론트엔드 상수 쌍 목록화 — `STALE_THRESHOLD_SECONDS`(lock.rs ↔ LockWarning.tsx) 같은 쌍을 scope.md 또는 CLAUDE.md에 명시해 한쪽 변경 시 다른 쪽 체크 의무화. 현재 파악된 쌍: stale 임계값(1쌍) | Medium | `CLAUDE.md` 또는 `docs/harness-engineering/` | Sprint 19 T0 |
| A114 | T8 sync_single_date INSERT를 generate_impl 이력 패턴으로 통일 — `effective_to` 이력을 올바르게 반영하도록 수정. 실사용 단순 스케줄 패턴에서는 즉각 위험 없으나, 스케줄 변경 이력이 쌓이면 OFF→ON 복원 시 출결 누락 가능 | Low | `src-tauri/src/commands/attendance.rs::sync_single_date` | Post-MVP 안정화 스프린트 |
| A115 | cipher 스모크 테스트 수행 — v1.0.1(.exe) 설치 후 integrity_check, 백업/복원, DB 폴더 변경 실동작 확인. A112 이월 3회차 | High | 배포 후 수동 검증 | Sprint 18 develop QA 후 (즉시) |
| A116 | AcademicSchedulePrint 동적 행 수 계산 — `Math.ceil(cells.length / 7)`로 5행 달에서 빈 마지막 행 제거. 인쇄 레이아웃 여백 낭비 제거 | Low | `src/components/academic/AcademicSchedulePrint.tsx:105` | Sprint 19 또는 인쇄 기능 개선 시 |
