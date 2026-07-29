---
Sprint: 24  |  Date: 2026-07-29  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/attendance.rs | [0회] | T0(A114 이력패턴), T1(A1 apply_schedule_change 태깅), T3(B1 move_attendance), T4(B2 build_day_schedules), T9(회귀 테스트) |
| src-tauri/src/commands/makeup.rs | [0회] | T2(A2 create_makeup 태깅), T4(B5 get_makeup_eligible_dates), T9(테스트) |
| src-tauri/src/commands/students.rs | [0회] | T4(B6 reinstate_student) |
| src-tauri/src/commands/notice.rs | [0회] | T4(B3 get_notice_month_info 보강데이 필터) |
| src-tauri/src/commands/dashboard.rs | [0회] | T4(B4 알림 카운트) |
| src-tauri/src/commands/academic.rs | [0회] | T5(B7 지난달 가드 — 설계 판단 후 조건부) |
| src-tauri/src/commands/diagnosis.rs | [0회] | T8(D1 year_month 불변식 검사 8번 + 진단 파라미터 점검) |
| src/components/attendance/MakeupRegisterDialog.tsx | [0회] | T6(B8 deadline 비교 prop 사용) |
| src-tauri/migrations/313__fix_year_month_tagging.sql | [0회] | T7(E1 전수 보정 — 신규) |
| docs/arch/adr-013-academic-guard-scope.md | [0회] | T5 결과에 따라 조건부 신규 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/src/commands/billing.rs, fees.rs — 감사 결과 SAFE(영향 0원). 코드 변경 금지, 회귀 테스트로만 확인
- [ ] src-tauri/src/commands/calendar.rs, schedules.rs, expiration.rs — 순수 소비/라벨 산술 SAFE. year_month 파생 없음
- [ ] src-tauri/src/lib.rs — 신규 커맨드 없으면 미수정 (D1 검사는 기존 run_diagnosis 내부 확장)

## 완료 기준 (이번 세션)
- ✅ T0: sync_single_date 이력 패턴 통일 (A114) — load_schedule_slices/minutes_for_date 공유 헬퍼로 통일
- ✅ T1: A1 apply_schedule_change 교습기간 year_month 태깅 + 경계일 테스트
- ✅ T2: A2 create_makeup 교습기간 year_month 태깅 + 경계일 테스트
- ✅ T3: B1 move_attendance 동월 판정을 study_periods 기준 + year_month 갱신
- ✅ T4: B2~B6 일괄 (달력월 → 교습기간 범위)
- ✅ T5: B7 학사일정 가드 — 설계 결정: 순수 달력-날짜 엔티티라 달력월 기준 유지(주석 문서화), ADR 불요
- ✅ T6: B8 프론트 deadline prop 교체
- ✅ T7: V313 전수 보정 마이그레이션 (멱등·트랜잭션) — dev DB 적용+멱등성 확인
- ✅ T8: D1 자가진단 검사 8번(year_month 불변식) + active_year_month 로 당월 파라미터 교습기간 기준화
- ✅ T9: D2 다월 회귀 테스트 (attendance 9 + makeup 3 + diagnosis 2 = 14건 신규)
- ✅ T10: 통합 검증 — cargo test 491 / clippy clean / cipher OK / lint / tsc / build 전수 통과

## 실제 수정 파일 (scope 대비)
scope 선언 8개 파일 + V313 신규 = 계획 대비 일치. B7은 ADR 미생성(설계상 불요).


## 발견된 이슈
(없음 — Step-back 발생 시 여기 기록)
