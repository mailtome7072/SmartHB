# Sprint Retrospective — Sprint 11

> Sprint 11: Phase 4 첫 마일스톤 — 청구+수납 도메인 완성
> 기간: 2026-05-28 ~ 2026-05-29 (2일, ~4 세션)
> 브랜치: `sprint11` (develop 머지 대기)
> 코드 리뷰 (2026-05-29): Critical 0 / High 0 / Medium 2 (F1, F3) / Low 2 (F2, F4)
> 자동 검증: cargo test cipher off 308 passed / clippy clean / lint clean / tsc clean / build 17/17

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint10-retrospective.md` (A58~A68)

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A58 | `build_day_schedules` succ_opt().expect() → ok_or_else() 전환 (R71) | ✅ 완료 | T0/F1에서 `.ok_or_else()` 안전 전환 완료 (`attendance.rs`) |
| A59 | `generate_impl` expire 호출 fail-soft 전환 (R72) | ✅ 완료 | T0/F2에서 fail-soft match 전환 완료. startup.rs 패턴 통일 |
| A60 | `get_makeup_management_data_impl` `_year_month` 파라미터 정리 | ✅ 완료 | T0/F3에서 파라미터 제거 + 함수 시그니처 정비 (`calendar.rs`) |
| A61 | `get_makeup_management_data_impl` N+1 → IN batch 전환 (R64 패턴) | ✅ 완료 | T0/F4에서 HashSet 기반 IN 절 batch 쿼리 전환 완료 (`calendar.rs`) |
| A62 | flaky 테스트 `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` `#[ignore]` 마킹 (R70) | ✅ 완료 | T0/F6에서 `#[ignore]` 마킹 + 상세 주석 추가 (`auth.rs`) |
| A63 | `get_absence_history` pagination.rs 헬퍼 적용 | ⏸️ 이연 | Phase 4 scope 외 유지 |
| A64 | `create_makeup_with_absences_impl` JSON_EACH 전환 | ⏸️ 이연 | Phase 4 scope 외 유지 |
| A65 | `get_attendance_grid` N+1 → batch 쿼리 | ⏸️ 이연 | Phase 4 scope 외 유지 |
| A66 | salt buffer ZeroizeOnDrop | ⏸️ 이연 | 보안 도메인. Phase 4 이후 |
| A67 | 반응형 폰트/셀 너비 clamp() | ⏸️ 이연 | UX 전반 |
| A68 | 한글 자모 부분 일치 검색 | ⏸️ 이연 | 검색 도메인 |

이행률: **적용 가능한 5건 중 5건 (100%)** — A58~A62 전원 T0 carry-over로 완료.

---

## 잘한 점

**T0 carry-over 일괄 흡수 패턴이 계획 대비 2.6배 빠르게 처리됐다.**

T0 carry-over 7건(F1~F7 + flaky + 메뉴)을 4h로 계획했으나 실측 약 1.5h에 완료. 수정 범위가 명확한 항목(succ_opt 전환, fail-soft 변경, 파라미터 정리, N+1 배치)을 한 세션에 집중 처리한 것이 효율을 이끌었다. Sprint 10 회고에서 A58~A62로 구체적으로 기록된 항목을 그대로 T0에 배정하는 방식이 유효했다. **다음 스프린트도 T0 carry-over 패턴을 유지한다.**

**Step-back 효과 — T1 진입 시 가정 오류를 조기 발견했다.**

Sprint 11 scope.md 초안에 "payment_methods는 codes 단일 테이블로 관리"라는 잘못된 가정이 있었다. T1 DB 마이그레이션 작성 전 실제 스키마(`payment_methods` 별도 테이블, `card_companies` 별도 테이블)를 확인하는 step-back으로 가정을 수정했다. 이 정정이 없었다면 V109 마이그레이션 설계가 틀어졌을 것이다. 비용은 30분 내외였고 얻은 이득은 크다. **DB 스키마 가정이 포함된 Task는 반드시 실제 테이블 구조를 먼저 확인한다.**

**Sprint 10 UI 컴포넌트 패턴 재사용으로 T5~T8 프론트엔드 개발 속도가 높았다.**

