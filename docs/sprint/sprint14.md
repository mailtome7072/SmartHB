# Sprint Plan sprint14

## 기간
2026-06-02 ~ 2026-06-15 (2주, 예상)

## 목표
모든 도메인 데이터(원생/출결/보강/청구/수납/일정)를 대시보드에 집계하고, 데이터 자가 진단과 내보내기를 완성하여 원장이 운영 상황을 한눈에 파악하고 데이터를 관리할 수 있게 한다. Sprint 13 carry-over를 해소한다.

## ROADMAP 연계 기능
- PRD §4.11 대시보드 (6개 위젯 + 5종 알림)
- PRD §6.6 데이터 자가 진단 (7종 검사 + 자동/수동 실행 + 12개월 이력)
- PRD §4.13.2 데이터 내보내기 (원생/출결/청구-수납 CSV/Excel)
- PRD §5.4 복원 리허설 모드 (설정 > 백업 관리 > 리허설)

## Capacity 분석

| 항목 | 값 |
|------|-----|
| 팀 인원 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10일 |
| 실작업 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |

| 영역 | 예상 소요 | 비고 |
|------|----------|------|
| T0 carry-over | 3h | A91/A93 + R93/R94 문서 정합 |
| T1 자가 진단 백엔드 | 5h | V303 마이그레이션 + IPC 7종 + 단위 테스트 |
| T2 자가 진단 프론트 | 3h | 설정 메뉴 UI + 진단 결과 뷰 |
| T3 대시보드 집계 IPC | 6h | 위젯별 전용 IPC 5종 + 알림 IPC |
| T4 대시보드 위젯 UI | 8h | 6개 위젯 + 알림 영역 + 라우트 활성화 |
| T5 내보내기 백엔드 | 3h | IPC 3종 (원생/출결/청구-수납) |
| T6 내보내기 프론트 | 3h | 설정 > 데이터 관리 메뉴 + 다이얼로그 |
| T7 복원 리허설 | 4h | 백엔드 IPC + 설정 UI |
| T8 통합 검증 | 3h | cargo test + clippy + lint + tsc + build |
| **합계** | **38h** | 가용 40h 이내 (여유 2h) |

