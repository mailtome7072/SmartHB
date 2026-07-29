---
name: sprint-next-session
description: "Sprint 24 구현·리뷰·QA 완료(2026-07-29) — 다월 교습기간 year_month 오태깅 계열 일괄수정 + V313 전수보정 + 수동검증 중 추가 5건(청구 이력인식·reconcile·팝업 등). develop 13커밋 푸시완료, deploy-prod(v1.5.1) 대기. 다월 함정=[[multi-month-period-pitfall]]. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint23-deploy-2026-07-23
  modified: 2026-07-29T09:08:02.312Z
---

## ✅ 2026-07-23 — Sprint 23 완료 + v1.5.0 프로덕션 배포

### 개요
2026-07-22 프로덕션 데이터 소실 사고 재발방지 스프린트. ADR-012 **A안**(클라우드 라이브 DB 유지 + 접근 강화). DB 마이그레이션 없음(V312 유지), 신규 의존성 없음. v1.4.0 → **v1.5.0**.
- RCA SSOT: `docs/incidents/2026-07-22-data-loss-rca.md`. 사고 배경/복구는 [[data-loss-recovery-method]].

### 구현(T1~T9) — 결함별
- **T1**(C3,H5): after_connect 훅으로 매 커넥션 PRAGMA key/startup 재적용.
- **T2**(C1,C2): create_if_missing 가드 + 빈 DB fail-hard. `paths::setup_completed` 캐시(SSOT). 마법사 순서(DB생성<complete_setup)로 최초실행 오탐 회피.
- **T3**(H1,H3,H4): 복원 다계층 폴백(exit→daily→weekly) + WAL 사이드카 제거 + fsync + 소스 검증(크기/quick_check/빈DB거부) + 신선도 경고.
- **T4**(H2): 빈 소스 백업 거부(perform_backup_with_cipher) + rotation 마지막 1개 보존.
- **T5**(M1,M2): config 처리 통일(paths↔setup read_status_from_path 공유, salt.bin SSOT) + set_password salt 하드 가드.
- **T6**(A1) **강력한 조치 A안**: 전역 POOL을 RwLock<Option>로, `pool()` async+owned+자동 재연결. 유휴 5분(IDLE_CLOSE_THRESHOLD_SECS=300) close(WAL TRUNCATE+close)+활동 재연결. pool_if_open(백그라운드/exit), RECONNECT_LOCK, POOL_DB_PATH, POOL_SHUTDOWN(change_data_folder), LAST_ACTIVITY. **호출부 89+곳 참조 섀도잉(`let pool=db::pool().await?; let pool=&pool;`)으로 downstream 무변경**. ⚠️구현 중 sqlx `num_idle()`가 쿼리 직후에도 0 반환(size=1) 발견→num_idle 가드 제거, graceful close+5분 임계로 대체.
- **T7**(B1): try_adopt_key IPC(2번째 PC PBKDF2 키 재유도+DB검증+키체인 채택, salt 재생성 금지) + LockScreen 폴백.
- **T8**(M3,M4): device.id 유실(첫실행 부재 제외) 자기오판 방지 + 활동기준 STALE(touch_lock mtime + seconds_since_lock_activity). STALE값 86400 유지→A113 프론트 동기화 불요.
- **T9**: 자동검증 7항목 통과(cargo test 478 / clippy / cargo check+test --features cipher 140(A115) / pnpm lint·tsc·build).

### QA(로컬 cipher-off) + 발견 버그 수정
- 로컬 스모크 통과: 수납 CRUD / 유휴 5분+→저장(②번 오류 재발 없음) / create_if_missing 가드(빈 DB 미생성).
- **UX 버그 수정(커밋 381e1a1)**: DB 부재 안내가 "비밀번호 틀림"으로 오표시되던 문제. 원인=①C1가드가 AppError::Config 사용→user_message가 generic으로 치환, ②LockScreen adopt 폴백이 문자열 매칭으로 실제 오류 덮어씀. 수정=Config→**UserFacing**(문구 그대로) + adopt 실패 시 원래 오류 표시. sprint-review M-02 실질 해소.
- 리뷰: Critical 0/High 0/Medium 2(수용)/Low 3. `docs/code-reviews/sprint23.md`, 회고 `docs/sprint-retrospectives/sprint23-retrospective.md`.

