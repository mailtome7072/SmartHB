# Sprint Plan sprint3

## 기간
2026-05-21 ~ 2026-06-03 (2주, 10 영업일)

## 목표
Phase 1 최종 스프린트로서, (1) 초기 설정 마법사(PRD SS4.0)와 salt 이전(R12)을 완성하여 사용자 첫 실행 흐름을 확립하고, (2) 원생 관리/코드 테이블 프론트엔드 화면을 구현하며, (3) Sprint 2 backlog(R13 PII 마스킹, R14 페이지네이션)를 해소한다. 앱 레이아웃 셸(사이드바 + 글로벌 검색바 + 상태바)과 접근성 기반(Pretendard 18pt, WCAG AA)을 확립하여 Phase 2 이후 모든 화면의 토대를 제공한다.

## ROADMAP 연계 기능
- Phase 1 > Sprint 3: 원생 관리 프론트 + 초기 설정 마법사 + 글로벌 검색 + 접근성 기반
- Phase 1 완료 마일스톤 M2: 원생 등록/조회 + 마법사 + 글로벌 검색
- Sprint 2 backlog: R12 salt 이전, R13 PII 마스킹, R14 페이지네이션

## 핵심 제약
- **마이그레이션 번호 예약**: V200~V299 (Sprint 3). 기존 V001/V008(Sprint 1), V101~V105(Sprint 2).
- **PR 단계 생략**: 단일 개발자 단일 저장소. `gh pr create` 호출 금지.
- **단일 사용자 가정**: PRD 원장 1인 모델.
- **하네스 5대 원칙**: scope.md 선 작성 후 코드 수정, 3-retry 원칙.
- **테스트 우선**: 비즈니스 규칙 100% 단위 테스트 커버 (PRD SS6.5).
- **접근성 기준**: Pretendard 본문 18pt, 헤더 24pt+, 행간 1.5, 명도 대비 4.5:1 이상, 클릭 영역 44x44px.
- **Forbidden Area**: `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` 변경 금지.
- **신규 의존성**: `tauri-plugin-dialog` (폴더 선택), `@tauri-apps/plugin-dialog` (JS 바인딩) 추가 필요 -- 사용자 허가 필요.
- **Pretendard 폰트**: ADR-006 결정에 따라 `public/fonts/` self-host 방식.

---

## 이전 회고 반영

> 출처: `docs/sprint-retrospectives/sprint2-retrospective.md`

- **A1** (PII 마스킹): `students.rs`의 `try_record` 3곳에서 `details=None` 전달로 수정 --> T2에서 처리
- **A3** (페이지네이션): `list_students`/`list_codes`에 `page/limit` 파라미터 추가 --> T3에서 처리
- **A4** (sprint-planner 사전 검토): data-model.md SSOT 대조 + 코드 현황 확인 --> 본 계획에서 적용 완료 (마이그레이션 컬럼 타입 사전 검증)
- **A5** (마이그레이션 V{NNN} 표기 통일): 별도 hotfix 또는 Sprint 3 포함 -- **사용자 결정 필요** (아래 옵션 참조)
- **A6** (cipher on 실측): v0.2.0 배포 후 사용자 환경에서 측정 -- Sprint 3 초반에 결과 수집

---

## 작업 목록

### Day 1: Sprint 2 backlog 해소 (백엔드)

- ⬜ **T1: Pretendard 폰트 self-host 설정** (0.5일)
  - `public/fonts/` 디렉토리에 Pretendard woff2 파일 배치
  - `src/app/globals.css`에 `@font-face` 선언 + Tailwind config `fontFamily` 설정
  - 본문 18px, 헤더 24px+, 행간 1.5 기본값 확립
  - **검증**: `pnpm build` 성공 + 폰트 로드 확인

- ⬜ **T2: R13 audit details PII 마스킹** (0.5일)
  - `src-tauri/src/commands/students.rs`의 `try_record` 호출 3곳 수정
  - `create_student`: `details` --> `None` (event_subject에 student id만 기록)
  - `update_student`: `details` --> `None`
  - `withdraw_student`: `details` --> `None`
  - **검증**: `cargo test` 통과 + 감사 로그에 원생 이름 미포함 확인

### Day 2: Sprint 2 backlog 해소 (백엔드)

