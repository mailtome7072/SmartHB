---
Sprint: 14  |  Date: 2026-06-06  |  Session: #4
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/startup.rs | [1회] | T0 — A91 cipher-off 동작 명시 주석 |
| docs/arch/adr-008-optional-pin-gate.md | [1회] | T0 — A91 구현 메모 실제 동작으로 수정 |
| src/app/lock/page.tsx | [9회 ⚠️] | T0 — A93 SplashScreen 이중 표시 통합 |
| src-tauri/migrations/303__create_diagnosis_history.sql | [신규] ✅ | T1 — 자가 진단 이력 테이블. **실제 파일명은 `303__`** (V 접두사 없음 — 기존 301/302 컨벤션). 임시 DB로 001~303 전체 적용 검증 |
| src-tauri/src/commands/diagnosis.rs | [신규] ✅ | T1 — 자가 진단 IPC 4종 + 검사 7종 + 단위테스트 20건. 검사 5는 `makeup_deadline`(계획의 expiry_date 오류 정정), 검사 7은 `bills.adjusted_amount`+카드사 누락 기준(계획의 payments.amount 없음 정정) |
| src-tauri/src/commands/dashboard.rs | [신규] ✅ | T3 — 대시보드 집계 IPC 7종(현황/당일/월요약/출결진행률/알림/메모 get·save) + 알림 5종 + 테스트 9건 |
| src-tauri/src/commands/export.rs | [신규] ✅ | T5 — CSV 내보내기 IPC 3종(원생/출결/청구-수납) + BOM(UTF-8) + 라벨변환 + 단위테스트 9건. 출결은 정규+보강 UNION(구분 컬럼), 청구는 청구상태 컬럼 추가, year_month는 Option(None=전체) |
| src-tauri/src/commands/backup.rs | [9회 ⚠️] | T7 — 복원 리허설 IPC 확장 (run_backup_rehearsal + RehearsalResult/TableCount). list_backups는 기존 재사용. cipher off는 평문 백업만 리허설(R98, apply_rehearsal_key 게이트) |
| src-tauri/src/commands/mod.rs | [3회] | T1/T3/T5 — pub mod diagnosis/dashboard/export 등록 (backup은 기존 등록) |
| src-tauri/src/lib.rs | [6회 ⚠️] | T1/T3/T5 + T7 — invoke_handler 4+7+3종 + run_backup_rehearsal 등록 |
| src/lib/tauri/index.ts | [10회 ⚠️] | T2/T4/T6 + T7 — 진단4 + 대시보드7 + 내보내기3 래퍼 + runBackupRehearsal (listBackups는 기존 재사용) |
| src/types/diagnosis.ts | [신규] ✅ | T2 — DiagnosisIssue/Result/HistoryRow |
| src/app/settings/diagnosis/page.tsx | [신규] ✅ | T2 — 자가 진단 화면(신규 라우트). **계획의 'page.tsx 인라인 섹션' 대신 전용 라우트로 구현** — hours/codes/pin 등 기존 설정 라우트 패턴과 일관. 실행 버튼 + 12개월 이력 + 결과 상세 + 이동 링크 |
| src/app/settings/page.tsx | [3회] | T2 '데이터 자가 진단' + T6 '데이터 내보내기' + T7 '백업 관리' 카드 추가 |
| src/components/layout/app-shell.tsx | [1회] ✅ | T2 — 자동 진단 트리거(세션 1회, unlock 후 백그라운드, AC-6.6-1/R97) |
| src/types/dashboard.ts | [신규] ✅ | T4 — Overview/TodaySchedule/MonthlySummary/Alert/Progress |
| src/app/page.tsx | [1회] ✅ | T4 — unlock 시 placeholder → `<DashboardView/>` 교체 (인증 게이트 로직 유지) |
| src/components/dashboard/ | [신규] ✅ | T4 — DashboardView(위젯 6 + 알림 5 + 메모) + charts.tsx(recharts, ssr:false 동적 import) |
| src/lib/menu-config.ts | [1회] ✅ | T4 — 대시보드 disabledHint 제거(F3 해소) |
| package.json | [신규] ✅ | T4 — recharts **3.8.1 설치** (계획 ^2.x → 최신 3.x. v3 API 사용, dynamic import ssr:false R96) |
| src/types/export.ts | [신규] ✅ | T6 — ExportResult/ExportTarget |
| src/app/settings/data/page.tsx | [신규] ✅ | T6 — 데이터 내보내기 화면(신규 라우트, diagnosis와 동일 패턴). 대상 3종 선택 + 기간(전체/특정월, 출결·청구만) + showCsvSaveDialog + 결과 배너 |
| src/components/layout/sidebar.tsx | [1회] | scope 외 추가 (2026-06-04, 사용자 요청) — "종료" 클릭 시 프로그램 종료 확인 다이얼로그 추가 (PRD §5.7, 기존 AlertDialog 재사용, 의존성·DB 변경 없음) |
| src/types/index.ts | [1회] | T7 — RehearsalResult/TableCount 타입 추가 (BackupMetadata 인접) |
| src/app/settings/backup/page.tsx | [신규] | T7 — 백업 관리 화면(신규 라우트, diagnosis/data 동일 패턴). 백업 목록 + 복원 리허설 버튼 + 결과 패널 + 운영데이터 무영향 안내 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD (hook 차단)
- [ ] SETUP.sh (hook 차단)
- [ ] docs/harness-engineering/

