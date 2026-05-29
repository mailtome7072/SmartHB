# Sprint 11 코드 리뷰

> 대상: Sprint 11 (develop...HEAD, 28개 파일) — Phase 4 청구+수납 도메인 완성
> 리뷰 일자: 2026-05-29
> 자동 검증 결과: cargo test 308 passed (cipher off) / clippy clean / lint clean / tsc clean / build 17/17

---

## 발견 사항 (4건)

### F1 — CloseMonthDialog가 summaryQuery 실패 시 렌더링 불가 (Medium, 미수정)

- 위치: `src/app/billing/page.tsx:270`
- 내용: `{summary && <CloseMonthDialog ...>}` 조건으로 다이얼로그를 게이팅. `showCloseButton`은 `billsQuery.data` 기준으로 계산되어 버튼이 표시될 수 있으나, `summaryQuery`가 실패(DB 오류 등)하면 `summary=undefined` → 다이얼로그 미렌더링.
- 실패 시나리오: billsQuery 성공(모두 confirmed) + summaryQuery 실패 → "당월 청구 마감" 버튼 표시됨 → 클릭해도 다이얼로그 미출력 → 원장이 마감 불가 상태에서 원인을 알 수 없음.
- 조치: summaryQuery 실패 시 에러 메시지 표시 또는 `showCloseButton` 조건에 `summary !== undefined` 추가 권장. ROADMAP 이연 (운용 실패 가능성 낮음, 일반적으로 두 쿼리는 병렬 성공).

### F2 — update_bill_impl SELECT→UPDATE 사이 트랜잭션 미적용 (Low, 미수정)

- 위치: `src-tauri/src/commands/billing.rs:266`
- 내용: `update_bill_impl`이 `SELECT status → 상태 검사 → UPDATE` 순서로 실행되지만 draft/confirmed 경로에 트랜잭션이 없음. SELECT와 UPDATE 사이에 `close_billing_month`가 실행되면 closed 상태 청구를 `close_reason=NULL`로 UPDATE 가능.
- 실패 시나리오: [드래프트 편집 IPC 요청] 사이에 [마감 IPC 요청]이 인터리브되면, draft 상태로 읽힌 청구가 closed로 전환된 후 else 브랜치 UPDATE 실행 → AC-4.9-8 위반 (close_reason 없이 마감 후 수정). 단일 사용자 데스크톱에서 실제 발생 가능성 매우 낮음.
- 조치: ROADMAP 이연. 보완 시 `BEGIN IMMEDIATE` 트랜잭션으로 SELECT+UPDATE 묶기. 현재 단일 사용자 모델에서 실질 위험 미미.

### F3 — PaymentsView dirtyEntries 필터에서 payerName 단독 입력 소실 (Medium, 미수정)

- 위치: `src/components/billing/PaymentsView.tsx:103`
- 내용: `dirtyEntries = drafts.filter(d => d.isPaid || d.paymentMethodId !== null)`. payerName만 입력하고 isPaid 체크 및 결제수단 미선택 시 해당 행이 필터에서 제외되어 저장 대상에서 누락.
- 실패 시나리오: 원장이 입금자 이름만 먼저 입력 후 "선택 일괄 저장" 클릭 → 버튼 비활성(dirtyEntries.length=0) 또는 해당 행 미포함 저장 → payerName 소실. UI에 별도 안내 없음.
- 조치: `d.payerName !== ''`를 dirtyEntries 필터에 추가하거나, 저장 버튼 비활성 이유를 UI에 표시 권장. 또는 "입금자명만 입력은 저장 불가" 정책을 명시적 안내로 처리. ROADMAP 이연.

### F4 — 테스트 헬퍼 seed_student에서 format!()으로 SQL 직접 삽입 (Low, 미수정)

- 위치: `src-tauri/src/commands/billing.rs:928`
- 내용: `seed_student` 테스트 헬퍼가 `withdraw_clause = format!("'{}'", w)`로 문자열을 SQL에 직접 삽입. `#[cfg(test)]` 블록 내 테스트 전용 코드이나, sqlx 파라미터 바인딩 패턴(`?`)과 불일치.
- 실패 시나리오: 테스트 코드 이므로 프로덕션 영향 없음. 그러나 withdraw 값에 작은따옴표가 포함되면 SQL 오류 또는 주입 가능. withdraw 값이 하드코딩된 날짜 문자열이므로 실제 발생 가능성 없음.
- 조치: 다음 기회에 `.bind(withdraw_opt)`로 통일 (Low, 테스트 코드 정리).

---

## 영역별 추가 점검

- 보안 (backend.md Critical) — SQL 인젝션 없음. 모든 IPC 쿼리는 `?` 파라미터 바인딩. 하드코딩 시크릿 없음.
- 보안 (backend.md High) — `unwrap()` 프로덕션 코드 미사용. 마이그레이션 V109 정상 적용. `.sqlx/` 캐시 갱신 확인. `PRAGMA integrity_check` 누락 없음 (startup.rs 기존 로직).
- 프론트엔드 (frontend.md Critical) — `dangerouslySetInnerHTML` 미사용. `invoke()` 직접 호출 없음 (`src/lib/tauri/index.ts` 경유). 민감 정보 localStorage 미저장.
- 프론트엔드 (frontend.md High) — TypeScript `any` 미사용. SSR 가드 불필요 (모든 billing 컴포넌트 `'use client'`). 글로벌 검색바 AppShell 내 포함 확인. 버튼 min-h-[44px] 접근성 기준 준수 확인.
- AI 생성 코드 추가 체크 — 비즈니스 규칙(청구 상태 머신 3단계, 마감 후 수정 사유, 카드사 필수) 단위 테스트 35건 커버. 도메인 타입 백엔드 serde camelCase와 1:1 정합 확인.

---

## 결론

Critical 0건, High 0건, Medium 2건(F1 다이얼로그 게이팅, F3 payerName 소실), Low 2건(F2 TOCTOU, F4 테스트 SQL 직접 삽입). 프로덕션 배포 차단 요인 없음. F1/F3는 ROADMAP에 등록하여 Sprint 12 또는 Phase 4 마감 시 처리 권장.
