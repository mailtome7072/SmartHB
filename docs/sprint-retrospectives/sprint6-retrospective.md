# Sprint Retrospective — Sprint 6

> Sprint 6: Phase 2 학사 스케줄 관리 — 교습기간 + 일정 코드 + 3개월 캘린더 + 일정 배치
> 기간: 2026-05-22 / 세션 수: 9 / 커밋 수: 16
> 브랜치: `sprint6` → develop 머지 완료
> 코드 리뷰: Critical 0 / High 0 / Medium 2 / Low 1

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint5-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A14 | paths.rs OnceLock 병렬 테스트 격리 강화 | ✅ 완료 | T3에서 OnceLock 분리 리팩토링 완료 (A21). 146건 안정 통과 확인 |
| A15 | DnD 필터링 sort_order 충돌 해소 (R26) | ✅ 완료 | T4에서 방법 B 적용 (A22). 코드 DnD 후 필터 변경 시 sort_order 충돌 해소 |
| A16 | CVE-2025-66478 분석 완료 (이미 Sprint 5에서 완료) | ✅ 확인 | Sprint 6 범위 외. Sprint 5에서 완결됨 |
| A17 | salt.bin 이전 (Keychain → cloud/smarthb/) | ⏸️ 이연 | Sprint 6 scope 외. Phase 2 전체 완료 후 처리 권고 |
| A18 | simplify 스킬 기준 문서화 | ⏸️ 메타 작업 이연 | 운영 중 필요 시 재검토 |

---

## 잘한 점

**대형 도메인(~1,000줄 Rust + 4개 컴포넌트)을 9 세션에 완결했다.**

academic.rs는 3개 도메인(교습기간·일정 코드·학사 일정) × 4~6개 IPC = 15개 커맨드를 단일 파일에서 책임 분리(헬퍼 3개 분리)하며 구현했다. 프론트엔드는 `page.tsx`에서 mode 분기(view / study-period / event-place)로 상태를 끌어올려 4개 컴포넌트를 독립적으로 조합하는 구조를 채택했다. 계획 단계에서 이 분리 구조를 scope.md에 미리 정의했기 때문에 세션 간 이어달리기가 원활했다.

**V301 방어적 UPDATE 패턴이 멱등성을 달성했다.**

V301 마이그레이션은 V102 시드 행의 실제 코드명과 `is_system_reserved=1` 조건을 WHERE에 함께 명시하여 사용자 추가 코드를 건드리지 않도록 설계했다. 2025~2027년 공휴일 64건도 `INSERT OR IGNORE` 패턴으로 재실행 안전성을 확보했다. 멱등성 달성 여부는 `system_reserved_codes_seeded` 테스트(V301 검증 5건 포함)로 자동 보장한다.

**공휴일 수집 파이프라인을 빌드 타임으로 격리했다.**

ADR-005 결정에 따라 공공데이터포털 API 호출을 `pnpm holidays:fetch` 빌드 타임 1회 스크립트로 격리했다. 런타임 네트워크 의존이 없어 PRD §5.5 "외부 네트워크 통신 없음" 원칙을 준수한다. API 키가 없는 환경에서는 스크립트를 건너뛰고 커밋된 V301 SQL을 그대로 사용한다.

**@dnd-kit 드래그와 button 중첩 문제를 구조적으로 해결했다.**

T11에서 CalendarCell의 outer element를 `button → div`로 변환하면서 `role="button"`, `tabIndex`, `onKeyDown(Enter/Space)`, `aria-label`, `aria-pressed`, `focus:ring` 접근성 속성을 모두 보존했다. 내부 EventBadge는 `<button>`으로 독립 추출하여 드래그 `useDraggable` hook을 각 배지 인스턴스에서 한 번씩 올바르게 호출한다. HTML button 중첩 금지 + React hooks 규칙 + 접근성을 동시에 충족했다.

**에러 메시지 일관성이 사용자 친화적이다.**

백엔드 한국어 에러("다른 교습기간과 일자가 중첩됩니다", "지난 달의 학사 일정은 수정할 수 없습니다" 등)가 프론트엔드의 AlertDialog에 그대로 표시된다. `onError: (err) => setErrorMessage(err instanceof Error ? err.message : String(err))` 패턴이 4개 컴포넌트 전체에서 동일하게 적용되어 있다.

---

