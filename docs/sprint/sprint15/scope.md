---
Sprint: 15  |  Date: 2026-06-07  |  Session: #1
---

## 이번 세션 목표 Task
- **T0**: Sprint 14 액션 아이템 해소 (A95 monthly_summary 리팩토링 + A97 inline style 통일)
- (여유 시) **T1**: 교습소 정보 화면 착수

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/dashboard.rs | [2회] | T0-A95: monthly_summary() GROUP BY b.id 리팩토링 + 테스트 수정/엣지케이스 추가 |
| src/components/dashboard/DashboardView.tsx | [1회] | T0-A97: 위젯 타이틀 inline fontSize 24px → Tailwind text-2xl (3건) |

<!-- T1 착수 시 추가 예정 (착수 전 scope 갱신):
| src-tauri/src/commands/settings.rs | [0회] | T1: get/save_academy_info IPC |
| src-tauri/src/commands/mod.rs | [0회] | T1: 커맨드 등록 |
| src-tauri/src/lib.rs | [0회] | T1: invoke_handler 등록 |
| src/app/settings/info/page.tsx | [0회] | T1: 교습소 정보 폼 |
| src/types/settings.ts | [0회] | T1: AcademyInfo 타입 |
| src/lib/tauri/index.ts | [0회] | T1: IPC 래퍼 |
-->

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/migrations/ — Sprint 15는 DB 마이그레이션 없음 (계획서 명시)
- [ ] package.json / Cargo.toml — 신규 의존성 없음 (계획서 명시)

## 완료 기준 (이번 세션 — T0)
- [x] A95: monthly_summary() GROUP BY 서브쿼리 리팩토링, R99 해소
- [x] A95: 기존 테스트(monthly_summary_totals_billing_and_paid) 통과 + 엣지 케이스 1건 추가
- [x] A97: DashboardView 위젯 타이틀 3건 Tailwind text-2xl 통일 (포스트잇 동적 높이 inline 유지)
- [~] A89: /notices/page.tsx 분리 여부 판단 → T4에서 처리 (이번 세션 범위 외, 계획대로)
- [x] cargo test / clippy / pnpm lint / tsc 통과 후 커밋

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌 발견 시 여기 기록 후 사용자 보고 -->
- **R99 전제 정정 (scope 내 처리, 보고 완료)**: F1/R99는 "1:1 *암묵* 의존"으로 기록됐으나, 실제로는 `payments.bill_id UNIQUE`(V109)로 DB가 1:1을 **명시적으로 강제** 중. 따라서 현재 중복 합산 위험은 없음. A95는 마이그레이션 없이 **방어적 GROUP BY 서브쿼리**로 전환 — 현재 동작 100% 동일, 향후 부분 수납(UNIQUE 해제 + 결제 금액 컬럼 추가) 도입 시 코드 변경 없이 fan-out 차단. **완전한 부분 수납 지원은 마이그레이션 필요 → Sprint 16+** (Sprint 15 scope: 마이그레이션 없음).
