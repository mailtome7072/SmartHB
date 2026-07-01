# Sprint Plan sprint17

## 기간
2026-06-30 ~ 2026-06-30 (당일 완료)

## 머지 정보
develop 머지: `sprint17 → develop` 직접 머지 (단일 개발자 정책, 2026-06-30)

## 목표
v1.0.0 실사용 중 발견된 DB 안전성 버그의 잔여 수정(Hotfix 미포함 3건)을 완료하고, 1인 운영 환경에 맞게 클라우드 동기화 관련 정책을 간소화(heartbeat 제거, hourly 간격 확대, SyncStatus 삭제)하여 MYBOX 부하와 lock 충돌을 줄인다.

## ROADMAP 연계 기능
- v1.0.0 실사용 피드백 — DB 장시간 사용 후 저장 실패 + 재시작 데이터 손실 버그 후속 조치
- Hotfix(`hotfix/db-lock-and-backup-fix`)에서 처리된 6건 이후 남은 안전성 수정 3건
- 클라우드 동기화 정책 간소화 3건 (1인 운영 최적화)

## Hotfix에서 이미 처리된 항목 (Sprint 17 범위 제외)
- `db.rs`: PRAGMA busy_timeout=30000, synchronous=NORMAL, journal_size_limit, pool 타임아웃
- `backup.rs`: rusqlite busy_timeout, 리허설 풀 타임아웃
- `startup.rs`: hourly 루프 WAL checkpoint(PASSIVE)

---

## 작업 목록

### 그룹 A — 남은 안전성 수정 (3건)

- ✅ **T1 (A1): DB 폴더 변경 WAL 체크포인트 에러 처리** — 예상 2h
  - 파일: `src-tauri/src/commands/setup.rs`
  - 현재: `let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(pool).await;` — 에러 묵살
  - 수정: 체크포인트 실패 시 복사를 중단하고 사용자에게 친화적 오류 메시지 반환 (`"데이터 정리 중 오류가 발생했습니다. 다시 시도해 주세요."`)
  - 검증: 단위 테스트 — 체크포인트 실패 시 Result::Err 반환 확인. 기존 `change_db_folder` 테스트 회귀 없음

- ✅ **T2 (A2): 백업 파일 임시 파일 후 이동 방식 (atomic write)** — 예상 3h
  - 파일: `src-tauri/src/commands/backup.rs`
  - 현재: 클라우드 폴더에 직접 쓰기 — 미완성 파일이 MYBOX에 즉시 동기화될 수 있음
  - 수정: `{filename}.tmp`에 먼저 쓰고, 무결성 검증(`PRAGMA quick_check`) 후 최종 이름으로 `fs::rename` (NTFS power-loss 패턴 적용: fsync 포함)
  - 검증: 단위 테스트 — tmp 파일 생성 후 rename 정상 동작. 기존 백업 순환 삭제 로직 회귀 없음
  - 참고: `.claude/memory/ntfs-power-loss-pattern.md` — rename 후 손상 감지 fallback 필수

- ✅ **T3 (A3): 자동 복원 후 재검증 (integrity double-check)** — 예상 3h
  - 파일: `src-tauri/src/startup.rs`, `src-tauri/src/commands/integrity.rs`
  - 현재: `auto_restore` 성공 후 바로 pool 초기화 — 복원 파일이 손상됐을 경우 앱 진입 불가
  - 수정: 복원 직후 `quick_check` 재실행. 재검증 실패 시 다음 백업으로 rollback 복원 시도 (최대 3회). 모두 실패 시 사용자에게 "복원할 수 있는 백업이 없습니다" 메시지 표시
  - 검증: 단위 테스트 — 복원 후 quick_check 실패 시 rollback 호출 확인. 정상 복원 시 pool 초기화 확인

### 그룹 B — 정책 간소화 (3건)

