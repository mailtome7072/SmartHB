# ADR-009: DB 폴더 변경(클라우드 동기화 경로 재지정) + salt.bin 이전 전략

- 상태: 채택 (Accepted)
- 날짜: 2026-06-10
- 관련: Sprint 16 T3, PI-16(2026-06-08 확정), PRD §5.3, ADR-001(SQLCipher)·ADR-002(app.lock)·ADR-003(백업)
- 참조 메모리: `keyring-v3-features-trap`, `ntfs-power-loss-pattern`, `sqlite-migration-fk-rebuild`

## Context (배경)

운영 중 데이터가 저장되는 클라우드 동기화 폴더(`{cloud}/smarthb/`)를 사용자가 재지정할 수 있어야 한다(PI-16). 이 폴더에는 다음이 모두 들어 있다:

- `app.db` (+ `-wal`, `-shm`) — SQLCipher AES-256 암호화 DB
- `salt.bin` — PBKDF2 salt (평문 32바이트, 비밀 아님 — 양 PC 자동 동기화 목적)
- `assets/` (공지문 배경·로고·바코드·달력 원본), `output/` (공지문 PNG)
- `backup/{exit,hourly,daily,weekly}/` — 4계층 백업
- `app.lock` — 동시성 제어 락 (ADR-002)

경로 설정은 **OS app_config_dir 의 `config.json`** 의 `cloud_folder_path`에 저장되며, **PC-로컬**(클라우드 동기화 대상 아님)이다. 시작 시 `paths::init_data_root_from_config`가 이를 읽어 `data_root = {cloud}/smarthb` 로 해석한다. 암호화 키는 OS Keychain/Credential Manager(`keyring` v3)에 PC별로 보관된다.

**위험**: 사용자의 유일한 운영 데이터(암호화 DB)와 salt를 물리적으로 이동하는 작업 — 중간 실패 시 데이터 접근 불가/손실 위험이 가장 크다.

## Decision (결정)

**copy-then-switch + 앱 재시작** 방식을 채택한다. 원본을 보존한 채 새 폴더로 복사·검증한 뒤, **마지막에** config.json 경로를 갱신하고 앱을 재시작한다.

### 절차 (순서 = 안전성의 핵심)

1. **대상 검증**
   - 새 경로 비어있지 않음 · 기존 경로와 동일/포함 관계 아님(재귀 복사 방지)
   - **대상 `{new}/smarthb/app.db` 가 이미 존재하면 차단 + 안내**(덮어쓰기 방지, 사용자 확정 2026-06-10)
   - 대상 폴더 쓰기 권한 확인
2. **WAL 체크포인트**: 현재 풀에 `PRAGMA wal_checkpoint(TRUNCATE)` → WAL 내용을 `app.db` 본체로 반영(복사본 정합)
3. **재귀 복사** `{old}/smarthb/` → `{new}/smarthb/`
   - 제외: `app.lock`(새 폴더는 재시작 시 새 락 생성), `app.db-wal`·`app.db-shm`(체크포인트로 본체 반영 완료 → stale 방지)
   - 각 파일 복사 후 `fsync`(`ntfs-power-loss-pattern` 대응)
4. **검증**: 새 `{new}/smarthb/app.db` 를 임시 커넥션으로 열어 (cipher 빌드는 `PRAGMA key` 적용 후) `PRAGMA integrity_check` 통과 확인 — 실패 시 전체 롤백
5. **마커 파일**: 원본 `{old}/smarthb/MOVED_TO.txt` 에 새 경로 + 시각 기록(역방향 참조, 사용자 확정: 원본 유지 + 마커)
6. **config.json 갱신**(마지막 mutation): `cloud_folder_path = {new}` (atomic tmp→rename)
7. **원본 락 해제**(best-effort)
8. **프론트 재시작**: 성공 응답 → 사용자 안내 → `@tauri-apps/plugin-process` `relaunch()` → 새 프로세스가 새 경로로 초기화

### 실패/롤백

- 1~5 단계 실패 → 즉시 `Err` 반환. **config.json 미변경 → 앱은 기존 폴더로 계속 동작**(무손상). 부분 복사된 새 `smarthb/`(우리가 생성한 경우)는 best-effort 제거.
- 6 단계(config 갱신) 이후 실패는 사실상 없음(파일 1개 atomic rename). 만약 재시작 실패해도 다음 수동 실행 시 새 경로로 기동.
- **원본은 어떤 경우에도 삭제하지 않는다**(사용자 확정) → 최후의 복구 수단 보존.

## Consequences (결과)

### 긍정
- 원본 무손상 보장 — 실패해도 즉시 기존 상태로 복귀.
- salt.bin 동반 이전으로 새 폴더에서 동일 키(Keychain) + 동일 salt → 복호화 정상.
- WAL 체크포인트 + integrity_check 로 복사본 무결성 확인 후에만 전환.

### 부정 / 주의
- **양 PC 정합 한계**: `config.json`은 PC-로컬이라 한 PC에서 폴더를 바꿔도 **다른 PC는 자동 인식 못 함**. → 변경 완료 안내에 "다른 PC에서도 동일 폴더를 재지정해야 함"을 명시(UI 경고). 자동 전파는 하지 않음(클라우드에 경로를 두면 chicken-egg: 경로를 알아야 클라우드 설정을 읽음).
- 디스크 사용량 일시 2배(원본 보존). 사용자가 추후 수동 정리.
- cipher OFF(개발) 빌드에서는 검증이 평문 open 확인으로 축소 — 암호화 정합은 `--features cipher` 빌드에서 검증.

## Alternatives (기각)

- **move-then-update (원본 이동/삭제)**: 디스크 절약하나 중간 실패 시 데이터 유실 위험 → 기각(데이터 안전 최우선).
- **백엔드 `app.restart()` 즉시 호출**: 새 의존성 불요하나 사용자에게 완료/재시작 안내를 못 띄움 → UX 저하. 사용자 승인대로 plugin-process 프론트 relaunch 채택.
- **config.json 을 클라우드 폴더에 저장(양 PC 자동 정합)**: 경로를 알아야 읽을 수 있는 순환 의존 → 불가.
