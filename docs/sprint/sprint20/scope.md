---
Sprint: 20  |  Date: 2026-07-19  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/billing.rs | [14회 ⚠️] | T1: 청구 대상 규칙 교습기간 기준 전환 + get_billing_summary 동기화 / T3: delete_bill |
| src-tauri/src/commands/audit.rs | [2회] | T3: BillDeleted audit variant |
| src-tauri/src/lib.rs | [1회] | T3: delete_bill 커맨드 등록 |
| src/lib/tauri/index.ts | [1회] | T4: deleteBill 래퍼 |
| src/types/billing.ts | [0회] | T4: 삭제 관련 타입(필요 시) |
| src/app/billing/page.tsx | [0회] | T4: 삭제 버튼 배치 |
| src/components/billing/*.tsx | [0회] | T4: 삭제 버튼 + 확인 다이얼로그 |
| src/lib/academic-print-html.ts | [5회 ⚠️] | T6: 인쇄 3개월+ 걸침 멀티페이지 |
| src/app/academic/print/page.tsx | [0회] | T6: 필요 시 |
| src-tauri/src/commands/attendance.rs | [0회] | T7: count_ungenerated / sync_single_date / generate_impl 태깅 |
| src/app/attendance/page.tsx | [0회] | T7: 생성 버튼 표시 조건 |
| src/components/attendance/AttendanceGrid.tsx | [0회] | T7 버그B: 컬럼 범위·매핑(분리 가능) |

## 신규 생성 파일 (문서/ADR)
| 파일 | 비고 |
|------|------|
| docs/arch/adr-NNN-bill-deletion-guard.md | T2: 삭제 가드 ADR(B안 확정 근거) |
| docs/sprint/sprint20/data-correction-procedure.md | T5/T5-b: 실 DB 보정 절차 + 퇴교취소 재생성 워크플로우 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] src-tauri/migrations/ — DB 마이그레이션 없음 (스키마 변경 불요)
- [ ] list_payment_view / 수납 IPC — CASCADE로 정합 자동 유지, 수정 불요

## 완료 기준 (이번 세션)
- [ ] T1: 교습기간 종료일 이후 입교 원생 청구 제외 + 미등록 월 차단 + get_billing_summary 동기화 (테스트 6건)
- [ ] T2: 삭제 가드 ADR(B안) 문서화
- [ ] T3: delete_bill IPC (B안 가드, payments CASCADE, BillDeleted audit) + 테스트
- [ ] T4: 청구 삭제 UI (미수납만 활성, 확인 다이얼로그)
- [ ] T5/T5-b: 실 DB 보정 절차 + 퇴교취소 재생성 워크플로우 문서
- [ ] T6: 인쇄 3개월+ 걸침 정상 출력 (1~2개월 회귀 없음)
- [ ] T7: 출결 버그A(부분생성→버튼 숨김) 수정. 버그B(그리드 다월 표시)는 범위 보고 분리 판단
- [ ] T8: 통합 검증 (cargo test / clippy --all-targets / cipher check / lint / tsc / build)

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌/설계 오류 발견 시 여기에 기록 후 사용자 보고 -->
(없음)
