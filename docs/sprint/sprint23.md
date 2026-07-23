# Sprint Plan sprint23

## 기간
2026-07-22 ~ 2026-08-04 (2주)

## 목표
프로덕션 데이터 소실 사고(2026-07-22) 재발방지 -- 라이브 DB를 클라우드 동기화 폴더에 유지하되(ADR-012 A안), 빈 DB 날조(C1) / 무결성 오판(C2) / 커넥션 키 유실(C3) / 복원 체계 결함(H1~H4) / 백업 품질 미검증(H2) / config 경로 불일치(M1~M2)를 근절하고, 유휴 시 DB 연결 close + 활동 시 재연결로 클라우드 간섭을 최소화하여 데이터 안전성을 확립한다. 2번째 PC 로그인(B1~B2)도 함께 해소한다.

## ROADMAP 연계 기능
- RCA 재발방지 범위 A1~A6 (데이터 안전 - 필수)
- RCA 재발방지 범위 B1~B2 (2번째 PC 로그인 - 권장)
- 사고 분석 문서: `docs/incidents/2026-07-22-data-loss-rca.md` (SSOT)

## 구현 범위

### 포함 (MUST -- A 데이터 안전)
- **A1**: 라이브 DB를 클라우드 폴더에 유지 + 접근 강화 (ADR-012 A안 확정) -- after_connect 훅(T1) / create_if_missing 가드(T2) / 유휴 시 close + 활동 시 재연결(T6) / 복원-백업 강화(T3~T4)
- **A2**: `create_if_missing` 가드 -- 셋업 흔적(salt.bin) 있는데 app.db 부재 시 생성 금지
- **A3**: `after_connect` 훅으로 매 커넥션 PRAGMA key + pragma 재적용
- **A4**: 복원 체계 강화 -- WAL 사이드카 원자 처리 + 소스 검증 + 다계층 폴백
- **A5**: 백업 빈/열세 소스 거부 + 마지막 정상 백업 축출 금지
- **A6**: config startup/setup 손상 처리 통일 + set_password 기존 salt 하드 가드

### 포함 (MUST -- B 2번째 PC 로그인, Capacity 확보로 SHOULD에서 승격)
- **B1**: 신규 PC에서 PBKDF2(PIN, salt) 키 재유도 + DB 검증 + 키체인 채택
- **B2**: device.id 손상 시 락 오판 방지 + STALE 판정 활동 기준 보정