`CloseReasonDialog`, `ConfirmBillUpdateDialog`, `CloseMonthDialog` 3종은 Sprint 10의 `WithdrawalMakeupDialog` 패턴(`fixed inset-0 z-[60]`, 상태 관리, 에러 표시 구조)을 그대로 따랐다. `PaymentsView`와 `/billing` 탭 구조는 Sprint 10 `/schedules` 라우트와 탭 통합 패턴을 재사용했다. 패턴이 확립된 이후 컴포넌트 추가는 설계 비용 없이 빠르게 진행됐다. **스프린트마다 신규 컴포넌트 패턴을 명시적으로 회고에 기록해두면 다음 스프린트 재사용이 쉬워진다.**

**백엔드 IPC 단위 테스트 35건을 구현과 동시에 작성했다.**

T2(17건), T3(9건), T4(9건) 각 IPC를 작성하면서 단위 테스트를 즉시 함께 작성했다. 청구 상태 머신 경계 조건(재확정 거부, 마감된 확정 거부, draft 없을 때 마감 OK)과 수납 트랜잭션 롤백(카드사 누락 시 배치 전체 취소)이 테스트로 명확하게 보장됐다. PRD §6.5 비즈니스 규칙 100% 단위 테스트 커버 지속.

---

## 아쉬운 점 / 개선 점

**Hook 차단이 일부 마찰을 유발했다.**

`posttooluse-scope-tracker.sh`의 scope.md 수정 횟수 자동 증가와 `tauri.conf.json` Forbidden Area 차단이 세션 중 수차례 발동했다. scope.md `[1회]` 자동 갱신은 루프 감지 목적이므로 유효하나, `tauri.conf.json` 차단은 수정이 실제로 필요 없었음에도 도구 실행 흐름이 중단됐다. Hook 자체는 올바른 보호 정책이지만, 불필요한 파일 접근 시도를 줄이는 사전 확인 습관이 필요하다.

**summaryQuery 실패 경로가 코드 리뷰에서 발견됐다 (F1).**

`CloseMonthDialog`를 `{summary && ...}`로 게이팅한 것은 summary 미로드 상태에서의 렌더링 방지 목적이지만, summaryQuery 실패 시 버튼이 표시된 채 다이얼로그가 미출력되는 케이스가 가능하다. 두 Query가 병렬 실행되므로 실제 발생 가능성은 낮으나, **다중 Query 의존 UI는 모든 Query 실패 경로를 함께 고려하는 습관이 필요하다.**

**PaymentsView dirtyEntries 필터 설계가 payerName 단독 입력을 고려하지 않았다 (F3).**

`isPaid || paymentMethodId !== null` 필터는 "입금 완료 또는 결제수단 선택" 기준으로 논리적으로 맞지만, 사용자가 payerName만 먼저 입력하는 운용 패턴을 배제했다. UI에 안내가 없어 사용자 혼란 가능성이 있다. **입력 폼 설계 시 사용자의 부분 입력 시나리오를 명시적으로 검토한다.**

---

## 다음 sprint 액션 항목

| ID | 항목 | 우선순위 | 위치 |
|----|------|----------|------|
| A69 | F1 — CloseMonthDialog `showCloseButton` 조건에 `summary !== undefined` 추가 (R80) | Medium | `src/app/billing/page.tsx:125` |
| A70 | F3 — PaymentsView dirtyEntries 필터에 `d.payerName !== ''` 추가 또는 안내 메시지 표시 (R81) | Low | `src/components/billing/PaymentsView.tsx:103` |
| A71 | R77 — develop 머지 후 50명 시드 환경에서 generate_bills 3초 이내 실측 확인 | Medium | 수동 검증 (DEPLOY.md) |
| A72 | R79 — PaymentsView 카드 계열 휴리스틱을 `is_card_type` 필드 기반으로 전환 (향후 결제수단 확장 대비) | Low | `src/components/billing/PaymentsView.tsx` + IPC |
| A73 | F4 — seed_student 테스트 헬퍼 `.bind(withdraw)` 파라미터 바인딩으로 전환 | Low | `src-tauri/src/commands/billing.rs:928` |
| A74 | A63 이월 — `get_absence_history` pagination.rs 헬퍼 적용 | Medium | `makeup.rs:742-780` |
| A75 | A64 이월 — `create_makeup_with_absences_impl` JSON_EACH 전환 | Low | `makeup.rs:434-467` |
| A76 | A65 이월 — `get_attendance_grid` N+1 → batch 쿼리 | Medium | `attendance.rs` |
| A77 | A66 이월 — salt buffer ZeroizeOnDrop | Medium | `auth.rs` |
| A78 | A67 이월 — 반응형 폰트/셀 너비 clamp() | Medium | `globals.css` + 컴포넌트 |
| A79 | A68 이월 — 한글 자모 부분 일치 검색 | Medium | `global-search.tsx` |
| A80 | R82 — 마감 후 추가 청구 정책 결정 + 구현 | Medium | `billing.rs::generate_bills` + UI 안내 |

