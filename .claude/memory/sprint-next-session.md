---
name: sprint-next-session
description: "Sprint 16 진행 중 — sprint16 브랜치. T0~T3 + 공지문달력 + 백업복원연결 + 청구/수납분리 + 사이드바UX + 폰트full + Win exe전용 완료(origin 동기화됨). **다음 세션 순서: ①daily/weekly 백업스케줄러 ②T11 통합검증 ③T10 릴리즈준비**. T4~T7 제외(T4~6=원장 직접, T7=반응형 피드백). ⚠️배포 금지. 새 세션/새 PC 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint16-dev-2026-06-09
---

**현재 위치(2026-06-10)**: **sprint16 브랜치**, 작업트리 clean. ⚠️ **origin/sprint16 대비 미push**(다른 PC 릴레이 전 `git push origin sprint16` 필요). develop 미머지.

## 2026-06-10 세션 완료 — 커밋
- **공지문 교습일정 달력**(`308ae73`): 청구년월 학사일정을 달력 PNG로 렌더해 공지문 캔버스에 합성. 2D바코드 아래 '교습일정' 체크박스(드래그·리사이즈). `src/lib/calendar-image.ts`(신규) — 일요일 시작 6주 그리드, 교습기간 빨간 외곽선(첫 평일 수업일~마지막 평일 수업일·경계 비수업일 트림·사이 평일 공휴일 포함·**토·일 항상 제외**), 특이일 라벨+기간 하이라이트(단원평가 주간 등), 보강데이 볼드·150%·단원평가 주간과 top 정렬, 셀선 검정. `NoticeImageKind`에 'calendar' 추가(런타임 생성). 신규 의존성·마이그레이션 없음. 사용자 시각검수 완료.
- **T3 DB 폴더 변경 + salt.bin 이전**(`92eb5ee`, ADR-009): copy-then-switch + 재시작. `setup.rs::change_data_folder` — 대상검증(기존DB차단·동일/포함차단) → WAL checkpoint(TRUNCATE) → smarthb/ 재귀복사(app.lock·-wal·-shm 제외,fsync) → 검증(cipher PRAGMA key+integrity_check) → 원본 MOVED_TO 마커 → config.json 갱신(마지막) → relaunch. 실패 시 config미변경=기존폴더유지·원본불삭제. tauri-plugin-process 추가(process:allow-restart). `/settings/db-folder` 신규+카드활성화. 단위테스트 8건. **dev relaunch 가드**: 개발빌드는 자동재시작 대신 수동안내(dev는 화면을 localhost서버에서 로드→relaunch시 dev서버 동반종료로 "localhost거부", 프로덕션 무관). 실데이터 시각검증(이전→원복) 완료.
- **백업 복원 연결**(`1cf3e77`): 백업/복원(ADR-003) IPC는 있으나 UI 미연결이던 공백 해소. ①자동: `startup.rs::run_startup` 인증후·DB초기화전 quick_check Failed면 `integrity::auto_restore_sync`(최신 정상 exit백업 교체·손상본 rollback보존) → `StartupResult.auto_restored` → 루트페이지 고지배너. cipher off 개발빌드는 stub Ok→미진입(dev/정상 무영향). ②수동: `/settings/backup`에 '이 백업으로 복원' 버튼+확인/완료 모달→`restoreBackup`→재시작(dev가드). ⚠️**실동작은 cipher 빌드에서만**(dev는 백업0건·stub). daily/weekly 스케줄러 미연결은 별도 backlog.
- **청구/수납 메뉴 분리**(`7b400b3`): '청구/수납 관리'→'청구 관리'(/billing, 청구목록만·탭제거)+'수납 관리'(/payments 신규, 수납+월별집계 탭). 공통 추출: `useBillingShared` 훅(청구년월·검색·요약 SSOT)+`BillingSummaryBar`·`BillingSearchBar`. 자가진단 링크 bills→청구/payments→수납 분리.
- **사이드바 UX**(`c5231f1`): 활성메뉴 강조(accent 좌측보더+배경+볼드, aria-current, `usePathname` `isMenuActive`=대시보드 정확일치·그외 하위경로포함) + 너비 20%축소(11.2rem) + 그룹여백(대시보드↔원생관리/공지문↔설정 각 mt-10=40px) + 그룹구분선(원생관리↔일정관리/수업관리↔청구관리/수납관리↔공지문, pseudo-element `before:inset-x-[20px]` 좌우20px마진) + 종료 좌측정렬 일치(border-l-4). 모두 `menuItemClass(href)` 헬퍼. **사용자 시각검수 완료**.
- **Pretendard subset→full 교체**(`dbcc0bd`): 공지문 캔버스 렌더 시 희귀 한자·특수문자 fallback 방지. `public/fonts/Pretendard-{Regular,SemiBold,Bold}.woff2`(full ~2.3MB), subset 3종 제거, globals.css 갱신. self-host 유지(pretendard 패키지 추출 후 제거).
- **Windows .exe 전용**(`1bb120e`): `tauri.conf.json` bundle.targets `"all"`→`["nsis","dmg"]`(Win=exe만·msi제외, mac=dmg). deploy.yml 릴리즈노트도 .exe 단일 안내(사용자 허가 후 .github 수정). 파일명 `SmartHB_{ver}_x64-setup.exe`.

