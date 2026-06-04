---
Sprint: 14  |  Date: 2026-06-02  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/startup.rs | [1회] | T0 — A91 cipher-off 동작 명시 주석 |
| docs/arch/adr-008-optional-pin-gate.md | [1회] | T0 — A91 구현 메모 실제 동작으로 수정 |
| src/app/lock/page.tsx | [1회] | T0 — A93 SplashScreen 이중 표시 통합 |
| src-tauri/migrations/303__create_diagnosis_history.sql | [신규] ✅ | T1 — 자가 진단 이력 테이블. **실제 파일명은 `303__`** (V 접두사 없음 — 기존 301/302 컨벤션). 임시 DB로 001~303 전체 적용 검증 |
| src-tauri/src/commands/diagnosis.rs | [신규] ✅ | T1 — 자가 진단 IPC 4종 + 검사 7종 + 단위테스트 20건. 검사 5는 `makeup_deadline`(계획의 expiry_date 오류 정정), 검사 7은 `bills.adjusted_amount`+카드사 누락 기준(계획의 payments.amount 없음 정정) |
| src-tauri/src/commands/dashboard.rs | [신규] ✅ | T3 — 대시보드 집계 IPC 7종(현황/당일/월요약/출결진행률/알림/메모 get·save) + 알림 5종 + 테스트 9건 |
| src-tauri/src/commands/export.rs | [신규] | T5 — CSV 내보내기 IPC 3종 |
| src-tauri/src/commands/backup.rs | [0회] | T7 — 복원 리허설 IPC 확장 |
| src-tauri/src/commands/mod.rs | [2회] | T1/T3 — pub mod diagnosis/dashboard 등록 (T5 추가 예정) |
| src-tauri/src/lib.rs | [2회] | T1/T3 — invoke_handler 4+7종 등록 (T5/T7 추가 예정) |
| src/lib/tauri/index.ts | [2회] | T2/T4 — 자가 진단 4종 + 대시보드 7종 IPC 래퍼 (T6/T7 추가 예정) |
| src/types/diagnosis.ts | [신규] ✅ | T2 — DiagnosisIssue/Result/HistoryRow |
| src/app/settings/diagnosis/page.tsx | [신규] ✅ | T2 — 자가 진단 화면(신규 라우트). **계획의 'page.tsx 인라인 섹션' 대신 전용 라우트로 구현** — hours/codes/pin 등 기존 설정 라우트 패턴과 일관. 실행 버튼 + 12개월 이력 + 결과 상세 + 이동 링크 |
| src/app/settings/page.tsx | [1회] ✅ | T2 — '데이터 자가 진단' 카드 추가 (T6/T7 데이터관리/백업 카드 추가 예정) |
| src/components/layout/app-shell.tsx | [1회] ✅ | T2 — 자동 진단 트리거(세션 1회, unlock 후 백그라운드, AC-6.6-1/R97) |
| src/types/dashboard.ts | [신규] ✅ | T4 — Overview/TodaySchedule/MonthlySummary/Alert/Progress |
| src/app/page.tsx | [1회] ✅ | T4 — unlock 시 placeholder → `<DashboardView/>` 교체 (인증 게이트 로직 유지) |
| src/components/dashboard/ | [신규] ✅ | T4 — DashboardView(위젯 6 + 알림 5 + 메모) + charts.tsx(recharts, ssr:false 동적 import) |
| src/lib/menu-config.ts | [1회] ✅ | T4 — 대시보드 disabledHint 제거(F3 해소) |
| package.json | [신규] ✅ | T4 — recharts **3.8.1 설치** (계획 ^2.x → 최신 3.x. v3 API 사용, dynamic import ssr:false R96) |
| src/types/export.ts | [신규] | T6 |
| src/components/layout/sidebar.tsx | [1회] | scope 외 추가 (2026-06-04, 사용자 요청) — "종료" 클릭 시 프로그램 종료 확인 다이얼로그 추가 (PRD §5.7, 기존 AlertDialog 재사용, 의존성·DB 변경 없음) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD (hook 차단)
- [ ] SETUP.sh (hook 차단)
- [ ] docs/harness-engineering/

## 신규 의존성 (사전 승인 완료)
- recharts ^2.x (대시보드 차트, 사용자 승인 2026-06-02). dynamic import 로 대시보드 라우트 한정 로드(R96).

## 신규 마이그레이션
- V303 diagnosis_history (300번대 도메인 확장 블록 연속). 추가 후 .sqlx 캐시 갱신 + CLAUDE.md 현황 갱신(A92).

## 완료 기준 (sprint14.md DoD 요약)
- [x] T0 carry-over (A91/A93) — startup cipher-off 주석 + ADR-008 정정 + /lock 단일 로딩. (R93=CLAUDE.md V302 sprint-review 기반영, R94=T8, F3=T4)
- [ ] T1/T2 자가 진단 (백엔드 7종 + 프론트 + 자동 트리거)
- [ ] T3/T4 대시보드 (집계 IPC 6종 + 위젯 6 + 알림 5)
- [ ] T5/T6 내보내기 CSV 3종
- [ ] T7 복원 리허설
- [ ] T8 통합 검증 (test/clippy/cipher/lint/tsc/build + .sqlx + CLAUDE.md V303)

