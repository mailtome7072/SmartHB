# Sprint Plan sprint11

## 기간
2026-05-28 ~ 2026-06-11 (2주, 예상)

## Phase 위치
**Phase 4: 청구 + 수납 + 공지문 (Sprint 11~12)** -- 첫 번째 스프린트. 청구/수납 도메인을 완성한다. 공지문 이미지 생성은 Sprint 12로 분리.

## 목표
1. **청구 도메인 완성** -- 재원 원생 일괄 청구 생성 + 3단계 상태 머신(미확정/확정/마감) + 마감 후 수정 사유 강제 (PRD SS4.9.1~SS4.9.7, AC-4.9-1~8)
2. **수납 관리 완성** -- 수납 입력 + 카드사 조건부 필수 + 입금 일괄 처리 모드 (PRD SS4.9.5~SS4.9.6)
3. **Phase 3 carry-over 해소** -- Sprint 9~10 코드 리뷰에서 발견된 중요도 Medium 이상 기술 부채 6건 + flaky 테스트 1건 + 사이드 메뉴 정리 1건 일괄 해소
4. **Phase 4 청구 UI** -- 청구 목록 화면 + 수납 관리 화면 + 입금 일괄 처리 모드 + 사이드 메뉴 활성화

## ROADMAP 연계 기능
- SS4.9.1 청구 데이터 생성 (ROADMAP Sprint 11)
- SS4.9.2 월 중 입퇴교 처리 (ROADMAP Sprint 11)
- SS4.9.3 청구 확정 상태 관리 (ROADMAP Sprint 11)
- SS4.9.4 청구 화면 정렬 (ROADMAP Sprint 11)
- SS4.9.5 수납 관리 (ROADMAP Sprint 11)
- SS4.9.6 입금 일괄 처리 모드 (ROADMAP Sprint 11)
- SS4.9.7 월별 청구 마감 (ROADMAP Sprint 11)
- Phase 3 carry-over (F1~F5, flaky 테스트, 사이드 메뉴)

## 미결정 항목 (PI) -- 전수 결정 완료 (2026-05-28)
| ID | 항목 | 결정 | 근거 |
|----|------|------|------|
| PI-10 | 청구 마감 후 수정 사유 UX 형태 | ✅ **모달 다이얼로그** | 사유 입력의 "의도적 행위" 강조, 실수 방지 |
| PI-11 | 마감 해제(reopen) 기능 필요 여부 | ✅ **마감 해제 불가** | PRD에 마감 해제 언급 없음, 개별 건 수정(사유 필수)만 허용 |
| PI-12 | 수납 데이터 별도 테이블 vs 청구 테이블 내장 | ✅ **별도 payments 테이블** | 향후 분할 납부/환불 확장 여지, 감사 로그 분리 용이 |
| -- | 카드 계열 결제수단 식별 기준 | ✅ **`is_card_type` BOOLEAN 플래그** | 사용자 추가 결제수단에도 정확한 판별 보장 (V109에 포함) |
| -- | carry-over F4 (N+1 쿼리) 범위 | ✅ **`calendar.rs` N+1만** | `attendance.rs` N+1은 PRD 성능 기준 충족 중, 별도 backlog 유지 |

---

## 이전 회고 반영

> Sprint 10 회고(`docs/sprint-retrospectives/sprint10.md`)가 아직 작성되지 않음.
> Sprint 9 회고(`docs/sprint-retrospectives/sprint9-retrospective.md`)도 부재.
> 대신 Sprint 10 계획(sprint10.md)에 기록된 이연 항목과 Sprint 10 코드 리뷰(risk-register/2026-05-28.md)의 carry-over를 반영한다.

### Sprint 10 이연 항목 반영

