# Test Report — 2026-05-29 (Sprint 11)

> Sprint 11: Phase 4 첫 마일스톤 — 청구+수납 도메인 완성 (PRD §4.9)
> 검증 일자: 2026-05-29
> 브랜치: sprint11

---

## 자동 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --lib` (cipher off) | **통과** | 308 passed / 0 failed / 4 ignored (26.75s) |
| `cargo test --features cipher` | 미실행 | macOS 환경 Perl 의존 — 회사 PC 검증으로 이연. cipher off 35건 신규 포함 전체 구조 동일 |
| `cargo clippy --lib -- -D warnings` (cipher off) | **통과** | 경고 0건 (6.71s) |
| `pnpm tsc --noEmit` | **통과** | 에러 0건 |
| `pnpm lint` | **통과** | ESLint 경고/에러 0건 |
| `pnpm build` | **통과** | static export 17/17 페이지 생성 성공 (`/billing` 신규 포함) |

---

## 단위 테스트 신규 추가 (35건)

| 모듈 | Task | 신규 테스트 | 주요 케이스 |
|------|------|------------|------------|
| `billing.rs` — T2 청구 IPC | T2 | 17건 | year_month_range 계산, 월중입교/퇴교 플래그, 일괄 생성 UNIQUE 차단, 표준교습비 매핑 없음 skip, 정렬 순서(draft+midmonth 최상단), 음수 조정 금액 거부, closed 수정 시 close_reason 필수, 공백 사유 거부, 디폴트 청구년월 반환 |
| `billing.rs` — T3 상태 머신 | T3 | 9건 | draft→confirmed 전이, 재확정 거부, closed 확정 거부, 존재하지 않는 청구 거부, 일괄 확정 draft만 영향, 미확정 있을 때 마감 거부 + 롤백 확인, confirmed→closed 전이, 청구 없는 월 마감 OK |
| `billing.rs` — T4 수납 IPC | T4 | 9건 | 수납 생성+라벨 반환, 카드사 누락 거부(AC-4.9-4), 카드+카드사 통과, 입금일 없이 paid 거부, 수납 갱신, 미납 목록 유납 제외, 일괄 UPSERT 트랜잭션, 일괄 카드사 누락 전체 롤백, 청구 요약 합계 계산 |
| 합계 신규 | — | **35건** | cipher off 누적: 308 passed (Sprint 10 273 → +35) |

---

## 마이그레이션 self-check (A39)

| 항목 | 결과 |
|------|------|
| `src-tauri/migrations/` 파일 수 | V001~V009 + V101~V105 + V200~V201 + V109 = 14파일 |
| `src-tauri/.sqlx/` 캐시 파일 | V109 대응 쿼리 캐시 갱신 확인 (`sqlx prepare` 완료) |
| 마이그레이션 순서 정합 | V109 단독 파일 — 사전순(109 > 105 > 009) 정상 |
| 신규 테이블 | `bills` (UNIQUE: student_id+bill_year_month), `payments` (UNIQUE: bill_id) |
| 기존 테이블 변경 | `payment_methods.is_card_type` ALTER TABLE ADD COLUMN (SQLite 호환, DEFAULT 0) |

---

## 수동 검증 항목

- `pnpm tauri:dev` 실행 후 앱 동작 수동 확인: ⬜ 미완료 (개발자 수행 필요)
  - ⬜ `/billing` 진입 → "청구 데이터 생성" → 청구 목록/금액 확인
  - ⬜ 개별 확정 / "미확정 N건 일괄 확정" / "당월 청구 마감" 흐름
  - ⬜ 수납 탭 → 입금 일괄 처리 (카드사 필수 검증 동작)
  - ⬜ 마감 후 금액 수정 시 CloseReasonDialog (10자 이상 사유) 동작
  - ⬜ 사이드 메뉴 '보강 관리' 미노출 확인 (F7 carry-over)

---

## 결론

5개 자동 검증 항목 모두 통과. 단위 테스트 35건 신규 추가로 누적 308건(cipher off) 달성. Critical/High 발견 0건. Medium 2건(F1 CloseMonthDialog 의존, F3 PaymentsView payerName 소실)은 코드 리뷰 보고서(`docs/code-reviews/sprint11.md`) 참조.
