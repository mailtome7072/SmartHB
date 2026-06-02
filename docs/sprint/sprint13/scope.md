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
| src/lib/menu-config.ts | [1회] | T1-a — '단원 평가'/'학습 보고서' 메뉴 제거 |
| PRD.md | [3회 ⚠️] | T1-c — Phase 5(§4.7/§4.8/§6.1) 취소 표기 |
| docs/arch/adr-008-optional-pin-gate.md | [신규] | T2 — 기기별 선택적 PIN 게이트 ADR |
| src-tauri/src/commands/setup.rs | [4회 ⚠️] | T3 — SetupStatus.skip_pin_on_launch + get/set IPC |
| src-tauri/src/startup.rs | [2회] | T4 — run_post_auth_sequence 추출 + auto_unlock_with_keychain IPC |
| src-tauri/src/lib.rs | [2회] | T3/T4 — invoke_handler 등록 |
| src/lib/tauri/index.ts | [1회] | T5-a — IPC 래퍼 3종 |
| src/app/settings/page.tsx | [5회 ⚠️] | T5-b/c — PIN 스킵 토글 UI |
| src/components/LockScreen.tsx | [0회] | T6 — 자동 잠금해제 분기/로딩/폴백 |
| src/app/lock/page.tsx | [0회] | T6 — 진입 흐름 분기(필요 시) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] docs/harness-engineering/ — Harness 정책 문서
- [ ] DB 마이그레이션 — Sprint 13은 마이그레이션 없음 (config.json 플래그, V111 유지)
- [ ] backend.md/frontend.md — 단원평가/학습보고서 제약은 `> [CANCELLED]` 최소 표기만(계획서 T1-c 참고, 과도 수정 금지)

## 완료 기준 (이번 세션 — sprint13.md DoD 중)
- [x] T0-a (A87) — R89로 이미 해소됨 (develop 반영)
- [x] T0-b (A85) — billing.rs 이미 단일 LEFT JOIN 통합(이전 스프린트, 검증만)
- [x] T0-c (R88) — save_notice_preview 경로 검증 구현 + 테스트
- [x] T0-d (A70) — PaymentsView payerName 필터 이미 적용(검증만)
- [x] T1 — Phase 5 메뉴 제거 + PRD 취소 표기 (ROADMAP은 계획 단계 반영)
- [x] T2 — ADR-008 작성
- [x] T3 — config skip_pin_on_launch + get/set IPC + 테스트
- [x] T4 — run_startup 추출 + auto_unlock_with_keychain IPC
- [x] T5 — IPC 래퍼 3종 + 설정 토글
- [x] T6 — /lock 진입 자동 잠금해제 분기 + 폴백
- [x] T7-a/c — 자동 검증 전수 통과 + 마이그레이션 신규 없음
- [ ] T7-b — 수동 시나리오 4건 (사용자 검증 필요)

## 자동 검증 결과 (T7-a)
- cargo test (cipher off) 315 passed ✅ / clippy clean ✅ / cargo check --features cipher 통과 ✅ (R91 회귀 없음)
- tsc / lint / build(static) 통과 ✅ / 마이그레이션 신규 없음(실제 최신 V302)

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌 발견 시 여기 기록 후 사용자 보고 -->
- **T0 carry-over 3건(A87/A85/A70) 이미 해소됨**: 계획서가 stale — 이전 스프린트에서 처리됨. 실제 신규 작업은 T0-c(R88)뿐. (차단 아님)
- **T0-c(R88) 편차**: 계획의 "output_root jail"은 '파일 저장 다이얼로그' UX와 충돌 → 절대경로+.png+traversal 차단 + data_root 밖 폴더 자동생성 금지로 대체(다이얼로그 UX 보존).
- **T5-c 단순화**: 설정 화면 도달 = 이미 이 PC 키체인 키로 잠금해제됨 → 토글 항상 활성. "키 없는 새 PC" 엣지는 T6 폴백이 안전 처리. has_keychain_key IPC 미추가.

## 신규 의존성
- 없음 (keyring/setup/auth 인프라 재사용)
