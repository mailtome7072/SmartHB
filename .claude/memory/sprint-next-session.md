---
name: sprint-next-session
description: "Sprint 15 마감 완료(close+review). sprint15 브랜치 develop 미머지(보류). 다음 = (선택) develop 머지+tauri:dev QA → Sprint 16 계획 또는 배포. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint15-close-2026-06-07
---

**현재 위치(2026-06-07, 집 Mac)**: **Sprint 15 마감 완료** (sprint-close + sprint-review 끝). `sprint15` 브랜치(develop 기반, **develop 미머지 — 보류**). 범위 = **T0~T6(+T5) 완료, T7~T9는 Sprint 16 이연**. 작업트리 clean.
> Sprint 14 완료·0.6.0 버전확정까지 됨. **deploy-prod(v0.6.0 태그)는 여전히 보류**. Phase 5 취소([[exam-feature-cancelled]]).

## 다음 할 일 (재개 시 선택)
사용자가 develop 머지/배포를 보류한 상태. 재개 시:
1. **sprint15 → develop 머지** (PR 생략 직접 머지 [[workflow-no-pr]]): `git checkout develop && git merge --no-ff sprint15 && git push origin develop`
2. **`pnpm tauri:dev` 수동 QA** (DEPLOY.md ⬜): 교습소 정보 폼·이미지 업로드, 자가진단 이력 삭제, Ctrl+F/Ctrl+N 단축키, 전역 툴팁 20px, 대시보드 위젯
3. 이후 분기: **Sprint 16 계획**(sprint-planner) — UAT + v1.0 + 이연항목(T7~T9, CSV가져오기, DB폴더변경 등) / 또는 v0.6.0·v1.0 배포 결정(deploy-prod)

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
