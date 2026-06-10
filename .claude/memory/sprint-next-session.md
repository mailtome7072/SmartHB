---
name: sprint-next-session
description: "Sprint 16 진행 중 — sprint16 브랜치. T0+T1+T2+공지문보강+공지문달력+T3(DB폴더변경) 완료(작업트리 clean, ⚠️origin 대비 ahead 7·미push). 다음 = T4(양OS빌드)~T11 또는 사용자 지시. ⚠️배포 금지. 새 세션/새 PC 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint16-dev-2026-06-09
---

**현재 위치(2026-06-10)**: **sprint16 브랜치**, 작업트리 clean. ⚠️ **origin/sprint16 대비 ahead 7 — 미push**(다른 PC 릴레이 전 `git push origin sprint16` 필요). develop 미머지.

## 2026-06-10 세션 완료 — 커밋
- **공지문 교습일정 달력**(`308ae73`): 청구년월 학사일정을 달력 PNG로 렌더해 공지문 캔버스에 합성. 2D바코드 아래 '교습일정' 체크박스(드래그·리사이즈). `src/lib/calendar-image.ts`(신규) — 일요일 시작 6주 그리드, 교습기간 빨간 외곽선(첫 평일 수업일~마지막 평일 수업일·경계 비수업일 트림·사이 평일 공휴일 포함·**토·일 항상 제외**), 특이일 라벨+기간 하이라이트(단원평가 주간 등), 보강데이 볼드·150%·단원평가 주간과 top 정렬, 셀선 검정. `NoticeImageKind`에 'calendar' 추가(런타임 생성). 신규 의존성·마이그레이션 없음. 사용자 시각검수 완료.
- **T3 DB 폴더 변경 + salt.bin 이전**(`92eb5ee`, ADR-009): copy-then-switch + 재시작. `setup.rs::change_data_folder` — 대상검증(기존DB차단·동일/포함차단) → WAL checkpoint(TRUNCATE) → smarthb/ 재귀복사(app.lock·-wal·-shm 제외,fsync) → 검증(cipher PRAGMA key+integrity_check) → 원본 MOVED_TO 마커 → config.json 갱신(마지막) → relaunch. 실패 시 config미변경=기존폴더유지·원본불삭제. tauri-plugin-process 추가(process:allow-restart). `/settings/db-folder` 신규+카드활성화. 단위테스트 8건. **dev relaunch 가드**: 개발빌드는 자동재시작 대신 수동안내(dev는 화면을 localhost서버에서 로드→relaunch시 dev서버 동반종료로 "localhost거부", 프로덕션 무관). 실데이터 시각검증(이전→원복) 완료.

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
