# Sprint Retrospective — post-Sprint 11 develop 보완 2건

> 대상: develop 브랜치 직접 커밋 2건 (`945e4a7`, `c93399e`) — Sprint 11 사용자 검수 후속 보완
> 리뷰 일자: 2026-05-30
> 코드 리뷰: Critical 0 / High 0 / Medium 1 (F1) / Low 2 (F2, F3)
> 자동 검증: cargo test 314 passed / 1 flaky (기존, lock.rs) / clippy clean / lint clean / tsc clean / build OK

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint11-retrospective.md` (A69~A80)

이번 보완 2건은 Sprint 11 직후 **사용자 검수에서 발견된 이슈** 처리로, 정식 Sprint 12 범위 이전에 develop 직접 커밋으로 완료됐다. Sprint 11 액션 아이템 중 금번 처리된 항목:

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| — | `945e4a7` 청구/수납 검수 후속 보완 8건 | ✅ 완료 | Sprint 11 검수에서 발견된 UX/로직 이슈 8건 일괄 해결 |
| — | `c93399e` PIN 전환 (ADR-007) | ✅ 완료 | 원장 피드백 — 매 실행 시 8자+ 비밀번호 입력 부담 → 6자리 PIN으로 전환 |
| A69 | F1 CloseMonthDialog `summary !== undefined` 게이팅 | ⏸️ 미이행 | 금번 커밋 범위 외. Sprint 12 T0 carry-over 유지 |
| A70 | F3 dirtyEntries payerName 필터 | ⏸️ 미이행 | 금번 커밋 범위 외. Sprint 12 T0 carry-over 유지 |

---

## 잘한 점

**검수 발견 이슈 8건을 단일 커밋으로 빠르게 해결했다.**

사용자 검수에서 확인된 UX 문제(마감 필터 부재, 건수 표기 누락, 포커스 이동)와 로직 결함(확정 버튼 비활성 버그, 결제수단 미검증, 수납취소 부재, 수납완료 마감 편집 불가)을 단일 커밋 `945e4a7`으로 일괄 처리했다. 신규 IPC를 추가하지 않고 기존 `batch_update_payments`를 재사용하여 수납취소를 구현한 것은 최소 변경 원칙에 부합한다. 신규 단위 테스트 3건(`update_bill_closed_paid_rejected`, `create_payment_rejects_paid_without_method`, `batch_cancel_payment_resets_is_paid`)도 동시에 작성해 회귀 방지를 보장했다.

**PIN 전환이 ADR-007 작성 → 백엔드 게이트 → 프론트 전환 3단계 순서를 지켰다.**

설계 결정(ADR-007: 보안 트레이드오프 명시, 위협 모델 판단, 완화책)을 먼저 기록한 뒤 백엔드 `validate_pin`(길이 6 + ASCII 숫자) 진입점 게이트를 추가하고, 이후 `LockScreen` / `RecoveryCodeInput` 프론트 전환 순서로 작업했다. 중요한 보안 변경에 ADR이 선행된 점이 좋았다.

**프론트 PIN 입력 UX가 50대 사용자 친화적으로 설계됐다.**

`inputMode="numeric"` + `maxLength=6` + 숫자 외 `onChange` 필터 + 가운데 정렬 + 넓은 자간(`tracking-[0.4em]`) + `●` 플레이스홀더 조합이 PIN 전용 입력 인터페이스를 명확하게 구현했다. 이전 `PasswordField`의 한글/영문 IME 감지 배지 및 안내 메시지도 깔끔하게 제거돼 화면이 단순해졌다.

---

## 아쉬운 점 / 개선 점

**`update_bill_impl` 내 수납 상태 조회가 트랜잭션 외부 단건 조회다 (F1).**

`status` 조회 후 `is_paid` 조회 사이 window가 존재한다. 단일 PC 오프라인 앱이라 실질 발생 가능성은 극히 낮지만, 두 조회를 단일 LEFT JOIN 쿼리로 통합하는 것이 설계 일관성 면에서 더 낫다. R83으로 등록했다. 비즈니스 로직은 올바르고 현 설계도 기능상 문제없으므로 하드 블로커는 아니다.

**`verify_password` 경로에 `validate_pin`이 없다 (F2 — 설계상 의도적).**

ADR-007에 "verify_password는 형식 검증 없이 키 비교만 수행"으로 명시됐으나, 코드 독자 입장에서는 `set_password`와 일관성이 없어 보일 수 있다. 짧은 주석이 있으면 의도가 더 명확해진다. 실질 보안 문제는 없다 (형식 불일치 시도는 키 불일치로 인증 실패).

**수납취소 대상(마감+수납완료)에 대한 백엔드 게이트가 없다 (F3 — 현재 정책).**

`batch_update_payments`는 `is_paid=false` UPSERT를 마감 상태 청구에도 허용한다. 현재 화면 흐름상 마감된 청구는 PaymentsView에 노출되지 않아 실질 혼용 가능성은 낮지만, 장기적으로 마감 청구의 수납 상태 변경이 정책상 허용되는지 명시가 필요하다. PRD §4.9.7(청구 마감 워크플로우)에 이 케이스가 기술되지 않은 점도 확인이 필요하다.

---

## 다음 sprint 액션 항목

Sprint 11 회고(A69~A80)를 유지하며, 금번 리뷰에서 추가된 항목:

| ID | 항목 | 우선순위 | 위치 |
|----|------|----------|------|
| A81 | R83 — `update_bill_impl` status + is_paid 조회를 단일 LEFT JOIN으로 통합 | Low | `billing.rs:282~299` |
| A82 | F2 — `verify_password` 에 `validate_pin` 미적용 의도를 짧은 주석으로 명시 | Low | `auth.rs:664` verify_password 함수 앞 |
| A83 | R85/ADR-007 — 향후 생체인증(Touch ID/Windows Hello) 도입 시 PIN 병행 vs 대체 정책 결정 (Phase 5 이후) | 낮음 | ADR 신규 또는 ADR-007 업데이트 |
| A84 | F3 — 마감 청구의 수납 취소 허용 여부를 PRD §4.9.7에 명시하고 필요 시 백엔드 게이트 추가 | Medium | PRD §4.9.7 + `batch_update_payments_impl` |

Sprint 12 진입 전 처리 권장: A69(CloseMonthDialog 게이팅), A71(성능 실측), A80(마감 정책 결정), A84(마감 청구 수납취소 정책 명시). 나머지는 Sprint 12 T0 carry-over로 흡수.

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 날짜 | 2026-05-30 (post-Sprint 11 develop 보완) |
| 커밋 수 | 2건 (`945e4a7` billing 8건, `c93399e` PIN 전환) |
| 변경 파일 | 8개 (RS: billing.rs, auth.rs, recovery.rs / TS: page.tsx, LockScreen.tsx, RecoveryCodeInput.tsx, BillingGrid.tsx, CloseReasonDialog.tsx, PaymentsView.tsx) |
| 신규 단위 테스트 | billing.rs 3건 + auth.rs 2건 = 5건 |
| 누적 단위 테스트 (cipher off) | **314 passed** (Sprint 11 308 → +6, flaky 1건 기존 lock.rs) |
| ADR 신규 | 1건 (ADR-007 PIN 전환) |
| 자동 검증 | clippy/lint/tsc/build 4종 통과, cargo test 1 flaky (기존, 금번 무관) |
| sprint-review 결함 | 3건 (F1 Medium, F2/F3 Low) — Critical/High 0건 |

---

## 종합 평가

post-Sprint 11 사용자 검수 후속 보완을 2건 커밋(8+1개 이슈)으로 완료했다. 청구/수납 도메인의 UX 빈틈(필터, 건수 표기, 포커스, 수납취소, 결제수단 검증)과 PIN 인증 전환이 모두 Critical/High 결함 없이 마무리됐다. 신규 단위 테스트 5건으로 핵심 비즈니스 규칙 회귀를 방어했다. 발견된 결함 3건(F1 Medium, F2/F3 Low)은 ROADMAP 등록(A81~A84)하여 Sprint 12에서 처리한다.

PIN 전환은 ADR-007 보안 트레이드오프를 명시 수용한 결정으로, 50대 단독 운영자 UX와 현실적 위협 모델을 균형 있게 고려했다. 복구 코드 12자리 + PBKDF2 600K iter 유지로 최소한의 보안 강도를 보존했다.

**청구+수납 도메인 검수 완료 — Sprint 12 (공지문 이미지 생성 + 대시보드 위젯) 진입 준비.**
