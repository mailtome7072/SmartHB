# Sprint Retrospective — Sprint 5

> Sprint 5: Phase 1.5b 안정화 — 환경 호환 + 다중 인스턴스 차단 + 시드 보정
> 기간: 2026-05-22 (단일 집중 세션)
> 브랜치: `sprint5` → develop 머지 (9 files 변경, sprint-review 테스트 수정 포함)
> 코드 리뷰: Critical 0 / High 0 / Medium 1 / Low 1

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint4-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A14 | `paths::tests::init_from_config_ignores_empty_path` flaky 테스트 격리 강화 | ⏸️ 범위 외 | Sprint 5 scope에 명시적으로 제외. OnceLock 병렬 격리 문제는 `--test-threads=1` 임시 우회 유지 |
| A15 | DnD 필터링 sort_order 충돌 해소 (R26) | ⏸️ 범위 외 | Sprint 5에서 codes 관련 로직 미수정. R26 계속 열림 |
| A16 | Next.js CVE-2025-66478 영향 분석 및 업그레이드 | ✅ 분석 완료 | T0에서 SmartHB에 대한 영향 분석 완료 — `output: 'export'` + Tauri WebView 로컬 로드 구조상 실질 위험 없음. scope.md에 상세 기록 |
| A17 | salt.bin 이전 (Keychain → cloud/smarthb/) | ⏸️ Sprint 6+ | 사용자 데이터 없는 시점이므로 Phase 2 진입 전 처리 권고 유지 |
| A18 | simplify 스킬 기준 "사용처 2곳 이상 시 추출 권고" 명시 | ⏸️ 메타 작업 이연 | 본 sprint 범위 외 |

---

## 잘한 점

**6개 Task 계획을 단일 세션에 전수 완료했다.**

T0부터 T5까지 계획 순서대로 진행하여 당일 완료했다. 특히 tauri-plugin-single-instance 도입(T1)에서 예상 의존성(`@tauri-apps/plugin-single-instance` npm 패키지)이 실제로는 불필요하다는 사실을 Tauri 공식 문서에서 사전 확인하여 scope.md를 즉시 조정했다. 미등록 의존성을 추가하지 않은 결과로 node_modules 불필요 증가를 막았다.

**방어적 마이그레이션 설계(V201)가 멱등성을 완전히 달성했다.**

V201은 V104/V001 baseline 값과 정확히 일치하는 행만 변경하도록 WHERE 조건을 세밀하게 작성했다. `INSERT OR IGNORE`로 신규 결제수단을 추가하고, 기존 행은 baseline 매칭 후에만 UPDATE하는 방식으로 사용자 수정 데이터를 보호한다. pre-release 시점이라 사용자 데이터가 없지만, 방어적 접근이 이후 마이그레이션 패턴의 좋은 선례가 되었다.

**LockPage 분기 렌더링이 체계적으로 구현되었다.**

T1-sub에서 `checkLockStatus()` IPC 결과에 따라 SplashScreen → LockWarning / LockScreen을 분기하는 구조가 깔끔하게 구현되었다. `useCallback`으로 `refresh`를 메모이제이션하고, LockWarning의 `onForceAcquired`와 `onRetry` 콜백이 동일한 `refresh` 함수를 재사용하는 패턴은 상태 일관성을 보장하면서 코드 중복을 제거했다.

**CVE 영향 분석을 sprint 내에서 완결했다.**

A16(Sprint 4 이월)의 CVE-2025-66478 분석을 T0에서 완료하여 hotfix 필요 여부를 확정했다. SmartHB의 정적 빌드 + Tauri WebView 구조가 외부 공격 표면이 없음을 확인하고, scope.md의 "발견된 이슈" 섹션에 근거를 기록했다. 분석 결과가 CHANGELOG에도 반영되어 추후 감사 가능성을 확보했다.

---

## 개선할 점

**V201 마이그레이션 추가 후 테스트 3건이 sprint-review 시점에야 발견되었다.**

V104 시드 값을 가정하고 작성된 테스트(`in_memory_pool_runs_migrations`, `match_fee_returns_exact_match`, `weekly_hours_unique_violation_returns_korean`)가 V201 적용 이후에도 업데이트되지 않아 sprint-review 단계에서 처음 실패했다. cargo test가 sprint5 T5 통합 검증에서도 통과된 기록이 있는 것은 T5 당시 테스트가 130건 통과한 것으로 기록되어 있으나, 이 시점에 인메모리 마이그레이션 순서와 테스트 격리 방식에 따라 이미 V201이 반영된 상태에서 통과했을 가능성이 있다. 어느 쪽이든, **DB 시드를 변경하는 마이그레이션 작성 시 관련 테스트를 동일 커밋에 함께 업데이트하는 규칙**이 필요하다.

**lock/page.tsx 에러 화면에 재시도 버튼이 누락되었다.**

