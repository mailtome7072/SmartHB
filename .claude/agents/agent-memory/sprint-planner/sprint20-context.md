---
name: sprint20-context
description: Sprint 20 계획 수립 시 발견한 주요 컨텍스트 -- 청구 버그 수정(study_periods 기준), 삭제 기능, 실 DB 보정
metadata:
  type: project
---

## Sprint 20 계획 수립 컨텍스트 (2026-07-19)

### 핵심 버그

`billing.rs::generate_bills_impl`(78~189행)의 청구 대상 선정이 **달력월**(year_month_range: 01~말일)을 기준으로 하여, 교습기간(study_periods) 종료일 이후 입교한 원생까지 포함하는 오류.

**Why:** study_periods는 `get_default_billing_year_month`(588행, UI 기본 월 표시)에만 사용되고, 실제 청구 생성 로직에는 관여하지 않았음. `year_month_range()`가 YYYY-MM → (YYYY-MM-01, 월말일) 변환만 수행.

**How to apply:** T1에서 `generate_bills_impl`이 study_periods를 조회하여 period_start/period_end로 사용하도록 전환. 조회 실패 시 차단.

### 삭제 기능 부재

현재 `delete_bill`/`DELETE FROM bills` 구현이 전혀 없음. 잘못 생성된 청구를 정정할 수 없었음.
payments는 `ON DELETE CASCADE`(V109)로 bills와 연결. `PRAGMA foreign_keys=ON`은 `db.rs:117`에서 설정.

### Sprint 19 회고 액션

A117~A120: 모두 해결(e72c50f). A114(Post-MVP 이연), A115(deploy QA 이연) — 반영할 미해결 항목 없음.

### Velocity 참고

- Sprint 17: 7 Task / 16h
- Sprint 18: 11 Task / 17h (T1+T2 이미 완료)
- Sprint 19: 11 Task / 33h (T5 디버깅 버퍼 7h)
- Sprint 20: 6 Task / 19h (여유 있음)

### 수정 대상 파일 (예상)

**백엔드:**
- `src-tauri/src/commands/billing.rs` — T1(generate_bills_impl, compute_mid_month_flag), T3(delete_bill)
- `src-tauri/src/commands/audit.rs` — T3(BillDeleted variant)
- `src-tauri/src/lib.rs` — T3(invoke_handler 등록)

**프론트엔드:**
- `src/lib/tauri/index.ts` — T4(deleteBill 래퍼)
- `src/components/billing/BillingGrid.tsx` 또는 관련 — T4(삭제 UI)

**문서:**
- `docs/arch/adr-{NNN}-bill-deletion-guard.md` — T2(ADR)
- `docs/sprint/sprint20/data-correction-procedure.md` — T5(보정 절차)

### 기타 확인 사항

- `year_month_range()` 함수가 billing.rs 외에서도 사용되는지 확인 필요 (dead code 후보)
- 청구 상태 모델: 2단계 (draft/confirmed) — V111에서 마감(closed) 폐기
- `update_bill_impl` 303행: is_paid=1이면 수정 거부 (삭제 가드 설계 시 참조)
