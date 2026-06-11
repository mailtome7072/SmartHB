---
description: Rust/Tauri 백엔드 파일 작업 시 자동 로드. SQLx/Tauri IPC 개발 제약 및 품질 기준.
globs: ["src-tauri/**/*.rs", "src-tauri/Cargo.toml", "src-tauri/migrations/**/*.sql"]
---

## 백엔드(Rust/Tauri) 개발 필수 준수 사항

코드 생성 또는 수정 시 아래를 자동 적용한다:

### Tauri 커맨드 구조
- Tauri IPC 커맨드는 반드시 `src-tauri/src/commands/` 디렉토리에 정의
- `src-tauri/src/lib.rs`에서 `tauri::Builder`에 등록
- 커맨드 함수 시그니처: `#[tauri::command] async fn my_cmd(...) -> Result<T, String>`

### DB 마이그레이션 (SQLx)
- DB 스키마 변경(테이블 추가/수정/삭제) 시 `src-tauri/migrations/` 에 마이그레이션 파일 필수 생성
- 파일명 형식: `V{NNN}__{설명}.sql` — `NNN` 은 **3자리 zero-pad** (V001, V002, ..., V010, V099)
  - 3자리는 향후 V100+ 확장 시 사전순 정렬을 보장 (`sqlx::migrate!` 가 파일명 사전순으로 적용)
  - 예: `V001__create_code_tables.sql`, `V101__create_students_and_schedules.sql`
  - **번호 정책 (실제 운용)**: 기능 영역별 100단위 블록을 끊는다.
    - `V001~V099` — 코드 테이블·감사 로그·앱 설정 (인프라성)
    - `V101~V199` — 핵심 도메인 (원생/스케줄/교습기간/수업료/학교)
    - `V200~V299` — 시드 데이터
    - 새 도메인 영역 추가 시 다음 100단위(V300~, V400~)에서 시작
- `sqlx migrate run` 으로 적용, `sqlx prepare` 로 `.sqlx/` 오프라인 캐시 갱신 후 커밋

### SQLx 쿼리
- `query!` 또는 `query_as!` 매크로 사용 — 컴파일 타임 타입 검사
- raw query string 직접 연결(concat) 금지 — SQL 인젝션 방지
- CI 환경: `SQLX_OFFLINE=true` + `src-tauri/.sqlx/` 커밋으로 DB 없이 컴파일

### DB 저장 위치 (PRD §5.3)
- 프로덕션 DB는 **클라우드 동기화 폴더 하위 `smarthb/app.db`** 에 위치 — Tauri `app_data_dir()` 사용 금지
- 경로는 사용자가 초기 설정 마법사(PRD §4.0)에서 지정 → 설정값을 secure config로 보관 후 startup에서 로드
- 개발 환경은 `./SmartHB-dev.db` (루트 기준, `.env`의 `DATABASE_URL`로 분리)

### SQLCipher 암호화 (PRD §5.1, §5.5)
- 프로덕션 DB는 SQLCipher AES-256 적용 — `sqlx` 기본 `sqlite` feature는 SQLCipher 미지원
- 도입 방법 후보: (1) `libsqlite3-sys` `bundled-sqlcipher` feature, (2) 시스템 sqlcipher 라이브러리 + 빌드 옵션. Sprint 시점에 ADR로 결정
- 암호화 키: 사용자 비밀번호를 PBKDF2로 유도 후 **OS Keychain(macOS) / Credential Manager(Windows)** 보관 — `keyring` crate 후보
- 키는 메모리에 최소 시간만 보유, 코드 어디에도 하드코딩·로깅 금지
- 사용자 비밀번호 분실 대비: 12자리 복구 코드(PRD §5.5) 발급/재발급 흐름 필요

### 동시성 제어 — app.lock (PRD §5.3)
- 앱 시작 시 클라우드 동기화 폴더의 `app.lock` 파일 확인 — 양 PC 시점 분리 사용 강제
- 락 구조: 디바이스 ID + 마지막 갱신 타임스탬프 → 60초마다 heartbeat 갱신
- 다른 디바이스가 점유 중이면 경고 화면 → "다른 PC에서 종료 후 재시도" 또는 5분 이상 미갱신 시 "강제 점유" 옵션
- 라이브러리 후보: `fs2` (advisory locking) + 자체 heartbeat. ADR로 결정
- 정상 종료 시 락 자동 해제, 비정상 종료 대비 5분 임계 시간 명확화

### 무결성 검증 (PRD §5.3, §5.4)
- 앱 시작 시 `PRAGMA integrity_check` 자동 실행 → 손상 감지 시 `backup/exit/` 최신본 자동 복원 + 사용자 알림 + 손상본 격리
- 백업 4계층: `backup/exit/`(5) + `backup/hourly/`(12) + `backup/daily/`(14) + `backup/weekly/`(4) — PRD §5.4 v1.5.2 (1인 시스템 축소, 합계 35). daily/weekly는 catch-up 방식(최근 백업 후 24h/7d 경과 시, 시작 시 + 매시 확인)
- 백업 구현: SQLite Online Backup API 사용 — `sqlx` 직접 지원 없음, `rusqlite::backup` 또는 sqlx raw connection 활용 검토
- 모든 백업 파일은 SQLCipher 암호화 상태 그대로 보관 (백업 시 복호화 금지)

### 에러 처리
- `unwrap()`/`expect()` 프로덕션 코드 사용 금지 — `?` 연산자 + `thiserror` 크레이트 사용
- Tauri 커맨드 반환 타입: `Result<T, String>` (에러는 String으로 직렬화)
- 커스텀 에러 타입: `thiserror::Error` derive, `src-tauri/src/error.rs`에 정의
- 사용자 친화 메시지 노출 — 기술 에러 코드/스택 트레이스 직접 노출 금지 (PRD §6.4)