### 제외
- A114 sync_single_date 이력 패턴 리팩터 -- attendance.rs 로직 미수정으로 범위 밖 (6회째 이연 유지, 다음 attendance.rs 수정 스프린트에서 포함)
- A127 cancel_makeup N+1 배치 전환 -- makeup.rs 미수정으로 범위 밖
- A128/A129 -- 범위 밖 (Low, 보강/출결 UI 미수정)
- RCA의 "B안(로컬 라이브 + 클라우드 핸드오프)" -- ADR-012에서 탈락. ROADMAP에 다음 Phase 후보로 등록 (A 배포 후 관찰 결과에 따라 승격)

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint22-retrospective.md`

| 액션 ID | 내용 | 반영 방법 |
|---------|------|-----------|
| A115 (High) | cipher 스모크 테스트 (Sprint 18~22 이월) | T9 통합 검증에서 cipher-on 빌드 + 실 DB 가드 동작 검증 수행 |
| A127 (Medium) | cancel_makeup N+1 배치 전환 | 범위 밖 유지 (makeup.rs 미수정) |
| A128 (Low) | cancel_makeup docstring 갱신 | 범위 밖 유지 |
| A129 (Low) | 보강 시간 입력 드롭다운 전환 | 범위 밖 유지 |
| A114 (Low, 5회 이연) | sync_single_date 이력 패턴 통일 | 범위 밖 유지 -- attendance.rs 로직 변경 없음 |

## 작업 목록

### T0: A1 데이터 위치 아키텍처 결정 (ADR-012) -- ✅ 완료

> **결정**: A안 (클라우드 폴더 유지 + 접근 강화) 채택. 사용자 확정 2026-07-22.

RCA 근본 원인에 대한 3개 대안(A: 클라우드 유지+강화 / B: 로컬+핸드오프 / C: 로컬+백업만)을 Weighted Matrix + SWOT로 비교. A안이 4.30점으로 B안(3.55) / C안(3.30) 대비 정량 우세. 사고의 확정 원인(빈 DB 날조, 무결성 오판, 키 유실)을 직접 제거하면서 2-PC 공유 모델을 보존하고 데이터 이전 없이 Sprint 23에 담김.

**산출물**: `docs/arch/adr-012-db-live-location.md`

**후속**: B안(로컬 라이브 + 클라우드 핸드오프)은 ROADMAP에 다음 Phase 후보로 등록. **A 배포 후에도 클라우드 간섭에 의한 손상/복원 이벤트 관찰 시** phase-planner로 B 설계에 착수한다.

---

### T1: A3 after_connect PRAGMA key 재적용 -- 2h

> **저비용 고효과 -- T6(유휴 close)과 상호보완: T1은 커넥션 단위 PRAGMA 보장, T6은 풀 라이프사이클 관리**

**수정 대상 결함**: C3 (`db.rs:91-98` 커넥션 재연결 시 PRAGMA key + pragma 유실), H5 (2시간 체크포인트가 첫 쿼리로 커넥션 깨뜨림)

**구현**:
- `db.rs`의 `build_pool`에서 현재 1회성으로 실행하는 PRAGMA 4종(`key`, `busy_timeout`, `journal_mode=WAL`, `foreign_keys`)을 sqlx `ConnectOptions::after_connect` 훅으로 이전
- 풀에서 새 커넥션이 열릴 때마다 자동 적용되어 유휴 후 커넥션 교체 시에도 키가 유지됨
- 기존 1회성 PRAGMA 실행 코드는 제거 (중복 방지)
- PRAGMA key는 `CachedCredentials`에서 가져오되 메모리 노출 최소화 (zeroize 유지)

**단위 테스트**:
- 인메모리 DB에서 커넥션 풀 idle timeout 후 재연결 시 PRAGMA 재적용 검증
- cipher-off 환경에서 key 이외 3종 PRAGMA 적용 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

---

### T2: A2 create_if_missing 가드 + 빈 DB fail-hard -- 3h

**수정 대상 결함**: C1 (`db.rs:89` 빈 DB 날조), C2 (`integrity.rs:309-314` 빈 DB 정상 판정)

**C1 수정 -- create_if_missing 가드**:
- `db.rs:build_pool` 진입부에 가드 삽입:
  1. `setup_completed` (config.json) + salt.bin 존재 확인
  2. 둘 다 있는데 app.db 부재 → `create_if_missing(false)` 강제 + 사용자 안내 에러 반환 ("DB 파일이 없습니다. 클라우드 동기화 완료 후 재시도하세요.")
  3. 둘 다 없으면 최초 설정 → 기존 `create_if_missing(true)` 유지
- `create_dir_all` (`db.rs:81`)도 동일 가드 적용 -- 셋업 완료 상태에서 폴더 자체가 없으면 생성 금지

**C2 수정 -- 빈 DB fail-hard**:
- `integrity.rs` startup quick_check에 **행수 검사** 추가:
  1. `students` 테이블 행수 0 + `code_tables` 시드 행수만 존재 → "빈 DB (날조 의심)" 판정
  2. 판정 결과 `Err` 반환 → `startup.rs`의 auto_restore 진입 트리거
- 기존 fail-soft (`Ok` 반환) 로직을 fail-hard로 전환
- 빈 DB가 라이브로 승격되는 경로 완전 차단

**단위 테스트**:
- salt.bin 존재 + app.db 부재 시 build_pool 실패 검증
- salt.bin 부재 + app.db 부재 시 build_pool 정상 (최초 설정) 검증
- 빈 DB (시드만 존재) 판정 → Err 반환 검증
- 정상 DB (도메인 데이터 있음) → Ok 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

---

### T3: A4 자동 복원 체계 강화 -- 4h

**수정 대상 결함**: H1 (복원 시 stale WAL/SHM 미처리), H3 (auto_restore가 exit 계층만 스캔), H4 (복원 소스 행수/신선도 무검증)

**H1 수정 -- WAL 사이드카 원자 처리**:
- `integrity.rs` 복원 로직에서 복원 전 stale `-wal`/`-shm` 파일 삭제 (이미 리허설에는 방어 있음 → 실전 복원에도 동일 적용)
- 복원 소스 복사 시에도 소스의 `-wal`/`-shm` 사이드카 함께 처리 (있으면 체크포인트 후 삭제, 없으면 무시)

**H3 수정 -- 다계층 폴백**:
- `auto_restore` 검색 체인 확장: `backup/exit/` → `backup/daily/` → `backup/weekly/`
- 각 계층에서 가장 최근 파일부터 역순 시도
- 모든 계층 실패 시 "백업 없음 -- 수동 복구 필요" 에러 반환

**H4 수정 -- 복원 소스 검증**:
- 복원 대상 `.db` 파일에 대해 **열기 전 검증**:
  1. 파일 크기 > 0 (빈 파일 거부)
  2. SQLite magic bytes 확인 (첫 16바이트 "SQLite format 3\000" 또는 SQLCipher 헤더)
- **열기 후 검증**:
  1. `PRAGMA quick_check` 통과
  2. `students` 테이블 행수 > 0 (빈 DB 거부 -- T2의 판정 로직 재사용)
  3. 복원 소스 mtime이 현재 라이브 DB mtime 이전이면 **경고** 로그 (신선도 역전 감지, 차단은 아님)
- 검증 실패 시 해당 백업 스킵 → 다음 후보로 폴백

**단위 테스트**:
- exit 전멸 시 daily 폴백 검증
- exit+daily 전멸 시 weekly 폴백 검증
- 빈 백업 파일 스킵 검증
- WAL 사이드카 삭제 후 복원 성공 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

**주의**: `ntfs-power-loss-pattern` 함정 -- 복원 시 fs::write+rename 패턴 적용, 복원 직후 fsync 호출하여 전원 손실 시 NULL 손상 방지

---

### T4: A5 백업 소스 검증 + 축출 방지 -- 3h

**수정 대상 결함**: H2 (`backup.rs:271,423,215,64` 빈 DB 무검증 백업 + 시각 기반 rotation으로 정상 백업 전멸)

**빈/열세 소스 거부**:
- `backup.rs`의 `try_create_backup` 진입부에 소스 DB 검증 추가:
  1. `students` 테이블 행수 조회 (`db.rs`의 풀 또는 별도 read-only 연결)
  2. 행수 0 → 백업 스킵 + 경고 로그 ("빈 DB 백업 거부")
  3. 기존 최신 백업 파일 크기 대비 소스 크기가 50% 미만이면 경고 로그 (급격한 축소 감지)
- exit 백업, hourly 백업 모두 동일 가드 적용

**마지막 정상 백업 축출 금지**:
- rotation 삭제 시 "최소 1개 정상 백업 보존" 규칙 추가
- 각 계층(exit/hourly/daily/weekly)에서 삭제 전 잔여 파일 수 확인
- 잔여 1개면 삭제하지 않음 (rotation limit 도달해도 보존)

**단위 테스트**:
- 빈 DB 백업 시도 → 스킵 검증
- rotation 시 마지막 1개 보존 검증
- 정상 DB 백업 → 성공 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

---

### T5: A6 config 처리 통일 + set_password salt 가드 -- 3h

**수정 대상 결함**: M1 (`paths.rs` startup 무음 fallback vs `setup.rs` 손상감지+백업 불일치), M2 (`auth.rs:626` set_password 기존 salt 존재 검사 없이 새 salt 생성)

**M1 수정 -- config 경로 처리 통일**:
- `paths.rs:122-137` startup의 무음 fallback(상대경로 `./SmartHB-data`) 제거
- config.json 손상/부재 시 동작 통일:
  1. salt.bin 존재(SSOT) → config.json만 손상된 것으로 판단 → setup_completed=true 유지 + 경고 로그 + salt.bin 기준으로 data_root 추정(salt.bin 동일 디렉토리)
  2. salt.bin 부재 + config.json 부재 → 최초 설정으로 판단 → 마법사 유도
  3. salt.bin 부재 + config.json 존재 → salt.bin 유실 → 에러 + 사용자 안내 ("salt.bin이 없습니다. 클라우드 동기화 확인 후 재시도")
- `setup.rs`와 동일한 손상 감지 + 백업 로직을 `paths.rs`에도 적용

**M2 수정 -- set_password 기존 salt 하드 가드**:
- `auth.rs:626` `set_password`에 진입 가드 추가:
  1. salt.bin 이미 존재하면 새 salt 생성 거부 → 에러 반환 ("기존 salt가 있습니다. 비밀번호 변경은 change_password를 사용하세요.")
  2. 마법사에서 최초 설정 시에만 새 salt 생성 허용
- 기존 프론트엔드 가드(same-folder 검사)와 이중 방어

**단위 테스트**:
- config 부재 + salt 존재 → 경고 + data_root 추정 검증
- config 부재 + salt 부재 → 최초 설정 판정 검증
- salt 존재 시 set_password 거부 검증
- salt 부재 시 set_password 정상 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

---

### T6: A1 클라우드 폴더 내 안전 접근 강화 -- 유휴 close + 활동 재연결 -- 5h

> **ADR-012 A안 확정에 따라 데이터 이전은 없다. 클라우드에 라이브 DB를 유지하되, 유휴 시 DB 연결을 닫아 클라우드 동기화 간섭을 최소화한다.**
>
> **T1(after_connect 훅)과의 역할 분담**: T1은 **커넥션 단위** PRAGMA 보장(새 커넥션마다 키 재적용). T6은 **풀 라이프사이클** 관리(유휴 시 풀 전체 close → 클라우드가 닫힌 파일을 안전하게 동기화 → 활동 시 풀 재생성).

**구현**:

*유휴 감지 + DB 연결 close*:
- `startup.rs` 또는 `db.rs`에 유휴 감지 메커니즘 추가:
  1. 마지막 IPC 호출 시각을 `AtomicU64` (epoch secs)로 추적
  2. 일정 시간(예: 5분) 경과 시 유휴 판정
  3. 유휴 전이 시: `PRAGMA wal_checkpoint(TRUNCATE)` 실행 → sqlx Pool close
  4. WAL 체크포인트로 `-wal`/`-shm` 파일을 DB 본체에 병합 → 클라우드 동기화가 단일 파일(`app.db`)만 처리하도록 보장
- 유휴 감지는 `tokio::spawn` 백그라운드 태스크 (기존 2시간 체크포인트 태스크 패턴 재사용)
- 유휴 전이 시 프론트엔드에 상태 알림 (선택적: 상태바 "유휴 -- DB 연결 해제됨" 표시)

*활동 재개 시 재연결*:
- IPC 호출 진입 시 풀이 닫혀 있으면 자동 재생성 (`build_pool` 재호출)
- T1의 `after_connect` 훅이 새 풀의 모든 커넥션에 PRAGMA key 자동 적용 → 키 유실 불가
- 재연결 전 app.db 존재 확인 (T2의 create_if_missing 가드와 동일 로직) → 부재 시 "동기화 대기" 안내
- 재연결 실패 시 재시도 1회 → 실패 지속 시 사용자 안내 에러

*파일 변경/부재 감지 (선택적 추가 방어)*:
- 재연결 시 app.db의 mtime/크기를 close 시점 스냅샷과 비교
- 크기가 급격히 감소(50% 미만)하거나 0이면 경고 로그 + auto_restore 진입 고려
- 구현 복잡도에 따라 이 부분은 Sprint 23 범위 내에서 조정 가능

**단위 테스트**:
- 유휴 시간 경과 후 풀 close 검증
- close 후 IPC 호출 시 자동 재연결 + PRAGMA 재적용 검증
- WAL 체크포인트 후 `-wal`/`-shm` 파일 제거 확인
- app.db 부재 시 재연결 거부 + 안내 메시지 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

**주의 사항**:
- `ntfs-power-loss-pattern` 함정 -- WAL 체크포인트 실패 시 강제 close하지 않음 (Sprint 17 T1 WAL 에러 처리 패턴 재사용)
- `cipher-test-gate-trap` 함정 -- cipher-on 테스트 시 Strawberry Perl 필요 + 테스트 모듈 게이트 확인

---

### T7: B1 신규 PC 키 유도 + 키체인 채택 -- 3h

> **ADR-012 A안 확정 + T6 축소로 Capacity 확보 -- SHOULD에서 MUST로 승격.**

**수정 대상 결함**: B (`auth.rs:336-356,659-695` 2번째 PC에서 DB 키 채택 경로 없음)

**구현**:
- `auth.rs`에 `try_adopt_key` IPC 신규:
  1. 신규 PC 감지 (키체인에 DB 키 없음)
  2. salt.bin (클라우드 폴더 공유)에서 salt 로드
  3. 사용자 입력 PIN으로 `PBKDF2(PIN, salt)` 키 유도
  4. 유도된 키로 DB 열기 시도 (`PRAGMA key` + `SELECT 1 FROM students`)
  5. 성공 → 키를 로컬 키체인에 저장 + 정상 진입
  6. 실패 → "PIN이 올바르지 않거나 DB가 손상되었습니다" 에러
- **salt 재생성 절대 금지** -- 기존 salt.bin 읽기 전용
- 기존 `set_password` / `unlock_db` 호출 경로에 영향 없음 (기존 PC는 키체인에서 직접 로드)

**단위 테스트**:
- 올바른 PIN → 키 유도 + DB 열기 성공 검증
- 잘못된 PIN → 실패 검증
- 키체인 저장 성공 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

**주의**: `keyring-v3-features-trap` -- keyring 3.x에서 `features = ["apple-native", "windows-native"]` 필수. 기존 Cargo.toml에 이미 설정돼 있으나 변경 시 재확인.

---

### T8: B2 device.id 손상 + STALE 판정 보정 -- 3h

> **ADR-012 A안 확정 + T6 축소로 Capacity 확보 -- SHOULD에서 MUST로 승격.**

**수정 대상 결함**: M3 (`lock.rs:35,76,155,327` device.id 손상 시 자기 락 오판 → 최대 24h 로그인 차단), M4 (`lock.rs:38-39,155` heartbeat 제거로 STALE 임계가 시작 시각 기준)

**M3 수정 -- device.id 손상 자기 오판 방지**:
- `lock.rs` 락 확인 시 device.id 파일 읽기 실패(손상/부재) → "자기 디바이스" 로 간주 (보수적 판단: 접근 허용)
- device.id 재생성 후 락 재점유
- 기존 device.id 영속화 (Sprint 7 T3, `app_config_dir`)는 유지

**M4 수정 -- STALE 판정 활동 기준 보정**:
- Sprint 17에서 heartbeat를 제거(`T5`)했으므로 현재 STALE 판정은 앱 시작 시각 기준
- 락 파일의 타임스탬프를 **마지막 DB 쓰기 작업 시각**으로 갱신하는 가벼운 "activity marker" 도입:
  - `lock.rs`에 `touch_lock()` 함수 추가 (DB 쓰기 IPC 성공 후 호출)
  - 락 파일 mtime을 현재 시각으로 갱신 (기존 내용 유지, mtime만 변경)
  - STALE 판정: `now - lock_mtime > STALE_THRESHOLD_SECONDS`
- `STALE_THRESHOLD_SECONDS` 값 변경 시 **A113 상수 쌍** 확인 필수:
  - 백엔드: `src-tauri/src/commands/lock.rs:39`
  - 프론트엔드: `src/components/LockWarning.tsx:17`

**단위 테스트**:
- device.id 부재 시 자기 디바이스 판정 검증
- device.id 손상(빈 파일) 시 재생성 + 접근 허용 검증
- touch_lock 후 STALE 판정 갱신 검증

**cipher 빌드 검증**: `cargo check --features cipher` 통과 필수

---

### T9: 통합 검증 + cipher 스모크 테스트 -- 3h

**자동 검증 7항목** (2026-07-23 전부 통과):
- ✅ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (478 passed)
- ✅ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ✅ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ✅ `pnpm lint` 통과
- ✅ `pnpm tsc --noEmit` 통과
- ✅ `pnpm build` (static export) 통과
- ✅ `cargo test --manifest-path src-tauri/Cargo.toml --features cipher` 통과 (140 passed, A115 cipher 스모크)

**수동 검증 (배포 후)**:
- ⬜ 유휴 → 재연결 시나리오 실 DB 검증 -- cipher-on 환경에서 유휴 close 후 IPC 정상 동작
- ⬜ 2번째 PC 키 채택 시나리오 검증 -- 신규 PC에서 PIN 입력 → 키체인 채택 → 정상 사용
- ⬜ v1.4.0 → v1.5.0 업그레이드 경로 검증 (기존 사용자 무중단 -- 데이터 이전 없음)

**마이그레이션 self-check**:
- V312 최신 유지 (Sprint 23 신규 DB 마이그레이션 없음 -- Rust 로직 변경만)

## 기술 사양

### 신규 DB 마이그레이션
없음 (V312 유지). 변경은 Rust 로직에 한정 (데이터 이전 없음, config.json 구조 변경 없음).

### 신규 의존성
없음. 기존 sqlx 0.8 / keyring / rusqlite / fs2 / uuid 활용.

### 수정 대상 파일 (예상)

| 파일 | 수정 내용 | 관련 Task |
|------|----------|-----------|
| `src-tauri/src/commands/db.rs` | after_connect 훅, create_if_missing 가드, 유휴 close/재연결 풀 관리 | T1, T2, T6 |
| `src-tauri/src/commands/integrity.rs` | 빈 DB fail-hard, WAL 처리, 다계층 폴백, 소스 검증 | T2, T3 |
| `src-tauri/src/commands/backup.rs` | 소스 검증, 축출 방지 | T4 |
| `src-tauri/src/commands/paths.rs` | config 처리 통일 (무음 fallback 제거, salt.bin SSOT) | T5 |
| `src-tauri/src/commands/setup.rs` | config 손상 처리 통일 | T5 |
| `src-tauri/src/commands/auth.rs` | salt 가드, try_adopt_key | T5, T7 |
| `src-tauri/src/commands/lock.rs` | device.id 손상 처리, touch_lock, STALE 보정 | T8 |
| `src-tauri/src/commands/startup.rs` | auto_restore 확장, 유휴 감지 백그라운드 태스크 | T3, T6 |
| `src-tauri/src/lib.rs` | 신규 IPC 등록 (try_adopt_key) | T7 |
| `src/lib/tauri/index.ts` | IPC 래퍼 추가 (try_adopt_key) | T7 |
| `src/components/LockWarning.tsx` | STALE_THRESHOLD_SECONDS 동기 (A113) | T8 |

### A113 백엔드-프론트엔드 상수 쌍 확인

| 상수명 | 백엔드 | 프론트엔드 | Sprint 23 영향 |
|--------|--------|-----------|---------------|
| `STALE_THRESHOLD_SECONDS` | `lock.rs:39` | `LockWarning.tsx:17` | T8에서 값 변경 시 양쪽 동기 필수 |
| `MEMO_DEFAULT_HEIGHT` | `dashboard.rs:28` | `DashboardView.tsx:70` | 미수정 |

## Capacity 산정

> ADR-012 A안 확정(데이터 이전 불필요)에 따라 T0 완료(0h) + T6 축소(8h -> 5h) = 6h 절감. B 항목을 MUST로 승격.

| 구분 | Task | 예상 (h) |
|------|------|----------|
| MUST | T0: ADR (완료) | 0 |
| MUST | T1: after_connect | 2 |
| MUST | T2: create_if_missing 가드 | 3 |
| MUST | T3: 복원 체계 강화 | 4 |
| MUST | T4: 백업 검증 | 3 |
| MUST | T5: config 통일 | 3 |
| MUST | T6: 안전 접근 강화 (유휴 close) | 5 |
| MUST | T7: 2번째 PC 키 유도 | 3 |
| MUST | T8: device.id + STALE | 3 |
| MUST | T9: 통합 검증 | 3 |
| **총계** | | **29** |

Capacity: 40h (1인 x 10일 x 4h/일) / 실용 Capacity: 34h (시각 검증 6h 차감)

- 전체 29h: 실용 Capacity(34h) 대비 **5h 여유**. 시각 검증 + 예상치 못한 복잡도 흡수 가능
- B 항목(T7+T8 = 6h) MUST 승격: 데이터 이전 삭제로 확보된 여유(6h)가 B 항목과 정확히 상쇄
- 이전 계획(35h, SHOULD 포함) 대비 **6h 감소** -- 더 안정적인 계획

## 의존성 및 리스크

### 작업 간 의존성

```
T0 (ADR) ── ✅ 완료
T1 (after_connect) ──┐
                      ├─ T6 (유휴 close -- T1의 after_connect 훅에 의존)