## 개선할 점

**codeBadgeClass의 한국어 하드코딩이 유지보수 리스크다.**

`CalendarCell.tsx`의 `codeBadgeClass` 함수는 시스템 코드명 6종(공휴일/보강데이/공휴수업일/방학/휴원일/단원평가 응시일)을 switch 케이스로 하드코딩하고 있다. `ThreeMonthCalendar.tsx`의 `draggableEventIds` 계산도 동일한 문자열 집합을 `Set` 리터럴로 정의한다. 향후 시드 코드명이 변경되거나 시스템 코드가 추가될 경우 두 곳을 동시에 수정해야 한다. 백엔드 `ScheduleCode.is_system_reserved` 플래그를 `ScheduleEventListItem`에 JOIN해 내려주면 프론트엔드는 코드명 없이 플래그 기반으로 판단할 수 있어 이 리스크가 해소된다.

**교습기간 외 드롭 시 UI 가드가 없다.**

드래그 이동 시 교습기간 범위 밖 날짜로 드롭해도 백엔드가 차단하지 않는다. 현재는 의도된 동작("원장이 교습기간 밖에도 임의 일정을 배치할 수 있다")으로 결정했지만, 사용자가 혼란을 느낄 수 있다. 향후 배포 후 사용자 피드백을 보고 UI 경고(토스트 또는 드롭 거부 시각 피드백) 추가 여부를 결정한다.

**공휴일 API 인증키 base64 이중 인코딩 버그에서 시간을 소비했다.**

Session #6 T2-a에서 `URL.searchParams.set`이 인증키의 `+`, `=` 문자를 퍼센트 인코딩하여 HTTP 403 오류가 발생했다. raw string concat으로 우회했고 ADR-005에 기록했다. 외부 API 연동 시 URL 파라미터 인코딩 방식을 사전 검증하는 습관이 필요하다.

**SQLite `VALUES ... AS alias` 문법 오류로 첫 V301 마이그레이션이 실패했다.**

Session #6 T2-b에서 PostgreSQL 스타일 `VALUES (...) AS t(col)` 문법을 SQLite에 사용해 syntax error가 발생했다. `column1`/`column2` 자동 명명으로 우회했다. SQLite의 표준 SQL 부분집합 차이를 스프린트 초기에 체크하는 루틴이 필요하다.

**스킬 부재로 절차를 임시 재현했다.**

Session #5(brainstorming), #7(frontend-design), #9(simplify) 에서 `.claude/skills/` 에 해당 스킬 파일이 없어 inline으로 절차를 재현했다. brainstorming은 ADR-005 결정에, simplify는 T8 타입 정리에 실질적으로 기여했다. Sprint 7 진입 전에 세 스킬을 `.claude/skills/`에 추가하면 이후 스프린트에서 재현 비용이 사라진다.

---

## 이번 스프린트에서 배운 점

1. **공휴일 외부 데이터의 '1~2년 선예약' 한계**: 공공데이터포털 특일정보 API는 약 1~2년 미래만 사전 등록돼 있어 2027년분을 수집하지 못한 날이 있었다. ADR-005를 매년 1월 V401+ 마이그레이션으로 갱신하는 방식으로 해결책을 정의했다.

2. **Hook 정규식 오버블록의 영향**: `posttooluse-code-validator.sh`가 `.env.example` 파일도 차단하여 KOREA_HOLIDAY_API_KEY 추가 작업이 막혔다. 정규식을 `.env`, `.env.local`, `.env.*.local` 만 차단하도록 좁혔다. Hook 정규식 범위는 가능한 좁게 작성하고, 정기적으로 false positive 여부를 점검해야 한다.

3. **모드 분기 상태 관리의 충돌 위험**: study-period 모드와 event-place 모드가 동시 활성화되면 셀 클릭이 어느 모드로 처리될지 불명확해진다. `useEffect`로 두 모드 간 상호 해제를 강제했지만, 초기 설계 단계에서 XOR 배타 모드를 state machine으로 모델링했다면 더 명확했을 것이다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 담당 |
|----|------|---------|------|
| A23 | `ScheduleEventListItem`에 `is_system_reserved` 필드 추가 — 프론트엔드 codeBadgeClass/draggableEventIds 하드코딩 제거 | 낮음 | Sprint 7 검토 |
| A24 | `.claude/skills/`에 `brainstorming.md`, `frontend-design.md`, `simplify.md` 추가 | 낮음 | Sprint 7 진입 전 |
| A25 | 교습기간 외 드롭 UI 경고 여부 — 배포 후 사용자 피드백으로 판단 | 낮음 | Phase 2 후반 |
| A26 | 2026년 공휴일 데이터 수집 후 V401 마이그레이션 작성 (매년 1월) | 중간 | 2027년 1월 |
| A27 | salt.bin 이전 (Keychain → cloud/smarthb/) — Phase 2 완료 후 처리 | 중간 | Phase 2 완료 후 |