**Sprint 12 진입 전 처리 권장**: A69(다이얼로그 게이팅), A71(성능 실측), **A80(마감 정책 결정 — PRD §4.9.7 보강 필요)**. 나머지는 Sprint 12 T0 carry-over로 흡수.

### A80 보강 — 마감 후 추가 청구 정책

post-Sprint 11 사용자 검토에서 발견된 정책 모호 (R82). `generate_bills` 가 INSERT OR IGNORE 라
마감된 월에 신규 학생이 등록되면 그 학생만 `draft` 로 추가 INSERT 됨. 사용자가 확정·재마감하면
같은 월에 두 시점의 `closed_at` 이 공존 — 회계상 "마감"의 본질(시점 잠금)과 충돌 가능.

검토 옵션:
- (a) **마감 후 신규 청구 차단** — `generate_bills` 가 month status='closed' 일 때 거부. 신규 학생은 다음 월부터.
- (b) **별도 보류 상태** — `pending_after_close` 상태로 추가. 별도 처리 흐름.
- (c) **현재 동작 유지 + UI 강한 안내** — 마감된 월에 "추가 청구 데이터 생성" 버튼 클릭 시 경고 다이얼로그.

원장 운영 흐름 확인 후 Sprint 12 초반에 결정·구현. PRD §4.9.7 보강 필요.

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 기간 | 2일 (2026-05-28 ~ 2026-05-29) |
| 커밋 수 (sprint11 브랜치 신규, docs 제외) | 10건 (T0~T9 각 1커밋 + sprint-close) |
| 백엔드 IPC 신규 | 13종 (T2 5종 + T3 3종 + T4 5종) |
| 단위 테스트 신규 | billing.rs 35건 (T2 17 + T3 9 + T4 9) |
| 누적 단위 테스트 (cipher off) | **308 passed** (Sprint 10 273 → +35) |
| TS IPC 래퍼 신규 | 13종 (T6) |
| 도메인 타입 신규 | `Bill`, `BillStatus`, `Payment`, `BillingSummary`, `PaymentInput`, `UnpaidBill`, `GenerateBillsResult` |
| UI 컴포넌트 신규 | 5종 (`BillingGrid`, `PaymentsView`, `CloseReasonDialog`, `ConfirmBillUpdateDialog`, `CloseMonthDialog`) |
| 마이그레이션 신규 | 1건 (V109 bills + payments + payment_methods.is_card_type) |
| 자동 검증 통과 | 5/5 |
| sprint-review 결함 | 4건 (F1/F3 Medium, F2/F4 Low) — Critical/High 0건 |
| Capacity | 계획 30.5h → 실측 약 16.5h (54%) |
| 이전 회고 액션 이행률 | 5/5 적용 가능 항목 100% (A58~A62) |

---

## 종합 평가

Sprint 11은 Phase 4 첫 마일스톤(청구+수납 도메인)을 2일 만에 완성했다. 계획 capacity 30.5h 대비 실측 약 16.5h(54%)로 절반 수준에서 완료됐다. T0 carry-over 5건을 1.5h에 처리(계획 4h)한 것이 여유를 만들었고, Sprint 10 UI 패턴 재사용이 프론트엔드 개발 속도를 높였다.

Critical/High 결함 없이 개발이 완료됐다. 코드 리뷰에서 발견된 F1(CloseMonthDialog 게이팅), F3(payerName 소실)은 다이얼로그/폼 설계에서 엣지 케이스 검토 습관이 부족했음을 보여준다. 두 건 모두 ROADMAP에 등록하여 Sprint 12 초반에 처리한다.

T0 carry-over 패턴(Sprint N-1 회고 액션 → Sprint N T0 일괄 처리)이 Sprint 10에 이어 다시 유효성을 입증했다. 이 패턴을 Sprint 12에도 유지한다.

**Phase 4 청구+수납 완성 — Sprint 12 (공지문 이미지 생성 + 대시보드 위젯) 준비 완료.**
