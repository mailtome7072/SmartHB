# SmartHB v1.0 출시 전 전체 코드리뷰 종합 보고서

- **일자**: 2026-06-11 (sprint16 브랜치, v1.0 릴리즈 직전)
- **검토팀**: 아키텍트 · DB 전문가 · 디자이너 · UX 전문가 · 시니어 개발자(백엔드/프론트/공통 3분할) · 매니저 종합
- **전제**: 코드 무수정 읽기 전용 검토. 결과 보고 + 우선순위 제안.
- **사용자 결정 (2026-06-11)**: P0 7건 + P1 11건 → **sprint16에서 즉시 진행** / 쿼리 검증 규칙 → 문서를 현실로 개정 / 본 보고서 docs 저장.

---

## ① 종합 평가

프로젝트 건강 상태는 **양호**. 데이터 보호 장치(백업 4계층, 암호화, 잠금, 트랜잭션 설계)와 비즈니스 규칙 테스트는 전 분야 우수 평가 일치. **앱 크래시 유발 치명 버그 없음**. 단 실사용 전 차단 필요한 데이터 유실·오기록 위험 7건(P0, 합계 약 1일)이 확인됨. 전문가 간 상충 1건(공지문 폰트 대기 누락 주장)은 매니저 코드 재확인으로 **기각**(`notice-generator.ts:214-220`에 `document.fonts.ready` 대기 구현 확인).

## ② P0 — 실사용 개시 전 반드시 (✅ sprint16 진행 확정)

| # | 이슈 | 위치 | 규모 |
|---|------|------|------|
| P0-1 | 종료 시 WAL 체크포인트 부재 — 양 PC 클라우드 동기화 시 -wal 미병합으로 최근 입력 유실 가능. `setup.rs:233` 동일 패턴 재사용 | `src-tauri/src/startup.rs` exit_hook | ~2h |
| P0-2 | config.json 저장 fsync 누락 — NTFS 정전 실사고 이력 파일만 방어 누락 | `src-tauri/src/commands/setup.rs:122` write_status | ~0.5h |
| P0-3 | `toISOString().slice(0,10)` UTC 기준 — KST 오전 9시 전 "오늘"이 어제. 입교일/퇴교일 기본값 + 스케줄 삭제 기준일 | `students/edit/page.tsx:65`, `student-form.tsx:52`, `schedule-editor.tsx:49` | ~0.5h |
| P0-4 | 수납 일괄 입력 draft 무경고 소실 — 탭/월 변경·메뉴 이동·창 포커스 복귀 refetch 모두. useUnsavedChanges 미연결 + `setDrafts({})` on data change | `src/components/billing/PaymentsView.tsx:85-92` | ~1.5h |
| P0-5 | 원생 폼 임시저장 무통보 자동 적용 — 묵은 draft가 DB 최신값 덮어 표시 → 데이터 역행 위험. "이어하기" 선택지 필요 (PRD §5.7) | `src/components/students/student-form.tsx:111-122` | ~2h |
| P0-6 | 출결 Ctrl+Z가 입력필드 undo 가로채 출결 역토글 — editable 가드 없음 | `src/components/attendance/AttendanceGrid.tsx:172-185` | ~0.5h |
| P0-7 | 요일 변경 IPC 3회 순차 — 중간 실패 시 기존 요일 종료+새 요일 미등록 반쪽 상태 | `schedule-editor.tsx:133-144` (백엔드 원자 커맨드 필요) | ~2h |

## ③ P1 — 저비용·고체감 (✅ sprint16 진행 확정)

