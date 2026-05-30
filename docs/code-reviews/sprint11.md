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

---

# post-Sprint 11 develop 보완 4커밋 코드 리뷰 (2026-05-30 추가)

> 대상: `70c59a1` `c1ae063` `2a964b0` `29fbe93` — 월별 집계 탭 + 마감 폐기 + 기간 한정 + 빈 데이터 디폴트
> 리뷰 일자: 2026-05-30
> 자동 검증 결과: cargo test 312 passed (cipher off) / clippy clean / lint clean / tsc clean / build OK

## 발견 사항 (2건)

### 🟡 F1 — update_bill_impl current_status dead variable (Medium, 미수정)

- 위치: `src-tauri/src/commands/billing.rs:287~293`
- 내용: `current_status` 조회 결과를 `ok_or_else()?`로 존재 확인만 하고 status 값 자체는 미사용. `SELECT status`와 `SELECT is_paid` 두 번 쿼리 왕복. clippy 경고 없이 통과하나 설계 비효율 및 R83 미해소.
- 실패 시나리오: 단일 PC 앱이라 실제 레이스 없음. 그러나 status + is_paid를 단일 LEFT JOIN으로 조회하지 않아 미세 불일치 window 존재 (R83 지속).
- 조치: `SELECT b.status, COALESCE(p.is_paid, 0) AS is_paid FROM bills b LEFT JOIN payments p ON p.bill_id=b.id WHERE b.id=?` 단일 쿼리로 통합. Sprint 12 carry-over (R86).

### 🔵 F2 — BillingSummaryView 년/월 토글에 checkbox 사용 (Low, 미수정)

- 위치: `src/components/billing/BillingSummaryView.tsx:90`
- 내용: 상호 배타 토글에 `type="checkbox"` 사용. 의미론 측면에서 `type="radio"`가 올바름. 스크린리더가 "체크박스 2개"로 안내 → 50대 사용자 접근성 기준(PRD §5.7) 미충족 가능.
- 실패 시나리오: 동작은 정상(onChange에서 mode 강제 전환). 접근성 도구 사용자만 혼란 가능. 시각 동작은 완전 정상.
- 조치: `type="radio"` + `name="billing-mode"` + `role="radiogroup"` 전환. 1줄 변경. Sprint 12 carry-over (R87).

## 영역별 추가 점검

- 보안 (backend.md Critical) — V111 마이그레이션: `_payments_backup` TEMP TABLE 패턴 안전. DROP bills 전 payments 비워서 CASCADE 데이터 소실 방지 확인. `PRAGMA foreign_keys`는 앱 연결 레벨에서 ON — 마이그레이션 트랜잭션 내 TEMP TABLE은 정상 동작. closed→confirmed 변환 누락 없음 확인 (`CASE WHEN status='closed' THEN 'confirmed' ELSE status END`).
- 보안 (backend.md High) — `period_like_pattern`: 'YYYY'→4자리 digit 체크→"YYYY-%" 변환. 5자리 입력('20260' 등)은 `validate_year_month` 통과 실패(len≠7)로 에러 반환 확인. LIKE 와일드카드 위치 안전('2026-'으로 다른 연도 누수 없음). 인덱스 재생성 확인(`idx_bills_year_month/student/status` V111에서 재생성).
- 프론트엔드 (frontend.md Critical) — `dangerouslySetInnerHTML` 미사용. `invoke()` 직접 호출 없음. 민감 정보 미노출.
- 프론트엔드 (frontend.md High) — `BillingSummaryView` useEffect 의존성: `[monthOptions, selectedMonth]` / `[yearOptions, selectedYear]` — monthOptions는 useMemo 안정화(billed 참조가 바뀌지 않으면 재생성 없음), selectedMonth/selectedYear는 state이므로 무한루프 없음. 조건부 setSelectedMonth는 `!monthOptions.includes(selectedMonth)` 가드로 중단됨 확인.
- AI 생성 코드 추가 체크 — V111 마이그레이션 실DB 시각검증 사용자 완료. 신규 단위 테스트 3건 모두 happy path + edge case 커버.

## 결론

Critical 0건, High 0건, Medium 1건(F1 dead variable + 2쿼리 비효율), Low 1건(F2 checkbox 의미론). V111 마이그레이션 안전성 확인. 마감 개념 제거 완전성 확인(타입·쿼리·감사·UI 잔재 없음). 프로덕션 배포 차단 요인 없음.
