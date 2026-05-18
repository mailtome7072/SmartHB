# 테스트 전략

> **역할**: 테스트 레벨, 도구, 통과 기준을 정의한다.
> **SSOT**: 검증 항목 및 자동화 기준은 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 5(검증 매트릭스)가 단일 소스(SSOT)다.

---

## 테스트 레벨

| 레벨 | 도구 | 경로 | 실행 시점 |
|------|------|------|----------|
| 백엔드 단위/통합 테스트 | `cargo test --manifest-path src-tauri/Cargo.toml` | `src-tauri/src/**/#[cfg(test)]` | CI (PR 체크) |
| 백엔드 정적 분석 | `cargo clippy -- -D warnings` + `cargo fmt --check` | `src-tauri/` | CI (PR 체크) |
| 프론트엔드 검증 | `pnpm tsc --noEmit` + `pnpm lint` + `pnpm build` | `src/` | CI (PR 체크) |
| 데스크톱 E2E 테스트 | Tauri WebDriver (`tauri-driver`) — PRD §6.5 UC-1~UC-6 | — | sprint-review / deploy-prod (도구 결정은 PI-11 참조) |
| 사용자 수용 테스트 (UAT) | 원장 2주 파일럿 (PRD §6.5) | 실데이터 일부 마이그레이션 | 릴리즈 후보 단계 |

## 통과 기준

- `cargo test` 전체 통과 필수 (PR merge 조건) — 비즈니스 규칙 100% 커버 (PRD §6.5)
- `cargo clippy -- -D warnings` 통과 필수
- `pnpm tsc --noEmit` + `pnpm lint` + `pnpm build` 전체 통과 필수
- 데이터 보안 Phase 완료 후: 락 / 백업 / 무결성 / 자가 진단 단위 테스트 통과 필수 (deployment-policy.md Category 5)

## 테스트 결과 기록

- 산출물: `docs/test-reports/YYYY-MM-DD.md`

## 참조

- 검증 매트릭스 (SSOT): [`docs/dev-process.md`](../docs/dev-process.md) 섹션 5
- CI 파이프라인: [`docs/ci-policy.md`](../docs/ci-policy.md) CI 파이프라인 섹션
- CI 워크플로우: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)
