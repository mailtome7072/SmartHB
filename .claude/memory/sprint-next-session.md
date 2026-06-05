---
name: sprint-next-session
description: "Sprint 14 진행 중 — T0~T6 + 검증-phase 보강(엑셀전환/대시보드UX/보강버그픽스/오늘열) 완료·push. 다음 진입점 T7(복원 리허설). 새 환경 릴레이 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint14-relay-prep-2026-06-04b
---

**현재 위치(2026-06-05)**: **Sprint 14** 진행 중. 브랜치 **`sprint14`**(develop 기반, **HEAD=origin/sprint14, 전부 push**, 최신 커밋 `b8afa6b`). **T0~T6 + 이번 세션 검증-phase 보강 완료·사용자 정상동작 확인**. 다음 진입점 = **T7(복원 리허설)**.
> Sprint 12·13 완료·머지. Phase 5 취소 ([[exam-feature-cancelled]]).

## 집(Mac) 릴레이 시작 절차
1. `git fetch origin && git checkout sprint14 && git pull`
2. **`pnpm install`** — recharts 3.8.1(pnpm-lock 반영) → install. (엑셀 크레이트 `rust_xlsxwriter`는 Rust 의존성이라 `cargo`가 tauri:dev 시 자동 fetch — pnpm 무관)
3. `pnpm tauri:dev` — 마이그레이션 자동 적용(**V304까지** — 퇴교생 미보강 결석 소멸 백필 포함). 집 Mac은 자체 cloud path/DB(회사와 별도·독립).
4. **PIN**: salt.bin 클라우드 동기화 → 같은 PIN. (집 DB는 데이터가 회사와 다를 수 있음 — 개발/검증엔 무방)
5. `.claude/memory/` 미러는 sprint14에 커밋돼 동기화됨.
6. ⚠️ dev 재시작 시 `app.lock` os-error-33/손상 로그가 잠깐 뜰 수 있으나 free fallback으로 정상 복구(설계됨, [[ntfs-power-loss-pattern]]).

## 완료 (이번 세션, 2026-06-04 회사 Win)
- ✅ T0 carry-over (이전 세션)
- ✅ **T1 자가진단 백엔드** — `303__create_diagnosis_history.sql`(**실제 무접두 컨벤션, 문서의 "V303"은 약칭**) + `commands/diagnosis.rs`(IPC4 + 검사7 + 테스트20)
- ✅ **T2 자가진단 프론트** — `types/diagnosis.ts` + 래퍼4 + **`/settings/diagnosis` 전용 라우트**(계획의 인라인 섹션 대신) + 설정 카드 + AppShell 자동 트리거(세션1회)
- ✅ **T3 대시보드 IPC** — `commands/dashboard.rs` IPC8(현황/당일/월요약/출결진행률/청구추이/알림/메모 get·save) + 알림5 + 테스트
- ✅ **T4 대시보드 UI** — `app/page.tsx` 대시보드 교체 + `components/dashboard/`(DashboardView + charts.tsx) + 위젯6 + 메모 최상단 + **recharts ssr:false 동적 import**(R96)
- **검증 중 수정/추가**: 종료 확인 다이얼로그(sidebar) / 출결진행률 **공휴일·휴원일 제외**(attendance::load_off_dates 미러) / **월별 청구총액 추이 위젯**(get_billing_trend, 마지막 청구월 기준 12개월·빈달0) / 우측열 고정높이(당일수업+추이 = 교습소현황, grid stretch+flex-1) / **check1 보강필요시간 음수 오탐 수정**(보강대상=absent+makeup_done, 소멸 makeup_expired 면제 제외, − makeup_attended)
- DB: 오탐 이력(diagnosis_history id=1, 수정 전 auto) 일회성 삭제 완료.

## 완료 (세션 #3, 2026-06-05 회사 Win — 전부 사용자 정상동작 확인)
- ✅ **T5/T6 데이터 내보내기** — `commands/export.rs` + `types/export.ts` + IPC래퍼3 + `showXlsxSaveDialog` + **`/settings/data` 라우트** + 설정 카드. **최종 형식 = 엑셀(.xlsx)** (당초 CSV였으나 사용자 요청으로 전환, `rust_xlsxwriter 0.95` 신규 의존성). IPC 3종(export_students/attendances/billing), `year_month: Option`(None=전체). 출결=정규+보강 UNION(구분), 청구=청구상태 컬럼.
  - **엑셀 서식**: 원생 **일련번호 오름차순**, **일련번호·학년 숫자형(Int)**, 금전(교습비/청구액/할인액/최종액) **천단위#,##0+우측정렬**, 그외 좌측, **헤더 중앙정렬**, **컬럼너비 수동(CJK=2 폭, autofit은 한글 미반영)**, 수업시간 **'시간' 통일(분→시간)**, 저장 기본폴더 **downloadDir**.
  - export.rs는 `SheetData/Cell(Text/Int/Money/Hours)` 구조로 테스트 분리 + `write_xlsx` 서식 적용. 커밋 `081dc96`→`15b3e4d`(xlsx전환)→`f0e2d61`(숫자형)→`b8afa6b`(헤더중앙·너비·다운로드).

