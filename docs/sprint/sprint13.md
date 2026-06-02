# Sprint Plan sprint13

## 기간
2026-06-02 ~ 2026-06-16 (2주, 예상)

## 목표
실행 시 PIN 인증을 옵션화(C안: 키체인 자동 스킵)하여 매 실행 PIN 입력 부담을 제거하고, Phase 5 전면 취소를 코드베이스와 문서에 반영하며, Sprint 12 carry-over 기술 부채를 해소한다.

## ROADMAP 연계 기능
- PIN 인증 옵션화 (PRD SS5.5 완화 — ADR-008 결정 기록)
- Phase 5 취소 반영 (단원평가/학습보고서 메뉴 제거 + ROADMAP 재정렬)
- Sprint 12 carry-over 3건 (A85/A87/A88) + R88 경로 검증

## 작업 목록

### T0: Sprint 12 carry-over 정리

- ⬜ **T0-a: A87 — 미저장 확인 모달 저장 실패 시 이동 강행 수정** — `doSaveTemplate` re-throw 패턴 적용 또는 호출부 try-catch 후 저장 성공 시에만 navigate 실행. `src/app/notices/page.tsx:1485` 부근. (R89 해소) · skill: systematic-debugging
- ⬜ **T0-b: A85 — `update_bill_impl` 2쿼리 단일 LEFT JOIN 통합** — `SELECT b.status, p.is_paid FROM bills b LEFT JOIN payments p ON p.bill_id=b.id WHERE b.id=?` 단일 쿼리로 통합. `billing.rs:287~293`. 3스프린트 이연된 2줄 수정. (R83/R86 해소)
- ⬜ **T0-c: R88 — `save_notice_preview` 경로 경계 검증** — `canonicalize` 후 `data_root()` 또는 `output_root()` 하위 경로만 허용. `notice.rs:677`. Dialog 신뢰 경계 외 IPC 직접 우회 방어.
- ⬜ **T0-d: A70 — `PaymentsView` dirtyEntries payerName 필터** — 결제수단/입금일 변경 시 실제 변경된 행만 dirty 마킹. Sprint 11부터 이연.

### T1: Phase 5 전면 취소 반영

- ⬜ **T1-a: 메뉴 항목 제거** — `src/lib/menu-config.ts`에서 '단원 평가'(`/exams`) + '학습 보고서'(`/reports`) 두 항목 삭제. 관련 라우트 디렉토리(`src/app/exams/`, `src/app/reports/`)가 존재하면 삭제.
- ⬜ **T1-b: ROADMAP.md Phase 5 제거 + 재정렬** — Phase 5(Sprint 13~14) 섹션을 '취소' 표기로 변경, Phase 6/7 번호 재정렬, Sprint 번호 재매핑(기존 Phase 6 Sprint 15 → Sprint 14, Phase 7 Sprint 16~17 → Sprint 15~16), 의존성 맵/마일스톤 테이블 업데이트.
- ⬜ **T1-c: 문서 정리** — PRD SS4.7/SS4.8/SS6.1, AC-4.7-*/AC-4.8-* 관련 항목에 취소선/폐기 표기. ROADMAP 내 Phase 5 마일스톤(M6) 제거.

### T2: ADR-008 작성 (PIN 옵션화 설계 결정)

- ⬜ **ADR-008: 기기별 선택적 PIN 게이트** — PRD SS5.5(인증 의무) 완화 결정 기록. "기기별 선택적 PIN 게이트, 데이터 보호는 OS 계정+키체인 ACL로 위임" 트레이드오프 명시. ADR-007(PIN 전환) 후속. 저장: `docs/arch/adr-008-optional-pin-gate.md` · skill: brainstorming

### T3: 백엔드 — config.json `skip_pin_on_launch` 플래그

- ⬜ **T3-a: `SetupStatus` 구조체 확장** — `setup.rs`의 `SetupStatus`에 `skip_pin_on_launch: bool` 필드 추가 (`#[serde(default)]`로 후방 호환). 기본값 `false` (PIN 인증 ON 유지).
- ⬜ **T3-b: config.json get/set IPC 추가** — `get_pin_skip_setting(app: AppHandle) -> bool` + `set_pin_skip_setting(app: AppHandle, skip: bool) -> ()`. unlock 전 호출 가능해야 하므로 DB 접근 없이 config.json만 읽기/쓰기. `setup.rs`의 기존 `read_status`/`write_status` 재사용.
- ⬜ **T3-c: 단위 테스트** — `skip_pin_on_launch` 기본값 false, set 후 get 일치, 기존 config.json 후방 호환(필드 누락 시 false) 테스트 3건.

### T4: 백엔드 — 키체인 자동 잠금해제 경로

