# Sprint Plan sprint5

## 기간
2026-05-22 ~ 2026-06-05 (2주, 예상)

## 상태
✅ 완료 (2026-05-22) — T0~T5 전수 통과, 자동+수동 검증 완료
develop 머지 커밋: `e717a78` (sprint5 → develop, --no-ff, 2026-05-22)

## 목표
Phase 1.5b 안정화 -- 스테이징 검증(2026-05-22, Mac)에서 발견된 **환경 호환 이슈 1건 + 앱 사용 불가 이슈 1건 + UX 이슈 2건 + 시드 데이터 변경 2건**을 해결하여 Phase 2(학사 스케줄) 진입 전 안정성을 확보한다. 핵심은 Node 25 호환성 보장, `tauri-plugin-single-instance`를 통한 동일 PC 다중 인스턴스 원천 차단, 마법사 완료 후 UX 흐름 교정, 교습비/결제수단 시드 데이터를 실제 운영 값으로 보정하는 것이다.

## ROADMAP 연계 기능
- Phase 1.5 품질 안정화 연속 -- Sprint 4 이후 추가 스테이징 검증에서 발견된 안정성 이슈
- §5.3 app.lock 동시성 제어 -- 동일 PC 다중 인스턴스 차단 (tauri-plugin-single-instance)
- §4.0 초기 설정 마법사 -- 완료 후 redirect 경로 교정
- §4.12 코드 테이블 -- 표준교습비/결제수단 시드 데이터 운영 값 반영

> **참고**: ROADMAP.md의 기존 Sprint 5(학사 스케줄)는 Sprint 6으로 이연된다. 본 Sprint 5는 Phase 1.5b 안정화 sprint이다. ROADMAP 번호 이동은 sprint-close 단계에서 일괄 처리한다.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint4-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 방법 |
|---------|------|----------------------|
| A14 | `paths::tests::init_from_config_ignores_empty_path` flaky 테스트 격리 강화 | 범위 외 (OnceLock 격리 문제, Sprint 5 scope과 무관) |
| A15 | DnD 필터링 sort_order 충돌 해소 (R26) | 범위 외 (코드 테이블 DnD 관련, 본 sprint에서 codes 수정 없음) |
| A16 | Next.js CVE-2025-66478 영향 분석 및 업그레이드 | **T0에서 함께 검토** -- Node 25 호환 수정 시 Next.js 패치 버전 확인. 단, 데스크톱 앱 특성상 외부 요청 미수신이므로 실제 위험도 낮음. 영향 분석 결과를 T0 AC에 포함 |
| A17 | salt.bin 이전 (Keychain -> cloud/smarthb/) | 범위 외 (별도 sprint 또는 hotfix, 현재 사용자 데이터 없는 시점) |
| A18 | simplify 스킬 기준 "사용처 2곳 이상 시 추출 권고" 명시 | 범위 외 (메타 작업) |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-05-21.md`

| 리스크 ID | 항목 | 이번 스프린트 반영 |
|-----------|------|------------------|
| R26 | DnD 필터링 sort_order 충돌 | 범위 외 -- 본 sprint에서 codes 관련 로직 미수정 |
| R18 | salt.bin 이전 마이그레이션 경로 | A17과 동일 -- 범위 외 |

---

## 작업 목록

### T0: Node 25/20 cross-OS 환경 호환 + CVE-2025-66478 영향 분석
> **배경**: Node 25가 기본 활성화하는 webstorage stub으로 인해 Next.js Dev Overlay의 `localStorage.getItem` 호출이 SSR에서 실패 -> `/` 페이지 500 에러 (macOS 환경 확인)
> **신규 의존성**: `cross-env` (devDependency) -- 사용자 허가 완료

**변경 사항**:
- `package.json`:
  - `devDependencies`에 `cross-env` 추가 (`pnpm add -D cross-env`)
  - `"dev"` 스크립트를 `"cross-env NODE_OPTIONS=--no-experimental-webstorage next dev -p 1420"`으로 변경
- A16 영향 분석: Next.js CVE-2025-66478이 SmartHB에 적용되는지 확인 (Tauri WebView 내 정적 파일 서빙 구조, 외부 네트워크 요청 미수신)

**예상 변경 파일**: `package.json` (1파일)
**예상 소요**: 1시간
**AC (Acceptance Criteria)**:
- AC-T0-1: Node 25 환경에서 `pnpm dev` 실행 시 `/` 페이지 500 에러 없이 정상 렌더링
- AC-T0-2: Node 20 환경에서도 `pnpm dev` 정상 동작 (cross-env가 `--no-experimental-webstorage` 옵션을 무해하게 전달)
- AC-T0-3: CVE-2025-66478 영향 분석 결과를 sprint5/scope.md에 기록 (위험도 판정 + 조치 여부)

---

### T1: tauri-plugin-single-instance 도입 + 동일 PC 다중 인스턴스 차단
> **배경**: 동일 PC에서 dev 서버 두 번 기동 시 두 번째 인스턴스가 자신의 락을 외부 디바이스로 오인 -> "다른 PC 사용 중" 메시지 표시, 잠금해제 버튼 무반응. PRD §5.3 본래 의도는 양 PC 간 시점 분리이므로 동일 PC 다중 인스턴스를 원천 차단하는 것이 올바른 해결책.
> **신규 의존성**: `tauri-plugin-single-instance` (Rust crate) + `@tauri-apps/plugin-single-instance` (JS) -- 사용자 허가 완료

**백엔드**:
- `src-tauri/Cargo.toml` -- `tauri-plugin-single-instance` 의존성 추가
- `src-tauri/src/lib.rs` -- `.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| { ... }))` 등록
  - 두 번째 인스턴스 감지 시: 기존 창 포커스 + 새 프로세스 즉시 종료
- `src-tauri/capabilities/default.json` -- `single-instance:default` 권한 추가 (필요한 경우)

**프론트엔드**:
- `package.json` -- `@tauri-apps/plugin-single-instance` devDependency 추가

**예상 변경 파일**: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`, `package.json` (4파일)
**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- AC-T1-1: `pnpm tauri:dev` 실행 중 두 번째 `pnpm tauri:dev` 실행 시 기존 창이 포커스되고 두 번째 프로세스 즉시 종료
- AC-T1-2: 첫 번째 인스턴스가 정상 종료된 후 재실행 시 정상 기동
- AC-T1-3: cargo test 전체 통과 (single-instance 플러그인 추가 후)
- AC-T1-4: cargo clippy -- -D warnings 통과