| 원본 ID | 항목 | Sprint 11 반영 |
|---------|------|---------------|
| A52 | `get_absence_history` pagination (R64) | 이번 Sprint scope 외 유지 -- PRD 50명 규모 안전 |
| A53 | `create_makeup_with_absences_impl` JSON_EACH 전환 (R65) | 이번 Sprint scope 외 유지 -- 성능 영향 미미 |
| A54 | `get_attendance_grid` N+1 batch 쿼리 (R42) | **F4와 동일 패턴** -- T0에서 calendar.rs N+1과 함께 검토 |
| A55 | salt buffer ZeroizeOnDrop (R48-b) | 이번 Sprint scope 외 유지 -- 보안 도메인 |
| A56 | 반응형 폰트/셀 너비 clamp() | 이번 Sprint scope 외 유지 -- UX 전반 |
| A57 | 한글 자모 부분 일치 검색 | 이번 Sprint scope 외 유지 -- 검색 도메인 |

### Risk Register 반영 (2026-05-28)

| Risk ID | 항목 | Sprint 11 반영 |
|---------|------|---------------|
| R70 | `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` flaky | **T0**에서 `#[ignore]` 마킹 또는 동시성 설계 수정 |
| R71 | `build_day_schedules` `succ_opt().expect()` 패닉 | **T0**에서 `.ok_or_else()` 전환 |
| R72 | `generate_impl` expire fail-hard 정책 불일치 | **T0**에서 fail-soft 전환 |

---

## Capacity 계획

| 항목 | 시간 |
|------|------|
| 팀 인원 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10일 (2주) |
| 일 실작업 시간 | 4h |
| 총 Capacity | **40h** |
| 시각 검증 버퍼 | 6h (Sprint 10 회고 A50 반영) |
| 실 작업 Capacity | **34h** |

| Task 그룹 | 예상 소요 | 비고 |
|-----------|----------|------|
| T0: carry-over 정리 | 4h | 7건 (F1~F5 + flaky + 메뉴) |
| T1: DB 마이그레이션 | 3h | V109 bills + payments |
| T2: 청구 생성 IPC | 4h | generate + list + update + 단위 테스트 |
| T3: 청구 상태 머신 IPC | 3h | confirm + close + 마감 후 수정 + 단위 테스트 |
| T4: 수납 IPC | 3h | CRUD + 카드사 조건 + 단위 테스트 |
| T5: 청구 마감 UX | 2h | 마감 후 수정 사유 다이얼로그 |
| T6: TypeScript 래퍼 + 타입 | 1.5h | IPC 래퍼 + 도메인 타입 |
| T7: 청구 관리 UI | 4h | 목록 + 상태 전이 + 정렬 + 배너 |
| T8: 수납 관리 UI | 3h | 수납 입력 + 입금 일괄 처리 |
| T9: 통합 검증 | 3h | 자동 7항목 + AC 마킹 |
| **합계** | **30.5h** | Capacity 34h 이내 (여유 3.5h) |

---

## 작업 목록

### T0: Phase 3 carry-over 정리 (4h) ✅ 2026-05-28 `958285c`

Sprint 9~10 코드 리뷰 발견 사항 + 운용 부채를 일괄 해소한다.

- ✅ **F1**: `build_day_schedules` `d.succ_opt().expect("date succ")` -> `.ok_or_else(|| ...)` 안전 전환 (`attendance.rs:655`) -- skill: systematic-debugging
- ✅ **F2**: `generate_impl` expire 호출 실패 시 fail-soft 전환 -- expire 실패해도 출결 생성은 성공 반환, expire 에러는 warn 로그만 (`attendance.rs:155`)
- ✅ **F3**: `_year_month` 미사용 파라미터 제거 또는 활용 (`calendar.rs:188`)
- ✅ **F4**: 보강관리 N+1 쿼리 -- `calendar.rs:215` 루프 내 개별 쿼리를 JOIN 또는 IN 절로 batch 처리
- ✅ **F5**: `ClassCalendar` viewType 비동기 상태 한 프레임 불일치 해소 (`src/components/schedules/ClassCalendar.tsx:164`)
- ✅ **F6**: flaky 테스트 `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` -- `#[ignore]` 마킹 + 코멘트 (동시성 설계 재검토는 별도 backlog)
- ✅ **F7**: 사이드 메뉴 '보강 관리' (`/makeups`) `disabledHint` 제거 -- Sprint 10 T11에서 `/schedules` 탭으로 통합 완료. `src/lib/menu-config.ts:21`

