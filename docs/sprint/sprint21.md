# Sprint Plan sprint21

## 기간
2026-07-19 ~ 2026-08-01 (2주, 예상)

## 목표
Sprint 20에서 의도적으로 후속 분리한 **출결 다월 교습기간 그리드 표시/태깅 불일치**(R136, A123)를 근본 수정한다. 교습기간이 여러 달에 걸칠 때(예: 8월 교습기간 7/30~9/2) 출결관리 그리드가 달력월 밖 날짜를 표시하지 못하고, 일(DD) 매핑 충돌로 셀이 서로 덮어쓰이는 구조적 결함을 해결한다. 백엔드 태깅 통일 + 프론트엔드 그리드 컬럼 모델 재설계의 2축으로 구성된다.

> **릴리즈 연계**: 이번 스프린트 완료 후 Sprint 20+21을 함께 **v1.3.0으로 배포** 예정 (사용자 결정).

## ROADMAP 연계 기능
- Post-v1.2: 출결 버그 B (R136) 해소 — Sprint 20에서 분리된 후속
- PRD SS4.5 출결관리 — 교습기간 기준 그리드 표시 정합성 복원
- A123 (Sprint 20 회고) — 그리드 컬럼 모델을 달력월 고정에서 교습기간 실제 일자 범위 기반으로 재설계

## 배경 — 근본 원인 분석 (코드 추적 검증 완료)

### 증상
교습기간이 달력월 경계를 넘으면(예: 8월 교습기간 7/30~9/2) 출결관리 그리드에 두 가지 결함이 발생한다:
1. 달력월 밖 날짜(7/30, 7/31, 9/1, 9/2)가 그리드에 **표시되지 않음**.
2. 일(DD) 매핑 충돌 — 8/1과 9/1(둘 다 DD=01), 8/2와 9/2(DD=02), 8/30과 7/30(DD=30) 등이 셀을 서로 덮어씀.

### 근본 원인 1 — 태깅 불일치 (백엔드)

| 함수 | 태깅 방식 | 결과 |
|------|----------|------|
| `generate_impl` (attendance.rs) | 생성 파라미터 `ym`(=교습기간 year_month)으로 태깅 | 9/1 행도 `year_month="2026-08"` → 교습기간 기준으로 일관 |
| `sync_single_date` (attendance.rs:1525) | `let ym = &date[..7]`(달력월)로 태깅 | 9/1 행을 `year_month="2026-09"`로 태깅 → generate와 불일치 |

### 근본 원인 2 — 그리드 모델 (프론트엔드)

| 컴포넌트/함수 | 문제 |
|--------------|------|
| `daysOfMonth(yearMonth)` (AttendanceGrid.tsx:134) | 달력월 1~말일로 고정 컬럼 생성 → 달력월 밖 날짜(7/30, 9/1 등) 비표시 |
| `buildAttendanceByDay` (AttendanceGrid.tsx:150) | `eventDate.slice(8,10)`(일 DD)로만 매핑 → 8/1과 9/1이 DD="01"로 충돌 |
| `MoveAttendanceDialog` (MoveAttendanceDialog.tsx:44-45) | `new Date(year, month, 0).getDate()` — 달력월 일수로 이동 대상 일자 한정 |

### 방향 확정 — A안(그리드를 교습기간 날짜 범위로) = 유일 정합

**결정적 사실**: 출결관리 월 선택 드롭다운은 **교습기간(study_periods) 등록 월만** 옵션으로 사용한다(`src/app/attendance/page.tsx:140,170` — `listStudyPeriods` 기반). "2026-08"을 선택하면 8월 교습기간(7/30~9/2) **전체**를 봐야 한다.

