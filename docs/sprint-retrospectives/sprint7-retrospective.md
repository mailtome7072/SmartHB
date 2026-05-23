# Sprint Retrospective — Sprint 7

> Sprint 7: Phase 2 carry-over 해소 + 인프라 안정화 — 교습기간 UX 재설계 / Keychain 최적화 / salt.bin 이전 / device_id 영속화 / 배치 제약 강화
> 기간: 2026-05-22 (본 작업 1일, 9 세션) + 2026-05-23 시각 검증 후속 fix 38건
> 브랜치: `sprint7` → develop 머지 완료 (--no-ff, 61e7bc3), 이후 post-review fix 14커밋 직접 develop 패치
> 초기 코드 리뷰 (2026-05-22): Critical 0 / High 0 / Medium 1 / Low 0
> post-review 코드 리뷰 (2026-05-23): Critical 0 / High 0 / Medium 3 / Low 1
> T2 별도 high-effort code review (S-T2-1~6 패치 동행): Critical 1 포함 전수 보안 처리
> 사용자 시각 검증 최종 결과 (2026-05-23): "모든 기능 정상 동작 확인"

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint6-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A23 | `ScheduleEventListItem`에 `is_system_reserved` 필드 추가 | ✅ 완료 | T4에서 `list_schedule_events` JOIN 확장 + `CalendarCell.tsx`/`ThreeMonthCalendar.tsx` 플래그 기반으로 변경. R33 해소 |
| A24 | `.claude/skills/`에 `brainstorming.md`, `frontend-design.md`, `simplify.md` 추가 | ⏸️ 미처리 | Sprint 7 scope에서 제외. 기존 inline 재현으로 운영 가능 수준. 별도 spare-time 작업 |
| A25 | 교습기간 외 드롭 UI 경고 | ✅ 완료 | T7에서 `check_placement_constraints` 헬퍼 도입 — 교습기간 내 배치만 허용. R34 해소 |
| A26 | 2027년 공휴일 V401 마이그레이션 | ⏸️ 이연 | 2026-12 이후 시점 작업. 스코프 외 |
| A27 | salt.bin 이전 (Keychain → cloud/smarthb/) | ✅ 완료 | T2에서 처리 + S-T2-1~6 보안 패치 동행 |

---

## 잘한 점

**1일(9 세션)에 8건 carry-over를 전수 해소했다.**

Sprint 6 시각 검증에서 발견된 Issues 1~8을 모두 처리했다. T1(Keychain 캐싱), T2(salt.bin 이전), T3(device_id 영속화), T4(is_system_reserved JOIN), T5(코드 관리 /settings 이전), T6(교습기간 UX 재설계), T7+T9(배치 제약 강화 + 공휴일 차단), T8(cascade 삭제) — 10개 태스크, 9 세션, 1일 완결. Sprint 6의 16커밋에 비해 Sprint 7은 8커밋으로 집중도가 높았다.

**T2 high-effort code review가 보안 취약점 6건을 조기에 발견·패치했다.**

T2(salt.bin 이전) 코드 리뷰를 별도 Session #2에서 high-effort 모드로 수행하여 Critical 1건(S-T2-1: 인증 전후 암호화 키 교체 경로 누락), High 3건(S-T2-2~4), Medium 2건(S-T2-5~6)을 발견하고 동행 패치로 즉시 반영했다. 보안 관련 코드는 구현 직후 별도 고강도 리뷰를 수행하는 패턴이 효과적임을 확인했다.

**NTFS power-loss 방어 패턴이 일관되게 적용됐다.**

T2(salt.bin write), T3(device.id write), T3(lock file write) 모두 tmp파일 → `sync_all()` → `rename()` 원자적 쓰기 패턴을 적용했다. `.claude/memory/ntfs-power-loss-pattern.md`에 기록된 패턴을 재사용하여 별도 설계 비용 없이 방어 로직이 완성됐다. 메모리 미러의 실용성이 입증된 사례다.

**`parse_lock_info` 소프트 폴백 변경이 Phase 2 전체 안정성을 높였다.**

기존의 잘못된 JSON 파싱 시 `Err` 반환에서 `Ok(None)` 폴백으로 전환하여, NTFS 손상 시나리오에서 앱이 hard crash 없이 lock을 재취득하도록 했다. 이 변경 하나가 양 PC 동기화 시나리오(OneDrive/iCloud 동기화 중 중간 파일 상태)에서의 비정상 종료 위험을 원천 차단한다.

**Sprint 6 회고 액션 아이템(A23, A25, A27) 3건이 모두 이행됐다.**

3회 이월된 A27(salt.bin 이전)을 포함하여 Sprint 6에서 확인된 기술 부채 3건이 Sprint 7에서 전수 처리됐다. Sprint 간 액션 아이템 이행률 100% (적용 가능한 3건 기준).

