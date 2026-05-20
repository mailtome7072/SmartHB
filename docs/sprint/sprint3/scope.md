---
Sprint: 3  |  Date: 2026-05-20  |  Session: #1
---

## 세션 #1 목표

Sprint 3 진입 + Day 1 (T1 Pretendard self-host + T2 R13 PII 마스킹) 완료.

## 사용자 결정 (Auto Mode 자동 채택, redirect 환영)

- **A5 마이그레이션 V{NNN} 표기 통일**: **옵션 (c) 보류** — V200부터 V prefix 적용, 기존 V001/V008/V101~V105는 v1.0 전 일괄 정리.
- **Sprint 3 범위**: **Option 1** — 마법사 + R12/R13/R14 + ROADMAP Sprint 3 전체 (plan 권장안).

## 이번 세션에서 수정할 파일

> 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행.

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `public/fonts/` (디렉토리 신규) | [0회] | T1 — Pretendard woff2 배치 |
| `src/app/globals.css` | [0회] | T1 — @font-face + body/heading 기본값 |
| `tailwind.config.ts` | [0회] | T1 — fontFamily.sans 등록 |
| `src-tauri/src/commands/students.rs` | [0회] | T2 — try_record 3곳 details=None |
| `src-tauri/src/commands/audit.rs` (테스트 추가용) | [0회] | T2 — PII 미포함 단위 테스트 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- ⬜ `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- ⬜ `docs/harness-engineering/` — Harness 정책 (정책 약화 방지)
- ⬜ `src-tauri/migrations/` (이번 세션) — V200 신설은 T8(Day 5~6) 이후
- ⬜ `src-tauri/src/commands/setup.rs` (이번 세션) — T8 도달 시 생성
- ⬜ `src/app/setup/`, `src/app/students/`, `src/components/layout/` (이번 세션) — Day 3 이후 Task

## 완료 기준 (이번 세션)

- ✅ `sprint3` 브랜치 생성 (develop 기반) + planning artifacts 커밋 (`2905663`)
- ✅ T1 Pretendard self-host 완료 (`7d8af2c`) — Regular/SemiBold/Bold subset 806KB / 본문 18px / 헤더 24px+ / 행간 1.5 / `pnpm build` 통과
- ✅ T2 R13 PII 마스킹 완료 (`6766693`) — `audit::try_record` 3곳 `details=None` / `cargo test` 97 passed 회귀 0 / clippy 통과
- ✅ 다음 세션 진입점 (아래 “다음 세션 진입점” 섹션) 명시

## 다음 세션 진입점 — T3 (R14 페이지네이션)

- **대상**: `src-tauri/src/commands/students.rs`, `src-tauri/src/commands/codes.rs`, `src/lib/tauri/index.ts`, `src/types/student.ts`
- **변경 요지**:
  - `StudentFilter`에 `limit: Option<u32>` / `offset: Option<u32>` 추가
  - `list_students` SQL에 `LIMIT ? OFFSET ?` 적용 (기본 limit=100, 상한 1000)
  - `count_students(filter)` 신규 IPC (총 건수 반환 — 동일 필터 재사용)
  - `list_codes` / `count_codes` 동일 적용
  - 프론트 IPC 래퍼: `src/lib/tauri/index.ts` 의 `listStudents`/`listCodes` 시그니처 갱신 + `countStudents`/`countCodes` 추가
- **단위 테스트 신규**: limit/offset 경계값 + count 정확성 (in-memory pool)
- **세션 시작 시 확인**: `git log develop..HEAD --oneline` 으로 T1/T2 커밋 존재 확인 + `scope.md` 세션 번호 +1 (#2)

## 발견된 이슈

(없음)

## 세션 #1 simplify 적용 사항

- 코드 리뷰 3개 에이전트 병렬 실행 → 채택 2건:
  - `tailwind.config.ts`의 `fontFamily.sans` 제거 (Tailwind 4 CSS-first `@theme` 중복)
  - `globals.css`의 미사용 변수 `--font-size-heading`, `--tap-target-min` 제거 (YAGNI)
- 기각 1건: audit::try_record 헬퍼 추출 (premature abstraction — 3줄 호출에 indirection 추가 불요)

## Re-planning 트리거 추적

- 실제 수정 파일이 위 선언 대비 30% 초과 시 → 본 scope.md 업데이트 + 사용자 보고
- 새 의존성/DB 변경 필요 발생 시 → 동일
