---
Sprint: 12  |  Date: 2026-05-30  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/billing.rs | [1회] | T0 — A73 seed_student .bind(withdraw) 전환 (A85 LEFT JOIN은 fb2a491에서 완료) |
| src-tauri/src/commands/auth.rs | [1회] | T0 — A82 verify_password validate_pin 미적용 의도 주석 1줄 |
| src/components/billing/PaymentsView.tsx | [1회] | T0 — A70 dirtyEntries payerName 필터 |
| src/components/billing/BillingSummaryView.tsx | [0회] | T0 — A86 년/월 토글 radio + radiogroup 전환 |
| src-tauri/src/commands/paths.rs | [3회 ⚠️] | T1 — assets_dir / notice_output_dir 헬퍼 |
| src-tauri/src/commands/notice.rs | [21회 ⚠️] | T2~T4 신규 — 배경서식/레이아웃/이미지 저장 IPC |
| src-tauri/src/commands/mod.rs | [1회] | T2 — pub mod notice 등록 |
| src-tauri/src/lib.rs | [4회 ⚠️] | T2~T4 — invoke_handler 등록 |
| src/lib/tauri/index.ts | [5회 ⚠️] | T5 — IPC 래퍼 8종 |
| src/types/notice.ts | [6회 ⚠️] | T5 신규 — 도메인 타입 |
| src/app/notice/page.tsx | [88회 ⚠️] | T6 신규 — /notice 라우트 |
| src/components/notice/ | [0회] | T6 신규 — 편집 컴포넌트 |
| src/lib/menu-config.ts | [1회] | T6 — '공지문' 메뉴 활성화 |
| src/lib/notice-generator.ts | [9회 ⚠️] | T7 신규 — 일괄 이미지 생성 |
| package.json | [0회] | T7 — html-to-image + react-rnd (계획 명시 — 사전 승인됨) |
| src-tauri/capabilities/default.json | [0회] | T8 — 필요 시 최소 권한 (std::fs 직접이면 변경 없음) |
| src/components/LockScreen.tsx | [1회] | scope 외 추가 — Sprint 1 T4 carry-over TODO 해결: "비밀번호를 잊으셨나요?" dead button → RecoveryCodeInput 연결 (회사 PC PIN 재설정 차단 해소, 사용자 요청) |
| src-tauri/src/commands/auth.rs | [2회] | scope 외 추가 — change_pin IPC (현 PIN 검증 후 새 PIN 으로 재설정, set_password 패턴 재사용) |
| src-tauri/src/lib.rs | [5회 ⚠️] | scope 외 추가 — change_pin invoke_handler 등록 |
| src/lib/tauri/index.ts | [6회 ⚠️] | scope 외 추가 — changePin 래퍼 |
| src/app/settings/page.tsx | [1회] | scope 외 추가 — 'PIN 번호 변경' 카드 추가 |
| src/app/settings/pin/page.tsx | [1회] | scope 외 추가 — PIN 변경 UI (신규 라우트, 회사 PC PIN 변경 차단 해소 사용자 요청) |
| src-tauri/src/commands/recovery.rs | [삭제] | scope 외 — 복구 코드 시스템 제거 (cipher OFF 환경에서 불필요, 사용자 요청) |
| src/components/RecoveryCodeInput.tsx | [삭제] | scope 외 — 복구 코드 입력 UI 제거 |
| src/components/RecoveryCodeDisplay.tsx | [삭제] | scope 외 — 복구 코드 표시 UI 제거 (orphan 컴포넌트) |
| src/lib/recovery-code.ts | [삭제] | scope 외 — 복구 코드 정규화 helper 제거 |
| src-tauri/src/commands/mod.rs | [2회] | scope 외 — `pub mod recovery` 제거 |
| src-tauri/src/commands/audit.rs | [1회] | scope 외 — RecoveryCodeIssued variant + 테스트 + 문서 주석 제거 |
| src-tauri/src/commands/setup.rs | [1회] | scope 외 — recovery 모듈 언급 주석 정리 |
| src/types/index.ts | [1회] | scope 외 — AuditEventType 의 'recovery-code-issued' 제거 |
| src-tauri/Cargo.toml | [1회] | scope 외 — argon2 의존성 제거 (복구코드 전용이었음) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] docs/harness-engineering/ — Harness 정책 문서
- [ ] DB 마이그레이션 — Sprint 12는 마이그레이션 없음 (파일시스템 + app_settings 활용)

## 완료 기준 (이번 세션 — sprint12.md DoD 중)
- [ ] T0 carry-over 6건 정리 (A70/A71/A73/A82/A85(완료)/A86)
- [ ] T1 경로 헬퍼 + 단위 테스트
- [ ] T2 배경서식 IPC 3종 + 단위 테스트
- [ ] T3 레이아웃 저장/로드 IPC + 단위 테스트
- [ ] T4 이미지 저장 IPC 3종 + 단위 테스트
- [ ] T5 TS 래퍼 + 타입
- [ ] (여력 시) T6 편집 화면 / T7 일괄 생성 엔진 — 분량 크면 다음 세션

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌 발견 시 여기 기록 후 사용자 보고 -->
(없음)

## 신규 의존성 (계획 명시 — 사전 승인)
- html-to-image ^1.11.13 (T7)
- react-rnd ^10.x (T6, PI-14 확정)
