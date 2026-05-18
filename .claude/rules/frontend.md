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

### UI 컴포넌트 (PRD §5.1)
- 새 UI 요소 작성 전 **shadcn/ui** 컴포넌트 라이브러리 우선 검토
- 재사용 컴포넌트는 `src/components/` 디렉토리에 분리
- Tailwind 클래스 사용, 인라인 style prop 지양
- **캘린더 뷰**: PRD §4.4(3개월 학사 캘린더) / §4.6(일/주/월 수업 뷰)에 FullCalendar 또는 React Big Calendar 사용 — Sprint 진입 시 비교 후 ADR로 결정
- **이미지 생성**: PRD §4.10 교습비 공지문 PNG 일괄 생성에 HTML5 Canvas + `html-to-image` 사용
- **차트 (학습보고서 추이)**: PRD §4.8.2 점수 추이 차트(가로축 회차, 1차·2차 점 + 분기 평균 라인) 구현 시 **Recharts 또는 Chart.js** 사용 — Sprint 진입 시 ADR로 결정 (인쇄 렌더링 품질·번들 크기 비교)

### 분기 학습보고서 화면·인쇄 (PRD §4.8)
- **분기 단위 작성**: 작성 주기는 분기 1회(학사력 3·6·9·12월 시작). 월 단위 보고서 UI 신설 금지
- **종합의견 단일 입력**: 멀티라인 `<textarea>` 1개로 단일화 (이전 3종 분리 입력 폐기) — 줄바꿈은 인쇄에 그대로 반영
- **참고 정보 영역 (읽기 전용)**: 해당 분기 6회 점수(회차별 1·2차) + 당해 연도 모든 과거 분기 점수 표 + 점수 추이 차트
- **차트 동적 구성**: 분기 6회 미만 시행 시 실제 시행 회차만 표시 (NULL 패딩 금지)
- **작성 시점 제약 UI**: 분기 마지막 월의 2차 단원평가 점수 미입력 시 보고서 입력 버튼 비활성화 + 안내 메시지 (AC-4.8-6)
- **인쇄 레이아웃**: A4 1장에 4명, **4등분 박스** — 각 박스 내부 수직 배치
  1. 상단: 원생 성명 + 분기 점수표 (회차 × 1·2차)
  2. 중단: 점수 추이 그래프 (회차별 점 + 분기 평균 라인)
  3. 하단: 종합의견 (멀티라인 줄바꿈 반영)
- **인쇄 전용 스타일**: CSS `@media print` + Tailwind `print:` variant로 박스 비율 균등 보장 (AC-4.8-4). 파일 저장(PDF) 기능은 §4.13 데이터 내보내기 메뉴에서 별도 처리, 보고서 화면 자체는 인쇄 직접 출력만 지원

### 상태 관리 (PRD §5.1)
- **Zustand**: 전역 상태(현재 사용자 세션, 락 점유 상태, 선택된 교습기간월 등)
- **TanStack Query**: Tauri IPC 응답 캐싱, 무효화, 백그라운드 새로고침
- 컴포넌트 내부 상태는 React `useState`로 충분 — 무리한 전역화 금지

### 접근성 — 50대 사용자 친화 (PRD §5.7)
- **폰트**: Pretendard 본문 18pt 권장(16pt 하한), 헤더 24pt+, 행간 1.5 — `src/app/globals.css` 또는 Tailwind config에 통일
- **색상**: 저자극 톤(차분한 베이지/연그레이 배경, 강조색은 무채도 기반)
- **명도 대비**: ≥ 4.5:1 (WCAG AA 준수)
- **클릭 영역**: 최소 44×44px — 버튼/체크박스 패딩 명시
- **키보드 단축키**: F1(도움말) / Ctrl+F(글로벌 검색 포커스) / Ctrl+N(신규 원생) / Ctrl+S(저장) / Ctrl+Z(Undo) / ESC(다이얼로그 닫기) / Ctrl+P(인쇄)
- 메뉴에 단축키 표기 병기 (예: "신규 원생 등록 (Ctrl+N)")

### 글로벌 검색바 (PRD §4.14)
- 모든 화면 상단에 글로벌 검색바 **상시 노출** — 레이아웃 컴포넌트(`src/app/layout.tsx`) 의무 구성요소
- 검색 대상: 원생 이름(우선) / 학교명 / 메뉴명
- 한글 자모 부분 일치 + 영문 대소문자 무관, 200ms 디바운싱, 300ms 이내 결과 표시
- 검색 결과 클릭 시 별도 메뉴 경유 없이 해당 화면으로 1클릭 이동

### 실수 복구 메커니즘 (PRD §5.7)
- 양식 입력 화면(원생 등록·수정, 학습보고서, 청구 조정 등)은 3분 단위 자동 임시저장
- 출결 토글·청구 금액 조정·보강 등록 등 주요 행위는 1단계 Undo 지원
- 메뉴 이동/창 닫기 시 미저장 변경분 있으면 경고 다이얼로그
- 위험 동작(삭제, 청구 재생성, 보강 삭제, 백업 복원, 보강소멸 환원)은 명시적 확인 대화상자

### 보안
- `dangerouslySetInnerHTML` 사용 지양 — XSS 방지
- 민감 정보(API 키, 사용자 비밀번호, SQLCipher 키 등) localStorage 저장 금지 — OS Keychain은 백엔드(Rust) 영역
- 외부 네트워크 호출 금지 — 모든 데이터 흐름은 Tauri IPC 경유

### 에러 처리
- Tauri IPC 실패 시 UI 크래시 방지를 위한 try-catch + 사용자 친화적 에러 처리
- 기술 에러 코드/스택은 콘솔에만, 사용자 화면에는 50대 친화적 한글 메시지

## 코드 리뷰 우선 체크 항목

상세 체크리스트: `.claude/skills/code-review.md` — **보안**, **코드 품질**, **패턴 준수** 섹션 우선 확인

- **Critical**: XSS (dangerouslySetInnerHTML, 사용자 입력 직접 렌더링), `invoke()` 직접 호출, SQLCipher 키나 사용자 비밀번호를 프론트엔드 메모리에 보관
- **High**: TypeScript any 남용, SSR 가드 누락 (typeof window 미확인), `'use client'` 과다 사용, 글로벌 검색바 누락, Pretendard/18pt/44×44px 접근성 기준 위반, **학습보고서 월 단위 UI 신설** (분기 단위 위반)
- **Medium**: Zustand/TanStack Query 적용 누락 (불필요한 props drilling), 임시저장/Undo 미적용, 단축키 미바인딩, **학습보고서 A4 4분할 인쇄 레이아웃 미적용** (AC-4.8-4 위반), **분기 마지막 월 2차 점수 미입력 시 입력 버튼 비활성화 미적용** (AC-4.8-6 위반)
