# Sprint Retrospective — Sprint 3

> Sprint 3: 원생 관리 프론트 + 초기 설정 마법사 + 글로벌 검색 + 접근성 기반 (Phase 1 완료)
> 기간: 2026-05-21 ~ 2026-05-21 (5 세션으로 분할 진행)
> 브랜치: `sprint3` → develop 머지 (47 files, +3256/-184)
> 코드 리뷰: Critical 0 / High 2(기지 이연 항목) / Medium 3 / Low 2

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint2-retrospective.md`

| 항목 | 이행 여부 | 비고 |
|------|-----------|------|
| A1 — audit details PII 마스킹 | ✅ 완료 | T2에서 `try_record` 3곳 `details=None` 적용 (R13 해소) |
| A2 — exit_hook 락 해제 교체 | ✅ 완료 | Sprint 2 종료 직후 hotfix로 처리됨 (6c85f5c) |
| A3 — list_students/list_codes 페이지네이션 | ✅ 완료 | T3에서 `pagination.rs` 모듈 신설 + LIMIT/OFFSET 적용 (R14 해소) |
| A4 — sprint-planner 사전 검토 강화 | ✅ 반영 | sprint-planner MEMORY.md에 data-model SSOT 대조 항목 추가 |
| A5 — 마이그레이션 파일명 V{NNN} 통일 | ⏸️ 보류 | 사용자와 협의 없이 실행되지 않음. 현행 표기(`001__`, `101__`)로 유지 중 |
| A6 — cipher on 환경 실측 | ⏸️ 보류 | v0.2.0 인스톨러 미배포. 다음 배포 시 측정 예정 |

---

## 잘한 점

**5 세션 분할 진행이 품질과 진행 속도 모두에 기여했다.**
세션별 `scope.md` 마감 메모(T 완료 표기 + 다음 진입점 명시)가 세션 재진입을 거의 마찰 없이 가능하게 했다. 47개 파일, 3,400 라인 이상의 변경이 단일 세션에 집중되지 않아 각 세션의 리뷰·검증이 실질적으로 수행되었다.

**step-back 프로토콜이 자연스럽게 작동했다 (T8 chicken-and-egg).**
T8 초기 설정 마법사 구현 중 `paths::data_root()` 동적화와 마법사 UI 구현을 동시에 진행하면 auth/recovery/backup/integrity 모듈 전체에 변경이 전파된다는 사실을 발견하고 즉시 중단했다. `app_config_dir/config.json` 분리 저장이라는 단순한 설계로 계획을 재정의하여 변경 범위를 `setup.rs` 1개 파일로 한정했다. 결과적으로 전체 sprint 변경량이 통제 가능한 범위 안에서 완료되었다.

**simplify 패턴이 코드 구조 개선에 실질적 기여를 했다.**
`build_filter_clause` / `bind_filter` 헬퍼 분리, `CodeRow`·`FeeRow`의 onBlur 저장 패턴, `GlobalSearch`의 `useMemo` 분해 캐싱이 simplify 사이클에서 도출된 구조다. 특히 `list_students` / `count_students`가 SQL 빌더를 공유하는 설계는 필터-페이지네이션-카운트 삼중 일관성을 보장한다.

**접근성 기반이 한 번에 확립되었다.**
Pretendard self-host(ADR-006), CSS 변수 토큰(`--font-size-body: 18px`, `--line-height-base: 1.5`), 헤더 타이포그래피 스케일(h1~h6), 44×44px 클릭 영역이 모두 Sprint 3에서 한꺼번에 정착했다. 이후 Phase 2+ 화면은 이 토큰을 그대로 상속받아 접근성 일관성을 유지할 수 있다.

**한글 자모 검색을 외부 라이브러리 없이 구현했다.**
`hangul-search.ts`의 자체 분해 구현(CHO/JUNG/JONG 테이블 + 유니코드 오프셋)은 원생 ~100명 + 메뉴 ~10개 규모에서 O(n) 선형 스캔으로 충분하다. 번들 의존성 없이 정확도와 성능 모두를 충족했다.

---

## 개선할 점

**T8 chicken-and-egg 는 sprint-planner 가 사전에 파악했어야 했다.**
`paths::data_root()` 동적화가 DB 초기화 이전에 폴더 경로를 알아야 한다는 제약은 Sprint 2 R12 이연 결정 당시 이미 기록된 내용이다. sprint-planner가 R12 이연 이유를 더 깊이 파악했다면 T8 계획 단계에서 `config.json` 분리 설계를 미리 결정하고 T8 구현 범위를 처음부터 좁게 잡을 수 있었다. step-back 1건은 비용이 크지 않았으나, sprint-planner 사전 설계 역량의 개선 여지를 보여준다.

**일부 simplify 사이클에서 premature 추출 권고가 발생했다.**
여러 세션에서 simplify 에이전트가 당장은 단 1곳에서만 사용되는 헬퍼를 별도 파일로 추출하도록 권고했다. 단일 사용처 함수의 파일 분리는 현 코드 규모에서 오히려 탐색 비용을 높인다. simplify 스킬 호출 전 "사용처가 2곳 이상일 때만 추출 권고" 기준을 적용하도록 조정할 필요가 있다.

**임시저장(localStorage)과 미저장 경고 다이얼로그는 구현되었으나 Undo 스택은 미구현이다.**
`StudentForm`의 3분 자동 임시저장 + 이탈 경고는 PRD §5.7을 충족하지만, 출결 토글·청구 금액 조정·보강 등록에 대한 1단계 Undo는 Medium 이슈로 남아 있다. Phase 2 출결 구현 시 함께 처리가 필요하다.

**`dialog:allow-open` 권한 명세가 최소 권한 원칙에 비해 넓다.**
현재 `capabilities/default.json`에 `dialog:allow-open`이 등록되어 있으나, 실제 필요는 폴더 선택(directory: true)으로 한정된다. `tauri-plugin-dialog` 2.x에서 하위 권한(`dialog:allow-open-file`, `dialog:allow-open-directory`)이 분리되어 있는지 확인하고, 가능하면 디렉토리 선택 권한만 부여하도록 좁히는 것이 바람직하다. 기능 동작에는 영향이 없어 Medium으로 분류한다.

**`window.confirm()` 퇴교 확인 다이얼로그는 임시 구현이다.**
`student-form.tsx`/`edit/page.tsx`에서 퇴교 확인에 `window.confirm()`을 사용하고 있다. Tauri WebView에서 `window.confirm()`은 동작하지만 스타일링이 불가하고 50대 사용자 친화적이지 않다. Phase 2 이후 shadcn/ui Dialog로 교체가 권장된다.

---

## 이연 항목 처리 권고

### R12 + paths::data_root() 동적화

현재 `paths.rs`의 `data_root()`는 `./SmartHB-dev.db`를 하드코딩 반환한다. 프로덕션에서는 마법사에서 선택한 클라우드 폴더 경로를 반영해야 한다. 이 변경은 auth/backup/integrity/lock 모듈 전체에 영향을 미치므로 다음 두 가지 처리 경로 중 하나를 선택해야 한다.

**권고 경로 A (Sprint 4 T0):** Sprint 4 계획 수립 시 `paths.rs` 동적화를 첫 번째 Task로 배치한다. Sprint 4가 출결·청구 등 DB 의존 기능을 구현하기 전에 경로 동적화를 완료하면 이후 모든 모듈이 올바른 경로를 사용하게 된다.

**권고 경로 B (hotfix sweep):** Sprint 4 진입 전 현재 develop 브랜치에서 hotfix로 처리한다. 변경 파일이 `paths.rs`, `startup.rs`, `commands/db.rs` 수준으로 한정된다면 hotfix 요건(3개 파일 이하, 50줄 이하)을 충족할 수 있다.

**어느 쪽이든 Sprint 4 진입 전 완료가 필요하다.** Sprint 4에서 구현될 기능이 `data_root()`를 직접 참조하기 시작하면 동적화 변경의 영향 범위가 급격히 커진다.

### salt.bin 이전 (R12의 나머지 절반)

Keychain에 보관된 salt를 `{cloud}/smarthb/salt.bin`으로 이전하는 작업은 `paths.rs` 동적화 완료 후에야 안전하게 구현할 수 있다. `paths.rs` 동적화 Task와 동일 sprint에서 후속 Task로 포함하는 것이 자연스럽다.

---

## Sprint 4 액션 아이템

| ID | 항목 | 우선도 | 비고 |
|----|------|--------|------|
| A7 | `paths::data_root()` 동적화 → config.json의 cloud_folder_path 반영 | 최우선 | Sprint 4 T0 또는 hotfix sweep |
| A8 | salt.bin 이전 (Keychain → cloud/smarthb/salt.bin) | 높음 | A7 완료 후 진행 |
| A9 | `dialog:allow-open` → 디렉토리 전용 최소 권한으로 좁히기 | 중간 | capabilities/default.json 1줄 수정 수준 |
| A10 | 출결 토글·청구 금액 조정 1단계 Undo 스택 구현 | 중간 | Phase 2(출결) sprint에서 처리 |
| A11 | 퇴교 확인 `window.confirm()` → shadcn/ui Dialog 교체 | 낮음 | Phase 2 이후 처리 가능 |
| A12 | cipher on 환경 실측 (v0.2.0 인스톨러 배포 후) | 중간 | A6 carry-over — 3초 예산(PRD §5.6) 검증 |
| A13 | simplify 스킬 호출 기준에 "사용처 2곳 이상 시 추출 권고" 명시 | 낮음 | CLAUDE.md 또는 skills/simplify.md 보완 |

---

## Phase 1 완료 종합 평가

Sprint 3에서 Phase 1(원생 관리 기반)이 완료되었다. ROADMAP M2 마일스톤(원생 등록/조회 + 마법사 + 글로벌 검색)을 충족하며, 접근성 기반과 앱 레이아웃 셸이 Phase 2 이후 모든 화면의 토대로 확립되었다.

주요 이연 항목(R12 paths 동적화, salt.bin 이전)은 Sprint 4 진입 전 완료가 필요하다. 이 작업이 완료되어야 비로소 개발 DB와 프로덕션 DB 경로가 분리되고, 실제 클라우드 동기화 환경에서 동작하는 앱이 된다.

cargo test 109건 전체 통과, 프론트엔드 TypeScript/ESLint/빌드 무오류 상태로 Sprint 4 진입 준비가 완료되었다.