---

## 시각 검증 후속 fix (2026-05-23, V1~V37b)

Sprint 7 본 작업 완료(61e7bc3) 후 사용자 시각 검증 과정에서 38건(V1~V37b)의 추가 fix가 필요했다. 15커밋으로 develop 직접 패치.

**카테고리별 요약**:

| 카테고리 | 건수 | 대표 항목 |
|----------|------|-----------|
| Critical 버그 fix | 5 | V1(year_month 저장 오류), V7(cross-month 표시), V9(공휴수업일 룰), V18(윈도우 상태), V32(블러 룰) |
| 백엔드 가드 강화 | 4 | V12(selection 중첩 차단), V20(사이 일자 매핑), V26(범위 충돌 검사), V29(보강데이 가드) |
| 데이터 모델 | 3 | V14(단원평가 라인 제거), V16+V302(is_seeded 마이그레이션) |
| UI 재설계 | 16 | V3·V8·V10·V11(통합 컨트롤 바), V13(S/E 마커), V15(확대 모달), V22·V23·V25·V27·V28(수업 가능 배경·배지 클릭), V34·V35·V36·V37b(TopBar·비밀번호 UI) |
| dev 편의 | 1 | V30(자동 로그인 우회) |
| 보안 UX | 1 | V37b(비밀번호 입력 모드 배지) |

**Lessons Learned**:
1. **시각 검증은 자동 검증을 대체한다.** 자동 검증이 모두 통과했음에도 실제 앱에서는 38건의 시각적/UX 이슈가 발견됐다. 학사 캘린더처럼 복잡한 상태를 다루는 UI는 단위 테스트로 커버하기 어려운 시각적 오류가 많다.
2. **1일 9세션 후 추가 38건은 초기 구현 밀도가 높아서 발생한 패턴이다.** 9세션에 8개 carry-over를 압축 처리한 결과 세부 UX 품질보다 기능 완성에 집중했다. 1세션당 1~2개 Task로 범위를 제한하면 초기 품질이 올라가고 post-review fix 건수가 줄어들 수 있다.
3. **사용자 시각 검증을 sprint-review 직후 별도 세션으로 명시 예약하는 것이 효율적이다.** V1~V37b fix가 1일 내에 이루어진 것은 사용자가 즉각 피드백을 주었기 때문이다. 다음 스프린트에서는 "sprint-review 완료 → 사용자 시각 검증 세션 (1시간) → post-review fix" 패턴을 명시적으로 계획에 포함한다.

---

## 개선할 점

**T2 high-effort code review carry-over 10건이 다음 스프린트로 이월됐다.**

Session #2에서 발견된 잔여 이슈(I-S2-2~10) 9건과 Session #4에서 발견된 I-S4-1 1건이 리스크 레지스터(R40~R49)에 등록됐다. 특히 R40(is_salt_corrupted partial-NULL 미감지), R41(set_password 재진입 가드 없음), R42(CRED_CACHE static Drop 미보장), R43(check_auth_status 마이그레이션 미트리거), R44(테스트가 실제 Keychain 삭제 가능) 5건이 High 등급이다. Keychain/salt 보안 경로는 단위 테스트 격리가 어려워 carry-over가 누적되는 구조적 문제가 있다.

**StudyPeriodEditor T6의 create+confirm 2단계 원자성 결여 (R39).**

`createStudyPeriod` 성공 후 `confirmStudyPeriod` 실패 시 미확정 교습기간이 DB에 잔존한다. 재시도 시 `create_study_period`의 overlap 검사(is_confirmed 필터 없음)에 의해 "중첩됩니다" 오류로 차단된다. 원인은 두 IPC를 프론트엔드에서 순차 호출하는 설계이며, 백엔드 단일 트랜잭션 IPC로 통합하거나 overlap 검사에 `AND is_confirmed = 1` 조건을 추가하면 해소된다.

**Flaky lock 테스트가 병렬 실행 환경에서 간헐적으로 실패한다.**

`release_lock_atomic_removes_self_owned_lock` 테스트가 `lock_path()` → `paths::data_root()` 의존으로 process-wide 전역 경로를 공유한다. `cargo test --lib` 병렬 실행 중 다른 테스트와 lock 파일 경합이 발생할 수 있다. 단일 인스턴스 프로덕션 환경에서는 문제가 없으나, CI 녹색 상태를 신뢰하려면 테스트 격리가 필요하다.

**Keychain 보안 코드의 테스트 격리 부재가 carry-over 누적의 구조적 원인이다.**

T2 관련 이슈 6건 모두 `paths::salt_path()`, `keyring::Entry` 등이 전역 상태에 직접 의존하여 단위 테스트에서 실환경을 건드리는 문제가 근본 원인이다. 의존성 주입(DI) 패턴 또는 trait-based mock 구조로 전환해야 한다.