### 보안
- 환경변수는 런타임에 `std::env::var()` 또는 `.env` 파일로 로드 — 코드 하드코딩 금지
- API 키, 비밀번호 등은 `.env.example`에 키 이름만 기재
- Tauri `capabilities/` 에서 최소 권한 원칙 준수 — 클라우드 폴더 접근/클립보드/인쇄 권한은 필요 시점에만 추가
- 외부 네트워크 통신 없음 — 클라우드 동기화는 OS 클라이언트가 담당, 앱은 로컬 파일만 접근 (PRD §5.5)

### 테스트
- 새 커맨드 추가 시 `src-tauri/src/commands/` 각 모듈에 `#[cfg(test)]` 블록 작성
- SQLite 테스트: `DATABASE_URL=sqlite::memory:` 인메모리 DB 사용 (개발/CI 분리, SQLCipher 미적용 모드)
- 비즈니스 규칙(보강 매칭, 소멸 처리, 청구 계산)은 100% 단위 테스트 커버 (PRD §6.5)
- 락 메커니즘, 백업, 무결성 검증, 자가 진단 로직은 독립 모듈로 분리하여 테스트

### 자가 진단 (PRD §6.6)
- 데이터 무결성 자가 진단 검사 항목: 보강필요시간 음수, 출결/청구 누락, 스케줄-출결 불일치, 소멸기한 누락, 고아 보강 데이터, 수납 정합성 등
- 매월 1일 첫 실행 시 자동 수행 + 수동 실행 가능, 최근 12개월 이력 보관

### 청구 마감 워크플로우 (PRD §4.9.7)
- 청구 데이터는 **3단계 상태**로 관리: `미확정` → `확정` → `마감`
- 상태 전이 규칙
  - 신규 생성: `미확정` (월 중 입퇴교 포함)
  - `확정`: 원장 검토 완료 후, 수정 시 확인 다이얼로그 필수
  - `마감`: 해당 월 모든 청구가 `확정` 상태일 때만 활성화 — 마감 후 수정 시 **사유 입력 필수**
- 스키마 권장: 청구 테이블에 `status` ENUM/TEXT 컬럼 + `closed_at`(마감 일시) + `close_reason`(마감 후 수정 사유) 분리
- AC-4.9-7/8 위반은 비즈니스 규칙 단위 테스트로 보장 (PRD §6.5)

### 분기 학습보고서 도메인 (PRD §4.8, §6.1)
- **분기 정의**: 학사력 기준 3·6·9·12월 시작 — 3~5월(1Q) / 6~8월(2Q) / 9~11월(3Q) / 12~2월(4Q)
- **UNIQUE 키**: `(분기, 원생)` — 한 원생은 한 분기에 보고서 1건
- **점수 종속**: 보고서는 단원평가 점수에 **직접 참조** — 점수 수정 시 보고서 표시 자동 반영. 보고서 저장 시 점수 스냅샷 복사 보관 금지
- **저장 필드 최소화**: `(분기 ID, 원생 ID, 종합의견, created_at, updated_at)`만 저장 (점수표·차트는 조회 시 동적 산출)
- **작성 가능 시점 제약**: 분기 마지막 월의 2차 단원평가 점수 입력 완료 후에만 작성 가능 — 백엔드 IPC 커맨드에서 검증 후 차단 (AC-4.8-6)
- **6회 미만 시행 대응**: 점수 조회 IPC는 실제 시행 회차만 반환 (NULL 패딩 금지)

### 핵심 비즈니스 키 & 제약 (PRD §6.2)
SQLx 마이그레이션 작성 시 아래 UNIQUE/CHECK 제약을 누락 없이 반영한다.

| 테이블 | 제약 |
|--------|------|
| 원생 | `일련번호` UNIQUE |
| 수업 스케줄 | `(원생, 요일)` UNIQUE |
| 정규 출결 | `(원생, 일자)` UNIQUE |
| 보강 출결 | 같은 일자 중복 허용 (UNIQUE 제약 금지) |
| 학습보고서 | `(분기, 원생)` UNIQUE |
| 청구 | `(원생, 청구년월)` UNIQUE |
| 학사 일정 | 중복불가 코드는 `(일자, 코드)` UNIQUE |
| 교습기간 | 일자 중첩 금지 (CHECK 또는 트리거) |

## 코드 리뷰 우선 체크 항목

상세 체크리스트: `.claude/skills/code-review.md` — **보안**, **성능**, **테스트** 섹션 우선 확인

- **Critical**: SQL 인젝션 (raw query concat), 하드코딩된 시크릿/암호화 키, Tauri 권한 과다 허용, SQLCipher 키를 Keychain 외부에 저장
- **High**: `unwrap()` 남용 (panic 유발), 마이그레이션 없는 스키마 변경, `.sqlx/` 캐시 미갱신, `PRAGMA integrity_check` 누락, 락 파일 heartbeat 미구현, **PRD §6.2 UNIQUE 제약 누락** (특히 학습보고서 `(분기, 원생)` / 청구 `(원생, 청구년월)`)
- **Medium**: `app_data_dir()` 사용 (클라우드 동기화 폴더 정책 위반), 백업 로직이 트랜잭션 외부 실행, **학습보고서가 단원평가 점수를 복사 보관** (점수 종속 원칙 위반 — PRD §4.8.3), **청구 마감 후 수정 시 사유 입력 미강제** (AC-4.9-8 위반)
