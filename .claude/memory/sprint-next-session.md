---
name: sprint-next-session
description: "Sprint 15 진행 중 — T0~T6 완료(sprint15 브랜치, develop 미머지). 남음: T7 양OS빌드·T8 양PC동기화·T9 통합검증(물리환경 의존). 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint15-dev-2026-06-07
---

**현재 위치(2026-06-07, 집 Mac)**: **Sprint 15 진행 중**. `sprint15` 브랜치(develop 기반, **develop 미머지**). T0~T6+T5 완료·커밋(13커밋 `e7cf0f7`~`03e4d58`). 작업트리 clean. 남은 Task = **T7(양 OS 빌드)·T8(양 PC 동기화)·T9(통합검증)** — 물리환경 의존이라 다음 세션/교습소에서 진행.
> Sprint 14 완료·develop머지·0.6.0 버전확정까지 됨. deploy-prod(v0.6.0 태그)는 여전히 **보류**. Phase 5 취소([[exam-feature-cancelled]]).

## 재개 방법
- `/sprint-dev 15` 재진입 → `docs/sprint/sprint15/scope.md`가 SSOT. T7부터.

## 완료된 Task (sprint15 브랜치)
- **T0**: monthly_summary GROUP BY 서브쿼리(R99 방어적, payments.bill_id UNIQUE라 동작 동일·마이그레이션 없음) + 대시보드 위젯 타이틀 22px.
- **T1**: 교습소 정보 화면 신설(`/settings/info`) — AcademyInfo(텍스트 9필드 + 로고/2D바코드 이미지). app_settings JSON + 이미지는 기존 notice_asset IPC 재사용(assets 파일, 파일명만 보관). 설정 허브 카드 활성화.
- **T5**(마이너 UI, 시각검증 완료): 설정 카드 순서(교습소정보 맨앞·PIN↔백업), 마법사→'DB폴더변경(예정)' 카드, 원생 상세 '원생관리 메인' 버튼, **전역 GlobalTooltip(모든 title 툴팁 20px 커스텀)**.
- **T2**: 자가진단 이력 수동 삭제(delete_diagnosis_history/clear IPC + UI 확인모달). 검증 완료.
- **T3**: 접근성 — text-gray-400→600(17건 대비 개선), **GlobalShortcuts(Ctrl+F 검색포커스·Ctrl+N 신규원생)**. 보고서 `accessibility-audit.md`. (밀집UI 44px·gray-500·F1·Ctrl+S는 Sprint16 이연)
- **T4**: 테스트 clippy `--all-targets` 부채 6건 해소(makeup.rs needless_borrow·dashboard.rs too_many_args). A89(notices 분리)는 **로직 이미 분리완료**(lib/notice-generator.ts·types/notice.ts), UI 구획화만 남아 Sprint16 이연.
- **T6**: 청구 standard_fees N+1 제거(루프 전 HashMap 1회 로드). 성능 보고서. staleTime:0은 양PC동기화 신선도 위한 의도적 설계라 유지.

## 남은 Task (T7~T9)
- **T7 양 OS 빌드**: macOS .dmg(`pnpm tauri:build`, 이 Mac 가능) + Windows CI 빌드(예: v0.7.0-beta 태그). Windows 실설치 검증은 교습소 방문 시.
- **T8 양 PC 동기화 시나리오**: Win↔Mac 전환 + app.lock + 비정상종료 강제점유. 단일 Mac에선 부분만 가능.
- **T9 통합검증**: cargo test/clippy(**--all-targets 포함**)/`cargo check --features cipher`/lint/tsc/build 전수 + 시각검증.
> 완료 후 sprint-close → sprint-review. PR 생략 직접 머지([[workflow-no-pr]]).

## Sprint 16 이연 (실측·마이그레이션 동반)
- 출결표 N+1 재설계 + AttendanceGrid 셀 memo + `makeup_attendances(student_id,year_month)` 복합인덱스(마이그레이션) — **실측 1초 초과 확인 후**.
- 공지문 50장 I/O 병렬화(notice.rs, NTFS atomic write 동반검토 [[ntfs-power-loss-pattern]]).
- A89 notices/page.tsx UI 구획화. DB폴더변경+salt.bin([[keyring-v3-features-trap]] 무관, 경로 재지정). CSV 가져오기. (내보내기 비번보호는 취소됨)

## 환경 주의
- **Node 25**: `pnpm tauri:dev` 중 `pnpm build` 금지(.next 충돌). 깨지면 프로세스 kill + `rm -rf .next` 재기동.
- **self-verify에 `--all-targets` 누락**으로 테스트 clippy 부채 누적됐었음(T4 해소). 회고에 self-verify/CI 명령 보강 제안 예정.
- cipher dev off / CI·release on ([[cipher-test-gate-trap]]).

관련: [[workflow-no-pr]], [[exam-feature-cancelled]], [[cipher-test-gate-trap]], [[sqlite-migration-fk-rebuild]], [[ntfs-power-loss-pattern]], [[keyring-v3-features-trap]]
