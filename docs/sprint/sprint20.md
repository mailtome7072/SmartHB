# Sprint Plan sprint20

## 기간
2026-07-19 ~ 2026-08-01 (2주, 예상)

## 목표
실사용 중 발견된 **청구 생성 규칙 오류**(교습기간 기반이 아닌 달력월 기준으로 대상 선정)를 근본 수정하고, **청구 삭제 기능**을 신규 추가하며, 프로덕션 DB에 이미 생성된 **오류 데이터를 안전하게 보정**한다. 아울러 실사용 중 발견된 **교습일정 인쇄 레이아웃 버그**(교습기간이 3개 월 이상 걸칠 때 달력 깨짐)와 **출결 다월 결함**(부분생성 오판으로 생성 버튼 숨김 + 그리드 표시 불일치)을 함께 수정한다. — T1(청구)·T6(인쇄)·T7(출결)은 모두 "교습기간이 여러 달에 걸침"이라는 동일 뿌리를 공유한다.

## ROADMAP 연계 기능
- Post-v1.2: 청구 도메인 버그 수정 + 기능 보강
- PRD SS4.9.1 청구 생성 규칙 — study_periods 기준 전환 (달력월 → 교습기간)
- PRD SS5.7 실수 복구 — 청구 삭제 기능 신규 (위험 동작 확인 다이얼로그)
- 교습일정 인쇄 버그 수정 — 교습기간 3개 월 이상 걸침 시 달력 레이아웃 깨짐 (프론트엔드)

## 배경 — 근본 원인 분석 (검증 완료)

`billing.rs::generate_bills_impl`(78~189행)의 청구 대상 선정 로직이 **달력월**(year_month_range: 2026-07-01 ~ 2026-07-31)을 기간 기준으로 사용하여, 교습기간(study_periods: 7/2~7/29) 종료 이후 입교한 원생(enroll_date=7/30)까지 7월 청구 대상에 포함하는 버그.

| 항목 | 현재 (버그) | 수정 후 |
|------|-------------|---------|
| 기간 기준 | `year_month_range()` — 달력월 1일~말일 | `study_periods.start_date/end_date` |
| 대상 필터 | `enroll_date <= 월말(07-31)` | `enroll_date <= 교습기간종료(07-29)` |
| 주 수업시간 | `effective_to IS NULL`만 — effective_from 무시 | `effective_to IS NULL AND effective_from <= 교습기간종료` |
| 월중입퇴교 | 달력월 경계로 판정 | 교습기간 경계로 판정 |
| 교습기간 미등록 | 무조건 생성 (달력월 사용이라 교습기간 미참조) | 차단 + 안내 메시지 |

추가 발견: 청구 삭제 기능(`delete_bill`)이 없어 잘못 생성된 청구를 정정할 수단이 없음.

