---
Sprint: 6  |  Date: 2026-05-22  |  Session: #4
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — 프론트엔드 첫 진입 세션.
> T8(프론트엔드 IPC 래퍼 + 도메인 타입) — 백엔드 14 커맨드(T5/T6/T7)의 TypeScript 1:1 매핑.
> 예상 2h. T9 캘린더 컴포넌트 진입 직전 마지막 인프라 단계.

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 (A20 lock 재시도) | `2c5b8a1` |
| #1 | T3 (A21 paths.rs OnceLock 분리) | `c2be584` |
| #1 | T4 (A22 DnD 방법 B) | `83f19d1` |
| #2 | T5+T6 (academic.rs 신규 — study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | T7 (academic.rs 확장 — schedule_events 5) | `a4c380e` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T8** | 신규 `src/types/academic.ts` 11 타입 + `src/lib/tauri/index.ts` 14 래퍼 | 2h |

> TypeScript 파일 2개만 작업. 신규 의존성·마이그레이션 없음. 백엔드 시그니처는 변경하지 않음.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/types/academic.ts | [1회] | 신규 — StudyPeriod/ScheduleCode/ScheduleEvent + 평탄화 List + Create/Update payload |
| src/lib/tauri/index.ts | [2회] | Sprint 6 섹션 추가 — 14 IPC 래퍼 (dev mode fallback 포함) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/` — 본 세션 백엔드 변경 없음 (T5/T6/T7 시그니처 그대로 매핑)
- [ ] `package.json` / `Cargo.toml` — 신규 의존성 없음
- [ ] `src-tauri/migrations/` — 본 세션 마이그레이션 없음
- [ ] 기존 `src/types/*` 파일들 — 새 파일만 추가, 기존 타입 수정 금지

## 완료 기준 (이번 세션)

### T8 — IPC 래퍼 + 타입 (PRD §4.4, sprint6.md L236-251)
- ✅ AC-T8-1: 모든 IPC 래퍼가 dev mode fallback 포함 (`if (!inv) return ...`)
- ✅ AC-T8-2: TypeScript strict 모드 통과 — `pnpm tsc --noEmit` 클린
- ✅ AC-T8-3: **15** 래퍼가 백엔드 `src-tauri/src/commands/academic.rs` 15 커맨드와 1:1 대응 (lib.rs L75-89 등록 검증)

> 주: 기존 메모리에 "14 래퍼"로 적혀 있었으나 실제 백엔드는 15 커맨드(study_periods 6 + schedule_codes 4 + schedule_events 5 — 마지막 5에 `auto_place_assessment_dates` 포함). 메모리 표기 오류였으며 본 구현은 15 매핑이 정합.

### 세션 종료 조건
- ✅ T8 단일 커밋 `5941d24` (`src/types/academic.ts` 신규 90줄 + `src/lib/tauri/index.ts` Sprint 6 섹션 232줄 추가)
- ✅ Self-verify: `pnpm tsc --noEmit` exit 0 + `pnpm lint` "No ESLint warnings or errors"
- ✅ simplify(code-review) 1회 실행 — `[]` 반환 (5각도 분석, 1:1 매핑 + 타입 정합 + 등록 확인 후 신규 결함 없음)

## 설계 결정 (메모리 가이드 따름)

- **타입 1:1 매핑**: Rust `Option<T>` → TS `T | null` (Tauri serde 직렬화 기본). Rust `i64` → TS `number`. Rust `bool` → TS `boolean`. Rust `String` → TS `string`.
- **camelCase 인자 변환**: Tauri invoke args는 자동 camelCase ↔ snake_case 변환. 예: Rust `from_month` ↔ TS `fromMonth`.
- **payload 매개변수 명**: 백엔드가 `payload: CreateStudyPeriodPayload` 형태로 받으므로 TS도 `inv('create_study_period', { payload })` 패턴 사용.
- **반환 타입 패턴**:
  - `Result<T, String>` → TS `Promise<T>` (실패 시 throw)
  - `Result<Option<T>, String>` → TS `Promise<T | null>` (예: `get_study_period`)
  - `Result<Vec<T>, String>` → TS `Promise<T[]>`
  - `Result<(), String>` → TS `Promise<void>` (예: `delete_study_period`, `delete_schedule_event`)
- **dev mode fallback 규칙** (기존 코드 패턴 따름):
  - `Promise<T[]>` → 빈 배열 `[]`
  - `Promise<T | null>` → `null`
  - `Promise<void>` → `return`
  - `Promise<T>` (단일 객체) → 더미 객체 (payload 값 + 0/false/빈 문자열 채움)

## 코드 패턴 SSOT (기존 src/lib/tauri/index.ts 인용)

```ts
export async function listStudents(filter: StudentFilter = {}): Promise<Student[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_students', { filter }) as Promise<Student[]>
}

export async function createStudent(payload: NewStudent): Promise<Student> {
  const inv = await getInvoke()
  if (!inv) {
    return { id: 0, /* payload + 디폴트 */ ... }
  }
  return inv('create_student', { payload }) as Promise<Student>
}
```

## 발견된 이슈

> 코드 수정 중 예상 외 충돌·구조 발견 시 여기에 기록 후 사용자에게 보고 (step-back 프로토콜).
