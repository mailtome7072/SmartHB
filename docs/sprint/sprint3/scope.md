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

- ⬜ `sprint3` 브랜치 생성 (develop 기반) + planning artifacts 커밋 (sprint3.md, risk-register, sprint-planner MEMORY 갱신, CLAUDE.md cipher feature 추가, ROADMAP 갱신)
- ⬜ T1 Pretendard self-host 완료 — `pnpm build` 통과 + 본문 18px / 헤더 24px+ / 행간 1.5 기본값 확립
- ⬜ T2 R13 PII 마스킹 완료 — `cargo test` 통과 + audit_logs.details에 원생 이름 미저장 단위 테스트 추가
- ⬜ 세션 종료 시 다음 세션 진입점(T3 페이지네이션)을 본 scope.md에 명시

## 발견된 이슈

(없음 — 발견 시 본 섹션에 기록 후 사용자 보고)

## Re-planning 트리거 추적

- 실제 수정 파일이 위 선언 대비 30% 초과 시 → 본 scope.md 업데이트 + 사용자 보고
- 새 의존성/DB 변경 필요 발생 시 → 동일