## 스키마 확인
- **DB 마이그레이션 불필요** — 기존 테이블(`bills`, `payments`, `study_periods`, `student_schedules`) 구조 변경 없음. V310 최신 유지.
- **새 의존성 없음** — 기존 sqlx, chrono 등으로 구현 가능.
- `payments.bill_id REFERENCES bills(id) ON DELETE CASCADE` (V109) — bills 삭제 시 payments 자동 삭제. `PRAGMA foreign_keys=ON`은 `db.rs:117`에서 풀 초기화 시 설정됨.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint19-retrospective.md`

| 항목 ID | 항목 | 이행 상태 | 이번 스프린트 반영 |
|---------|------|-----------|-------------------|
| A117 | GradePromotionDialog catch 블록 추가 | ✅ 해결 (e72c50f) | 반영 불필요 |
| A118 | useTableSort desc tiebreak 오름차순 유지 | ✅ 해결 (e72c50f) | 반영 불필요 |
| A119 | students.rs EnrollDate tiebreak 통일 | ✅ 해결 (e72c50f) | 반영 불필요 |
| A120 | AttendanceGrid school_level fallback | ✅ 해결 (e72c50f) | 반영 불필요 |
| A114 | sync_single_date 이력 패턴 통일 | ⏸️ Post-MVP | 계속 이연 |
| A115 | cipher 스모크 테스트 수행 | ⏸️ deploy QA | 다음 배포 시 수행 |

> 직전 스프린트(Sprint 19) 회고 액션 아이템 중 미해결 항목(A114, A115)은 모두 이전부터 이연 중이며, 이번 스프린트 범위와 무관하여 계속 이연한다.

---

## 작업 목록

### T1: 청구 대상 규칙 교습기간 기준 전환 -- skill: systematic-debugging

**핵심 버그 수정. 최우선.**

수정 대상 파일: `src-tauri/src/commands/billing.rs`

1. `generate_bills_impl` 수정 (78~189행)
   - `year_month_range()` 호출을 **study_periods 조회**로 교체:
     ```sql
     SELECT start_date, end_date FROM study_periods WHERE year_month = ?
     ```
   - 조회 결과 없으면 `Err("해당 월({year_month}) 교습기간이 등록되지 않았습니다. 먼저 교습기간을 등록하세요.")` 반환하여 생성 중단
   - `period_start`, `period_end`를 study_periods의 `start_date`, `end_date`로 교체

2. 청구 대상 선정 쿼리 수정 (99~113행)
   - 기존: `WHERE s.enroll_date <= ? AND ...` (period_end = 달력월 말일)
   - 수정: `WHERE s.enroll_date <= ? AND ...` (period_end = study_periods.end_date)
   - student_schedules JOIN 조건에 `AND sch.effective_from <= ?`(period_end) 추가 — 교습기간 종료일 이후 시작 스케줄 제외

3. `compute_mid_month_flag` (632행)
   - 인터페이스 변경 불필요 (이미 period_start/period_end 파라미터 수신)
   - 호출부에서 교습기간 날짜가 전달되면 자동으로 교습기간 경계 기준 동작

4. **`get_billing_summary_impl` 동기화 (필수 — 유령 버튼 방지)** (947~986행)
   - `total_billable_students` 산정(970~986행)이 `generate_bills`와 **완전히 동일한 대상 규칙**을 써야 함. 현재는 둘 다 `year_month_range`(달력월)라 우연히 일치하지만, T1에서 generate_bills만 교습기간 기준으로 바꾸고 이 함수를 방치하면 `total_billable_students`(달력월, 7/30 입교생 포함) > `bill_count`(교습기간, 7/30 입교생 제외)가 영구 고착 → 프론트 "추가 청구 데이터 생성 (N명)" 버튼이 눌러도 사라지지 않는 **유령 버튼** 발생 (billing/page.tsx:84-87의 `ungeneratedCount = totalBillableStudents - billCount`).
   - 이 함수의 `year_month_range` + enroll/withdraw 필터도 **study_periods 기준으로 동일 교체**. (같은 로직 2곳 트랩 — harness-engineering.md 백엔드-프론트 상수쌍 원칙과 동일 취지)
   - **T1 완료 조건에 두 함수(`generate_bills_impl`·`get_billing_summary_impl`) 동시 수정을 명시.**

5. `year_month_range()` 함수 (611행)
   - `generate_bills_impl`·`get_billing_summary_impl` 양쪽을 교습기간 기준으로 대체한 뒤, 다른 사용처(list 등) 확인. 미사용이면 simplify로 제거

6. **회귀 방지 단위 테스트** (인메모리 DB, `#[cfg(test)]` 블록)
   - (a) `enroll_date`가 교습기간 종료일 이후인 원생 → 청구 **제외** 확인
   - (b) 교습기간 내 입교(`enroll_date` between start_date~end_date) → `is_mid_month=1`, `mid_month_type="enrolled"` 확인
   - (c) 교습기간 미등록 월(`study_periods` 행 없음) → 에러 반환 확인
   - (d) `effective_from`이 교습기간 종료일 이후인 스케줄 → weekly_hours 집계에서 **제외** 확인
   - (e) 정상 케이스 (교습기간 내 재원 원생) → 기존과 동일하게 청구 생성 확인
   - (f) `get_billing_summary.total_billable_students`가 generate_bills 대상과 **일치**(7/30 입교생 제외) → 생성 후 `ungeneratedCount=0` 확인 (유령 버튼 회귀 방지)

**예상 소요**: 4~5시간

---

### T2: 청구 삭제 가드 정책 ADR (정책 = B안 확정) -- skill: brainstorming

