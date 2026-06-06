# Sprint Retrospective Sprint 14

> 대상: Sprint 14 (develop...sprint14) — 대시보드(위젯 7종+알림 4종) + 데이터 자가 진단(검사 7종+자동/수동) + 엑셀 내보내기(3종) + 복원 리허설 + 원생 생년월일 + V303~V305 마이그레이션
> 리뷰 일자: 2026-06-06
> 코드 리뷰: Critical 0 / High 0 / Medium 1 (F1) / Low 2 (F2, F3)
> 자동 검증: cargo test 369 passed (cipher off) / clippy clean / cargo check --features cipher 통과 / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint13-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A90 | 계획 수립 시 carry-over 항목 코드 현황 먼저 확인 | ✅ 완료 | sprint-planner가 T0 배치 전 A91/A93 코드 현황 직접 확인. A91은 주석/ADR 수정 필요 확인, A93은 이중 표시 여전히 존재 확인 후 T0 배치. |
| A91 | `startup.rs` cipher-off 동작 명시 주석 + ADR-008 수정 | ✅ 완료 | T0에서 처리. 주석을 실제 동작(keyring 시도 + 프론트 개발 모드 예외 차단)으로 갱신. |
| A92 | CLAUDE.md 마이그레이션 현황 갱신 절차화 | ✅ 완료 | scope.md 체크리스트에 "마이그레이션 추가 시 CLAUDE.md 갱신" 포함. V305까지 갱신 완료. |
| A93 | `/lock` SplashScreen 이중 표시 단일 통합 | ✅ 완료 | T0에서 처리. 단일 로딩 상태로 통합. |
| A89 | 공지문 페이지 분리 검토 | ⏸️ 이연 | Sprint 14 범위 외. Sprint 15 우선순위 검토 시 결정. |

---

## 잘한 점

**사용자 실DB 검증이 4~5회 수행되어 AI 생성 코드의 비즈니스 로직 오류를 조기에 발견·수정했다.**

보강필요시간 음수 오탐(성춘향 케이스), 공휴일 출결 진행률 오탐, 자가 진단 이력 중복 누적, 퇴교생 보강 소멸 알림 오탐(홍길동/고길동 케이스) 등 4건의 비즈니스 로직 오류가 모두 사용자 실DB 검증에서 발견됐다. 각 발견에 대해 근본 원인을 분석하고 회귀 테스트를 추가한 후 수정했다. 총 테스트 369건이 이 과정의 품질 누적을 반영한다.

**계획 대비 사용자 요청으로 기능이 크게 확장됐으나 스프린트 내 수용했다.**

당초 계획(CSV 내보내기)이 엑셀(.xlsx) 전환으로 변경됐고, 청구총액 추이 그래프 신설, 포스트잇 3장(드래그 리사이즈), 이달의 생일 위젯, 원생 생년월일, 이전/다음 월 전환, 완전 0건 자가 진단, 해결 항목 자동 재검증 등 다수의 기능이 검증 중 추가됐다. rust_xlsxwriter를 도입하여 엑셀 서식(천단위 콤마·우측정렬·autofit)을 순수 Rust로 구현했다.

**출결 진행률 위젯의 설계 무결성 문제를 사용자 검증에서 조기에 발견하고 제거했다.**

"출결이 월 단위 'present' 기본값으로 일괄 생성되는 모델에서는 미입력 상태가 존재하지 않아 진행률이 항상 100%"라는 근본 원인을 분석해, 위젯 자체를 제거하는 결단을 했다. 무의미한 지표를 제거함으로써 대시보드의 신뢰성이 높아졌다. 관련 알림도 함께 제거했다.

**자가 진단의 "완전 0건" 정책과 `last_auto_diagnosis` 분리가 설계를 단순화했다.**

이상 0건이면 이력을 남기지 않는 정책(완전 0건 정책)으로 이력 테이블이 실제 이상 항목만 보관한다. 월 1회 자동 진단 추적은 `app_settings.last_auto_diagnosis`로 분리하여 이력 유무에 독립적이다. 해결 항목 자동 재검증(`reconcile_resolved_issues`)이 이력을 자동 정리하므로 수동 삭제 기능 없이도 이력이 깨끗하게 유지된다.