### 배포
- develop→master 머지(15c0b91), v1.5.0 태그, GitHub Actions 성공(Run 29990099827), 릴리스 발행: https://github.com/mailtome7072/SmartHB/releases/tag/v1.5.0
- 아티팩트: `SmartHB_1.5.0_x64-setup.exe` / `SmartHB_1.5.0_aarch64.dmg`. 버전 4파일(+Cargo.lock) 동기화 확인([[deploy-version-three-files]]).
- develop=master 동기화, 역머지 완료.

## ⬜ 다음 세션 / 남은 작업
1. **[최우선] 실환경 배포 후 검증** (원장 PC 교습소 + 자택 Mac, cipher-on 실 DB) — 로컬 cipher-off로는 검증 불가한 항목:
   - **기존 데이터 무손실**: v1.4.0→v1.5.0 업그레이드 후 원생/수납 정상 로드 (최우선)
   - 유휴 5분+ 후 저장 오류 재발 없음(R150 관찰), after_connect 재시작 후 DB 접근 정상
   - 자택 Mac: PIN→try_adopt_key→DB 열기 성공
   - create_if_missing 가드(salt 있고 app.db 없을 때 안내 메시지 정확히 표시 — 위 UX 수정 확인)
   - 이월: 교습일정 인쇄 미리보기(Sprint 20 A122)
2. **Sprint 24 구현·리뷰·QA 완료 — `deploy-prod` 대기** (2026-07-29). develop 13커밋 푸시 완료. **v1.5.1 PATCH 후보**. SSOT: `docs/sprint/sprint24.md`, `docs/code-reviews/sprint24.md`, CHANGELOG [Unreleased].
   - **주제**: 다월 교습기간(예: 8월=7/30~9/2) "날짜의 달력월 = 소속 교습기간 year_month" 오가정 계열 버그 일괄 수정 + V313 전수 보정. Sprint 21 R136 잔존분. CS 발단=A원생 목요일 변경 적용일 7/30 후 7/30이 8월 그리드에서 사라짐.
   - **계획분**: A1 apply_schedule_change / A2 create_makeup 태깅(오염) / B1~B8 표시·동작 / E1 V313 마이그레이션 / D1 diagnosis 검사8(year_month↔교습기간 불변식) + D2 회귀테스트 / A114(7회 이연) 최종 해소.
   - **수동검증 중 발견·수정 5건(중요)**: ①교습기간 (재)설정 시 기존 출결 재태깅 갭 → `periods::reconcile_attendance_year_month`(생성/수정/확정 훅 + **startup 자가치유**) ②시수 변경 시 청구 재확인 팝업 신설(`list_affected_bill_months` IPC + 모달) ③**청구 주당시간을 현행 스케줄→교습기간 종료일 유효 스케줄(이력 인식)로 수정** — billing "SAFE 전제가 틀림": "교습기간이 스케줄 변경 적용일보다 앞선" 케이스 누락. 7월 3h 정상화 + "추가 청구 데이터 생성" 유령 버튼 해소 ④이동 팝업 달력 교습기간 범위 확장(B1 프론트) ⑤리뷰지적 M1/M2/L1/L2(공유 헬퍼 `commands/periods.rs` 신설 등). 상세 함정: [[multi-month-period-pitfall]].
   - 검증: cargo test 497 / clippy / cipher / lint / tsc / build 전수 통과. sprint-review 초기+재리뷰 완료(Critical/High 0, Low 2 이연=A132 로깅/팝업 advisory). **배포 게이트 통과.**
   - 남은 것: **deploy-prod(v1.5.1 권장)** + 배포 후 실PC(원장/자택) 확인. 새 버전 첫 실행 시 V313 + startup 재동기화가 기존 오태깅 자동 교정(양 PC 모두 새 버전 실행 필요).
   - (선택) Notion API 명세에 신규 IPC `list_affected_bill_months` 반영.
   - ADR-012 **B안(로컬 라이브+클라우드 핸드오프)**은 A 배포 후 클라우드 간섭 손상 관찰 시 phase-planner로 착수(ROADMAP 후보 등록됨).

관련: [[workflow-no-pr]], [[deploy-version-three-files]], [[data-loss-recovery-method]], [[dev-pc-db-is-test-data]], [[cipher-test-gate-trap]]
