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
- ⬜ 하드코딩된 시크릿, API 키, 비밀번호 없음
- ⬜ SQL 인젝션 방지 (ORM 파라미터 바인딩 사용)
- ⬜ 인증/인가 체크 누락 없음

### 품질
- ⬜ TypeScript 타입 안전성 (any 사용 최소화)
- ⬜ 에러 핸들링 적용
- ⬜ 새 기능에 pytest 테스트 추가

### CI
- ⬜ `pytest -v` 통과 확인
- ⬜ `pnpm test` 통과 확인 (프론트엔드 있을 경우)
- ⬜ Docker 빌드 성공

## 테스트 방법

```bash
# 백엔드 테스트
pytest backend/tests/ -v

# 로컬 스테이징
docker compose up --build
```

## 스크린샷 (UI 변경 시)

<!-- 변경된 UI 스크린샷을 첨부하세요 -->