> **사용자 결정 완료(2026-07-19): B안 채택.** 아래 대안은 기록용이며, ADR은 결정을 새로 내리는 것이 아니라 **B안 채택 근거와 대안 기각 사유를 문서화**한다.

**삭제 가드 정책 (확정)**:
- **미수납(`is_paid=0`)이면 상태(`draft`/`confirmed`) 무관 삭제 허용**
- **수납완료(`is_paid=1`)는 삭제 거부** — 먼저 수납을 해제(`is_paid=0`)한 후 삭제해야 함
- `confirmed` 청구 삭제 시 확인 다이얼로그에 확정 상태임을 명시

**검토 대안 (기록용)**:
- A안: 미확정(draft) + 미수납만 삭제 허용 — 가장 보수적. **기각**: 이미 확정된 오류 청구를 정정 못 함
- **B안: 확정(confirmed)까지 + 미수납만 삭제 허용 — ✅ 채택**. 정정 유연성과 데이터 안전성(수납완료 보호) 균형
- C안: 모든 상태(수납완료 포함) 삭제 허용 — **기각**: 수금 완료 데이터까지 실수로 삭제 위험

**검토 기준**: 데이터 안전성, 운영 유연성(1인 원장 UX), 감사 추적성, 구현 복잡도

**산출물**: `docs/arch/adr-{NNN}-bill-deletion-guard.md`

**예상 소요**: 1시간 (정책 확정으로 단축)

---

### T3: 청구 삭제 백엔드

T2 ADR 결정에 따라 `delete_bill` Tauri IPC 커맨드를 구현한다.

수정/생성 대상 파일:
- `src-tauri/src/commands/billing.rs` — `delete_bill`, `delete_bill_impl` 함수 추가
- `src-tauri/src/commands/audit.rs` — `AuditEventType::BillDeleted` variant 추가
- `src-tauri/src/lib.rs` — `invoke_handler`에 `commands::billing::delete_bill` 등록

구현 요구사항:
1. **B안 가드 적용** — 미수납(`is_paid=0`)이면 `draft`/`confirmed` 무관 삭제 허용, **수납완료(`is_paid=1`)는 거부** + 사유 메시지("수납완료된 청구는 삭제할 수 없습니다. 먼저 수납을 해제하세요."). `bills LEFT JOIN payments`로 상태를 1쿼리로 확인(`update_bill` 291행 패턴 재사용)
2. `PRAGMA foreign_keys=ON` 환경에서 `DELETE FROM bills WHERE id = ?` 실행 시 `payments`(미수납 행) 자동 CASCADE 삭제 확인
3. 삭제 전 감사 로그 기록: `AuditEventType::BillDeleted`, subject = 원생명, details = `{bill_id, year_month, amount, status, had_payment}`
4. 트랜잭션 내 실행 (BEGIN IMMEDIATE)
5. **단위 테스트**:
   - (a) `draft` + 미수납 삭제 성공 + payments 행 CASCADE 삭제 확인
   - (b) `confirmed` + 미수납 삭제 성공 확인 (B안 핵심)
   - (c) 수납완료(`is_paid=1`) 삭제 요청 거부 확인
   - (d) 존재하지 않는 bill_id 삭제 시 에러
   - (e) 감사 로그 기록 확인

**예상 소요**: 3~4시간

---

### T4: 청구 삭제 프론트엔드 -- skill: frontend-design

수정/생성 대상 파일:
- `src/lib/tauri/index.ts` — `deleteBill(id: number)` 래퍼 추가
- `src/components/billing/BillingGrid.tsx` (또는 해당 컴포넌트) — 삭제 버튼 + 확인 다이얼로그
- `src/types/billing.ts` — 필요 시 타입 추가

구현 요구사항:
1. **IPC 래퍼**: `deleteBill(id: number): Promise<void>` — dev mode fallback 포함
2. **삭제 버튼**: 청구 목록의 각 행에 삭제 아이콘/버튼 배치 — **미수납 행만 활성화**. 수납완료(`is_paid=1`) 행은 비활성화 + 툴팁 "수납 해제 후 삭제 가능"
3. **확인 다이얼로그** (frontend.md 실수 복구 규칙 준수):
   - 원생명, 청구월, 금액, 상태 표시
   - `confirmed`(확정) 청구면 "확정된 청구입니다" 경고 문구 추가
   - "삭제" / "취소" 버튼 (삭제 = 위험색 빨강)