---

## 이번 스프린트에서 배운 점

1. **보안 코드는 구현 즉시 별도 high-effort 리뷰를 수행해야 한다.** T2에서 Critical + High 4건이 동행 패치로 해결됐다. 이 패턴을 Keychain/암호화 관련 코드 변경 시 표준 절차로 정립할 필요가 있다.

2. **`OnceLock<Mutex<Option<...>>>` 패턴의 static lifetime 함정을 확인했다.** process exit 시 static 객체의 Drop이 미보장되는 환경이 있어 ZeroizeOnDrop 주석이 "자동 폐기"를 보장하지 못할 수 있다(R42). shutdown hook에서 명시적 무효화 호출이 필요하다.

3. **T2의 고강도 리뷰가 I-S2 carry-over를 만들었고, 그 carry-over가 Sprint 7 범위를 초과했다.** 발견된 이슈 중 현 스프린트 내에서 전수 처리할 수 없었다. 보안 이슈는 발견 즉시 처리하는 것이 이상적이지만, Sprint scope와 일정 제약 사이에서 trade-off가 발생한다. High 등급 이슈는 다음 스프린트 첫 번째 작업으로 예약하는 규칙이 필요하다.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 담당 |
|----|------|---------|------|
| A28 | `create_study_period`의 overlap 검사에 `AND is_confirmed = 1` 추가 — 또는 `create+confirm` 단일 원자 IPC 구현 (R39 해소) | 높음 | Sprint 8 hotfix 후보 |
| A29 | R40(`is_salt_corrupted` partial-NULL 감지), R41(`set_password` 재진입 가드), R42(shutdown hook 명시적 cache 무효화), R43(`check_auth_status` 마이그레이션 트리거) — High 4건 우선 처리 | 높음 | Sprint 8 우선 태스크 |
| A30 | R44 테스트 격리: `load_salt_backs_up_corrupted_file`에 `#[ignore]` 마킹 또는 mock salt_path 주입 패턴 도입 | 높음 | Sprint 8 |
| A31 | lock 테스트 격리 강화: `release_lock_atomic_removes_self_owned_lock`에 temp dir 기반 격리 적용 (R40 carry-over I-S2-2 연관) | 중간 | Sprint 8 |
| A32 | R46(Mutex poison 영구 brick 방지) — `lock().ok()` 또는 `unwrap_or_else` 패턴으로 교체 | 중간 | Sprint 8 |
| A33 | R47(migrate audit 누락) — `migrate_keyring_salt_to`에 `AuditEventType::SecurityEvent` 기록 추가 | 중간 | Sprint 8 |
| A34 | A24 이연 항목: `.claude/skills/`에 `brainstorming.md`, `frontend-design.md`, `simplify.md` 추가 | 낮음 | Sprint 8 진입 전 spare-time |
| A35 | 2027년 공휴일 V401 마이그레이션 작성 (매년 1월) | 낮음 | 2027년 1월 |
| A36 | R50 대응: `LockScreen.tsx:45` 주석 정정 + 릴리즈 빌드 전 체크리스트에 `NEXT_PUBLIC_DEV_AUTOLOGIN` 미설정 확인 추가 | 중간 | Sprint 8 또는 deploy-prod 시 hotfix 검토 |
| A37 | R51 UX 개선: `studyPeriodMode` 활성 중 배지 클릭 시 삭제 다이얼로그 표시 여부 사용자 피드백 수집 후 결정 | 낮음 | Sprint 8 사용자 피드백 후 |
| A38 | 시각 검증을 sprint 계획에 명시 예약: "sprint-review 완료 → 사용자 시각 검증 (1시간) → post-review fix" 패턴 표준화 | 중간 | Sprint 8부터 적용 |

---

## 정량 데이터

| 항목 | 수치 |
|------|------|
| 세션 수 | 9 (본 작업) + 시각 검증 세션 8차 |
| 커밋 수 | 8 (T1~T10) + 15 (post-review fix, 2bd1a90~d832c0b) |
| 주요 변경 파일 (최종) | ~44 (본 작업 ~15 + post-review 29) |
| 신규 IPC 커맨드 | 2 (`get_cascade_delete_preview`, `delete_study_period_cascade`) |
| 신규 TS IPC 래퍼 | 2 (`getCascadeDeletePreview`, `deleteStudyPeriodCascade`) |
| 보안 패치 (T2 high-effort) | 6건 (S-T2-1~6) |
| post-review fix | 38건 (V1~V37b) |
| carry-over 리스크 등록 | 15건 (R39~R53) |
| 백엔드 테스트 총합 | 187 (cipher off) / 127 (cipher on) — 전수 통과 |
| flaky 테스트 | 1건 (`release_lock_atomic_removes_self_ordered_lock`) |
| 해소된 이전 회고 액션 | 3건 (A23, A25, A27) |
| 미이행 이전 회고 액션 | 1건 (A24 — 이연) |
| 사용자 시각 검증 결과 | "모든 기능 정상 동작 확인" (2026-05-23) |

