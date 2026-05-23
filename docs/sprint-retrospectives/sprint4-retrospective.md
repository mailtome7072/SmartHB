# Sprint Retrospective — Sprint 4

> Sprint 4: Phase 1.5 품질 안정화 — 스테이징 검증 14개 이슈 전수 해소 + post-T11 4건
> 기간: 2026-05-21 (단일 집중 세션 다중 분할)
> 브랜치: `sprint4` → develop 머지 (34 files, Sprint 4 핵심 변경)
> 코드 리뷰: Critical 0 / High 0 / Medium 1 / Low 1

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint3-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A7 | `paths::data_root()` 동적화 | ✅ 완료 | Sprint 3 직후 hotfix `82eb1b2`에서 완료. Sprint 4 진입 전 해소 |
| A8 | salt.bin 이전 (Keychain → cloud/smarthb/) | ⏸️ Sprint 5 이연 | sprint4.md 명시 — 본 sprint 범위 외 |
| A9 | `dialog:allow-open` 최소 권한으로 좁히기 | ✅ 완료 | T1에서 Tauri 2.x `dialog:allow-open`이 최소 단위임 확인 후 현행 유지 결정 (R21 해소) |
| A10 | 출결 토글 1단계 Undo 스택 | ⏸️ Phase 2 | Sprint 5~6 출결 구현 시 처리 |
| A11 | `window.confirm()` → shadcn/ui Dialog 교체 | ✅ 완료 | T1 핵심 작업. Critical Runtime Error 동시 해소 (R22 해소) |
| A12 | cipher on 환경 실측 | ⏸️ 인스톨러 배포 후 | v0.2.0 배포 후 측정 예정 |
| A13 | simplify 기준 "사용처 2곳 이상 시 추출 권고" 명시 | ⏸️ 메타 작업 이연 | 본 sprint 범위 외 |

---

## 잘한 점

**14개 이슈 전수 해소 + post-T11 4건 추가 처리로 Phase 1 품질 확립에 성공했다.**

Sprint 3 스테이징 검증에서 발견된 14개 이슈(Critical 1건 포함)를 모두 해결했다. T1의 `window.confirm` 차단 이슈는 Tauri 2.x CSP 정책에서 비롯된 Critical Runtime Error로, shadcn/ui AlertDialog 도입이 퇴교 확인 UX 개선과 기술 부채 해소를 동시에 달성했다. T11 통합 검증 과정에서 사용자가 추가 보고한 4건도 즉시 처리하여 scope 외 확장을 자연스럽게 흡수했다.

**`defense in depth` 패턴이 일련번호 보호에 적용되었다.**

일련번호(`serial_no`) 수정 차단을 프론트엔드 `readOnly` + 백엔드 UPDATE SQL 컬럼 제외 이중 가드로 구현했다. 단일 레이어 방어에 그치지 않고 각 레이어가 독립적으로 보호하는 구조가 명확히 드러났다. 주석에 "defense in depth" 의도를 명시하여 이후 유지보수자의 혼동을 예방했다.

**correlated subquery로 N+1 IPC 문제를 사전에 방지했다.**

원생 목록에 `weekly_hours`와 `schedule_days_csv`를 추가할 때 별도 IPC 호출 없이 `list_students` 단일 쿼리에 correlated subquery로 묶었다. 100명 규모에서 SQLite 최적화로 충분하다는 근거도 주석으로 명시했다. IPC 요청 증가 없이 기능을 확장한 좋은 예다.

**운영 시간 저장에 이중 검증(FE + BE)을 적용했다.**

`save_operating_hours` IPC가 7개 요일 수 검증, 요일 코드(1~7) 범위 검증, open/close 시간 쌍 일관성 검증, HH:MM 형식 검증을 백엔드에서 수행하며, 프론트엔드도 저장 전에 open >= close 사전 검증을 수행한다. 백엔드 단독 검증만으로는 UX 응답이 느려지는 문제를 FE 사전 검증으로 해소했다.

**세션 내 scope.md 관리가 carry-over 없이 완료에 기여했다.**

세션 #4 scope.md에 "사전 확인" 섹션을 두어 V201 마이그레이션이 불필요함을 검증하고 시작했다. 불필요한 마이그레이션 작업을 제거하고 이미 존재하는 컬럼(`withdraw_date`)을 활용함으로써 작업 범위를 줄이고 기존 데이터 위험을 피했다.

---

## 개선할 점

**DnD 필터링과 sort_order 일관성이 충분히 검토되지 않았다.**

코드 테이블 DnD 구현(T10)에서 활성 필터가 적용된 상태로 드래그하면 숨겨진 항목과 sort_order가 충돌할 수 있다는 사실이 코드에 주석으로만 기록되었다. 구현 시점에 이 문제를 인식했음에도 "단순화를 위해 허용"으로 결정했는데, 이 결정이 Risk Register에 등록되지 않았다. 기술 트레이드오프는 코드 주석 외에 Risk Register에 반드시 등록해야 다음 스프린트에서 처리 여부를 검토할 수 있다.

