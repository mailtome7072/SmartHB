# ADR-003: SQLCipher 자동 백업 구현 방식 선정

- **상태**: Proposed
- **날짜**: 2026-05-19
- **결정자**: SmartHB 개발팀 (Sprint 1 T7)

> **개정 (2026-06-11, Sprint 16 / PRD v1.5.2)**: 보관 개수 축소 — `exit(10→5)` / `hourly(24→12)` / `daily(30→14)` / `weekly(4 유지)`, 합계 68→35. 1인 사용 시스템 + 백업 위치가 클라우드 동기화 폴더(업로드 트래픽·용량 점유)인 점 반영. daily/weekly 트리거는 catch-up 방식(최근 백업 후 24h/7d 경과 시 생성, 시작 시 + hourly tick마다 확인)으로 구현 — 본문 수치는 결정 당시 기록 그대로 보존.

## Context (배경)

PRD §5.3 / §5.4 는 4계층 자동 백업(`exit(10)` / `hourly(24)` / `daily(30)` / `weekly(4)`)을 의무로 규정한다. 모든 백업 파일은 SQLCipher 암호화 상태 그대로 보관되어야 하며 (복호화 금지), exit 백업은 종료 직전 ~50ms 예산 내 동기 완료되어야 한다. hourly 백업은 UI 비블로킹으로 백그라운드에서 수행한다.

### 제약 사항

- 프로덕션 DB 는 SQLCipher AES-256 암호화 (PRD §5.5, ADR-001) — 백업 파일도 동일 키로 암호화 상태 유지
- 양 OS (Windows 10/11 + macOS 12+) 지원
- 백업 후 `PRAGMA quick_check` 즉시 검증 (PRD §5.4) — 실패 시 파일 폐기
- 백업 중 동시 쓰기 발생 가능 — 트랜잭션 일관성 필수
- `cipher` feature off (개발 빌드, ADR-001) 호환성 검토 필요
- sqlx 0.8.6 + libsqlite3-sys 0.30 기존 의존성 매트릭스 보존

---

## 1단계: Weighted Decision Matrix

| 기준 | 가중치 | 선택지 A | A | 선택지 B | B | 선택지 C | C |
|------|--------|---------|---|---------|---|---------|---|
| 빌드 단순성 (cipher feature 호환) | 0.20 | rusqlite 의존성 1개 추가, libsqlite3-sys 0.30 sqlx 와 매칭 | **4** | 의존성 추가 0 — sqlx 만 사용 | **5** | 의존성 추가 0 | **5** |
| SQLCipher 호환성 (암호화 유지) | 0.25 | Online Backup API 가 PRAGMA key 자동 전파 — 공식 권장 패턴 | **5** | sqlx connection 에 적용된 키 그대로 — FFI 직접 호출이라 검증 부담 | **4** | `VACUUM INTO` 결과 DB 는 평문 — `ATTACH ... KEY ...` 우회 필요, 공식 권장 아님 | **2** |
| 성능 (~50ms exit 백업) | 0.20 | Online Backup API page-level copy, 수십 ms | **5** | 동일 (FFI 직접 호출) | **5** | 전체 DB 재작성 — 큰 DB 에서 점진 성능 저하 | **3** |
| 트랜잭션 일관성 (백업 중 동시 쓰기) | 0.15 | SQLite 내부 page lock + checkpoint 처리 | **5** | 동일 | **5** | VACUUM INTO 도 트랜잭션 일관성 보장 | **5** |
| API 단순성 | 0.10 | `Connection::backup()` 한 줄 + progress callback 지원 | **5** | unsafe FFI 호출 + 핸들 관리 + 라이프타임 | **2** | `sqlx::query("VACUUM INTO 'path'")` 한 줄 | **5** |
| 유지보수성 (crate 활성도) | 0.10 | rusqlite 0.32 안정 (2024-09 릴리스), 광범위 채택 | **5** | libsqlite3-sys 직접 사용 — SQLite C API 버전 변경 시 영향 | **3** | sqlx 만 사용 — 가장 단순 | **5** |

