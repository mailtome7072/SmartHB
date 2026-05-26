# Sprint Retrospective — Sprint 9

> Sprint 9: Phase 3 첫 마일스톤 — 보강 등록 + 보강-결석 매칭 + 취소/환원 + 결석 이력
> 기간: 2026-05-24 ~ 2026-05-26 (12 세션 — T1~T9 기본 흐름 + T10~T12 시각 검증 3라운드 흡수)
> 브랜치: `sprint9` (develop 머지 대기)
> 코드 리뷰 (2026-05-26): Critical 0 / High 0 / Medium 1 (F1, dead code) / Low 2 (F2, F3)
> 자동 검증: cargo test cipher off 253 passed / cipher on 133 passed / clippy clean / lint clean / tsc clean / build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint8-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A39 | sprint-close 직전 마이그레이션 self-check — scope.md 설계 vs 실제 SQL 1:1 대조 | ✅ 완료 | T9 통합 검증 DoD에 항목 명시 + `.claude/agents/sprint-review.md` 반영 (커밋 `64746da`) |
| A40 | sprint-review 산출물 파일 4종 작성 강제 | ✅ 완료 | sprint-review 에이전트 prompt에 self-check 섹션 추가 (커밋 `64746da`) |
| A41 | "결석(일)" 라벨 → "미처리 결석(일)" 변경 + `compute_summary` 주석 | ✅ 완료 | T7에서 흡수 — `AttendanceGrid` 헤더 + `compute_summary` 주석 명확화 |
| A42 | `get_attendance_grid` N+1 → batch 쿼리 리팩토링 | ⏸️ 이연 | 보강 도메인 scope 압박 + PRD 요건 충족 중. Sprint 10 이후 |
| A43 | `validate_year_month` 월 범위(01-12) 검증 강화 | ✅ 완료 | T2에서 흡수 — 보강 IPC가 year_month 입력받으므로 자연스러운 타이밍 |
| A44 | R48-b salt buffer ZeroizeOnDrop 시그니처 변경 | ⏸️ 이연 | 보안 도메인, 보강과 무관. 광범위 영향 |
| A45 | 반응형 폰트/셀 너비 clamp() 전환 | ⏸️ 이연 | UX 전반, 본 sprint scope 외 |
| A46 | 한글 자모 부분 일치 검색 | ⏸️ 이연 | 검색 영역, capacity 초과 우려 |
| A47 | 한국어 리터럴 잔존 (R49) CalendarCell.tsx | ⏸️ 이연 | 기능 영향 없음 |
| A48 | 2027년 공휴일 V401 (A35 이월) | ⏸️ 이연 | 시점 미도래 |

이행률: **적용 가능한 5건 중 5건 (100%)** — A39/A40/A41/A43 완료, A42/A44~A48 은 이연(scope 외 또는 시점 미도래).

---

## 잘한 점

**보강 도메인을 백엔드 IPC 7종 + 단위 테스트 28건 + UI 4 다이얼로그로 완결했다.**

T2~T4(백엔드 IPC 7종) + T5(TS 래퍼 6종) + T6~T8(UI 4 다이얼로그) + T9(통합 검증)로 보강 도메인 전체를 한 sprint 에 완성. `create_makeup_with_absences`, `cancel_makeup`, `get_pending_absences`, `get_makeup_eligible_dates`, `get_absence_history` 등 핵심 IPC 7종이 각각 3~9건의 단위 테스트를 보유하여 PRD §6.5 비즈니스 규칙 100% 단위 테스트 커버를 달성했다.

**시각 검증 3라운드(18건)를 sprint 내 흡수하여 도메인 모델을 정제했다.**

I1~I8(1차: 시간 단위 변환 + UX 보완) 및 J1~J10(2~3차: 도메인 모델 정제 + UX 보완) 18건을 T10~T12로 흡수. 특히 I3(보강 가능일 재정의)는 단순 UX 문제가 아닌 **도메인 모델 수준의 정제** 였다:

- 기존: `study_periods` 활성 기간 내 날짜만 보강 가능
- 변경: 케이스 A(평일 + `schedule_codes`에 `allows_makeup_class=0` 코드 없음) OR 케이스 B(`allows_makeup_class=1` 명시) — study_periods 제약 제거

이 정의 변경이 없었다면 원장이 실제 보강 등록 시 의도한 날짜가 차단되는 사용성 문제가 프로덕션에 나타났을 것이다. 시각 검증이 도메인 설계 오류를 조기에 발견한 사례.

