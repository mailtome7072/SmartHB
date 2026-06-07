# Sprint 15 코드 리뷰

> 대상: Sprint 15 (develop...sprint15) — 교습소 정보 화면 + 자가 진단 이력 삭제 + 전역 단축키/툴팁 + 접근성 대비 개선 + 청구 N+1 최적화 + monthly_summary 리팩토링
> 리뷰 일자: 2026-06-07
> 자동 검증 결과: cargo test 375 passed (cipher off) / clippy --all-targets clean / cargo check --features cipher 통과 / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 발견 사항 (3건)

### F1 — GlobalTooltip: 빠른 mouseover 시 title 미복원 엣지 케이스 (Low, 기록만)

- 위치: `src/components/layout/GlobalTooltip.tsx:59-68` (`onOver` 핸들러)
- 시나리오: 사용자가 `[title]` 요소에서 다른 `[title]` 요소로 매우 빠르게 마우스를 이동할 때, `onOut` 이 발화되지 않고 `onOver` 만 연속 발화되는 경우 `restore()` 가 이전 앵커의 `title` 을 올바르게 복원하는지 여부. 현재 코드는 `onOver` 진입 시 `restore()` 를 먼저 호출하므로 정상 복원된다. 단, 타 컴포넌트가 `active` 엘리먼트를 DOM 에서 제거(언마운트)한 직후 `restore` 가 호출되면 `removeAttribute('data-shb-title')` 는 이미 제거된 노드에 도달하지 못하나, DOM 의 특성상 부작용 없음. 실제 결함은 아니나 스크린리더 환경에서 `title` 이 일시적으로 제거되는 구간이 존재한다.
- 조치: Low — 이번 수정 없이 기록만. 스크린리더 영향이 실제 이슈가 되면 `aria-label` 우선 활용 권고.

### F2 — GlobalShortcuts: Ctrl+N 입력 중 단축키 충돌 미방어 (Low, 기록만)

- 위치: `src/components/layout/GlobalShortcuts.tsx:33-35`
- 시나리오: `<input>` 또는 `<textarea>` 에 포커스가 있을 때 Ctrl+N 을 누르면 `/students/new` 로 이동한다. Ctrl+F 는 `<input>` 포커스를 이동하는 효과라 부작용이 없으나, Ctrl+N 은 입력 중 원하지 않는 화면 전환을 일으킬 수 있다.
- 실패 시나리오: 원생 이름 입력 칸에서 Ctrl+N(한글 자모 입력 관련 OS 입력기 단축키와 무관하게 영문 키 기준)을 누르면 저장 없이 신규 원생 화면으로 전환됨.
- 조치: Low — 허용 가능 범위. `미저장 변경 경고 다이얼로그`(PRD §5.7)가 구현되면 자연스럽게 보호된다. Sprint 16에서 Ctrl+S/F1 구현 시 함께 방어 로직 추가 권고: `if ((e.target as HTMLElement).tagName === 'INPUT' || ...) return`.

### F3 — T1 이미지 업로드: 미저장 이탈 경고 누락 (Medium, ROADMAP 이연 기록)

- 위치: `src/app/settings/info/page.tsx` — 이미지 업로드/삭제 직후 텍스트 변경 미저장 상태에서 화면 이탈 경고 없음.
- 시나리오: 이미지를 업로드(즉시 파일 저장)한 뒤 텍스트 필드를 수정하고 저장 버튼을 누르지 않은 채 다른 메뉴로 이동하면, 이미지는 저장됐으나 텍스트 변경분이 소실됨.
- 영향: 이미지(즉시 저장)와 텍스트(저장 버튼 필요)의 저장 시점 불일치에서 오는 사용자 혼동. 실 데이터 손실 위험 낮음(재입력 가능).
- 조치: Medium — `미저장 변경 경고 다이얼로그`(PRD §5.7)는 전체 화면에 공통 구현이 예정됨. 이번 스프린트 수정 없이 리스크 등록, Sprint 16 공통 미저장 경고 구현 시 포함.

---

## 영역별 추가 점검

### 보안 (backend.md Critical)
- SQL 인젝션: `dashboard.rs`, `diagnosis.rs`, `settings.rs`, `billing.rs` 모두 `sqlx::query!` 매크로 또는 `bind()` 파라미터 바인딩 사용. raw string concat 없음. 이상 없음.
- 하드코딩 시크릿: 변경 파일 전체 스캔 결과 없음.
- 인증/인가 누락: 교습소 정보 IPC(`get_academy_info`, `save_academy_info`)는 앱 설정 데이터로 인증 레이어 뒤에서만 호출되는 구조. 이상 없음.

### 보안 (backend.md High)
- N+1 쿼리: T6에서 `billing.rs` `standard_fees` N+1 제거 완료(HashMap 단일 배치). `overview()` 분기별 쿼리 4회는 고정 횟수로 허용 가능.
- 예외 처리: `unwrap()`/`expect()` 프로덕션 경로에서 미사용 확인. `expect("분기 시작일")` 등은 상수 계산으로 panic 불가 패턴.

### 프론트엔드 (frontend.md Critical/High)
- XSS: `dangerouslySetInnerHTML` 미사용 확인. 이미지 미리보기는 `bytesToDataUrl()` 에서 생성한 data URL 을 `<img src>` 에 사용 — 사용자 입력 직접 렌더링 아님. 이상 없음.
- `invoke()` 직접 호출: 신규 컴포넌트(`GlobalTooltip`, `GlobalShortcuts`) 모두 Tauri IPC 호출 없음. `info/page.tsx` 는 `lib/tauri/index.ts` 래퍼 경유 확인.
- TypeScript `any` 남용: 없음. 에러 캐치에서 `e: unknown` + instanceof 패턴 준수.
- `img` 태그 직접 사용: `info/page.tsx:345` eslint-disable 주석(`@next/next/no-img-element`) — data URL 미리보기라 `next/image` 사용 불가(unoptimized + base64 혼용 문제), 올바른 예외 처리.

### AI 생성 코드 추가 체크
- `monthly_summary` 리팩토링: 기존 동작(1:1 스키마)과의 동치성이 단위 테스트 2건(`monthly_summary_totals_billing_and_paid`, `monthly_summary_unpaid_payment_row_excluded_and_no_fanout`)으로 보장됨. R99 해소 확인.
- `delete_diagnosis_history` / `clear_diagnosis_history`: 멱등성(존재하지 않는 id 삭제 시 정상)이 단위 테스트 3건으로 보장됨.
- `GlobalTooltip`: `useEffect` cleanup 함수에서 이벤트 리스너 3종 모두 제거 + `restore()` 호출로 마운트 해제 시 부작용 없음.

---

## 결론

Critical 0건, High 0건, Medium 1건(F3 — 미저장 이탈 경고, 이연), Low 2건(F1/F2 — 기록만). 코드 품질 양호. Sprint 16 이전에 처리할 필수 수정 없음.