| # | 이슈 | 위치 | 규모 |
|---|------|------|------|
| P1-1 | 확인 다이얼로그 버튼 32px/14px — 위험 동작 확인이 앱 최소 크기. Button 사용처 alert-dialog 1곳 | `src/components/ui/button.tsx:7,24`, `alert-dialog.tsx:136` | ~1h |
| P1-2 | 퇴교/번복 실패 무반응 (catch 누락) | `students/edit/page.tsx:94-115,129-140` | ~0.5h |
| P1-3 | 공지문 템플릿·배경서식 삭제 확인창 없음 — confirmDialog 재사용 | `notices/page.tsx:1101-1117, 729-739` | ~0.5h |
| P1-4 | 12px 이하 글씨 핵심 화면 — 출결 일자 헤더, 보강데이 라벨(10px), 캘린더 원생 이름 등 | AttendanceGrid, ClassCalendar 등 | ~2h |
| P1-5 | text-gray-500 72곳 베이지 배경 AA 미달 → --muted-foreground 치환 | 전역 | ~1h |
| P1-6 | 삭제 확정 버튼 색 비일관 — 학사일정 삭제·퇴교가 파랑 → 빨강 통일 | `academic/page.tsx:337`, `students/edit/page.tsx:216` | ~1h |
| P1-7 | IPC 에러 `e instanceof Error` 분기로 백엔드 메시지 유실 10곳 → errMsg 헬퍼 | notices 등 (settings 계열은 올바름) | ~1.5h |
| P1-8 | 바이트 슬라이싱 panic 가능 — `from_date[..7]`, `&ym[4..5]` | `attendance.rs:1166`, `notice.rs:566` | ~0.5h |
| P1-9 | Ctrl+S 원생 폼 미동작 — onSave 연결 | `student-form.tsx` | ~0.5h |
| P1-10 | AI 가드레일 문서 드리프트 — backend.md 폐기된 청구 3단계, CLAUDE.md V305 표기, 파일명 규칙, query! 매크로 의무 → 런타임 쿼리+테스트 필수로 개정(사용자 결정) | `.claude/rules/backend.md`, `CLAUDE.md` | ~1h |
| P1-11 | 공지문 빈 상태 구 메뉴명 "청구/수납 관리" | `notices/page.tsx:1145` | ~0.2h |

## ④ P2 — 중기 리팩토링 (v1.0 이후 별도 스프린트)

| # | 이슈 | 출처 | 기대 효과 |
|---|------|------|----------|
| P2-1 | 출결 그리드 반응성 — 토글마다 전체 재조회+전체 리렌더(StudentRow memo 무력화: useCallback 의존성 `[toggleMutation]`, 인라인 핸들러) + 백엔드 원생당 7쿼리 N+1(50명=~350쿼리). ToggleResult 권위 응답을 버리고 invalidate 중 → setQueryData 셀 패치 + 배치 쿼리(~6개, calendar.rs F4 전례) | 시니어+DB | 일일 핵심 작업 반응속도 |
| P2-2 | 에러 한글화 통일 — attendance(38)/billing(26)/notice(22)/makeup(20)/expiration(10)/calendar(5) 121곳 raw sqlx 노출(PRD §6.4). `*_impl` → `Result<T, AppError>` 통일 | 아키텍트+시니어 | 50대 친화 메시지 |
| P2-3 | tauri/index.ts 1,890줄/145함수/as 단언 113 — `ipc<T>` 제네릭 헬퍼(~600줄 감소)+도메인 분할+혼입 타입 3종 src/types 이동 | 아키텍트+시니어 | 유지보수·타입 안전 |
| P2-4 | notices/page.tsx ~2,000줄/useState 25 — 분리안 7단위 제시됨. 부수: 배경서식 로드 race(cancelled 플래그), customImages 실패 자산 무한 재시도 | 아키텍트+시니어 | 안정성·드래그 성능 |
| P2-5 | AppShell → 라우트 그룹 layout — 21개 page 반복 래핑, 전환마다 IPC 3회 재발사+폴링 리셋 | 아키텍트 | 화면 전환 속도 |
| P2-6 | ConfirmDialog 공통화 — fixed 모달 복붙 12벌 + ESC 닫기 일괄 | 시니어+UX | 일관성 |
| P2-7 | text-sm 204곳 단계 상향(에러/안내→폼 라벨→테이블) + --success/--warning 토큰 | 디자이너 | 접근성 수렴 |
| P2-8 | Undo 확대(청구 금액·보강 등록) + 저장 성공 피드백 통일(토스트/배너/무 혼재) | UX | PRD §5.7 |
| P2-9 | settings/codes 저장 실패 무표시+props 파생 useState 불일치, settings/info 비원자 저장 | 시니어 | 설정 신뢰성 |
| P2-10 | 백엔드 위생 — 날짜 유틸 중복 3쌍→util_date.rs, academic fail-soft 불일치(커밋 후 소멸 전이 실패 전파), async 동기 IO 3곳(notice 배치 저장/폴더 복사/lock IPC), billing 집계 테스트 4종 부재, diagnosis 검사3 영구 오탐(스케줄 없는 재원생) | 시니어 | 잠재 버그 제거 |
| P2-11 | queryKey 20여종 문자열 산재 → query-keys.ts 중앙화 | 아키텍트 | 캐시 무효화 누락 예방 |
| P2-12 | 글로벌 검색 학교명 미지원(PRD §4.14) + 검색 UX 비일관(청구/수납만 Enter) | UX | PRD 약속 이행 |
| P2-13 | 학사코드 색 매핑 3중 정의(ClassCalendar teal vs calendar-image 핑크 vs CalendarCell) → schedule-code-colors.ts SSOT | 시니어 | 시각 일관성 |
| P2-14 | PaymentsView rows useMemo 누락, 학년 input max=6(중등 1~3 미반영), SelectionRange 타입 2중, 키보드 접근성 공백(칩 onKeyDown, 보강 셀 button 부재) 등 소규모 묶음 | 시니어 | 위생 |

