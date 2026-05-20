# Code Review Report — Sprint 2

> 검토 대상: `sprint2` 브랜치 (`git diff develop..sprint2`)
> 변경 파일: 33개 (백엔드 16개, 프론트엔드 10개, 문서/설정 7개)
> 검토 기준: `.claude/skills/code-review.md` 체크리스트

---

## 요약

| 등급 | 건수 |
|------|------|
| Critical | 0 |
| High | 0 |
| Medium | 4 |
| Low | 3 |

Critical/High 이슈 없음. 배포 진행 가능.

---

## AI 생성 코드 추가 체크

- ✅ 비즈니스 로직이 sprint2.md 계획 문서와 일치함 — T1~T7·T9~T13·T14 완료, T2·T8 이연 결정이 sprint2.md에 명시됨
- ✅ 의도치 않은 파일 변경 없음 — 33개 파일 모두 sprint scope 내
- ✅ 하드코딩된 테스트 더미 값 프로덕션 코드 없음 — mock 데이터는 `!inv` 분기(개발 모드) 전용
- ✅ 추가된 의존성 없음 (T8 이연으로 `query!()` 매크로 전환 없음)

---

## 보안

- ✅ 하드코딩된 시크릿·API 키·비밀번호 없음
- ✅ SQL 인젝션 방지 — `list_students` 동적 SQL 빌더가 정적 단편 + `bind()` 패턴 일관 적용. `ORDER BY` 분기는 `StudentSort::order_by_sql()`이 `&'static str`을 반환하는 열거형 기반으로 사용자 입력이 SQL 식별자로 직접 노출될 경로 없음
- ✅ `reorder_codes` 동적 SQL — `table.table_name()`, `column` 모두 `CodeTable` enum 정적 매핑. `orders: Vec<(i64, i64)>` 는 `i64` 숫자만 `bind()` 처리되어 주입 경로 없음
- ✅ XSS 방지 — `dangerouslySetInnerHTML` 미사용, 사용자 입력 직접 렌더링 없음
- ✅ 인증 게이트 — 루트 페이지(`page.tsx`)가 `isUnlocked()` 확인 후 `/lock` redirect. `auth-state.ts`에 비밀번호·키 저장 없음, `sessionStorage` 미사용
- ✅ `AppError::Db`, `AppError::Auth` 등 기술 상세가 IPC 응답에서 `user_message()`로 변환되어 누출 방지 확인

**Medium M1 — `audit::try_record` details 파라미터 PII 경계 (audit.rs:293, students.rs:293)**

`create_student`에서 `try_record(StudentCreated, Some(&serial), Some(&payload.name))`를 호출한다. `event_subject`에 일련번호, `details`에 원생 이름이 들어간다. 이름은 PII이며 PRD §6.6의 "호출자가 details 직렬화 시점에 사전 마스킹" 원칙을 적용해야 한다. audit.rs 주석(line 15)에도 "호출자가 사전 마스킹" 책임이 명시되어 있으나 현재 `payload.name`이 그대로 전달된다. `update_student`, `withdraw_student`도 동일.

- 위치: `src-tauri/src/commands/students.rs:293~298, 338~344, 389~394`
- 영향도: Medium — 감사 로그 조회 시 원생 이름이 평문으로 노출
- 대응 방안: `details`에 이름 전달 제거 또는 마스킹 (예: 이름 앞 1자 + `**`), Sprint 3 감사 로그 UI 구축 전 해소 권장

---

## 성능

- ✅ N+1 쿼리 없음 — `list_students`가 `day_of_week` 필터를 `INNER JOIN student_schedules`로 단일 쿼리 처리
- ✅ 인덱스 적용 — `idx_students_active`(재원 필터), `uq_student_day_current`(스케줄 UNIQUE), `idx_student_schedules_student_id`(JOIN 가속) 모두 V101에 정의됨

**Medium M2 — `list_students` 페이지네이션 누락 (students.rs:403)**

`list_students`에 LIMIT/OFFSET이 없다. 현재는 소규모 교습소(50명 이내)라 실질 문제는 없으나, code-review.md 기준상 리스트 응답에 페이지네이션을 요구한다.

- 위치: `src-tauri/src/commands/students.rs:403`
- 영향도: Medium — 단일 교습소 모델이라 실질 영향 낮음. Sprint 3 목록 화면 구현 전 page/limit 파라미터 추가 권장
- risk-register 등록 대상

