---
Sprint: 15  |  Date: 2026-06-07  |  Session: #1
---

## 이번 세션 목표 Task
- **T0**: Sprint 14 액션 아이템 해소 (A95 monthly_summary 리팩토링 + A97 inline style 통일)
- (여유 시) **T1**: 교습소 정보 화면 착수

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행. 아래는 실제 Edit/Write 호출 기준 정정값(hook 자동 카운트는 일부 오집계). -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/dashboard.rs | [2회] | T0-A95: monthly_summary() GROUP BY 서브쿼리 + 엣지 테스트 |
| src/components/dashboard/DashboardView.tsx | [2회] | T0-A97: inline fontSize→text-2xl / T5: 24px→22px(text-[22px]) |
| src-tauri/src/commands/settings.rs | [2회] | T1: AcademyInfo + get/save_academy_info IPC |
| src-tauri/src/lib.rs | [1회] | T1: invoke_handler 등록 |
| src/app/settings/info/page.tsx | [4회 ⚠️] | T1: 교습소 정보 폼 신규 생성 (Write 1회 — hook이 [5회]로 오집계, 실제 루프 아님: lint/tsc 통과) |
| src/app/settings/page.tsx | [4회] | T1: 카드 활성화 / T5: 순서변경(정보 맨앞)+마법사→DB폴더변경(예정)+PIN↔백업 순서. (disabledHint 확대는 오해로 적용 후 원복) |
| src/lib/tauri/index.ts | [1회] | T1: AcademyInfo 타입 + 래퍼 (types/settings.ts 미사용, 기존 패턴 따름) |
| src/app/students/edit/page.tsx | [1회] | T5: 상단 '원생관리 메인' 버튼 추가 |
| src/components/schedules/ClassCalendar.tsx | [2회] | T5: 월 보기 인원 배지 hover 툴팁 native title→커스텀 div(폰트 24px, 2배) |

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
