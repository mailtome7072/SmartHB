---
Sprint: 3  |  Date: 2026-05-20  |  Session: #4
---

## 세션 #4 목표

T5 → T6 → T7 → T8 → T9 순차 구현. 각 Task 종료마다 self-verify + simplify + 분리 커밋.

> **세션 범위**: Phase 1 의 핵심 산출물 5개 (앱 셸 / 검색바 / dialog 플러그인 / 마법사 백엔드 / 마법사 프론트). 단일 세션 내 완료를 목표로 하되 도중 발견 이슈로 지연 시 다음 세션 진입점에 명시.

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | Task | 비고 |
|------|---------|------|------|
| `src/components/layout/sidebar.tsx` (신규) | [0회] | T5 | 메뉴 + 단축키 표기 + disabled 처리 |
| `src/components/layout/top-bar.tsx` (신규) | [0회] | T5 | 점유 디바이스 / 마지막 백업 / 동기화 |
| `src/components/layout/app-shell.tsx` (신규) | [0회] | T5 | sidebar + top-bar + 컨텐츠 조합 |
| `src/app/page.tsx` | [1회] | T5 | AppShell 적용 |
| `src/components/layout/global-search.tsx` (신규) | [0회] | T6 | 원생/학교/메뉴 검색 + Ctrl+F |
| `src/lib/hangul-search.ts` (신규 또는 의존성) | [0회] | T6 | 한글 자모 부분 일치 |
| `src-tauri/Cargo.toml` | [0회] | T7 | tauri-plugin-dialog 추가 |
| `package.json` | [1회] | T7 | @tauri-apps/plugin-dialog 추가 |
| `src-tauri/src/lib.rs` | [1회] | T7 + T8 | Builder.plugin + setup IPC 등록 |
| `src-tauri/capabilities/default.json` | [0회] | T7 | dialog:default 권한 |
| `src/lib/tauri/index.ts` | [1회] | T7 + T8 + T9 | selectFolder, setup IPC 래퍼 |
| `src-tauri/src/commands/setup.rs` (신규) | [0회] | T8 | save_cloud_folder / complete_setup / get_setup_status |
| `src-tauri/src/commands/mod.rs` | [1회] | T8 | setup 모듈 등록 |
| `src-tauri/src/commands/paths.rs` | [0회] | T8 | data_root() 동적화 |
| `src-tauri/migrations/200__seed_setup_settings.sql` (신규) | [0회] | T8 | app_settings 행 시드 |
| `src-tauri/src/commands/auth.rs` (또는 recovery.rs) | [0회] | T8 | salt 이전 로직 (Keychain → 파일) |
| `src/app/setup/page.tsx` (신규) | [0회] | T9 | 마법사 4단계 컴포넌트 |
| `src/types/index.ts` 또는 `src/types/setup.ts` (신규) | [0회] | T9 | SetupStatus 타입 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/` — Harness 정책
- ⬜ `src/app/students/`, `src/app/settings/` (이번 세션) — Day 7 이후 Task

## 완료 기준 (이번 세션)

- ✅ T5: AppShell + Sidebar + TopBar — Pretendard 18pt / 44×44px / disabled 메뉴 처리 (`9efd4d7`)
- ✅ T6: 글로벌 검색바 — 한글 자모 부분 일치, useDeferredValue 디바운스, Ctrl+F 단축키 (`9efd4d7`)
- ✅ T7: `tauri-plugin-dialog` 통합 + `selectFolder()` 래퍼 (`a7b02d3`)
- ✅ T8: setup.rs 모듈 (3 IPC) + V200 마이그레이션 시드 + config.json 분리 + 단위 테스트 3건 (`c97f260`)
  - **부분 구현 — salt 이전(R12)·paths::data_root() 동적화는 후속 sweep**
- ✅ T9: `/setup` 라우트 4단계 마법사 + LockScreen 재사용 + 라우팅 분기 (`d137c4f`)
- ✅ 각 Task self-verify (cargo test 108 passed / clippy / lint / tsc / build) 통과
- ✅ simplify — Task 그룹별 3-agent 병렬 리뷰 후 분리 커밋

## 발견된 이슈

### T8: cloud_folder_path 저장 위치 — chicken-and-egg

sprint3.md plan 은 `app_settings.cloud_folder_path` 에 클라우드 폴더 경로를 저장하고 `paths::data_root()` 가 이를 조회하도록 했으나, DB 자체가 클라우드 폴더 안에 있어 **DB 열기 전에는 경로를 알 수 없음**.

**Auto Mode 결정**: 클라우드 폴더 경로는 Tauri `app_config_dir()` 의 `config.json` 에 별도 보관. DB 는 클라우드, config 는 OS 로컬(양 PC 가 자기 config 를 유지). `app_settings.cloud_folder_path` 는 보조 메타데이터/디버그용으로만 사용.

추가 의존성: Tauri Path API 는 `app_config_dir()` 가 `tauri::Manager` trait 에 있어 별도 plugin 불요. config 파일 read/write 는 std::fs 로 충분.

## 다음 세션 진입점 — T10 (원생 목록 화면)

- **대상**:
  - `src/app/students/page.tsx` (신규): TanStack Query 로 listStudents/countStudents 구독, 필터(이름·학교급·학년·학교·요일·성별·재원상태), 정렬, 페이지네이션 UI
  - 200ms 이내 필터 반응, 44×44px 행, Ctrl+N → 신규 등록 단축키 (T14 와 연계)
- **활용 가능한 자산**: T3 의 listStudents/countStudents IPC, T4 의 TanStack Query/Zustand, T5 의 AppShell, T6 의 GlobalSearch.
- **참고**: T7 부터의 sprint3.md plan 잔여(T10~T15)는 분량이 크므로 다음 세션에서 T10·T11 2 Task 진행 권장.
- **세션 시작 시 확인**: `git log develop..HEAD --oneline` 으로 T1~T9 커밋 9건 + scope 마감 커밋 확인 + 세션 번호 +1 (#5).

### 이연된 후속 sweep (Sprint 3 또는 Sprint 4)

- **R12 salt 이전**: Keychain → `{cloud_folder}/smarthb/salt.bin`. auth.rs/recovery.rs/backup.rs 광범위 영향.
- **paths::data_root() 동적화**: backup/integrity/lock/sync/startup 모듈 통합. setup_completed=true 시점에 OnceLock 캐싱.

---

## 세션 #1~#4 결과 (참고)

- ✅ `2905663` — Sprint 3 진입
- ✅ `7d8af2c` — T1 Pretendard self-host
- ✅ `6766693` — T2 R13 audit PII 마스킹
- ✅ `b955ff1` — 세션 #1 마감
- ✅ `58aeab6` — T3 R14 페이지네이션
- ✅ `db3ca53` — 세션 #2 마감
- ✅ `c441f5c` — T4 Zustand + TanStack Query
- ✅ `4c0ce54` — 세션 #3 마감
- ✅ `9efd4d7` — T5+T6 앱 셸 + 글로벌 검색
- ✅ `a7b02d3` — T7 dialog 플러그인 + selectFolder
- ✅ `c97f260` — T8 마법사 백엔드 (config.json 분리)
- ✅ `d137c4f` — T9 마법사 프론트엔드