## 세션 체크포인트 / 다음 진입점
- **세션 #1 (2026-06-02)**: 계획 + T0 완료, 커밋됨. 자동검증(clippy/lint/tsc) 통과.
- **세션 #2 (2026-06-04)**: T1(자가 진단 백엔드) 완료. 마이그레이션 303 + diagnosis.rs(IPC 4 + 검사 7 + 테스트 20). `cargo test --lib` 334 passed / clippy clean.
- **세션 #2 (2026-06-04) 이어서**: T2(자가 진단 프론트엔드) 완료. 타입 + IPC 래퍼 4종 + `/settings/diagnosis` 라우트 + 설정 카드 + AppShell 자동 트리거. `pnpm lint`/`tsc`/`build`(export 3/3, diagnosis.html 생성) 통과.
- **세션 #2 (2026-06-04) 이어서**: T3(대시보드 집계 IPC) 완료. `dashboard.rs` — IPC 7종 + 알림 5종 + 테스트 9건. `cargo test --lib` 343 passed / clippy clean.
  - **정의 결정(사용자 검증 후 조정 가능)**: 출결 진행률 = 당월 1일~오늘 중 현행 스케줄 요일 수업일의 정규출결 미기록 일자(휴원/방학 제외 미반영) / 분기 = 학사력 3·6·9·12월 시작 / 보강 소멸 임박 = makeup_deadline(YYYY-M)이 당월(월 단위 근사).
- **세션 #2 (2026-06-04) 이어서**: T4(대시보드 위젯 UI) 완료. dashboard.ts + 래퍼 7종 + `/` 대시보드 교체 + 위젯 6 + 알림 5 + 메모. shadcn 내장 차트 부재 확인 → recharts 3.8.1 설치(ssr:false 동적 import). F3 해소. `lint`/`tsc`/`build`(export 3/3, index.html) 통과.
- **다음 진입점 = T5 (데이터 내보내기 백엔드)**: `export.rs` 신규 — CSV IPC 3종(원생/출결/청구-수납) + BOM(UTF-8) + 단위 테스트 6. (Excel/비번보호는 Sprint 15 이연)
  - `.sqlx` 캐시: 런타임 `query()` 패턴이라 갱신 불필요. T8 cipher 빌드 점검 시 일괄 확인.
  - **사용자 검증 대기**: T2 자가 진단(A/B/C/D) + T4 대시보드(위젯 6 렌더링/차트/알림 클릭 이동/출결 진행률 금요일 강조/메모 자동저장). 모두 `/restart` 후 실앱 시각 검증.

## 발견된 이슈
- **T2 검증 중 자가진단 check 1(보강필요시간 음수) 오탐 수정** (2026-06-04, 사용자 보고 — 성춘향 케이스): 정상 매칭된 결석↔보강 쌍이 음수로 오탐. 원인은 결석 합산이 `status='absent'`만 세고 `makeup_done`을 누락 → 보강완료분만 차감돼 음수. 앱 SSOT(`attendance.rs` 보강필요시간 정의)에 맞춰 결석 대상을 **`absent`+`makeup_done`(소멸 `makeup_expired`은 면제로 제외)**로 변경. 회귀 테스트 2건 추가(성춘향 매칭 쌍 / 소멸 제외). 사용자 DB의 오탐 이력 1건(id=1 auto, 수정 전 생성)은 일회성 수동 삭제. **자가 진단 이력 수동 삭제 기능(B안)은 Sprint 15로 이연**(ROADMAP 기록) — 자동 삭제는 감사로그 훼손 우려로 미도입 결정.
- **T4 검증 중 대시보드 추가 요청** (2026-06-04, 사용자): 당일 수업 박스 높이 축소(max-h-40 스크롤) + 빈 공간에 **교습소 월별 청구총액 추이 그래프** 위젯 신설 (마지막 청구월 기준 최근 12개월, 빈 달 0). 백엔드 `get_billing_trend` IPC + 테스트 2건, charts.tsx 라인차트 추가. 우측 열을 당일수업+추이 스택으로 재배치.
- **T4 검증 중 출결 진행률 공휴일 오탐 수정** (2026-06-04, 사용자 보고): 선거 공휴일(6/3) 등 비수업일이 "미입력"으로 잘못 노출됨. 출결 진행률의 "수업일"이 단순 요일 매칭이라 공휴일/방학/휴원일을 제외하지 않은 게 원인. → `attendance::load_off_dates` 와 동일 기준(allows_regular_class=0 학사일정 기간 확장)으로 비수업일을 제외하도록 `dashboard::attendance_progress` 수정 + 회귀 테스트 추가. 메모 위젯도 대시보드 상단으로 이동(사용자 요청).
- **계획(sprint14.md) 컬럼명 2건 정정** (T1 착수 시 실제 스키마 대조): (1) 검사 5 소멸기한 컬럼 `expiry_date`→`makeup_deadline`(V106), (2) 검사 7 `payments.amount` 미존재 → 결제수단/카드사 누락 기준으로 재정의. sprint14.md 본문은 sprint-close 시 정정 반영 권장.
- **마이그레이션 파일명**: 문서의 "V303"은 약칭, 실제 파일은 `303__`(기존 301/302 무접두 컨벤션 일치).