> **이연 판단**: 내보내기 비밀번호 보호 옵션(AC-4.13-4, Excel 암호화)은 구현 복잡도가 Sprint 15로 이연 적합. 이번 스프린트는 CSV 기본 내보내기에 집중하고, 비밀번호 보호 Excel은 Sprint 15에서 구현한다. 복원 리허설은 기존 백업 모듈(`backup.rs` + `integrity.rs`)을 활용하므로 Capacity 내 수용 가능.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint13-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 |
|---------|------|-------------------|
| A90 | 계획 수립 시 carry-over 항목 코드 현황 먼저 확인 | **적용 완료** — T0 배치 전 A91/A93의 현재 코드 상태를 sprint-planner가 직접 확인하여 stale 여부 판별 완료. A91은 주석/ADR 수정이 필요한 상태 확인. A93은 `/lock` SplashScreen 이중 표시가 여전히 존재 확인 |
| A91 | cipher off 동작 명시 주석 + ADR-008 수정 | T0에서 처리 |
| A92 | 마이그레이션 현황 표기 갱신 절차화 | T8 scope.md 체크리스트에 "마이그레이션 추가 시 CLAUDE.md 현황 갱신" 포함 |
| A93 | `/lock` SplashScreen 이중 표시 개선 | T0에서 처리 (Low, 대시보드 UI 작업과 함께) |
| A89 | 공지문 페이지 분리 검토 | 이번 스프린트 범위 외 — Sprint 15 이연 (기능 변경 없음) |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-06-02.md`

| 리스크 ID | 설명 | 반영 방법 |
|-----------|------|----------|
| R93 | `auto_unlock_with_keychain` cipher-off 동작 vs ADR-008 문구 불일치 | T0에서 주석+ADR 수정으로 해소 |
| R94 | CLAUDE.md/ROADMAP.md 마이그레이션 현황 불일치 | T8에서 V303 추가 후 CLAUDE.md 갱신 포함 |

---

## 작업 목록

### T0: Sprint 13 carry-over 해소 (3h)

- ⬜ **A91**: `src-tauri/src/startup.rs` `AuthStep::Keychain` 분기에 cipher off 동작 명시 주석 추가. `docs/arch/adr-008-optional-pin-gate.md` 구현 메모의 "stub/즉시성공" 기술을 실제 동작(keyring 시도 + 프론트 개발 모드 예외 차단)으로 수정
- ⬜ **A93**: `/lock` SplashScreen 이중 표시("잠금 상태 확인 중" -> "자동 로그인 중") 단일 로딩 상태로 통합. `src/app/lock/page.tsx` 수정
- ⬜ **R93/R94 문서 정합**: CLAUDE.md 마이그레이션 현황 V302 확인 (sprint-review에서 이미 수정 완료 여부 검증). `menu-config.ts` 대시보드 `disabledHint` 텍스트 "Phase 5 에서 제공" 부정확 — T4에서 대시보드 활성화 시 자연 해소

### T1: 데이터 자가 진단 백엔드 (5h) · skill: systematic-debugging

- ✅ **V303 마이그레이션** (`303__create_diagnosis_history.sql`): `diagnosis_history` 테이블 생성
  ```sql
  CREATE TABLE diagnosis_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_date TEXT NOT NULL,           -- ISO 8601 (YYYY-MM-DD)
    run_type TEXT NOT NULL CHECK(run_type IN ('auto', 'manual')),
    total_checks INTEGER NOT NULL,
    issues_found INTEGER NOT NULL,
    details TEXT NOT NULL,            -- JSON: [{check_id, severity, message, target_table, target_id}]
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
  );
  CREATE INDEX idx_diagnosis_history_run_date ON diagnosis_history(run_date);
  ```
  - 12개월 초과 이력 자동 정리 로직은 IPC에서 `DELETE WHERE run_date < date('now', '-12 months')` 실행
- ⬜ **`diagnosis.rs` 신규 모듈** — IPC 커맨드:
  - `run_diagnosis(run_type: String) -> DiagnosisResult` — 7종 검사 일괄 실행 + 이력 저장 + 12개월 초과 자동 정리
  - `get_diagnosis_history(limit: i64) -> Vec<DiagnosisHistoryRow>` — 이력 조회
  - `get_latest_diagnosis() -> Option<DiagnosisHistoryRow>` — 대시보드 알림용 최신 결과
  - `check_auto_diagnosis_needed() -> bool` — 매월 1일 자동 실행 판단 (당월 auto 기록 존재 여부)
- ⬜ **검사 항목 7종 구현** (PRD §6.6.1):
  1. 보강필요시간 음수/이상값 보유 원생 — `regular_attendances` 집계
  2. 재원중 원생 당월 출결 미생성 — `students` LEFT JOIN `regular_attendances`
  3. 재원중 원생 당월 청구 미생성 — `students` LEFT JOIN `bills`
  4. 수업 스케줄 vs 출결 불일치 — `student_schedules` vs `regular_attendances` 요일 비교
  5. 결석 소멸기한 미설정 — `regular_attendances` WHERE status='absent' AND `makeup_deadline` IS NULL
  6. 고아 보강 데이터 — `makeup_attendances` LEFT JOIN 매칭 결석
  7. 수납 정합성 — `payments.is_paid=1` AND (payment_method_id IS NULL OR (카드 계열인데 card_company_id IS NULL))
- ⬜ **단위 테스트**: 검사 항목별 최소 1건 (정상/이상 케이스) = 14건 이상
- ⬜ `commands/mod.rs` + `lib.rs` 커맨드 등록

### T2: 데이터 자가 진단 프론트엔드 (3h) · skill: frontend-design

- ⬜ **TypeScript IPC 래퍼 4종** + `src/types/diagnosis.ts` 도메인 타입
- ⬜ **설정 메뉴 > "데이터 자가 진단" 섹션** — `/settings` 페이지 하위
  - "자가 진단 실행" 버튼 (수동 실행, AC-6.6-2)
  - 최근 12개월 진단 이력 목록 (날짜 / 유형 / 발견 건수)
  - 진단 결과 상세 뷰 — 이상 항목별 설명 + 해결 가이드 텍스트 + 해당 화면 이동 링크 (AC-6.6-3)
- ⬜ **앱 시작 시 자동 진단 트리거**: `/lock` 또는 `/` 진입 시 `check_auto_diagnosis_needed()` 호출 → true이면 백그라운드 `run_diagnosis('auto')` 실행 (AC-6.6-1)

### T3: 대시보드 집계 IPC (6h)

- ⬜ **`dashboard.rs` 신규 모듈** — IPC 커맨드:
  - `get_academy_overview() -> AcademyOverview` — Feature 4.11.1 교습소 현황 (재원 총원, 분기별 입퇴교, 학년별/성별/학교별 비율)
  - `get_today_schedule() -> TodaySchedule` — Feature 4.11.2 당일 수업 (시간대별 인원+명단)
  - `get_monthly_summary(year_month: String) -> MonthlySummary` — Feature 4.11.3 월 핵심 요약 (출결 진행률, 청구/입금/미납, 당월 입퇴교)
  - `get_dashboard_alerts() -> Vec<DashboardAlert>` — Feature 4.11.4 알림 5종 (출결 미입력/보강 소멸 임박/미확정 청구/학사 미수립/자가 진단 이상)
  - `get_attendance_progress(year_month: String) -> AttendanceProgress` — Feature 4.11.5 출결 진행률 상세 (미입력 일자 목록)
  - `save_dashboard_memo(content: String)` / `get_dashboard_memo() -> Option<String>` — Feature 4.11.6 메모 위젯 (`app_settings` JSON 활용)
- ⬜ **알림 트리거 조건 구현** (PRD §4.11.4):
  - 출결 미입력: 오늘/이번 주 수업일 중 출결 미기록 존재 → 빨강
  - 보강 소멸 임박: 소멸기한이 당월 교습기간 종료일 기준 7일 이내 → 주황
  - 미확정 청구: `bills.status='draft'` 건수 → 주황
  - 학사 미수립: 현재 월 25일 이후 + 다음 달 교습기간 미등록 → 주황/빨강 (AC-4.11-5)
  - 자가 진단 이상: 최신 진단 결과 `issues_found > 0` → 주황
- ⬜ **단위 테스트**: IPC별 최소 1건 정상/빈 데이터 = 12건 이상
- ⬜ `commands/mod.rs` + `lib.rs` 커맨드 등록

### T4: 대시보드 위젯 UI (8h) · skill: frontend-design

- ⬜ **TypeScript IPC 래퍼 8종** + `src/types/dashboard.ts` 도메인 타입
- ⬜ **`/` 라우트 활성화** — `src/app/page.tsx` 현재 리다이렉트를 대시보드 컴포넌트로 교체
  - `menu-config.ts` 대시보드 항목에서 `disabledHint` 제거 (F3 해소)
- ⬜ **알림 영역** (상단, PRD §4.11.4):
  - 5종 알림 카드 가로 배치 (우선순위별 색상: 빨강/주황/파랑)
  - 클릭 시 해당 화면 1클릭 이동 (AC-4.11-4)
  - 처리 완료 시 자동 해제 (TanStack Query 무효화)
- ⬜ **교습소 현황 위젯** (Feature 4.11.1):
  - 재원 총원 대형 숫자 + 학년별/성별 비율 미니 도넛/바 차트
  - 분기별 입퇴교 추이 (최근 4분기)
- ⬜ **당일 수업 정보 위젯** (Feature 4.11.2):
  - 시간대별 원생 명단 타임라인 뷰
- ⬜ **월 핵심 요약 위젯** (Feature 4.11.3):
  - 출결 진행률 프로그레스 바 + 청구/입금/미납 금액 카드
  - 당월 신규 입교/퇴교 숫자
- ~~**출결 입력 진행률 위젯** (Feature 4.11.5)~~: **제거** — 출결 생성 시 전체 수업일이 `present` 기본값으로 일괄 INSERT되므로 "미입력" 상태가 존재하지 않아 항상 100%로 무의미. 자가진단 검사 2번이 관련 이상을 커버.
- ⬜ **메모 위젯** (Feature 4.11.6):
  - 포스트잇 스타일 자유 메모. 자동 저장 (디바운스 1초)
  - `app_settings` JSON에 `dashboard_memo` 키로 저장
- ⬜ **차트 라이브러리 결정**: 학년별 분포, 분기별 추이에 간단한 차트 필요
  - **Recharts 채택** (ADR 불요 — 이미 PRD §4.8.2에서 후보로 명시, Phase 5 취소로 학습보고서 차트 불필요해졌으나 대시보드 차트에 Recharts가 최적: React 네이티브, 번들 ~45KB gzip, Tailwind 호환)
  - 대안: shadcn/ui의 내장 차트(Recharts 래퍼) 우선 검토 — 추가 설치 없이 사용 가능 여부 확인. 불가 시 `recharts` 직접 설치
  - **신규 의존성**: `recharts` (pnpm add recharts, 사전 승인 필요)
- ⬜ **위젯 로드 성능**: 모든 위젯 5초 이내 로드 (AC-4.11-1). TanStack Query `staleTime` 활용, 쿼리 병렬 실행

### T5: 데이터 내보내기 백엔드 (3h)

- ✅ **`export.rs` 신규 모듈** — IPC 커맨드:
  - `export_students(file_path: String) -> ExportResult` — 원생 명단 엑셀 (이름/성별/학년/학교/생년월일/입교일/퇴교일/수업시간/교습비, 일련번호 오름차순)
  - `export_attendances(year_month: Option<String>, file_path: String) -> ExportResult` — 출결 데이터 엑셀 (정규+보강 UNION, 구분 컬럼 포함)
  - `export_billing(year_month: Option<String>, file_path: String) -> ExportResult` — 청구-수납 엑셀 (청구상태 컬럼 포함, 금전 천단위 콤마+우측정렬)
- ✅ 엑셀(.xlsx) 생성 — rust_xlsxwriter 0.95 (순수 Rust, Win/Mac 안전). autofit 컬럼너비, 금전 우측정렬, 수업시간 시간 단위 통일
- ✅ 기간 선택: 단월(`year_month`) 또는 전체(`year_month: Option<String>` None=전체)
- ✅ 파일 저장 경로는 프론트엔드가 Tauri Dialog로 사전 취득 후 전달 (AC-4.13-3)
- ✅ **단위 테스트**: IPC별 정상/빈 데이터 = 9건 (정렬순서/금전셀/시간환산/파일생성 포함)
- ✅ `commands/mod.rs` + `lib.rs` 커맨드 등록

> **변경**: CSV 기본 내보내기→엑셀(.xlsx) 전환(사용자 요청 2026-06-05). 비밀번호 보호 옵션(AC-4.13-4)은 Sprint 15로 이연.

### T6: 데이터 내보내기 프론트엔드 (3h) · skill: frontend-design

- ✅ **TypeScript IPC 래퍼 3종** + `src/types/export.ts` 도메인 타입
- ✅ **설정 메뉴 > "데이터 관리" 섹션** — `/settings/data` 신규 라우트
  - 내보내기 대상 선택 (원생/출결/청구-수납)
  - 기간 선택 드롭다운 (단월/전체, 출결·청구만)
  - "내보내기" 버튼 → `showXlsxSaveDialog` → IPC 호출 → 결과 배너
- ✅ **Tauri Dialog `save` 다이얼로그**: `@tauri-apps/plugin-dialog` 기존 플러그인 재사용 (신규 의존성 불필요)
  - 기본 파일명: `{대상}_{기간}.xlsx` (예: `원생명단_전체.xlsx`, `출결_2026-06.xlsx`)

### T7: 복원 리허설 모드 (4h)

- ⬜ **`recovery.rs` 또는 `backup.rs` 확장** — IPC 커맨드:
  - `run_backup_rehearsal(backup_path: String) -> RehearsalResult` — PRD §5.4 복원 리허설
    1. 백업 파일을 임시 디렉토리에 복사
    2. `PRAGMA integrity_check` 실행
    3. 주요 테이블 행 수 카운트 (검증된 데이터 건수)
    4. 결과 반환 (성공/실패, 건수, 손상 항목)
    5. 임시 파일 자동 삭제
  - `list_backup_files() -> Vec<BackupFileInfo>` — 백업 목록 (시점/크기/계층 표시) — 기존 `backup.rs`에 이미 유사 로직 존재 시 확장
- ⬜ **설정 > 백업 관리 UI 확장**:
  - 백업 목록 표시 (기존 설정 페이지에 섹션 추가)
  - "백업 복원 리허설" 버튼 → 백업 선택 → 진행 표시 → 결과 표시 (성공/실패, 검증 건수)
  - 결과는 운영 데이터에 영향 없음을 명시 안내
- ⬜ **단위 테스트**: 리허설 정상/손상 백업 2건

### T8: 통합 검증 (3h)

- ⬜ `cargo test --lib --manifest-path src-tauri/Cargo.toml` 전수 통과
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- ⬜ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ⬜ `pnpm lint` clean
- ⬜ `pnpm tsc --noEmit` clean
- ⬜ `pnpm build` static export 성공 (라우트 수 증가 확인)
- ⬜ `.sqlx/` 오프라인 캐시 갱신 + 커밋
- ⬜ CLAUDE.md 마이그레이션 현황 V303 갱신 (A92 절차 적용)
- ⬜ 대시보드 5개 위젯 + 메모 위젯 + 알림 5종 시각 검증
- ⬜ 자가 진단 수동 실행 + 결과 표시 시각 검증
- ⬜ CSV 내보내기 3종 파일 생성 + Excel 열기 검증
- ⬜ 복원 리허설 실행 + 결과 표시 시각 검증

---

## 신규 의존성

| 패키지 | 측 | 버전 | 용도 | 승인 상태 |
|--------|---|------|------|----------|
| `recharts` | 프론트엔드 | 3.8.1 | 대시보드 차트 (학년 분포, 분기별 추이, 청구추이) | 사용자 승인 완료 (2026-06-02) |
| `rust_xlsxwriter` | 백엔드(Rust) | 0.95 | 엑셀(.xlsx) 내보내기 (순수 Rust, Win/Mac 안전) | 사용자 승인 완료 (2026-06-05) |

> shadcn/ui 내장 차트 미존재 확인(T4 착수 시) → recharts 직접 설치. dynamic import(ssr:false)로 대시보드 라우트 한정 로드(R96).

## 신규 마이그레이션

| 번호 | 파일명 | 설명 | 도메인 블록 |
|------|--------|------|------------|
| V303 | `303__create_diagnosis_history.sql` | `diagnosis_history` 테이블 (자가 진단 이력) | V300~V399 도메인 확장 |
| V304 | `304__expire_withdrawn_pending_makeup.sql` | 퇴교생 미보강 결석 일괄 `makeup_expired` 백필 (버그픽스) | V300~V399 도메인 확장 |
| V305 | `305__add_birth_date_to_students.sql` | `students.birth_date` nullable 컬럼 추가 (원생 생년월일) | V300~V399 도메인 확장 |

> V302(`add_is_seeded_to_schedule_events`)에서 V303→V304→V305 도메인 확장 블록 연속.

## scope 경계 (포함/제외)

### 포함
- 대시보드 6개 위젯 + 5종 알림 (PRD §4.11 전체)
- 데이터 자가 진단 7종 + 자동/수동 실행 + 12개월 이력 (PRD §6.6)
- 데이터 내보내기 CSV 3종 (원생/출결/청구-수납) (PRD §4.13.2 부분)
- 복원 리허설 모드 (PRD §5.4)
- Sprint 13 carry-over (A91/A93/R93/R94)
- `menu-config.ts` 대시보드 `disabledHint` 제거 (F3 해소)
- V303 마이그레이션 (diagnosis_history)
- CLAUDE.md 마이그레이션 현황 갱신 (A92)

### 제외 (Sprint 15 이연)
- 데이터 가져오기(Import) — CSV/Excel 가져오기 (PRD §4.13.1)
- 내보내기 Excel(.xlsx) 형식 + 비밀번호 보호 (AC-4.13-4)
- 공지문 페이지 분리 리팩토링 (A89)
- AC-4.11-2 "새벽 자동 갱신" — 데스크톱 앱 비활성 시간 트리거 복잡도 고려, 수동 새로고침만 Sprint 14에서 구현. 자동 갱신은 Sprint 15에서 앱 포커스 복귀 트리거로 검토
- 반응형 폰트/셀 너비 (Sprint 8 backlog)
- 한글 자모 검색 (Sprint 8 backlog)
- N+1 쿼리 최적화 (`attendance.rs` F3 — Sprint 8 backlog)

---

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R95 | 대시보드 집계 쿼리 복잡도 — 6개 위젯이 각각 복수 테이블 JOIN/집계 실행. 데이터량 증가 시 5초 초과 우려 | 중간 | 위젯별 전용 IPC로 분리 + TanStack Query 병렬 + staleTime 캐싱. 50명 기준 사전 프로파일링 |
| R96 | Recharts 번들 크기 — 대시보드 전용이지만 ~45KB gzip 추가. static export 빌드 크기 증가 | 낮음 | dynamic import(`next/dynamic`)로 대시보드 라우트에서만 로드. 다른 라우트 영향 없음 |
| R97 | 자가 진단 자동 실행 타이밍 — 매월 1일 앱 시작 시 진단이 startup 시퀀스 지연 유발 가능 | 중간 | 비동기 백그라운드 실행 (startup 완료 후 별도 spawn). UI 진입을 차단하지 않음 |
| R98 | 복원 리허설에서 cipher 빌드 백업 파일 접근 — cipher off 개발 빌드에서 암호화된 백업 파일 열기 실패 가능 | 중간 | 개발 빌드(cipher off)에서는 평문 백업만 리허설 대상. cipher on 빌드에서 암호화 백업 리허설은 별도 검증 |
| R99 | CSV 한글 인코딩 — Excel에서 UTF-8 CSV 열 때 한글 깨짐 | 낮음 | BOM(0xEF 0xBB 0xBF) 접두로 Excel 자동 인식. 검증 필수 |

---

## 완료 기준 (Definition of Done)

**필수**
- ✅ 대시보드 위젯(현황/당일/월요약/청구추이/이달의 생일/메모 3슬롯/알림) 렌더링 동작 — 출결 진행률 위젯 제거(항상 100% 무의미)
- ✅ 4종 알림 조건별 표시 + 클릭 시 해당 화면 1클릭 이동 (AC-4.11-3, AC-4.11-4) — 출결 미입력 알림 제거
- ✅ 모든 위젯 5초 이내 로드 (AC-4.11-1)
- ✅ 자가 진단 수동 실행 + 7종 검사 결과 표시 + 해결 가이드 링크 (AC-6.6-3)
- ✅ 매월 1일 자동 진단 동작 — `last_auto_diagnosis` app_settings 분리 (AC-6.6-1)
- ✅ 진단 이력 12개월 보관 + 초과분 자동 정리 (AC-6.6-4)
- ✅ 엑셀(.xlsx) 내보내기 3종(원생/출결/청구-수납) 파일 생성 + OS 저장 다이얼로그 (AC-4.13-3) — CSV→엑셀 전환(rust_xlsxwriter)
- ✅ 복원 리허설 실행 + 결과 표시 (격리 환경, 운영 데이터 무영향)
- ✅ `cargo test --lib` 368 passed
- ✅ `cargo clippy -- -D warnings` clean (cipher off)
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ✅ `.sqlx/` 오프라인 캐시 — 런타임 query() 패턴이라 갱신 불필요

**프로세스 (sprint-close 에이전트가 처리)**
- ✅ ROADMAP.md Sprint 14 상태 업데이트
- ✅ CHANGELOG.md 업데이트
- ✅ DEPLOY.md 업데이트
- ✅ CLAUDE.md 마이그레이션 현황 V305 갱신 (이미 반영 완료)

---

## 예상 산출물

| 산출물 | 경로 |
|--------|------|
| 마이그레이션 V303 | `src-tauri/migrations/V303__create_diagnosis_history.sql` |
| 자가 진단 모듈 | `src-tauri/src/commands/diagnosis.rs` |
| 대시보드 집계 모듈 | `src-tauri/src/commands/dashboard.rs` |
| 내보내기 모듈 | `src-tauri/src/commands/export.rs` |
| 복원 리허설 (backup.rs 확장) | `src-tauri/src/commands/backup.rs` |
| 대시보드 페이지 | `src/app/page.tsx` (교체) |
| 대시보드 위젯 컴포넌트 | `src/components/dashboard/` |
| 진단 결과 UI | `src/app/settings/` 섹션 확장 |
| 내보내기 UI | `src/app/settings/` 섹션 확장 |
| 복원 리허설 UI | `src/app/settings/` 섹션 확장 |
| TypeScript 타입 | `src/types/diagnosis.ts`, `src/types/dashboard.ts`, `src/types/export.ts` |
| IPC 래퍼 | `src/lib/tauri/index.ts` 확장 (15종+) |

---

## 참고 사항

- **대시보드 AC-4.11-2 "새벽 자동 갱신"**: 데스크톱 앱이 새벽에 실행 중일 가능성이 낮으므로, Sprint 14에서는 사용자 수동 새로고침(F5 또는 새로고침 버튼)만 구현한다. 앱 포커스 복귀 시 자동 갱신은 Sprint 15에서 검토한다.
- **`운영 일정 알림` (PRD §4.11.4 5번째)**: "보강데이 D-3, 단원평가 응시 D-5" 중 단원평가는 Phase 5 취소로 제외. 보강데이도 Sprint 9에서 일괄 기능 폐기됨. 실질적으로 이번 스프린트에서 구현할 운영 일정 알림은 학사 일정 코드 기반 리마인더(교습기간 시작/종료 D-3 등)로 대체한다.
- **V303 마이그레이션 번호**: 300번대 도메인 확장 블록 연속 사용 (V301 schedule_codes 보정, V302 is_seeded 플래그에 이어 V303 diagnosis_history).
- **메모 위젯 저장**: 별도 테이블 대신 `app_settings` JSON 활용 (Sprint 12 `notice_layout`과 동일 패턴). 마이그레이션 추가 불필요.
