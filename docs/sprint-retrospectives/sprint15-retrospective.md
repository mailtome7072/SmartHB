# Sprint Retrospective Sprint 15

> 대상: Sprint 15 (develop...sprint15) — 교습소 정보 화면 + 자가 진단 이력 삭제 + 전역 단축키/툴팁 + 접근성 대비 개선(17건) + 청구 N+1 최적화 + monthly_summary 리팩토링
> 리뷰 일자: 2026-06-07
> 코드 리뷰: Critical 0 / High 0 / Medium 1 (F3) / Low 2 (F1, F2)
> 자동 검증: cargo test 375 passed (cipher off) / clippy --all-targets clean / cargo check --features cipher 통과 / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint14-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A94 | 계획 시 위젯 동작·UI를 더 구체적으로 확정 | ✅ 완료 | Sprint 15는 안정화 스프린트로 신규 위젯 없음. T1(교습소 정보) 필드 목록·이미지 저장 방식을 계획 단계에서 확정하여 검증-phase 추가 요청 최소화 |
| A95 | `monthly_summary` GROUP BY 리팩토링 | ✅ 완료 | T0에서 처리. R99 해소. 단위 테스트 2건으로 동치성 보장 |
| A96 | 복원 리허설 dev 환경 개선 | ⏸️ 이연 | Low 우선순위. Sprint 15 범위 외. Sprint 16 이후 기회 발생 시 |
| A97 | 대시보드 위젯 타이틀 inline `fontSize` → Tailwind 통일 | ✅ 완료 | T0에서 처리. `text-2xl` 클래스로 교체 |
| A89 | 공지문 페이지(`/notices`) 분리 검토 | ✅ 판단 완료 (구현 이연) | T4에서 로직 분리 이미 완료(`notice-generator.ts`) 확인. UI 구획화(`/notices/page.tsx` 3분할)만 Sprint 16 이연 결정 |

---

## 잘한 점

**사용자 시각 검증을 T5 버퍼로 계획에 편입해 체계적으로 진행했다.**

T5(가변 버퍼 3h)를 "사용자 검증 병행 수집"으로 명시적으로 배정하여, 시각 검증에서 발견된 개선 사항(설정 카드 순서, 글로벌 툴팁, 폰트 미세 조정 등)을 범위 초과 없이 수용했다. Sprint 14에서 검증-phase가 계획의 2~3배로 확대됐던 경험을 반영한 결과다.

**`--all-targets` clippy 부채를 T4에서 체계적으로 해소했다.**

Sprint 14까지 `cargo clippy -- -D warnings` (기본 타깃) 만 실행하여 테스트 코드의 clippy 경고가 누적됐다(`makeup.rs`, `dashboard.rs` 테스트 코드 6건). T4에서 `--all-targets` 플래그로 전수 정리했고, 이번 sprint-review 자동 검증도 `--all-targets` 기준으로 clean을 확인했다.

**안정화 스프린트 기조를 지켜 Capacity 초과 없이 완료했다.**

신규 비즈니스 로직 없이 검증/감사/리팩토링 중심으로 T0~T6(+T5)를 완료했다. T7~T9(물리 환경 의존)는 계획 수립 당일에 Sprint 16 이연을 결정하여 무리한 일정을 사전에 차단했다. 계획 38h 내에서 수용 완료.

**GlobalTooltip의 document 이벤트 위임 패턴이 개별 컴포넌트 수정 없이 전체 앱에 적용됐다.**

50대 사용자 가독성을 위한 툴팁 폰트 확대(20px)를 위해 기존 모든 `title` 속성 요소를 일일이 수정하지 않고, `document.addEventListener('mouseover')` 캡처 위임으로 단일 컴포넌트에서 일괄 처리했다. cleanup 함수에서 이벤트 3종 제거 + `restore()` 호출로 메모리 누수도 없음.

**Explore 조사 결과의 과대평가를 코드 리뷰에서 교정했다.**

Sprint-dev 중 `button.tsx` 실사용 여부와 `staleTime` 설정에 대한 Explore 조사가 실제 코드와 불일치(실사용 0, staleTime 의도적)를 지적했는데, 이는 AI 에이전트가 범위 외 파일을 조사하면서 과대평가한 사례다. scope.md 기반 범위 제한이 실제로 동작했음을 확인.

---

## 개선할 점

**`self-verify` 기본 명령에 `--all-targets`가 없어 테스트 코드 clippy 부채가 장기 누적됐다.**

CLAUDE.md 및 harness-engineering.md의 self-verify 명령이 `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 로 명시되어 있어, 프로덕션 코드만 검사하고 테스트 코드(`#[cfg(test)]` 블록, `src/bin/` 등)는 검사 대상에서 제외됐다. Sprint 14까지 6건의 테스트 코드 clippy 경고가 발견되지 않고 누적됐다. `--all-targets`를 기본 명령에 포함해야 이 문제가 반복되지 않는다.

**T1 교습소 정보 화면의 이미지·텍스트 저장 시점 불일치가 미저장 이탈 경고 누락을 만든다 (F3 Medium).**

이미지는 업로드 즉시 파일 저장, 텍스트는 저장 버튼 필요라는 이중 저장 경로가 혼란을 줄 수 있다. PRD §5.7의 미저장 경고 다이얼로그 공통 구현이 Sprint 16에서 이루어져야 한다.

**Ctrl+N 단축키가 입력 필드 포커스 중에도 동작한다 (F2 Low).**

`GlobalShortcuts`에서 활성 포커스 엘리먼트(`INPUT`, `TEXTAREA`)를 체크하는 방어 로직이 없어, 텍스트 입력 중 Ctrl+N 이 원치 않는 화면 전환을 일으킬 수 있다. Sprint 16 Ctrl+S/F1 구현 시 방어 로직을 함께 추가해야 한다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A98 | self-verify 명령 및 CLAUDE.md에 `--all-targets` 추가 — `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 로 교체하여 테스트 코드 clippy 경고를 빌드 시점에 차단 | High | `CLAUDE.md` self-verify 섹션, `.claude/rules/harness-engineering.md` self-verify 절차 | Sprint 16 착수 전 (즉시 적용) |
| A99 | Sprint 16 Ctrl+S/F1 단축키 구현 시 입력 필드 포커스 방어 로직 추가 — `GlobalShortcuts`에서 `(e.target as HTMLElement).tagName` 이 `INPUT`/`TEXTAREA`/`SELECT`이면 Ctrl+N을 억제하는 조건 추가 | Medium | `src/components/layout/GlobalShortcuts.tsx` | Sprint 16 단축키 구현 시 |
| A100 | 미저장 이탈 경고 다이얼로그(PRD §5.7) 공통 구현 — 양식 입력 화면(교습소 정보 포함)에서 미저장 상태의 이탈 시 경고. React Context 또는 router.events 기반 공통 훅으로 구현 | Medium | `src/hooks/useUnsavedChanges.ts` (신규) + `src/app/settings/info/page.tsx` 등 적용 대상 화면 | Sprint 16 T 배정 시 포함 |
| A96 | 복원 리허설 dev 환경 개선 — cipher off 빌드에서 리허설 동작 확인 가능하도록 테스트용 평문 DB 파일 자동 생성 또는 dev 전용 더미 백업 지원 검토 | Low | `src-tauri/src/commands/backup.rs` 또는 dev 스크립트 | 기회 발생 시 (Sprint 16 또는 이후) |