- A 총점 = 4×0.20 + 5×0.25 + 5×0.20 + 5×0.15 + 5×0.10 + 5×0.10 = 0.80+1.25+1.00+0.75+0.50+0.50 = **4.80**
- B 총점 = 5×0.20 + 4×0.25 + 5×0.20 + 5×0.15 + 2×0.10 + 3×0.10 = 1.00+1.00+1.00+0.75+0.20+0.30 = **4.25**
- C 총점 = 5×0.20 + 2×0.25 + 3×0.20 + 5×0.15 + 5×0.10 + 5×0.10 = 1.00+0.50+0.60+0.75+0.50+0.50 = **3.85**
- A 우세 (B 대비 0.55, C 대비 0.95) — SQLCipher 호환성 + API 단순성 + 유지보수성에서 결정적 차이

---

## 2단계: SWOT + Trade-off

### 선택지 A: `rusqlite` + Online Backup API

- **Strengths**
  1. SQLite 공식 권장 백업 방식 (`sqlite3_backup_init/step/finish` 래퍼)
  2. SQLCipher PRAGMA key 자동 전파 — source/destination connection 모두 동일 키 적용 시 암호화 상태 그대로 page-by-page 복사
  3. `Connection::backup(DatabaseName::Main, dest_path, Some(progress_fn))` 한 줄 호출
- **Weaknesses**
  1. 의존성 1개 추가 — rusqlite 0.32 + `backup` feature
  2. cipher feature on/off 에 따라 rusqlite 의존성 게이트 설계 필요 (libsqlite3-sys 0.30 충돌 방지)
- **Opportunities**
  1. Online Backup API progress callback — 향후 큰 DB 백업 시 UI 진행률 표시 가능
  2. rusqlite 의 `Connection` 을 백업 후 `PRAGMA quick_check` 검증에 재사용
- **Threats**
  1. rusqlite/sqlx 의 libsqlite3-sys 버전 차이로 빌드 충돌 가능 → feature 게이트로 mitigation
  2. rusqlite crate deprecate 가능성 (낮음, 활발 유지)

### 선택지 B: `sqlx` raw connection + SQLite C API FFI

- **Strengths**
  1. 의존성 추가 0 — 기존 sqlx + libsqlite3-sys 재사용
  2. connection pool 단일화 — 운영 단순
- **Weaknesses**
  1. `unsafe` FFI 호출 + raw connection 핸들 라이프타임 관리
  2. `sqlite3_backup_init/step/finish` API 직접 호출 — 코드 양 약 3배 증가
  3. sqlx 의 connection 을 libsqlite3-sys raw pointer 로 변환하는 unstable API 의존
- **Opportunities**
  1. 의존성 graph 최소화
- **Threats**
  1. libsqlite3-sys major 업데이트 시 API 수동 마이그레이션 부담
  2. sqlx 내부 connection 구현 변경 시 raw access 코드 깨질 위험

### 선택지 C: `sqlx::query("VACUUM INTO ...")`

- **Strengths**
  1. SQL 한 줄 — 가장 단순한 코드
  2. SQLite 내부 트랜잭션 일관성 자동 보장
- **Weaknesses**
  1. **`VACUUM INTO 'path'` 결과 DB 는 평문** — SQLCipher 키 미전파 (SQLCipher 공식 문서 한계)
  2. 우회: `ATTACH DATABASE 'path' AS bak KEY 'k'; SELECT sqlcipher_export('bak'); DETACH bak;` — 별도 패턴, 공식 권장 아님
  3. 전체 DB 재작성 — VACUUM 특성상 큰 DB 에서 점진 성능 저하 (PRD 50ms 예산 위협)
- **Opportunities**
  1. 백업 + 무결성 검증 통합 (VACUUM 자체가 일관성 검증)
- **Threats**
  1. SQLCipher 백업 흐름이 공식 권장 패턴 아님 — 향후 SQLCipher 메이저 업데이트 시 동작 변경 가능
  2. `sqlcipher_export` 우회 시 키 메모리 누출 위험 (현재 메모리 zeroize 정책과 충돌)

