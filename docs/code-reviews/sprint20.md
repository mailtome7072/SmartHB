# Sprint 20 코드 리뷰

> 대상: Sprint 20 (0ce55d5~52441d5) — 청구 규칙 교습기간 기준 전환 + 청구 삭제 + 인쇄/출결 버그 수정
> 리뷰 일자: 2026-07-19
> 자동 검증 결과: cargo test 441 passed / clippy clean / tsc clean / lint clean / build 성공

## 발견 사항 (2건)

### F1 — delete_bill_impl 트랜잭션 미사용 (Medium) — ✅ 해결(2026-07-19)

- 위치: `src-tauri/src/commands/billing.rs:345`
- Sprint 20 T3 DoD에서 "트랜잭션 내 실행 (BEGIN IMMEDIATE)"을 명시했으나, 최초 구현에서 SELECT 체크(is_paid 확인)와 DELETE 실행이 트랜잭션 없이 수행됨.
- 실패 시나리오: 이론상 체크(is_paid=0)와 삭제 사이에 수납이 완료(`is_paid=1`)되면 수납완료 청구가 삭제될 수 있음. 단, 단일 사용자 데스크톱 앱 구조상 동시 접근이 없어 실질적 발생 가능성 없음.
- **조치(해결)**: `delete_bill_impl`을 `pool.begin()` + `SELECT 1 FROM bills LIMIT 0`(generate_bills 와 동일 BEGIN IMMEDIATE 패턴)으로 감싸 SELECT 가드~DELETE 를 원자화. 감사 로그는 커밋 후 best-effort(전역 pool). 삭제 테스트 4건 통과(CASCADE 포함). R137 종결.

### F2 — get_billing_summary_impl INNER JOIN vs generate_bills_impl LEFT JOIN 의미적 차이 (Low, 기록만)

- 위치: `src-tauri/src/commands/billing.rs:1047` (`get_billing_summary_impl`)
- `generate_bills_impl`은 `LEFT JOIN student_schedules` + 코드 레벨 `weekly_hours==0` skip 방식. `get_billing_summary_impl`은 `INNER JOIN + HAVING SUM > 0` 방식.
- fee 매핑이 없는 `weekly_hours` 값을 가진 원생의 경우: generate에서는 skip(청구 미생성), summary에서는 `total_billable_students`에 포함 → 이 케이스에서 `ungeneratedCount > 0`이 될 수 있음.
- 실제 영향: `standard_fees`에 모든 활성 수업시간이 등록되어 있어 이 케이스 발생 가능성 매우 낮음. T1의 핵심 수정(날짜 경계 유령 버튼)은 test(f) `summary_total_billable_matches_generate_target_teaching_period`로 검증 완료. pre-existing 이슈.
- 조치: 코드 주석으로 기록, 배포 차단 없음.

## 영역별 추가 점검

### 보안 (backend.md Critical/High)
- SQL 인젝션: T1/T3 모든 쿼리 `.bind()` 파라미터 사용 확인 ✅
- 하드코딩 시크릿: `git diff` 스캔 결과 없음 ✅
- T3 감사 로그 PII: `student_id`(ID)만 기록, 원생명 미포함 확인 ✅
- FK CASCADE 전제: `db.rs:117 PRAGMA foreign_keys=ON` 풀 초기화 시 설정 확인 ✅, 단위 테스트 `enable_fk` 헬퍼로 인메모리 테스트에서도 검증 ✅

### 보안 (backend.md High)
- `unwrap()/expect()` 프로덕션 코드 사용: 없음 ✅ (테스트 코드에서만 사용)
- 마이그레이션 없는 스키마 변경: V310 최신 유지, 신규 마이그레이션 없음 ✅
- 새 쿼리 단위 테스트: T1 6건, T3 4건, T7 1건 추가 ✅

### 프론트엔드 보안 (frontend.md Critical/High)
- XSS: `escapeHtml()` 적용 — `band.display_name`, `ev.display_name`, `title`에 모두 이스케이프 적용 ✅
- `invoke()` 직접 호출: `DeleteBillDialog`/`BillingGrid`에서 `src/lib/tauri/index.ts`의 `deleteBill` 래퍼 경유 ✅
- TypeScript any 남용: 없음 ✅

### AI 생성 코드 추가 체크
- 비즈니스 로직이 sprint 계획 문서와 일치하는가: ✅ (T1/T3/T6/T7 모두 계획 대비 정확히 구현)
- 의도치 않은 파일 변경: sprint20 커밋 범위 내 파일 외 변경 없음 ✅
- 하드코딩된 테스트 데이터/더미 값: 없음 ✅
- 새 의존성 추가: 없음 ✅

## 결론

Critical 0건, High 0건. Medium 1건(F1: 트랜잭션 미사용 — 실질 위험 없음, ROADMAP 이연). Low 1건(F2: INNER/LEFT JOIN 차이 — pre-existing). 배포 진행에 지장 없음.