---

### T1-sub: 양 PC 간 강제 점유 버튼 동작 검증 및 수정
> **배경**: T1에서 동일 PC 다중 인스턴스를 차단한 후, 양 PC 간 실 시나리오에서 락 점유 화면의 "강제 점유" 버튼 onClick 동작이 정상인지 검증하고 필요 시 수정

**프론트엔드**:
- `src/app/lock/page.tsx` -- 락 점유 화면에서 "강제 점유" 버튼 클릭 시 `force_acquire_lock` IPC 호출 동작 확인
- 강제 점유 후 정상적으로 잠금 해제 -> 메인 화면 진입 흐름 확인

**백엔드** (필요 시):
- `src-tauri/src/commands/` 내 lock 관련 모듈 -- `force_acquire_lock` IPC 동작 확인 및 수정

**예상 변경 파일**: `src/app/lock/page.tsx`, lock 관련 백엔드 모듈 (최대 3파일)
**예상 소요**: 2시간
**skill**: systematic-debugging
**AC (Acceptance Criteria)**:
- AC-T1s-1: 다른 디바이스가 락을 점유 중일 때 "강제 점유" 버튼 클릭 시 락 교체 성공 + 잠금 해제 진행
- AC-T1s-2: 강제 점유 후 기존 디바이스 재실행 시 "다른 PC 사용 중" 메시지 정상 표시

---

### T2: 마법사 완료 redirect 수정
> **배경**: 폴더 설정 -> 비밀번호 설정 -> 마법사 완료 후 메인 화면(`/`)으로 이동하는데, 사용자 의도는 설정 화면(`/settings`)으로 먼저 진입하여 교습소 운영 시간/코드 테이블 등을 확인하는 것

