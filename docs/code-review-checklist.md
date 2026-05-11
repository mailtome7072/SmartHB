# 코드 리뷰 체크리스트

> **참고**: 이 파일은 사람이 읽기 위한 사본입니다.
> 코드 리뷰 체크리스트의 **단일 소스(SSOT)** 는 [`.claude/skills/code-review.md`](../.claude/skills/code-review.md)입니다.
>
> 체크리스트 내용을 수정할 때는 `.claude/skills/code-review.md`를 수정하고 이 파일도 함께 동기화하세요.
> sprint-close agent와 hotfix-close agent는 `.claude/skills/code-review.md`를 직접 참조합니다.

---

## 보안

- ⬜ 하드코딩된 시크릿, API 키, 비밀번호 없음
- ⬜ SQL 인젝션 방지 (ORM 파라미터 바인딩 사용)
- ⬜ XSS 방지 (React 기본 이스케이프 사용, 인라인 HTML 주입 최소화)
- ⬜ 인증/인가 체크 누락 없음

## 성능

- ⬜ N+1 쿼리 없음 (SQLAlchemy relationship 로딩 전략 확인)
- ⬜ 불필요한 API 호출 없음
- ⬜ 리스트 응답에 페이지네이션 적용

## 코드 품질

- ⬜ TypeScript 타입 안전성 (any 사용 최소화)
- ⬜ 에러 핸들링 (FastAPI HTTPException, 프론트엔드 에러 바운더리)
- ⬜ 구조화 로깅 (JSON 형식, Request ID 포함)

## 테스트

- ⬜ 새 기능에 pytest 테스트 추가 여부
- ⬜ 기존 테스트 회귀 없음 (`pytest -v` 통과)

## 패턴 준수

- ⬜ 프로젝트 컨벤션에 맞는 파일/디렉토리 구조
- ⬜ API 클라이언트 추상화 레이어 사용