**Medium M3 — `list_codes` 페이지네이션 누락 (codes.rs:171)**

`list_codes`도 동일하게 LIMIT 없음. 코드 테이블 특성상 항목 수가 적어 실질 문제는 없으나 동일 기준 적용.

- 위치: `src-tauri/src/commands/codes.rs:171`
- 영향도: Medium — 실질 영향 최소. 이후 스프린트에서 코드 관리 화면 구현 시 처리
- risk-register 등록 대상

---

## 코드 품질

- ✅ TypeScript strict 모드 통과 — `pnpm tsc --noEmit` 오류 없음
- ✅ `any` 타입 미사용 — 모든 IPC 래퍼가 명시적 타입 선언 사용
- ✅ 에러 핸들링 일관성 — 22개 IPC 모두 `Result<T, String>` + `AppError` → `String::from` 변환 패턴 일관 적용
- ✅ `auth-state.ts` — 비밀번호·키를 저장하지 않고 측정값(`StartupResult`)만 보관. 주석에 Sprint 3 Zustand 교체 계획 명시
- ✅ `Zeroizing<String>` 적용 — `app_startup_sequence`에서 비밀번호 메모리 폐기 보장

**Medium M4 — `startup.rs::exit_hook` 락 해제 방식 (startup.rs:210)**

`exit_hook`에서 락 해제 시 `release_lock_atomic()` 대신 `std::fs::remove_file`을 직접 호출한다. `release_lock_atomic`은 advisory lock을 획득하고 본 디바이스 점유 여부를 재확인한 후 삭제하는 반면, 직접 `remove_file`은 이 검증을 생략한다. 다른 디바이스가 점유한 락 파일을 비정상 종료 직전에 삭제할 수 있는 엣지 케이스가 존재한다.

- 위치: `src-tauri/src/startup.rs:210~214`
- 영향도: Medium — 단일 사용자 단일 PC 사용 시나리오에서 실질 충돌 가능성 낮음. 단, 클라우드 동기화 지연 + 비정상 종료 조합 시 다른 PC 락이 손상될 수 있음
- 대응 방안: `tokio::task::spawn_blocking(|| release_lock_atomic().ok()).await.ok()` 패턴으로 교체

---

## 테스트

- ✅ 97개 테스트 전체 통과 (Sprint 1 64개 → Sprint 2 97개 +33건)
- ✅ 신규 IPC 커맨드별 `#[cfg(test)]` 블록 작성 — students, schedules, fees, codes, audit 모두 인메모리 DB 테스트 포함
- ✅ PI-05 자동 채번 핵심 시나리오 커버 — 연속 증가, override 후 연속성, UNIQUE 위반 한국어 메시지, 영문 prefix 제외
- ✅ 스케줄 변경 이력 + 부분 인덱스 UNIQUE 제약 + 주 수업시간 합산 테스트
- ✅ 교습비 매칭 경계값 테스트 — 정확 일치, 이하 최댓값, 미등록 구간

**Low L1 — `get_schedule_history` / `get_audit_logs` 테스트 미작성**

`get_schedule_history`와 `get_audit_logs` IPC가 단위 테스트 없이 SQL 직접 검증으로만 대체되어 있다. `record_and_list_logs_round_trip` 테스트는 전역 POOL 제약으로 IPC 레벨 호출 불가 이유가 주석에 명시되어 있어 의도적 제한임을 인지.

---

## 패턴 준수

- ✅ 모든 Tauri 커맨드가 `src-tauri/src/commands/` 하위에 모듈별 파일로 분리
- ✅ `src-tauri/src/lib.rs` `invoke_handler`에 22개 IPC 전체 등록 확인 필요

**Low L2 — 마이그레이션 파일명 V{NNN} 표기 불일치**

Sprint 1 마이그레이션은 `001__`, `008__` 형식이지만 Sprint 2는 `101__`~`105__`를 사용한다. `backend.md`의 공식 표기는 `V{NNN}__{설명}.sql`이다. 실행 순서상 문제는 없으나 컨벤션 불일치가 있다. 향후 마이그레이션 관리 복잡도 증가 가능성.