- ✅ **대시보드 UX 보강** (`966a237`) — 당일수업 `pm.N시 (X명) - 이름…`(시간만 강조, (N명) 검정), **메모 포스트잇 3장**(가변너비/개별높이 드래그저장 ResizeObserver, 박스높이=최대), '메모' 타이틀·박스 chrome 제거. 메모 백엔드 단일→**3슬롯**(`dashboard_memo_{i}`+`_h`, 레거시 흡수), IPC `get_dashboard_memos`/`save_dashboard_memo(index,content,height)`.
- ✅ **보강 정합성 버그픽스** (`966a237`) — (A)퇴교생(홍길동) 미보강 결석이 소멸알림 노출 → **마이그레이션 304** 백필(퇴교생 absent→makeup_expired) + dashboard 알림 **퇴교생 제외**. (B)고길동 6월 보강필요시간 0 → `compute_summary` **이월 누적**(소멸기한≥조회월+퇴교제외, earliest_pending과 정합). + **출결 보강필요/보강완료 셀 hover 힌트**(title, 백엔드 `pending_absences` 필드).
- ✅ **출결관리 진입 시 오늘 열 자동 노출** (`2afd7b9`) — 현재월이면 오늘 열로 가로 스크롤(중앙) + 오늘 헤더 accent 강조.

## 남은 작업 (SSOT: `docs/sprint/sprint14.md` + `docs/sprint/sprint14/scope.md`)
- ⬜ **T7 복원 리허설** (4h) ← **다음 진입점** — `backup.rs` 확장(run_backup_rehearsal: 임시복사→PRAGMA integrity_check→행수→삭제 + list_backup_files) + 설정>백업관리 UI. cipher off 개발빌드는 평문백업만 리허설(R98).
- ⬜ **T8 통합 검증** (3h) — test / clippy / **cargo check --features cipher**(rust_xlsxwriter 순수 Rust라 무관하나 점검) / lint / tsc / build / `.sqlx`(런타임 query패턴이라 갱신 불필요) / CLAUDE.md 마이그레이션 현황 **V304** 갱신(이미 반영됨, 확인만)

## 마무리 후
- **sprint-close → sprint-review** (코드리뷰+검증+회고), DoD/AC 전수 마킹.
- **sprint14.md 본문 정정 필요**(검증 중 발견, sprint-close 시): 검사5 컬럼 `expiry_date`→`makeup_deadline`, 검사7 `payments.amount` 미존재→결제수단/카드사 누락 기준, 마이그레이션 "V303"→실제 `303__`, dashboard IPC "6종"→실제 8종, **내보내기 CSV→엑셀(.xlsx) 전환**(Excel은 Sprint15 예정이었으나 14로 당김), 메모 단일→3슬롯.
- **검증-phase 보강은 scope.md "발견된 이슈"에 전수 기록됨** — sprint-close 시 sprint14.md/CHANGELOG 반영.

## Sprint 15로 이연 (ROADMAP 기록됨)
- 교습소 정보 화면 / **'DB 폴더 변경'**(마법사 재실행 대체 — copy-then-switch + salt.bin/WAL/backup 동반 이전, R12와 상호의존, ADR 필요) / **자가진단 이력 수동 삭제(B안 — 자동삭제 미도입)** / **내보내기 비밀번호 보호 옵션**(Excel 본체는 Sprint14에서 완료, 비번보호만 잔류).

## 정책
- **PR 생략, 직접 머지** ([[workflow-no-pr]]). 메모리 추가/수정 시 **사용자 메모리 + `.claude/memory/` 양쪽 갱신 후 commit**. cipher: dev off / CI·release on ([[cipher-test-gate-trap]]).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[keyring-v3-features-trap]], [[sqlite-migration-fk-rebuild]], [[ntfs-power-loss-pattern]]
