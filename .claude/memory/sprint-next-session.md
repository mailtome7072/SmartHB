---
name: sprint-next-session
description: "Sprint 15 마감+develop 머지+tauri:dev QA 완료(develop push 7857963). 결정확정: Sprint 16까지 묶어 v1.0 직행(v0.6.0 배포 폐기). ⚠️배포(deploy-prod)는 사용자 명시 지시 전까지 금지. 다음 = Sprint 16 계획. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint15-close-2026-06-07
---

**현재 위치(2026-06-07, 집 Mac)**: **Sprint 15 마감 + develop 머지 + tauri:dev QA 완료**. develop HEAD `7857963`(origin push 완료, 로컬=원격). sprint15 머지 `--no-ff`. 범위 = **T0~T6(+T5) 완료, T7~T9는 Sprint 16 이연**. 작업트리 clean(develop 브랜치).
> Sprint 14 완료·0.6.0 버전확정까지 됨. **결정(2026-06-08, 사용자): v0.6.0 단독 배포는 폐기, Sprint 16까지 묶어 v1.0 직행**. Phase 5 취소([[exam-feature-cancelled]]).

## 다음 할 일 (확정 경로 — v1.0 직행)
develop에 Sprint 15까지 반영됨. **Sprint 16까지 완료 후 한 번에 v1.0 배포**가 확정 방향.
1. **Sprint 16 계획**(sprint-planner) — Phase 6 마지막: UAT + v1.0 릴리즈 + 이연항목 흡수(T7 양OS빌드·T8 양PC동기화·T9 통합검증 빌드부, CSV가져오기, DB폴더변경+salt.bin, 출결표 성능, 공지문 I/O, A89, R105 등).
2. ⚠️ **배포 금지**: deploy-prod(태그 push)는 **사용자가 명시적으로 지시할 때까지 절대 진행하지 않는다**. Sprint 16 완료 후에도 사용자 지시 대기. 프로덕션 브랜치 `master`(develop→master `--no-ff` + `v*` 태그 push → GitHub Actions 인스톨러).

## Sprint 15 결과 (sprint15 브랜치, 15커밋)
- **코드리뷰**: Critical 0 / High 0 / **Medium 1**(F3: 교습소 정보 미저장 이탈 경고 누락 → R105, Sprint16 이연) / Low 2(GlobalTooltip title 일시제거 스크린리더 영향 낮음 / Ctrl+N 입력필드 방어 미적용). `docs/code-reviews/sprint15.md`
- **자동검증 전수 통과**: cargo test **375** / clippy `--all-targets` / `cargo check --features cipher` / lint / tsc / build.
- **신규 의존성 없음 / DB 마이그레이션 없음** (최신 V305 유지).
- 완료 Task: T0(monthly_summary GROUP BY·R99 방어적) / T1(교습소 정보 화면 `/settings/info` — AcademyInfo 9필드+이미지, IPC 2종) / T2(자가진단 이력 삭제 IPC 2종+UI) / T3(접근성: gray대비 17건·GlobalShortcuts Ctrl+F·Ctrl+N) / T4(테스트 clippy 부채 6건·A89 로직분리 확인) / T5(GlobalTooltip 20px·설정카드·원생관리버튼) / T6(청구 standard_fees N+1 제거).
- 산출물: `docs/sprint/sprint15.md`, `docs/sprint/sprint15/{scope,accessibility-audit,performance-report}.md`, `docs/sprint-retrospectives/sprint15-retrospective.md`, `docs/test-reports/2026-06-07.md`, 리스크 R105.

## A98 (Sprint 15 회고 액션, 즉시 적용됨)
- self-verify clippy 명령에 `--all-targets` 누락 → 테스트 코드 clippy 부채 누적이 원인. **CLAUDE.md / `.claude/rules/harness-engineering.md`의 clippy 명령을 `--all-targets` 포함으로 교정 완료.** 향후 신규 테스트도 clippy clean 보장.

## Sprint 16 이연 (실측·마이그레이션·물리환경 동반)
- **T7 양 OS 빌드 / T8 양 PC 동기화 / T9 통합검증(빌드·실설치부)** — UAT와 통합.
- CSV 가져오기(UAT 데이터 이관), DB폴더변경+salt.bin, 출결표 N+1 재설계·셀 memo·makeup_attendances 인덱스(실측 후), 공지문 50장 I/O 병렬화([[ntfs-power-loss-pattern]]), 접근성 밀집UI 44px·gray-500·F1·Ctrl+S, A89 notices UI 구획화, R105(미저장 이탈 경고).

## 환경 주의
- **Node 25**: `pnpm tauri:dev` 중 `pnpm build` 금지(.next 충돌). 깨지면 kill + `rm -rf .next` 재기동.
- cipher dev off / CI·release on ([[cipher-test-gate-trap]]). 프로덕션 브랜치 `master`.

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[sqlite-migration-fk-rebuild]], [[ntfs-power-loss-pattern]], [[keyring-v3-features-trap]]
