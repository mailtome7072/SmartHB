---
name: sprint-next-session
description: "Sprint 16 진행 중 — sprint16 브랜치 T0(수업일 변경 케이스1/2) 구현+시각검증 완료(32커밋, origin push됨). 다음 = T1(회고액션)~T11 또는 T0만 sprint-close. ⚠️배포 금지. 새 세션/새 PC 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint16-dev-2026-06-08
---

**현재 위치(2026-06-08)**: **sprint16 브랜치**에서 작업 중. `develop` 대비 **32커밋**, 작업트리 clean, **origin/sprint16 push 완료**(다른 PC에서 이어가기 가능). develop 미머지.

## 다른 PC(집)에서 이어가기 — 릴레이 절차
1. `git fetch origin && git checkout sprint16 && git pull origin sprint16`
2. `pnpm install` (의존성 변동 없음, 안전 차원)
3. `.env` 없으면 `.env.example` 복사 → `sqlx migrate run` (dev DB에 **V306·V307** 적용). 단, 앱 실행 시 `sqlx::migrate!` 가 자동 적용하므로 dev DB 직접 작업할 때만 필요.
4. `.claude/memory/` ↔ 사용자 메모리 미러 동기화 (이 파일 포함 — 절차: `.claude/memory/README.md`)
5. 개발 서버: `pnpm tauri:dev` (Node 25 주의 — dev 중 `pnpm build` 금지, .next 충돌 시 kill + `rm -rf .next` 재기동. 백엔드 변경 후엔 .next 정리 후 재기동 권장 — ChunkLoadError 예방)
6. **실 DB는 클라우드 동기화 폴더(MYBOX)** — 양 PC 공유. 집 PC도 동일 클라우드 폴더면 데이터 그대로.

## 완료된 것 — T0: 수업일 변경 도메인 (사용자 이슈 2026-06-08)
- **케이스1(1회성 이동)**: 출결표 출석 셀 우클릭 → [수업일 이동/보강 등록] → 달력에서 도착일+시작시간(시 단위) 선택. 평일·미점유·비휴일만, 동월 한정. `move_attendance` IPC. **V306**(`regular_attendances.note`), **V307**(`regular_attendances.start_time`).
- **케이스2(특정일 이후 영구 변경)**: 원생 스케줄 수정 시 적용 시작일(사전/사후) 지정 → `apply_schedule_change`로 변경일 이후 present만 재생성·결석/보강 보존. `generate_impl` 날짜 인식 리팩토링(effective_to exclusive).
- **수업 캘린더(ClassCalendar) 대폭 개선**: 원생별 색칩+시간표기 / 월 헤더 요일 / 주=2열묶음·일=개별블록(실제 길이) / 칩 hover 시 수업 시간범위 테두리(월 셀 hover 스타일) / 토·일 컬럼 폭 절반 / 시간행 14:00 시작 / 주 행높이 5rem(6명 2×3) / 이전·다음 이모티콘+년월 18px.
- **스케줄 편집**: 요일 변경(원래 요일 종료) / 평일·미등록 요일만 / 삭제 시 출결 정리 / 추가·변경·삭제 확인 다이얼로그.
- **검증**: cargo test **384** / clippy --all-targets / tsc / lint 전수 통과 + 실 앱 시각검증 "이상 없음"(사용자).
- 결정사항 **PI-20~30** 은 `docs/sprint/sprint16.md` 확정사항/미결정 섹션, 상세는 `docs/sprint/sprint16/scope.md`.

## 다음 할 일 (택1)
1. **T1부터 순서대로**: T1(회고액션 A99/A100/Ctrl+S) → T2(CSV 가져오기) → T3(DB폴더 변경+salt.bin) → T4~T11(양OS빌드/양PC동기화/실사용개시/v1.0릴리즈준비/통합검증)
2. **특정 Task 지정** 진행
3. **T0만 우선 마무리**: sprint-close(문서화+develop 머지) → sprint-review. 나머지 Task는 후속.
> ⚠️ **배포 금지**: deploy-prod(태그 push)는 사용자 명시 지시 전까지 금지. 프로덕션 브랜치 `master`.

## 마이그레이션 현황
최신 **V307** (V306 note, V307 start_time 추가). CLAUDE.md "현재 상태"의 V305 표기는 sprint-close 시 갱신 예정.

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[sprint16-plan]], [[ntfs-power-loss-pattern]], [[cipher-test-gate-trap]], [[migration-numbering]]
