---
name: sprint-next-session
description: "Sprint 14 구현 전부 완료(T0~T8). 다음 진입점 = sprint-close(사용자 시각검증 후). 미push 커밋 2건 있음. 새 환경 릴레이 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint14-T7-T8-2026-06-06
---

**현재 위치(2026-06-06, 집 Mac)**: **Sprint 14 구현 전부 완료(T0~T8)**. 브랜치 **`sprint14`**(develop 기반). **다음 진입점 = sprint-close**(사용자 시각 검증 통과 후). ⚠️ **로컬에 미push 커밋 2건**(`1182111` T7, `98264cb` scope) — 다른 환경 릴레이 전 `git push` 필요.
> Sprint 12·13 완료·머지. Phase 5 취소 ([[exam-feature-cancelled]]).

## 릴레이 시작 절차 (집/회사 공통)
1. `git fetch origin && git checkout sprint14 && git pull`
2. **`pnpm install`** — recharts 3.8.1(pnpm-lock 반영). 엑셀 크레이트 `rust_xlsxwriter 0.95`는 Rust 의존성이라 cargo가 자동 fetch.
3. `pnpm tauri:dev` — 마이그레이션 자동 적용(**V304까지**). 집/회사 각자 cloud path·DB(독립).
4. **PIN**: salt.bin 클라우드 동기화 → 같은 PIN.
5. `.claude/memory/` 미러는 sprint14에 커밋돼 동기화됨.
6. ⚠️ dev 재시작 시 `app.lock` os-error-33/손상 로그 잠깐 → free fallback 정상 복구(설계됨, [[ntfs-power-loss-pattern]]).

## 완료 (세션 #4, 2026-06-06 집 Mac)
- ✅ **T7 복원 리허설** (`1182111`) — `backup.rs::run_backup_rehearsal(backup_path)`: 임시 디렉토리 복사 → **read-only sqlx 풀**로 열기 → `PRAGMA integrity_check` → 주요 6종(students/student_schedules/regular_attendances/makeup_attendances/bills/payments) `COUNT(*)` → 사본 폐기. 운영 DB 무영향. cipher 게이트 `apply_rehearsal_key`(on=PRAGMA key via paths::pragma_key_sql, **off=평문 백업만 R98**). `list_backups`는 기존 재사용. `RehearsalResult{backup_path,size_bytes,success,integrity_detail,table_counts,total_rows}`+`TableCount{table,count}`(snake_case IPC). 프론트: `/settings/backup` 라우트(목록+리허설+결과패널) + 설정 카드 + 타입 + `runBackupRehearsal` 래퍼. **테스트 4건**(정상/손상/없는파일/REHEARSAL_TABLES 스키마동기화 가드). simplify 적용: 검증 로직을 `Result<Vec<TableCount>,String>`+`?`로 리팩터(반복 튜플 4개 제거).
  - **설계 근거**: integrity.rs는 rusqlite+cipher-only(off는 stub Err)라 R98 위반 → 별도 sqlx 경로가 정당(altitude 검토 통과). cipher on 빌드는 sqlx도 SQLCipher 링크(libsqlite3-sys 공유)라 PRAGMA key 동작.
- ✅ **T8 통합 검증(자동)** (`98264cb`) — cargo test **365 passed** / clippy clean / **`cargo check --features cipher` clean** / lint / tsc / build(`/settings/backup` 2.5kB) / `.sqlx` 런타임 query 패턴이라 갱신 불필요 / CLAUDE.md 마이그레이션 현황 이미 **V304** 반영.

## 사용자 시각 검증 대기 (sprint-close 전 — `pnpm tauri:dev`)
- ⬜ **T7 복원 리허설**: ⚠️ cipher off 개발빌드는 백업 파일이 없어 `/settings/backup` 목록이 **빈 상태**(빈 UI만 확인 가능). 실제 리허설 보려면 평문 SQLite를 `SmartHB-data/backup/{exit|daily|...}/app_YYYYMMDD_HHMMSS.db` 형식으로 수동 배치하거나 cipher on 빌드에서 확인.
- ⬜ **T6 내보내기**(이전 세션 잔여): `/settings/data` 엑셀 저장 전체/월 동작.

## 마무리 후 (sprint-close 시 sprint14.md 본문 정정 필요)
- 검사5 `expiry_date`→`makeup_deadline` / 검사7 `payments.amount` 미존재→결제수단·카드사 누락 / 마이그레이션 "V303"→실제 `303__` / dashboard IPC "6종"→실제 8종 / **내보내기 CSV→엑셀(.xlsx) 전환** / 메모 단일→3슬롯 / **T7 list_backup_files→기존 list_backups 재사용**.
- **검증-phase 보강·simplify 결정은 scope.md "발견된 이슈"/체크포인트에 전수 기록됨**.
- 순서: **sprint-close → sprint-review**(코드리뷰+검증+회고), DoD/AC 전수 마킹.

## Sprint 15로 이연 (ROADMAP 기록됨)
- 교습소 정보 화면 / **'DB 폴더 변경'**(copy-then-switch + salt.bin/WAL/backup 동반, ADR 필요) / **자가진단 이력 수동 삭제(B안)** / **내보내기 비밀번호 보호 옵션** / **CSV 가져오기**(PRD §4.13.1).

## 정책
- **PR 생략, 직접 머지** ([[workflow-no-pr]]). 메모리 추가/수정 시 **사용자 메모리 + `.claude/memory/` 양쪽 갱신 후 commit**. cipher: dev off / CI·release on ([[cipher-test-gate-trap]]).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[keyring-v3-features-trap]], [[sqlite-migration-fk-rebuild]], [[ntfs-power-loss-pattern]]
