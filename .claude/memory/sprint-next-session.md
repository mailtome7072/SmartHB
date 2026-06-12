---
name: sprint-next-session
description: "Sprint 16 진행 중 — sprint16 브랜치. T0~T3+공지문달력+백업복원+청구수납분리+사이드바+백업스케줄러+보관축소+**전체코드리뷰 P0 7/P1 11/P2 선별 7건 반영+원생폼 UX개선 모두 시각검수 완료**(보고서 docs/code-review/full-review-2026-06.md). **다음 세션 시작점: ①T11 통합검증 ②T10 v1.0 릴리즈준비**. ⚠️배포 금지·로컬 미push. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint16-dev-2026-06-09
---

**현재 위치(2026-06-11)**: **sprint16 브랜치**, 작업트리 clean. 마지막 커밋 `ea334e3`(원생 폼 UX 개선). ⚠️ **origin/sprint16 대비 미push**(다른 PC 릴레이 전 `git push origin sprint16` 필요). develop 미머지. **다음 세션은 T11 통합검증부터** (P0/P1/P2/폼개선 시각검수 모두 완료).

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

## 2026-06-11 세션 완료
1. **daily/weekly 백업 스케줄러** (`5630018`): catch-up 방식 — 시작 시+hourly tick마다 scan_layer 최신 created_at 24h/7d 경과(또는 0건) 판정→try_create_backup. is_due 순수함수+테스트 5건.
2. **백업 보관 축소** (`76b10c1`): exit 5/hourly 12/daily 14/weekly 4 (합계 68→35, 1인 시스템+클라우드 점유). PRD §5.4 v1.5.2+ADR-003 개정노트+rules/ARCHITECTURE 동기화.
3. **전체 코드리뷰** (6인 전문팀: 아키텍트/DB/디자이너/UX/시니어3분할/매니저): 보고서 `docs/code-review/full-review-2026-06.md` (`e982038`). P2 14건/P3는 v1.0 후.
4. **P0 7건 반영** (`7775a15`,`7d543d2`): WAL checkpoint+pool close(exit_hook, 양PC torn-sync 차단)/config.json fsync/todayLocalISO(UTC 어제 버그 3곳)/수납 draft 보호(refetch 보존+탭월변경 모달+useUnsavedChanges)/원생폼 임시저장 이어하기 배너/출결 Ctrl+Z editable 가드/change_schedule_day 원자 커맨드(테스트 3건).
5. **P1 11건 반영** (`8357a40`): 확인버튼 h-11/text-base(button.tsx)/퇴교 catch/notices 삭제확인/핵심화면 12px 제거/gray-500→muted-foreground 72곳/삭제버튼 빨강/errMsg 헬퍼(src/lib/errors.ts)/panic 가드 2곳/폼 Ctrl+S/문서드리프트(backend.md 청구2단계·query!규칙 폐기→런타임+테스트 표준, CLAUDE.md V307·0.6.0)/구메뉴명.
- 검증: cargo test 411/clippy/cipher check/tsc/lint/build 전수 통과.
6. **P2 선별 7건 반영** (`3a04770`): 자가진단 검사3 청구대상만(만성오탐 제거)/academic fail-soft 통일(expire_fail_soft)/billing 집계 테스트 3건(415통과)/학사코드 색 SSOT(`src/lib/schedule-code-colors.ts`, 공지문 달력색을 앱과 일치 teal·pink)/학년 max 학교급별/PaymentsView rows useMemo/settings/codes 저장실패 표시+props 동기화. **미진행 P2 10건은 v1.0 후**(P2-1 출결그리드 반응성, P2-2 에러한글화 121곳, P2-3 index.ts 분할, P2-4 notices 분리, P2-5 AppShell layout, P2-6 ConfirmDialog 공통화, P2-7 글씨 전면, P2-8 Undo·피드백, P2-11 queryKey 중앙화, P2-12 검색 학교명).
7. **원생 폼 UX 개선** (`ea334e3`, 시각검수 중 추가 요청): 임시저장 입력 즉시 저장+안내/그리드·신규버튼 임시저장 배지(STUDENT_DRAFT_PREFIX export)/성별·학년·학교 (미지정)기본값+필수검증 차단/학교급↔학교명 정합성/검증·저장실패 ErrorDialog 팝업화.
- **모든 P0/P1/P2/폼개선 사용자 실앱 시각검수 완료(2026-06-11).**
- ⚠️ `release_lock_atomic_is_idempotent_when_no_file` 병렬 실행 간헐 flake(전역 락 경로 공유, 기존 잠재) — T11 재현 시 직렬화 검토.

## 다음 세션 할 일 — **T11부터 시작**
**T4~T7 제외** — T4(양OS빌드)·T5(양PC동기화)·T6(실사용개시)는 **원장님 직접**, T7(피드백)은 반응형. 시각검증 완료됨.
1. **T11 통합 검증** (시작점): cargo test/clippy --all-targets/cargo check --features cipher/lint/tsc/build 전수 재실행 + develop 반영 점검. 코드리뷰 권고 시나리오 추가 — 양 PC 종료→재기동 정합(WAL checkpoint), 오전 시간대 날짜 입력. flake 테스트 재현 여부 확인.
2. **T10 v1.0 릴리즈 준비**: CHANGELOG 1.0.0 작성 + 버전 0.6.0→1.0.0(package.json/Cargo.toml/tauri.conf.json) + README 갱신 + deploy.yml 확인. **마지막**.
> 이후: ROADMAP 업데이트 → sprint-close(develop 직접 머지, PR 생략) → sprint-review.
> ⚠️ **배포 금지**: deploy-prod(v1.0.0 태그 push)는 사용자 명시 지시 전까지 금지. v1.0 후 개선 스프린트는 보고서 P2 미진행 10건 참조.

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