| 대안 | 설명 | 판정 |
|------|------|------|
| **A안** | 그리드 컬럼을 교습기간 실제 날짜 범위(start_date~end_date)로 표시 + sync 태깅을 교습기간 ym으로 통일 | **채택** — 유일 정합 |
| B안 | 달력월 기준 태깅 → 7/30, 7/31을 7월 탭에 표시 | **불가** — 7월 교습기간이 등록돼 있지 않으면 드롭다운에 "2026-07" 옵션이 없어 그 날짜들이 어느 탭에도 뜨지 않는 고아 발생 |
| Sprint 20 인쇄 수정 | T6에서 "주 월 달력" 방식 채택 — 교습기간 전체를 주 월 그리드에 표기 | A안과 **일관** |

> A/B 판단은 코드 추적으로 확정됨 — brainstorming 재검토 불필요.

## 스키마 확인
- **DB 마이그레이션 불필요** — V310 최신 유지. year_month 컬럼 값은 런타임에 올바르게 입력되면 되므로 스키마 변경 없음.
- **새 의존성 없음** — 기존 코드로 구현 가능.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint20-retrospective.md`

| 항목 ID | 항목 | 이행 상태 | 이번 스프린트 반영 |
|---------|------|-----------|-------------------|
| A121 | delete_bill_impl BEGIN IMMEDIATE 트랜잭션 | ✅ 해결 (R137, 2026-07-19) | 반영 불필요 |
| A122 | DEPLOY.md 스테이징 검증에 "교습일정 인쇄 미리보기 확인" 항목 추가 | 📋 예정 | **T0에서 처리** — 다음 배포(v1.3.0) 전 반영 필수 |
| A123 | 출결 그리드 다월 표시 — 그리드 컬럼 모델 교습기간 기준 재설계 | 📋 예정 | **본 스프린트 핵심 목표 (T1~T3)** |
| A114 | sync_single_date 이력 패턴 통일 | ⏸️ Post-MVP | 계속 이연 — T1에서 태깅 통일만 수행, 이력 패턴은 별도 |
| A115 | cipher 스모크 테스트 수행 | ⏸️ deploy QA | v1.3.0 배포 시 수행 예정 |

---

## 데이터 정정 안내 (문서화만, 코드 변경 없음)

- `generate_impl`으로 생성된 기존 출결은 이미 교습기간 `year_month`로 태깅되어 있어 **정정 불필요**.
- `sync_single_date`로 잘못 태깅된 행이 실 DB에 존재할 수 있다(예: 학사일정 변경 시 sync가 생성한 9/1 출결이 `year_month="2026-09"`로 태깅). 해당 교습기간의 출결을 **재생성(`generate_attendances`)하면 `INSERT OR IGNORE`로 기존 행 보존 + 누락 행 채움**이므로 자동 정정된다. 단, 잘못 태깅된 기존 행의 year_month는 갱신되지 않는다.
- 잘못 태깅된 행이 실제 문제를 일으키는지는 T1 완료 후 확인한다. 문제가 있으면 보정 SQL(`UPDATE regular_attendances SET year_month = (SELECT sp.year_month FROM study_periods sp WHERE sp.start_date <= ra.event_date AND sp.end_date >= ra.event_date) WHERE ...`)을 별도 안내한다.

---

## 작업 목록

### T0: Sprint 20 회고 액션 아이템 처리

**A122**: `DEPLOY.md` 스테이징 검증에 "교습일정 인쇄 미리보기 확인(1/2/3개월 걸침)" 항목 추가.

수정 대상 파일: `DEPLOY.md`

v1.3.0 배포 전 반영 필수. Sprint 20 회고에서 인쇄 시각 QA 자동화 불가 문제로 배포 전 수동 검증 의무화가 필요하다고 판단됨.

**예상 소요**: 0.5시간

---

### T1: sync_single_date 태깅 통일 (백엔드)

**핵심 버그 수정. T2 선행 조건.**

수정 대상 파일: `src-tauri/src/commands/attendance.rs`

`sync_single_date`(1471행)의 INSERT 시 year_month 태깅을 `&date[..7]`(달력월) 대신 **날짜가 속한 study_period의 year_month**로 변경한다.

1. **교습기간 조회 추가** (sync_single_date 내부, INSERT 분기)
   - `in_period` 조회(1512~1520행)에서 이미 study_periods를 조회하여 확정 교습기간 존재를 확인하고 있다.
   - 기존 `SELECT 1`을 `SELECT year_month`로 변경하여 교습기간의 year_month를 가져온다.
   - 교습기간 중첩 없음이 `academic.rs` IPC 레벨에서 보장됨(교습기간 일자 중첩 금지 — CLAUDE.md 백엔드 규칙, 핵심 비즈니스 키).

2. **태깅 변경** (1525행)
   - 기존: `let ym = &date[..7];` → 달력월
   - 수정: study_periods 조회 결과의 `year_month` 사용 → 교습기간 ym
   - `generate_impl`과 태깅 기준 통일.

3. **회귀 방지 단위 테스트** (인메모리 DB, `#[cfg(test)]` 블록)
   - (a) 다월 교습기간(예: year_month="2026-08", start_date=7/30, end_date=9/2)에서 `sync_single_date("2026-09-01")` 호출 시 INSERT된 행의 year_month가 `"2026-08"`(교습기간 ym)인지 확인
   - (b) `sync_single_date("2026-09-01")`이 교습기간 범위 밖이면(9/3 이후) INSERT 하지 않는 것(기존 동작) 확인
   - (c) 단일월 교습기간(start_date=8/1, end_date=8/31)에서 sync가 기존과 동일하게 동작하는지 회귀 확인