**Node 25 + Next.js 15 webpack 캐시 abort 크래시를 dev 전용 설정으로 격리했다.**

집 Mac의 Node 25 환경에서 webpack 캐시 직렬화 abort가 dev 서버를 크래시시키는 문제를 `next.config.ts` dev 전용 `config.cache = false`로 격리했다. production 빌드(`pnpm build`)와 Windows PC에는 영향이 없다. 환경별 회피 방법을 문서화했다(scope.md 이슈 기록).

---

## 아쉬운 점 / 개선할 점

**검증-phase에서 사용자 요청이 매우 많아 계획 대비 구현 범위가 크게 확장됐다.**

스프린트 계획 시점에는 T8까지 38시간으로 설계됐으나, 검증 중 기능 추가(그래프·포스트잇·생일·생년월일·월 전환·완전 0건 등)와 버그 수정(퇴교생 보강·오탐·중복 등)이 반복됐다. 실제 검증-phase가 계획의 2~3배 분량을 소화했다. 이는 사용자가 실앱을 보면서 요구사항을 구체화하는 자연스러운 과정이나, 향후 스프린트에서 "대시보드 위젯 개수와 동작을 계획 시 더 구체적으로 확정"하면 검증-phase 확장을 줄일 수 있다.

**`monthly_summary` 청구 집계가 1:1 청구-수납 모델에 암묵적으로 의존한다 (F1 / Medium).**

`bills LEFT JOIN payments`에서 `COUNT(*)`를 bill_count로 사용하는 패턴은 현재 스키마(1:1)에서 올바르나, Sprint 15 청구 마감 워크플로우 확장 시 `GROUP BY b.id` 없이 집계를 확장하면 중복 합산이 발생할 수 있다. 선제적 리팩토링이 필요하다.

**복원 리허설의 cipher off 빌드 한계가 시각 검증을 어렵게 한다.**

cipher off 개발 빌드에서는 실제 백업 파일이 없어 `/settings/backup` 목록이 항상 비어있다. 평문 SQLite 파일을 수동 배치하거나 cipher on 빌드로만 리허설을 체험할 수 있다. 개발 편의를 위한 "테스트 백업 파일 생성" 기능 또는 dev 환경 전용 더미 백업 자동 생성을 검토할 수 있다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A94 | 계획 시 대시보드 위젯 동작·UI를 더 구체적으로 확정 — "위젯 6개"가 아닌 각 위젯의 표시 데이터·인터랙션을 계획 단계에서 명세화하여 검증-phase 추가 요청 최소화 | Medium | 프로세스 (sprint-planner) | Sprint 15 계획 시 적용 |
| A95 | `monthly_summary` 청구 집계 GROUP BY 리팩토링 — `bills LEFT JOIN payments GROUP BY b.id` 패턴으로 부분 수납 확장 대비 | Medium | `src-tauri/src/commands/dashboard.rs` | Sprint 15 청구 마감 워크플로우 확장 시 포함 |
| A96 | 복원 리허설 dev 환경 개선 — cipher off 빌드에서도 리허설 동작을 확인할 수 있도록 테스트용 평문 DB 파일 자동 생성 또는 dev 전용 더미 백업 지원 검토 | Low | `src-tauri/src/commands/backup.rs` 또는 dev 스크립트 | Sprint 15 또는 기회 발생 시 |
| A97 | 대시보드 위젯 타이틀 inline `fontSize` → Tailwind 클래스 통일 — `text-2xl`(24px) 상수 클래스로 교체하여 테마 일관성 확보 | Low | `src/components/dashboard/DashboardView.tsx` | Sprint 15 UI 정비 시 |
| A89 | 공지문 페이지(`/notices`) 분리 검토 — 1534줄 단일 컴포넌트를 캔버스/편집/저장 섹션으로 분리 | Low | `src/app/notices/page.tsx` | 다음 스프린트 우선순위 검토 시 결정 |
