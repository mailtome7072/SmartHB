---
Sprint: 21  |  Date: 2026-07-19  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| DEPLOY.md | [1회] | T0: 인쇄 미리보기 확인 항목 추가(A122) |
| src-tauri/src/commands/attendance.rs | [2회] | T1: sync_single_date 태깅을 교습기간 ym으로 통일 + 테스트 |
| src/app/attendance/page.tsx | [2회] | T2: 교습기간 range를 그리드/다이얼로그에 전달 |
| src/components/attendance/AttendanceGrid.tsx | [12회 ⚠️] | T2: 컬럼 daysOfMonth→교습기간 범위, ISO 매핑, 헤더 월표기 |
| src/components/attendance/MoveAttendanceDialog.tsx | [2회] | T3: 달력월 가정 제거, 교습기간 범위 기준 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/migrations/ — DB 마이그레이션 없음(V310 유지)
- [ ] 백엔드 IPC 신규 추가 없음 — 교습기간 범위는 기존 listStudyPeriods 데이터 재사용

## 완료 기준 (이번 세션)
- [ ] T0: DEPLOY.md 인쇄 미리보기 확인 항목 추가
- [ ] T1: sync_single_date 교습기간 ym 태깅 통일 + 단위 테스트 3건(다월/범위밖/단일월)
- [ ] T2: AttendanceGrid 컬럼 교습기간 범위 + 전체 ISO 매핑 + 월경계 헤더 + 폴백
- [ ] T3: MoveAttendanceDialog 교습기간 범위 대응
- [ ] T4: 통합 검증(cargo test/clippy/cipher/lint/tsc/build) + 시각검증 안내(단일월 회귀/다월 전 일자)

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌/설계 오류 발견 시 여기에 기록 후 사용자 보고 -->
1. **[T3 접근 조정]** 계획서 T3는 MoveAttendanceDialog를 "교습기간 범위" 기준으로 바꾸려 했으나,
   백엔드 `move_attendance_impl`이 **달력월 동월 한정**(cross-month 이동 차단, 다른 달은 보강 기능)
   임을 확인. 교습기간 범위(다월)로 달력을 그리면 백엔드가 거부할 날짜를 보여주게 됨 →
   대신 **출발일(fromDate)의 달력월** 기준으로 달력을 그리도록 수정(이웃 달 출발일도 그 달 안에서
   이동). 백엔드 제약과 정합하며 계획 의도(달력월 고정 가정 제거)도 충족.
