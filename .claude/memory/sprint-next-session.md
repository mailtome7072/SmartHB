---
name: sprint-next-session
description: "Sprint 14 완료 + develop 머지·QA 통과. 다음 = deploy-prod(프로덕션 배포, 보류 중). 새 환경 릴레이 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint14-close-2026-06-06
---

**현재 위치(2026-06-06, 집 Mac)**: **Sprint 14 완료 + develop 머지·QA 통과**. sprint14 → develop `--no-ff` 머지 완료(develop HEAD `4df06dd`, origin push 완료). 머지 후 검증 통과(test 369/clippy/lint/tsc, V303~V305 적용). 사용자 수동 검증 완료. **다음 진입점 = deploy-prod(프로덕션 배포) — 사용자가 "지금은 배포 안 함" 선택해 보류 중.**
> Sprint 12·13 완료·머지. Phase 5 취소 ([[exam-feature-cancelled]]).

## 다음 할 일 — 프로덕션 배포 (재개 시)
- 사용자가 **"프로덕션 배포 준비해줘"** 하면 → **deploy-prod 에이전트**.
- ⚠️ **프로덕션 브랜치는 `master`** (원격에 `main` 없음 — CLAUDE.md의 "main"은 master로 읽을 것).
- **버전 상향 필요**: 현재 0.5.0은 이미 `v0.5.0` 태그 릴리즈됨. Sprint 14 배포는 **0.6.0(minor) 권장**(대시보드·자가진단·내보내기·복원리허설·생년월일 신규). package.json + src-tauri/Cargo.toml 동시 갱신 → develop→master 머지 → `vX.Y.Z` 태그 push → GitHub Actions가 Win/macOS 인스톨러·Release 자동 생성.
- (참고) 이미 끝난 단계 — develop 머지/QA/마이그레이션 적용:
  ```
  # 완료됨: git merge --no-ff sprint14 → develop, sqlx migrate run --source src-tauri/migrations
  ```
2. **`sqlx migrate run`** — V303~V305 적용 확인 (개발 DB).
3. **`pnpm tauri:dev` 수동 확인** (DEPLOY.md ⬜ 항목): 엑셀 내보내기 3종 + **복원 리허설**(cipher off 개발빌드는 평문 백업을 `SmartHB-data/backup/{layer}/app_*.db`에 수동 배치해야 동작). 대시보드 위젯·자가진단·원생 생년월일은 이미 검수 완료.
4. develop QA 통과 시: "프로덕션 배포 준비해줘" → **deploy-prod** 에이전트(태그 push → GitHub Actions 인스톨러 빌드).

## Sprint 14 결과 요약
- **마이그레이션 최신 = V305** (V303 diagnosis_history / V304 퇴교생 미보강결석 백필 / V305 students.birth_date). CLAUDE.md 현황 갱신됨.
- **신규 의존성**: recharts 3.8.1(대시보드 차트), **rust_xlsxwriter 0.95**(엑셀 내보내기 — 내보내기가 CSV→엑셀(.xlsx)로 전환됨).
- **sprint-review**: Critical 0 / High 0 / **Medium 1**(F1: monthly_summary 1:1 청구-수납 암묵 의존, Sprint 15 확장 시 리팩토링) / Low 2. cargo test **369** / clippy / `cargo check --features cipher` / lint / tsc / build 전수 통과.
- 산출물: `docs/test-reports/2026-06-06.md`, `docs/code-reviews/sprint14.md`, `docs/sprint-retrospectives/sprint14-retrospective.md`, 리스크 R99.

## 이번 세션(#4) 주요 구현 (origin 1ecd789..5d114d4)
- **T7 복원 리허설** `backup.rs::run_backup_rehearsal`(임시복사→read-only sqlx→PRAGMA integrity_check→주요6종 행수→폐기, cipher 게이트 apply_rehearsal_key, R98). `/settings/backup` 라우트.
- **T8 통합검증** 통과.
- **자가진단 "완전 0건"**: 실행마다 `reconcile_resolved_issues`로 해결항목 자동제거+빈이력 삭제, skip-0(0건이면 이력 미보관), auto_needed를 app_settings `last_auto_diagnosis`로 월1회 판정(AC-6.6-1). 0건이면 화면은 실행결과 직접 표시(`resultToRow`).
- **출결 입력 진행률 위젯·알림 제거**(출결이 월단위 'present' 일괄생성 모델이라 항상 100% 무의미).
- **대시보드**: 월 청구총액 추이 위젯 / 월 요약 이전·다음월 전환(메모 아래 배치) / **이달의 생일 위젯**(`get_birthdays_this_month`, '홍길동(5일)' 형식, 일자순) — 당일수업과 50/50. 타이틀 폰트 inline 24px(당일수업·생일·월요약), 다른 위젯은 전역 h2 28px.
- **원생 생년월일**(V305): 등록/수정 폼(생년월일→입교일 순) + 원생 목록 컬럼 + 엑셀 컬럼.
- **Node 25 dev 크래시 회피**: `next.config.ts` dev 전용 `config.cache=false`(webpack 캐시 직렬화 abort 회피). 정석은 Node LTS(v20/v22) 전환이나 이 Mac엔 node@25만 설치.

## 환경 주의 (재릴레이 시)
- **Node 25 + dev 동시 build 금지**: `pnpm tauri:dev` 실행 중 `pnpm build` 돌리면 `.next` 충돌로 dev가 500/CSS404로 깨짐 → `.next` 삭제 + dev 클린 재기동으로 복구. build는 dev 끈 상태에서만.
- dev 화면 깨지면: 프로세스 kill + `rm -rf .next` + 재기동.

## Sprint 15 이연 (ROADMAP)
- 교습소 정보 화면 / **DB 폴더 변경** / 자가진단 이력 수동삭제(완전0건으로 사실상 해소됨) / **내보내기 비밀번호 보호**(엑셀 본체는 14에서 완료) / CSV 가져오기(§4.13.1) / monthly_summary 청구집계 리팩토링(F1).

## 정책
- **PR 생략, 직접 머지** ([[workflow-no-pr]]). 메모리 추가/수정 시 **사용자 메모리 + `.claude/memory/` 양쪽 갱신 후 commit**. cipher: dev off / CI·release on ([[cipher-test-gate-trap]]).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[keyring-v3-features-trap]], [[sqlite-migration-fk-rebuild]], [[ntfs-power-loss-pattern]]
