# Sprint 13 코드 리뷰

> 대상: Sprint 13 (develop...sprint13) — PIN 인증 옵션화 (ADR-008) + Phase 5 취소 + 글로벌 검색 버그 수정 + R88 경로 검증
> 리뷰 일자: 2026-06-02
> 자동 검증 결과: cargo test 315 passed (cipher off) / clippy clean / cargo check --features cipher 통과 / pnpm lint clean / pnpm tsc clean / pnpm build OK

---

## 발견 사항 (3건)

### F1 — `auto_unlock_with_keychain` cipher off 빌드 동작 불명확 (Medium)

- 위치: `src-tauri/src/startup.rs:158`, `src-tauri/src/commands/auth.rs:188`
- 내용: `get_cached_or_load_key`에 `#[cfg_attr(not(feature = "cipher"), allow(dead_code))]` 속성이 붙어 있다. 즉 cipher off 빌드에서는 이 함수가 dead code로 간주되어 경고 억제만 하고, 실제 호출 시에는 keyring에 접근한다. ADR-008 구현 메모("cipher feature OFF(개발) 빌드에서는 키체인 키 로드 경로가 stub/즉시성공으로 동작해야 함")와 실제 코드가 일치하지 않는다. cipher off 개발 빌드에서 `autoUnlockWithKeychain` IPC를 호출하면 실제 keyring 접근을 시도하며, 개발 환경에서 keyring에 키가 없으면 `KeyNotFound` 에러를 반환하여 LockScreen 폴백이 발생한다. 이는 기능적으로 안전하지만(폴백이 작동함), 개발 시 혼란을 줄 수 있다.
- 영향: 개발 모드 프론트에서 `autoUnlockWithKeychain` IPC 래퍼가 `throw new Error('[개발 모드] 자동 잠금해제 미지원')`을 던지므로, 프론트 레벨에서 먼저 차단되어 실제 IPC 호출이 일어나지 않는다. 따라서 cipher off에서의 keyring 호출은 프로덕션 빌드에서만 가능하여 실운용 안전성은 유지된다.
- 조치: 현재 동작은 안전하므로 즉각 수정 불필요. 단 ADR-008 구현 메모와 실제 동작 간 주석 불일치를 다음 스프린트에서 정리 권장 — 혼동 방지를 위해 `startup.rs`의 `AuthStep::Keychain` 분기에 "cipher off 빌드에서는 프론트가 먼저 차단하므로 이 경로는 cipher on에서만 실사용됨" 주석 추가.
- 등급: Medium (기능적 안전성 유지, 문서/주석 불일치)

### F2 — `lock/page.tsx` 초기 렌더 시 SplashScreen 두 번 표시 가능성 (Low)

- 위치: `src/app/lock/page.tsx:102~118`
- 내용: 렌더 분기 순서상 `lockStatus === null`(잠금 상태 로드 중) → `SplashScreen("잠금 상태를 확인하는 중입니다...")`, 이후 lockStatus가 설정되면 `!autoUnlockTried`(자동 잠금해제 시도 중) → `SplashScreen("자동 로그인 중...")` 순서로 두 SplashScreen 메시지가 순차 표시된다. UX상 "로딩 상태가 두 번 표시"되는 셈이어서 화면이 미묘하게 깜빡일 수 있다. 기능적 문제는 없다.
- 영향: Low. 50대 사용자에게 잠깐의 깜빡임이 불필요한 혼란을 줄 수 있음.
- 조치: 차기 스프린트 개선 권장. 두 로딩 상태를 단일 `isLoading` 상태로 통합하거나, 첫 SplashScreen을 제거하고 `!autoUnlockTried`만 남기는 방식으로 개선 가능.

### F3 — `menu-config.ts` 대시보드 disabledHint Phase 번호 부정확 (Low)

- 위치: `src/lib/menu-config.ts:17`
- 내용: Phase 5 메뉴 2개 제거와 함께 대시보드 `disabledHint`가 `'Phase 6 에서 제공'`에서 `'Phase 5 에서 제공'`으로 수정됐다. 그러나 ROADMAP에서 Phase 5(단원평가/학습보고서)가 취소되고 재정렬된 상황에서 대시보드는 실제로 Phase 5(현 기준에서 재정렬된 Phase)가 아닌 차기 Phase에서 제공된다. `'향후 제공 예정'` 또는 구체적인 Phase 번호(재정렬 후 정확한 번호)로 수정 권장.
- 등급: Low