4. **삭제 후**: TanStack Query 무효화 (`bills`, `payments` 관련 쿼리 키)
5. **에러 처리**: try-catch + 사용자 친화적 한글 에러 메시지 (toast)

**예상 소요**: 3~4시간

---

### T5: 실 DB 보정 절차 문서

프로덕션 DB(클라우드 동기화 폴더 `smarthb/app.db`, SQLCipher)에 이미 생성된 잘못된 7월 청구/수납 데이터를 보정하는 안전 절차를 문서화한다.

**산출물**: `docs/sprint/sprint20/data-correction-procedure.md`

**절차 내용**:
1. **사전 백업**: 앱 종료 → 수동 백업 (app.db 사본 생성) — 자동 exit 백업 외 추가
2. **대상 식별**: 앱에서 7월 청구 목록 조회 → enroll_date가 7월 교습기간 종료일(7/29) 이후인 원생의 청구 건 식별
3. **삭제 실행**: T3/T4에서 구현한 삭제 기능으로 대상 건 삭제 — UI에서 1건씩 확인 후 삭제. **대상 청구가 수납완료(`is_paid=1`) 상태면 B안 가드로 삭제가 막히므로, 먼저 해당 청구의 수납을 해제(`is_paid=0`)한 뒤 삭제**한다
4. **검증**: 삭제 후 7월 청구 목록 재조회 → 대상 원생이 제외되었는지 확인. 8월 교습기간 생성 후 8월 청구 생성 시 해당 원생이 정상 포함되는지 확인
5. **롤백 계획**: 보정 결과 이상 시 사전 백업본으로 앱 복원 절차 안내

> 실 데이터 보정은 코드 배포(develop 머지 + 인스톨러 설치) 후 원장이 직접 수행하며, 필요 시 화면 공유로 지원한다.

**예상 소요**: 1~2시간

---

### T5-b: 퇴교 취소 → 월별 데이터 재생성 운영 워크플로우 안내 (문서)

실사용 중 발생한 관련 케이스를 문서화한다. 원장이 7월 교습기간 종료 이후 입교(7/30)한 신규 원생을 **이슈 회피 목적으로 일시 퇴교** 처리해 둔 상태에서, 8월 교습기간 등록 후 **퇴교를 취소**했을 때 원생의 청구·수납·교습일이 어떻게 되는지에 대한 운영 안내다.

**산출물**: `docs/sprint/sprint20/data-correction-procedure.md` 내 별도 섹션(또는 동일 폴더 `reinstate-regeneration-workflow.md`)

**검증된 현재 동작 (코드 근거)**:
- 퇴교 취소(`students.rs::reinstate_student_impl`, 462~530행)는 `withdraw_date=NULL` 복귀 + 퇴교 시 강제소멸된 미보강 결석 환원만 수행 — **청구·수납·교습일을 자동 생성하지 않는다.**
- 교습기간 등록(`academic.rs::create_study_period`)도 소멸 자동전이만 트리거 — **출결·청구를 자동 생성하지 않는다.**
- 정규출결(교습일) 생성 `attendance.rs::generate_impl`(77~179행)은 재호출 허용 + `INSERT OR IGNORE`(82~84행 hotfix) — 재실행 시 새로 재원 상태가 된 원생만 추가되고 기존 데이터는 무손상.
- 대상 원생 필터 `load_active_students`(282~310행): `withdraw_date IS NULL OR withdraw_date >= start_date` — **퇴교 상태(withdraw_date가 해당 월 시작일 이전)면 생성에서 제외.**
- 출결 생성은 해당 월 교습기간이 **존재 + 확정(is_confirmed=1)** 되어야 동작(`load_confirmed_period`, 217~246행). 미확정이면 "확정한 후 다시 시도" 에러.

