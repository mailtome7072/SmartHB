# 코드 리뷰 체크리스트

> **참고**: 이 파일은 사람이 읽기 위한 사본입니다.
> 코드 리뷰 체크리스트의 **단일 소스(SSOT)** 는 [`.claude/skills/code-review.md`](../.claude/skills/code-review.md)입니다.
>
> 체크리스트 내용을 수정할 때는 `.claude/skills/code-review.md`를 수정하고 이 파일도 함께 동기화하세요.
> sprint-close agent와 hotfix-close agent는 `.claude/skills/code-review.md`를 직접 참조합니다.

---

## 보안

- ⬜ 하드코딩된 시크릿, API 키, 비밀번호, SQLCipher 키 없음
- ⬜ SQL 인젝션 방지 (`sqlx::query!` / `query_as!` 매크로 + 파라미터 바인딩, raw concat 금지)
- ⬜ XSS 방지 (React 기본 이스케이프 사용, `dangerouslySetInnerHTML` 사용 시 정당화 검토)
- ⬜ Tauri 권한 (`src-tauri/capabilities/`) 최소 허용 원칙 준수
- ⬜ 외부 네트워크 호출 없음 (PRD §5.5, §8.1 — 클라우드 동기화는 OS 클라이언트 위임)

## 성능

- ⬜ N+1 쿼리 없음 — 필요 시 JOIN 또는 사전 IN 조회로 묶기
- ⬜ 출결표 50명 × 31일 < 1초, 청구 생성 50명 < 3초, 공지문 50장 < 30초 (PRD §5.6)
- ⬜ 리스트 응답에 가상화 또는 페이지네이션 적용

## 코드 품질

- ⬜ TypeScript 타입 안전성 (any 사용 최소화, `src/types/` 공유 타입 활용)
- ⬜ Rust `unwrap()`/`expect()` 프로덕션 사용 없음 — `?` + `thiserror` 사용
- ⬜ Tauri IPC 추상화 레이어(`src/lib/tauri/`)만 사용 — 컴포넌트에서 `invoke()` 직접 호출 없음
- ⬜ 에러 메시지 사용자 친화 (PRD §6.4 — 기술 에러 코드/스택 노출 금지)

## 테스트

- ⬜ 새 비즈니스 규칙(보강 매칭, 소멸 처리, 청구 계산 등)에 `#[cfg(test)]` 단위 테스트 추가
- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
- ⬜ `pnpm tsc --noEmit && pnpm lint && pnpm build` 전체 통과
- ⬜ SQLx 스키마 변경 시 `sqlx prepare` 로 `.sqlx/` 캐시 갱신·커밋

## 패턴 준수

- ⬜ 프로젝트 컨벤션에 맞는 파일/디렉토리 구조 (`src/`, `src-tauri/src/commands/`, `src-tauri/migrations/`)
- ⬜ Tauri IPC 커맨드는 `src-tauri/src/commands/` + `lib.rs` invoke_handler 등록 + `src/lib/tauri/` 래퍼 3단 구조 준수
- ⬜ 글로벌 검색바 / Pretendard 18pt / 44×44px 클릭 영역 / 단축키 등 PRD §5.7 접근성 기준 준수
