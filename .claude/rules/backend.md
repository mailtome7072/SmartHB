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
- 파일명 형식: `V{NNN}__{설명}.sql` (예: `V001__create_students.sql`)
- `sqlx migrate run` 으로 적용, `sqlx prepare` 로 `.sqlx/` 오프라인 캐시 갱신 후 커밋

### SQLx 쿼리
- `query!` 또는 `query_as!` 매크로 사용 — 컴파일 타임 타입 검사
- raw query string 직접 연결(concat) 금지 — SQL 인젝션 방지
- CI 환경: `SQLX_OFFLINE=true` + `src-tauri/.sqlx/` 커밋으로 DB 없이 컴파일

### 에러 처리
- `unwrap()`/`expect()` 프로덕션 코드 사용 금지 — `?` 연산자 + `thiserror` 크레이트 사용
- Tauri 커맨드 반환 타입: `Result<T, String>` (에러는 String으로 직렬화)
- 커스텀 에러 타입: `thiserror::Error` derive, `src-tauri/src/error.rs`에 정의

### 보안
- 환경변수는 런타임에 `std::env::var()` 또는 `.env` 파일로 로드 — 코드 하드코딩 금지
- API 키, 비밀번호 등은 `.env.example`에 키 이름만 기재
- Tauri `capabilities/` 에서 최소 권한 원칙 준수

### 테스트
- 새 커맨드 추가 시 `src-tauri/src/commands/` 각 모듈에 `#[cfg(test)]` 블록 작성
- SQLite 테스트: `DATABASE_URL=sqlite::memory:` 인메모리 DB 사용

## 코드 리뷰 우선 체크 항목

상세 체크리스트: `.claude/skills/code-review.md` — **보안**, **성능**, **테스트** 섹션 우선 확인

- **Critical**: SQL 인젝션 (raw query concat), 하드코딩된 시크릿, Tauri 권한 과다 허용
- **High**: `unwrap()` 남용 (panic 유발), 마이그레이션 없는 스키마 변경, `.sqlx/` 캐시 미갱신