- ✅ **T4 (B1): Hourly 백업 간격 1시간 -> 2시간** — 예상 0.5h
  - 파일: `src-tauri/src/startup.rs`
  - 수정: `HOURLY_BACKUP_INTERVAL_SECS` 상수를 3600 -> 7200으로 변경
  - 효과: MYBOX 업로드 부하 감소, lock 충돌 빈도 절반
  - 검증: 상수 변경만으로 로직 영향 없음. 기존 hourly 백업 테스트가 상수 참조 시 테스트도 함께 조정

- ✅ **T5 (B2): Heartbeat 제거** — 예상 3h
  - 파일: `src-tauri/src/startup.rs`, `src-tauri/src/commands/lock.rs`
  - 현재: 60초마다 `app.lock` mtime 갱신 -> MYBOX가 60초마다 동기화 트리거
  - 수정:
    - `startup.rs`: heartbeat 루프(`spawn_heartbeat` 또는 유사 함수) 제거
    - `lock.rs`: app.lock은 앱 시작 시 생성, 종료 시 삭제로 단순화. stale 판정 로직(5분 mtime 기반)은 유지 또는 앱 시작 시 config.json 기반 "마지막 사용 PC" 판정으로 대체
  - 전제: 집<->교습소 동시 사용 없음 (사용자 확인 완료)
  - 검증: 단위 테스트 — lock 생성/삭제 정상 동작. heartbeat 미갱신 상태에서 stale 판정 정상 동작. 기존 lock 테스트 회귀 없음

- ✅ **T6 (B3): SyncStatus 삭제** — 예상 3h
  - 삭제 대상 파일:
    - `src-tauri/src/commands/sync.rs` — 백엔드 IPC 전체 삭제 (모듈 삭제 또는 빈 모듈로)
    - `src-tauri/src/lib.rs` — `sync.rs` 관련 invoke_handler 등록 제거
    - `src/lib/tauri/index.ts` — `checkSyncStatus` 래퍼 삭제
    - `src/components/layout/app-shell.tsx` — 60초 polling 로직 제거
    - `src/components/layout/top-bar.tsx` — 동기화 표시 UI 제거
  - 전제: PC 전환 시 MYBOX 동기화 완료를 사용자가 육안으로 확인
  - 주의: 상단바에서 "동기화: 준비됨" 표시가 사라짐 — 사용자가 인지하고 있음
  - 검증: `cargo test` — sync 관련 테스트 삭제 후 전체 통과. `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` — 프론트엔드 참조 제거 후 빌드 성공

### 통합 검증

- ✅ **T7: 통합 검증** — 예상 1.5h
  - `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
  - `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
  - `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
  - 수정 파일 목록과 scope.md 일치 확인

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint15-retrospective.md` (Sprint 16 회고 미존재)

| 항목 ID | 액션 아이템 | 이번 스프린트 반영 |
|---------|------------|-------------------|
| A98 | self-verify에 `--all-targets` 추가 | Sprint 16에서 이미 적용 완료. T7 통합 검증에서 `--all-targets` 기준으로 실행 |
| A99 | Ctrl+N 입력 필드 포커스 방어 로직 | Sprint 16 T1에서 이미 처리됨 (ROADMAP 확인). 회귀 없음 확인 |
| A100 | 미저장 이탈 경고 공통 훅 | Sprint 16 T1에서 이미 처리됨 (ROADMAP 확인). 회귀 없음 확인 |
| A96 | 복원 리허설 dev 환경 개선 | Low 우선순위. Sprint 17 범위 외. 기회 발생 시 이후 반영 |

---

## Capacity 추정

| 항목 | 추정 시간 |
|------|----------|
| T1 (A1 WAL 체크포인트 에러 처리) | 2h |
| T2 (A2 백업 atomic write) | 3h |
| T3 (A3 자동 복원 후 재검증) | 3h |
| T4 (B1 hourly 간격 변경) | 0.5h |
| T5 (B2 heartbeat 제거) | 3h |
| T6 (B3 SyncStatus 삭제) | 3h |
| T7 (통합 검증) | 1.5h |
| **합계** | **16h** |