**예상 소요**: 2~3시간

---

### T2: AttendanceGrid 컬럼 모델 재설계 (프론트엔드) -- skill: frontend-design

**핵심 그리드 재설계. 본 스프린트 최대 Task.**

수정 대상 파일:
- `src/components/attendance/AttendanceGrid.tsx` — 컬럼 모델 + 매핑 로직
- `src/app/attendance/page.tsx` — 교습기간 범위 데이터를 그리드에 전달

#### 변경 1: 컬럼 생성 — `daysOfMonth` -> 교습기간 날짜 범위

기존 `daysOfMonth(yearMonth)` (134~139행)은 달력월 1~말일 고정 컬럼을 생성한다.

교습기간의 `start_date`~`end_date` 전체 날짜를 컬럼으로 생성하는 함수로 교체한다:
```typescript
// 예: start_date="2026-07-30", end_date="2026-09-02"
// → ["2026-07-30", "2026-07-31", "2026-08-01", ..., "2026-09-02"]
function periodDates(startDate: string, endDate: string): string[]
```

페이지(`attendance/page.tsx`)가 이미 `listStudyPeriods` 데이터를 보유하고 있으므로(140행), 선택된 yearMonth에 해당하는 교습기간의 `start_date`/`end_date`를 `AttendanceGrid` props로 전달한다. **백엔드 IPC 추가 불필요.**

단일월 교습기간(예: 8/1~8/31)이면 기존과 동일한 컬럼 수가 생성되어 회귀 없음.

#### 변경 2: 날짜 매핑 — DD 추출 -> 전체 ISO 날짜

기존 `buildAttendanceByDay`(150~160행)가 `eventDate.slice(8,10)`(DD)로만 매핑하여 충돌이 발생한다.

매핑 키를 전체 ISO 날짜(`YYYY-MM-DD`)로 변경한다:
```typescript
function buildAttendanceByDate(
  attendances: AttendanceCell[],
): Map<string, AttendanceCell> {
  const map = new Map<string, AttendanceCell>()
  for (const a of attendances) {
    map.set(a.eventDate, a)  // 전체 날짜로 매핑 — DD 충돌 해소
  }
  return map
}
```

보강 셀 매핑(530행 `m.eventDate.slice(8,10)`)도 동일하게 전체 날짜로 변경한다.

#### 변경 3: 컬럼 헤더 — 월 경계 날짜에 월 표기

