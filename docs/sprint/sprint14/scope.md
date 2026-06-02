---
Sprint: 14  |  Date: 2026-06-02  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/startup.rs | [1회] | T0 — A91 cipher-off 동작 명시 주석 |
| docs/arch/adr-008-optional-pin-gate.md | [1회] | T0 — A91 구현 메모 실제 동작으로 수정 |
| src/app/lock/page.tsx | [1회] | T0 — A93 SplashScreen 이중 표시 통합 |
| src-tauri/migrations/V303__create_diagnosis_history.sql | [신규] | T1 — 자가 진단 이력 테이블 |
| src-tauri/src/commands/diagnosis.rs | [신규] | T1 — 자가 진단 IPC 4종 + 검사 7종 |
| src-tauri/src/commands/dashboard.rs | [신규] | T3 — 대시보드 집계 IPC 6종 + 알림 |
| src-tauri/src/commands/export.rs | [신규] | T5 — CSV 내보내기 IPC 3종 |
| src-tauri/src/commands/backup.rs | [0회] | T7 — 복원 리허설 IPC 확장 |
| src-tauri/src/commands/mod.rs | [0회] | T1/T3/T5 — pub mod 등록 |
| src-tauri/src/lib.rs | [0회] | T1/T3/T5/T7 — invoke_handler 등록 |
| src/lib/tauri/index.ts | [0회] | T2/T4/T6/T7 — IPC 래퍼 15종+ |
| src/types/diagnosis.ts | [신규] | T2 |
| src/types/dashboard.ts | [신규] | T4 |
| src/types/export.ts | [신규] | T6 |
| src/app/page.tsx | [0회] | T4 — 대시보드로 교체(현재 리다이렉트) |
| src/components/dashboard/ | [신규] | T4 — 위젯 컴포넌트 6종 + 알림 |
| src/lib/menu-config.ts | [0회] | T4 — 대시보드 disabledHint 제거(F3) |
| src/app/settings/page.tsx | [0회] | T2/T6/T7 — 자가진단/데이터관리/백업 섹션 |
| package.json | [신규] | T4 — recharts (사전 승인 완료) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD (hook 차단)
- [ ] SETUP.sh (hook 차단)
- [ ] docs/harness-engineering/

## 신규 의존성 (사전 승인 완료)
- recharts ^2.x (대시보드 차트, 사용자 승인 2026-06-02). dynamic import 로 대시보드 라우트 한정 로드(R96).

## 신규 마이그레이션
- V303 diagnosis_history (300번대 도메인 확장 블록 연속). 추가 후 .sqlx 캐시 갱신 + CLAUDE.md 현황 갱신(A92).

## 완료 기준 (sprint14.md DoD 요약)
- [x] T0 carry-over (A91/A93) — startup cipher-off 주석 + ADR-008 정정 + /lock 단일 로딩. (R93=CLAUDE.md V302 sprint-review 기반영, R94=T8, F3=T4)
- [ ] T1/T2 자가 진단 (백엔드 7종 + 프론트 + 자동 트리거)
- [ ] T3/T4 대시보드 (집계 IPC 6종 + 위젯 6 + 알림 5)
- [ ] T5/T6 내보내기 CSV 3종
- [ ] T7 복원 리허설
- [ ] T8 통합 검증 (test/clippy/cipher/lint/tsc/build + .sqlx + CLAUDE.md V303)

## 세션 체크포인트 / 다음 진입점
- **세션 #1 (2026-06-02)**: 계획 + T0 완료, 커밋됨. 자동검증(clippy/lint/tsc) 통과.
- **다음 진입점 = T1 (자가 진단 백엔드)**: `/sprint-dev 14` 재진입 → 본 scope.md + `docs/sprint/sprint14.md`(SSOT) 확인 후 T1부터.
  - T1 착수 전: 검사 7종 쿼리용 실제 스키마 컬럼명 확인(regular_attendances/students/bills/student_schedules/makeup_attendances/payments) — `src-tauri/migrations/` V101~V111 참조.
  - V303 추가 후 `sqlx prepare` 로 `.sqlx` 캐시 갱신 필수.
  - recharts는 T4 착수 시 설치(shadcn 내장 차트 가능 여부 먼저 확인).

## 발견된 이슈
(없음)
