# Sprint Retrospective — Sprint 2

> Sprint 2: 기반 도메인 백엔드 (원생 CRUD, 수업 스케줄, 표준 교습비, 코드 테이블)
> 기간: 2026-05-20 ~ 2026-05-20 (계획 대비 단기 완료)
> 브랜치: `sprint2` (develop 대비 12 commits ahead)
> 코드 리뷰: Critical 0 / High 0 / Medium 4 / Low 3

---

## 이전 회고 액션 아이템 이행 결과

Sprint 1 회고 문서가 작성되지 않아 `docs/risk-register/sprint1-risks.md`의 Medium 이슈 처리 결과를 기준으로 검토한다.

| 항목 | 이행 여부 | 비고 |
|------|-----------|------|
| R7 release_lock advisory lock 미적용 | ✅ 해소 | `release_lock_atomic()` 구현, `try_lock_exclusive` 보호 |
| R8 cipher on 실측 미수행 | ✅ 부분 해소 | timing breakdown 4필드 도입. 실측은 v0.2.0 배포 후 위임 |
| R6 salt Keychain 저장 이전 | ✅ 결정 완료 | chicken-and-egg 확인, Sprint 3 마법사 통합으로 이연 (R12) |

---

## 잘한 점

**step-back 프로토콜이 실제로 작동했다.**
T2(salt 이전)와 T8(sqlx 매크로) 모두 구현 진입 전 또는 직후에 기술적 불가 상황을 발견하고 즉시 scope를 재정의했다. T2는 chicken-and-egg 충돌(DB 안에 salt 보관 불가)을, T8은 매크로 전환 효과가 없음(현 코드에 `query!()` 사용 0건)을 사전 발견하여 불필요한 작업을 회피했다. 이연 결정을 risk-register(R12)와 sprint2.md에 명시적으로 기록하여 다음 스프린트에서 추적 가능하다.

**PI-05 자동 채번 + override 허용 패턴의 단순성과 유연성 균형.**
`MAX(CAST(serial_no AS INTEGER)) + 1` 쿼리로 숫자 채번을, `GLOB '[0-9]*'` 필터로 영문 prefix 행 제외를, `Option<String>` 파라미터로 사용자 override를 동시에 처리했다. 단일 사용자 모델이라는 제약 안에서 과도한 복잡성 없이 요구사항을 충족했다.

**`AppError::UserFacing` variant로 도메인 메시지 책임 분리.**
기존 `AppError` variant들이 `user_message()`에서 generic 메시지를 반환하는 반면, `UserFacing`은 호출자가 작성한 한국어 메시지를 그대로 노출한다. UNIQUE 위반(`serial_no`, `weekly_hours`, 코드 중복) 처리가 각 도메인 맥락에서 정확한 메시지를 제공한다.

**22개 IPC 시그니처 일관성 유지.**
모든 IPC가 `Result<T, String>` 반환 + `AppError` → `String::from` 변환을 일관 적용했다. 프론트엔드 18개 래퍼도 `!inv` 분기에 구조적으로 호환되는 mock 데이터를 제공한다.

**테스트 33건 증가.**
비즈니스 규칙(자동 채번, 매칭, 이력 생성)을 인메모리 DB로 검증하고, 경계값 시나리오(1시간 미만 매칭 없음, 영문 prefix 제외, override 후 연속성)를 모두 커버했다.

---

## 개선할 점

**sprint-planner 명세에 구현 시점에서야 확인 가능한 불가 항목이 포함되었다.**
T2와 T8 모두 계획 단계에서 기술적 제약을 완전히 파악하지 못한 채 Task로 포함되었다. T2의 chicken-and-egg는 DB 경로 설정(Sprint 3 마법사)이 확정되지 않은 상태에서 salt 이전을 계획한 것이고, T8의 무효화는 현재 코드베이스의 `query!()` 사용 현황을 사전에 확인하지 않은 것이다. 두 건 모두 sprint-planner가 data-model.md와 현재 코드 상태를 사전 검토했다면 계획 단계에서 제외할 수 있었다.

**sprint2.md SSOT와 data-model.md 간 컬럼 타입 불일치가 발생했다.**
`serial_no INTEGER`(sprint2.md) vs `TEXT`(data-model §1.1), `duration_hours REAL`(sprint2.md) vs `INTEGER`(data-model §1.2). 마이그레이션은 data-model.md를 따라 올바르게 작성되었으나, 계획 문서의 오류가 리뷰 단계까지 발견되지 않고 잔존했다. 마이그레이션 주석에 "미스 보정" 메모로 대응했지만 계획 문서 자체가 수정되지 않았다.