**TDD 패턴(단위 테스트 선행 or 동시 작성)이 안정적으로 정착했다.**

T2, T3, T4 각각 9건, 9건, 7건 단위 테스트를 IPC 구현과 동시에 작성했다. 취소 트랜잭션(`cancel_makeup`)의 FK 위반 회피 순서(UPDATE absences 먼저 → DELETE makeup)나 보강 등록 race 방지(`rows_affected()=1` 검증)는 단위 테스트 작성 과정에서 설계가 명확해진 사례다. Sprint 8 에서 "비즈니스 규칙 100% 단위 테스트 커버 success pattern"으로 인정된 이 방식이 Sprint 9에서도 동일하게 작동했다.

**도메인 간 결정을 사용자와 즉시 수렴했다.**

J5(보강 미등원 UI 삭제), J7(보강데이 일괄 폐기) 등 도메인 결정이 필요한 순간마다 사용자와 즉시 수렴하여 방향을 확정하고 코드에 반영했다. 이 결정들이 sprint-review 단계에서 dead code 발견(F1)으로 이어졌지만, 미결 상태로 Sprint 10에 넘어가는 것보다 훨씬 낫다.

**A39 마이그레이션 self-check가 실제로 동작했다.**

T1에서 V106/V107 기존 마이그레이션이 Sprint 9 전체 보강 도메인을 지원함을 scope.md 설계와 1:1 대조하여 검증했다. 신규 마이그레이션 없이 기존 스키마를 활용한다는 판단이 T9 통합 검증까지 문제없이 유지되었다 — Sprint 8의 V106 FK 누락과는 대조적인 결과.

---

## 아쉬운 점 / 개선 점

**Capacity가 계획 38h 대비 실측 약 52h로 37% 초과했다.**

시각 검증 3라운드(I1~I8 10h + J1~J10 4h = +14h)가 주요 원인. 시각 검증 자체는 가치 있는 활동이지만, Sprint 9 계획 단계에서 T9에 "사용자 시각 검증 세션 1시간"으로만 예약했고 다라운드 반복 가능성을 capacity에 반영하지 않았다. **시각 검증은 라운드당 1회 fixed로 계획하기보다, "1라운드 + n건 버퍼" 형태로 capacity를 별도 분리해야 한다.**

**J5/J7 결정으로 백엔드 dead code가 발생했다.**

`mark_makeup_absent`, `batch_create_makeups` IPC 7종 단위 테스트가 있는 상태에서 프론트엔드 호출 경로가 제거되었다. 결정 자체는 옳았지만, 백엔드 dead code 정리를 즉시 수행하지 않아 Sprint 10 carry-over(F1/R63)로 이어졌다. **UI 폐기 결정 시 즉시 "백엔드 IPC도 폐기 대상인지" 체크하는 습관이 필요.**

**T3 검증 3(정규 수업 요일 차단)이 I3 시각 검증에서 폐기되어 재작업이 발생했다.**

T3 구현 시 "보강은 비수업일 한정"으로 설계하고 단위 테스트 9건 중 1건이 이 가정을 검증했으나, I3 시각 검증에서 원장의 실제 운용 방식(정규 수업 요일에도 보강 허용)과 충돌하여 T10에서 검증 3를 폐기했다. 단위 테스트도 수정이 필요했다. **도메인 규칙 중 운용 관행과 교차하는 부분(요일 제약 등)은 T1 설계 단계에서 사용자 확인을 먼저 받아야 한다.**

**시간 단위 UI vs 분 단위 백엔드 불일치가 후기(I1)에야 발견되었다.**

`class_minutes`(분)를 백엔드에서 관리하면서 UI 입력/표시는 시간 단위로 맞추는 변환 계층이 T5(TS 래퍼) 작성 시점에 설계되었어야 했으나, I1 시각 검증에서 실제 UI를 보고 나서야 발견됐다. T5 scope.md 선언 시 "사용자가 보는 단위는 무엇인가"를 명시하는 습관이 필요.

---

## 다음 sprint 액션 항목

