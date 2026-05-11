---
description: 백엔드 파일 작업 시 자동 로드. FastAPI/Python 개발 제약 및 품질 기준.
globs: ["app/backend/**/*.py", "app/backend/requirements*.txt", "app/backend/**/*.toml"]
---

## 백엔드 개발 필수 준수 사항

코드 생성 또는 수정 시 아래를 자동 적용한다:

### 테스트
- 새 API 엔드포인트 추가 시 `app/backend/tests/`에 pytest 통합 테스트 필수 작성

### DB 마이그레이션
- DB 스키마 변경(모델 추가/수정/삭제) 시 alembic migration 파일을 함께 생성한다

### 보안
- 환경변수는 반드시 `.env`에서만 로드 — 코드에 하드코딩 절대 금지
- API 키, 비밀번호, 토큰은 `.env.example`에 키 이름만 기재

### 성능
- SQLAlchemy relationship 조회 시 N+1 쿼리 방지: `joinedload` 또는 `selectinload` 사용
- 목록 API에는 페이지네이션 필수

### 코드 구조
- 새 엔드포인트는 `/api/v1/` 경로 구조 준수
- 비즈니스 로직은 `services/` 레이어에 분리

## 코드 리뷰 우선 체크 항목

상세 체크리스트: `.claude/skills/code-review.md` — **보안**, **성능**, **테스트** 섹션 우선 확인

- **Critical**: SQL 인젝션, 하드코딩된 시크릿, 인증/인가 누락
- **High**: N+1 쿼리, 페이지네이션 누락, 예외 미처리
