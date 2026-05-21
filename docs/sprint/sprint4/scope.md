---
Sprint: 4  |  Date: 2026-05-21  |  Session: #1
---

## 세션 #1 목표

**T1: Critical — window.confirm 차단 해소 + shadcn/ui AlertDialog 도입 + capabilities 권한 정비**

본 세션은 T1만 다룬다. T1은 Tauri 2.x runtime error 로 Sprint 3 스테이징 검증을 차단하던 최우선 이슈. 회고 A9 + A11 carry-over 통합.

## 사전 확인 결과 (2026-05-21 11:50 KST)

- ⚠️ **R23 발현**: `components.json` 없음, `src/components/ui/` 디렉토리 없음 → shadcn 미초기화. T1 첫 단계로 `npx shadcn@latest init` 선행 필요.
- ✅ `window.confirm` / `window.alert` / `window.prompt` 전수 조사 결과 단 1곳 (`src/app/students/edit/page.tsx:65`). 일괄 교체 부담 최소.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `components.json` (신규) | [0회] | `npx shadcn@latest init` 산출물 |
| `src/lib/utils.ts` (신규) | [0회] | shadcn init 의 `cn` 헬퍼 |
| `src/components/ui/alert-dialog.tsx` (신규) | [0회] | `npx shadcn@latest add alert-dialog` 산출물 |
| `src/app/globals.css` | [0회] | shadcn init 이 CSS 변수 추가 (테마 토큰) |
| `tailwind.config.ts` 또는 `.js` | [0회] | shadcn init 이 plugins/content 갱신 가능 |
| `package.json` / `pnpm-lock.yaml` | [0회] | Radix UI primitive (`@radix-ui/react-alert-dialog`) + `clsx`/`tailwind-merge` 자동 추가 |
| `src/app/students/edit/page.tsx` | [0회] | `window.confirm` → `AlertDialog` 교체 (line 65 영역) |
| `src-tauri/capabilities/default.json` | [0회] | `dialog:allow-open` 유지 + A9 권한 좁히기 검토 |

> 위 8개 파일이 T1 의 핵심 scope. shadcn init 산출물은 사전 예측이 일부 어려워, 실제 명령 실행 후 30% 초과 변경 발생 시 본 scope.md 를 갱신한다.

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook 차단)
- ⬜ `docs/harness-engineering/` — Harness 정책 문서
- ⬜ `src-tauri/src/commands/`, `src-tauri/migrations/` — 본 세션은 백엔드 비즈니스 로직/DB 무변경
- ⬜ `docs/sprint/sprint4.md`, `ROADMAP.md` — 계획 문서 (구현 중 변경 금지)

## 완료 기준 (이번 세션 = T1 DoD) — 모두 충족

- ✅ shadcn 초기화 (`components.json` + `src/lib/utils.ts` 수동 보완 + `src/components/ui/` 생성). SSL 차단은 `NODE_OPTIONS="--use-system-ca"` 로 우회
- ✅ `src/components/ui/alert-dialog.tsx` + `button.tsx` 추가 (Base UI primitive 기반 — shadcn v4 의 Base UI 채택)
- ✅ `src/app/students/edit/page.tsx` `window.confirm` → AlertDialog 교체 (44×44px 영역·Pretendard 상속)
- ✅ `src-tauri/capabilities/default.json` 변경 없음 — Tauri 2.x dialog 권한 단위 분리 불가 확인 (`message` / `open` / `save` 3 명령만, `open` 은 file/directory 인자 구분). R21 갱신 예정 (T11)
- ✅ `pnpm tauri:dev` 실행 후 AlertDialog 표시 + 가독성 확인 + 취소 버튼 동작 확인
- ✅ `tsc --noEmit` + `next lint --max-warnings 0` EXIT 0
- ✅ simplify 적용 (단일 사용처 추출 premature, 변경분 단순화 없음 — 그대로 커밋)
- ⬜ 단일 통합 커밋 (의도 명확한 한국어 메시지) ← 진행 중

## 적용 스킬

- **systematic-debugging** (sprint4.md T1 명시) — dialog runtime error 의 차단 메커니즘 (Tauri 2.x security policy + WebView native dialog 차단) 확인 후 진행

## 발견된 이슈 (실제 진행 중 발생)

### shadcn init 부분 동작 — R23 발현
1. **SSL 인증서 검증 실패**: `unable to verify the first certificate` (사내 환경 의심). `NODE_OPTIONS="--use-system-ca"` 로 우회.
2. **globals.css 변수 미추가**: init 이 CSS 변수를 안 넣어 `bg-popover`/`bg-muted`/`text-muted-foreground` 가 모두 미정의 → 다이얼로그 투명+무색 렌더링. **수동으로 globals.css `:root` + `@theme` 에 13개 토큰 추가** (popover/card/muted/primary/secondary/destructive/input/ring/radius/foreground 변형).
3. **src/lib/utils.ts 미생성**: shadcn add 가 utils.ts 안 만들어 수동 작성 (cn = clsx + tailwind-merge).

### msw transitive dep 빌드 차단
shadcn 신규 deps 가져온 후 pnpm 10 이 `runDepsStatusCheck` 에서 msw ignored builds 로 install exit 1 → tauri:dev 진입 차단. `package.json` 의 `pnpm.ignoredBuiltDependencies: ["msw"]` 명시로 해소. shadcn 도 devDependencies 로 이동(CLI 도구, prod 번들 제외).

### Next.js 15.3.2 CVE-2025-66478 경고
pnpm install 시 deprecated 경고. **Sprint 4 scope 외 — 별도 hotfix 후보로 기록** (release 전 업그레이드 필수).

## 다음 세션 진입점 (예정)

T1 완료 → **T2 (교습소 설정 메뉴)** 진입. 백엔드 IPC 신규 (`src-tauri/src/commands/settings.rs`) + 프론트 화면 신설 (`src/app/settings/page.tsx`, `src/app/settings/hours/page.tsx`). session #2 scope.md 작성.
