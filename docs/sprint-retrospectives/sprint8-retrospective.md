# Sprint Retrospective — Sprint 8

> Sprint 8: 출결 관리 + Sprint 7 carry-over 흡수 — Phase 2 마지막 마일스톤
> 기간: 2026-05-23 ~ 2026-05-24 (9 세션 + T9 follow-up 3건 + V107 hotfix)
> 브랜치: `sprint8` → develop 머지 완료 (--no-ff, `bffecb2`), 이후 V107 review F2 hotfix 1커밋 직접 develop 패치 (`f100708`)
> 코드 리뷰 (2026-05-24): Critical 0 / High 1 (F2, V107로 해소) / Medium 2 (F1·F3, 이연) / Low 2 (F4 무시 / F5 이연)
> 사용자 시각 검증 (2026-05-24): "모두 통과" — T1~T9 + T9 follow-up 3건 전수 정상

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint7-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A28 | `create_study_period` overlap 검사에 `AND is_confirmed = 1` 추가 (R39) | ✅ 완료 | T8에서 `create/update_study_period` 양쪽 처리 + `overlap_skips_unconfirmed_periods` 단위 테스트 |
| A29 | R40~R43 High 4건 우선 처리 | ✅ 완료 | T6 통합 — `is_salt_corrupted` partial-NULL / `set_password` RAII 가드 / `invalidate_credential_cache` pub + exit_hook / `salt_exists_at` fallback 검증 |
| A30 | R44 테스트 격리: `load_salt_backs_up_corrupted_file` `#[ignore]` | ✅ 완료 (Hotfix v0.3.2 선행) | Sprint 8 진입 전 hotfix 로 해소 |
| A31 | lock 테스트 격리 강화 | ✅ 완료 (검토 후 변경 불필요 판정) | T8 검토 — 기존 `if acquired.is_err() { return; }` 가드가 외부 점유 케이스 차단 충분 |
| A32 | R46 Mutex poison 방지: `lock().ok()` 패턴 | ✅ 완료 | T8 `cred_cache_lock` 헬퍼 도입 — 7곳 일괄 `unwrap_or_else(\|e\| e.into_inner())` 적용 |
| A33 | R47 migrate audit 누락 | ✅ 완료 | T8 `migrate_keyring_salt_to` 성공 시 `SecurityEvent` fire-and-forget spawn |
| A34 | `.claude/skills/` 파일 추가 (A24 이연) | ⏸️ 미처리 | Sprint 8 scope 외 — spare-time 작업 |
| A35 | 2027년 공휴일 V401 | ⏸️ 이연 | 2027-01 이후 시점 |
| A36 | R50 NEXT_PUBLIC_ 주석 정정 | ✅ 완료 (Hotfix v0.3.2 선행) | — |
| A37 | R51 배지 클릭 UX | ✅ 완료 | T8 `calendarEventClick` 에 `studyPeriodMode` early return — selection 모드 의도치 않은 삭제 다이얼로그 차단 |
| A38 | 시각 검증을 sprint 계획에 명시 예약 | ✅ 완료 | T9 (통합 검증) 에 "사용자 시각 검증 세션 1시간" 명시. 실제 검증에서 결함 3건(sticky 컬럼, 셀 너비, 검색 필터) 발견 → T9 follow-up 으로 즉시 흡수 |

이행률: **적용 가능한 9건 중 9건 (100%)** — A34/A35 는 적용 시기/scope 외 항목.

---

## 잘한 점

**1.5일(9 세션)에 8건 carry-over + 신규 도메인 1건을 동시 해소했다.**

T1~T5(출결 도메인 신규 구축) + T6~T8(Sprint 7 carry-over 흡수 9건) + T9(통합 검증) 를 한 sprint 안에 처리했다. Sprint 7 의 "carry-over 전담" 패턴을 한 단계 더 발전시켜 **신규 기능 + 부채 청산 병행** 패턴을 실증. 22 커밋 / 9 세션 / 1.5일.

**T7 sprint-review 직전 검증에서 진짜 race 를 발견·해소했다.**

sprint8.md T7 의 가설은 "Sprint 7 T1 CredentialCache 가 race 를 이미 해소했는지 검증"이었지만, 실제 코드 조사에서 `get_cached_or_load_key` + `verify_password` 둘 다 double-checked locking 누락이 잔존함을 확인. `LOAD_MUTEX` + `ensure_cache_loaded` 패턴으로 해소 + 16 스레드 동시 진입 단위 테스트 추가. **계획 단계 가정을 검증으로 뒤집은 사례** — sprint task 가 단순 작업이 아닌 진짜 발견을 동반할 수 있음.

