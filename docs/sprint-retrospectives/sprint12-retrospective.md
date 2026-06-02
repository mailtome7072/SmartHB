# Sprint Retrospective sprint12

> 대상: Sprint 12 (develop...sprint12) — 공지문 이미지 생성 + PIN UI 통일 + 복구 코드 제거 + 메뉴 정비
> 리뷰 일자: 2026-06-02
> 코드 리뷰: Critical 0 / High 0 / Medium 2 (F1, F2) / Low 2 (F3, F4)
> 자동 검증: cargo test 312 passed (cipher off) / clippy clean / lint clean / tsc clean / build OK

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint11-post-retro2.md` 액션 아이템 + `docs/sprint/sprint12.md` T0 carry-over

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A82 | `verify_password`에 `validate_pin` 미적용 의도 주석 | ✅ 완료 | `auth.rs:657` — ADR-007 의도 명시 주석 추가됨 |
| A83 | 생체인증 병행/대체 정책 | ⏸️ 미이행 | Phase 5 이후 장기 과제. 유지 |
| A85 | `update_bill_impl` 2쿼리 → 단일 LEFT JOIN | ⏸️ 미이행 | 공지문 범위 외. Sprint 13 이연 |
| A86 | `BillingSummaryView` 년/월 토글 radio 전환 | ⏸️ 미이행 | 사용자 "체크박스 선호" 의사 반영, 개선 불필요로 최종 처리. 재등록 없음 |
| A70 | `PaymentsView` dirtyEntries payerName 필터 | ⏸️ 미이행 | 공지문 우선 진행으로 이연 |
| A73 | `billing.rs` seed_student `.bind(withdraw)` 전환 | ⏸️ 미이행 | Low, 이연 |

---

## 잘한 점

**macOS WKWebView html-to-image 결함을 Canvas 2D 직접 렌더로 정확히 회피했다.**

Sprint 계획 시 html-to-image + toPng()를 기본 방향으로 설정했으나, 실구현 과정에서 macOS WKWebView의 SVG foreignObject + data URL `<img>` 렌더링 결함으로 빈 PNG가 생성되는 문제가 발견됐다. html-to-image 의존성을 제거하고 Canvas 2D 직접 드로잉 방식(`notice-generator.ts`)으로 전환한 것이 Windows/macOS 공통 안정성을 보장한다. 이 전환은 배경서식 → Canvas → 텍스트박스 런(buildColorRuns) 순서의 명확한 파이프라인으로 정착했고, 미리보기와 생성 로직이 동일한 `renderNoticeDataUrl` 코어를 공유하여 WYSIWYG 보장도 개선됐다.

**경로 traversal 방어가 단위 테스트로 증명된다.**

`sanitize_component` / `sanitize_path_part` 두 함수가 파일명/원생명 정규화를 전담하며, `../etc/passwd`, 경로 구분자, 제어문자를 각각 차단함을 `notice::sanitize_blocks_traversal_and_separators` 테스트로 검증한다. IPC에 도달하는 모든 파일명 파라미터가 이 함수를 통과하도록 설계된 점이 좋다.

**한글 파일명 NFC 정규화를 IPC 래퍼 단일 지점에 집중시켰다.**

macOS APFS는 파일명을 NFD 형태로 저장하여 Windows/macOS 간 파일명 불일치가 발생할 수 있다. `src/lib/tauri/index.ts`의 `nfc()` 래퍼를 `saveNoticeImage`, `saveNoticeImagesBatch`, `saveNoticePreview`, `openNoticeOutputDir` 4개 IPC에 일괄 적용하여, 호출 측에서 별도 처리 없이 일관된 NFC 파일명이 보장된다.

**미저장 변경 네비게이션 가드가 전역 스토어 패턴으로 깔끔하게 구현됐다.**

`useAppStore`의 `unsavedGuard` 슬롯에 화면별 가드 함수를 등록/해제하는 방식은 사이드바와 글로벌 검색 양쪽에서 동일한 가드를 재사용한다. 공지문 화면 unmount 시 `setUnsavedGuard(null)` 해제도 cleanup으로 구현되어 메모리 누수 없음.

**복구 코드 시스템 제거가 코드베이스를 실제 정책과 일치시켰다.**

PRAGMA rekey 미구현 환경에서 복구 코드가 보안 보호 효과 없이 복잡성만 추가한다는 판단을 반영했다. `argon2` crate 의존 제거, `RecoveryCodeIssued` audit variant 제거, 관련 UI(RecoveryCodeDisplay, RecoveryCodeInput의 이전 사용처) 정리로 auth 도메인이 단순화됐다. `change_pin` IPC는 `set_password`와 동일한 atomic 패턴(keyring → salt → cache, salt 실패 시 keyring rollback)을 재사용하여 일관성을 유지한다.

**PIN 6박스 공용 컴포넌트가 접근성 기준을 충족한다.**

`pin-field.tsx`의 각 박스는 `h-[76px] w-[52px]` (76×52px — 44×44px 최소 기준 초과), `aria-label` 개별 제공, Backspace/방향키/붙여넣기 처리, `inputMode="numeric"` 적용으로 접근성과 사용성을 모두 확보한다. LockScreen과 `/settings/pin` 두 곳에서 동일 컴포넌트를 재사용하여 UI 일관성이 보장된다.

---

## 아쉬운 점 / 개선할 점

**미저장 확인 모달의 "네(저장 후 이동)" 경로에서 저장 실패 시에도 이동이 강행된다 (F2).**

`doSaveTemplate`가 try-catch로 예외를 내부 처리하고 re-throw하지 않아, 호출부에서 저장 성공/실패를 판별할 수 없다. 저장 실패 시 에러 토스트가 표시되지만 `runPendingAction`이 이어서 실행되어 navigate kind라면 화면이 이동된다. 사용자 의도("저장 후 이동")와 실제 동작("저장 실패 후 이동")이 다른 데이터 유실 시나리오다.

**`save_notice_preview`가 임의 절대 경로를 수용한다 (F1).**

Tauri Dialog에서 반환한 경로만 실제로 전달되므로 현실적 위험은 낮지만, IPC 직접 우회 시 `data_root()` 경계 바깥에 파일을 쓸 수 있다. 단일 사용자 로컬 앱이라 공격 가능성은 거의 없으나, 다른 IPC들이 `sanitize_path_part` + `notice_output_dir()` 조합으로 경계를 제한하는 것과 일관성이 없다.

**T0 carry-over 항목(A85 등)이 Sprint 12에서도 이연됐다.**

`update_bill_impl`의 2쿼리 비효율(A85)은 3개 회고에 걸쳐 이연된 상태다. 공지문이 Sprint 12의 주요 작업이었으므로 이연 자체는 합리적이지만, 다음 스프린트에서 반드시 처리해야 할 기술 부채다.

**공지문 페이지가 1534줄의 단일 컴포넌트로 커졌다.**

복잡한 캔버스 편집 화면 특성상 불가피한 면이 있으나, 미리보기 캔버스 섹션, 텍스트박스 편집 컨트롤, 저장 패널, 미저장 확인 로직 등이 혼재하여 가독성이 낮다. 기능 분리 없이 더 커지면 유지보수 비용이 증가한다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A87 | F2 — `doSaveTemplate`가 예외를 catch 후 re-throw하도록 수정, 또는 호출부에서 try-catch 후 저장 성공 시에만 navigate 실행 | Medium | `src/app/notices/page.tsx:1485` 부근 | Sprint 13 T0 carry-over |
| A88 | F1 — `save_notice_preview` 경로를 `output_root()` 하위로 제한하거나 `canonicalize` 후 접두 검증 추가 | Low | `src-tauri/src/commands/notice.rs:677` | Sprint 13 이후 (기준 미정 — 다음 계획 시 구체화) |
| A85 | `update_bill_impl` status + is_paid 단일 LEFT JOIN 통합 — 3스프린트 이연된 2줄 수정 | Low | `billing.rs:287~293` | Sprint 13 T0 carry-over (반드시 처리) |
| A89 | 공지문 페이지 분리 검토 — 캔버스 편집 섹션(`NoticeCanvas`), 텍스트박스 컨트롤(`TextboxControls`), 저장 패널(`TemplatePanel`)을 별도 컴포넌트로 추출. 1534줄 → 300~500줄 목표 | Low | `src/app/notices/page.tsx` | (기준 미정 — Sprint 13 이후 여유 시 진행) |

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 날짜 | 2026-06-02 |
| 커밋 수 | 20건 (주요: feat/fix/refactor/chore + docs) |
| 변경 파일 | 50개 (BE: 13개, FE: 23개, docs/config: 14개) |
| 신규 도메인 | `notice.rs` (806줄), `notice-generator.ts` (250줄), `pin-field.tsx` (124줄) |
| DB 마이그레이션 | 0건 (공지문은 파일시스템 + app_settings JSON) |
| 신규 단위 테스트 | 6건 (notice 4건, paths 2건) |
| 누적 단위 테스트 (cipher off) | 312 passed (이전과 동일 — 신규 6건, 회고 코드 변화 없음) |
| 자동 검증 | clippy/lint/tsc/build 4종 통과, cargo test 312 passed |
| sprint-review 결함 | 4건 (F1/F2 Medium, F3/F4 Low) — Critical/High 0건 |

---

## 종합 평가

Phase 4 마지막 마일스톤(M5: 청구 완성)이자 PRD §4.10 공지문 기능의 핵심 구현이 완료됐다. macOS WKWebView html-to-image 결함 발견 및 Canvas 2D 대체 전환이 Sprint 중 이루어진 것이 가장 큰 특이사항으로, 이 결함은 실제 Windows/macOS 동시 지원 환경에서 치명적이었을 것이다. 조기 발견 및 대응이 적절했다.

복구 코드 제거, PIN 6박스 통일, 메뉴 정비, 미저장 가드 등 Sprint 12 범위 외 작업 6건이 추가됐다. scope.md에 일괄 기록하여 추적성을 유지한 것은 좋으나, 범위 외 작업이 많아질수록 코드 리뷰와 회고 커버리지가 분산된다.

Critical/High 결함 0건으로 프로덕션 배포 차단 요인 없음. Medium 2건(F1, F2)은 R88/R89로 risk-register에 등록됐고, 특히 F2(저장 실패 후 이동 강행)는 Sprint 13 T0에서 반드시 처리해야 한다.

**Phase 4 완료 — 다음 단계: Phase 5 취소 결정 반영 후 Phase 6(대시보드·유틸리티) 계획 수립.**