## 신규 의존성 (사전 승인 완료)
- recharts ^2.x (대시보드 차트, 사용자 승인 2026-06-02). dynamic import 로 대시보드 라우트 한정 로드(R96).
- **rust_xlsxwriter 0.95** (내보내기 .xlsx, 사용자 승인 2026-06-05). 순수 Rust(Perl/OpenSSL 불필요 — Win/Mac 안전). CSV→엑셀 전환으로 정렬·천단위콤마·우측정렬·컬럼너비 서식 지원. Sprint 15 예정분(Excel)을 14로 당김(비번보호만 15 잔류).

## 신규 마이그레이션
- V303 diagnosis_history (300번대 도메인 확장 블록 연속). 추가 후 .sqlx 캐시 갱신 + CLAUDE.md 현황 갱신(A92).
- V304 expire_withdrawn_pending_makeup (퇴교생 미보강 결석 백필, 2026-06-05 버그픽스).
- V305 add_birth_date_to_students (원생 생년월일 nullable 컬럼, 2026-06-06 사용자 요청). 런타임 query 패턴이라 .sqlx 불필요. CLAUDE.md 현황 V305 갱신 완료.

## 완료 기준 (sprint14.md DoD 요약)
- [x] T0 carry-over (A91/A93) — startup cipher-off 주석 + ADR-008 정정 + /lock 단일 로딩. (R93=CLAUDE.md V302 sprint-review 기반영, R94=T8, F3=T4)
- [ ] T1/T2 자가 진단 (백엔드 7종 + 프론트 + 자동 트리거)
- [ ] T3/T4 대시보드 (집계 IPC 6종 + 위젯 6 + 알림 5)
- [ ] T5/T6 내보내기 CSV 3종
- [x] T7 복원 리허설 — run_backup_rehearsal(임시복사→integrity_check→행수6종→폐기, cipher off 평문 R98) + /settings/backup 라우트 + 카드. 테스트 4건. cargo test 365/clippy/lint/tsc/build 통과. 커밋 `1182111`.
- [x] T8 통합 검증 (자동) — cargo test 365 passed / clippy clean / `cargo check --features cipher` clean / pnpm lint·tsc·build(/settings/backup 2.5kB) / `.sqlx` 런타임 query 패턴이라 갱신 불필요 / CLAUDE.md 마이그레이션 현황 이미 V304. **시각 검증(복원 리허설 실행)은 사용자 대기** — cipher off 개발빌드는 실제 백업 파일이 없어 빈 목록 표시(평문 백업 수동 배치 시에만 리허설 가능).