**원장이 수행할 재생성 순서 (순서 중요)**:
1. **8월 교습기간 등록 + 확정** (`is_confirmed=1`) — 출결 생성의 전제 조건
2. **원생 퇴교 취소**(reinstate) — 반드시 생성보다 **먼저**. 퇴교 상태로 두면 8월 출결/청구 생성 대상에서 제외됨
3. **8월 출결 생성** 재실행 → 이 원생의 8월 교습일 생성 (`INSERT OR IGNORE`로 이 원생 몫만 추가, 기존 원생 무영향)
4. **8월 청구 생성** 재실행 → 이 원생의 8월 청구 생성
5. **수납 수동 기록** (`create_payment`)

**안내에 명시할 주의점**:
- ⚠️ 퇴교 취소를 생성보다 먼저 하지 않으면 이 원생만 8월 데이터가 누락됨 — 단, 취소 후 재실행하면 채워지므로 영구 유실은 아님(재생성 안전).
- 8월 청구는 T1 수정 여부와 무관하게 정상(8월 날짜는 8월 교습기간에 정확히 포함) — T1 수정이 필요한 것은 **7월 과청구**뿐이다.
- 7월 교습일은 원래 0건(입교 7/30 > 7월 교습기간 종료 7/29). 7월 과청구(있다면)는 T3/T4 삭제 기능(T5 절차)으로 정정한다.

**예상 소요**: 0.5~1시간 (문서 작성만, 코드 변경 없음)

---

### T6: 교습일정 인쇄 — 교습기간이 3개 월 이상 걸칠 때 달력 레이아웃 깨짐 수정 -- skill: frontend-design

> **실사용 발견 버그 (검증 완료).** 청구 도메인과 무관하나 사용자 요청으로 Sprint 20에 포함. 프론트엔드 전용, DB/백엔드 변경 없음.

**증상**: 일정관리 → '교습일정 인쇄'에서 교습기간이 3개 월에 걸치면(예: 7/30~9/4 → 7·8·9월 달력 3개) 인쇄 달력이 깨져 보임.

**근본 원인 (검증 완료)**: `src/lib/academic-print-html.ts`
- 인쇄 진입점 `src/app/academic/print/page.tsx`(61행)가 단일 `study_period`(start_date~end_date)를 전달 → `buildAcademicPrintHtml`(159행)이 start~end를 **월 단위로 펼침**(163~174행). 7/30~9/4면 `months = [7월, 8월, 9월]` 3개.
- STYLE의 `.print-root { height: 100vh }`(94~100행) + 각 `.print-month { flex: 1 1 0 }`(102~103행)이 **전체를 항상 A4 1페이지에 눌러 담는** 구조. CSS 주석(92~93행)이 명시하듯 **"두 달"까지만 상정** → 3개 달력이 1페이지 높이의 1/3씩으로 축소되어 셀/글자가 깨짐.
- `@media print { .print-month { page-break-inside: avoid } }`(154행)만 있고 월 간 페이지 분할(멀티페이지) 로직은 없음.

**구현 결과 (2026-07-19, 사용자 피드백 반영)**: 3개월 걸침은 대부분 "이전달 말주 + 주 월 + 다음달 첫주" 형태라, **교습기간의 주(主) 월(`year_month`) 달력 한 장**에 표기하도록 결정. 주 월 달력 그리드(앞뒤 이웃 달 칸 포함)에 교습기간 전체가 들어오면 단일 달력으로 렌더하고, 앞뒤 며칠(7/30·31, 9/1·2)은 이웃 달 칸에 교습일로 강조 표기(회색 처리 해제·교습영역 외곽선 연결). 그리드 범위를 벗어나는 드문 대규모 기간만 멀티페이지 폴백.

**초기 수정 방향 (기록용 — 위 구현으로 대체됨)**:
1. `months.length >= 3`이면 **멀티페이지 분할** 도입 — 한 페이지에 최대 2개월씩 배치하고 페이지 사이 `page-break-after: always`. **1~2개월은 기존 단일 페이지 레이아웃 유지**(회귀 방지)
2. 대안: 각 `.print-month`에 읽기 가능한 **최소 높이** 보장 + `.print-root` 높이를 콘텐츠 기반으로 전환 후 자연 페이지네이션(`page-break-inside: avoid`). 단일 월이 페이지를 채우도록 `min-height` 처리
3. (부수) 인쇄 제목이 시작월만 표기(`${sm}월 교습일정`, 224행) — 3개 월 걸침 시 오해 소지. 기간 범위 표기는 유지되므로 필수는 아니나 검토