**시각 검증이 sprint 결함 3건을 잡았고 즉시 흡수했다.**

A38 의 의도대로 T9 사용자 시각 검증에서:
1. 출결표 좌측 가로 스크롤 시 요약 4컬럼이 같이 스크롤되어 시야 이탈
2. 셀이 과도하게 넓어 일자 영역 시야 좁음
3. 출결관리에 원생 이름 인플레이스 필터 부재

3건 모두 T9 follow-up 으로 같은 sprint 안에 해결. **시각 검증을 sprint 계획에 명시 예약**한 효과가 즉각 입증됨.

**sprint-review 가 V106 FK 누락(F2)을 발견·즉시 해소했다.**

V106 작성 시 sprint8.md L93 의 의도 (`REFERENCES makeup_attendances(id)`) 와 실제 마이그레이션(`INTEGER` 만) 사이 불일치를 sprint-review 가 발견. V107 hotfix 로 테이블 재생성 패턴 적용 + 기존 dummy id 999 사용 테스트 2건도 `seed_makeup` 헬퍼로 일관 정리. **Phase 3 보강 매칭 도입 전 참조 무결성 확보** — 늦었으면 Phase 3 진입 후 데이터 손상으로 나타날 결함을 사전 차단.

**비즈니스 규칙 단위 테스트 100% 커버.**

T5 에서 보강필요시간 / 소멸기한 10 시나리오 (T3 6건 + T5 신규 4건) 모두 단위 테스트. PRD §6.5 "비즈니스 규칙 100% 단위 테스트 커버" 요건을 출결 도메인에서 충족. cipher off 222 passed / on 133 passed.

---

## 아쉬운 점 / 개선 점

**V106 작성 시 FK 절을 검토 단계에서 빠뜨렸다.**

scope.md Session #1 L45-46 "순환 참조 (regular ↔ makeup)" 에서 "두 테이블 모두 CREATE 후 FK 활성"이라고 설계까지 마쳤으나 실제 SQL 작성 시 `INTEGER` 로만 선언. sprint-review (사용자 위임 시점) 까지 검출되지 않았다. **마이그레이션 작성 후 scope 설계와 1:1 대조 단계** 가 필요. 자동 검증으로 잡기 어려운 종류라 sprint-close 직전 마이그레이션 self-check 항목을 다음 sprint 부터 도입.

**`absent_count` 라벨의 의미가 코드와 헤더 사이 어긋남.**

`compute_summary` 가 `status='absent' AND makeup_attendance_id IS NULL` 만 카운트 (보강완료/소멸 제외) 하지만, 헤더는 단순 "결석(일)" 로 표시. 사용자가 총 결석 수로 오해 가능. T4 UI 작성 시 "어떤 결석을 표시할지" 의도가 코드와 라벨 양쪽에 명문화되지 않았다 — 차기 sprint 에서 "미처리 결석(일)" 으로 변경하거나 툴팁 추가.

**sprint-review 에이전트가 산출물 문서를 만들지 않았다.**

에이전트에게 명시적으로 `docs/code-reviews/sprint8.md` + `docs/sprint-retrospectives/sprint8-retrospective.md` 작성을 지시했으나 JSON 분석만 보고하고 파일 생성은 누락. **에이전트 호출 시 산출물 파일 경로를 prompt 결말부에 1줄 체크리스트로 명시** 하는 패턴이 필요. 본 회고와 코드 리뷰 문서는 사용자 결정 후 별도 직접 작성으로 보충.

**`get_attendance_grid` N+1 패턴이 PRD 요건은 통과하지만 잠재 위험.**

현재 50명×31일 < 1초 통과 (cipher off). 그러나 학생 루프 안에 4쿼리가 박혀 있어 데이터 누적 / 느린 HDD / 클라우드 sync 지연 환경에서 성능 회귀 가능. 차기 sprint 에서 JOIN 또는 단일 IN 쿼리 batch 처리로 리팩토링.

**T9 follow-up 3건이 sprint scope 초과의 신호.**

T9 follow-up 으로 sticky 컬럼 + 너비 조정 + 검색 필터를 흡수했지만, 검색 필터는 "기존 UX 미흡 보완"이 아닌 **신규 기능** 이라 엄밀히는 차기 sprint 작업. 단일 개발자 + 사용자 즉시 요구라 sprint 내 흡수가 합리적이었지만, 패턴화되면 sprint 계획 수렴 어려움. 다음 sprint planner 가 "T9 follow-up 신규 기능 카운트" 모니터링 필요.