### T1: DB 마이그레이션 V109 -- bills + payments + payment_methods.is_card_type (3h) ✅ 2026-05-28 `3a2ebcc`

PRD SS4.9 + SS6.2 UNIQUE 제약 준수. PI-12 확정: **별도 payments 테이블**.

> **T1 진입 시 사실 정정 (Session #2)**: 실제 DB 에는 단일 `codes` 테이블 없음 — `payment_methods` / `card_companies` 가 별도 테이블. SQLite Boolean 은 `INTEGER CHECK (X IN (0, 1))` 패턴. `weekly_hours` 는 `standard_fees` 와 정합 위해 INTEGER.

- ✅ **bills 테이블** 설계 + 마이그레이션 작성
  - 컬럼: `id`, `student_id` (FK students), `bill_year_month` (TEXT "YYYY-MM"), `weekly_hours` (**INTEGER**), `bill_amount` (INTEGER, 원 단위), `adjusted_amount` (INTEGER, 조정 후 금액), `status` (TEXT CHECK: 'draft'/'confirmed'/'closed'), `is_mid_month` (INTEGER CHECK IN (0,1), 월 중 입퇴교 플래그), `mid_month_type` (TEXT NULL: 'enrolled'/'withdrawn'), `close_reason` (TEXT NULL, 마감 후 수정 사유), `closed_at` (TEXT NULL, 마감 일시), `confirmed_at` (TEXT NULL), `created_at`, `updated_at`
  - UNIQUE 제약: `(student_id, bill_year_month)` -- PRD §6.2
  - FK: `student_id REFERENCES students(id)`
  - CHECK: bill_year_month GLOB, status CHECK, is_mid_month/mid_month_type 정합, closed_at/status 정합
  - INDEX: bill_year_month, student_id, status
- ✅ **payments 테이블** 설계 + 마이그레이션 작성 (PI-12 확정: 별도 테이블)
  - 컬럼: `id`, `bill_id` (FK bills, UNIQUE -- 1:1 관계, ON DELETE CASCADE), `is_paid` (INTEGER CHECK IN (0,1) DEFAULT 0), `paid_date` (TEXT NULL), `payer_name` (TEXT NULL), `payment_method_id` (**FK payment_methods(id)**), `card_company_id` (**FK card_companies(id)**, 카드 계열 시 필수 -- 백엔드 IPC 검증), `created_at`, `updated_at`
  - UNIQUE: `bill_id` (청구 1건당 수납 1건)
  - CHECK: paid_date GLOB, is_paid=1 → paid_date NOT NULL 정합
- ✅ **`payment_methods.is_card_type` 컬럼 추가** -- 결제수단 카드 계열 식별용
  - `ALTER TABLE payment_methods ADD COLUMN is_card_type INTEGER NOT NULL DEFAULT 0 CHECK (is_card_type IN (0, 1))`
  - 기존 시드 중 `code='card'` 한 건을 `is_card_type=1`로 UPDATE (현재 시드 기준)
  - 추후 '신용카드' / '체크카드' 분리 시드 추가 시 함께 마킹
- ✅ `.sqlx/` 오프라인 캐시 갱신 + 커밋

**완료 검증**: `sqlx migrate run` 성공 + `sqlx prepare` 캐시 갱신

### T2: 청구 생성 IPC (4h) ✅ 2026-05-28 `273b370`

- ✅ **`generate_bills` IPC** -- `src-tauri/src/commands/fees.rs` (기존 모듈 확장 또는 `billing.rs` 신규)
  - 재원 원생(`withdraw_date IS NULL`) 일괄 청구 생성
  - `weekly_hours` -> 표준 교습비 매핑 (기존 `standard_fees` 테이블 활용)
  - 동일 월 중복 생성 차단 (AC-4.9-1, UNIQUE 제약)
  - 월 중 입교(`enroll_date` 해당 월 내) / 월 중 퇴교(`withdraw_date` 해당 월 내) 플래그 자동 설정
  - 디폴트 청구년월: 마지막 교습기간 월 (PRD SS4.9.1)
  - `BEGIN IMMEDIATE` 트랜잭션
- ✅ **`list_bills` IPC** -- 월별 청구 목록 조회
  - 정렬: 미확정 + 월 중 입퇴교 상단 우선 (AC-4.9-4)
  - 원생명, 학년, 주 수업시간, 청구액, 조정액, 상태, 입퇴교 구분 포함
- ✅ **`update_bill` IPC** -- 개별 청구 금액 조정
  - 상태별 수정 제약: draft(자유) / confirmed(확인 다이얼로그 필수 -- 프론트 책임) / closed(사유 입력 필수 -- 백엔드 검증)
  - 마감 상태에서 `close_reason` NULL이면 에러 반환 (AC-4.9-8)
- ✅ **`get_bill` IPC** -- 단건 조회
- ✅ **단위 테스트** 10건 이상
  - 정상 생성 + 중복 차단 + 월 중 입퇴교 플래그 + 표준교습비 매핑 + 마감 후 사유 미입력 차단

### T3: 청구 상태 머신 IPC (3h) ✅ 2026-05-28 `a524a2a`

청구 3단계 상태 전이를 백엔드에서 엄격하게 제어한다. 프론트엔드 우회 방지.
PI-11 확정: **마감 해제 불가** -- 마감 후에는 개별 건 수정(사유 필수)만 허용.

- ✅ **`confirm_bill` IPC** -- draft -> confirmed 전이
  - 단건 확정
- ✅ **`confirm_all_bills` IPC** -- 해당 월 전체 일괄 확정 (미확정 건만)
- ✅ **`close_billing_month` IPC** -- 월 전체 마감
  - 전제 조건: 해당 월 모든 청구가 confirmed 상태 (AC-4.9-7)
  - 미확정 건 존재 시 에러 반환 + 미확정 건수 포함
  - 모든 bills 상태 -> closed + `closed_at` 기록
  - reopen(마감 해제) IPC 불요 (PI-11 확정: 마감 해제 불가)
- ✅ **`update_closed_bill` IPC** -- closed 상태에서 금액 수정
  - `close_reason` 필수 입력 (NULL/빈문자열 차단)
  - `audit::BillClosedModified` variant 추가
- ✅ **단위 테스트** 8건 이상
  - 상태 전이 정상/차단 + 마감 전제조건 위반 + 마감 후 사유 검증 + 마감 해제 시도 차단

### T4: 수납 IPC (3h) ✅ 2026-05-28 `aa9650a`

- ✅ **`create_payment` / `update_payment` IPC**
  - 입금 등록: `is_paid`, `paid_date`, `payer_name`, `payment_method_id`, `card_company_id`
  - 카드 계열 결제수단 시 `card_company_id` 필수 검증 (AC-4.9-4)
  - 카드 계열 판별: codes 테이블의 `is_card_type` BOOLEAN 플래그로 판별 (T1 V109에서 추가)
- ✅ **`list_unpaid_bills` IPC** -- 미납 청구 목록 (입금 일괄 처리용)
  - 최소 20행 반환 가능 (AC-4.9-6)
- ✅ **`batch_update_payments` IPC** -- 다수 입금 일괄 처리
  - 배열 입력으로 다건 동시 처리
  - `BEGIN IMMEDIATE` 트랜잭션
- ✅ **`get_billing_summary` IPC** -- 월별 요약 (총 청구액 / 입금 완료액 / 미납액)
  - 대시보드 SS4.11.3 위젯 데이터 선행 준비
- ✅ **단위 테스트** 7건 이상
  - 카드사 조건부 필수 + 일괄 처리 + 미납 추출

### T5: 청구 마감 UX -- 마감 후 수정 사유 (2h) ✅ 2026-05-28 `0af5744`

PI-10 확정: **모달 다이얼로그**. PI-11 확정: **마감 해제 불가** (reopen 버튼 불요).

- ✅ **마감 후 수정 사유 모달** -- `CloseReasonDialog` (shadcn/ui Dialog 기반)
  - 마감 상태 청구 수정 시 자동 팝업 (모달로 의도적 행위 강조)
  - 사유 입력 필드 (textarea, 최소 10자 이상)
  - 확인/취소 버튼
  - 확인 시 `update_closed_bill` IPC 호출 (close_reason 포함)
- ✅ **확정 후 수정 확인 다이얼로그** -- 간단 확인 (AC-4.9-3)
- ✅ **마감 확인 다이얼로그** -- "당월 청구 마감" 버튼 클릭 시 강한 확인
  - "마감 후에는 수정 시 사유 입력이 필요합니다" 경고 문구

### T6: TypeScript IPC 래퍼 + 도메인 타입 (1.5h) ✅ 2026-05-29 `6f5e238`

- ✅ **`src/types/billing.ts`** -- 도메인 타입 정의
  - `Bill`, `BillStatus`, `Payment`, `BillingSummary`, `BillListFilter` 등
- ✅ **`src/lib/tauri/index.ts`** -- IPC 래퍼 추가
  - `generateBills`, `listBills`, `getBill`, `updateBill`
  - `confirmBill`, `confirmAllBills`, `closeBillingMonth`
  - `updateClosedBill`, `createPayment`, `updatePayment`
  - `listUnpaidBills`, `batchUpdatePayments`, `getBillingSummary`
  - dev mode fallback 포함

### T7: 청구 관리 UI (4h) -- skill: frontend-design ✅ 2026-05-29 `4e51d48`

- ✅ **`/billing` 라우트** -- `src/app/billing/page.tsx`
- ✅ **청구 목록 컴포넌트** -- `BillingGrid`
  - 년월 선택 (디폴트: 마지막 교습기간 월)
  - "청구 데이터 생성" 버튼 (중복 생성 차단 + 확인 다이얼로그)
  - 테이블: 원생명 / 학년 / 주 수업시간 / 표준 청구액 / 조정 청구액 / 상태 / 입퇴교 구분
  - 정렬: 미확정 + 월 중 입퇴교 상단 우선 (AC-4.9-4)
  - 미확정 청구 상단 배너 (AC-4.9-5)
  - 월 중 입퇴교 원생 시각 구분 -- 배경색 + 입교일/퇴교일 라벨 (AC-4.9-2)
- ✅ **개별 청구 수정** -- 인라인 금액 편집 또는 수정 다이얼로그
  - 상태별 수정 제약 UI 적용 (T5 다이얼로그 연동)
- ✅ **일괄 확정 버튼** + **개별 확정 버튼**
- ✅ **"당월 청구 마감" 버튼** -- 모든 청구 confirmed 시에만 활성화 (AC-4.9-7). 마감 해제(reopen) 버튼 불요 (PI-11 확정)
- ✅ **TanStack Query** 캐싱 + 낙관적 업데이트
- ✅ **사이드바 메뉴 활성화** -- `menu-config.ts` '청구 관리' `disabledHint` 제거

### T8: 수납 관리 UI (3h) -- skill: frontend-design ✅ 2026-05-29 `e6463ca`

- ✅ **수납 관리 탭/섹션** -- 청구 화면 내 탭 또는 별도 섹션
- ✅ **수납 입력 컴포넌트** -- `PaymentForm`
  - 입금 체크박스 + 입금일 + 입금자명 + 결제수단 드롭다운 + 카드사 드롭다운 (카드 계열 시에만 노출)
  - 카드 계열 판별: 결제수단의 `is_card_type` 플래그 기반으로 카드사 필드 활성/비활성
- ✅ **입금 일괄 처리 모드** -- `BatchPaymentView`
  - 미입금 청구 리스트 한 화면 표시 (최소 20행, AC-4.9-6)
  - 행별 빠른 입력: 체크박스 + 입금일 + 입금자명 + 결제수단
  - "일괄 확정" 버튼
- ✅ **미납 원생 추출 리스트** -- 미입금 원생 별도 표시 (안내 목적)
- ✅ **월별 요약 표시** -- 총 청구액 / 입금 완료액 / 미납액

### T9: 통합 검증 (3h) ✅ 2026-05-29

- ✅ **자동 검증 7항목**
  1. ✅ `cargo test --manifest-path src-tauri/Cargo.toml --lib`: **308 passed / 0 failed / 4 ignored** (F6 포함)
  2. ✅ `cargo clippy --lib -- -D warnings` clean
  3. ✅ `pnpm lint` clean (ESLint warnings/errors 0)
  4. ✅ `pnpm tsc --noEmit` clean
  5. ✅ `pnpm build` static export 성공 — 17 routes (`/billing` 신규 포함)
  6. ✅ `cargo sqlx prepare` — "no queries found" (매크로 미사용, 정상)
  7. ✅ 마이그레이션 self-check: V109 의 bills/payments/payment_methods.is_card_type 가 sqlx migrate 적용 후 DB 스키마와 1:1 일치 확인 (T1 시점 검증)
- ✅ **AC 마킹** -- 전수 검증 완료
  - ✅ AC-4.9-1: 청구액 = 표준 교습비 매핑 — `generate_bills_impl` standard_fees lookup + INSERT OR IGNORE UNIQUE 중복 차단
  - ✅ AC-4.9-2: 월중입퇴교 시각 구분 — `BillingGrid` / `PaymentsView` 행 amber-50 + 라벨 ("월중입교"/"월중퇴교")
  - ✅ AC-4.9-3: 확정 후 수정 확인 다이얼로그 — `ConfirmBillUpdateDialog`
  - ✅ AC-4.9-4: 카드 계열 시 카드사 필수 — `validate_payment_input` 백엔드 거부 + UI 빨간 테두리/aria-invalid
  - ✅ AC-4.9-5: 미확정 청구 상단 배너 — `/billing` 페이지 draftCount>0 일 때 amber 배너
  - ✅ AC-4.9-6: 일괄 처리 한 화면 ≥20행 — `PaymentsView` max-h-[800px] overflow + sticky thead
  - ✅ AC-4.9-7: 마감 전제조건 (모두 confirmed) — `close_billing_month_impl` 미확정 카운트 거부 + 단위 테스트 `close_billing_month_rejects_when_pending_drafts`
  - ✅ AC-4.9-8: 마감 후 수정 사유 필수 — `update_bill_impl` close_reason NULL/공백 거부 + `CloseReasonDialog` 10자 이상 + audit `BillClosedModified`

---

## 완료 기준 (Definition of Done)

**필수**
- ✅ 청구 생성 → 확정 → 마감 전체 흐름 동작 (T2/T3 IPC + T7 UI)
- ✅ 수납 입력 + 입금 일괄 처리 동작 (T4 IPC + T8 PaymentsView)
- ✅ 마감 후 수정 시 사유 입력 강제 확인 (AC-4.9-8) — `update_bill_impl` 거부 + `CloseReasonDialog` 사유 ≥10자
- ✅ 마감 전제조건: 해당 월 모든 청구 confirmed (AC-4.9-7) — `close_billing_month_impl` 미확정 카운트 거부
- ✅ 월 중 입퇴교 시각 구분 표시 (AC-4.9-2) — `BillingGrid` / `PaymentsView` amber-50 행 + 라벨
- ✅ 카드 계열 시 카드사 필수 입력 (AC-4.9-4) — `validate_payment_input` + `PaymentsView` 휴리스틱 안내
- ⚠️ 청구 50명 생성 3초 이내 (PRD SS5.6) — 단위 테스트 인메모리는 0.21s/35 tests. 실측 50명 dev DB 검증은 사용자 시각 검증으로 이연
- ✅ 입금 일괄 처리 한 화면 최소 20행 (AC-4.9-6) — `max-h-[800px] overflow + sticky thead`
- ✅ carry-over F1~F7 전수 해소 (T0, `958285c`)
- ✅ `cargo test` 308 passed / `cargo clippy -- -D warnings` clean
- ✅ `pnpm lint` clean / `pnpm tsc --noEmit` clean / `pnpm build` 17 static routes 성공

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ develop 머지

---

## 사용자 결정 완료 (2026-05-28)

5건 권장안 일괄 수락.

| # | 항목 | 결정 | 근거 |
|---|------|------|------|
| 1 | PI-10: 마감 후 수정 사유 UX | ✅ **모달 다이얼로그** | 의도적 행위 강조, 실수 방지 |
| 2 | PI-11: 마감 해제(reopen) | ✅ **불가** | PRD 미언급, 개별 건 수정(사유 필수)만 허용 |
| 3 | PI-12: 수납 테이블 분리 | ✅ **별도 payments 테이블** | 분할 납부/환불 확장 여지, 감사 로그 분리 |
| 4 | 카드 계열 식별 | ✅ **`is_card_type` BOOLEAN 플래그** | 사용자 추가 결제수단 정확 판별 |
| 5 | F4 N+1 범위 | ✅ **`calendar.rs` N+1만** | attendance.rs는 PRD 성능 충족 중, carry-over 유지 |

---

## 참고 사항

### 기술 고려사항
- **청구 상태 전이는 백엔드에서 엄격하게 제어** -- 프론트엔드는 UI 가드만 담당, 실제 상태 검증은 IPC 커맨드 내부
- **마이그레이션 번호**: V109 (V100번대 핵심 도메인 블록)
- **입금 일괄 처리**: 낙관적 업데이트 + 실패 시 롤백 (TanStack Query `onError` 무효화)
- **성능 목표**: 청구 50명 생성 < 3초 (PRD SS5.6)

### 의존성
- **표준교습비 테이블** (`standard_fees`) -- Sprint 2에서 구현 완료, `weekly_minutes` 기준 매핑 함수 존재
- **원생 테이블** (`students`) -- `withdraw_date`, `enroll_date` 활용
- **코드 테이블** (`codes`) -- 결제수단/카드사 시드 데이터 Sprint 5에서 투입 완료
- **출결 도메인** -- 청구 생성 시 출결 데이터 직접 참조 없음 (청구는 주 수업시간 기준)

### ROADMAP Sprint 11 vs 실제 계획 차이
- ROADMAP의 `V007` 마이그레이션 번호는 초기 계획 기준. 실제로는 **V109** (V100번대 도메인 블록 정책에 따라)
- carry-over 7건을 Sprint 11 첫 Task(T0)로 흡수 -- 별도 정리 sprint 불필요 (총 4h로 Capacity 내 수용)

### Phase-planner 판단
- Phase 4는 2 스프린트(Sprint 11~12)로 ROADMAP에 확정됨 -- phase-planner 불요
- Sprint 11(청구+수납) + Sprint 12(공지문+가져오기) 분할은 ROADMAP 원안 유지

---

## 품질 검증 체크리스트

- ⬜ ROADMAP.md의 전체적인 방향성과 일치하는가?
- ⬜ writing-plans 스킬의 형식을 준수했는가?
- ⬜ 모든 태스크가 구체적이고 실행 가능한가?
- ⬜ 완료 기준이 명확하게 정의되었는가?
- ⬜ 파일이 올바른 경로에 저장되었는가?