**수정 대상 파일**: `src/lib/academic-print-html.ts`(주), 필요 시 `src/app/academic/print/page.tsx`

**검증**: 교습기간 1개월/2개월/3개월/4개월 걸침 각각 인쇄 미리보기로 달력이 읽을 수 있는 크기로 나오는지 시각 확인. 1~2개월 기존 레이아웃 회귀 없음 확인.

**예상 소요**: 3~4시간

---

### T7: 출결 — 다월 교습기간에서 부분생성 오판(생성 버튼 숨김) + 그리드 표시 결함 수정 -- skill: systematic-debugging

> **실사용 발견 버그 (검증 완료).** 교습기간이 여러 달에 걸치는 경우(예: 7/30~9/2) 발생. T1(청구)·T6(인쇄)와 동일한 "교습기간 다월 걸침" 계열.
>
> **진행 결정 (2026-07-19)**: 버그 A(사용자 차단)는 이번 스프린트에서 **완료**. 버그 B(다월 그리드 표시/태깅)는 그리드 컬럼 모델 재설계 범위가 커 **후속 스프린트로 분리**(R136) — 아래 내용은 후속 착수용 분석으로 보존한다.

두 개의 하위 결함으로 구성된다.

**버그 A — 부분생성 오판으로 "출결 생성" 버튼 숨김 (사용자 차단, 최우선)**
- 증상: 단원평가(8/3~5) 등 학사일정을 만들면 sync가 **그 날짜에만** 출결을 생성 → 8/6+ 미생성인데 "출결 생성" 버튼이 사라져 나머지를 생성할 방법이 없음.
- 원인: `count_ungenerated_attendance_students_impl`(attendance.rs:506)이 **"그 달 출결 행이 0개인 학생"만** 미생성으로 카운트. `단원평가 응시일`은 `allows_regular_class=1`(V102 시드)이라 `sync_attendance_on_schedule_change`(1440행)가 시험일(8/3~5)에 출결을 INSERT → 해당 요일 수업 학생은 "행 1건 보유"로 미생성에서 제외됨. 모든 재원생이 월/화/수 수업을 가지면 `ungeneratedCount=0`. 동시에 `check_exists`=true → 프론트(attendance/page.tsx:193 `existsQuery===false || ungeneratedCount>0`) 버튼이 **완전히 숨김**.
- 수정 방향(택1/조합, systematic-debugging으로 결정):
  1. 미생성 판정을 "행 0개"가 아니라 **교습기간 기대 수업일 대비 실제 생성 여부**로 정밀화
  2. **"출결 생성/보정" 액션 상시 노출** — `generate_attendances`는 idempotent(INSERT OR IGNORE)라 언제 눌러도 안전하게 나머지를 채움 (가장 단순·견고)
  3. `count_ungenerated_attendance_students_impl`의 `ym_to_range`(달력월)도 study_periods 기준으로 정합 (T1과 동일 취지)

**버그 B — 다월 교습기간에서 그리드 표시/태깅 불일치**
- 증상: 교습기간이 달력월 밖(7/30·31, 9/1·2)까지 걸치면 그 날짜 출결이 출결관리 그리드에 표시되지 않고, 일부 셀이 충돌.
- 원인: `generate_impl`은 교습기간 전체를 **`year_month`=생성월**로 태깅하는데(9/1 행도 "2026-08"), `sync_single_date`(1512행)는 **`date[..7]`(달력월)**로 태깅 → **태깅 기준 불일치**. 그리드는 `daysOfMonth`(달력월 1~말일 고정 컬럼) + `buildAttendanceByDay`(일 DD로만 매핑, AttendanceGrid.tsx:134·150) → 7/30·31, 9/1·2 비표시 + DD 충돌(8/1↔9/1, 8/30↔7/30 등).
- 수정 방향: generate와 sync의 `year_month` 태깅 규칙 **통일**, 그리고 그리드가 달력월 고정이 아니라 **교습기간 실제 일자 범위**를 컬럼으로 그리도록 재설계. 컬럼 모델 변경이 크므로 **설계 판단 필요**(필요 시 brainstorming 서브스텝) — 범위가 커지면 버그 A만 이번 스프린트에 처리하고 버그 B는 후속으로 분리하는 것도 검토.