**마이그레이션 파일명 컨벤션이 스프린트 간에 불일치한다.**
Sprint 1: `001__`, `008__` / Sprint 2: `101__`~`105__` / backend.md 표기: `V{NNN}__`. SQLx 실행 순서에는 영향이 없으나 세 가지 표기가 혼재되어 있다. 이후 마이그레이션 관리 시 혼란 가능성이 있다.

**cipher on 환경 실측이 이번 스프린트에서 수행되지 않았다.**
R8 해소 항목으로 timing breakdown 코드가 도입되었으나 실제 PBKDF2 600K iter + SQLCipher 초기화 소요 시간은 사용자 환경에서만 측정 가능하다. v0.2.0 인스톨러 배포 전까지 3초 예산(PRD §5.6) 통과 여부를 확인할 수 없다.

**PII(원생 이름)가 audit details에 마스킹 없이 기록된다 (M1).**
`try_record(StudentCreated, Some(&serial), Some(&payload.name))`에서 원생 이름이 그대로 감사 로그에 저장된다. audit.rs 주석에 "호출자가 사전 마스킹" 책임이 명시되어 있으나 현재 호출자들이 이를 이행하지 않고 있다.

---

## 액션 아이템

**A1 (Sprint 3, 감사 로그 UI 전): audit details PII 마스킹 적용**
`students.rs`의 `try_record` 호출 3곳(`create_student`, `update_student`, `withdraw_student`)에서 `details` 파라미터에 원생 이름 대신 마스킹값(예: 이름 앞 1자 + `**`) 또는 `None`을 전달하도록 수정한다. Sprint 3 감사 로그 화면 구현 전에 완료한다.
- 대상 파일: `src-tauri/src/commands/students.rs`

**A2 (hotfix 후보, 즉시): exit_hook 락 해제를 release_lock_atomic으로 교체**
`startup.rs::exit_hook`에서 `std::fs::remove_file` 직접 호출을 `release_lock_atomic().ok()`로 교체한다. advisory lock 획득 + 본 디바이스 확인 + 삭제 순서를 보장하여 다른 디바이스 락 파일 손상 엣지 케이스를 제거한다.
- 대상 파일: `src-tauri/src/startup.rs:210~214`
- 변경 규모: 3줄 이내 (hotfix 요건 충족)

**A3 (Sprint 3 목록 화면 전): list_students / list_codes 페이지네이션 파라미터 추가**
`list_students` IPC에 `page: Option<u32>`, `limit: Option<u32>` 파라미터를 추가하고 SQL에 `LIMIT ? OFFSET ?`을 적용한다. `list_codes`도 동일하게 처리한다. Sprint 3 학생 목록 화면 구현 시 함께 작업한다.

**A4 (Sprint 3 계획 전): sprint-planner 가이드에 사전 검토 항목 추가**
sprint-planner agent가 계획 수립 시 data-model.md SSOT와 현재 코드베이스의 사용 현황(의존성, 기존 API 현황)을 사전 확인하도록 `.claude/agents/agent-memory/sprint-planner/MEMORY.md`에 체크 항목을 추가한다. 구체적으로: (1) 마이그레이션 대상 컬럼 타입은 data-model.md 기준 명시, (2) 외부 라이브러리/매크로 사용 Task는 현재 사용 현황 확인 후 포함.

**A5 (Sprint 3 진입 전): 마이그레이션 파일명 hotfix로 V{NNN} 통일**
기존 `001__`, `008__` → `V001__`, `V008__` 로 파일명 변경 및 `101__`~`105__` → `V101__`~`V105__` 로 통일한다. SQLx migrate는 알파벳/숫자 순 정렬로 실행 순서를 결정하므로, 파일명 변경 후 `sqlx migrate run` 재실행하여 순서가 유지됨을 검증한다.
- 주의: SQLx migrate 이력 테이블(`_sqlx_migrations`)이 파일명을 버전 키로 사용하므로 기존 DB에서의 변경은 사용자 데이터가 없는 현 시점에만 안전하다.

**A6 (v0.2.0 배포 직후): cipher on 환경 실측**
v0.2.0 Windows 인스톨러를 실제 교습소 PC(Windows 10/11)에 설치 후 `app_startup_sequence` 콘솔 출력(`[startup] total=...ms`)을 확인한다. `elapsed_ms > 3000`이면 `password_verify_ms`(PBKDF2 병목 후보) 중심으로 `PRAGMA cache_size` 또는 PBKDF2 iter 조정을 검토한다. 측정값을 `docs/risk-register/` 또는 Sprint 3 계획에 기록한다.