---

## 코드 리뷰 요약

### 초기 리뷰 (2026-05-22, T1/T3~T10 대상)

**Critical**: 0건 / **High**: 0건 / **Medium**: 1건 / **Low**: 0건

| ID | 위치 | 내용 |
|----|------|------|
| M-S7-01 | `StudyPeriodEditor.tsx:104` | `createStudyPeriod` 성공 + `confirmStudyPeriod` 실패 시 미확정 교습기간 잔존 → overlap 검사로 재시도 차단. R39 등록. |

### T2 high-effort code review 요약 (Session #2, 별도 수행)

| 등급 | 건수 | 처리 결과 |
|------|------|---------|
| Critical | 1 | S-T2-1 동행 패치 완료 |
| High | 3 | S-T2-2~4 동행 패치 완료 |
| Medium | 2 | S-T2-5~6 동행 패치 완료 |
| carry-over | 9 | I-S2-2~10 → R40~R48 리스크 등록 |

### 검토 확인 항목 (이상 없음, T1/T3~T10 대상)

- SQL 바인드 파라미터 — `check_placement_constraints`, `delete_study_period_cascade` 등 신규 쿼리 전체 `query!().bind()` 사용. raw concat 없음
- `invoke()` 직접 호출 — 0건. 신규 래퍼 2개 모두 `src/lib/tauri/index.ts` 경유
- `dangerouslySetInnerHTML` — 0건
- localStorage 민감 정보 — 0건
- TypeScript `any` 남용 — 0건
- SSR 가드 — `getCascadeDeletePreview`, `deleteStudyPeriodCascade` 래퍼 모두 `getInvoke()` 경유, `typeof window` 처리 포함
- `check_placement_constraints` — 교습기간 범위 체크 SQL: `WHERE is_confirmed = 1 AND start_date <= ? AND end_date >= ?` 정확한 경계 검사
- cascade delete 트랜잭션 — `begin() → DELETE schedule_events → DELETE study_periods → commit()` 명시적 트랜잭션, 공휴일 보존 조건(`code_name != '공휴일'`) 적용
- CHANGELOG.md `[Unreleased]` — Sprint 7 본 작업 + V1~V37b 전수 기재 완료 (2026-05-23 갱신)
- 하드코딩 시크릿 — 변경 파일 전수 스캔 결과 없음

### post-review 리뷰 (2026-05-23, V1~V37b 대상)

**Critical**: 0건 / **High**: 0건 / **Medium**: 3건 / **Low**: 1건

| ID | 위치 | 내용 |
|----|------|------|
| M-S7-02 | `LockScreen.tsx:45` | `NEXT_PUBLIC_DEV_AUTOLOGIN` 주석 오류 — NEXT_PUBLIC_ 변수는 빌드 타임 인라인됨. R50 등록. |
| M-S7-03 | `academic/page.tsx:245` | 교습기간 선택 모드 중 배지 클릭 시 삭제 다이얼로그 표시 (의도된 변경이지만 UX 혼란 가능). R51 등록. |
| M-S7-04 | `302__add_is_seeded.sql:31` | V302 UPDATE가 사용자 추가 공휴일도 시드로 마킹 가능 (pre-release 가정 위반 시). R52 등록. |
| L-S7-01 | `academic.rs:801` | `has_no_class_blocker` 변수명이 `.any()` 반환값과 의미 역전 — 로직은 정확하지만 유지보수 혼동 위험. |

### 검토 확인 항목 (이상 없음, V1~V37b 대상)

- SQL 바인드 파라미터 — V26/V29 신규 쿼리 모두 `.bind()` 사용. raw concat 없음
- `invoke()` 직접 호출 — 0건. 신규 `getOperatingHours`, `listStudyPeriods` 호출 모두 래퍼 경유
- `dangerouslySetInnerHTML` — 0건
- localStorage 민감 정보 — 0건 (NEXT_PUBLIC_DEV_AUTOLOGIN은 .env.example 주석 처리, 실제 저장 없음)
- TypeScript `any` 남용 — 0건
- `tauri-plugin-window-state` 권한 — `capabilities/default.json`에 `window-state:default` 명시됨
- V302 마이그레이션 — `ALTER TABLE` + `UPDATE` 순서 정상, `is_seeded` CHECK (0/1) 적용
- 하드코딩 시크릿 — `*.rs`, `*.ts`, `*.tsx` 변경 파일 전수 스캔 결과 없음
