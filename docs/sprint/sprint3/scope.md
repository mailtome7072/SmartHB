---
Sprint: 3  |  Date: 2026-05-20  |  Session: #3
---

## 세션 #3 목표

T4 Zustand + TanStack Query 셋업 — 세션·앱 전역 store 도입, IPC 응답 캐싱 인프라 확립.

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `package.json` | [0회] | zustand, @tanstack/react-query 신규 의존성 |
| `src/stores/session-store.ts` (신규) | [0회] | 인증 여부 + 디바이스 정보 + StartupResult (auth-state.ts 대체) |
| `src/stores/app-store.ts` (신규) | [0회] | 락 점유, 사이드바 열림/닫힘, 선택된 교습기간월 |
| `src/providers/query-provider.tsx` (신규) | [0회] | TanStack Query Provider client component |
| `src/app/layout.tsx` | [0회] | Provider 래핑 |
| `src/app/page.tsx` | [0회] | auth-state → session-store 마이그레이션 |
| `src/app/lock/page.tsx` | [0회] | auth-state → session-store 마이그레이션 |
| `src/lib/auth-state.ts` | [0회] | 제거 (session-store가 대체) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/` — Harness 정책
- ⬜ `src-tauri/` (이번 세션) — T4는 프론트 전용
- ⬜ `src/app/setup/`, `src/app/students/`, `src/components/layout/` (이번 세션)

## 완료 기준 (이번 세션)

- ✅ `pnpm add zustand@5.0.13 @tanstack/react-query@5.100.11`
- ✅ session-store: `useSessionStore` (unlocked, lastStartup, markUnlocked)
- ✅ app-store: `useAppStore` (lockStatus, sidebarOpen, selectedPeriodMonth + setters/toggle)
- ✅ QueryProvider client component + layout.tsx 래핑
- ✅ page.tsx / lock/page.tsx — auth-state → session-store 마이그레이션
- ✅ `src/lib/auth-state.ts` 삭제
- ✅ Self-verify: pnpm lint / pnpm tsc / pnpm build 통과
- ✅ simplify — session-store 의 reset 액션 제거 (YAGNI)

## simplify 적용 사항

- 3-agent 병렬 리뷰 → 채택 1건:
  - **채택**: `session-store.ts` 의 `reset` 액션 미사용 → 제거 (YAGNI — T5 이후 필요 시 재도입)
- 나머지 항목은 모두 ✓ 또는 "no issues":
  - app-store 의 forward-looking 필드(lockStatus/sidebarOpen/selectedPeriodMonth) — T5/T8 직접 소비 예정이라 premature 아님
  - QueryClient defaults (staleTime 30s, retry 1, refetchOnWindowFocus false) — Tauri 데스크톱 환경에 적절
  - page.tsx 의 dual selector 패턴 — Zustand 권장 패턴

## 발견된 이슈

(없음)

## 다음 세션 진입점 — T5 (앱 레이아웃 셸)

- **대상**: 신규 파일 + page.tsx 리팩토링
  - `src/components/layout/sidebar.tsx`: 메뉴 항목 + 단축키 표기 (대시보드/원생/수업/출결/청구/단원평가/학습보고서/공지문/설정). Phase 2+ 메뉴는 disabled 처리 + "다음 업데이트 예정" 툴팁
  - `src/components/layout/top-bar.tsx`: 점유 디바이스, 마지막 백업 시각, 동기화 상태 (app-store.lockStatus 소비)
  - `src/components/layout/app-shell.tsx`: sidebar + top-bar + 콘텐츠 영역 조합
  - `src/app/layout.tsx`: 인증 완료 후 AppShell 렌더링 — 현재는 page.tsx 내부에서만 분기 처리 중. layout 자체에는 AppShell 을 넣지 않고 page.tsx 가 AppShell 로 감싸도록 유지(/lock 등 비-AppShell 페이지 분기)
- **저자극 톤**: 베이지/연그레이 배경, 명도 대비 4.5:1 이상, 18pt/44×44px 충족
- **검증**: pnpm build + 시각 확인
- **세션 시작 시 확인**: `git log develop..HEAD --oneline` 으로 T1~T4 커밋(7d8af2c, 6766693, 58aeab6, c441f5c) 존재 확인 + 세션 번호 +1 (#4)

---

## 세션 #1·#2·#3 결과 (참고)

- ✅ `2905663` — Sprint 3 진입
- ✅ `7d8af2c` — T1 Pretendard self-host
- ✅ `6766693` — T2 R13 audit PII 마스킹
- ✅ `b955ff1` — 세션 #1 마감
- ✅ `58aeab6` — T3 R14 페이지네이션
- ✅ `db3ca53` — 세션 #2 마감
- ✅ `c441f5c` — T4 Zustand + TanStack Query 셋업

---

## 세션 #1·#2 결과 (참고)

- ✅ `2905663` — Sprint 3 진입
- ✅ `7d8af2c` — T1 Pretendard self-host
- ✅ `6766693` — T2 R13 audit PII 마스킹
- ✅ `b955ff1` — 세션 #1 마감
- ✅ `58aeab6` — T3 R14 페이지네이션
- ✅ `db3ca53` — 세션 #2 마감