| ID | 항목 | 우선순위 | 위치 |
|----|------|----------|------|
| A49 | F1 — `mark_makeup_absent` + `batch_create_makeups` 백엔드 IPC + audit variant dead code 정리. IPC handler 등록 제거 + 단위 테스트 폐기 또는 stub 처리 | High | `src-tauri/src/commands/makeup.rs`, `src-tauri/src/lib.rs` |
| A50 | 시각 검증 capacity buffer 분리 — Sprint 10 계획 시 "시각 검증 라운드 버퍼 4h" 별도 예약 (T9 내 포함하지 않음) | High | Sprint 10 계획 문서 |
| A51 | 도메인 규칙 중 운용 관행 교차 항목 T1에서 사용자 확인 선행 — 특히 요일 제약, 시간 단위 등 | Medium | sprint-planner 에이전트 / T1 설계 패턴 |
| A52 | F2 — `get_absence_history` pagination.rs 헬퍼 적용 (R64) | Medium | `src-tauri/src/commands/makeup.rs:742-780` |
| A53 | F3 — `create_makeup_with_absences_impl` 결석 루프 검증 → `JSON_EACH` IN 절 교체 (R65) | Low | `src-tauri/src/commands/makeup.rs:434-467` |
| A54 | A42 이월 — `get_attendance_grid` N+1 → batch 쿼리 (R42) | Medium | `src-tauri/src/commands/attendance.rs::get_grid_impl` |
| A55 | A44 이월 — salt buffer ZeroizeOnDrop 시그니처 변경 (R48-b) | Medium | `auth.rs` load/store/generate/migrate salt 함수군 |
| A56 | A45 이월 — 반응형 폰트/셀 너비 clamp() + rem 일괄 전환 | Medium | `globals.css` + 컴포넌트 셀 너비 |
| A57 | A46 이월 — 한글 자모 부분 일치 검색 | Medium | `global-search.tsx` + `/attendance/page.tsx` |

**우선 처리 권장**: A49(dead code 정리)와 A50(capacity 계획 패턴 변경)은 Sprint 10 진입 전에 반영. A51은 sprint-planner에게 T1 설계 체크리스트로 전달.

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 기간 | 3일 (2026-05-24 ~ 2026-05-26) |
| 세션 수 | T1~T9 기본 (9) + T10~T12 시각 검증 3라운드 (3) = **12 세션** |
| 커밋 수 (sprint9 브랜치) | 22 (docs 커밋 포함) |
| 백엔드 IPC 신규 | 7종 (`get_pending_absences`, `get_makeup_eligible_dates`, `create_makeup_with_absences`, `cancel_makeup`, `mark_makeup_absent`, `batch_create_makeups`, `get_absence_history`) |
| 단위 테스트 신규 | T2 9건 + T3 9건 + T4 7건 + T8 3건 = **28건** |
| 누적 단위 테스트 (cipher off) | **253 passed** (Sprint 8 222 → +31 보강 도메인) |
| 누적 단위 테스트 (cipher on) | **133 passed** |
| TS IPC 래퍼 신규 | 6종 (T5) |
| 도메인 타입 신규 | 8종 (`PendingAbsence`, `EligibleDate`, `CreateMakeupPayload`, `MakeupResult`, 등) |
| UI 다이얼로그 신규 | 4종 (`MakeupRegisterDialog`, `MakeupManageDialog`, `AbsenceHistoryDialog`, `BatchMakeupDialog` — 이후 폐기) |
| 자동 검증 통과 | 7/7 (cargo test off/on, clippy off/on, pnpm lint/tsc/build) |
| 시각 검증 라운드 | 3라운드 18건 (I1~I8 + J1~J10) — 전수 sprint 내 흡수 |
| sprint-review 결함 | 3건 (F1 Medium, F2/F3 Low) — Critical/High 0건 |
| Capacity | 계획 38h → 실측 약 52h (+14h, **+37%**) |
| 이전 회고 액션 이행률 | 5/5 적용 가능 항목 100% (**A39, A40, A41, A43 완료**) |

---

## 종합 평가

Sprint 9는 Phase 3의 첫 마일스톤으로서 보강 도메인 전체(등록 + 매칭 + 취소 + 결석 이력)를 완성했다. Critical/High 결함 없이 개발이 완료되었고, 시각 검증 3라운드가 도메인 모델 수준의 설계 오류(I3 보강 가능일)를 포함한 18건을 사전에 발견하여 프로덕션 품질을 높였다.

Capacity 37% 초과는 부정적이지만, 초과 원인이 명확하다(시각 검증 다라운드). Sprint 10에서는 시각 검증 버퍼를 capacity에 명시적으로 예약하여 계획 정확도를 회복할 수 있다.

A49(dead code 정리)를 Sprint 10 진입 전에 처리하면 코드베이스 부채를 최소화한 상태로 소멸 자동 전이 + 캘린더 뷰에 집중할 수 있다.

**Phase 3 Sprint 9 완료 — Sprint 10 (소멸 자동 전이 + 퇴교 보강 처리 + 캘린더 뷰) 진입 준비 완료.**
