---
name: sprint-next-session
description: "✅ Sprint 23 완료 + v1.5.0 프로덕션 배포 완료(2026-07-23). 데이터 소실 사고(2026-07-22) 재발방지(T1~T9, ADR-012 A안). 남은 것=원장 PC/자택 Mac 실환경 검증(무손실 업그레이드 최우선). Sprint 24 대기. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint23-deploy-2026-07-23
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
2. **Sprint 24 대기** — 필수 남은 작업 없음. ADR-012 **B안(로컬 라이브+클라우드 핸드오프)**은 A 배포 후 클라우드 간섭에 의한 손상/복원 이벤트 관찰 시 phase-planner로 착수(ROADMAP 후보 등록됨).
- 이연: A114(sync_single_date 이력 패턴), A127(cancel_makeup N+1) — 7회째 이연.

관련: [[workflow-no-pr]], [[deploy-version-three-files]], [[data-loss-recovery-method]], [[dev-pc-db-is-test-data]], [[cipher-test-gate-trap]]
