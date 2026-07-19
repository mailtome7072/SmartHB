---
name: sprint-next-session
description: "✅ Sprint 20+21 → develop 머지 + v1.3.0 프로덕션 배포 + master→develop 역머지 전부 완료(2026-07-19). 다음 스프린트 계획 대기 중. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint21-deploy-2026-07-19
  modified: 2026-07-19T15:18:44.545Z
---

## ✅ 2026-07-19 세션 — Sprint 21 마무리 + v1.3.0 프로덕션 배포

### 완료 작업
1. **Sprint 21 (출결 다월 교습기간 그리드 R136)** sprint-close + sprint-review 완료 산출물 커밋:
   - 회고(`docs/sprint-retrospectives/sprint21-retrospective.md`), 테스트 보고서(`docs/test-reports/2026-07-19-sprint21.md`), DEPLOY.md
2. **스테이징 검증** 전부 통과:
   - 자동: lint ✅ / tsc ✅ / cargo test 444 passed ✅ / clippy -D warnings ✅ / pnpm build ✅ (DB 마이그레이션 변경 없음, V310 유지)
   - 수동: 출결 단일월 회귀 없음 / 다월 그리드 표시 / 토글+보강 / 수업일 이동 / 교습일정 인쇄(A122) 원장 확인 완료
   - `pnpm build` 캐시 플레이크 재현됨(빌드 직후 out/ 삭제 시) → `.next`+`out` 삭제 후 재빌드로 해결. 코드 문제 아님
3. **v1.3.0 프로덕션 배포 (deploy-prod 에이전트)**:
   - develop → master 직접 머지(PR 없음, [[workflow-no-pr]]), 버전 3파일 bump([[deploy-version-three-files]])
   - GitHub Actions success(22분), Release 아티팩트 `SmartHB_1.3.0_x64-setup.exe` + `SmartHB_1.3.0_aarch64.dmg` 검증 완료
   - https://github.com/mailtome7072/SmartHB/releases/tag/v1.3.0
   - 배포 범위: Sprint 20(청구 교습기간 기준 전환, 청구 삭제 ADR-010, 교습일정 인쇄, 출결 버그 A) + Sprint 21(R136)
   - Policy Gate: risk-register R131/R132(High)·R133/R138(Medium) 완화계획 있는 미해결 항목 → 사용자 인지 후 진행
4. **master → develop 역머지** 완료 후, deploy-prod 에이전트가 생성만 하고 미커밋으로 남긴 배포 마무리 문서(CHANGELOG [Unreleased]→[1.3.0] 전환, DEPLOY.md 배포현황, `docs/deploy-history/2026-07-20.md` 아카이브)를 커밋(`5d5ee55`)·push해 마무리.
   - **최종 동기화 검증 완료(clean)**: develop=`5d5ee55`(local=remote), master=`74bb447`(local=remote), 태그 v1.3.0=`74bb447`(local=remote). 미커밋 변경 없음
   - 교훈: deploy-prod 에이전트가 CHANGELOG/DEPLOY/deploy-history 문서를 수정하고도 최종 커밋을 빠뜨릴 수 있음 → 배포 후 `git status`로 미커밋 문서 확인 필요

### ⚠️ gh CLI 인증 트랩 (이번 세션 발견 — 중요)
- `~/.zshrc` 2번째 줄에 **무효 `GH_TOKEN`(ghp_...)** 이 하드코딩돼 있었음 → gh가 이 무효 토큰을 우선 사용해 모든 gh 명령 실패. `git push`는 macOS 키체인 자격증명이라 정상.
- **부모 프로세스 환경에 GH_TOKEN이 상속**되어 있어 ~/.zshrc 주석 처리해도 현재 세션 프로세스에는 40자로 남음(Bash 도구 매 호출 재상속). 완전 제거는 Claude Code 재시작 필요.
- 조치: (1) ~/.zshrc 해당 줄 주석 처리 완료 (2) 사용자가 `! unset GH_TOKEN && gh auth login`으로 keyring 로그인 생성
- **이 세션 이후로도 GH_TOKEN이 env에 남아있는 한, 모든 `gh` 명령은 `env -u GH_TOKEN gh ...` 접두사로 실행해야 함.** (재시작 후엔 불필요)
- 노출된 무효 토큰은 사용자에게 GitHub에서 Revoke 권고함(무효 상태지만 위생상)

## 마이그레이션 현황
최신 **V310** (schools.school_type 자동 보정). develop+master 모두 반영 완료. Sprint 20/21은 스키마 변경 없음.

## ⬜ 다음 세션 진입 시
다음 스프린트(Sprint 22) 계획 수립 대기 중 — 특별히 남은 작업 없음. 사용자가 다음 기능/개선 요청하면 그때부터 신규 사이클 시작.

관련: [[workflow-no-pr]], [[deploy-version-three-files]]
