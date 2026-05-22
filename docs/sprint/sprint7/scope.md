---
Sprint: 7  |  Date: 2026-05-22  |  Session: #5
---

> Sprint 7 Session #5 — T5 단독 (학사 일정 코드 관리 /settings 이동).
> Issue 3 carry-over 해소. 설정성 ↔ 운영성 작업 분리. 예상 3h.

## 이전 세션 결과

- Session #1 (`8eb1c92`): T1 — Keychain 통합 캐싱
- Session #2 (`4178324`): T2 — salt.bin 이전 + 보안 패치 6건 + I-S2-1
- Session #3 (`2fad4fb`): T3 — device_id 영속화
- Session #4 (`6b5f8de`): T4 — is_system_reserved JOIN + 프론트 하드코딩 제거 + I-S4-1
  - cargo test cipher off 166 / on 127, pnpm lint+tsc clean

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T5** | 학사 일정 코드 관리 `/settings/schedule-codes` 페이지 분리 + `/academic` 코드 패널 → 컴팩트 selector | 3h |

> 사용자 결정 (2026-05-22): Session #5 = T5 단독.

## 설계 결정 (T5)

### 분리 원칙
- **설정성 작업** (코드 CRUD, 활성 토글, 시스템 코드 🔒): `/settings/schedule-codes` 신규 페이지로 이동
- **운영성 작업** (일정 배치 시 활성 코드 선택): `/academic` 페이지 내 컴팩트 selector
- 두 페이지 모두 동일 데이터 (`schedule_codes` 테이블) 를 TanStack Query 로 공유 — 한쪽 변경 시 다른 쪽 자동 invalidate

### 컴포넌트 책임 분리
- `ScheduleCodePanel` (기존): `src/components/academic/` 위치 유지 — CRUD 전용 컴포넌트
  - `selectedCodeId` / `onSelect` props 를 **옵셔널** 로 변경 — settings 페이지에서는 선택 기능 없이 사용
  - settings 페이지에서 props 생략 시 selection 시각화 비활성 (카드 클릭 무동작)
- `ScheduleCodeSelector` (신규): `/academic` 페이지 전용 컴팩트 selector
  - **활성 사용자 코드만** 표시 (시스템 코드는 자동 배치되므로 일정 배치 selection 대상 외)
  - 드롭다운 또는 라디오 그룹 형태 (44px 클릭 영역, 18pt 폰트)
  - "설정에서 관리" Link → `/settings/schedule-codes`

### `/academic` 페이지 변경
- `ScheduleCodePanel` import + 마운트 제거
- `ScheduleCodeSelector` import + 마운트
- `selectedCode` state 유지 (배치 모드 진입에 사용)
- 기존 `EventPlacer` / 캘린더 동작은 영향 없음

### `/settings` 허브 페이지
- `CARDS` 배열에 `{ href: '/settings/schedule-codes', title: '학사 일정 코드 관리', description: '공휴일·보강데이 등 시스템 코드 + 사용자 추가 코드의 활성 토글 및 CRUD' }` 추가

### 신규 의존성
- 없음 — 기존 Next.js App Router + shadcn/ui 구성.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/academic/ScheduleCodePanel.tsx | [2회] | props 옵셔널화 (selectedCodeId, onSelect) |
| src/app/settings/schedule-codes/page.tsx | [5회 ⚠️] | 신규 — CRUD 전용 페이지 |
| src/app/settings/page.tsx | [0회] | CARDS 배열에 항목 추가 |
| src/components/academic/ScheduleCodeSelector.tsx | [1회] | 신규 — 컴팩트 selector |
| src/app/academic/page.tsx | [0회] | Panel → Selector 교체 |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/**` — 본 세션 백엔드 변경 없음
- [ ] `src/types/academic.ts` — 타입 변경 없음
- [ ] `src/lib/tauri/index.ts` — IPC 인터페이스 변경 없음

## 완료 기준 (이번 세션)

### T5 — 학사 일정 코드 관리 /settings 이동 (sprint7.md L151-172)
- ✅ AC-T5-1: `/settings/schedule-codes` 페이지에서 코드 CRUD 전체 동작 — ScheduleCodePanel 재사용 (props 옵셔널화)
- ✅ AC-T5-2: `/academic` 페이지에서 ScheduleCodePanel import + 마운트 제거
- ✅ AC-T5-3: `/academic` 일정 배치 시 ScheduleCodeSelector (활성 사용자 코드 + radiogroup) 사용
- ✅ AC-T5-4: `/settings` 허브 CARDS 에 "학사 일정 코드 관리" 항목 추가
- ✅ AC-T5-5: Selector 의 min-h-[44px] + text-base + 시맨틱 색상 토큰 — 기존 접근성 정책 준수

### 세션 종료 조건
- ✅ Self-verify: pnpm lint clean / pnpm tsc --noEmit clean / cargo check clean
- ✅ simplify 검토 — Selector 가 활성 사용자 코드 필터 + radiogroup 만 담당, CRUD 책임은 Panel 로 완전 분리. stale 주석 2건(academic/page.tsx, EventPlacer.tsx) 정정.
- ⬜ 단일 커밋 (6파일 + scope.md)

## 발견된 이슈

(없음 — Step-back 트리거 발생 시 여기에 기록)

## carry-over

- Session #2 발견 9건 (I-S2-2 ~ I-S2-10) — 후속 세션 또는 hotfix
- Session #4 발견 1건 (I-S4-1: CalendarCell.tsx hasHoliday/hasAssessment 비즈니스 식별) — 후속
