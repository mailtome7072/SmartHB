---
Sprint: 11  |  Date: 2026-05-29  |  Session: #2
---

> Sprint 11 Session #2 — T1 (DB 마이그레이션 V109: bills + payments + payment_methods.is_card_type).
> Session #1 (T0 carry-over) 완료 → 본 세션은 청구·수납 도메인 진입의 스키마 토대.

## Session #1 (T0) 완료 사항 — 요약

7건 carry-over 일괄 정리 (F1~F7). cargo test 273 passed / clippy / lint / tsc clean. 커밋 `958285c`.

## 발견된 이슈 (T1 진입 시 step-back)

sprint11.md T1 의 일부 가정이 실제 DB 구조와 불일치 → 사용자 확인 후 정정:
1. `codes` 단일 테이블 없음 → `payment_methods` / `card_companies` 별도 테이블
2. `is_card_type` 컬럼은 **`payment_methods`** 에 추가
3. `payments.payment_method_id` FK → **`REFERENCES payment_methods(id)`**
4. `payments.card_company_id` FK → **`REFERENCES card_companies(id)`**
5. `bills.weekly_hours` → **`INTEGER`** (`standard_fees` 와 정합)
6. Boolean 컬럼은 SQLite 패턴 따라 `INTEGER ... CHECK (X IN (0, 1))`
7. 카드 시드 마킹: 현재 `payment_methods` 는 `code='card'` 한 건만 카드 계열

위 정정은 sprint11.md T1 섹션에도 반영.

## 이번 세션의 Task

| Task | 작업 | 예상 |
|------|------|------|
| **T1** | V109 마이그레이션 작성 (bills + payments + payment_methods.is_card_type) + sqlx prepare 캐시 갱신 | 3h |

## T1 작업 범위

### V109 — `src-tauri/migrations/109__create_billing_tables.sql`

**1. `payment_methods.is_card_type` 컬럼 추가**
- `ALTER TABLE payment_methods ADD COLUMN is_card_type INTEGER NOT NULL DEFAULT 0 CHECK (is_card_type IN (0, 1));`
- `UPDATE payment_methods SET is_card_type = 1 WHERE code = 'card';`

**2. `bills` 테이블 (PRD §4.9.1~§4.9.7)**
- 컬럼: id, student_id (FK students), bill_year_month, weekly_hours INTEGER, bill_amount, adjusted_amount, status('draft'/'confirmed'/'closed'), is_mid_month, mid_month_type, close_reason, closed_at, confirmed_at, created_at, updated_at
- UNIQUE: `(student_id, bill_year_month)` — PRD §6.2
- CHECK: bill_year_month GLOB, status CHECK, is_mid_month/mid_month_type 정합, closed_at/status 정합
- INDEX: bill_year_month, student_id, status

**3. `payments` 테이블 (PRD §4.9.5~§4.9.6, PI-12 별도 테이블)**
- 컬럼: id, bill_id (FK bills UNIQUE 1:1), is_paid, paid_date, payer_name, payment_method_id (FK payment_methods), card_company_id (FK card_companies), created_at, updated_at
- UNIQUE: bill_id (청구 1건당 수납 1건)
- ON DELETE CASCADE: 청구 삭제 시 수납도 정리
- CHECK: paid_date GLOB, is_paid=1 → paid_date NOT NULL 정합

### sqlx 준비
- `.env` 의 `DATABASE_URL` (`sqlite:./SmartHB-dev.db`) 로 `sqlx migrate run`
- `sqlx prepare --manifest-path src-tauri/Cargo.toml` 캐시 갱신 (T2 이후 query! 매크로 추가 예정 — T1 본체에는 query! 변경 없으나 일관성 유지)

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/109__create_billing_tables.sql | [1회] | 신규 마이그레이션 |
| docs/sprint/sprint11.md | [1회] | T1 정정 (codes → payment_methods, weekly_hours INTEGER 등) |
| docs/sprint/sprint11/scope.md | [수정 중] | 본 파일 (Session #2) |
| src-tauri/.sqlx/ | [-] | `sqlx prepare` 가 갱신 (자동) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [x] .github/workflows/
- [x] SETUP.sh
- [x] src-tauri/tauri.conf.json
- [x] src-tauri/src/commands/ — T2 이후 IPC 단계
- [x] docs/harness-engineering/

## 완료 기준 (T1 AC)

- [ ] V109 마이그레이션 파일 생성 + 컬럼/UNIQUE/CHECK/FK/INDEX 정의 완성
- [ ] `sqlx migrate run` 성공 (dev DB 적용)
- [ ] `.sqlx/` 캐시 갱신 후 변경분 확인
- [ ] DB 직접 검증: bills/payments 테이블 존재 + payment_methods.is_card_type 컬럼 추가 + code='card' 행이 is_card_type=1
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --lib` 통과 (T1 본체는 코드 변경 없으나 회귀 확인)
- [ ] sprint11.md T1 섹션이 본 정정안과 정합 (codes → payment_methods 등)

## 세션 종료 조건

- [ ] T1 AC 모두 통과
- [ ] 단일 커밋 (V109 + sprint11.md 정정 + scope.md)
- [ ] 다음 세션 (T2 청구 생성 IPC) 진입점 메모

## 다음 세션 (T2) 미리보기

- `src-tauri/src/commands/billing.rs` 신규 (또는 `fees.rs` 확장)
- `generate_bills`, `list_bills`, `update_bill`, `get_bill` IPC
- standard_fees 매핑 + 월 중 입퇴교 플래그 자동 설정
- 단위 테스트 10건 이상
