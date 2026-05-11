---
description: 프론트엔드 파일 작업 시 자동 로드. TypeScript/React 개발 제약 및 품질 기준.
globs: ["app/frontend/**/*.ts", "app/frontend/**/*.tsx", "app/frontend/**/*.css", "app/frontend/next.config.*", "app/frontend/vite.config.*"]
---

## 프론트엔드 개발 필수 준수 사항

코드 생성 또는 수정 시 아래를 자동 적용한다:

### TypeScript
- TypeScript strict 모드 준수 — `any` 타입 사용 최소화
- 공유 타입은 `types/` 디렉토리에 정의

### API 통합
- 백엔드 API 호출은 반드시 `api/` 디렉토리의 클라이언트 추상화 레이어를 통해서만 호출
- 컴포넌트에서 `fetch`/`axios` 직접 호출 금지

### 보안
- `dangerouslySetInnerHTML` 사용 지양 — XSS 방지
- 인증 토큰은 httpOnly 쿠키 또는 메모리에만 저장 (localStorage 금지)

### UI 컴포넌트
- 새 UI 요소 작성 전 shadcn/ui 컴포넌트 라이브러리 우선 검토
- 재사용 컴포넌트는 `components/` 디렉토리에 분리

### 에러 처리
- API 실패 시 UI 크래시 방지를 위한 에러 바운더리 적용

## 코드 리뷰 우선 체크 항목

상세 체크리스트: `.claude/skills/code-review.md` — **보안**, **코드 품질**, **패턴 준수** 섹션 우선 확인

- **Critical**: XSS (dangerouslySetInnerHTML, 사용자 입력 직접 렌더링), 민감 정보 노출
- **High**: TypeScript any 남용, API 직접 호출 패턴
