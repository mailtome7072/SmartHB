---
description: 프론트엔드 파일 작업 시 자동 로드. Next.js 15 + Tauri IPC 개발 제약 및 품질 기준.
globs: ["src/**/*.ts", "src/**/*.tsx", "src/**/*.css", "next.config.*", "tailwind.config.*"]
---

## 프론트엔드 개발 필수 준수 사항

코드 생성 또는 수정 시 아래를 자동 적용한다:

### TypeScript
- TypeScript strict 모드 준수 — `any` 타입 사용 최소화
- 공유 타입은 `src/types/` 디렉토리에 정의

### Tauri IPC 통합
- 컴포넌트에서 `@tauri-apps/api/core`의 `invoke()` 직접 호출 금지
- 반드시 `src/lib/tauri/` 추상화 레이어를 통해서만 Tauri 커맨드 호출
- Tauri IPC 응답 타입은 `src/types/`에 정의된 타입으로 명시

### Static Export 대응 (Next.js + Tauri)
- `window`, `document`, `navigator` 등 브라우저 전용 API 접근 시 반드시 가드:
  ```ts
  if (typeof window !== 'undefined') { ... }
  ```
- `'use client'` 지시어는 Tauri IPC 호출이 필요한 최소 컴포넌트에만 적용
- Server Component 우선 설계 (단, Tauri IPC는 Client Component에서만 가능)
- `next/image`의 `<Image>` 사용 시 `unoptimized` 속성 필수 (static export)

### UI 컴포넌트
- 새 UI 요소 작성 전 shadcn/ui 컴포넌트 라이브러리 우선 검토
- 재사용 컴포넌트는 `src/components/` 디렉토리에 분리
- Tailwind 클래스 사용, 인라인 style prop 지양

### 보안
- `dangerouslySetInnerHTML` 사용 지양 — XSS 방지
- 민감 정보(API 키 등) localStorage 저장 금지

### 에러 처리
- Tauri IPC 실패 시 UI 크래시 방지를 위한 try-catch + 사용자 친화적 에러 처리

## 코드 리뷰 우선 체크 항목

상세 체크리스트: `.claude/skills/code-review.md` — **보안**, **코드 품질**, **패턴 준수** 섹션 우선 확인

- **Critical**: XSS (dangerouslySetInnerHTML, 사용자 입력 직접 렌더링), `invoke()` 직접 호출
- **High**: TypeScript any 남용, SSR 가드 누락 (typeof window 미확인), `'use client'` 과다 사용
