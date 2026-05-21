---
Sprint: 4  |  Date: 2026-05-21  |  Session: #3
---

## 세션 #3 목표 — 병렬 묶음 (T3 + T10 + T5 utils)

서로 도메인이 완전히 독립적인 3개 항목을 한 세션에서 순차 처리:

1. **T5 (utils 신규 파일만)** — `src/lib/format.ts` 작성 (formatPhone + formatCurrency). 적용은 세션 #4 students 도메인에서.
2. **T3** — TopBar 상태바 락/백업/동기화 + 시작시간 표시 (사용자 이슈 #1, #2)
3. **T10** — 코드 테이블 DnD + 활성 필터 (사용자 이슈 #11, #12, dnd-kit R24 검증)

각 Task 완료 시 개별 self-verify + 단위 커밋.

## 사전 확인 (root cause 식별)

### T3
- `setLockStatus` 호출자가 **앱 전체에 없음** (LockWarning 만 `checkLockStatus` 호출, store 미반영) → TopBar 영원히 "확인 중..."
- TopBar 는 현재 lock 만 표시. 백업/동기화 컬럼 신규 추가 필요
- `lastStartup` 표시는 `src/app/page.tsx` 메인에 이미 있음 — 단 markUnlocked 호출 후 표시. 표시 자체는 작동할 가능성 — 사용자 시각 미확인 (다이얼로그 차단으로 못 본 것 의심)

### T10
- `@dnd-kit/core` + `@dnd-kit/sortable` 미설치 — R24 (React 19 호환) 검증 필요
- `src/app/settings/codes/page.tsx` 현황: 학교/표준교습비/결제수단/카드사 4 탭, sort_order 수정 가능하나 버튼/숫자 입력. DnD 미구현
- 활성 필터 (전체/사용/미사용 라디오) 미구현 — is_active 컬럼은 V105 부터 schools, codes 에 있음
- 백엔드 `reorder_codes` IPC 는 Sprint 3 에서 도입 — 다중 항목 일괄 sort_order 갱신 시그니처 확인 필요

### T5 utils
- `src/lib/` 아래에 format.ts 없음 — 신규 작성

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | Task | 비고 |
|------|---------|------|------|
| `src/lib/format.ts` (신규) | [0회] | T5 | formatPhone / formatCurrency + 단위 테스트는 vitest 부재로 생략 (적용 시 사용처에서 행동 검증) |
| `src/components/layout/app-shell.tsx` | [0회] | T3 | useEffect 로 checkLockStatus 호출 + setInterval polling |
| `src/components/layout/top-bar.tsx` | [0회] | T3 | 백업/동기화 컬럼 추가, lastStartup ms 표시 |
| `src/stores/app-store.ts` | [0회] | T3 | (필요 시) backupAt / syncStatus state 추가 |
| `package.json` / `pnpm-lock.yaml` | [0회] | T10 | @dnd-kit/core + @dnd-kit/sortable 추가 |
| `src/app/settings/codes/page.tsx` | [0회] | T10 | DnD wrapper + 활성 필터 라디오 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` / `SETUP.sh` / `docs/harness-engineering/`
- ⬜ DB 마이그레이션 — T3/T10 모두 schema 변경 없음
- ⬜ `src-tauri/src/commands/codes.rs` — reorder_codes 기존 시그니처가 충분하면 변경 없음. 부족하면 scope 확장 보고
- ⬜ `src/components/students/**` — students 도메인은 세션 #4

## 완료 기준 (세션 #3 DoD)

### T5 utils
- ⬜ `src/lib/format.ts` 작성 — formatPhone(휴대폰/지역번호 + 일반전화 패턴), formatCurrency(ko-KR Intl)
- ⬜ tsc + lint 통과
- ⬜ 개별 커밋

### T3
- ⬜ AppShell useEffect 에서 checkLockStatus → setLockStatus, 60초 polling
- ⬜ TopBar 백업/동기화 + 시작시간 ms 표시 (page.tsx 분리 또는 store 추가)
- ⬜ "확인 중..." 무한 표시 회귀 검증
- ⬜ dev 빌드 + 사용자 시각 검증
- ⬜ 개별 커밋

### T10
- ⬜ `pnpm add @dnd-kit/core @dnd-kit/sortable` — React 19 호환 확인 (peer dep 또는 install error)
- ⬜ codes/page.tsx 에 DnD 적용 (4 탭 모두) — drag handle + 행 위치 이동 → reorder_codes 호출
- ⬜ 화면 상단에 전체/사용/미사용 라디오 — 클라이언트 filter (가능하면 백엔드 list_codes 시그니처 활용)
- ⬜ 신규 추가 항목이 맨 마지막 sort_order 부여 (사용자 이슈 #11 후반)
- ⬜ dev 빌드 + 사용자 시각 검증
- ⬜ 개별 커밋

## 적용 스킬

- **T3 systematic-debugging** (sprint4.md T3 명시) — IPC 미연결 추적 완료, fix 진행
- T5/T10 일반 implementation

## 발견된 이슈 (실시간 기록)

### T3 root cause (확정)
`setLockStatus` 호출자 부재 — store 도입(Sprint 3 T4) 이후 어떤 컴포넌트도 store 갱신을 안 함. lock-store 갱신을 책임지는 단일 컴포넌트(AppShell) 도입으로 해소.

## 이전 세션 (#2) 완료 항목

- ✅ T2 — 교습소 설정 메뉴 + 운영 시간 (`04131a9`)
- ✅ HOUR_OPTIONS 10:00~20:00 범위 적용

## 다음 세션 진입점 (예정)

세션 #4 — **students 도메인 직렬 묶음** (T4 → T6 → T7 → T8). T5 utils 적용도 흡수. T8 에 DB 마이그레이션 V201 (students.withdrawn_at) 포함.