코드 리뷰에서 Medium 이슈로 발견되었다. `checkLockStatus()` IPC 호출이 일시적으로 실패하면 에러 메시지만 표시되고 재시도 수단이 없어 사용자가 앱을 강제 종료해야 한다. 앱 시작 직후 IPC 호출 타이밍 문제로 일시 실패 가능성은 낮지 않다. 재시도 버튼 추가는 1줄 변경이므로 Sprint 6 진입 전 hotfix 또는 Sprint 6 초반에 처리하는 것이 적절하다.

**lock/page.tsx refresh() 후 순간 스테일 렌더링 가능성이 존재한다.**

코드 리뷰에서 Low 이슈로 발견되었다. `refresh()` 호출 시 `setError(null)`이 즉시 실행되면서 이전 `lockStatus` 값으로 잘못된 화면이 잠깐 렌더될 수 있다. 실제 체감 빈도는 낮지만, `refresh()` 호출 시 `lockStatus`도 `null`로 초기화하여 SplashScreen을 재표시하는 방식으로 완전히 해소할 수 있다.

**A14(flaky 테스트)와 A15(DnD sort_order) 이월이 3번째 이월이다.**

A14와 A15 모두 Sprint 4, 5에 이어 이번에도 "범위 외"로 처리되었다. 두 항목 모두 현재 `--test-threads=1` 우회와 코드 주석 경고만으로 관리되고 있어 언제 터질지 모르는 기술 부채로 누적되고 있다. Sprint 6 계획 수립 시 Phase 2 작업 사이에 반드시 할당해야 한다.

---

## 이연 항목 처리 권고

### Medium 이슈: lock/page.tsx 에러 화면 재시도 버튼 누락

에러 화면에 `<button onClick={refresh}>다시 시도</button>` 추가. 동시에 `refresh()` 시작 시 `setLockStatus(null)`도 호출하여 스테일 렌더링(Low 이슈)을 함께 해소. 파일 1개, 5줄 이하 수정 → Sprint 6 초반 또는 hotfix 대상.

### R26 — DnD 필터링 sort_order 충돌 (3번째 이월)

Sprint 6에서 코드 테이블 관련 작업이 예정되지 않으면 별도 spike(1~2h)로 방법 B 구현 권고. 방법 B: `handleDragEnd`에서 visibleCodes 기준 재정렬 후 전체 codes 배열 재매핑.

### A14 — flaky 테스트 격리 강화 (3번째 이월)

`--test-threads=1` 우회 제거 목표. OnceLock 초기화 방식을 테스트별 분리 가능한 구조로 리팩토링. Sprint 6 백엔드 작업 블록에 2h 할당 권고.

---

## Sprint 6 액션 아이템

| ID | 항목 | 우선도 | 비고 |
|----|------|--------|------|
| A19 | DB 시드 변경 마이그레이션 작성 시 관련 테스트 동일 커밋에 업데이트 규칙 도입 | 높음 | CLAUDE.md 또는 `.claude/rules/backend.md`의 "테스트" 섹션에 명문화 |
| A20 | lock/page.tsx 에러 화면 재시도 버튼 추가 + refresh 시 lockStatus null 초기화 | 중간 | Sprint 6 초반 1~2h 또는 hotfix. 코드 리뷰 Medium+Low 이슈 동시 해소 |
| A21 | A14 flaky 테스트 격리 강화 — OnceLock 테스트별 분리 구조 리팩토링 | 중간 | Sprint 6 백엔드 블록에 2h 할당. 3번째 이월 — 더 이상 미룰 수 없음 |
| A22 | A15 DnD 필터링 sort_order 충돌 해소 (R26) — 방법 B 구현 | 중간 | Sprint 6 codes 관련 작업 시 처리 또는 별도 spike 2h. 3번째 이월 |
| A17 | salt.bin 이전 (Keychain → cloud/smarthb/) | 높음 | Phase 2 진입 전 처리 필수. 사용자 데이터 없는 지금이 최적 타이밍 |

---

## Sprint 5 종합 평가

Phase 1.5b 목표를 단일 세션에 전수 완료했다. 6개 Task 모두 AC를 충족했으며, 코드 리뷰 Critical/High 이슈 없음. 자동 검증은 sprint-review 시점에 테스트 3건이 최초 발견·수정되었으나, 수정 자체는 명확한 V201 시드 변경 반영이었다.

sprint-review에서 새롭게 발견된 사항: V201 마이그레이션 추가 시 연동 테스트를 동일 커밋에 업데이트하지 않은 점이 유일한 프로세스 이슈였다. 이를 A19 액션 아이템으로 규칙화한다.

Phase 2(학사 스케줄) 진입 기반이 마련되었다. 다중 인스턴스 차단과 LockWarning 라우팅이 안정화되어 양 PC 사용 시나리오 대응이 완성되었다. 시드 데이터도 실제 교습소 운영 값으로 보정되어 Phase 2 이후 출결·청구 도메인 구현 시 올바른 기초 데이터가 갖춰졌다.

cargo test: 130건 통과 (sprint-review 수정 포함) | cargo clippy: 경고 0 | pnpm tsc --noEmit: 오류 0 | pnpm lint: 경고/오류 0 | pnpm build: 성공