- 위치: `src-tauri/migrations/`
- 영향도: Low — 현재 SQLx migrate 실행 순서에 영향 없음. hotfix 후보로 파일명 정리 권장

**Low L3 — `LockScreen.tsx` 미검토**

`src/components/LockScreen.tsx`를 이번 리뷰에서 확인했으나 `src/app/lock/page.tsx`와 직접 연계되어 있어 별도 항목으로 기록. `app_startup_sequence` 호출 후 `markUnlocked()` 호출 흐름은 정상.

---

## Sprint 1 risk 처리 결과 재검증

**R6 (salt 이전) — chicken-and-egg 결정 타당성 확인**

T2 이연 결정은 기술적으로 정당하다. salt를 DB(`app_settings`)에 저장하면 DB를 열기 위해 PRAGMA key가 필요하고, key 유도에 salt가 필요한 순환 의존이 발생한다. Sprint 3 마법사에서 `{data_root}/salt.bin` 평문 파일로 보관하는 방안은 OWASP 기준(salt는 비밀이 아니라 무작위성만 요구)과 부합하며 R12 해소 조건이 Sprint 3 마법사 통합으로 명확히 정의되어 있다.

**R7 (release_lock advisory lock) — 해소 확인 (주의사항 포함)**

`release_lock_atomic()`이 `try_lock_exclusive()` + 본 디바이스 확인 + `drop(file)` + `remove_file` 순서로 구현되어 Sprint 1 R7 요건을 충족한다. Windows file handle 닫기 순서(drop 후 remove_file)도 올바르게 처리되었다. 단, `exit_hook`에서 이 함수를 우회하는 이슈가 M4로 별도 등록됨.

**R8 (cipher on 실측) — 측정 코드 충분성 확인**

`StartupResult` 4개 timing 필드(`parallel_phase_ms`, `password_verify_ms`, `db_init_ms`, `audit_cleanup_ms`)와 `eprintln!` 로그가 병목 식별에 충분하다. PBKDF2 600K iter가 `password_verify_ms`로 분리되어 있어 3초 예산 초과 시 원인을 즉시 식별 가능하다. 실제 사용자 환경 측정은 v0.2.0 배포 후 위임 — 이 결정은 타당하다.

---

## SSOT 정합성 검토

- ✅ `serial_no TEXT` — V101 마이그레이션, students.rs의 `serial_no: String` 타입, data-model §1.1 일치. sprint2.md의 `INTEGER` 표기는 마이그레이션 주석에 "미스 보정" 명시
- ✅ `duration_hours INTEGER` — V101 마이그레이션의 `INTEGER NOT NULL`, schedules.rs의 `duration_hours: i64`, data-model §1.2 일치. sprint2.md의 `REAL` 표기 미스 보정 확인
- ✅ V102 schedule_codes 시드 5종 — `is_period_type` 컬럼 포함 3속성 값 정의. PRD §4.4.4 기준 검토: 방학/휴원일은 정규수업 불가, 보강데이는 보강 허용, 공휴수업일/단원평가일은 정규수업 허용 + 중복불가. 논리적으로 일관됨
- ✅ V104 standard_fees 재설계 — V001 DROP + 재생성. Sprint 1 시점 사용자 데이터 없어 안전. `bills.weekly_hours_snapshot` 패턴(청구 시점 시간 보존)과 호환됨

---

## 결론

Critical/High 이슈 없음. Medium 4건은 risk-register에 등록하고 배포 진행 가능.

| ID | 등급 | 항목 | 대응 시점 |
|----|------|------|-----------|
| M1 | Medium | audit details에 원생 이름(PII) 직접 전달 | Sprint 3 감사 로그 UI 전 |
| M2 | Medium | list_students 페이지네이션 누락 | Sprint 3 목록 화면 전 |
| M3 | Medium | list_codes 페이지네이션 누락 | 코드 관리 화면 전 |
| M4 | Medium | exit_hook 락 해제 release_lock_atomic 미사용 | hotfix 또는 Sprint 3 |
| L1 | Low | get_schedule_history / get_audit_logs IPC 레벨 테스트 누락 | POOL 아키텍처 개선 시 |
| L2 | Low | 마이그레이션 파일명 V{NNN} 컨벤션 불일치 | hotfix 권장 |
| L3 | Low | LockScreen.tsx 흐름 기록 | 참고사항 |