- ⬜ **T4-a: `auto_unlock_with_keychain` IPC 신규** — `startup.rs` 또는 별도 모듈에 신규 IPC 추가. `auth::get_cached_or_load_key()` 호출하여 키체인에서 유도키 로드 시도. 키 존재 시 `verify_password` 비교 단계를 스킵하고 바로 `db::initialize` + 후속 시퀀스(락/무결성/audit/heartbeat/backup/expiration) 실행. 키체인에 키 없으면 `Err("KeyNotFound")` 반환 → 프론트에서 LockScreen 표시.
- ⬜ **T4-b: 기존 `app_startup_sequence`와 공통 로직 추출** — 락 획득 + 무결성 체크 + DB 초기화 + audit cleanup + heartbeat/backup spawn + 소멸 전이가 양쪽 경로에서 중복되므로, 내부 함수 `run_post_auth_sequence(key, force_lock)` 추출. `app_startup_sequence`는 `verify_password` 후 이 함수 호출, `auto_unlock_with_keychain`은 키체인 키 로드 후 이 함수 호출.
- ⬜ **T4-c: 단위 테스트** — 키체인 키 존재 시 자동 잠금해제 성공, 키 부재 시 에러 반환, 이중 호출 시 `BACKGROUND` OnceLock 중복 spawn 방지 테스트.

### T5: 프론트엔드 — 설정 화면 PIN 스킵 토글

- ⬜ **T5-a: TypeScript IPC 래퍼 추가** — `src/lib/tauri/index.ts`에 `getPinSkipSetting()`, `setPinSkipSetting(skip: boolean)`, `autoUnlockWithKeychain()` 3종 래퍼 추가.
- ⬜ **T5-b: 설정 화면 토글 UI** — `/settings` 페이지에 '실행 시 PIN 인증 사용' 토글(Switch) 추가. 기본 ON. OFF 전환 시 경고 안내("이 PC에서 앱 실행 시 PIN 입력을 건너뜁니다. OS 계정 보호에 의존합니다.") 표시 후 저장. 토글 상태는 `getPinSkipSetting` IPC로 로드.
- ⬜ **T5-c: 토글 제약 조건 UI** — 키체인에 키가 없는 상태(새 PC, 아직 한 번도 PIN 인증 안 함)에서는 토글 비활성화 + 안내 메시지("이 PC에서 최초 1회 PIN 인증 후 사용 가능합니다.").

### T6: 프론트엔드 — 앱 진입 흐름 LockScreen 분기

- ⬜ **T6-a: LockScreen 렌더 분기** — 앱 진입 시 (1) `getPinSkipSetting()` 확인, (2) true이면 `autoUnlockWithKeychain()` 호출, (3) 성공 시 LockScreen 렌더 없이 메인 화면 진입, (4) 실패(키 없음 등) 시 기존 LockScreen 표시. · skill: frontend-design
- ⬜ **T6-b: 자동 잠금해제 중 로딩 상태** — 키체인 접근 중(특히 macOS에서 OS 프롬프트 가능) 로딩 스피너 표시. "자동 로그인 중..." 안내 텍스트.
- ⬜ **T6-c: 에러 처리** — 자동 잠금해제 실패 시 에러 토스트 없이 자연스럽게 PIN 입력 화면으로 폴백. 키체인 접근 OS 에러 발생 시 콘솔 로그만(사용자 화면 노출 금지).

### T7: 통합 검증