**프론트엔드**:
- `src/app/setup/page.tsx` -- `handleComplete` 내 `router.replace('/')` -> `router.replace('/settings')` 변경

**예상 변경 파일**: `src/app/setup/page.tsx` (1파일)
**예상 소요**: 30분
**AC (Acceptance Criteria)**:
- AC-T2-1: 마법사 4단계 완료 후 `/settings` 화면으로 이동 (이전: `/`)
- AC-T2-2: 설정 화면에서 모든 설정 메뉴(운영 시간, 코드 테이블 등) 정상 접근 가능
- AC-T2-3: 마법사 완료 이력(`completeSetup`)이 정상 저장되어 이후 앱 재시작 시 마법사 미재진입

---

### T3: 표준교습비 시드 데이터 변경
> **배경**: 현재 V104 시드(2~6시간 5종, 15~35만원)를 실제 교습소 운영 값(3~6시간 4종, 16~26만원)으로 변경. 2시간 항목 삭제 + 금액 조정.

**DB 마이그레이션**:
- `src-tauri/migrations/201__update_standard_fees_seed.sql` (V201)
  - 기존 사용자 데이터 보호: 마이그레이션은 **빈 테이블이거나 기본 시드만 존재하는 경우에만** 시드를 교체
  - 구현 전략: V104에서 INSERT한 5행의 기본 시드를 감지하여 조건부 교체
    ```sql
    -- 기본 시드 상태 감지: 행 수 5 + weekly_hours 2,3,4,5,6 존재 시에만 교체
    -- 사용자가 이미 수정했으면(행 수 다르거나 weekly_hours 변경) 건드리지 않음
    DELETE FROM standard_fees
      WHERE (SELECT COUNT(*) FROM standard_fees) = 5
        AND (SELECT COUNT(*) FROM standard_fees WHERE weekly_hours IN (2,3,4,5,6)) = 5;
    INSERT OR IGNORE INTO standard_fees (weekly_hours, amount, sort_order) VALUES
      (3, 160000, 1),
      (4, 200000, 2),
      (5, 230000, 3),
      (6, 260000, 4);
    ```
  - 대안: 현재 사용자 데이터가 없는 시점(pre-release)이므로 단순 DELETE + INSERT도 가능하나, 방어적 접근을 기본으로 함

**예상 변경 파일**: `src-tauri/migrations/201__update_standard_fees_seed.sql` (1파일, 신규)
**예상 소요**: 1.5시간 (마이그레이션 작성 + sqlx prepare + 인메모리 테스트)
**AC (Acceptance Criteria)**:
- AC-T3-1: `sqlx migrate run` 후 standard_fees 테이블에 3/4/5/6시간 4행만 존재 (2시간 삭제됨)
- AC-T3-2: 금액이 각각 16만/20만/23만/26만원
- AC-T3-3: sort_order가 1/2/3/4로 순차 설정
- AC-T3-4: 이미 사용자가 시드를 수정한 DB에서 `sqlx migrate run` 실행 시 기존 데이터 보존 (idempotent)
- AC-T3-5: `.sqlx/` 오프라인 캐시 갱신 + 커밋

---

### T4: 결제수단 시드 데이터 변경
> **배경**: V001 결제수단 시드(현금/카드/계좌이체/기타 4종)를 실제 운영 값(현금 비활성 + 계좌이체/카드/결제선생/성남사랑 활성 5종)으로 변경