- ⬜ **T3: R14 list_students / list_codes 페이지네이션** (1일)
  - `StudentFilter` 구조체에 `limit: Option<u32>`, `offset: Option<u32>` 추가
  - `list_students` SQL에 `LIMIT ? OFFSET ?` 적용 (기본 limit=100, 상한 limit_max=1000)
  - `count_students(filter)` 신규 IPC 커맨드 추가 (총 건수 반환)
  - `list_codes`에도 동일하게 `limit/offset` 적용
  - 프론트엔드 IPC 래퍼 업데이트: `src/lib/tauri/index.ts`
  - **검증**: 단위 테스트 -- limit/offset 경계값, count 정확성

### Day 3~4: 앱 레이아웃 셸 + 상태 관리 기반

- ⬜ **T4: Zustand 스토어 + TanStack Query 설정** (0.5일) - skill: frontend-design
  - `pnpm add zustand @tanstack/react-query` (신규 의존성)
  - `src/stores/session-store.ts`: 세션 상태 (인증 여부, 디바이스 정보)
  - `src/stores/app-store.ts`: 락 점유 상태, 선택된 교습기간월, 사이드바 열림/닫힘
  - `src/providers/query-provider.tsx`: TanStack Query Provider (client component)
  - `src/app/layout.tsx`에 Provider 래핑
  - **검증**: `pnpm tsc --noEmit` + `pnpm lint` 통과

- ⬜ **T5: 앱 레이아웃 셸 (사이드바 + 상단바)** (1일) - skill: frontend-design
  - `src/components/layout/sidebar.tsx`: 메뉴 항목 + 단축키 표기 병기
    - 메뉴: 대시보드, 원생관리, 수업관리, 출결관리, 청구관리, 단원평가, 학습보고서, 공지문, 설정
    - 비활성 메뉴(Phase 2+ 영역)는 `disabled` 스타일 + 툴팁 "다음 업데이트 예정"
  - `src/components/layout/top-bar.tsx`: 점유 디바이스, 마지막 백업 시각, 동기화 상태
  - `src/components/layout/app-shell.tsx`: 사이드바 + 상단바 + 콘텐츠 영역 조합
  - 저자극 톤 (베이지/연그레이 배경), 명도 대비 4.5:1 이상
  - `src/app/layout.tsx` 업데이트: 인증 완료 후 AppShell 렌더링
  - **검증**: `pnpm build` + 시각적 확인 (Pretendard 18pt, 44x44px 클릭 영역)

- ⬜ **T6: 글로벌 검색바 (PRD SS4.14)** (0.5일) - skill: frontend-design
  - `src/components/layout/global-search.tsx`: 상단바 내 상시 노출
  - 검색 대상: 원생 이름(우선), 학교명, 메뉴명
  - 한글 자모 부분 일치 구현 (hangul-js 또는 자체 분해 로직)
  - 200ms 디바운싱 + 300ms 이내 결과 표시
  - 결과 클릭 시 해당 화면 1클릭 이동
  - Ctrl+F 단축키 바인딩
  - **검증**: 원생 이름 검색 + 메뉴 검색 동작 확인

### Day 5~6: 초기 설정 마법사 인프라 (백엔드 + 프론트)

- ⬜ **T7: tauri-plugin-dialog 의존성 추가 + 폴더 선택 IPC** (0.5일)
  - `cargo add tauri-plugin-dialog --manifest-path src-tauri/Cargo.toml`
  - `pnpm add @tauri-apps/plugin-dialog`
  - `src-tauri/src/lib.rs` Builder에 `.plugin(tauri_plugin_dialog::init())` 등록
  - `src-tauri/capabilities/default.json`에 `dialog:default` 권한 추가
  - `src/lib/tauri/index.ts`에 `selectFolder()` 래퍼 추가
  - **검증**: `cargo build` + `pnpm tsc --noEmit` 통과

- ⬜ **T8: 마법사 백엔드 -- 설정 저장 + salt 이전 + DB 경로 동적화** (1일)
  - `src-tauri/src/commands/setup.rs` 신규 모듈:
    - `save_cloud_folder(path: String)`: `app_settings` 테이블에 `cloud_folder_path` 저장
    - `complete_setup()`: 마법사 완료 플래그 설정
    - `get_setup_status()`: 마법사 완료 여부 반환
  - R12 salt 이전 로직:
    - 마법사에서 클라우드 폴더 선택 완료 시 `{cloud_folder}/smarthb/salt.bin`에 salt 저장
    - 기존 Keychain salt 있으면 파일로 복사 후 Keychain 항목 삭제
    - `paths::data_root()` 를 `app_settings.cloud_folder_path` 조회 결과로 동적 변환
  - DB 마이그레이션 V200 (필요 시): `app_settings`에 `setup_completed` / `cloud_folder_path` 행 추가 시드
  - `lib.rs` invoke_handler에 setup 커맨드 등록
  - **검증**: 단위 테스트 -- salt 파일 생성/읽기, setup 상태 전환

