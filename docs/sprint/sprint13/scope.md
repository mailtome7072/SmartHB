---
Sprint: 13  |  Date: 2026-06-02  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/billing.rs | [0회] | T0-b — update_bill_impl 2쿼리 → 단일 LEFT JOIN |
| src-tauri/src/commands/notice.rs | [2회] | T0-c — save_notice_preview 경로 경계 검증(R88) |
| src/components/billing/PaymentsView.tsx | [0회] | T0-d — dirtyEntries payerName 필터(A70) |
| src/lib/menu-config.ts | [0회] | T1-a — '단원 평가'/'학습 보고서' 메뉴 제거 |
| PRD.md | [0회] | T1-c — Phase 5(§4.7/§4.8/§6.1) 취소 표기 |
| docs/arch/adr-008-optional-pin-gate.md | [신규] | T2 — 기기별 선택적 PIN 게이트 ADR |
| src-tauri/src/commands/setup.rs | [0회] | T3 — SetupStatus.skip_pin_on_launch + get/set IPC |
| src-tauri/src/startup.rs | [0회] | T4 — run_post_auth_sequence 추출 + auto_unlock_with_keychain IPC |
| src-tauri/src/lib.rs | [0회] | T3/T4 — invoke_handler 등록 |
| src/lib/tauri/index.ts | [0회] | T5-a — IPC 래퍼 3종 |
| src/app/settings/page.tsx | [0회] | T5-b/c — PIN 스킵 토글 UI |
| src/components/LockScreen.tsx | [0회] | T6 — 자동 잠금해제 분기/로딩/폴백 |
| src/app/lock/page.tsx | [0회] | T6 — 진입 흐름 분기(필요 시) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] docs/harness-engineering/ — Harness 정책 문서
- [ ] DB 마이그레이션 — Sprint 13은 마이그레이션 없음 (config.json 플래그, V111 유지)
- [ ] backend.md/frontend.md — 단원평가/학습보고서 제약은 `> [CANCELLED]` 최소 표기만(계획서 T1-c 참고, 과도 수정 금지)

## 완료 기준 (이번 세션 — sprint13.md DoD 중)
- [x] T0-a (A87) — R89로 이미 해소됨 (develop 반영, doSaveTemplate boolean 반환 + 저장 성공 시에만 이동)
- [ ] T0-b/c/d — billing JOIN / notice 경로 검증 / PaymentsView 필터
- [ ] T1 — Phase 5 메뉴 제거 + 문서 취소 표기 (ROADMAP은 계획 단계에서 반영 완료)
- [ ] T2 — ADR-008
- [ ] T3~T6 — PIN 옵션화 (백엔드 config + auto_unlock, 프론트 토글 + 진입 분기)
- [ ] T7 — 통합 검증 (cipher off/on)

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌 발견 시 여기 기록 후 사용자 보고 -->
- T0-a(A87)는 Sprint 12 세션 중 R89 커밋으로 이미 해소됨 → 본 세션에선 검증만.

## 신규 의존성
- 없음 (keyring/setup/auth 인프라 재사용)
