---
name: sprint-next-session
description: "Sprint 14(자가진단·대시보드·내보내기·복원리허설) 진행 중 — T0 완료·커밋, 다음 진입점 T1(자가 진단 백엔드). 새 환경 릴레이 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint14-relay-prep-2026-06-04
---

**현재 위치(2026-06-04)**: **Sprint 14** 진행 중. 브랜치 **`sprint14`**(develop 기반). 세션 #1에서 **계획 수립 + T0 carry-over 완료·커밋**. 다음 진입점 = **T1(자가 진단 백엔드)**.
> Sprint 12·13은 완료·머지됨. Phase 5(단원평가/학습보고서)는 취소 — 계획에서 제외 ([[exam-feature-cancelled]]).

## 회사 Win 환경 릴레이 시작 절차
1. `git fetch origin && git checkout sprint14` (또는 `git checkout -b sprint14 origin/sprint14`) → `git pull`
2. **`pnpm install`** — 현재 신규 의존성 없음(html-to-image·react-rnd는 Sprint 12에서 반영됨). recharts는 **T4 착수 시** 설치 예정이라 아직 미설치.
3. `pnpm tauri:dev` — 앱 시작 시 마이그레이션 자동 적용. DB·서식·템플릿은 클라우드 동기화 폴더(`smarthb/`)에 있어 자동 공유.
4. **PIN**: 키체인 키는 PC별 → 회사 PC 첫 실행 시 PIN 입력 필요(salt.bin은 클라우드 동기화 → 같은 PIN으로 해제). 회사 DB는 cloud path 별도라 독립.
5. `.claude/memory/` 미러는 sprint14에 커밋돼 있어 컨텍스트 동기화됨.

## Sprint 14 작업 목록 (SSOT: `docs/sprint/sprint14.md` + `docs/sprint/sprint14/scope.md`)
- ✅ **T0** carry-over (A91 startup cipher-off 주석 + ADR-008 정정 + A93 `/lock` 단일 로딩) — 커밋 `f532561`. gitignore 위생(WAL 사이드카) `8463dc3`.
- ⬜ **T1** 자가 진단 백엔드 (5h) — `V303__create_diagnosis_history.sql` + `commands/diagnosis.rs`(IPC 4종 + 검사 7종) + 단위 테스트 ← **다음 진입점**
- ⬜ **T2** 자가 진단 프론트 (3h) — `types/diagnosis.ts` + 설정 메뉴 진단 UI
- ⬜ **T3** 대시보드 집계 IPC (6h) — `commands/dashboard.rs`(집계 6종 + 알림)
- ⬜ **T4** 대시보드 위젯 UI (8h) — `app/page.tsx` 교체 + `components/dashboard/` 위젯6 + 알림 + **recharts 설치**(shadcn 내장 차트 먼저 확인, dynamic import로 라우트 한정 R96)
- ⬜ **T5** 내보내기 백엔드 (3h) — `commands/export.rs`(CSV IPC 3종: 원생/출결/청구-수납)
- ⬜ **T6** 내보내기 프론트 (3h) — `types/export.ts` + 설정 > 데이터 관리
- ⬜ **T7** 복원 리허설 (4h) — `commands/backup.rs` 확장 + 설정 UI
- ⬜ **T8** 통합 검증 (3h) — test·clippy·**cipher**·lint·tsc·build + `.sqlx` 캐시 + CLAUDE.md V303 현황 갱신(A92)

## T1 착수 전 선행 확인 (scope.md 기록)
1. **실제 스키마 컬럼명 확인** — 검사 7종 쿼리용: `regular_attendances / students / bills / student_schedules / makeup_attendances / payments`. 마이그레이션 V101~V111 참조.
2. **V303 추가 후 `sqlx prepare`로 `.sqlx` 캐시 갱신 필수** ([[cipher-test-gate-trap]] 게이트 유지) — 안 하면 CI(SQLX_OFFLINE) 컴파일 실패.
3. V303 번호는 300번대 도메인 확장 블록 연속(V302 다음).

## 진행 방법
- `/sprint-dev 14` 재진입 → scope.md + sprint14.md(SSOT) 확인 후 T1부터. 각 Task 완료 후 simplify 1회.
- 마무리: **sprint-close → sprint-review** (코드리뷰+검증+회고). DoD/AC 전수 마킹.

## 정책
- **PR 생략, 직접 머지** ([[workflow-no-pr]]).
- 메모리 추가/수정 시 **사용자 메모리 + `.claude/memory/` 양쪽 갱신 후 commit**.
- cipher: dev는 off, release/CI는 on 게이트 유지 ([[cipher-test-gate-trap]]).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[keyring-v3-features-trap]], [[sqlite-migration-fk-rebuild]]
