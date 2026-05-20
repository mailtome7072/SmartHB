---
Sprint: 3  |  Date: 2026-05-20  |  Session: #2
---

## 세션 #2 목표

T3 R14 페이지네이션 완료 — `list_students`/`list_codes`에 `limit/offset` + `count_*` 신규 IPC + 프론트 IPC 래퍼 + 타입 갱신.

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `src-tauri/src/commands/students.rs` | [0회] | StudentFilter에 limit/offset 추가, list_students SQL 갱신, count_students 신규 |
| `src-tauri/src/commands/codes.rs` | [0회] | list_codes에 limit/offset 적용, count_codes 신규 |
| `src-tauri/src/lib.rs` | [0회] | count_students/count_codes invoke_handler 등록 |
| `src/lib/tauri/index.ts` | [0회] | listStudents 시그니처 갱신, countStudents/listCodes(limit·offset)/countCodes 추가 |
| `src/types/student.ts` | [0회] | StudentFilter에 limit/offset 필드 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/` — Harness 정책
- ⬜ `src-tauri/migrations/` (이번 세션) — V200 마이그레이션은 T8 도달 시
- ⬜ `src/app/setup/`, `src/app/students/`, `src/components/layout/` (이번 세션) — Day 3 이후 Task

## 완료 기준 (이번 세션)

- ✅ StudentFilter 에 `limit`/`offset` 추가 (codes 는 별도 인자 — over-engineering 회피)
- ✅ `list_students` / `list_codes` SQL 에 `LIMIT ? OFFSET ?` 적용
- ✅ `count_students` / `count_codes` 신규 IPC
- ✅ `lib.rs` invoke_handler 등록
- ✅ 프론트 래퍼 갱신 (`listStudents` / `listCodes` / `countStudents` / `countCodes`)
- ✅ TypeScript 타입 갱신
- ✅ 단위 테스트 — pagination 모듈 2건 + students 3건 + codes 2건 (총 +7)
- ✅ Self-verify: cargo test 106 passed / clippy / lint / tsc / build 모두 통과
- ✅ simplify — `pagination.rs` 공유 모듈 추출, `build_filter_clause` 시그니처 정리

## simplify 적용 사항

- 3-agent 병렬 리뷰 → 채택 2건, 기각 1건:
  - **채택**: `DEFAULT_LIST_LIMIT`/`MAX_LIST_LIMIT` + clamp 로직 중복(students/codes) → `commands/pagination.rs` 신규 모듈로 통합 (audit.rs 도 동일 패턴이지만 scope 외 — 별도 sweep)
  - **채택**: `build_filter_clause` 반환 `(WHERE, bool)` → `(WHERE, JOIN)` 두 String — 호출부의 if 분기 제거
  - **기각**: `listCodes` 도 CodeFilter 구조체로 통일 — codes 에는 다른 필터가 없어 over-engineering. 현재 `(table, limit, offset)` 인자 시그니처가 자연스러움.

## 발견된 이슈

(없음)

## 다음 세션 진입점 — T4 (Zustand + TanStack Query 셋업)

- **대상**: 신규 의존성 + 신규 디렉토리/파일
  - `pnpm add zustand @tanstack/react-query`
  - `src/stores/session-store.ts` (인증 여부, 디바이스 정보)
  - `src/stores/app-store.ts` (락 점유 상태, 선택된 교습기간월, 사이드바 열림/닫힘)
  - `src/providers/query-provider.tsx` (TanStack Query Provider client component)
  - `src/app/layout.tsx` Provider 래핑
- **신규 의존성 허가 필요**: sprint3.md 계획서에 명시된 의존성이므로 별도 사용자 허가 불요 (계획 단계에서 승인됨)
- **검증**: `pnpm tsc --noEmit` + `pnpm lint` 통과
- **세션 시작 시 확인**: `git log develop..HEAD --oneline` 으로 T1/T2/T3 커밋(7d8af2c, 6766693, 58aeab6) 존재 확인 + 세션 번호 +1 (#3)

---

## 세션 #1·#2 결과 (참고)

- ✅ `2905663` — Sprint 3 진입 (계획 + scope.md + CLAUDE.md cipher feature)
- ✅ `7d8af2c` — T1 Pretendard self-host (세션 #1)
- ✅ `6766693` — T2 R13 audit PII 마스킹 (세션 #1)
- ✅ `b955ff1` — 세션 #1 마감
- ✅ `58aeab6` — T3 R14 페이지네이션 (세션 #2)

---

## 세션 #1 결과 (참고)

- ✅ `2905663` — Sprint 3 진입 (계획 + scope.md + CLAUDE.md cipher feature)
- ✅ `7d8af2c` — T1 Pretendard self-host
- ✅ `6766693` — T2 R13 audit PII 마스킹
- ✅ `b955ff1` — 세션 #1 마감