### Trade-off

| 선택 시 | 개선 (↑) | 저하 (↓) |
|---------|----------|----------|
| **A 선택** | API 단순, SQLCipher 자동 호환, 공식 권장 방식, 성능 우수 | rusqlite 의존성 1개, cipher feature 게이트 설계 |
| B 선택 | 의존성 추가 0, connection pool 단일 | `unsafe` FFI, 코드 양 3배, 유지보수 부담 |
| C 선택 | 코드 가장 단순 (SQL 한 줄) | SQLCipher 별도 export 패턴, 대용량 성능, 보안 검증 부담 |

### Risk

| 리스크 | 관련 | 영향도 | 완화 |
|--------|------|--------|------|
| rusqlite/sqlx 의 libsqlite3-sys 버전 충돌 | A | 중간 | cipher feature 게이트로 rusqlite optional 처리 + version pin (libsqlite3-sys 0.30 양쪽 동일) |
| Online Backup API 백업 중 동시 쓰기 | A·B 공통 | 낮음 | SQLite 내부 page-level lock + checkpoint 처리 — 별도 락 불필요 |
| `cipher` feature off (개발 빌드) 에서 백업 IPC 호출 | A | 낮음 | `#[cfg(feature = "cipher")]` 게이트로 백업 모듈 실구현/스텁 분리. 스텁은 사용자 친화 안내 메시지 반환 |
| `VACUUM INTO` 결과 DB 평문 노출 | C | **높음** | `sqlcipher_export` 우회 시 별도 보안 검증 필요 — 채택 시 위협 전담 검토 |
| 백업 디렉토리 디스크 공간 부족 | 공통 | 중간 | 백업 후 `PRAGMA quick_check` + 실패 시 파일 폐기 + AppError::Backup 사용자 메시지 |

---

## 3단계: Decision

**선택지 A — `rusqlite` 0.32 + Online Backup API + `bundled-sqlcipher-vendored-openssl` 채택**.

> 1단계 총점: A=4.80, B=4.25, C=3.85 → A 우세 (B 대비 0.55, C 대비 0.95)
> 핵심 Trade-off: A 채택으로 의존성 1개 추가 및 cipher feature 게이트 설계 부담을 감수하는 대신, SQLCipher 공식 권장 백업 방식 / 단순 API / 광범위 채택 / progress callback 확장성을 얻는다. C 의 `VACUUM INTO` 우회는 보안 검증 부담이 본 sprint capacity 를 초과한다.

### 구체 적용 방안

1. **Cargo.toml 추가** (cipher feature 게이트):
   ```toml
   [dependencies]
   rusqlite = { version = "0.32", features = ["bundled-sqlcipher-vendored-openssl", "backup"], optional = true }

   [features]
   default = []
   cipher = ["dep:libsqlite3-sys", "dep:rusqlite"]
   ```
   - `cargo build` (default): rusqlite 없음 → backup 모듈 스텁
   - `cargo build --features cipher`: rusqlite + Online Backup API 활성 → 정식 백업
   - libsqlite3-sys 0.30 은 sqlx 0.8.6 + rusqlite 0.32 양쪽이 공통으로 의존 → 중복 빌드 없음

2. **계층 enum + 보관 정책**:
   ```rust
   #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
   #[serde(rename_all = "kebab-case")]
   pub enum BackupLayer { Exit, Hourly, Daily, Weekly }

   impl BackupLayer {
       pub fn max_keep(self) -> usize {
           match self { Self::Exit=>10, Self::Hourly=>24, Self::Daily=>30, Self::Weekly=>4 }
       }
       pub fn subdir(self) -> &'static str {
           match self { Self::Exit=>"exit", Self::Hourly=>"hourly",
                        Self::Daily=>"daily", Self::Weekly=>"weekly" }
       }
   }
   ```

3. **백업 메타데이터** (IPC 응답):
   ```rust
   #[derive(Debug, Serialize)]
   pub struct BackupMetadata {
       pub path: String,
       pub layer: BackupLayer,
       pub created_at: DateTime<Utc>,
       pub size_bytes: u64,
   }
   ```