## 세션 체크포인트 / 다음 진입점
- **세션 #1 (2026-06-02)**: 계획 + T0 완료, 커밋됨. 자동검증(clippy/lint/tsc) 통과.
- **세션 #2 (2026-06-04)**: T1(자가 진단 백엔드) 완료. 마이그레이션 303 + diagnosis.rs(IPC 4 + 검사 7 + 테스트 20). `cargo test --lib` 334 passed / clippy clean.
- **세션 #2 (2026-06-04) 이어서**: T2(자가 진단 프론트엔드) 완료. 타입 + IPC 래퍼 4종 + `/settings/diagnosis` 라우트 + 설정 카드 + AppShell 자동 트리거. `pnpm lint`/`tsc`/`build`(export 3/3, diagnosis.html 생성) 통과.
- **세션 #2 (2026-06-04) 이어서**: T3(대시보드 집계 IPC) 완료. `dashboard.rs` — IPC 7종 + 알림 5종 + 테스트 9건. `cargo test --lib` 343 passed / clippy clean.
  - **정의 결정(사용자 검증 후 조정 가능)**: 출결 진행률 = 당월 1일~오늘 중 현행 스케줄 요일 수업일의 정규출결 미기록 일자(휴원/방학 제외 미반영) / 분기 = 학사력 3·6·9·12월 시작 / 보강 소멸 임박 = makeup_deadline(YYYY-M)이 당월(월 단위 근사).
- **세션 #2 (2026-06-04) 이어서**: T4(대시보드 위젯 UI) 완료. dashboard.ts + 래퍼 7종 + `/` 대시보드 교체 + 위젯 6 + 알림 5 + 메모. shadcn 내장 차트 부재 확인 → recharts 3.8.1 설치(ssr:false 동적 import). F3 해소. `lint`/`tsc`/`build`(export 3/3, index.html) 통과.
- **세션 #3 (2026-06-05)**: T5(데이터 내보내기 백엔드) 완료. `export.rs` 신규 — IPC 3종 + CSV 유틸(csv_field/csv_row/with_bom/write_csv) + 라벨변환 4종 + 단위테스트 9건(계획 6 + escape/BOM/필터 보강). `cargo test --lib` 356 passed / clippy clean. simplify 4-agent 검토 결과 변경 없음(제네릭 래퍼·enum Display 승격은 스코프 밖/이득 미미로 skip).
  - `.sqlx` 캐시: 런타임 `query()` 패턴이라 갱신 불필요. T8 cipher 빌드 점검 시 일괄 확인.
- **세션 #3 (2026-06-05) 이어서**: T6(데이터 내보내기 프론트) 완료. `types/export.ts` + IPC 래퍼 3종 + `showCsvSaveDialog` + `/settings/data` 신규 라우트(diagnosis 패턴) + 설정 카드. `pnpm lint`/`tsc`/`build`(export 성공, `/settings/data` 1.97kB 생성) 통과.
  - **사용자 검증 완료(2026-06-05)**: T2 자가 진단(A/B/C/D) + T4 대시보드(위젯 6/차트/알림 이동/출결 진행률/메모 자동저장) 실앱 시각 검증 통과.
- **세션 #4 (2026-06-06, 집 Mac)**: T7(복원 리허설) + T8(통합 검증 자동) 완료. `backup.rs` run_backup_rehearsal(임시디렉토리 복사→read-only sqlx 풀→integrity_check→주요 6종 행수→사본 폐기, cipher 게이트 apply_rehearsal_key) + RehearsalResult/TableCount + `/settings/backup` 라우트 + 설정 카드 + 타입/래퍼. 테스트 4건(정상/손상/없는파일/스키마동기화가드). simplify 적용: Result<Vec,String> 리팩터(반복 튜플 4개 제거) + staleness 가드 테스트. 커밋 `1182111`.
  - **T8 자동 검증 통과**: cargo test 365 / clippy / `cargo check --features cipher` / lint / tsc / build / `.sqlx` 불필요 / CLAUDE.md V304 기반영.
  - **사용자 시각 검증 대기 (sprint-close 전)**: ① **T7 복원 리허설** — cipher off 개발빌드는 백업 파일이 없어 `/settings/backup` 목록이 비어있음(빈 상태 UI만 확인 가능). 실제 리허설은 평문 SQLite 파일을 `SmartHB-data/backup/{layer}/`에 수동 배치하거나 cipher on 빌드에서 확인. ② **T6 내보내기** — `/settings/data` 엑셀 저장(전체/월). `pnpm tauri:dev`로 확인.
  - **다음 진입점 = sprint-close** (구현 전부 완료). sprint14.md 본문 정정 목록은 아래 "마무리 후" 참조.