- ⬜ **T9: 초기 설정 마법사 프론트엔드** (1일) - skill: frontend-design
  - `src/app/setup/page.tsx` 신규 (client component)
  - 마법사 단계 (ROADMAP 9단계 중 Phase 1 핵심 단계 우선):
    1. 환영 화면 + 안내
    2. 클라우드 동기화 폴더 선택 (`selectFolder()` 호출)
    3. 비밀번호 설정 + 복구 코드 발급 (기존 `set_password` IPC 재활용)
    4. 완료 + 메인 화면 이동
  - 나머지 단계 (운영시간/학교코드/표준교습비/결제수단/백업폴더/가져오기/샘플등록)는 각 도메인 Sprint에서 점진 추가 -- "건너뛰기" 버튼으로 패스 가능
  - `src/app/page.tsx` 라우팅 분기 업데이트: `not-initialized` --> `/setup`
  - 각 단계 독립 저장, 뒤로가기 지원
  - **검증**: 마법사 전체 흐름 완주 가능 + `pnpm build` 성공

### Day 7~8: 원생 관리 화면

- ⬜ **T10: 원생 목록 화면** (1일) - skill: frontend-design
  - `src/app/students/page.tsx` 신규
  - TanStack Query로 `list_students` / `count_students` 캐싱
  - 필터 UI: 이름/학교급/학년/학교명/요일/성별/재원상태 다중 조합
  - 정렬 UI: 이름순/입교일순/학년순
  - 페이지네이션 UI (T3 백엔드 연동)
  - 200ms 이내 필터 반응
  - Ctrl+N --> 신규 원생 등록 단축키
  - 44x44px 행 클릭 영역
  - **검증**: 목록 렌더링 + 필터/정렬 동작 + 페이지네이션 동작

- ⬜ **T11: 원생 등록/수정 폼** (1일) - skill: frontend-design
  - `src/app/students/[id]/page.tsx` 또는 모달/드로어 방식
  - 필드: 일련번호(자동 채번 표시 + override), 이름, 성별, 학교급, 학년, 학교, 연락처 3종, 입교일
  - `create_student` / `update_student` IPC 호출
  - 3분 자동 임시저장 (localStorage)
  - 미저장 경고 다이얼로그 (페이지 이탈 시)
  - 퇴교 처리 버튼: 확인 다이얼로그 (보강 잔여 경고는 Phase 3에서 실구현, 지금은 UI 껍데기)
  - **검증**: 등록 --> 목록 반영 + 수정 --> 저장 + 퇴교 처리 흐름

### Day 9: 코드 테이블 관리 + 수업 스케줄 편집

- ⬜ **T12: 코드 테이블 관리 화면 (PRD SS4.12)** (0.5일) - skill: frontend-design
  - `src/app/settings/codes/page.tsx` 신규
  - 탭: 학교 / 표준교습비 / 결제수단 / 카드사
  - CRUD UI: 추가/수정/사용안함 토글/정렬순서 변경
  - `list_codes`, `create_code`, `update_code` IPC 연동
  - **검증**: 각 코드 유형 CRUD 동작

- ⬜ **T13: 수업 스케줄 편집 UI (PRD SS4.2)** (0.5일) - skill: frontend-design
  - 원생 상세 화면 내 탭 또는 섹션
  - 요일별 시작 시간 + 1회 수업 시간 입력
  - 운영 시간 내 선택만 허용 (AC-4.1.1-2, AC-4.1.1-5)
  - 주 총 수업시간 실시간 표시 + 표준 교습비 자동 매칭 표시
  - `set_schedule`, `get_schedules`, `get_weekly_hours` IPC 연동
  - **검증**: 스케줄 설정 --> 주 총 시간 표시 --> 교습비 매칭 확인

### Day 10: 키보드 단축키 + 통합 검증