---

## 다음 sprint 액션 항목

| ID | 항목 | 우선순위 | 위치 |
|----|------|----------|------|
| A39 | V106 FK 누락 회고 — sprint-close 직전 마이그레이션 self-check 체크리스트 도입 (scope.md 설계 vs 실제 SQL 1:1 대조) | High | `.claude/skills/` 또는 sprint-close 에이전트 prompt |
| A40 | sprint-review 에이전트 prompt 결말부에 산출물 파일 경로 명시 체크리스트 추가 (`docs/code-reviews/sprintN.md` + `docs/sprint-retrospectives/sprintN-retrospective.md` 필수 작성) | High | `.claude/agents/sprint-review.md` |
| A41 | F1 — "결석(일)" 라벨 의미 명확화 ("미처리 결석(일)" 변경 또는 툴팁) | Medium | `AttendanceGrid` + `compute_summary` 주석 |
| A42 | F3 — `get_attendance_grid` N+1 → batch 쿼리 리팩토링 | Medium | `attendance.rs::get_grid_impl` |
| A43 | F5 — `validate_year_month` 월 범위(01-12) 검증 강화 | Low | `attendance.rs::validate_year_month` |
| A44 | R48-b — salt buffer ZeroizeOnDrop 시그니처 변경 (광범위) | Medium | `auth.rs` load/store/generate/migrate salt 함수군 |
| A45 | 반응형 폰트/셀 너비 — `clamp()` viewport 또는 html font-size 미디어쿼리 + rem 일괄 전환. AttendanceGrid 셀 너비도 동기 | Medium | `globals.css` + 전 컴포넌트 셀 너비 |
| A46 | 한글 자모 부분 일치 검색 — `hangul-js` 또는 직접 자모 분해 알고리즘. 글로벌 검색바 / AttendancePage 양쪽 적용 | Medium | `global-search.tsx` + `/attendance/page.tsx` |
| A47 | 한국어 리터럴 잔존 (R49) — Sprint 7 미해소. 영향 없음 평가지만 코드 정합성 위해 후속 정리 | Low | `CalendarCell.tsx` |
| A48 | 2027년 공휴일 V401 (A35 이월) | Low | 2027-01 이후 작업 |

**우선 처리 권장**: A39/A40 은 프로세스 개선이라 다음 sprint 진입 전 적용. A41/F3 는 Sprint 9 (보강 도메인) 와 자연스럽게 묶어 처리.

---

## 메트릭

| 지표 | 값 |
|------|-----|
| 기간 | 1.5일 (2026-05-23 ~ 2026-05-24) |
| 세션 수 | 9 + T9 follow-up 3 + V107 hotfix 1 = **13 세션** |
| 커밋 수 (sprint8 → develop) | 22 (sprint8 머지 후 V107 +1 → develop 기준 23) |
| 단위 테스트 신규 | T1 5건 + T2 9건 + T3 9건 + T5 4건 + T6 5건 + T7 2건 + T8 3건 + V107 1건 = **38건** |
| 누적 단위 테스트 (cipher off) | **222 passed** (T9 진입 시 213 → +9 T6/T7/T8/V107) |
| 누적 단위 테스트 (cipher on) | **133 passed** |
| 자동 검증 통과 | 7/7 (cargo test off/on, clippy off/on, pnpm lint/tsc/build) |
| 시각 검증 결함 발견 | 3건 (T9 follow-up 즉시 흡수) |
| sprint-review 결함 발견 | 5건 (F1~F5) — F2 V107 hotfix 즉시 해소, F1/F3/F5 이연, F4 무시 |
| 이전 회고 액션 이행률 | 9/9 (적용 가능한 항목 기준) |

---

## 종합 평가

Phase 2 마감 시점에 출결 도메인 + Sprint 7 carry-over 9건을 한 sprint 로 완결했다. 시각 검증 + sprint-review 두 단계 검증이 각각 3건/5건 결함을 잡았고, F2(High) 는 V107 hotfix 로 develop 머지 후에도 즉시 해소되어 Phase 3 진입에 무리 없다.

다음 sprint 부터는 sprint-close 직전 **마이그레이션 self-check** + sprint-review 에이전트의 **산출물 파일 작성 강제** 두 프로세스 개선을 도입하면 본 sprint 의 두 가지 아쉬운 점이 사라진다.

**Phase 2 (학사 + 출결) 종결 — Sprint 9 (Phase 3 보강 + 소멸) 진입 준비 완료**.