달력월 밖 날짜는 헤더에 월을 표기하여 구분한다:
- 주 월 날짜: `1`, `2`, ..., `31` (기존과 동일)
- 이전/다음 달 날짜: `7/30`, `7/31`, `9/1`, `9/2` (월/일 표기)

요일 라벨(`weekdayLabel`)도 전체 ISO 날짜 기반으로 요일을 계산하도록 변경한다(기존 `yearMonth + day` 조합 대신).

#### 변경 4: 보강 가능일 판정

`isMakeupEligible`(96~109행)이 `new Date(year, month - 1, day).getDay()`로 요일을 계산한다. 전체 ISO 날짜 기반으로 변경하면 자동으로 정확한 요일이 산출된다.

#### 변경 5: 교습기간 범위가 없는 경우 폴백

`listStudyPeriods`에서 선택된 yearMonth에 해당하는 교습기간이 없거나 start_date/end_date가 null인 경우, 기존 `daysOfMonth`(달력월 고정)로 폴백하여 안전하게 동작한다.

**예상 소요**: 4~6시간

---

### T3: MoveAttendanceDialog 교습기간 범위 대응 (프론트엔드)

수정 대상 파일: `src/components/attendance/MoveAttendanceDialog.tsx`

`MoveAttendanceDialog`(출결 이동 다이얼로그, 205행)도 달력월 가정을 사용한다:
- 44행: `const lastDay = new Date(year, month, 0).getDate()` — 달력월 말일
- 45행: `const firstDow = new Date(year, month - 1, 1).getDay()` — 달력월 1일 요일
- 70행: `dateStr(day)` — `${yearMonth}-DD` 형식으로 달력월 날짜 생성

T2와 동일한 원칙으로 교습기간 `start_date`~`end_date` 범위 기준으로 변경한다:
1. Props에 `periodStartDate`/`periodEndDate`를 추가 (AttendanceGrid가 이미 보유한 데이터를 전달)
2. 달력 그리드를 교습기간 날짜 범위로 생성
3. 월 경계 날짜에 월 표기
4. `dateStr` 함수를 전체 ISO 날짜 배열에서 인덱스 참조로 변경

단일월 교습기간이면 기존과 동일한 달력 표시 — 회귀 없음.

**예상 소요**: 2~3시간

---

### T4: 통합 검증

1. `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (Sprint 20 기준 441건 + T1 신규)
2. `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
3. `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
4. `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
5. 마이그레이션 self-check: V310 최신 유지 (신규 마이그레이션 없음)
6. **시각 검증** (필수 — 그리드 UI 변경):
   - (a) **단일월 교습기간** (예: 8/1~8/31): 기존과 동일한 그리드 표시, 출결 토글/보강 등록 정상 → **회귀 없음** 확인
   - (b) **다월 교습기간** (예: 7/30~9/2): 7/30, 7/31, 9/1, 9/2 모든 날짜 그리드에 표시 + 출결 토글 정상 + 보강 등록 정상 + 월 경계 헤더 표기 확인
   - (c) **출결 이동** (MoveAttendanceDialog): 다월 교습기간에서 달력월 밖 날짜로 이동 가능 확인
   - (d) **인쇄 미리보기 확인** (A122 반영): 교습일정 인쇄 1/2/3개월 걸침 정상 출력 확인
7. 수정 파일 목록과 scope.md 대조 — 30% 이상 괴리 시 re-planning

**예상 소요**: 2~3시간

---

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 팀 규모 | 1인 AI 페어 프로그래밍 |
| 스프린트 일수 | 10일 |
| 일 실작업 시간 | 4시간 |
| 총 가용 시간 | 40시간 |
| 시각 검증 버퍼 | 6시간 (velocity-reference 반영) |
| 실가용 시간 | 34시간 |
| Task 수 | 5개 (T0~T4) |
| 예상 총 소요 | 10.5~15.5시간 |