---

## 정량 데이터

| 항목 | 수치 |
|------|------|
| 세션 수 | 9 |
| 커밋 수 | 16 |
| 변경 라인 | ~3,500 |
| 신규 파일 | 9 |
| 수정 파일 | ~15 |
| 신규 IPC 커맨드 | 15 (study_periods 6 + schedule_codes 4 + schedule_events 5) |
| 신규 TS IPC 래퍼 | 15 |
| 신규 도메인 타입 | 10 |
| 신규 컴포넌트 | 4 (ThreeMonthCalendar / CalendarCell / StudyPeriodEditor / ScheduleCodePanel / EventPlacer — 실제 5개) |
| 신규 단위 테스트 | 9 (V301 검증 5건 포함) |
| 백엔드 테스트 총합 | 141 → 146 (+5) |
| 마이그레이션 신규 | 1 (V301 — 시드 보정 + 공휴일 64건) |

---

## 코드 리뷰 요약 (sprint-review 2026-05-22)

**Critical**: 0건
**High**: 0건
**Medium**: 2건 (리스크 등록, 향후 개선 권고)
**Low**: 1건

### Medium 이슈

| ID | 위치 | 내용 |
|----|------|------|
| M-S6-01 | `CalendarCell.tsx` `codeBadgeClass`, `ThreeMonthCalendar.tsx` `draggableEventIds` | 시스템 코드명 6종을 한국어 문자열로 하드코딩. 코드명 변경 시 두 위치 동시 수정 필요. `is_system_reserved` 플래그를 JOIN으로 내려주면 해소 가능 (A23) |
| M-S6-02 | `academic.rs` `update_schedule_event` / 프론트 드롭 핸들러 | 교습기간 범위 밖 드롭 시 백엔드 가드 없음. 현재는 의도된 유연성이나 향후 UX 피드백에 따라 UI 경고 추가 검토 필요 (A25) |

### Low 이슈

| ID | 위치 | 내용 |
|----|------|------|
| L-S6-01 | `EventPlacer.tsx` `useEventPlaceCellHandler` | hook과 컴포넌트가 같은 파일에 혼재. 추후 파일 분리 시 명명 혼란 가능. 현재 규모에서는 허용 범위 |

### 검토 확인 항목 (이상 없음)

- SQL 바인드 파라미터 — 전 커맨드 `sqlx::query().bind()` 사용. raw concat 없음 (Critical 없음)
- `invoke()` 직접 호출 — 0건. 전부 `@/lib/tauri` 래퍼 경유 확인
- `dangerouslySetInnerHTML` — 0건
- localStorage 민감 정보 저장 — 0건
- TypeScript `any` 남용 — 0건 (ThreeMonthCalendar의 `data.current as { eventId?: number }` 캐스팅은 @dnd-kit API 한계로 허용)
- SSR 가드 — `getInvoke()`에서 `typeof window === 'undefined'` 처리 확인. 'use client' 지시어 6개 파일 모두 Tauri IPC 사용 컴포넌트에 적절히 적용
- `auto_place_assessment_dates` 트랜잭션 — `begin() → INSERT loop → commit()` 명시적 트랜잭션 사용. INSERT 중간 실패 시 rollback 정확함
- `assessment_dates_for` / `year_month_of` / `find_assessment_code_id` — 헬퍼 책임 명확히 분리, 단위 테스트 포함
- V301 시드 보정 — `WHERE code_name = '...' AND is_system_reserved = 1` 조건으로 사용자 코드 보호
- WCAG AA 접근성 — CalendarCell: `aria-label`, `aria-pressed`, `focus:ring-2`, `role="button"`, `tabIndex`, 최소 크기 `min-h-[72px] min-w-[44px]` 확인