## 발견된 이슈
- **T2 검증 중 자가진단 check 1(보강필요시간 음수) 오탐 수정** (2026-06-04, 사용자 보고 — 성춘향 케이스): 정상 매칭된 결석↔보강 쌍이 음수로 오탐. 원인은 결석 합산이 `status='absent'`만 세고 `makeup_done`을 누락 → 보강완료분만 차감돼 음수. 앱 SSOT(`attendance.rs` 보강필요시간 정의)에 맞춰 결석 대상을 **`absent`+`makeup_done`(소멸 `makeup_expired`은 면제로 제외)**로 변경. 회귀 테스트 2건 추가(성춘향 매칭 쌍 / 소멸 제외). 사용자 DB의 오탐 이력 1건(id=1 auto, 수정 전 생성)은 일회성 수동 삭제. **자가 진단 이력 수동 삭제 기능(B안)은 Sprint 15로 이연**(ROADMAP 기록) — 자동 삭제는 감사로그 훼손 우려로 미도입 결정.
- **T4 검증 중 대시보드 추가 요청** (2026-06-04, 사용자): 당일 수업 박스 높이 축소(max-h-40 스크롤) + 빈 공간에 **교습소 월별 청구총액 추이 그래프** 위젯 신설 (마지막 청구월 기준 최근 12개월, 빈 달 0). 백엔드 `get_billing_trend` IPC + 테스트 2건, charts.tsx 라인차트 추가. 우측 열을 당일수업+추이 스택으로 재배치.
- **T4 검증 중 출결 진행률 공휴일 오탐 수정** (2026-06-04, 사용자 보고): 선거 공휴일(6/3) 등 비수업일이 "미입력"으로 잘못 노출됨. 출결 진행률의 "수업일"이 단순 요일 매칭이라 공휴일/방학/휴원일을 제외하지 않은 게 원인. → `attendance::load_off_dates` 와 동일 기준(allows_regular_class=0 학사일정 기간 확장)으로 비수업일을 제외하도록 `dashboard::attendance_progress` 수정 + 회귀 테스트 추가. 메모 위젯도 대시보드 상단으로 이동(사용자 요청).
- **계획(sprint14.md) 컬럼명 2건 정정** (T1 착수 시 실제 스키마 대조): (1) 검사 5 소멸기한 컬럼 `expiry_date`→`makeup_deadline`(V106), (2) 검사 7 `payments.amount` 미존재 → 결제수단/카드사 누락 기준으로 재정의. sprint14.md 본문은 sprint-close 시 정정 반영 권장.
- **마이그레이션 파일명**: 문서의 "V303"은 약칭, 실제 파일은 `303__`(기존 301/302 무접두 컨벤션 일치).
- **보강 정합성 버그픽스 2건 + 호버 힌트** (2026-06-05, T6 검증 중 사용자 보고 — 실DB 데이터로 근본원인 확정): **버그A 홍길동(퇴교생)** 미보강 결석이 보강소멸 임박 알림에 노출 — 원인①퇴교 시 자동소멸 누락(update_student/withdraw_date 직접설정 경로가 process_withdrawal_makeup 우회) ②알림 쿼리에 퇴교 필터 없음. **버그B 고길동** 6월 보강필요시간 0 — `compute_summary`가 `year_month=조회월` 제약으로 이월 결석 누락. **수정**(사용자 결정: 이월누적 + 쿼리제외+백필): (1) `attendance.rs::compute_summary` 보강필요시간을 **이월 누적**(소멸기한 ≥ 조회월 + 퇴교 제외, earliest_pending과 정합)으로, (2) `dashboard.rs` 보강소멸 알림 **퇴교생 제외**, (3) **신규 마이그레이션 `304__expire_withdrawn_pending_makeup.sql`**(기존 퇴교생 미보강 결석 일괄 makeup_expired 백필 — 멱등), (4) **보강필요/보강완료 셀 hover 힌트**(기존 title 방식): 보강필요=이월 결석 내역(백엔드 `pending_absences` 필드 신설), 보강완료=당월 보강출결 내역. 회귀테스트 3건. **수정파일**: attendance.rs(struct+compute_summary+fetch_pending_absences+grid), dashboard.rs, migration 304, types/attendance.ts, AttendanceGrid.tsx, CLAUDE.md(V304). 실DB 검증: 홍길동→makeup_expired, 알림 4→2건, 고길동 6월 120분. cargo test 361 / clippy / lint / tsc 통과.
- **T4 대시보드 추가 개선 2건** (2026-06-05, T6 검증 중 사용자 요청): (1) **당일 수업 표현 변경** — `16:00:00 성춘향, 이자영 (2명)` → `pm.4시 2명 - 성춘향, 이자영` (DashboardView `formatSlotTime`, 12시간제 am/pm + 분 있으면 표기). (2) **메모를 포스트잇 3장**으로 변경 — 한 행에 flex-1 가변 너비, 각 높이 드래그 조정(resize-y + ResizeObserver 디바운스 저장, reload 복원), 박스 높이=가장 큰 포스트잇(items-start). 백엔드: dashboard.rs 메모를 단일→3슬롯(`dashboard_memo_{i}` 내용 + `_h` 높이, 슬롯0은 레거시 단일메모 흡수), IPC `get_dashboard_memos`+`save_dashboard_memo(index,content,height)`, 테스트 3건. lib.rs 등록 갱신. **수정 파일**: dashboard.rs, lib.rs, types/dashboard.ts, lib/tauri/index.ts, components/dashboard/DashboardView.tsx.
- **내보내기 CSV→엑셀(.xlsx) 전환** (2026-06-05, 사용자 요청 — 정렬/우측정렬/컬럼너비는 CSV 불가능): `export.rs` 전면 재작성(SheetData/Cell 구조로 테스트 분리 + write_xlsx 서식 적용). 요청 충족: 원생 **일련번호 오름차순**, 금전(교습비/청구액/할인액/최종액) **천단위 콤마 숫자+우측정렬**, 그외 좌측정렬, **autofit 컬럼너비**, 수업시간 **'시간' 단위 통일**(분→시간). 신규 의존성 rust_xlsxwriter. 프론트: showCsvSaveDialog→showXlsxSaveDialog(.xlsx 필터), 파일명 .xlsx, 카피 수정. 테스트 6건 재작성(정렬순서/금전셀/시간환산/파일생성). **수정파일**: export.rs, lib/tauri/index.ts, app/settings/data/page.tsx, app/settings/page.tsx, Cargo.toml. cargo test 358 / clippy / lint / tsc 통과.
- **T5 계획 대비 정제 3건** (2026-06-05, 구현 시 결정 — sprint14.md 본문은 sprint-close 시 반영 권장): (1) 출결 CSV는 단순 "보강여부" 플래그 대신 **정규+보강 출결 UNION + `구분` 컬럼** — 보강 세션도 실제 출결 기록이라 누락 방지. (2) 청구 CSV에 **`청구상태`(미확정/확정) 컬럼 추가**. (3) 출결/청구 `year_month`는 **`Option<String>`**(None=전체) — 계획의 단월/전체 요구 충족. 교습비는 `standard_fees`(V201 보정값, 주4시간=200000) LEFT JOIN.
- **Node 25 dev 서버 크래시 회피** (2026-06-06, T7/T6 시각 검증 시도 중 — 집 Mac Homebrew node@25): 메뉴 이동 불가 증상의 원인은 코드가 아니라 **Node 25 V8이 Next.js 15.3.6 webpack 캐시 직렬화 중 `Lazy deopt after a fast API call ...` abort로 dev 서버 프로세스를 크래시**(`SerializerMiddleware._serialize`→`node:buffer.byteLength`). 죽은 뒤 미컴파일 라우트 접근 불가. **조치(사용자 선택 A)**: `next.config.ts`에 dev 전용 `config.cache=false`(webpack 파일시스템 캐시 비활성화) — 크래시 경로 제거. production `output:'export'` 빌드·Windows PC 무영향, dev 콜드 리컴파일만 약간 느려짐. 11개 라우트 전수 200·크래시 0건 재현 확인. NODE_OPTIONS는 `--no-turbo-fast-api-calls` 미허용이라 config 경로 채택. (정석은 Node LTS 전환이나 이 Mac엔 node@25만 설치·버전매니저 없음 → 머신 전역 영향 회피).
- **자가진단 이력 중복 누적 수정** (2026-06-06, T7 검증 중 사용자 보고 — 실DB 3행 확인: 06-06 auto/manual/manual 동일결과): `run_diagnosis`가 실행마다 무조건 INSERT → 클릭 반복·자동+수동 동일결과가 중복 행으로 누적. **수정(사용자 선택: 변경 시에만 기록)**: `diagnosis.rs::run_and_record`에 `is_same_as_latest`(최신 행 issues_found+details JSON 비교) 가드 추가 — 직전과 동일하면 INSERT·12개월 정리 스킵(화면엔 최신 결과 반환). 결과 변동 시에만 새 이력. 마이그레이션 불필요. 회귀 테스트 2건(동일3회→1건 / 변경시→2건) + 기존 `get_latest_returns_most_recent` 데이터 변경 보정. **부수효과(허용)**: 수동 후 동일결과 auto는 'auto' 행 미생성 → auto_needed가 그달 매 세션 true(R97로 세션1회 한정, 검사 비용 경미). 실DB 중복 2건(id 2,3) 일회성 삭제, id1(auto) 유지. cargo test 365 / clippy 통과. **수정파일**: diagnosis.rs.
- **출결 입력 진행률 위젯·알림 제거** (2026-06-06, T4 검증 중 사용자 보고 — "항상 100% 아닌가?"): 출결은 `attendance::generate_attendances`가 교습기간 전체의 모든 수업일을 `status='present'` 기본값으로 **일괄 INSERT**하는 모델 → "행 존재=기록됨" 기준 진행률은 출결 생성 후 **항상 100%**(미입력 0). "미입력" 상태가 존재하지 않아 per-day 진행률은 측정 대상 자체가 없음. 의미 있는 신호("당월 출결 생성 여부")는 이미 자가진단 검사2/별도 경로가 커버. **조치(사용자 선택: 위젯+알림 제거)**: ① dashboard.rs `attendance_progress`/`get_attendance_progress`/`AttendanceProgress`/`last_day_of_month`/`off_dates`(셋 다 progress 전용)/`attendance_missing` 알림 + 테스트 2건 제거, 알림 번호 재정렬, ② lib.rs IPC 등록 제거, ③ types/dashboard.ts `AttendanceProgress` + lib/tauri `getAttendanceProgress` 제거, ④ DashboardView 위젯·쿼리·`ProgressBody`·`attendance_missing` 라우트 제거 + **"월 요약"을 전체 너비로 재배치**(2col 그리드 빈칸 해소, stat 그리드 `lg:grid-cols-6`). `attendance_recorded_days`(월요약 필드)는 별도 위젯이라 유지. cargo test 365 / clippy / lint / tsc / build 통과. **수정파일**: dashboard.rs, lib.rs, types/dashboard.ts, lib/tauri/index.ts, components/dashboard/DashboardView.tsx.
- **대시보드 월 요약 보강 3건** (2026-06-06, 사용자 요청): ① **이전/다음 월 전환** — 월 요약을 자체 월 상태 `MonthlySummaryWidget`으로 분리 + `‹`/`이번 달`/`›` 버튼(`shiftMonth` 헬퍼, Widget에 `action` 슬롯 추가). ② 버튼을 제목 **앞(왼쪽)**으로 배치. ③ 월 요약을 **메모 아래**로 이동(메모→월요약→알림→위젯들). **수정파일**: components/dashboard/DashboardView.tsx. 커밋 `4b4c274`/`3537413`/`aeef088`.
- **Node 25 dev CSS 404(화면 깨짐) 복구** (2026-06-06): 빠른 편집→HMR 반복 + dev 캐시 비활성화로 `layout.css` 청크 해시 어긋나 stale 404 → 무스타일 화면. `.next` 전체 삭제 + dev 클린 재기동으로 해소(코드 무관, `pnpm build`는 정상). 재발 시 동일 조치.
- **자가진단 해결 항목 자동 재검증·정리** (2026-06-06, 사용자 요청 — 버튼 아닌 자동): 수동/자동 실행 시마다 이전 진단결과 항목이 현재도 검출되는지 재검증해 **해결된(미검출) 항목을 각 이력에서 자동 제거**, 남은 항목 메시지 최신화, **이상이 있던 이력의 모든 항목이 해결되면 그 이력 삭제**. **구현**: `diagnosis.rs`에 `issue_identity`(check_id+target_table+target_id, 메시지 제외)·`reconcile_resolved_issues`(run_and_record 시작부 호출) 추가. 항목 식별자로 현재 검출 집합과 대조. 0건 결과는 "이상 없음" 1건 유지(화면 표시·월 자동진단 추적, 재검증이 과거 해결 이력 정리하므로 누적 없음) — **부분 해결 = 항목만 제거, 전부 해결 = 이력 삭제**. 마이그레이션 불필요. 회귀 테스트 2건(부분 해결→1건 남김 / 전부 해결→이상 보유 이력 0). cargo test 367 / clippy 통과. **수정파일**: diagnosis.rs.
  - **참고(미결)**: 전부 해결 시 옛 이상-이력은 삭제되나 현재 실행의 "이상 없음" 이력 1건은 남김(UX·auto_needed 보존 목적). 사용자가 "완전 0건" 원하면 skip-0 + 프론트 결과 직접표시 + app_settings 자동진단월 분리로 후속 처리.
