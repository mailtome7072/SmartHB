---
Sprint: 2  |  Date: 2026-05-20  |  Session: #1
---

## 세션 진행 기록

- **Session #1** (T1 루트 라우팅 + 인증 게이트): 🔄 진행 중 (현재)

## 이번 세션의 목표 (T1 — Day 1)

**루트 라우팅 + 인증 게이트 미들웨어** (Sprint 1 잔여)

PRD §5.6 인수 기준 "최초 실행 시 비밀번호 설정 화면 자동 진입" 충족.

### 흐름

```
앱 시작 → src/app/page.tsx (root)
    ├── checkAuthStatus() 호출
    ├── not-initialized → /lock?mode=setup redirect
    ├── locked        → /lock redirect
    └── unlocked (메모리 상태) → 메인 화면 진입

/lock 페이지
    ├── 비밀번호 입력
    ├── 성공 시 app_startup_sequence(password) 호출
    └── startup 성공 시 / 으로 redirect
```

### 구현 포인트

- **`src/app/page.tsx`**: 클라이언트 컴포넌트로 변경. `useEffect` + `checkAuthStatus()` IPC 호출 → 분기 redirect. 데모 `greet` 코드 제거.
- **`src/app/lock/page.tsx`**: 인증 성공 콜백에서 `app_startup_sequence(password, force_lock=false)` 호출 추가. 결과 `StartupResult` 의 `elapsed_ms` 검토 (3초 초과 경고).
- **`src/components/LockScreen.tsx`**: 기존 컴포넌트 그대로 — onSuccess 콜백 시그니처만 확장 (필요 시).
- **인증 후 메모리 상태**: 단순 module-scope 변수 또는 Zustand. Sprint 2 에서 Zustand 도입은 보류 (T1 에서 Zustand 도입까지 하면 범위 초과). React state + sessionStorage 또는 단순 module 변수로 처리.
- **에러 핸들링**: 모든 IPC 호출 try/catch + 사용자 친화 한국어 메시지 표시 (LockScreen 의 errorMessage state 활용).

### Next.js static export 제약

- `src/middleware.ts` 는 static export 에서 동작하지 않음 — root layout 또는 page.tsx 의 클라이언트 가드 패턴 사용
- `next/navigation` 의 `useRouter().replace('/lock')` 활용
- `'use client'` 지시어 필수

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint2/scope.md | [0회] | 본 파일 — Session #1 생성 |
| ROADMAP.md | [0회] | sprint-planner 결과물 (sprint2 ✅) — 첫 커밋에 포함 |
| docs/sprint/sprint2.md | [0회] | sprint-planner 결과물 — 첫 커밋에 포함 |
| docs/risk-register/2026-05-20.md | [0회] | sprint-planner 결과물 — 첫 커밋에 포함 |
| .claude/agents/agent-memory/sprint-planner/MEMORY.md | [0회] | sprint-planner 결과물 — 첫 커밋에 포함 |
| src/app/page.tsx | [0회] | T1 — 데모 greet 제거 + checkAuthStatus 가드 |
| src/app/lock/page.tsx | [0회] | T1 — 인증 성공 시 app_startup_sequence 호출 |
| src/components/LockScreen.tsx | [0회] | T1 — onSuccess 시그니처 확장 (필요 시) |
| src/lib/auth-state.ts | [0회] | (신규 가능) — 모듈 스코프 인증 상태 헬퍼 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (사용자 허가 후만 변경 가능)
- ⬜ `SETUP.sh` — 초기화 스크립트
- ⬜ `docs/harness-engineering/`, `.claude/agents/` (sprint-planner 메모리 외) — 정책·에이전트
- ⬜ `PRD.md`, `docs/phase/`, `docs/sprint/sprint2.md` (sprint-planner 결과물 외 변경 금지)
- ⬜ `.env`, `src-tauri/migrations/` (T5~T7 전까지 변경 없음)
- ⬜ `src-tauri/src/commands/*.rs` (T1 은 frontend 한정)

## 이번 세션의 완료 기준 (T1)

- ⬜ `src/app/page.tsx` 데모 greet 코드 제거 + checkAuthStatus 분기 redirect
- ⬜ `src/app/lock/page.tsx` 인증 성공 시 `app_startup_sequence` 호출 + 메인 redirect
- ⬜ 모든 IPC 호출 try/catch + 한국어 에러 메시지
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- ⬜ 첫 진입 / 재시작 / 인증 성공 후 흐름 모두 로컬에서 시연 가능 (수동 검증은 사용자 일정 시)

## 다음 세션 예정 (참고)

- **Session #2**: T2 R6 salt 이전 준비 + T3 R7 release_lock + T4 R8 startup 측정 (Day 2 묶음, 총 1.5일)
- **Session #3**: T5 V101 마이그레이션 (Day 3, 1일)
- **Session #4+**: T6~T8 마이그레이션 마무리 + T9 원생 CRUD IPC 시작

본 scope.md 는 각 세션 시작 시 갱신한다.
