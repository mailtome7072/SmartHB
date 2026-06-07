---
Sprint: 15  |  Date: 2026-06-07  |  Session: #1
---

## 이번 세션 목표 Task (실제 진행)
- **T0** ✅ Sprint 14 액션 아이템 (A95 monthly_summary GROUP BY + A97 위젯 폰트)
- **T1** ✅ 교습소 정보 화면 (텍스트 9필드 + 로고/2D바코드 이미지)
- **T5** ✅ 마이너 UI 개선 (시각 검증 병행: 위젯 폰트, 설정 카드 순서, DB폴더 카드, 원생관리 버튼, 전역 툴팁 20px)
- **T2** ✅ 자가 진단 이력 수동 삭제 (행 단위 + 전체 비우기, 검증 완료)
- **T3** ✅ 접근성 감사 (보고서 + Critical: gray-400→600 대비, Ctrl+F·Ctrl+N 단축키. 밀집UI 44px·gray-500·F1·Ctrl+S는 Sprint16 이연)
- **T4** ✅ 기술 부채 정리 (테스트 clippy `--all-targets` 부채 6건 해소, console.log 0·문서 정확 확인, A89는 로직 분리 완료·UI구획화만 Sprint16 이연)
- **T6** ✅ 성능 프로파일링 (정적 분석 + 보고서. 안전 최적화: 청구 standard_fees N+1 제거. staleTime:0은 의도적 유지. 출결표 N+1·인덱스·공지문 I/O는 실측 후 Sprint16)

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행. 아래는 실제 Edit/Write 호출 기준 정정값(hook 자동 카운트는 일부 오집계). -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/dashboard.rs | [3회] | T0: monthly_summary GROUP BY+엣지테스트 / T4: insert_student 픽스처 clippy allow(too_many_args). 루프 아님(별개 의도) |
| src-tauri/src/commands/makeup.rs | [2회] | T4: 테스트 clippy 정리(needless_borrow 4 + doc 주석) — `--all-targets` 미적용 누적 부채 |
| src/components/dashboard/DashboardView.tsx | [2회] | T0-A97: inline fontSize→text-2xl / T5: 24px→22px(text-[22px]) |
| src-tauri/src/commands/settings.rs | [2회] | T1: AcademyInfo + get/save_academy_info IPC |
| src-tauri/src/lib.rs | [2회] | T1+T2: invoke_handler 등록 (academy_info, diagnosis 이력 삭제) |
| src/app/settings/info/page.tsx | [1회] | T1: 교습소 정보 폼 신규 생성 (Write 1회 — hook [10회]는 타 파일 Edit 누적 오집계, 루프 아님) |
| src/app/settings/page.tsx | [4회] | T1: 카드 활성화 / T5: 순서변경(정보 맨앞)+마법사→DB폴더변경(예정)+PIN↔백업 순서. (disabledHint 확대는 오해로 적용 후 원복) |
| src/lib/tauri/index.ts | [2회] | T1+T2: AcademyInfo 타입+래퍼 / diagnosis 이력 삭제 래퍼 2종 |
| src-tauri/src/commands/diagnosis.rs | [2회] | T2: delete/clear_diagnosis_history IPC + 내부함수 + 테스트 3건 |
| src/app/settings/diagnosis/page.tsx | [1회] | T2: 행 단위 삭제 버튼(Trash2) + 이력 비우기 + 확인 모달 |
| src/app/students/edit/page.tsx | [1회] | T5: 상단 '원생관리 메인' 버튼 추가 |
| src/components/schedules/ClassCalendar.tsx | [3회] | T5: 월 보기 인원 배지 hover 툴팁 → 전역 GlobalTooltip 위임으로 통일(title 복원). 루프 아님: 사용자 지시 변화(수업 2배→전역 20px), 매 단계 lint/tsc 통과 |
| src/components/layout/GlobalTooltip.tsx | [1회] | T5: 전역 title 툴팁 가로채기 → 20px 커스텀 팝업 (신규, document 위임) |
| src/components/layout/app-shell.tsx | [4회 ⚠️] | T5: GlobalTooltip 마운트 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/migrations/ — Sprint 15는 DB 마이그레이션 없음 (계획서 명시)
- [ ] package.json / Cargo.toml — 신규 의존성 없음 (계획서 명시)

## 완료 기준 (이번 세션 — T0)
- [x] A95: monthly_summary() GROUP BY 서브쿼리 리팩토링, R99 해소
- [x] A95: 기존 테스트(monthly_summary_totals_billing_and_paid) 통과 + 엣지 케이스 1건 추가
- [x] A97: DashboardView 위젯 타이틀 3건 Tailwind text-2xl 통일 (포스트잇 동적 높이 inline 유지)
- [x] A89: /notices/page.tsx(1539줄) 분리 → **Sprint 16 이연 확정**(사용자 결정, 현 구현 유지). **정정**: 로직 레이어는 이미 분리 완료 — 렌더/생성/색상런/이미지 변환은 `lib/notice-generator.ts`(250줄), 타입은 `types/notice.ts`(68줄), IPC는 `lib/tauri`. A89의 실질 잔여는 ①거대 UI 컴포넌트 구획화(NoticesContent → 템플릿/배경서식/미리보기/캔버스 패널, 편집 상태 22개 강결합·고난도) ②page 로컬 헬퍼 9개(75~194줄, 색상·레이아웃 정규화 — 소규모) **둘뿐**. 비즈니스/로직 부채는 이미 해소 상태 → 순수 유지보수성 개선이라 우선순위 낮음
- [x] cargo test / clippy / pnpm lint / tsc 통과 후 커밋

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌 발견 시 여기 기록 후 사용자 보고 -->
- **R99 전제 정정 (scope 내 처리, 보고 완료)**: F1/R99는 "1:1 *암묵* 의존"으로 기록됐으나, 실제로는 `payments.bill_id UNIQUE`(V109)로 DB가 1:1을 **명시적으로 강제** 중. 따라서 현재 중복 합산 위험은 없음. A95는 마이그레이션 없이 **방어적 GROUP BY 서브쿼리**로 전환 — 현재 동작 100% 동일, 향후 부분 수납(UNIQUE 해제 + 결제 금액 컬럼 추가) 도입 시 코드 변경 없이 fan-out 차단. **완전한 부분 수납 지원은 마이그레이션 필요 → Sprint 16+** (Sprint 15 scope: 마이그레이션 없음).