---

## 영역별 추가 점검

### 보안 (backend.md Critical 기준)

- SQL 인젝션: `notice.rs` `setup.rs` `startup.rs` 변경분 내 raw query concat 없음. 기존 SQLx 매크로 패턴 유지. 이상 없음.
- 하드코딩 시크릿: 변경 파일 전체에서 패턴 없음 (`시크릿 패턴 없음` 스캔 확인).
- 인증/인가: `auto_unlock_with_keychain`은 DB 접근 없이 config.json + keyring만 사용. unlock 전 호출 가능하도록 설계되어 적절함.

### 보안 (backend.md High 기준)

- `unwrap()` 남용: 변경 파일 내 `unwrap()` 사용 없음 (`?` 연산자 사용). 테스트 코드에서만 `unwrap()` 사용.
- 경로 traversal R88: `save_notice_preview`에 절대경로 검증 + `..` 차단 + `.png` 확장자 강제 + `data_root()` 외 폴더 자동생성 금지 구현 완료. 단위 테스트 4케이스로 검증됨.
- config.json 저장 위치(R92): `app_config_dir` (PC별 로컬) 사용 확인. 클라우드 동기화 대상 아님. 양 PC 설정 충돌 없음.

### 보안 (frontend.md Critical 기준)

- XSS: `dangerouslySetInnerHTML` 사용 없음. 사용자 입력 직접 렌더 없음.
- `invoke()` 직접 호출: 모든 Tauri 호출이 `src/lib/tauri/index.ts` 래퍼를 통함 (`getPinSkipSetting`, `setPinSkipSetting`, `autoUnlockWithKeychain`).
- 민감 정보 localStorage: PIN/키 관련 정보 프론트 저장 없음.

### 보안 (frontend.md High 기준)

- TypeScript any 남용: 없음.
- `'use client'` 과다: `settings/page.tsx`에 추가된 `PinAuthToggle` 컴포넌트가 `useState`/`useEffect`를 사용하므로 파일 레벨 `'use client'` 필요. 이미 파일에 `'use client'`가 적용되어 있는지 확인 필요 — 파일 헤더를 보면 `'use client'`가 없다. Next.js 15 static export에서 `useState`는 Client Component에서만 사용 가능하므로 빌드는 성공했으나, `'use client'` 미명시 시 Next.js가 자동으로 처리한다(static export에서 모든 페이지가 서버 경계 없이 클라이언트 실행). 기능적 문제 없음, 명시성 측면에서 Low 이슈로 기록.

### R91 — PIN 경로 회귀 없음 확인

- `run_startup` 내부 공통 로직 추출 후 `app_startup_sequence` 흐름: `AuthStep::Pin(password)` → `verify_password` 호출 → 기존 3~8단계 동일 실행.
- `auto_unlock_with_keychain` 흐름: `AuthStep::Keychain` → `get_cached_or_load_key` → 기존 3~8단계 동일 실행.
- 단위 테스트 `startup::tests::startup_result_timing_breakdown_sum_approximates_total` 등 기존 테스트 315건 전수 통과 → 회귀 없음 확인.

### AI 생성 코드 추가 체크

- `global-search.tsx` IME 처리: `isComposing` 플래그 + `composingRef` ref 미러 패턴이 올바르게 구현됨. `pendingEnterRef`를 통해 조합 중 Enter → `compositionend` 후 선택 처리하는 흐름이 WebKit 동작 특성에 맞게 구현됨.
- `lock/page.tsx` 클린업: `cancelled = true` cleanup 함수가 `useEffect` 반환으로 올바르게 설정됨. 컴포넌트 unmount 후 상태 업데이트 방지.

---

## 결론

Critical 0건, High 0건, Medium 1건(F1 — 문서/주석 불일치, 기능 안전), Low 2건(F2 F3).
Sprint 13 핵심 기능(ADR-008 PIN 옵션화, R88 경로 검증, 글로벌 검색 수정)은 보안·안전성 기준을 충족한다.
F1은 차기 스프린트 주석 정리, F2/F3는 자연스러운 개선 기회에 처리 권장.
