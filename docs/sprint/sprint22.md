# Sprint Plan sprint22

## 기간
2026-07-21 ~ 2026-08-03 (2주)

## 목표
보강(makeup) 시스템을 **분(分) 단위 부분 차감** 모델로 전환하여, 결석 시간의 일부만 보강 등록해도 잔여분이 보강 대상 목록에 계속 노출되도록 한다. 기존 일 단위 매칭으로 유실된 데이터를 자동 백필로 보정한다. 부차적으로 출결 그리드 sticky z-index 깨짐을 수정한다.

## ROADMAP 연계 기능
- 보강-결석 매칭 모델 전환: 일 단위(PI-02 옵션 A) -> 분 단위 부분 차감
- 유실 데이터 자동 백필 (마이그레이션 포함)
- 출결 그리드 UI 버그 수정 (sticky header z-index)

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint21-retrospective.md`

| 항목 ID | 액션 아이템 | 이번 스프린트 반영 |
|---------|-----------|-------------------|
| A126 | MoveAttendanceDialog `yearMonth` prop -> `invalidationYm` 명확화 | T7에서 보강/출결 프론트엔드 수정 시 함께 처리 |
| A114 | sync_single_date 이력 패턴 통일 (4스프린트 이연) | **이연 유지** — 보강 잔여 계산과 무관한 별도 리팩터, 범위 외로 판단 (T4 체크리스트 8번 참조) |
| A124/A125/A115 | cipher 스모크 테스트 / 시각 QA | 배포 단계 항목 -- 스프린트 범위 외 (deploy-prod 시 수행) |

## 작업 목록

### T0: 부분 보강 스키마 설계 ADR (설계 결정) -- skill: brainstorming

**목표**: 부분 소진 상태를 DB 스키마에 어떻게 표현할지 대안 비교 후 ADR 작성.

**후보**:
- (A안) 결석 레코드에 `makeup_attended_minutes` 누적 컬럼 추가 -- 잔여 = `class_minutes - makeup_attended_minutes`
- (B안) 결석-보강 배분(allocation) 링크 테이블 (`makeup_allocations`) -- N:M 관계 표현

**고려 사항**:
- 현재 결석 -> 보강은 `regular_attendances.makeup_attendance_id` (FK) 1:N 관계
- 부분 보강 시 1결석:N보강도 가능해지므로 N:M 표현 필요 여부 검토
- 기존 `makeup_attendance_id` FK와의 호환성 / 마이그레이션 복잡도
- 쿼리 성능 (목록/요약/소멸/진단에서 잔여 계산 쿼리 복잡도)
- 백필 알고리즘 복잡도 비교
- **⚠️ 테이블 재구성 회피 우선 (배포 안전성)**: FK/CHECK 제약 변경으로 인한 테이블 재생성(CREATE new → INSERT SELECT → DROP → RENAME)은 `foreign_keys=ON` + 트랜잭션 환경에서 **deferred FK 카운터 함정(V108 code 787)**을 재현시킴. A안(`ADD COLUMN`)/B안(신규 테이블 추가)처럼 **단순 ALTER로 끝나 재구성이 불필요한 설계를 최우선**으로 선택한다. 불가피하게 부모 테이블 재생성이 필요하면 V108의 "자식 FK 값 TEMP 보존 → NULL → 부모 재구성 → 동일 id 복원" 패턴을 반드시 따르고 인덱스 재생성을 누락하지 않는다. → ADR에 이 판단을 명시.

**산출물**: `docs/arch/adr-011-partial-makeup-schema.md`

### T1: DB 마이그레이션 V311 -- 부분 보강 스키마 변경

**목표**: T0 ADR 결정에 따른 스키마 변경 마이그레이션 작성.

**파일**: `src-tauri/migrations/311__{설명}.sql`

**제약 사항**:
- 기존 데이터 무손실 마이그레이션 (ALTER TABLE / 재구성)
- FK 정합성 유지 (V107 패턴 참조)
- 인메모리 단위 테스트 필수 커버 (backend.md SQLx 규칙)

### T2: 보강 등록 로직 부분 차감 전환 (백엔드 핵심)

**목표**: `create_makeup_with_absences_impl`을 분 단위 부분 차감 모델로 전환.

**변경 파일**: `src-tauri/src/commands/makeup.rs` (328-493행)

**구현 요점**:
- 보강 시간이 결석 시간 합계보다 작을 때 부분 소진 허용
- 소진 순서: 소멸기한 임박(오래된) 순으로 보강분 배분
- 1결석의 잔여가 0이 될 때만 `makeup_done` 전이, 잔여 > 0이면 부분 소진 상태 유지
- 여러 번에 걸친 부분 보강으로 결석 점진 소진 가능
- 검증 4(PI-02 분 단위, 434-439행 주석) 활성화 및 확장

**비즈니스 규칙 단위 테스트 (100% 커버 필수)**:
- 60분 결석에 60분 보강 -> 완전 소진 (기존 동작 회귀 없음)
- 120분 결석에 60분 보강 -> 부분 소진, 잔여 60분
- 120분 결석에 60분 보강 2회 -> 완전 소진
- 60분 결석 2건에 120분 보강 -> 양쪽 완전 소진
- 보강 시간 > 결석 합계 -> 에러 또는 초과분 무시 (ADR 결정 반영)
- 소멸기한 임박 순서 배분 검증

### T3: 보강 취소 부분 차감 대응 (백엔드)

**목표**: `cancel_makeup_impl`을 부분 차감 모델에 맞게 수정.

**변경 파일**: `src-tauri/src/commands/makeup.rs` (520-559행)

**구현 요점**:
- 취소 시 해당 보강에 배분된 분만큼 결석의 누적 소진분 차감
- 완전 소진(`makeup_done`)이었던 결석이 취소로 잔여 복원되면 `absent`로 환원
- 부분 소진 중인 결석에서 해당 보강분만 정확히 제거

**단위 테스트**:
- 부분 소진 보강 취소 -> 잔여 복원
- 완전 소진 보강 취소 -> absent 환원
- 다중 보강 중 1건 취소 -> 나머지 보강 영향 없음

### T4: 보강 대상 목록/요약/소멸/진단 쿼리 일괄 변경 (백엔드)

**목표**: 부분 차감 모델에 맞게 잔여 보강필요시간 계산 쿼리를 일괄 수정.

**회귀 위험 체크리스트** (누락 없이 전수 변경):

| # | 파일 | 함수/위치 | 현재 술어 | 변경 내용 |
|---|------|----------|----------|----------|
| 1 | `calendar.rs` | 보강 대상 목록 (193-287, 특히 201-217) | `status='absent' AND makeup_attendance_id IS NULL` | 잔여분 > 0 기준으로 변경 (`HAVING remaining_minutes > 0`) |
| 2 | `attendance.rs` | 월간 요약 보강필요시간 (1002-1065) | 미매칭 결석 class_minutes 합 | 잔여분 합계로 변경 |
| 3 | `expiration.rs` | 소멸 대상 조회 (82-140) | `status='absent' AND makeup_attendance_id IS NULL` | 잔여분 > 0 + 소멸기한 경과 기준 |
| 4 | `expiration.rs` | 소멸 전이 실행 (196-205) | status -> `makeup_expired` | 부분 소진 상태의 잔여분도 소멸 대상 |
| 5 | `expiration.rs` | 퇴교 보강 처리 (259-323) | 미매칭 결석 조회 | 잔여분 > 0 기준 |
| 6 | `diagnosis.rs` | 보강필요시간 음수 탐지 (86-118) | 음수만 탐지 | 부분 보강 부족분도 감지하도록 재검토 |
| 7 | `students.rs` | 원생 상세 집계 (498행) | 미매칭 결석 카운트 | 잔여분 > 0 기준 |
| 8 | ~~`attendance.rs` sync_single_date 이력 패턴 (A114)~~ | -- | **이연 유지** — present 행 자동 생성/삭제 로직으로 보강 잔여 계산과 무관. 회고 원문 확인 필요한 별도 리팩터라 본 스프린트 범위(보강 부분차감) 외로 판단, 무리한 끼워넣기 회피 |

**단위 테스트**: 각 쿼리별 부분 소진 상태에서의 정확한 결과 검증

### T5: 유실 데이터 자동 백필 마이그레이션 (DB 마이그레이션)

**목표**: 기존 일 단위 매칭으로 인한 잔여분 유실 데이터를 자동 보정.

**파일**: `src-tauri/migrations/312__backfill_partial_makeup.sql` (또는 Rust 코드 마이그레이션) + `src-tauri/src/startup.rs` (사전 스냅샷 백업 로직)

**알고리즘** (사용자 확정):
1. 각 보강의 `class_minutes` vs 매칭된 결석의 `class_minutes` 합 비교
2. 보강분 < 결석분 (부분 보강): 소멸기한 임박(오래된) 순으로 보강분 배분, 잔여분 복원
3. 보강분 >= 결석분 (정상/초과): 완전 소진 유지, 초과분 버림
4. `makeup_expired` 결석은 보정 대상 제외 (원장 판단 존중)

**⚠️ 사전 스냅샷 백업 (필수 안전장치)**:
- **문제**: 정상 경로(무결성 quick_check Ok)에서는 마이그레이션 직전에 백업이 생성되지 않고, 실사용 암호화 `app.db`에 인플레이스로 바로 적용됨(`startup.rs:213` `db::initialize`). 백필이 **커밋에는 성공했으나 계산 오류로 데이터를 오염**시키면 트랜잭션 롤백으로 못 잡고, 복구 수단은 이전 세션 종료 시 exit 백업(`backup/exit/`, 최대 5개)뿐.
- **대응**: `startup.rs`의 startup 시퀀스에서 **무결성 검사 통과 후 ~ `db::initialize`(마이그레이션) 직전** 위치에 사전 스냅샷 백업을 1회 생성한다. 미적용 마이그레이션이 존재할 때만 뜨도록(`_sqlx_migrations` 최신 버전 대조) 하거나 최소 회전 정책을 따른다. cipher-off 개발 빌드에서는 백업이 stub이므로 no-op 허용.

**제약**:
- 멱등(idempotent) 설계 -- 재실행해도 결과 동일 (각 문장 단위 멱등 권장 — 파일 후반 실패 후 재시도 대비)
- 개발 PC DB는 테스트용이라 실데이터 사전 확인 불가
- 인메모리 단위 테스트 필수 커버 (단, FK/실데이터 패턴은 인메모리로 못 잡음 → T9 cipher-on 스모크로 보완)
- **완전 자동·무알림 (사용자 확정 2026-07-21)**: 앱 첫 실행 시 조용히 자동 보정한다. 원장(최종 사용자)에게 UI 안내/토스트/다이얼로그/리포트를 **일절 표시하지 않는다**. 보정 건수는 개발/추적 목적으로 `audit_logs`에만 기록(UI 비노출) — 사후 문제 발생 시 검증용.

**단위 테스트**:
- 부분 보강 보정 (120분 결석 + 60분 보강 -> 잔여 60분 복원)
- 정상 매칭 보정 (변경 없음)
- makeup_expired 제외 확인
- 멱등성 확인 (2회 실행 결과 동일)

### T6: (취소됨) 백필 결과 안내 UI

> **취소 사유 (2026-07-21 사용자 확정)**: 백필을 완전 자동·무알림으로 진행하기로 결정. 원장에게 어떤 UI 안내도 하지 않으므로 안내 UI·`get_backfill_result` IPC 모두 불필요. 보정 건수는 T5에서 `audit_logs`에만 기록.

### T7: 보강 등록 UI 1시간 단위 선택 전환 (프론트엔드)

**목표**: `MakeupRegisterDialog`의 보강 시간 입력을 1시간(60분) 단위 선택으로 변경.

**변경 파일**: `src/components/attendance/MakeupRegisterDialog.tsx` (224-233행 시간 편집, 100-120행 자동합산)

**구현**:
- 시간 입력: 1h / 2h / 3h 드롭다운 또는 라디오 버튼 (60분 단위)
- 자동합산 로직: 선택한 결석의 잔여분 합계 표시
- 보강 시간 <= 잔여분 합계 검증
- 잔여분 표시: 각 결석 항목 옆에 "(잔여 60분)" 등 표기
- A126 반영: `yearMonth` prop -> `invalidationYm`으로 명확화 (MoveAttendanceDialog)

### T8: 출결 그리드 sticky z-index 수정 (프론트엔드)

**목표**: 상하 스크롤 시 thead 헤더가 tbody 셀에 의해 덮이는 z-index 버그 수정.

**변경 파일**: `src/components/attendance/AttendanceGrid.tsx`

**원인**: thead `sticky top-0 z-10` (329행)과 tbody 좌측 고정 셀 `sticky left-* z-10` (590-633행)의 z-index 동일.

**수정**:
- z-index 층위 재정렬:
  - 코너 셀 (좌상단): `z-30` (최상위)
  - thead 헤더: `z-20`
  - tbody 좌측 고정 셀: `z-10`
  - 일반 셀: z-index 없음
- `border-collapse` (328행) + sticky 조합의 WebView 테두리 렌더링 이슈 점검 -> 필요 시 `border-separate border-spacing-0`으로 전환

**단위 테스트**: CSS 변경이므로 시각 검증 (cargo test 해당 없음)

### T9: 통합 검증

**자동 검증**:
- ✅ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (457 pass, 4 ignored)
- ✅ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ✅ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과 (cipher 빌드 컴파일 정상)
- ✅ `pnpm lint` clean
- ✅ `pnpm tsc --noEmit` clean
- ✅ `pnpm build` static export 성공
- ✅ 마이그레이션 self-check: V311+V312 적용 확인 (인메모리 test_pool 이 전체 마이그레이션 적용 후 456+ 테스트 통과 — 스키마·백필 정합)
- ⏸️ **cipher-on 백필 스모크** — 배포 단계(deploy-prod) 실 DB/암호화 빌드에서 수행. 백필 로직 자체는 인메모리 5건 테스트(원장님 케이스 포함)로 검증됨
- ⏸️ **사전 스냅샷 생성 확인 (cipher-on)** — 배포 단계에서 확인. 로직은 cipher-off 컴파일·clippy 통과

**시각 검증** (개발 서버 `pnpm tauri:dev` — 사용자 확인 필요):
- ⬜ 보강 등록: 1시간 단위 선택 UI 동작 + 잔여분('잔여 N시간(총 M시간)') 표시
- ⬜ 부분 보강 후 보강 대상 목록에 잔여분 노출 확인 (원장님 버그 재현 케이스)
- ⬜ 2회 부분 보강으로 완전 소진 확인
- ⬜ 보강 취소 후 잔여분 복원 확인
- ⬜ 출결 그리드 상하 스크롤 시 헤더 고정 정상 동작 (2번 버그)
- ⬜ 출결 그리드 좌우 스크롤 시 좌측 컬럼 고정 정상 동작 (회귀 없음)

## 완료 기준 (Definition of Done)

**필수**
- ✅ ADR-011 부분 보강 스키마 설계 결정 완료 (Accepted)
- ✅ V311 마이그레이션 정상 적용 (makeup_allocations 신규 테이블)
- ✅ V312 백필 마이그레이션 정상 적용 + 멱등성 확인 (테스트 5건)
- ✅ 마이그레이션 직전 사전 스냅샷 백업 안전장치 구현 (cipher-on 컴파일 통과, 실 DB 스모크는 배포 단계)
- ✅ 부분 보강 비즈니스 규칙 단위 테스트 커버 (등록/취소/목록/요약/소멸/진단 — makeup 36 + diagnosis 31 등)
- ✅ 회귀 위험 체크리스트 전수 변경 + 테스트 통과 (calendar/attendance/expiration/makeup/diagnosis; students 재원로직은 변경 불필요 판정)
- ✅ 출결 그리드 z-index 수정 (시각 검증은 사용자 확인 대기)
- ✅ cargo test 전체 통과 (457 pass)
- ✅ cargo clippy --all-targets -D warnings clean
- ✅ pnpm build 성공 (Next.js static export)
- ⏸️ A114 sync_single_date 이력 패턴 통일 — 이연 유지 (범위 외, 별도 리팩터)
- ✅ A126 MoveAttendanceDialog prop 명확화

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

## Capacity 확인

- 팀: AI 페어 프로그래밍 1인 개발
- 스프린트: 2주 (10 영업일)
- 실작업 가능: 하루 4시간 = 40시간
- 작업 추정:
  - T0 (ADR): ~2h
  - T1 (마이그레이션): ~3h
  - T2 (등록 로직): ~6h (핵심, 테스트 포함)
  - T3 (취소 로직): ~3h
  - T4 (쿼리 일괄 변경): ~8h (8개 파일, 가장 큰 회귀 위험)
  - T5 (백필 + 사전 스냅샷 안전장치): ~5h (알고리즘 + 테스트)
  - T6 (취소됨): 0h
  - T7 (보강 UI): ~3h
  - T8 (z-index): ~1h
  - T9 (통합 검증): ~3h
- **총 추정: ~34h** (Capacity 40h 이내)

## 참고 사항

### 의존성
- T1은 T0(ADR 결정)에 의존
- T2/T3/T4/T5는 T1(스키마 변경)에 의존
- T7은 T2(등록 로직)에 의존
- T8은 독립 (다른 작업과 병행 가능)
- T9은 모든 작업 완료 후

### 리스크
- **R139**: 부분 소진 모델로 전환 시 기존 보강 매칭 쿼리 8개 파일 일괄 변경 필요 -- 누락 시 잔여분 계산 불일치로 보강 대상 목록 오류
- **R140**: 백필 마이그레이션이 실 DB에서 커밋 성공 후 계산 오류로 데이터 오염 시 트랜잭션 롤백으로 복구 불가 -- T5 **사전 스냅샷 백업**(마이그레이션 직전) + 멱등 설계 + T9 cipher-on 스모크 테스트로 완화
- **R141**: 부분 소진 상태 표현이 기존 `makeup_done`/`absent` 2값 모델과 호환되지 않을 수 있음 -- ADR에서 명시적 결정
- **R142**: 스키마 설계가 테이블 재구성(FK/CHECK 변경)을 요구하면 deferred FK 카운터 함정(V108 code 787) 재발 -- T0 ADR에서 재구성 불필요 설계 우선, 불가피 시 V108 NULL 복원 패턴 강제
- **R143**: 기존 마이그레이션 파일(V001~V310) 수정 시 sqlx 체크섬 불일치로 전 실사용 PC startup 실패 -- 신규 변경은 반드시 새 파일(V311+)로만 추가

### 백엔드-프론트엔드 상수 쌍
- 이번 스프린트에서 새로운 상수 쌍 도입 시 `harness-engineering.md` 표에 추가