- ⬜ **T14: 키보드 단축키 체계** (0.5일)
  - `src/hooks/use-keyboard-shortcuts.ts` 커스텀 훅
  - F1(도움말 -- placeholder), Ctrl+F(글로벌 검색 포커스), Ctrl+N(신규 원생), Ctrl+S(저장), Ctrl+Z(Undo -- placeholder), ESC(다이얼로그 닫기), Ctrl+P(인쇄 -- placeholder)
  - 메뉴 항목에 단축키 표기 병기
  - **검증**: 각 단축키 바인딩 동작 확인

- ⬜ **T15: 통합 검증 + self-verify** (0.5일)
  - `cargo test` 전체 통과 (R13/R14 신규 테스트 포함)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
  - `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
  - 마법사 --> 원생 등록 --> 목록 --> 수정 --> 검색 전체 흐름 수동 확인
  - Pretendard 18pt / 44x44px / WCAG AA 시각 검증
  - **검증**: 전체 CI 파이프라인 통과 가능 상태

---

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 팀 규모 | AI 페어 프로그래밍 1인 |
| 스프린트 일수 | 10 영업일 |
| 실작업 가능 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |
| Task 수 | 15건 |
| 평균 Task 소요 | 약 0.7일 (2.8시간) |
| 총 예상 소요 | 약 10.5일 (42시간) |
| 초과 여부 | 약간 초과 (~5%) -- Day 10 통합 검증에서 흡수 가능 |

> Sprint 2 velocity: 14 Task (이연 2건 제외 12건 완료) / 10일. Sprint 3는 15건으로 유사 규모. 프론트엔드 작업은 shadcn/ui + Tailwind 활용으로 생산성 높음.

---

## 의존성 및 리스크

| # | 리스크 | 영향도 | 대응 |
|----|------|--------|------|
| R16 | `tauri-plugin-dialog` 양 OS 호환성 미확인 | 중간 | T7에서 양 OS 빌드 검증. 실패 시 `rfd` crate 대안 검토 |
| R17 | Pretendard woff2 파일 크기로 번들 증가 | 낮음 | Pretendard subset (한글+영문+숫자+기호)만 포함, 전체 포함 시 ~2MB 이내 |
| R18 | salt.bin 이전 시 기존 v0.1.0/v0.2.0 사용자 마이그레이션 경로 | 중간 | 현재 사용자 데이터 없는 시점이므로 이전 호환성은 v1.0 릴리즈 전 재검토 |
| R19 | 글로벌 검색 한글 자모 분해 정확도 | 낮음 | `hangul-js` 라이브러리 검토, 자체 구현(초성 검색) 대안 |

---

## 기술 접근 방법

### 프론트엔드 아키텍처
- **상태 관리**: Zustand (세션/앱 전역) + TanStack Query (IPC 캐싱/무효화)
- **UI**: shadcn/ui + Tailwind CSS, Pretendard self-host
- **라우팅**: Next.js App Router, `output: 'export'` 정적 빌드
- **Tauri IPC**: `src/lib/tauri/index.ts` 래퍼 경유만 허용
- **SSR 가드**: `typeof window !== 'undefined'` 필수, `'use client'` 최소화

### 마법사 설계
- 단계별 독립 저장: 각 단계 완료 시 즉시 `app_settings`에 저장
- 미완료 재진입: `get_setup_status()` 조회 후 마지막 완료 단계 다음부터 시작
- Phase 1에서는 핵심 4단계(환영/폴더/비밀번호/완료)만 구현, 나머지는 "건너뛰기" placeholder

### salt 이전 (R12 해소)
- 마법사 "클라우드 폴더 선택" 단계 완료 시:
  1. `{cloud_folder}/smarthb/` 디렉토리 생성
  2. PBKDF2 salt 생성 --> `{cloud_folder}/smarthb/salt.bin` 저장
  3. 기존 Keychain에 salt가 있으면 파일로 복사 후 Keychain 항목 삭제
  4. `paths::data_root()` 가 `cloud_folder_path` 설정값을 반환하도록 변경

### 페이지네이션 패턴
- 백엔드: `LIMIT ? OFFSET ?` + 별도 `COUNT(*)` IPC
- 프론트: TanStack Query `keepPreviousData` 옵션으로 페이지 전환 시 깜빡임 방지
- 기본값: limit=50, 상한 limit_max=1000

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ 초기 설정 마법사 4단계 완주 가능 (환영 --> 폴더 선택 --> 비밀번호 --> 완료)
- ⬜ salt.bin 파일 생성 + Keychain 이전 동작 (R12 해소)
- ⬜ 원생 등록/수정/조회/퇴교 프론트엔드 전체 흐름 동작
- ⬜ 글로벌 검색바에서 원생 이름 검색 + 1클릭 이동
- ⬜ R13 PII 마스킹 적용 완료 (audit details에 원생 이름 미포함)
- ⬜ R14 페이지네이션 적용 완료 (list_students/list_codes + count IPC)
- ⬜ Pretendard 18pt 본문, 44x44px 클릭 영역, WCAG AA 명도 대비 확인
- ⬜ `cargo test` 전체 통과 (Sprint 2 97건 + Sprint 3 신규)
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md Sprint 3 상태 업데이트
- ⬜ CHANGELOG.md v0.3.0 항목 추가
- ⬜ DEPLOY.md 업데이트

---

## Playwright MCP 검증 시나리오

```
1. browser_navigate --> http://localhost:1420
2. browser_snapshot --> 초기 설정 마법사 환영 화면 확인
3. browser_click --> "시작하기" 버튼 클릭
4. browser_snapshot --> 클라우드 폴더 선택 단계 확인
5. browser_click --> 폴더 선택 (또는 기본 경로 사용)
6. browser_snapshot --> 비밀번호 설정 단계 확인
7. browser_click --> 비밀번호 입력 + "완료" 버튼
8. browser_snapshot --> 메인 화면 (앱 셸 + 사이드바) 진입 확인
9. browser_click --> 사이드바 "원생 관리" 메뉴 클릭
10. browser_snapshot --> 원생 목록 화면 렌더링 확인
11. browser_click --> "신규 원생 등록" 버튼 (또는 Ctrl+N)
12. browser_snapshot --> 등록 폼 렌더링 확인
13. browser_click --> 폼 입력 + 저장
14. browser_snapshot --> 목록에 신규 원생 표시 확인
15. browser_click --> 글로벌 검색바에 원생 이름 입력
16. browser_snapshot --> 검색 결과 표시 + 클릭 시 이동 확인
17. browser_console_messages(level: "error") --> 콘솔 에러 없음 확인
```

---

## 참고 사항

### 마이그레이션 V{NNN} 표기 통일 (A5) -- 사용자 결정 필요

Sprint 2 회고 A5에서 마이그레이션 파일명 `001__`/`008__`/`101__~105__` --> `V001__`/`V008__`/`V101__~V105__` 통일이 권고되었다. 현재 사용자 데이터가 없어 안전한 시점이다.

**옵션**:
- **(a) Sprint 3 T1 전에 hotfix로 처리**: Sprint 3 브랜치 생성 전 main에서 hotfix/migration-rename. 파일 7개 리네임 + `_sqlx_migrations` 테이블 정합성 확인. 가장 깔끔.
- **(b) Sprint 3 첫 Task로 포함**: Sprint 3 브랜치 내에서 처리. 마이그레이션 이력이 sprint3 커밋에 포함됨.
- **(c) 보류**: V200부터 `V` prefix 적용, 기존 파일은 그대로 둠. 향후 통일은 v1.0 전 정리 시 일괄 처리.

### Sprint 3 범위 옵션 -- 사용자 결정 필요

Sprint 3가 Sprint 2 backlog 전체 + ROADMAP 새 도메인을 담기에 적절한지 확인 필요:

**Option 1 (본 계획 채택안): 마법사 + R12/R13/R14 + ROADMAP Sprint 3 전체**
- Phase 1 완료를 달성하여 Phase 2 착수 가능
- Capacity 약간 초과(~5%)이나 프론트엔드 shadcn/ui 활용으로 흡수 가능
- **권장 이유**: Phase 1을 이 스프린트에서 확실히 마무리하여 프로젝트 일정 지연 방지

**Option 2: 마법사 + R12/R13/R14만 우선, ROADMAP 새 도메인 축소**
- 원생 관리 화면 일부(목록만)만 포함, 등록/수정 폼 + 코드 테이블은 Sprint 4로 이월
- 여유 있게 진행하되 Phase 1 완료가 Sprint 4로 밀림
- Phase 2 착수가 1스프린트 지연

### 기존 Sprint 2 이연 항목 중 미포함 건

- **T8 `query!()` 매크로 전환**: Sprint 2에서 무효화 결정 (현 코드에 사용 0건). 별도 backlog에 유지하되 Sprint 3 범위 미포함. 향후 sqlx 쿼리 수가 증가하면 재검토.