## ⑤ P3 — 보류/실측 후 결정

- DB 읽기 풀 분리(MAX_CONNECTIONS=1) — 실사용 성능 실측 후
- tauri-specta 도입 — P2-3 수동 분할 효과 확인 후
- ~~query! 매크로 전환~~ — **사용자 결정: 문서 개정으로 종결** (P1-10에 포함)
- OS 다크모드 간섭 차단(button.tsx dark: variant), tailwind.config.ts 죽은 파일
- F1 도움말·메뉴 단축키 병기(V19 제거 결정 vs PRD)·청구→공지문 연결 배너·보강데이 D-3 알림 — 실사용 피드백 후
- dead_code allow 정리, ADR-001~005 상태 Accepted 갱신, greet/diagnose_sqlcipher 예시 IPC 제거
- ~~공지문 첫 생성 폰트 미적용~~ — **기각** (이미 구현 확인)

## ⑥ 분야별 핵심 요약

- **아키텍처**: 구조 건전(cipher 격리 모범, startup 병렬화·계측 우수). "한 파일 비대화 + 에러 두 갈래"가 누적 부채. ADR-구현 정합은 핵심부 일치.
- **DB**: 백업·암호화·제약 설계 우수(V108/V111 FK 재구성 모범). WAL 체크포인트(P0-1)가 유일한 출시 전 필수. SQL 인젝션 제로.
- **디자인**: 토큰 설계·44px 준수(159곳)는 좋으나 글씨 크기 전반 미달(12px 59곳/14px 204곳). 위험 확인 버튼이 앱 최소 크기인 역설(P1-1).
- **UX**: 출결 입력 모델(월 일괄+예외 토글)·컨텍스트 전달·수납 자동 채움은 높은 수준. 실수 복구 인프라가 정작 핵심 화면 미연결(P0-4·5).
- **코드품질**: 치명 버그 없음. 트랜잭션 설계(보강 매칭 race 검사 등)·비즈니스 테스트·보안 위생(zeroize, bind 일관) 우수. invoke() 직접 호출 0건.

## ⑦ 잘된 점 (전 분야 공통 평가)

1. startup 병렬화 + timing 계측 + fail-soft (PRD §5.6 견고 충족)
2. cipher feature 격리 — 5파일 9게이트에 집중, 도메인 무관
3. 핵심 트랜잭션 경로 13곳 정확(부분 실패 시나리오 명시 처리)
4. 비즈니스 규칙 단위 테스트 충실(PRD §6.5 실질 충족, 대형 모듈 40~45%가 테스트)
5. IPC 추상화 규율 100%(invoke 직접 호출 0건) + PRD/AC 번호 JSDoc 추적성
6. 한글 검색(자모 분해+IME 처리), use-unsaved-changes 훅 설계, 리뷰 피드백 누적 흔적(이슈 번호 주석)

## ⑧ 실행 로드맵 (사용자 결정 반영)

1. **sprint16 (지금)**: P0 7건 + P1 11건 → T11 통합검증(양 PC 종료→재기동 정합 + 오전 시간대 날짜 시나리오 추가) → T10 릴리즈 준비
2. **실사용 안정화 후 개선 스프린트 1차**: P2-1 출결 그리드 / P2-2 에러 한글화 / P2-8 Undo·피드백
3. **개선 스프린트 2차**: P2-3~6 구조 분할 / P2-7 글씨 전면 / P2-10 백엔드 위생
4. **상시**: P3는 실사용 피드백·실측 기반 분기별 재평가