T2 (create_if_missing) ──┐
                          ├─ T3 (복원 -- T2의 빈 DB 판정 재사용)
T4 (백업 검증) ── 독립
T5 (config 통일) ──┐
                    └─ T7 (키 유도 -- T5의 salt 가드와 auth.rs 공유)
T8 (device.id) ── 독립
T9 (통합 검증) ── 전체 완료 후
```

### 리스크 (상세는 `docs/risk-register/2026-07-22.md`)

| ID | 설명 | 영향도 | 대응 |
|----|------|--------|------|
| ~~R142~~ | ~~데이터 위치 이전 실패~~ | -- | ADR-012 A안(이전 없음)으로 **폐기** |
| R143 | after_connect 훅에서 PRAGMA key 적용 시 sqlx 커넥션 풀 동작과의 상호작용 -- idle timeout, max_connections 설정에 따라 예상치 못한 동작 가능 | 중간 | T1에서 인메모리 테스트 + dev 환경 실 DB 테스트. sqlx 문서의 after_connect 사용 예제 참조 |
| ~~R144~~ | ~~FK rebuild 함정~~ | -- | 데이터 이전 없으므로 **폐기** |
| R145 | `ntfs-power-loss-pattern` 함정 -- T3/T4 파일 조작 시 fs::write 후 데이터 NULL 손상 | 중간 | 모든 파일 조작에 atomic write(tmp+rename+fsync) 패턴 적용 |
| R146 | `cipher-test-gate-trap` -- T1/T6/T7 cipher-on 테스트 시 Windows Strawberry Perl 의존 + 테스트 모듈 게이트 | 중간 | T9에서 `cargo test --features cipher` 통합 수행. 게이트 패턴 확인 |
| R147 | `keyring-v3-features-trap` -- T7 키체인 저장 시 features 누락으로 silent fail 가능 | 중간 | 기존 Cargo.toml의 keyring features 설정 확인. 변경 없으면 리스크 낮음 |
| R148 | T2 빈 DB 판정 강화로 최초 설정(마법사) 흐름에서 오탐 가능 -- 시드만 있는 신규 DB를 "빈 DB"로 잘못 판정 | 중간 | 가드 조건에 "salt.bin 존재" 전제를 포함하여 최초 설정 경로 분리. 마법사 흐름 단위 테스트 |
| R149 | (신규) T6 유휴 close/재연결 시 IPC 응답 지연 -- 유휴 후 첫 IPC 호출 시 풀 재생성 + PRAGMA 적용에 1-2초 소요 가능. 사용자가 "느려짐"으로 체감 | 중간 | 재연결 시 splash/spinner 표시 검토. WAL 체크포인트는 DB 크기 1MB 미만이라 수백ms 이내 예상 |
| R150 | (신규) 클라우드-라이브-DB 잔여 리스크 -- ADR-012 A안은 안티패턴을 완전 제거하지 않음. 강화 후에도 활성 사용 중 torn-sync 잔여 가능 | 중간 | A2(생성 가드) + A4(복원 강화) + A5(백업 보호)로 데이터 손실이 아닌 안전 실패로 전환. 관찰 후 B안 승격 트리거 (ROADMAP 등록) |

## 완료 기준 (Definition of Done)

**필수**
- ✅ T0 ADR-012 작성 완료 (A안 확정, `docs/arch/adr-012-db-live-location.md`)
- ✅ C1~C3 결함 3건 수정 확인 (T1, T2)
- ✅ H1~H4 결함 4건 수정 확인 (T3, T4)
- ✅ M1~M2 결함 2건 수정 확인 (T5)
- ✅ A1 클라우드 안전 접근 강화 -- 유휴 close + WAL 체크포인트 + 활동 재연결 동작 (T6, A안 완전 close+재연결)
- ✅ B1 2번째 PC 키 유도 + 키체인 채택 동작 (T7)
- ✅ B2 device.id 손상 시 자기 디바이스 판정 + STALE 활동 기준 보정 (T8)
- ✅ cargo test 전체 통과 (478 passed — 신규 테스트 T1~T8 포함)
- ✅ cargo clippy --all-targets -- -D warnings clean
- ✅ cargo check --features cipher 통과
- ✅ cargo test --features cipher 통과 (140 passed, A115)
- ✅ pnpm lint + tsc --noEmit + build 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md v1.5.0 항목 추가
- ⬜ DEPLOY.md 업데이트

## 참고 사항

- **버전**: v1.4.0 → v1.5.0
- **사고 분석 SSOT**: `docs/incidents/2026-07-22-data-loss-rca.md`
- **ADR 산출물**: `docs/arch/adr-012-db-live-location.md` (T0, A안 확정 -- 클라우드 유지 + 접근 강화)
- **ADR-012 B안 후속**: 로컬 라이브 + 클라우드 핸드오프는 ROADMAP에 다음 Phase 후보로 등록됨. 승격 트리거: A 배포 후 클라우드 간섭에 의한 손상/복원 이벤트 관찰 시
- **데이터 이전 없음**: ADR-012 A안 확정으로 기존 사용자(v1.4.0)는 무중단 업그레이드. DB 파일 경로 변경 없음
- **A114 이연 유지**: sync_single_date 이력 패턴 리팩터는 attendance.rs 미수정으로 7회째 이연. 다음 attendance.rs 관련 스프린트에서 반드시 포함할 것
- **R01 이연 유지**: cancel_makeup_impl N+1 배치 전환 (A127) -- makeup.rs 미수정으로 이연 유지