**DB 마이그레이션**:
- T3와 동일 마이그레이션 파일(V201)에 통합하거나, 별도 V202로 분리
- 권장: V201에 통합 (동일 sprint 시드 보정이므로)
- 구현 전략:
  - 기존 4행 시드 감지 후 조건부 교체
  - 교체 내용:
    | code | label | display_order | is_active |
    |------|-------|---------------|-----------|
    | cash | 현금 | 1 | 0 (비활성) |
    | transfer | 계좌이체 | 2 | 1 |
    | card | 카드 | 3 | 1 |
    | pay_teacher | 결제선생 | 4 | 1 |
    | seongnam_love | 성남사랑 | 5 | 1 |
  - `INSERT OR IGNORE`로 신규 코드(pay_teacher, seongnam_love) 추가
  - 기존 코드(cash)의 is_active를 0으로 UPDATE
  - 기존 'other' 코드 처리: 삭제 또는 비활성화 (사용자 데이터 없는 시점이므로 삭제 가능)

**예상 변경 파일**: `src-tauri/migrations/201__update_standard_fees_seed.sql`에 통합 (파일명을 `201__update_seed_data.sql`로 변경) 또는 `202__update_payment_methods_seed.sql` (1파일)
**예상 소요**: 1.5시간
**AC (Acceptance Criteria)**:
- AC-T4-1: payment_methods 테이블에 5행 존재: cash(비활성), transfer, card, pay_teacher, seongnam_love(모두 활성)
- AC-T4-2: display_order가 1~5 순차
- AC-T4-3: V001의 'other' 코드가 제거됨 (또는 비활성)
- AC-T4-4: 이미 사용자가 결제수단을 수정한 DB에서도 안전 (idempotent)
- AC-T4-5: 코드 테이블 관리 화면(`/settings/codes`)에서 결제수단 탭 정상 표시

---

### T5: 통합 검증
> 전체 변경사항 검증

- `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (V201/V202 마이그레이션 포함)
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- `pnpm tauri:dev` 실행 후 전수 검증:
  - T0: Node 25 환경에서 `/` 정상 렌더링
  - T1: 두 번째 인스턴스 기동 시 기존 창 포커스 + 새 프로세스 종료
  - T1-sub: 강제 점유 버튼 동작 확인 (양 PC 시뮬레이션)
  - T2: 마법사 완료 후 `/settings` 이동
  - T3: 표준교습비 4종 표시 (3/4/5/6시간, 16/20/23/26만원)
  - T4: 결제수단 5종 표시 (현금 비활성, 4종 활성)
- `.sqlx/` 오프라인 캐시 갱신 + 커밋

**예상 소요**: 2시간
**AC (Acceptance Criteria)**:
- AC-T5-1: 위 검증 항목 전수 통과
- AC-T5-2: 콘솔에 에러/경고 없음

---

## Task 의존성 그래프

```
T0 (Node 호환) ── 독립, 최우선 (dev 서버 기동 전제 조건)

T1 (single-instance) ── 독립
  |
  v
T1-sub (강제 점유 검증) ── T1 완료 후

T2 (마법사 redirect) ── 독립

T3 (표준교습비 시드) ── 독립
T4 (결제수단 시드) ── T3와 동일 마이그레이션 파일 사용 가능 (순차 또는 통합)

