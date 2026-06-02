# Sprint Retrospective sprint13

> 대상: Sprint 13 (develop...sprint13) — PIN 인증 옵션화 (ADR-008) + Phase 5 취소 반영 + carry-over 해소 + 글로벌 검색 버그 수정 2건
> 리뷰 일자: 2026-06-02
> 코드 리뷰: Critical 0 / High 0 / Medium 1 (F1) / Low 2 (F2, F3)
> 자동 검증: cargo test 315 passed (cipher off) / clippy clean / cargo check --features cipher 통과 / pnpm lint clean / pnpm tsc clean / pnpm build OK

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint12-retrospective.md` 액션 아이템

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A87 | F2 — `doSaveTemplate` 저장 실패 시 이동 강행 수정 | ✅ 완료 (carry-over stale 확인) | Sprint 12 공지문 작업 완료 시점에 이미 반영되어 있던 항목. T0-a 진입 시 코드 현황 확인으로 stale 판정 |
| A88 | F1 — `save_notice_preview` 경로 경계 검증 | ✅ 완료 | T0-c에서 절대경로 + `.png` 확장자 + traversal 차단 + `data_root()` 외 폴더 자동생성 금지 구현. 단위 테스트 4케이스 추가 |
| A85 | `update_bill_impl` 2쿼리 → 단일 LEFT JOIN (3스프린트 이연) | ✅ 완료 (carry-over stale 확인) | Sprint 12 청구 도메인 작업에서 이미 통합됐던 항목. T0-b 진입 시 stale 판정 |
| A70 | `PaymentsView` dirtyEntries payerName 필터 | ✅ 완료 (carry-over stale 확인) | Sprint 11 post-review 또는 12에서 이미 처리. T0-d 진입 시 stale 판정 |
| A89 | 공지문 페이지 1534줄 분리 검토 | ⏸️ 이연 | 기능 변경 없음. 다음 스프린트 우선순위에 따라 결정 |
| A83 | 생체인증 병행/대체 정책 | ⏸️ 이연 | 장기 과제 유지 |

---

## 잘한 점

**T0 carry-over stale 3건을 조기에 판별하여 낭비 없이 처리했다.**

Sprint 13 T0(carry-over 해소)에서 A87/A85/A70 세 항목이 실제 코드베이스에서 이미 해소된 상태임을 구현 진입 전 코드 현황 확인으로 발견했다. 덕분에 불필요한 중복 수정 없이 핵심 기능(ADR-008)에 집중할 수 있었다. 다만 이 발견은 계획 시 코드 확인 부재로 인한 것이었으므로, 계획 단계에서의 현황 확인 강화가 필요하다.

**`run_startup` 공통 추출로 R91(PIN 경로 회귀) 리스크를 구조적으로 차단했다.**

`app_startup_sequence`와 `auto_unlock_with_keychain` 두 IPC가 단일 `run_startup(auth: AuthStep)` 내부 함수를 공유함으로써, 락 획득 + 무결성 체크 + DB 초기화 + audit cleanup + heartbeat/backup spawn + 소멸 전이 로직이 분기별로 이원화되지 않는다. 기존 315개 테스트 전수 통과로 PIN 경로 회귀 없음이 검증됐다.

**R88 경로 검증이 단위 테스트로 명시적으로 검증된다.**

`save_notice_preview`에 대한 4가지 경계 케이스(비절대경로, 비PNG, traversal, data_root 하위 정상)가 `save_preview_validates_path` 단위 테스트로 보장된다. 단순히 조건 추가에 그치지 않고 테스트로 의도를 문서화한 점이 좋다.

**글로벌 검색 버그 2건이 사용자 검수 중 발견·즉시 수정됐다.**

404 버그(`/students/{id}` → `/students/edit?id=`)와 방향키/IME 처리 누락이 검수 단계에서 발견됐다. 발견 즉시 동일 스프린트 내에서 수정·커밋되어 회귀 없이 마감됐다. 한글 IME `compositionend` + `pendingEnterRef` 패턴이 WebKit 동작 특성에 맞게 올바르게 구현됐다.

**ADR-008이 트레이드오프를 명시적으로 기록한다.**

ADR-008은 기각된 3개 대안(하드코딩, 평문 DB, 생체인증)과 각 기각 사유를 테이블로 비교하고, 채택한 C안의 보안 트레이드오프("데이터 보호를 OS 계정 로그인 + Keychain ACL에 위임")를 명시한다. 사용자(원장)의 요구와 보안 원칙 간 균형을 문서로 증명한다.

---

## 아쉬운 점 / 개선할 점

**계획 시 코드 현황 확인이 부족하여 T0 carry-over 3건이 stale 항목이었다.**

sprint-planner가 이전 회고의 carry-over 항목을 그대로 T0에 배치했으나, 실제 코드베이스에서는 이미 해소된 상태였다. sprint-planner가 계획 수립 시 해당 파일의 현재 코드를 직접 확인하거나, sprint-review 단계에서 이전 회고 carry-over 항목의 실제 이행 여부를 먼저 체크한 후 계획을 수립해야 한다.

**cipher off 빌드에서 ADR-008 구현 메모와 실제 동작이 불일치한다 (F1 / Medium).**

ADR-008 구현 메모는 "cipher feature OFF 빌드에서는 키체인 키 로드 경로가 stub/즉시성공으로 동작해야 함"이라고 기술하지만, 실제 `get_cached_or_load_key`는 cipher off에서도 keyring을 시도한다(단 프론트 개발 모드 예외로 차단). 현재는 안전하나, 향후 코드 변경 시 혼란을 방지하기 위해 주석 정합이 필요하다.

**문서 마이그레이션 현황이 실제와 불일치했다.**

sprint-close가 sprint13.md의 "T7-c 마이그레이션 self-check: V111이 최신 유지"를 그대로 기록했으나, 실제 최신은 V302다. CLAUDE.md의 마이그레이션 현황 표기도 V201까지만 기재되어 있었다. sprint-review에서 CLAUDE.md를 V302 기준으로 수정했으나, 향후 스프린트에서 마이그레이션 파일 추가 시 CLAUDE.md 마이그레이션 현황도 함께 갱신하는 절차가 필요하다.

**`/lock` 초기 렌더 시 SplashScreen이 두 번 순차 표시된다 (F2 / Low).**

"잠금 상태 확인 중" → "자동 로그인 중" 두 메시지가 순차 표시되며 화면이 미묘하게 깜빡인다. 기능적 문제는 없으나 50대 사용자에게 로딩 상태 변화가 불필요한 혼란을 줄 수 있다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A90 | 계획 수립 시 carry-over 항목 코드 현황 먼저 확인 — sprint-planner가 carry-over 파일을 직접 읽고 이행 여부 검증 후 T0 배치 결정 | High | 프로세스 (sprint-planner) | Sprint 14 계획 시 적용 |
| A91 | `startup.rs AuthStep::Keychain` 분기에 cipher off 동작 명시 주석 + ADR-008 구현 메모 수정 (stub/즉시성공 → 실제 동작 설명으로 수정) | Medium | `src-tauri/src/startup.rs`, `docs/arch/adr-008-optional-pin-gate.md` | Sprint 14 T0 carry-over |
| A92 | CLAUDE.md + ROADMAP.md 마이그레이션 현황 표기 갱신 절차화 — 마이그레이션 파일 추가 시 CLAUDE.md 마이그레이션 현황도 함께 갱신 (scope.md에 명시) | Medium | 프로세스 (sprint-dev scope.md 체크리스트) | Sprint 14 scope.md 작성 시 포함 |
| A93 | `/lock` SplashScreen 이중 표시 개선 — 두 로딩 상태를 단일 `isLoading` 상태로 통합 | Low | `src/app/lock/page.tsx` | Sprint 14 또는 기회 발생 시 |
| A89 | 공지문 페이지(`/notices`) 분리 검토 — 1534줄 단일 컴포넌트를 캔버스/편집/저장 섹션으로 분리 | Low | `src/app/notices/page.tsx` | 다음 스프린트 우선순위 검토 시 결정 |