- **원생 생년월일 추가** (2026-06-06, 사용자 요청 — 신규 기능): 원생에 생년월일(선택) 필드 추가. **마이그레이션 V305**(`students.birth_date TEXT` nullable). 백엔드 `students.rs`: Student/NewStudent/StudentUpdate + from_row + create/update/get/list 쿼리 4종에 birth_date 반영. 프론트: `types/student.ts`(3 인터페이스) + `lib/tauri/index.ts`(createStudent dev fallback) + `student-form.tsx`(FormState/emptyForm/studentToForm/formToPayload + 생년월일 date input) + `students/edit/page.tsx`(StudentUpdate 매핑 `?? null`). 등록/수정 폼에서 입력·표시. **후속(사용자 요청)**: 원생 목록 컬럼(`students/page.tsx`, 입교일 뒤·colSpan 8) + 엑셀 내보내기 컬럼(`export.rs` build_students_sheet, '학교' 뒤·테스트 인덱스 보정) 추가 → 폼·목록·엑셀 3곳 반영. cargo test 367 / clippy / lint / tsc / build 통과. **수정파일**: migration 305, students.rs, CLAUDE.md, types/student.ts, lib/tauri/index.ts, student-form.tsx, students/edit/page.tsx, students/page.tsx, export.rs.
- **자가진단 "완전 0건" 전환** (2026-06-06, 사용자 요청 — 위 "이상 없음 1건 남김" 미결 해소): 모든 이상이 해결되면 **아무 이력도 남기지 않음**. ① `run_and_record` skip-0(`issues_found > 0` 일 때만 INSERT), ② `reconcile_resolved_issues` 빈 이력 **무조건 삭제**(레거시 '이상 없음' 이력도 정리), ③ **자동진단 월 추적을 app_settings `last_auto_diagnosis` 로 분리**(이력 없이도 AC-6.6-1 월 1회 유지) — `auto_needed` 가 이 설정값 비교로 판정, ④ 프론트 `/settings/diagnosis` handleRun: 0건이면 이력이 비므로 **방금 실행 결과를 합성 행(`resultToRow`)으로 직접 표시**("이상 없음"). 테스트 보정 + 신규 `auto_run_tracked_even_when_no_issues`. cargo test 368 / clippy / lint / tsc / build 통과. **수정파일**: diagnosis.rs, app/settings/diagnosis/page.tsx.