T5 (통합 검증) ── 모든 Task 완료 후 최종
```

**권장 실행 순서**: T0 -> T1 -> T1-sub -> T2 -> T3+T4 (통합) -> T5

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ cargo test 전체 통과 (V201 마이그레이션 포함)
- ⬜ cargo clippy -- -D warnings 통과
- ⬜ pnpm build 성공 (Next.js static export)
- ⬜ pnpm lint + pnpm tsc --noEmit 통과
- ⬜ `pnpm tauri:dev` 실행 후 T0~T4 전수 검증 통과
- ⬜ Node 25 + Node 20 양 환경 dev 서버 정상 기동 확인
- ⬜ 동일 PC 다중 인스턴스 차단 동작 확인
- ⬜ .sqlx/ 오프라인 캐시 갱신 및 커밋

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트 (Sprint 5 재정의 + Sprint 6 이연 반영)
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 신규 의존성

| 패키지 | 구분 | 용도 | 사용자 허가 |
|--------|------|------|-----------|
| `cross-env` | npm devDep | Node 25/20 환경 변수 호환 (T0) | 완료 |
| `tauri-plugin-single-instance` | Rust crate | 동일 PC 다중 인스턴스 차단 (T1) | 완료 |
| `@tauri-apps/plugin-single-instance` | npm dep | JS 바인딩 (T1) | 완료 |

---

## DB 마이그레이션

| 번호 | 파일명 (권장) | 내용 |
|------|--------------|------|
| V201 | `201__update_seed_data.sql` | 표준교습비 시드 변경 (2시간 삭제, 3~6시간 금액 조정) + 결제수단 시드 변경 (현금 비활성, 결제선생/성남사랑 추가, 기타 삭제) |

> Sprint 5 마이그레이션 예약 범위: V201~V299.
> T3+T4를 단일 마이그레이션으로 통합하여 마이그레이션 실행 순서 의존성을 제거한다.

---

## Capacity 확인

- 팀: AI 페어 프로그래밍 1인 개발
- 스프린트 기간: 2주 (10 영업일)
- 실작업 가능 시간: 하루 4시간 = 총 40시간
- Task 수: 6개 (T5 통합 검증 포함)
- 예상 소요: T0(1h) + T1(3h) + T1-sub(2h) + T2(0.5h) + T3(1.5h) + T4(1.5h) + T5(2h) = **11.5시간**
- 여유율: 71% (40h 대비 11.5h)
- 결론: **충분히 수용 가능** -- 소규모 안정화 sprint. 여유분은 A16(CVE 분석) 심층 조사 및 예기치 않은 이슈 대응에 활용

---

## 위험 및 대응

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R27 | `tauri-plugin-single-instance`가 Tauri 2.x와 호환되지 않을 가능성 | 중간 | Tauri 2.x 공식 플러그인 목록에서 호환성 확인. 비호환 시 `fs2` crate로 프로세스 레벨 락 파일 구현 (대안) |
| R28 | V201 시드 교체 마이그레이션이 기존 DB에서 조건 감지 실패 | 낮음 | pre-release 시점이므로 실제 사용자 데이터 없음. 조건 감지 로직은 방어적 작성하되 실패해도 기존 데이터 보존 (INSERT OR IGNORE) |
| R29 | cross-env가 Tauri dev 환경에서 환경 변수 전달 실패 | 낮음 | `pnpm dev` 스크립트만 변경 (tauri:dev는 Next.js dev를 내부 호출). tauri:dev에서도 NODE_OPTIONS가 전달되는지 검증 필요 |

---

## ROADMAP 변경 사항

본 Sprint 5는 ROADMAP.md의 원래 Sprint 5 범위(학사 스케줄 관리)와 다르다. ROADMAP 업데이트 시:
- Sprint 5 설명을 "Phase 1.5b 안정화 -- 환경 호환 + 다중 인스턴스 차단 + 시드 보정"으로 변경
- 원래 Sprint 5 범위(학사 스케줄)는 Sprint 6으로 이연
- 원래 Sprint 6(출결 관리)은 Sprint 7로 이연
- Phase 2 시작이 Sprint 6으로 조정됨을 명시

> 이 변경은 sprint-close 에이전트가 ROADMAP.md를 공식 업데이트할 때 반영한다. sprint-planner는 계획 문서에 의도만 기록한다.

---

## 참고 사항

- **PRD 확인**: §5.3 (app.lock 의도 = 양 PC 간 시점 분리), §4.0 (마법사 완료 흐름), §4.12 (코드 테이블 시드)
- **마이그레이션 번호**: V200 (Sprint 3) 사용 완료 -> V201부터 시작. Sprint 4는 V201을 계획했으나 기존 컬럼 활용으로 마이그레이션 불필요했음 -> V201 미사용 상태
- **Node 25 이슈**: `--no-experimental-webstorage` 플래그는 Node 20에서도 인식되나 무해하게 무시됨 (unknown flag가 아닌 기존 옵션의 부정형)
- **tauri-plugin-single-instance**: Tauri 2.x 공식 플러그인. https://v2.tauri.app/plugin/single-instance/
- **시드 데이터 idempotent**: pre-release 시점이므로 기존 사용자 데이터가 없지만, 방어적 마이그레이션으로 향후 재실행 안전성 확보