- ⬜ **T7-a: 자동 검증** — `cargo test --lib` 전수 통과, `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean, `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과.
- ⬜ **T7-b: 수동 검증 시나리오** — (1) 토글 OFF 설정 후 앱 재시작: PIN 입력 없이 메인 진입 확인, (2) 토글 ON 상태: 기존 PIN 입력 흐름 동작 확인, (3) 새 PC 시뮬레이션(키체인 키 제거 후): 토글 OFF여도 PIN 입력 요구 확인, (4) Phase 5 메뉴 항목 제거 확인.
- ⬜ **T7-c: 마이그레이션 self-check** — DB 마이그레이션 변경 없음 확인 (V111이 최신 유지).

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint12-retrospective.md` 액션 아이템

| 액션 ID | 항목 | 이번 스프린트 반영 |
|---------|------|-------------------|
| A87 | F2 — `doSaveTemplate` 저장 실패 시 이동 강행 | T0-a에서 해소 (re-throw 패턴 적용) |
| A88 | F1 — `save_notice_preview` 경로 경계 검증 | T0-c에서 해소 (canonicalize + data_root 접두 검증) |
| A85 | `update_bill_impl` 2쿼리 통합 (3스프린트 이연) | T0-b에서 반드시 해소 |
| A70 | `PaymentsView` dirtyEntries payerName 필터 | T0-d에서 해소 |
| A89 | 공지문 페이지 1534줄 분리 검토 | 이번 스프린트 범위 외 — 기능 변경 없으므로 다음 스프린트 이연 |
| A83 | 생체인증 병행/대체 정책 | 장기 과제 유지 |

## 완료 기준 (Definition of Done)

**필수**
- ⬜ cargo test --lib 전수 통과 (cipher off/on 양쪽)
- ⬜ cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings clean
- ⬜ pnpm lint + pnpm tsc --noEmit + pnpm build (static export) 통과
- ⬜ ADR-008 문서 완성 (`docs/arch/adr-008-optional-pin-gate.md`)
- ⬜ PIN 스킵 토글 OFF: 앱 재시작 시 PIN 입력 없이 메인 진입 동작
- ⬜ PIN 스킵 토글 ON: 기존 PIN 입력 흐름 정상 동작 (회귀 없음)
- ⬜ 키체인 키 부재 시: 토글 OFF여도 PIN 입력 요구 (안전 폴백)
- ⬜ Phase 5 메뉴 항목(단원 평가, 학습 보고서) 완전 제거 확인
- ⬜ ROADMAP.md Phase 5 취소 + Phase 6/7 재정렬 완료
- ⬜ Sprint 12 carry-over A85/A87/A88/A70 전수 해소

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ CHANGELOG.md 업데이트
- ⬜ develop 머지 (직접 --no-ff)

## Capacity 확인

| 항목 | 값 |
|------|-----|
| 작업 수 | 19개 세부 Task |
| 평균 소요 | 약 2시간/Task |
| 총 예상 | 약 38시간 |
| 실가용 | 1인 x 10일 x 4시간 = 40시간 |
| 여유율 | 약 5% — 타이트하지만 carry-over(T0) 4건이 소규모 수정이므로 실현 가능 |

## 의존성 및 리스크

| ID | 설명 | 영향도 | 대응 |
|----|------|--------|------|
| R90 | macOS 키체인 접근 OS 프롬프트 — dev 빌드 재컴파일마다 "항상 허용" 재프롬프트 가능. 사용자 혼란 | 중간 | T6-b에서 로딩 상태 + 안내 표시. dev 환경 한정이므로 프로덕션 서명 빌드에서는 1회만 발생 |
| R91 | `auto_unlock_with_keychain` + `app_startup_sequence` 공통 로직 추출 시 기존 startup 경로 회귀 | 높음 | T4-b에서 리팩토링 후 기존 테스트 + 신규 테스트로 양쪽 경로 검증. cipher off/on 양쪽 테스트 |
| R92 | config.json `skip_pin_on_launch` 플래그가 클라우드 동기화 폴더에 있으면 양 PC 간 설정 충돌 | 중간 | 설계상 `app_config_dir` (PC별 로컬) 저장 확정 — 클라우드 동기화 X. 코드 리뷰에서 경로 검증 |

## 예상 산출물

| 산출물 | 경로 |
|--------|------|
| 스프린트 계획 | `docs/sprint/sprint13.md` |
| ADR-008 | `docs/arch/adr-008-optional-pin-gate.md` |
| 리스크 레지스터 | `docs/risk-register/2026-06-02.md` (추가) |
| 변경 파일 (백엔드) | `setup.rs` (SetupStatus 확장 + IPC 2종), `startup.rs` (공통 로직 추출 + auto_unlock IPC), `lib.rs` (invoke_handler 등록), `billing.rs` (A85 통합), `notice.rs` (R88 경로 검증) |
| 변경 파일 (프론트) | `menu-config.ts` (Phase 5 제거), `lib/tauri/index.ts` (래퍼 3종), `/settings` (토글), LockScreen 진입 분기, `notices/page.tsx` (A87 수정) |
| 변경 파일 (문서) | `ROADMAP.md`, `PRD.md` (Phase 5 취소선), `CHANGELOG.md` |

## 참고 사항

- **DB 마이그레이션 없음** — 토글은 config.json(DB 밖)에 저장. V111이 최신 유지.
- **신규 의존성 없음** — 기존 keyring/setup/auth 인프라 재사용.
- **cipher feature 주의** — `get_cached_or_load_key()`는 cipher feature 게이트 내부. cipher off 빌드에서 auto_unlock 경로가 정상 동작하는지 반드시 검증 (cipher off 시 키 로드 불필요하므로 stub 반환 또는 즉시 성공 처리).
- **복구 코드 제거 완료** — Sprint 12에서 제거됨. PIN 분실 시 복구 코드 안내 불필요. 대신 설정 화면에 "PIN을 잊으면 앱 데이터를 초기화해야 합니다" 경고를 토글 UI 근처에 표시.
- **PRD SS4.7/SS4.8 관련 CLAUDE.md 규칙 정리**: `backend.md`, `frontend.md`의 단원평가/학습보고서 관련 제약은 T1-c에서 폐기 표기. 단, 파일 자체를 수정하면 scope 외 변경이 되므로, 취소선 처리는 해당 섹션에 `> [CANCELLED]` 주석으로 최소 표기.
