---
name: sprint-next-session
description: "Sprint 16 진행 중 — sprint16 브랜치. T0+T1+T2+공지문보강 완료(develop 대비 36커밋, 작업트리 clean, ⚠️로컬만·origin 미push). 다음 세션 = 공지문 추가 보강 완료 → 그 다음 T3(DB폴더변경) 등 이어가기. ⚠️배포 금지. 새 세션/새 PC 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint16-dev-2026-06-09
---

**현재 위치(2026-06-09)**: **sprint16 브랜치**, `develop` 대비 **36커밋**, 작업트리 clean. ⚠️ **이번 세션 커밋들은 로컬만 — origin/sprint16 미push**(다른 PC 릴레이 전 `git push origin sprint16` 필요). develop 미머지.

## 이번 세션(2026-06-09) 완료 — 커밋
- **T1**(`d3a3884`): 회고 액션 — `useUnsavedChanges` 공통 훅(beforeunload + Ctrl+S `app:save` + 메뉴이동 가드 `unsavedGuard`), A99 입력필드 Ctrl+N 방어. `src/lib/use-unsaved-changes.ts`.
- **T2**(`0478e8f`): 원생 CSV 가져오기(PRD §4.13.1) — `import.rs`(UTF-8/EUC-KR 자동, 학년 "초3" 파싱, 중복 skip, 백업 후 create_student 위임) + `/settings/import`. csv/encoding_rs 의존성.
- **공지문 보강**(`9e85887`): 캔버스 이미지 요소(교습소 로고/2D바코드 체크박스 + **임의 이미지 추가** customImages) / 텍스트박스 **배경색**(background_color, 밝은노랑 #FFEC99) / 배경서식 글씨 깨짐 해결(생성 PNG를 배경 **원본 해상도** naturalWidth로 렌더). react-rnd lockAspectRatio 비율유지. z-order=배경→추가이미지→로고바코드→텍스트.

## 다음 세션 할 일
1. **공지문 추가 보강 완료** (사용자가 추가 요청 예정 — 미완 항목 이어서)
2. 이후 **T3: DB 폴더 변경 + salt.bin 이전**(8h, 최대 위험 — ADR부터) → T4~T11(양OS빌드/양PC동기화/실사용개시/v1.0릴리즈/통합검증)
> ⚠️ **배포 금지**: deploy-prod(태그 push)는 사용자 명시 지시 전까지 금지. 프로덕션 브랜치 `master`.

## 릴레이 절차 (다른 PC에서 이어가기)
1. (이 PC에서 먼저) `git push origin sprint16`
2. (다른 PC) `git fetch && git checkout sprint16 && git pull origin sprint16`
3. `pnpm install` → `.env` 없으면 복사 → 앱 실행 시 `sqlx::migrate!` 자동(또는 `sqlx migrate run`으로 dev DB에 V306·V307 적용)
4. `.claude/memory/` ↔ 사용자 메모리 미러 동기화 (절차: `.claude/memory/README.md`)
5. `pnpm tauri:dev` (Node 25 — 백엔드 변경 후 `.next` 정리 후 재기동 권장, ChunkLoadError 예방)
6. 실 DB는 클라우드 동기화 폴더(MYBOX) — 양 PC 공유

## 마이그레이션 현황
최신 **V307**(V306 note, V307 start_time). 이번 세션 신규 마이그레이션 없음(T2 CSV는 런타임쿼리, 공지문은 app_settings JSON). CLAUDE.md "현재 상태" V305 표기는 sprint-close 시 갱신 예정.

## 검증 상태
T1/T2/공지문보강 모두: cargo test(전체 395+ / import 11 / notice 5)·clippy --all-targets·tsc·lint 통과 + 실 앱 시각검증 완료(사용자).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[sprint16-plan]], [[tauri-window-confirm-blocked]], [[ntfs-power-loss-pattern]], [[migration-numbering]]
