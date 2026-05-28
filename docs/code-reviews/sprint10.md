# Sprint 10 코드 리뷰

> 대상: Sprint 10 (develop...HEAD, 63개 파일) — Phase 3 완료: 소멸 자동 전이 + 캘린더 뷰
> 리뷰 일자: 2026-05-28
> 자동 검증 결과: cargo test 273 passed (cipher off) / clippy clean / tsc clean / lint clean / build 16/16

---

## 발견 사항 (5건)

### F1 — build_day_schedules 날짜 루프 succ_opt().expect() 패닉 가능 (Medium, 미수정)

- 위치: `src-tauri/src/commands/attendance.rs:655`
- 내용: `d.succ_opt().expect("date succ")` — NaiveDate 범위 초과 시 패닉.
- 실패 시나리오: `schedule_events.period_end_date`에 9999-12-31 같은 극단값이 저장된 경우 succ_opt()가 None을 반환, expect()가 패닉 → 애플리케이션 크래시.
- 비교: 동일 파일 attendance.rs의 line 139, 259은 동일 패턴에 `.ok_or_else(|| "날짜 계산 오버플로".to_string())?`로 안전하게 처리.
- 조치: ROADMAP 이연 (실데이터에서 극단값 입력 가능성 낮음, 다음 스프린트 일괄 정리)

### F2 — generate_impl에서 expire 실패 시 출결 생성 커밋 후 IPC 에러 반환 (Medium, 미수정)

- 위치: `src-tauri/src/commands/attendance.rs:155`
- 내용: 출결 생성 트랜잭션 커밋 후 `expire_overdue_absences_impl`이 `?`로 에러 propagate.
- 실패 시나리오: DB 일시 오류 등으로 expire가 실패하면 출결은 DB에 반영됐으나 IPC가 에러를 반환 → 프론트엔드는 실패로 인식, 사용자가 출결 생성을 재시도할 경우 "이미 존재" 에러 발생.
- 비교: startup.rs는 동일 expire 함수를 fail-soft (에러 무시 + Ok 반환)로 처리 — 일관성 불일치.
- 조치: ROADMAP 이연 (실운용 환경에서 DB 오류 후 expire 실패 발생 가능성 매우 낮음)

### F3 — get_makeup_management_data_impl의 _year_month 파라미터 미사용 (Low, 미수정)

- 위치: `src-tauri/src/commands/calendar.rs:186-188`
- 내용: `_year_month: &str` 파라미터가 SQL 쿼리에서 전혀 사용되지 않음. 함수는 연월 무관하게 전체 미보강 결석을 집계.
- 실패 시나리오: 보강 관리 뷰 사용자가 특정 월 기준 필터링을 기대할 때 다른 월 결석도 표시. 현재 UI는 전체 보강 현황 뷰로 설계되어 있어 의도적일 수 있으나 파라미터명이 혼란 유발.
- 조치: 의도적 미사용임을 주석으로 명확화 필요 (PRD §4.6.3 전체 현황 뷰 기준)

### F4 — get_makeup_management_data_impl 원생별 study_periods 개별 조회 N+1 (Low, 미수정)

- 위치: `src-tauri/src/commands/calendar.rs:215-230`
- 내용: 원생 N명에 대해 각 `earliest_deadline`별로 `study_periods`를 개별 쿼리 조회.
- 실패 시나리오: 보강 필요 원생 50명 → 최대 50번 추가 SELECT. 현 규모에서 성능 영향 미미하나 원생 수 증가 시 선형 증가. LEFT JOIN으로 단일 쿼리 처리 가능.
- 조치: A52(R64) 이연 패턴과 동일 — Phase 4 이후 N+1 일괄 개선 시 포함

### F5 — ClassCalendar events useMemo viewType 비동기 업데이트로 한 프레임 이벤트 상태 불일치 (Low, 미수정)

- 위치: `src/components/schedules/ClassCalendar.tsx:164-191`
- 내용: events useMemo가 viewType state에 의존하나, viewType은 FullCalendar의 datesSet 콜백 후 비동기로 업데이트.
- 실패 시나리오: 월→주 뷰 전환 직후 한 render에서 events=[]가 FC에 전달되어 이벤트 블록이 잠깐 사라졌다가 다음 render에서 표시. 7라운드 시각 검증에서 미발견(미미한 깜빡임), 기능 정확성에는 영향 없음.
- 조치: 차기 시각 검증 또는 UX 개선 Sprint에서 처리

---

## 영역별 추가 점검

### 보안 (backend.md Critical 체크)

- SQL 인젝션 — `query!` 매크로 대신 raw `sqlx::query`를 일부 사용하나 파라미터 바인딩 전수 적용 확인. `?.bind(memo)`, `?.bind(student_id)` 패턴 일관. **이상 없음**
- 하드코딩 시크릿 — git diff 스캔 결과 패턴 없음. **이상 없음**
- Tauri 권한 — 새 커맨드 6종 모두 `invoke_handler` 등록만, `capabilities/default.json` 변경 없음 (기존 권한 범위 내). **이상 없음**

### 보안 (backend.md High 체크)

- `unwrap()`/`expect()` — 프로덕션 코드 내 `expect("validated")` 패턴은 입력 검증 후 불변성 가정에 사용. F1(attend.rs:655)은 Medium으로 등록. **허용 범위**
- 마이그레이션 — V108 1:1 self-check 통과. **이상 없음**
- `.sqlx/` 캐시 — 회사 PC에서 갱신 후 커밋 완료 (커밋 `dde74aa` ~ `a3b4915` 범위). **이상 없음**

### 프론트엔드 (frontend.md Critical/High 체크)

- XSS — `dangerouslySetInnerHTML` 사용 없음. 사용자 입력은 제어 컴포넌트 패턴. **이상 없음**
- `invoke()` 직접 호출 — `src/lib/tauri/index.ts` 추상화 계층만 사용. **이상 없음**
- TypeScript any — ClassCalendar.tsx의 FullCalendar 콜백 인자 암시적 any 6건 (tsc 통과, ESLint 무시) — Low 수준
- 접근성 — 신규 다이얼로그(`MakeupManageDialog`, `WithdrawalMakeupDialog`) 모두 `role="dialog" aria-modal="true"` + ESC 키 처리 적용. **이상 없음**

### AI 생성 코드 추가 체크

- 도메인 규칙 단위 테스트: `expire_overdue_absences_impl` 7건, `process_withdrawal_makeup_impl` 6건 — 경계조건(교습기간 미등록, 이미 소멸된 결석, 복수 원생 batch) 포함 충분한 커버. **양호**
- DB 트랜잭션 원자성: process_withdrawal_makeup_impl은 memo 업데이트 + 상태 전이 + withdraw_date 설정을 단일 트랜잭션 내 수행. **양호**
- cipher 게이트 정합(a3b4915): 8개 테스트 모듈에 `#[cfg(all(test, not(feature="cipher")))]` 게이트 적용 — cipher on 빌드/테스트가 처음으로 통과. **Sprint 10 의미 있는 개선**

---

## 결론

Critical 0 / High 0 / Medium 2 (F1, F2) / Low 3 (F3, F4, F5). 모두 ROADMAP 이연 처리.
주요 도메인 로직(소멸 전이 트리거 3개소, 퇴교 보강 처리 3선택지, 캘린더 집계)은 단위 테스트와 단일 트랜잭션 원자성으로 충분히 보장됨.
V108 FK 재구성(d402bb9)은 실데이터 자식 FK 위반(code 787)을 시각 검증으로 발견하여 수정 — 빈 인메모리 테스트 한계를 시각 검증이 보완한 사례로 회고에 기록.
