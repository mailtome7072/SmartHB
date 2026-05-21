---
Sprint: 4  |  Date: 2026-05-21  |  Session: #2
---

## 세션 #2 목표

**T2: 교습소 설정 메뉴 화면 신설 (사용자 이슈 #0, PRD §4.0/§4.12)**

현재 사이드바 "설정" 항목이 `/settings/codes` 로 직접 연결 — 코드 테이블만 접근 가능하고 교습소 운영 시간 등 영구 설정 화면 부재. T2 는 설정 허브 페이지 + 교습소 운영 시간 편집 화면 + 백엔드 IPC 신규.

운영 시간은 마법사 단계가 아니라 사용 중 변경 가능한 영구 설정 — `app_settings` key/value (`operating_hours` 키, JSON 직렬화) 활용. DB 마이그레이션 불필요.

## 사전 확인 결과

- ✅ `app_settings` 테이블 존재 (V008 created, V200 seeded — `cloud_folder_path` 등)
- ✅ `src/app/settings/codes/page.tsx` 만 존재 — `/settings` 허브 페이지 부재
- ✅ `src/lib/menu-config.ts` 의 `MENU_ITEMS` 에 `'설정'` 항목이 `/settings/codes` 로 하드코딩 — 허브로 변경 필요
- ✅ Sidebar 컴포넌트는 `MENU_ITEMS` 만 참조 (단일 SSOT) — `menu-config.ts` 1회 수정으로 전파

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `src-tauri/src/commands/settings.rs` (신규) | [0회] | `get_operating_hours()` / `save_operating_hours()` IPC + 단위 테스트 |
| `src-tauri/src/commands/mod.rs` | [0회] | settings 모듈 등록 |
| `src-tauri/src/lib.rs` | [0회] | `invoke_handler!` 에 신규 커맨드 등록 |
| `src/types/settings.ts` (신규) | [0회] | `OperatingHours`, `DayHours` 타입 정의 |
| `src/lib/tauri/index.ts` | [0회] | `getOperatingHours()` / `saveOperatingHours()` 래퍼 |
| `src/app/settings/page.tsx` (신규) | [0회] | 설정 허브 (운영 시간 / 코드 테이블 / 마법사 재실행 링크) |
| `src/app/settings/hours/page.tsx` (신규) | [0회] | 요일별 시작/종료 편집 (1시간 단위 콤보) |
| `src/lib/menu-config.ts` | [0회] | `'설정'` href → `/settings` (허브) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` / `SETUP.sh` / `docs/harness-engineering/`
- ⬜ DB 마이그레이션 (V201+) — T2 는 schema-less app_settings key/value 만 사용
- ⬜ `src-tauri/src/commands/setup.rs` — 마법사 (T2 는 사용 중 영구 설정이라 분리)
- ⬜ `docs/sprint/sprint4.md` / `ROADMAP.md` — 계획 문서

## 완료 기준 (T2 DoD) — 모두 충족

- ✅ `get_operating_hours()` IPC — 저장값 없으면 디폴트(월~금 13:00~19:00, 토/일 미운영) fallback
- ✅ `save_operating_hours(hours: Vec<DayHours>)` IPC — 7요일 검증 + HH:MM 형식 검증 + JSON upsert
- ✅ `src/app/settings/page.tsx` — 허브 카드 4 종 (운영 시간 / 코드 테이블 / 교습소 정보 비활성 / 마법사 재실행 비활성)
- ✅ `src/app/settings/hours/page.tsx` — 요일별 시작/종료 콤보 + 미운영 토글 + 저장 버튼. 콤보 범위 10:00~20:00 (사용자 요청)
- ✅ `menu-config.ts` "설정" → `/settings` 허브로 갱신
- ✅ `cargo test commands::settings` 6 건 신규 통과 (default 검증 3종 + serde roundtrip 2종 + vec 형식)
- ✅ `tsc --noEmit` + `next lint --max-warnings 0` + `cargo clippy -D warnings` 모두 EXIT 0
- ✅ `pnpm tauri:dev` 빌드 성공 (시각 검증은 사용자가 별도 확인)
- ✅ simplify — T2 변경분 단순화 여지 없음 (CRUD 흐름 명확, 추출 premature)
- ⬜ T2 단일 통합 커밋 ← 진행 중

## 적용 스킬

- T2 는 sprint4.md 에 skill 미명시 — 일반 implementation. 신규 IPC 패턴은 기존 setup.rs / codes.rs 참조

## 발견된 이슈

(미발생 — 발생 시 본 섹션에 기록)

## 이전 세션 (#1) 완료 항목

- ✅ T1 — window.confirm → shadcn AlertDialog (`a06dbd6`)
- ✅ shadcn 초기화 트랩 해소 (R23 발현 후 globals.css 토큰 13 종 수동 추가)
- ✅ msw 빌드 차단 해소 (pnpm-workspace.yaml allowBuilds.msw=false)
- ✅ 사용자 시각 검증 통과 (AlertDialog 가독성 + 취소 버튼 동작)
- 기록: Next.js 15.3.2 CVE-2025-66478 별도 hotfix 후보

## 다음 세션 진입점 (예정)

T2 완료 → **T3 (상태바 점유/백업/동기화 + 시작시간 표시 수정)** 진입. systematic-debugging 스킬 적용 (IPC 응답 미연결 추적). session #3 scope.md 작성.
