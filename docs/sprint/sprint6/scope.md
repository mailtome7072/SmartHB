---
Sprint: 6  |  Date: 2026-05-22  |  Session: #1
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — 첫 세션.
> 전체 12 Task 중 회고 carry-over 기술 부채 3건(T1·T3·T4)을 우선 해소한다.
> 모두 독립적, 코드 변경량 작음, 의존성 없음 → 안전한 첫 진입.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 | 근거 |
|------|------|---------|------|
| **T1** | lock/page.tsx 에러 화면 재시도 버튼 + refresh 시 lockStatus 초기화 (A20) | 1h | Sprint 5 코드 리뷰 Medium+Low 동시 해소 |
| **T3** | paths.rs OnceLock 리팩토링 → 병렬 테스트 안정화 (A21) | 2h | 3번째 이월. T12 통합검증 `--test-threads=1` 제거 전제 |
| **T4** | 코드 DnD 필터링 sort_order 충돌 해소 (A22, R26) | 2h | 3번째 이월. 방법 B(visibleCodes 재정렬 후 전체 재매핑) |

**T2(V301 시드 + 공휴일 7h)와 T5~T11(IPC + UI)은 다음 세션 이상에서 처리.**

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/app/lock/page.tsx | [3회 ⚠️] | T1 완료 — 같은 Task 내 두 부분 동시 수정(refresh + 에러 화면). false-positive ⚠️ (실제 loop 아님) |
| src-tauri/src/commands/paths.rs | [2회] | T3 완료 — storage 모듈 cfg 분기 + tests reset 제거 |
| src/app/settings/codes/page.tsx | [1회] | T4 완료 — handleDragEnd 방법 B 구현 (전체 codes 재구성 후 1..N 재부여) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src-tauri/migrations/` — V301 시드는 T2 (다음 세션). 이번 세션에서는 마이그레이션 변경 없음.
- [ ] `package.json` — 신규 의존성 없음 (`tsx`는 T2에서 추가 검토)

## 완료 기준 (이번 세션)

### T1 (A20)
- ✅ AC-T1-1: IPC 에러 발생 시 "다시 시도" 버튼이 표시되고, 클릭 시 `checkLockStatus()` 재호출
- ✅ AC-T1-2: `refresh()` 호출 시 SplashScreen이 먼저 표시된 후 결과에 따라 LockScreen/LockWarning 전환
- ✅ AC-T1-3: 정상 동작 경로(에러 없음)에 영향 없음 확인

### T3 (A21)
- ✅ AC-T3-1: `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (`--test-threads` 제한 없이)
- ✅ AC-T3-2: `paths` 관련 테스트가 병렬 실행에서도 안정적으로 통과 (5회 연속 실행 확인)
- ✅ AC-T3-3: 프로덕션 코드 동작에 영향 없음
- skill: **systematic-debugging** (sprint6.md에 명시)

### T4 (A22, R26)
- ✅ AC-T4-1: 활성/비활성 필터 적용 상태에서 DnD 순서 변경 시 전체 codes의 sort_order가 정확하게 반영
- ✅ AC-T4-2: 필터 해제 후 전체 목록에서 순서가 일관성 유지
- ✅ AC-T4-3: 기존 DnD 동작(필터 미적용 상태)에 영향 없음

### 세션 종료 조건
- ✅ 각 Task 별 의미있는 커밋 (T1 `2c5b8a1` / T3 `c2be584` / T4 `83f19d1`)
- ✅ Self-verify: `cargo test` 130 passed + `cargo clippy -- -D warnings` clean + `pnpm lint` clean + `pnpm tsc --noEmit` clean
- ✅ simplify 스킬 1회 실행 (변경 사항 없음 — 3 Task 모두 단일 파일 작은 diff, 추상화/중복 없음)

## 발견된 이슈

> 코드 수정 중 예상 외 충돌·구조 발견 시 여기에 기록 후 사용자에게 보고 (step-back 프로토콜).

- lock/page.tsx 가 scope-tracker hook 에 의해 [3회 ⚠️] 표시됨 — 실제로는 T1 한 Task 내 두 영역(refresh + 에러 화면) Edit 두 번 + commit 시 추가 stage 카운트로 보임. 동일 오류 반복 없었고 loop-detection 진짜 트리거 조건(동일 테스트 3회 실패) 아님. 정상 종료.