가용 시간: 1인 x 10일 x 4h/일 = 40h (실작업 기준)
안전 마진: 24h 잔여 — 검증 중 발견 이슈 대응 버퍼 충분

---

## 수정 범위 요약

| 파일 | 관련 Task | 변경 유형 |
|------|-----------|-----------|
| `src-tauri/src/commands/setup.rs` | T1 | 수정 (에러 처리 강화) |
| `src-tauri/src/commands/backup.rs` | T2 | 수정 (tmp + rename) |
| `src-tauri/src/startup.rs` | T3, T4, T5 | 수정 (재검증 + 상수 + heartbeat 제거) |
| `src-tauri/src/commands/integrity.rs` | T3 | 수정 (재검증 로직) |
| `src-tauri/src/commands/lock.rs` | T5 | 수정 (heartbeat 제거, 생성/삭제 단순화) |
| `src-tauri/src/commands/sync.rs` | T6 | 삭제 |
| `src-tauri/src/lib.rs` | T6 | 수정 (sync 등록 제거) |
| `src/lib/tauri/index.ts` | T6 | 수정 (checkSyncStatus 삭제) |
| `src/components/layout/app-shell.tsx` | T6 | 수정 (polling 제거) |
| `src/components/layout/top-bar.tsx` | T6 | 수정 (동기화 표시 제거) |

- DB 마이그레이션: 없음
- 새 의존성: 없음

---

## 완료 기준 (Definition of Done)

**필수**
- ✅ WAL 체크포인트 실패 시 DB 폴더 변경 중단 + 사용자 오류 메시지 반환 (T1)
- ✅ 백업 파일이 tmp 파일로 생성 후 검증 뒤 rename으로 확정 (T2)
- ✅ 자동 복원 후 quick_check 재검증, 실패 시 rollback 복원 시도 (T3)
- ✅ hourly 백업 간격 2시간으로 변경 (T4)
- ✅ heartbeat 루프 제거, app.lock은 시작/종료 시에만 생성/삭제 (T5)
- ✅ sync.rs IPC + 프론트엔드 polling + 동기화 표시 UI 완전 제거 (T6)
- ✅ `cargo test` 전체 통과 411건 (T7)
- ✅ `cargo clippy --all-targets -- -D warnings` clean (T7)
- ✅ `cargo check --features cipher` 통과 (T7)
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과 (T7)

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R117 | T5 heartbeat 제거 후 stale lock 판정 기준 변경 — mtime 기반 5분 판정이 heartbeat 없이 동작하는지 확인 필요 | 중간 | lock 생성 시 mtime 기록, 종료 시 삭제. 비정상 종료 시 mtime이 앱 시작 시점으로 고정되므로 5분 경과 판정은 유효. 단위 테스트로 시나리오 커버 |
| R118 | T6 SyncStatus 삭제 후 사용자 혼란 — 동기화 상태 표시 사라짐에 대한 인지 부족 | 낮음 | 사용자가 이미 삭제에 동의. MYBOX 트레이 아이콘으로 동기화 상태 확인 가능 |
| R119 | T2 atomic write 시 tmp 파일 잔류 — 앱 비정상 종료 시 .tmp 파일이 남을 수 있음 | 낮음 | 앱 시작 시 backup 디렉토리의 .tmp 파일 정리 로직 추가 (선택적) |

---

## 참고 사항

- 이번 스프린트는 Hotfix(`hotfix/db-lock-and-backup-fix`) 후속으로, 긴급 6건 처리 후 남은 안전성 수정과 정책 간소화를 다룬다.
- 모든 변경은 기존 코드의 수정/삭제이며, 새로운 기능 추가는 없다.
- heartbeat 제거(T5)와 SyncStatus 삭제(T6)는 "집<->교습소 동시 사용 없음" 전제에 기반한다. 향후 동시 사용이 필요해지면 재도입을 검토해야 한다.
- Sprint 17 완료 후 v1.0.1 패치 배포를 고려할 수 있다.