4. **IPC 함수** (cipher feature on 기준):
   - `create_backup(layer)`: 지정 계층에 `app_YYYYMMDD_HHMMSS.db` 생성 → `PRAGMA quick_check` 검증 → 순환 삭제
   - `list_backups(layer)`: 계층별 또는 전체 백업 메타데이터 조회
   - `restore_backup(path)`: T8 무결성 모듈과 통합 (현 sprint 에서는 스텁 + 검증 로직만)

5. **파일명 규칙**: `app_YYYYMMDD_HHMMSS.db` (UTC 기준) — 정렬 가능, 충돌 없음

6. **백업 위치**: T7 임시로 `./SmartHB-data/backup/{exit,hourly,daily,weekly}/` (dev). T9 마법사 통합 시 클라우드 동기화 폴더 하위 `smarthb/backup/...` 로 이전.

7. **순환 삭제**: 계층 디렉토리 스캔 → 파일명(타임스탬프) 정렬 → `max_keep` 초과 시 가장 오래된 파일 제거. 백업 직후 동기 수행 (~5ms).

8. **`PRAGMA quick_check` 검증**: 백업 완료 직후 rusqlite Connection 으로 검증. 실패 시 파일 삭제 + `AppError::Backup` 반환.

9. **동시성**: 백업 IPC 호출은 spawn_blocking 처리. `Arc<Mutex<()>>` 또는 module 내부 OnceLock 으로 동시 백업 1건으로 제한.

10. **cipher off 빌드**: backup 모듈 stub — `AppError::Backup("암호화 빌드에서만 백업이 가능합니다.")` 반환. 개발 환경 한정 동작.

11. **백그라운드 task 통합**: T7 에서는 IPC 만 구현. hourly 백그라운드 task 는 T10 (시작 시퀀스) 에서 `tokio::spawn` + 1시간 interval.

---

## Consequences

### 긍정적 영향

- SQLite 공식 권장 백업 방식 사용 — 향후 SQLCipher/SQLite 업데이트 시 호환성 보장
- API 단순 — `Connection::backup()` 한 줄 + progress callback 확장 여지
- 백업 후 `PRAGMA quick_check` 즉시 검증 — 손상 백업 자동 폐기
- T8 (무결성 검증) 에서 rusqlite Connection 재사용 가능 — 모듈 통합 단순화

### 부정적 영향 / 주의사항

- 의존성 1개 추가 (rusqlite 0.32) — `bundled-sqlcipher-vendored-openssl` feature 활성화 시 Strawberry Perl 필요 (이미 ADR-001 에서 처리됨)
- `cipher` feature off 빌드에서는 백업 미동작 — 개발 환경에서 백업 흐름 테스트 불가 (수용 가능, 프로덕션은 항상 cipher on)
- cipher feature 게이트 설계 — rusqlite optional 처리 + 모듈 `#[cfg]` 분기 코드 발생

### 후속 액션

- **T8 (무결성 검증)**: rusqlite Connection 재사용으로 `PRAGMA quick_check` / `integrity_check` 통합. 손상 시 `backup/exit/` 최신본 자동 복원 로직.
- **T10 (시작 시퀀스)**: hourly 백업 백그라운드 task 시작 (`tokio::spawn` + 1시간 interval), exit 백업은 종료 hook 에서 동기 호출.
- **T9 (마법사)**: 백업 디렉토리 정식 경로 (클라우드 폴더 하위 `smarthb/backup/...`) 이전.
- **T11 (테스트)**: cipher feature on/off 양쪽 빌드에서 모듈 동작 검증 — cipher off 는 스텁 메시지 반환 확인.
- **CI**: `.github/workflows/` 의 빌드 매트릭스에 `cargo build --features cipher` 추가 (Forbidden Area, 사용자 허가 후 진행).
- **장기**: PRD §5.3 — 백업 디렉토리 디스크 사용량 모니터링 IPC 추가 검토 (수동 정리 옵션).
