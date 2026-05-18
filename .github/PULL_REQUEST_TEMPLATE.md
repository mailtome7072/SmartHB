# PR 설명

## 변경 유형

- ⬜ 버그 수정 (Hotfix)
- ⬜ 새 기능 (Sprint)
- ⬜ 리팩토링
- ⬜ 문서 수정

## 관련 스프린트 / 이슈

- 관련 Sprint: `sprint{n}` / Hotfix: `hotfix/{설명}`

## 변경 내용 요약

-

## 코드 리뷰 체크리스트

> 상세 기준: [`docs/dev-process.md` 섹션 7](docs/dev-process.md#7-코드-리뷰-체크리스트)

### 보안
- ⬜ 하드코딩된 시크릿, API 키, 비밀번호, SQLCipher 키 없음
- ⬜ SQL 인젝션 방지 (`sqlx::query!` 매크로 + 파라미터 바인딩 사용)
- ⬜ Tauri 권한 최소 허용 원칙 준수 (`src-tauri/capabilities/`)

### 품질
- ⬜ TypeScript 타입 안전성 (any 사용 최소화)
- ⬜ Rust `unwrap()`/`expect()` 프로덕션 사용 없음 (`?` + `thiserror`)
- ⬜ 새 비즈니스 규칙에 `#[cfg(test)]` 단위 테스트 추가

### CI
- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 통과 확인
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과 확인
- ⬜ `pnpm tsc --noEmit && pnpm lint && pnpm build` 통과 확인
- ⬜ SQLx 스키마 변경 시 `.sqlx/` 오프라인 캐시 갱신·커밋

## 테스트 방법

```bash
# 백엔드 테스트 (Rust/Tauri)
cargo test --manifest-path src-tauri/Cargo.toml

# 프론트엔드 검증
pnpm tsc --noEmit && pnpm lint && pnpm build

# 로컬 스테이징 (Tauri 앱 + Next.js dev)
pnpm tauri:dev
```

## 스크린샷 (UI 변경 시)

<!-- 변경된 UI 스크린샷을 첨부하세요 -->