**수정 대상 파일**: `src-tauri/src/commands/attendance.rs`(`count_ungenerated_attendance_students_impl`, `sync_single_date`, `generate_impl` 태깅), `src/app/attendance/page.tsx`(버튼 표시), `src/components/attendance/AttendanceGrid.tsx`(컬럼 범위·매핑)

**단위/통합 테스트**:
- (a) 부분생성(시험 sync로 8/3~5만 존재) 상태에서 재생성 경로가 열려 8/6+가 채워지는지
- (b) 다월 교습기간(7/30~9/2) 생성 후 7/30·9/2 출결이 조회/표시되는지
- (c) `generate_impl`과 `sync_single_date`의 year_month 태깅이 일관되는지

**예상 소요**: 6~9시간 (버그 B 그리드 재설계 포함 시 상단). 버그 B 분리 시 3~4시간.

---

### T8: 통합 검증

1. `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (Sprint 19 기준 431건 + 신규)
2. `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
3. `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
4. `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
5. 마이그레이션 self-check: V310 최신 유지 (신규 마이그레이션 없음)
6. 수정 파일 목록과 scope.md 대조 — 30% 이상 괴리 시 re-planning

**예상 소요**: 1~2시간

---

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 팀 규모 | 1인 AI 페어 프로그래밍 |
| 스프린트 일수 | 10일 |
| 일 실작업 시간 | 4시간 |
| 총 가용 시간 | 40시간 |
| Task 수 | 8개 + T5-b(문서) |
| 예상 총 소요 | 22.5~33시간 |

Velocity 기준(Sprint 11~13: 8~10 Task/40h) 대비 Task 8개(22.5~33h)는 40h 안에 들지만 **상단이 빡빡**하다. 특히 T7 버그 B(출결 그리드 다월 재설계)는 컬럼 모델 변경이라 리스크가 크다 — 상단 초과 조짐이 보이면 **T7은 버그 A(버튼 숨김, 사용자 차단)만 이번 스프린트에 확정 처리하고 버그 B(그리드 표시)는 후속 스프린트로 분리**한다. T1·T3(비즈니스 규칙 변경)은 테스트 작성, T6은 인쇄 시각 검증에 추가 시간이 필요하다. UX 보강 예산 2~3시간 별도 확보.

---

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 방안 |
|----|--------|--------|-----------|
| R131 | CASCADE 삭제 시 PRAGMA foreign_keys=ON 미적용 — payments가 고아로 잔존 | 높음 | T3 단위 테스트에서 FK ON 환경 삭제 후 payments 0건 확인. `db.rs:117`에서 풀 초기화 시 이미 설정됨을 재확인 |
| R132 | 실 DB 보정 중 잘못된 건 삭제 — 정상 청구 소실 | 높음 | T5 절차의 사전 백업 필수화 + 1건씩 확인 후 삭제 + 삭제 전 감사 로그 기록으로 추적 가능 |
| R133 | 교습기간 미등록 월 차단이 기존 워크플로우 변경 — 원장 혼란 | 중간 | 에러 메시지에 "교습기간 등록" 안내를 명확히 표시. 기존 달력월 기준 생성은 교습기간 등록 없이 사용하던 것이므로, 전환 후 안내 필요 |
| R134 | T6 인쇄 멀티페이지 전환이 기존 1~2개월 단일 페이지 레이아웃을 회귀시킴 | 중간 | 1/2/3/4개월 걸침 각각 인쇄 미리보기 시각 검증. 1~2개월은 기존 100vh 단일 페이지 경로를 그대로 유지하고 3개월 이상에서만 분기 |
| R135 | T1이 `generate_bills`만 교습기간 기준으로 바꾸고 `get_billing_summary`를 방치 → "추가 청구 생성 (N명)" 유령 버튼 | 높음 | T1 완료 조건에 두 함수 동시 수정 + 테스트 (f) 명시. 같은 로직 2곳 트랩 |
| R136 | T7 버그 B(출결 그리드 다월 재설계)가 컬럼 모델 변경이라 범위·리스크 큼 | 중간 | 상단 초과 시 버그 A(버튼 숨김)만 확정 처리, 버그 B는 후속 스프린트로 분리 |