## 이번 세션(2026-06-09) 완료 — 커밋
- **T1**(`d3a3884`): 회고 액션 — `useUnsavedChanges` 공통 훅(beforeunload + Ctrl+S `app:save` + 메뉴이동 가드 `unsavedGuard`), A99 입력필드 Ctrl+N 방어. `src/lib/use-unsaved-changes.ts`.
- **T2**(`0478e8f`): 원생 CSV 가져오기(PRD §4.13.1) — `import.rs`(UTF-8/EUC-KR 자동, 학년 "초3" 파싱, 중복 skip, 백업 후 create_student 위임) + `/settings/import`. csv/encoding_rs 의존성.
- **공지문 보강**(`9e85887`): 캔버스 이미지 요소(교습소 로고/2D바코드 체크박스 + **임의 이미지 추가** customImages) / 텍스트박스 **배경색**(background_color, 밝은노랑 #FFEC99) / 배경서식 글씨 깨짐 해결(생성 PNG를 배경 **원본 해상도** naturalWidth로 렌더). react-rnd lockAspectRatio 비율유지. z-order=배경→추가이미지→로고바코드→텍스트.

## 다음 세션 할 일 (사용자 확정 순서 2026-06-10)
**T4~T7 제외** — T4(양OS빌드)·T5(양PC동기화)·T6(실사용개시)는 **원장님이 직접** 수행, T7(초기 피드백)은 **반응형**(실사용 중 이슈 들어오면 즉시 대응, 선제 작업 없음). 따라서 코드 작업은 아래 순서:
1. **daily/weekly 백업 스케줄러 연결** (backlog 발견 — 정책만 있고 미생성). 실사용 직전 데이터 안전 보강이라 최우선.
   - 현재 `startup.rs::spawn_background_tasks`는 heartbeat(60s)+hourly(3600s)만 spawn. daily(30)/weekly(4)는 `backup.rs`에 계층·max_keep 정의만 있고 **트리거 없음**.
   - 권장 설계: 순수 interval은 앱이 계속 떠 있어야만 fire → 간헐적 사용엔 부적합. **catch-up 방식**: 시작 시(+hourly tick마다) `backup.rs::scan_layer(Daily/Weekly)` 최신 created_at 확인 → 24h/7d 경과 또는 0건이면 `try_create_backup` 호출. "is due" 판정은 feature 무관 단위테스트, 실제 create_backup은 cipher 빌드만 동작.
2. **T11 통합 검증**: cargo test/clippy --all-targets/cargo check --features cipher/lint/tsc/build 전수 + develop 반영 점검.
3. **T10 v1.0 릴리즈 준비**: CHANGELOG 1.0.0 작성 + 버전 0.6.0→1.0.0(package.json/Cargo.toml/tauri.conf.json) + README 갱신 + deploy.yml 확인. **모든 변경 안정화 후 마지막**.
> 이후 마무리: ROADMAP 업데이트 → sprint-close(develop 머지) → sprint-review(리뷰+회고).
> ⚠️ **배포 금지**: deploy-prod(v1.0.0 태그 push)는 사용자 명시 지시 전까지 금지. 프로덕션 브랜치 `master`.

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
이번 세션 전 작업: cargo test(403)·clippy --all-targets·cargo check --features cipher·tsc·lint 통과. 실앱 시각검증: T0~T3·공지문달력·청구수납분리·사이드바 모두 사용자 검수 완료. ⚠️ 백업복원·무결성·daily/weekly는 **cipher 빌드에서만 실동작**(dev는 stub·백업0건) — dev에선 회귀만 확인됨.

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[sprint16-plan]], [[tauri-window-confirm-blocked]], [[ntfs-power-loss-pattern]], [[migration-numbering]]
