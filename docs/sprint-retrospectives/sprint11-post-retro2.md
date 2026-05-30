# Sprint Retrospective — post-Sprint 11 develop 보완 4커밋

> 대상: develop 브랜치 직접 커밋 4건 (`70c59a1`, `c1ae063`, `2a964b0`, `29fbe93`) — Sprint 11 마감 폐기 결정 + 월별 집계 탭 신규 + 후속 보완
> 리뷰 일자: 2026-05-30
> 코드 리뷰: Critical 0 / High 0 / Medium 1 (F1) / Low 1 (F2)
> 자동 검증: cargo test 312 passed (cipher off) / clippy clean / lint clean / tsc clean / build OK

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint11-post-retro.md` (A81~A84)

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A69 | CloseMonthDialog `summary !== undefined` 게이팅 | ✅ 해소 | `CloseMonthDialog` 자체가 V111에서 폐기됨 — 문제의 근본이 제거됨 |
| A80 | 마감 정책 결정 — 마감 유지 vs 폐기 | ✅ 완료 | 원장 결정으로 마감 개념 전면 폐기. V111 마이그레이션 + `c1ae063` 실행됨 |
| A81 | R83 — `update_bill_impl` 2쿼리 비효율 | ⏸️ 미이행 | 금번 커밋에서 close_reason 제거로 코드 단순화됐으나 2쿼리 패턴 유지. R86으로 재등록 |
| A82 | F2 — `verify_password` 주석 | ⏸️ 미이행 | 금번 커밋 범위 외. Sprint 12 T0 carry-over |
| A83 | 생체인증 병행/대체 정책 | ⏸️ 미이행 | Phase 5 이후 장기 과제. 유지 |
| A84 | 마감 청구 수납취소 정책 명시 | ✅ 해소 | 마감 개념 폐기로 문제 자체가 소멸 |

---

## 잘한 점

**원장 결정을 코드에 빠르게 반영하여 기술 부채를 정리했다.**

"마감 후 수정 시 사유 입력" 워크플로우(Sprint 11에서 AC-4.9-7/8로 구현)를 원장이 실사용 검수 후 "불필요하게 복잡하다"고 판단했고, 그 피드백을 당일 V111 마이그레이션 + `c1ae063`으로 반영했다. `CloseMonthDialog`, `CloseReasonDialog`, `close_billing_month` IPC, 감사 이벤트 2종, `update_bill`의 `close_reason` 파라미터를 일괄 제거하여 코드베이스가 실제 운용 정책과 다시 일치하게 됐다. 불필요한 코드 경로를 장기간 유지하지 않은 점이 좋았다.

**V111 마이그레이션이 V108에서 학습한 FK CASCADE 함정을 올바르게 회피했다.**

`_payments_backup` TEMP TABLE 패턴(payments 백업 → 비우기 → bills 재구성 → 복원)이 V108 SQLite FK deferred 카운터 문제와 동일한 보호 패턴을 사용했다. payments.id / bills.id를 모두 보존하여 복원 시 FK 즉시 만족함을 확인했고, sqlx 마이그레이션 트랜잭션 래핑과 TEMP TABLE 세션 생명주기가 정합임을 검증했다. 주석에 경고(`⚠️ FK 함정 (V108 학습)`)를 명시하여 미래 작업자에게 맥락을 전달한 점도 좋았다.

**월별 집계 기간 선택을 "실제 청구가 존재하는 년월"로 한정하여 UX 혼란을 방지했다.**

`list_billed_months` IPC(`DISTINCT bill_year_month DESC`)로 실존 데이터가 있는 년월만 제시하고, 청구 0건 시에는 현재 년월을 디폴트로 낙하산 처리(`29fbe93`)하여 빈 화면 대신 "0건" 상태를 명확히 노출한다. 사용자가 존재하지 않는 기간을 선택하여 혼란스러운 빈 화면을 보는 케이스를 원천 차단한 작은 UX 개선이다.

**신규 단위 테스트 3건이 핵심 비즈니스 규칙 변경을 검증한다.**

- `update_bill_paid_rejected`: 마감 개념 폐기 후에도 "수납완료된 청구 금액 수정 불가" 규칙이 is_paid 기준으로 유지됨을 증명.
- `list_billed_months_returns_distinct_desc`: distinct + 내림차순 정렬 정합성.
- `billing_period_stats_groups_by_method`: 연도/월 LIKE 패턴이 결제수단별로 올바르게 집계됨. 연도 매칭('YYYY-%')이 다른 연도를 누수하지 않음을 3원생 데이터로 검증.

---

## 아쉬운 점 / 개선 점

**`update_bill_impl`의 current_status 조회가 dead variable 패턴으로 2회 DB 왕복을 유지한다 (F1).**

마감 개념 제거로 update_bill_impl이 단순화됐지만, `SELECT status`로 존재 확인 후 그 값을 사용하지 않고 다시 `SELECT is_paid`로 2번째 쿼리를 실행하는 비효율이 남아 있다. R83(이전 회고)이 R86으로 재등록됐으며 단일 LEFT JOIN으로 해소 가능한 1~2줄 수정이다. Sprint 12 carry-over.

**BillingSummaryView의 년/월 토글이 `type="checkbox"` 사용으로 접근성 의미론이 부정확하다 (F2).**

상호 배타 선택에는 `type="radio"` + `role="radiogroup"`이 의미론 표준이다. 동작은 완전 정상이나, 스크린리더 환경에서 "체크박스"로 안내되어 50대 사용자 접근성 기준(PRD §5.7)과 미미하게 충돌한다. 1줄 변경으로 해소 가능하므로 Sprint 12 T0에서 정리 권장.

---

## 다음 sprint 액션 항목

이전 회고(A69~A84) 처리 상태를 반영하여 잔여 항목:

| ID | 항목 | 우선순위 | 위치 |
|----|------|----------|------|
| A82 | F2 — `verify_password`에 `validate_pin` 미적용 의도를 주석으로 명시 | Low | `auth.rs:664` |
| A83 | 생체인증 도입 시 PIN 병행/대체 정책 결정 | 낮음 | Phase 5 이후 |
| A85 | R86 — `update_bill_impl` status + is_paid 단일 LEFT JOIN 통합 | Low | `billing.rs:287~293` |
| A86 | R87 — `BillingSummaryView` 년/월 토글 `type="radio"` 전환 | Low | `BillingSummaryView.tsx:90` |

Sprint 12 진입 전 처리 권장: A82(주석 1줄), A86(radio 전환 1줄). 나머지 A83/A85는 Sprint 12 T0 carry-over.

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 날짜 | 2026-05-30 (post-Sprint 11 develop 보완 4커밋) |
| 커밋 수 | 4건 (`70c59a1` 월별 집계 탭, `c1ae063` 마감 폐기, `2a964b0` 기간 한정, `29fbe93` 빈 데이터 디폴트) |
| 변경 파일 | 14개 (RS: billing.rs, audit.rs, lib.rs / TS: page.tsx, BillingGrid.tsx, BillingSummaryView.tsx(신규), CloseMonthDialog.tsx(삭제), CloseReasonDialog.tsx(삭제), ConfirmBillUpdateDialog.tsx, tauri/index.ts, types/billing.ts / SQL: V111) |
| DB 마이그레이션 | 1건 (V111 bills 재구성) |
| 삭제 코드 | CloseMonthDialog, CloseReasonDialog, close_billing_month IPC, BillMonthClosed/BillClosedModified audit 이벤트, close_reason 파라미터 |
| 신규 단위 테스트 | billing.rs 3건 (list_billed_months_returns_distinct_desc, billing_period_stats_groups_by_method, update_bill_paid_rejected) |
| 누적 단위 테스트 (cipher off) | **312 passed** (이전 2건 대비: 마감 관련 테스트 삭제 + 신규 3건 순증) |
| 자동 검증 | clippy/lint/tsc/build 4종 통과, cargo test 312 passed |
| sprint-review 결함 | 2건 (F1 Medium, F2 Low) — Critical/High 0건 |

---

## 종합 평가

원장의 실사용 피드백("마감 워크플로우 불필요하게 복잡")을 V111 마이그레이션으로 즉시 반영하여 청구 도메인이 실제 운용 정책과 일치하게 됐다. Sprint 11에서 AC-4.9-7/8로 구현한 마감 워크플로우가 단 1~2주 만에 폐기됐지만, 빠른 피드백 루프와 신속한 코드 정리가 장기적으로는 기술 부채 감소로 이어진다. V111이 V108 FK 함정을 올바르게 회피한 점, 월별 집계 탭이 단위 테스트 포함 완성도 있게 구현된 점도 긍정적이다.

Critical/High 결함 0건으로 프로덕션 배포 차단 요인 없음. 발견 결함 2건(F1 Medium, F2 Low)은 ROADMAP 등록(A85, A86)으로 Sprint 12에서 처리한다.

**청구+수납 도메인 최종 정리 완료 — Sprint 12 (공지문 이미지 생성 + 대시보드 위젯) 진입 준비.**
