# Sprint 12 코드 리뷰

> 대상: Sprint 12 (develop...sprint12) — 공지문 이미지 생성 + PIN UI 통일 + 복구 코드 제거 + 메뉴 정비
> 리뷰 일자: 2026-06-02
> 자동 검증 결과: cargo test 312 passed (cipher off) / clippy clean / lint clean / tsc clean / build OK

## 발견 사항 (4건)

### F1 — `save_notice_preview`: 임의 경로 파일 쓰기 (Medium, risk-register 등록)

- 위치: `src-tauri/src/commands/notice.rs:677-684`
- 내용: `path` 파라미터를 그대로 `PathBuf::from(&path)`로 변환하여 임의 절대경로에 파일 쓰기. `sanitize_component`/`sanitize_path_part` 적용 없음.
- 맥락 분석: 호출 경로가 `showSaveDialog`(Tauri Dialog `save`) → 사용자가 선택한 경로 → `saveNoticePreview(chosen, ...)` 단일 흐름으로 한정됨. Tauri Dialog는 OS 수준 파일 저장 다이얼로그이므로 실질적 임의 경로 공격 가능성은 낮음. 단, IPC를 직접 호출하는 우회 시나리오에서는 무제한 경로 쓰기가 가능함.
- 조치: 위험도 낮음(로컬 단독 앱, 인터넷 노출 없음)으로 risk-register 등록 후 ROADMAP 이연. 개선 시 `data_root()` 경계 검증 또는 `canonicalize` 후 접두 확인 적용 권장.

### F2 — 미저장 저장 후 이동 시 저장 실패해도 이동 강행 (Medium, risk-register 등록)

- 위치: `src/app/notices/page.tsx:1485`
- 내용: "네(저장 후 이동)" 버튼 핸들러에서 `await doSaveTemplate(saveName)` 후 `runPendingAction(action)` 을 호출. `doSaveTemplate`는 내부에서 예외를 catch하여 `setError()`로 처리하고 re-throw하지 않음. 결과적으로 저장 실패 시에도 `runPendingAction`이 실행되어 navigate kind면 페이지 이동이 강행됨.
- 실패 시나리오: 사용자가 템플릿 편집 후 메뉴 이동 시도 → "저장하시겠습니까?" 모달에서 "네" 클릭 → `saveNoticeLayoutNamed` IPC 실패(DB 오류 등) → 에러 토스트 표시되나 화면은 이미 다른 페이지로 이동 → 변경사항 유실.
- 조치: risk-register 등록. 수정 시 `doSaveTemplate`가 예외를 re-throw하도록 변경하거나, 호출부에서 try-catch로 성공 여부 확인 후 조건부 navigate.

### F3 — `notices/page.tsx` 인라인 `style` 다수 사용 (Low, 기록만)

- 위치: `src/app/notices/page.tsx` 여러 곳 (약 15+개 `style={{}}`)
- 내용: Tailwind 우선 정책에서 벗어나 인라인 style prop이 광범위하게 사용됨.
- 맥락 분석: 공지문 편집 캔버스 특성상 동적 계산값(`scale`, `bgDims.w`, `transform: scale(...)`, 글자별 `color`)이 필수라 Tailwind arbitrary value로 대체 불가한 케이스가 대부분. 특히 `react-rnd`의 좌표·크기, Canvas 오버레이 절대 배치는 정적 클래스로 표현 불가.
- 조치: Low — 허용 예외 케이스로 기록. 단, TEMPLATE_PANEL_WIDTH(`style={{ width: TEMPLATE_PANEL_WIDTH }}`)처럼 상수 값은 Tailwind arbitrary value(`w-[280px]` 등)로 전환 가능.

### F4 — `sanitize_path_part`: 단일 점(`.`) 파일명 제거 후 빈 문자열 가능성 (Low, 기록만)

- 위치: `src-tauri/src/commands/notice.rs:568-584`
- 내용: `.trim_matches('.')` 적용 시 `".png"` → `""` 케이스에서 "unnamed" 폴백으로 처리되나, 실제 도달 경로는 `sanitize_path_part(notice_name)`이고 공지문 이름은 빈 문자열 입력 차단(`name.trim().is_empty()` 검증)이 프론트와 백엔드 양쪽에 있어 실질적 위험 없음.
- 조치: Low — 기록만.

## 영역별 추가 점검

### 보안 (backend.md Critical)
- SQL 인젝션: `sqlx::query()` + `.bind()` 파라미터 바인딩만 사용. raw concat 없음. ✅
- 하드코딩 시크릿: 없음 (시크릿 패턴 스캔 통과). ✅
- 인증/인가 누락: `save_notice_preview` 임의 경로(F1 기록). 나머지 커맨드는 data_root 기반 경로 구성으로 자연 격리됨. ✅

### 보안 (backend.md High)
- `unwrap()` 남용: 프로덕션 코드에서 없음 (테스트 코드 내 `.expect()` 정상). ✅
- 페이지네이션 누락: notice.rs는 배경서식 목록 전체 반환 — 배경서식 파일이 수십 건 수준으로 페이지네이션 불필요(PRD 범위 내). ✅
- `.sqlx/` 캐시: notice.rs 쿼리가 `query!` 매크로가 아닌 `sqlx::query()`(런타임) 사용 — 오프라인 캐시 갱신 불필요. ✅

### 프론트엔드 (frontend.md Critical/High)
- XSS (`dangerouslySetInnerHTML`): 없음. ✅
- `invoke()` 직접 호출: 없음. `@/lib/tauri` 래퍼만 사용. ✅
- TypeScript `any` 남용: 없음 (tsc clean). ✅
- SSR 가드(`typeof window`): `notices/page.tsx`에서 canvas 생성 전 `typeof window === 'undefined'` 가드 적용됨. `notice-generator.ts`의 `renderNoticeDataUrl`도 동일 가드 내재. ✅
- 글로벌 검색바: 기존 `AppShell`을 사용하므로 누락 없음. ✅
- 접근성(44×44px): PIN 필드 박스 `h-[76px] w-[52px]` — 기준 충족. ✅

### AI 생성 코드 추가 체크
- `sanitize_component` 경로 traversal 차단: `..` → `_` 치환 + `trim_matches('.')` 적용. 단위 테스트로 검증됨. ✅
- `notice-generator.ts` `buildColorRuns`: UTF-16 코드유닛 기준 인덱스 처리 — 한글 1글자 = 1코드유닛으로 정합. ✅
- `guardedNavigate` / `unsavedGuard`: Zustand 스토어에 함수 저장 후 사이드바 `Link onClick`에서 검사. 가드가 `false` 반환 시 `e.preventDefault()` 로 이동 차단 후 화면에 위임. 설계 의도대로 구현됨. ✅
- NFC 정규화: `saveNoticeImage`, `saveNoticeImagesBatch`, `saveNoticePreview`, `openNoticeOutputDir` 모두 IPC 호출 직전 `nfc()` 래퍼 적용. macOS APFS 파일명 쌍자모 문제 차단. ✅

## 결론

Critical 0건 / High 0건 / Medium 1건(F1) / Low 2건(F2, F3).

공지문 도메인 전반적으로 경로 traversal 방어, SQLx 바인딩, IPC 추상화, NFC 정규화, SSR 가드가 모두 적절히 적용됨. `save_notice_preview`의 임의 경로 수용은 Tauri Dialog 신뢰 경계 내에서 로컬 단독 앱 맥락으로 수용 가능 수준이나 장기적 개선 권고. 인라인 style은 Canvas 오버레이 특성상 불가피한 예외로 처리.
