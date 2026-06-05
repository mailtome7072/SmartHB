---
name: sprint-next-session
description: "Sprint 14 진행 중 — T0~T6 완료·커밋·push, 다음 진입점 T7(복원 리허설). 새 환경 릴레이 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint14-relay-prep-2026-06-04b
---

**현재 위치(2026-06-05)**: **Sprint 14** 진행 중. 브랜치 **`sprint14`**(develop 기반, **origin push 완료**). **T0~T6 완료** + T2/T4 사용자 시각검증 통과. 다음 진입점 = **T7(복원 리허설)**.
> Sprint 12·13 완료·머지. Phase 5 취소 ([[exam-feature-cancelled]]).

## 집(Mac) 릴레이 시작 절차
1. `git fetch origin && git checkout sprint14 && git pull`
2. **`pnpm install`** — **recharts 3.8.1 신규 설치됨**(pnpm-lock 반영) → 반드시 install.
3. `pnpm tauri:dev` — 마이그레이션 자동 적용(V303 포함). 집 Mac은 자체 cloud path/DB(회사와 별도·독립).
4. **PIN**: salt.bin 클라우드 동기화 → 같은 PIN. (집 DB는 데이터가 회사와 다를 수 있음 — 개발/검증엔 무방)
5. `.claude/memory/` 미러는 sprint14에 커밋돼 동기화됨.

## 완료 (이번 세션, 2026-06-04 회사 Win)
- ✅ T0 carry-over (이전 세션)
- ✅ **T1 자가진단 백엔드** — `303__create_diagnosis_history.sql`(**실제 무접두 컨벤션, 문서의 "V303"은 약칭**) + `commands/diagnosis.rs`(IPC4 + 검사7 + 테스트20)
- ✅ **T2 자가진단 프론트** — `types/diagnosis.ts` + 래퍼4 + **`/settings/diagnosis` 전용 라우트**(계획의 인라인 섹션 대신) + 설정 카드 + AppShell 자동 트리거(세션1회)
- ✅ **T3 대시보드 IPC** — `commands/dashboard.rs` IPC8(현황/당일/월요약/출결진행률/청구추이/알림/메모 get·save) + 알림5 + 테스트
- ✅ **T4 대시보드 UI** — `app/page.tsx` 대시보드 교체 + `components/dashboard/`(DashboardView + charts.tsx) + 위젯6 + 메모 최상단 + **recharts ssr:false 동적 import**(R96)
- **검증 중 수정/추가**: 종료 확인 다이얼로그(sidebar) / 출결진행률 **공휴일·휴원일 제외**(attendance::load_off_dates 미러) / **월별 청구총액 추이 위젯**(get_billing_trend, 마지막 청구월 기준 12개월·빈달0) / 우측열 고정높이(당일수업+추이 = 교습소현황, grid stretch+flex-1) / **check1 보강필요시간 음수 오탐 수정**(보강대상=absent+makeup_done, 소멸 makeup_expired 면제 제외, − makeup_attended)
- DB: 오탐 이력(diagnosis_history id=1, 수정 전 auto) 일회성 삭제 완료.

## 완료 (세션 #3, 2026-06-05 회사 Win)
- ✅ **T5 내보내기 백엔드** — `commands/export.rs` 신규. IPC 3종(export_students/attendances/billing) + CSV유틸(csv_field/csv_row/with_bom/write_csv) + 라벨변환4 + 테스트9. **UTF-8 BOM**(Excel 한글). **출결=정규+보강 UNION(구분 컬럼)**, **청구=청구상태 컬럼 추가**, `year_month: Option`(None=전체). 교습비 standard_fees LEFT JOIN(V201 보정값 주4h=200000). simplify 4-agent 검토→변경없음(제네릭래퍼·enum Display는 스코프밖/이득미미 skip). **cargo test --lib 356 passed / clippy clean**. 커밋 `a60ca26` push 완료.

- ✅ **T6 내보내기 프론트** — `types/export.ts`(ExportResult/ExportTarget) + IPC 래퍼3 + `showCsvSaveDialog` + **`/settings/data` 신규 라우트**(diagnosis 패턴: 대상3종+기간 전체/특정월+결과배너) + 설정 카드. lint/tsc/build(라우트 생성) 통과. 커밋 `081dc96`.
  - **T6 사용자 검증 대기**: `/settings/data`에서 CSV 저장→엑셀 한글 정상 + 기간 동작. `pnpm tauri:dev` 실앱 확인 필요.

## 남은 작업 (SSOT: `docs/sprint/sprint14.md` + `docs/sprint/sprint14/scope.md`)
- ⬜ **T7 복원 리허설** (4h) ← **다음 진입점** — `backup.rs` 확장(run_backup_rehearsal: 임시복사→PRAGMA integrity_check→행수→삭제 + list_backup_files) + 설정>백업관리 UI. cipher off 개발빌드는 평문백업만 리허설(R98).
- ⬜ **T8 통합 검증** (3h) — test / clippy / **cargo check --features cipher** / lint / tsc / build / `.sqlx`(런타임 query패턴이라 갱신 불필요하나 cipher 빌드 점검) / CLAUDE.md V303 현황 갱신(A92)

## 마무리 후
- **sprint-close → sprint-review** (코드리뷰+검증+회고), DoD/AC 전수 마킹.
- **sprint14.md 본문 정정 필요**(검증 중 발견, sprint-close 시): 검사5 컬럼 `expiry_date`→`makeup_deadline`, 검사7 `payments.amount` 미존재→결제수단/카드사 누락 기준, 마이그레이션 "V303"→실제 `303__`, dashboard IPC "6종"→실제 8종.

## Sprint 15로 이연 (ROADMAP 기록됨)
- 교습소 정보 화면 / **'DB 폴더 변경'**(마법사 재실행 대체 — copy-then-switch + salt.bin/WAL/backup 동반 이전, R12와 상호의존, ADR 필요) / **자가진단 이력 수동 삭제(B안 — 자동삭제 미도입)** / 내보내기 Excel+비번보호.

## 정책
- **PR 생략, 직접 머지** ([[workflow-no-pr]]). 메모리 추가/수정 시 **사용자 메모리 + `.claude/memory/` 양쪽 갱신 후 commit**. cipher: dev off / CI·release on ([[cipher-test-gate-trap]]).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[keyring-v3-features-trap]], [[sqlite-migration-fk-rebuild]], [[ntfs-power-loss-pattern]]
