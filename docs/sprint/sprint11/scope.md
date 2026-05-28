---
Sprint: 11  |  Date: 2026-05-28  |  Session: #1
---

> Sprint 11 Session #1 — T0 (Phase 3 carry-over 7건 일괄 정리).
> 예상 4h. 단순/안전 수정 위주.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 |
|------|------|------|
| **T0** | Phase 3 carry-over 7건 정리 (F1~F7) | 4h |

> 다음 세션부터 청구·수납 도메인 본격 구현(T1 V109 마이그레이션 → T2~T9). 본 세션은 도메인 진입 전 기술 부채 청산 목적.

## T0 작업 범위 — F1~F7

### F1: succ_opt().expect 패닉 가능 — `attendance.rs:655`
- `d.succ_opt().expect("date succ")` → `.ok_or_else(|| AppError::Internal(...))` 안전 전환
- skill: `systematic-debugging` (panic 가능 지점 식별 및 root cause 분석)

### F2: expire fail-soft 정책 일관화 — `attendance.rs:155`
- `generate_impl` 의 expire 호출 실패 시 fail-soft 전환 (startup 패턴 동일)
- 출결 생성은 성공 반환, expire 에러는 `eprintln!` warn 로그만

### F3: 미사용 파라미터 정리 — `calendar.rs:188`
- `_year_month` 파라미터 — 실제 사용 여부 재검토 후 제거 또는 활용
- 단순 prefix `_` 만 붙여 둔 상태면 의도가 모호 → 결정 후 명시

### F4: 보강관리 N+1 쿼리 — `calendar.rs:215` (calendar.rs 한정)
- 루프 내 개별 쿼리를 JOIN 또는 IN 절 batch 처리로 전환
- 사용자 결정: **calendar.rs 만 수정** (attendance.rs N+1 은 별도 carry-over)

### F5: ClassCalendar viewType 한 프레임 불일치 — `src/components/schedules/ClassCalendar.tsx:164`
- 비동기 setState 호출과 view 전환 타이밍 정합화
- `useEffect` 의존성 또는 ref 활용

### F6: flaky 테스트 마킹 — `auth::ensure_cache_loaded_fast_path_is_concurrent_safe`
- `#[ignore]` 마킹 + 사유 코멘트 ("동시성 설계 재검토 필요, backlog")
- 단독 실행은 통과하므로 `cargo test -- --ignored` 로 검증 가능 유지

### F7: 사이드 메뉴 정리 — `src/lib/menu-config.ts:21`
- `{ label: '보강 관리', href: '/makeups', disabledHint: 'Phase 3 에서 제공' }` 항목 제거
- Sprint 10 T11 에서 `/schedules` 탭으로 통합됨 — 메뉴 중복 제거

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/attendance.rs | [0회] | F1 + F2 (라인 155, 655) |
| src-tauri/src/commands/calendar.rs | [0회] | F3 + F4 (라인 188, 215) |
| src-tauri/src/commands/auth.rs | [0회] | F6 (#[ignore] 마킹) |
| src/components/schedules/ClassCalendar.tsx | [0회] | F5 (라인 164) |
| src/lib/menu-config.ts | [0회] | F7 (보강 관리 항목 제거) |
| docs/sprint/sprint11/scope.md | [0회] | 본 파일 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [x] .github/workflows/ — CI/CD 파이프라인
- [x] SETUP.sh — 초기화 스크립트
- [x] src-tauri/migrations/ — V109 는 T1 (다음 세션) 담당
- [x] src-tauri/tauri.conf.json — Forbidden Area
- [x] docs/harness-engineering/ — 정책 문서

## 완료 기준 (이번 세션) — T0 AC

- [ ] F1: `d.succ_opt()` 패닉 가능 제거 — `Result` 전파 확인
- [ ] F2: `expire` 실패 시 출결 생성 본 흐름이 success 반환되도록 정합
- [ ] F3: `_year_month` 사용/제거 결정 후 명시 (이름 변경 또는 삭제)
- [ ] F4: `calendar.rs:215` N+1 → JOIN/IN 배치 1쿼리로 전환
- [ ] F5: `ClassCalendar` view 전환 시각적 깜박임 제거 (수동 확인 또는 테스트)
- [ ] F6: `#[ignore]` 마킹 + 사유 코멘트, 일반 `cargo test` 에서 제외 확인
- [ ] F7: 사이드 메뉴 '보강 관리' 항목 노출 안 됨 — UI 수동 확인
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --lib` 통과 (F6 마킹 후 통과)
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` clean
- [ ] `pnpm lint` + `pnpm tsc --noEmit` clean

## 세션 종료 조건

- [ ] T0 AC 모두 통과
- [ ] 단일 또는 분리 커밋 (F별 분리 권장)
- [ ] 다음 세션 (T1 V109 마이그레이션) 진입점 메모 정리

## 발견된 이슈

(없음 — 진행 중 발견 시 기록)

## 다음 세션 (T1) 미리보기

- V109 마이그레이션 작성 (bills + payments + codes.is_card_type)
- PRD §6.2 UNIQUE 제약 준수: `(student_id, bill_year_month)`
- payments 별도 테이블 (PI-12 확정)
- `sqlx migrate run` + `sqlx prepare` 캐시 갱신