**T11 통합 검증 중 사용자 추가 보고(4건)가 scope 없이 즉시 수정되었다.**

post-T11 4건은 사용자가 직접 시각 검증 중 발견한 항목으로, scope.md 업데이트 없이 즉시 처리되었다. 하네스 엔지니어링 원칙 1(Planning First)에 따르면 scope.md를 먼저 업데이트해야 하지만 긴급성이 낮고 범위가 소규모라 건너뛴 것으로 보인다. 소규모 추가 수정이라도 scope.md에 `post-T11 추가 항목` 섹션을 추가하여 추적 가능성을 유지하는 것이 바람직하다.

**Next.js CVE-2025-66478 대응이 현 sprint에 포함되지 않았다.**

CHANGELOG의 Security 섹션에 "Sprint 5 또는 hotfix 업그레이드 필수"로 기록되었으나, 이 CVE가 실제로 SmartHB의 공격 표면에 영향을 미치는지 여부를 확인하지 않았다. 데스크톱 앱 특성상 외부 요청을 받지 않으므로 실제 위험도는 낮을 가능성이 있으나, Sprint 5 진입 전 영향 범위를 명확히 파악하고 처리 방식(hotfix vs sprint 내 포함)을 결정해야 한다.

---

## 이연 항목 처리 권고

### R26 — DnD 필터링 sort_order 충돌

활성 필터 적용 상태에서 DnD 정렬 시 보이는 행만 재번호 부여되어 숨겨진 행과 sort_order가 충돌할 수 있다. 해결 방법 2가지:

- **방법 A**: DnD 활성 시 필터를 '전체'로 강제 전환 (UX 변경 있음)
- **방법 B**: handleDragEnd에서 visibleCodes 기준 재정렬 후 전체 codes 배열을 재매핑하여 숨겨진 항목도 연속 번호 유지

방법 B가 UX 변경 없이 더 자연스럽다. Sprint 5 codes 관련 작업 시 함께 처리.

### Next.js CVE-2025-66478

Sprint 5 진입 전 또는 진입 직후 영향 분석 후 처리 방식 결정 필요. 분석 시 참고: 이 앱은 외부 네트워크 요청을 받지 않으며 Tauri WebView 내에서 정적 파일을 서빙하는 구조.

---

## Sprint 5 액션 아이템

| ID | 항목 | 우선도 | 비고 |
|----|------|--------|------|
| A14 | `paths::tests::init_from_config_ignores_empty_path` flaky 테스트 격리 강화 | 중간 | OnceLock 병렬 격리 문제. `--test-threads=1` 임시 우회 중 |
| A15 | DnD 필터링 sort_order 충돌 해소 (R26) | 중간 | 방법 B 권장 — visibleCodes DnD 후 전체 재정렬 |
| A16 | Next.js CVE-2025-66478 영향 분석 및 업그레이드 | 높음 | Sprint 5 진입 전 결정 필요 |
| A17 | salt.bin 이전 (Keychain → cloud/smarthb/) | 높음 | A8 carry-over — paths 동적화 완료된 지금 처리 가능 |
| A18 | simplify 스킬 기준 "사용처 2곳 이상 시 추출 권고" 명시 | 낮음 | A13 carry-over — CLAUDE.md 또는 skills/simplify.md 보완 |

---

## Sprint 4 종합 평가

Phase 1.5 품질 안정화 목표를 완전히 달성했다. 14개 사용자 보고 이슈 + post-T11 4건 모두 해소, 자동 검증(cargo test 130건 / clippy 0건 / tsc 0건 / lint 0건 / build 성공) 전체 통과, 코드 리뷰 Critical/High 이슈 없음.

특히 Sprint 3에서 A11(낮음 우선도)로 이연했던 `window.confirm()` 교체가 실제로 앱을 사용 불가 상태로 만드는 Critical Runtime Error임이 스테이징 검증에서 확인된 점은 중요한 교훈이다. 이론적 우선도와 실제 사용자 체감 우선도의 차이를 다음 sprint 계획 수립 시 반영해야 한다.

Phase 2(학사 스케줄 + 출결 관리) 진입 기반이 확립되었다. 교습소 운영 시간 IPC가 Sprint 5 스케줄 제약 구현에 바로 활용 가능하며, format.ts 유틸리티(formatPhone, formatCurrency)도 Phase 4 청구 화면에서 재사용 가능하다.

cargo test: 130건 통과 | 프론트엔드 TypeScript/ESLint/빌드 무오류 | `pnpm tauri:dev` 수동 검증 대기 중.