---

## 완료 기준 (Definition of Done)

**필수**
- ✅ 교습기간 종료일 이후 입교 원생이 해당 월 청구에서 **제외**되는 것을 단위 테스트로 확인
- ✅ 교습기간 미등록 월에 청구 생성 시도 시 **차단 + 안내 메시지** 반환 확인
- ✅ `compute_mid_month_flag`가 교습기간 경계 기준으로 동작 확인
- ✅ `delete_bill` IPC가 B안 가드대로 동작 확인 — 미수납(draft/confirmed) 삭제 성공, 수납완료(is_paid=1) 삭제 거부
- ✅ 삭제 시 payments CASCADE 삭제 동작 확인 (PRAGMA foreign_keys=ON)
- ✅ 삭제 시 감사 로그(`BillDeleted`) 기록 — 코드 구현 완료 (try_record 는 전역 pool 사용이라 인메모리 단위 검증 대상 아님, 기존 audit 패턴과 동일)
- ✅ 청구 화면에서 삭제 버튼 + 확인 다이얼로그 정상 동작 (미수납만 활성)
- ✅ 실 DB 보정 절차 문서 완성
- ✅ 퇴교 취소 → 월별 데이터 재생성 운영 워크플로우 안내 문서 완성 (T5-b)
- ✅ 교습일정 인쇄가 3개 월 이상 걸침 교습기간에서도 각 월 달력이 읽을 수 있는 크기로 출력됨 (T6) — 코드 완료, 인쇄 미리보기 시각 QA는 실기기 확인 권장
- ✅ 교습일정 인쇄 1~2개월 걸침 기존 레이아웃 회귀 없음 (T6) — 단일 페이지 경로 보존
- ✅ 출결 부분생성 상태(시험 sync로 일부만 존재)에서도 "출결 생성/보정" 경로가 열려 나머지 기간이 채워짐 (T7 버그 A)
- ✅ `get_billing_summary.total_billable_students`가 generate_bills 대상과 일치 → 생성 후 유령 "추가 생성" 버튼 없음 (T1 f)
- ⏸️ (버그 B) 다월 교습기간의 달력월 밖 출결(7/30·31, 9/1·2) 그리드 표시 — **후속 스프린트로 분리**(R136, 그리드 컬럼 재설계 범위)
- ✅ `cargo test` 전체 통과 (441 passed, 신규 테스트 포함)
- ✅ `cargo clippy --all-targets -- -D warnings` clean
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과 (tsc 는 `pnpm install` 로 plugin-process 설치 후 clean)

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 예상 산출물

| 산출물 | 경로 |
|--------|------|
| 스프린트 계획 | `docs/sprint/sprint20.md` (본 문서) |
| 삭제 가드 ADR | `docs/arch/adr-{NNN}-bill-deletion-guard.md` |
| 보정 절차 문서 | `docs/sprint/sprint20/data-correction-procedure.md` |
| 퇴교취소 재생성 워크플로우 안내 (T5-b) | `docs/sprint/sprint20/data-correction-procedure.md` 내 섹션 (또는 `reinstate-regeneration-workflow.md`) |
| 리스크 레지스터 | `docs/risk-register/2026-07-19.md` |

## 참고 사항
- `year_month_range()` 함수는 `generate_bills_impl` 외에 다른 곳에서 사용 중인지 확인 후, 미사용이면 dead code로 제거 (simplify 스킬 자동 적용)
- PRD SS4.9.7 "마감(closed)" 개념은 V111에서 폐기됨(2단계: draft/confirmed). 삭제 가드 설계 시 마감 상태를 고려할 필요 없음
- 수납 화면(`list_payment_view`)은 `bills LEFT JOIN payments`이므로, bills 삭제 시 수납 목록에서도 자동 제거됨
- 이번 스프린트에서 `list_payment_view`나 수납 관련 IPC는 수정하지 않음 — CASCADE 삭제로 정합성 자동 유지