Velocity 기준(Sprint 11~13: 8~10 Task/40h) 대비 **소규모 집중 스프린트**. Task 5개, 예상 15.5h 상단으로 34h 가용 시간 안에 충분히 들어온다. 그리드 컬럼 모델 변경(T2)이 가장 큰 리스크이나, 백엔드 IPC 추가 없이 프론트엔드 로직 변경만으로 구현 가능하여 상단도 관리 가능하다.

UX 보강 예산(2~3h)은 시각 검증 후 발생할 수 있는 미세 조정에 할당한다.

---

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 방안 |
|----|--------|--------|-----------|
| R136 | 다월 교습기간 그리드 표시/태깅 불일치 (Sprint 20에서 분리) | 높음 | **본 스프린트에서 근본 해결** — T1(태깅 통일) + T2(그리드 재설계) |
| R138 | 그리드 컬럼 모델 변경이 단일월 교습기간 UX 회귀 유발 | 중간 | T4 시각 검증 (a)에서 단일월 교습기간 회귀 없음을 명시적으로 확인. T2에 폴백 로직 포함. 단일월이면 `periodDates`가 기존 `daysOfMonth`와 동일한 컬럼 수 생성 |

---

## 완료 기준 (Definition of Done)

**필수**
- ✅ `sync_single_date`가 교습기간 `year_month`로 태깅 — `generate_impl`과 통일 (T1)
- ✅ 다월 교습기간(7/30~9/2) 태깅 단위 테스트 통과 (T1 a~c)
- ✅ 출결 그리드가 교습기간 날짜 범위(start_date~end_date) 전체를 컬럼으로 표시 (T2)
- ✅ 그리드 매핑이 전체 ISO 날짜 기반 — DD 충돌 없음 (T2)
- ✅ 월 경계 날짜 헤더에 월 표기(7/30, 9/1 등) (T2)
- ✅ MoveAttendanceDialog가 출발일 달력월 기준으로 이동 대상 표시 (T3 — 백엔드 동월 한정과 정합, scope 발견이슈 1 참조)
- ✅ **교습월 기준 확정** — 교습기간 소속 = 교습월(9/1·9/2도 8월 교습기간이면 8월). 사용자 2026-07-19 확정
- 🔄 **단일월 교습기간 회귀 없음** (T4 a) — 자동검증 통과, **실기기 시각 확인은 배포 전 수동 QA**
- 🔄 **다월 교습기간** 전 일자 표시·토글·보강 (T4 b) — 로직 완료, **실기기 시각 확인은 배포 전 수동 QA**
- ✅ DEPLOY.md에 인쇄 미리보기 확인 항목 추가 (T0, A122)
- ✅ `cargo test` 전체 통과 (444건, T1 신규 3건 포함)
- ✅ `cargo clippy --all-targets -- -D warnings` clean
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 예상 산출물

| 산출물 | 경로 |
|--------|------|
| 스프린트 계획 | `docs/sprint/sprint21.md` (본 문서) |
| 리스크 레지스터 | `docs/risk-register/2026-07-19.md` (R138 추가) |

## 참고 사항
- `generate_impl`으로 생성된 기존 출결은 교습기간 ym 태깅이라 정정 불필요. `sync_single_date`로 잘못 태깅된 행은 해당 교습기간 출결 재생성으로 정정 가능(선택적).
- `AttendanceGrid` props에 교습기간 범위를 추가할 때, 페이지가 이미 `listStudyPeriods`를 TanStack Query로 보유하고 있으므로 새로운 백엔드 IPC 호출 없이 데이터 전달만으로 구현한다.
- 보강 셀 매핑(530행 `m.eventDate.slice(8,10)`)도 T2에서 함께 전체 날짜 매핑으로 변경한다 — 누락 시 보강 표시 충돌 잔존.
- 이번 스프린트 완료 후 v1.3.0 배포 시 A115(cipher 스모크 테스트)를 deploy QA에서 수행한다.
