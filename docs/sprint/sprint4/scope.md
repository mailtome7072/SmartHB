---
Sprint: 4  |  Date: 2026-05-21  |  Session: #4
---

## 세션 #4 목표 — students 도메인 직렬 묶음 (T4 + T6 + T7 + T8 + T5 적용)

같은 파일군(`student-form.tsx`, `students/edit/page.tsx`, `students.rs`)을 건드리는
4 Task 를 직렬 처리. T5 utils 의 student-form 적용도 흡수.

## 사전 확인 (사실 정정)

- ✅ `students.withdraw_date` 컬럼 **이미 존재** (V101) — **DB 마이그레이션 V201 불필요**.
  사용자 이슈 #7 ("퇴교일 정보 없음") 은 백엔드 OK, 단지 프론트가 오늘 날짜를
  자동 설정하고 퇴교일 입력/표시 화면이 없는 게 진짜 문제
- ✅ `withdraw_student(id, withdraw_date)` IPC 이미 날짜 받는 시그니처
- ❌ `reinstate_student` IPC 부재 — T8 에서 신규 추가 (`UPDATE students SET withdraw_date = NULL`)
- ⚠️ `update_student` 가 serial_no 컬럼을 업데이트 — T6 에서 백엔드 가드 + 프론트 readonly
- ✅ `school_id` 컬럼 + `list_codes('schools')` IPC 이미 존재 — T4 는 프론트 Select 추가만

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | Task | 비고 |
|------|---------|------|------|
| `src/components/students/student-form.tsx` | [0회] | T4+T5+T6 | 학교 Select + phone 하이픈 + serial_no readonly (수정 모드) |
| `src/app/students/page.tsx` | [0회] | T4 | 필터에 학교 옵션 추가 |
| `src/app/students/new/page.tsx` | [0회] | T7 | submit 후 redirect → `/students/edit?id=N` |
| `src/app/students/edit/page.tsx` | [0회] | T8 | 퇴교 AlertDialog 에 날짜 입력 + 번복 버튼 + 퇴교 시 ScheduleEditor 비활성 |
| `src-tauri/src/commands/students.rs` | [0회] | T6+T8 | update_student serial_no 변경 거부 + reinstate_student IPC 신규 |
| `src-tauri/src/lib.rs` | [0회] | T8 | reinstate_student invoke_handler 등록 |
| `src/lib/tauri/index.ts` | [0회] | T8 | reinstate_student 래퍼 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- ⬜ `.github/workflows/` / `SETUP.sh` / `docs/harness-engineering/`
- ⬜ `src-tauri/migrations/` — V201 불필요 (사전 확인)
- ⬜ T9/T10 도메인 (`schedule-editor.tsx`, `settings/codes/page.tsx`) — 다른 세션

## 완료 기준 (세션 #4 DoD)

### T4 (학교명 선택란 + 필터)
- ⬜ StudentForm 에 `school_id` 필드 + listCodes('schools') 데이터 fetch
- ⬜ students/page.tsx 필터 패널에 학교 select 옵션 (선택 시 list_students filter 에 반영)

### T5 (utils 적용 — phone 자동 하이픈)
- ⬜ StudentForm phone_student/mother/father onChange 에서 formatPhone 호출

### T6 (일련번호 수정 차단)
- ⬜ StudentForm: initial 있을 때(수정 모드) serial_no input readonly
- ⬜ 백엔드 update_student: serial_no 컬럼 업데이트 SQL 에서 제외 → 기존 값 유지

### T7 (등록 후 스케줄 등록 UX)
- ⬜ students/new 의 onSubmit 성공 후 `router.push('/students/edit?id=${createdId}')`
- ⬜ 안내 메시지: "원생이 등록되었습니다. 이어서 수업 스케줄을 입력하세요"

### T8 (퇴교일 + 번복 + 가드)
- ⬜ 백엔드: reinstate_student(id) IPC 신규 — UPDATE ... SET withdraw_date = NULL
- ⬜ 프론트: 퇴교 AlertDialog 에 `<input type="date" />` 추가 (기본값 오늘)
- ⬜ 퇴교일 표시 영역에 "퇴교 번복" 버튼 + 확인 다이얼로그
- ⬜ 퇴교 상태일 때 ScheduleEditor 비활성 (또는 readonly)

### 공통
- ⬜ tsc + lint + cargo test + clippy 모두 통과
- ⬜ Task 단위 개별 커밋 (5개)

## 적용 스킬

T4/T7 일반 implementation. T6/T8 백엔드 변경 — backend.md 규칙 자동 로드.

## 이전 세션 (#3) 완료 항목

- ✅ T5 utils (`7374392`) — formatPhone, formatCurrency
- ✅ T3 상태바 (`d725747`) — IPC 연결 + 4컬럼 표시
- ✅ T10 코드 DnD + 필터 (`5e8e58a`)

## 다음 세션 진입점 (예정)

T4~T8 완료 → **T9 (스케줄 시작시간 콤보 + 수정/삭제)** — T2 운영시간 의존, 이미 가능.
이후 T11 통합 검증.

---

## Sprint 4 종료 — 최종 결과 (2026-05-21)

### Task 완료
- ✅ T1 (`a06dbd6`) — dialog 차단 해소 + shadcn AlertDialog
- ✅ T2 (`04131a9`) — 교습소 설정 메뉴 + 운영 시간
- ✅ T3 (`d725747`) — 상태바 IPC 연결
- ✅ T4/T5/T6 (`150bfd4`) — 학생 폼 학교/하이픈/일련번호 + reinstate IPC
- ✅ T5 utils (`7374392`) — formatPhone/formatCurrency
- ✅ T7/T8/T4 필터 (`76abb3d`) — 등록후 안내·퇴교날짜·번복·가드·학교필터
- ✅ T9 (`37fb823`) — 스케줄 시작시간 콤보 + 수정/삭제
- ✅ T10 (`5e8e58a`) — 코드 테이블 DnD + 활성 필터
- ✅ T11 — 통합 검증 (사용자 시각 검증 통과, 14 매트릭스 + 4 추가)

### T11 통합 검증 중 사용자 추가 보고 (4건, 모두 처리)
- ✅ `61d97fa` — #1 스케줄 폼 위치 + 1시간 단위, #2 운영시간 디폴트 19→20, #3 컬럼 헤더 정렬 + 번호 디폴트
- ✅ `49cfd15` — #4 원생 목록 주총 수업시간 + 요일 컬럼

### 자동 검증 결과
- cargo test 130 (Sprint 3 종료 시 123 → +7건: settings 6 + serial sort 1)
- cargo clippy -D warnings 0건
- tsc --noEmit EXIT 0
- next lint --max-warnings 0 OK

### 알려진 flaky (회고 carry-over)
- `paths::tests::init_from_config_ignores_empty_path` — 병렬 실행 시 OnceLock 격리 부족.
  --test-threads=1 직렬 실행 시 OK. 기존 테스트 결함이며 우리 변경 무관 — Sprint 5
  carry-over (테스트 격리 강화).

### 신규 의존성
- `@base-ui/react`, `class-variance-authority`, `clsx`, `lucide-react`, `tailwind-merge`,
  `tw-animate-css` (shadcn init 부산물, 모두 사용 중)
- `@dnd-kit/core`, `@dnd-kit/sortable`, `@dnd-kit/utilities` (T10 DnD)
- (devDependencies) `shadcn`

### Sprint DoD 달성
14개 사용자 보고 이슈 + 4개 post-T11 fix 모두 해소. sprint-close 진입 준비.
