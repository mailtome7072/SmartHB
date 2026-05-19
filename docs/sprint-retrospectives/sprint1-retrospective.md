# Sprint Retrospective Sprint 1

> **Prime Directive**: 이 회고에 참여하는 모든 사람은 그 시점에 주어진 정보, 역량, 자원, 상황 하에서 최선을 다했다고 가정한다.

---

## 잘한 점

**SQLCipher PoC 조기 집중으로 R1 리스크 해소**

Sprint 계획의 핵심 제약("첫 2일 PoC 우선")을 T1에서 그대로 이행하여 R1(컴파일 실패)을 Sprint 초반에 해소했다. `bundled-sqlcipher-vendored-openssl` feature 기반의 선택적 cipher 게이트 설계(`#[cfg(feature = "cipher")]`)는 개발 빌드(cipher off)와 프로덕션 빌드(cipher on)를 명확하게 분리하여 이후 모든 Task에서 개발 편의성을 유지하면서도 보안 경로를 보장하는 토대가 되었다.

**paths/runtime/app_err 통합 리팩토링 (T11)**

backup/integrity/lock 세 모듈에 흩어진 중복 패턴(공유 경로 함수, `AppError::Variant(format!(...))`패턴, `spawn_blocking` 어댑터)을 T11에서 통합했다. `app_err!` 매크로와 `run_blocking` 헬퍼는 이후 Sprint에서 새 모듈이 추가될 때 표준 패턴을 제공한다. simplify 원칙("불필요한 추상화 없이 동작이 같아야 한다")을 실천한 사례다.

**audit best-effort 패턴 (`try_record`)**

`pool` 미초기화(unlock 전) 상태에서 감사 로그 기록이 호출될 수 있는 설계 문제를 `try_record` best-effort 래퍼로 해결했다. startup 흐름을 차단하지 않으면서도 감사 이벤트 7종을 모두 통합할 수 있었다. 테스트에서도 silent fail 동작을 명시적으로 검증하고 있다.

**보안 체크리스트 전 항목 통과**

Sprint 계획 문서의 보안 체크리스트(6개 항목) 전체가 코드 리뷰에서 확인되었다:
- `DerivedKey` ZeroizeOnDrop + Debug 마스킹
- 백업 파일 SQLCipher 암호화 상태 그대로 유지
- audit_logs 민감 데이터 미기록
- 디바이스 ID UUIDv4 OnceLock
- Tauri capabilities 최소 권한 (core:default + shell:allow-open)
- SQL 인젝션 방지 (sqlx bind 파라미터, PRAGMA key hex 전용)

**단위 테스트 74건**

pure 함수(키 유도, 파일명 파싱, 순환 삭제, PRAGMA 결과 분류, serde 정합)에 집중된 테스트 설계로 OS/Keychain/파일시스템 의존 없이 CI에서 안정적으로 실행된다. Keychain 통합 테스트는 의도적으로 제외하고 사용자 환경 검증으로 위임한 판단이 적절했다.

---

## 개선할 점

**cipher on 실측 미수행**

Sprint 목표 "앱 시작 < 3초"는 측정 코드(`Instant::now` + `elapsed_ms`)를 포함한 `StartupResult`로 구현되었으나, 단위 테스트는 모두 cipher off(인메모리) 환경에서만 통과했다. cipher on 빌드의 실제 PBKDF2 600K 반복, SQLCipher 키 파생, quick_check 연산 합산 시간은 실측하지 못했다. 계획 단계에서 "cipher on 빌드 성능 측정"을 Sprint 내 Task로 명시하지 않은 것이 원인이다.

**통합 시나리오 테스트 부재**

인메모리 테스트는 개별 함수 동작을 검증하지만, "비밀번호 설정 → 앱 재시작 → 잠금 해제 → 앱 종료 → exit 백업 생성 → 무결성 검증" 전체 흐름은 테스트되지 않았다. cipher off 빌드에서도 파일 기반 DB를 사용한 통합 시나리오 테스트가 없어 각 모듈 연결부에서 숨은 버그가 있을 가능성을 배제할 수 없다.

**`integrity_check` 통합 테스트가 cipher off 빌드에서 stub만 검증**

`check_returns_friendly_error_without_cipher` 테스트는 cipher off stub의 에러 메시지를 검증하지만, 실제 무결성 검증 로직(PRAGMA 실행, 손상 감지, 자동 복원 경로)은 cipher on 환경에서만 확인 가능하다. 개발 사이클에서 cipher on 빌드 기반 테스트가 누락될 위험이 있다.

**`release_lock` 미세 경쟁 조건**

`read_lock_info()`와 `remove_file()` 사이 advisory lock 누락이 Medium 리스크(R7)로 식별되었다. 단일 사용자 모델에서 실제 발생 확률은 낮지만, 설계 불일치(`acquire_lock_atomic`은 완전한 원자성, `release_lock`은 부분적)는 향후 리더가 오해할 소지가 있다.

---

## 액션 아이템

1. **[Sprint 2 계획 시 포함] cipher on 빌드 Windows 환경 시작 시간 실측**
   - 항목: DEPLOY.md `⬜ 앱 시작 ~ 메인 화면 < 3초 측정` 완료
   - 기준: `StartupResult.elapsed_ms < 3000` ms 달성 여부 기록
   - 담당: 개발자 (Windows 교습소 환경, cipher on 프로덕션 빌드)

2. **[Sprint 2 계획 시 포함] `KEYRING_USER_SALT` 클라우드 파일 이전 (R6 해소)**
   - 초기 설정 마법사(PRD §4.0) Task에 salt 파일(`smarthb/salt.bin`) 마이그레이션 명시
   - 양 PC 간 salt 공유 경로(`app_settings` 또는 클라우드 폴더 평문 파일) 결정
   - Sprint 계획 문서에 "T1 auth.rs KEYRING_USER_SALT 이전" 항목 명기

3. **[Sprint 2 계획 시 포함] `release_lock` advisory lock 적용 (R7 해소)**
   - `release_lock` 내 `read_lock_info` + `remove_file`을 단일 fs2 advisory lock 범위로 래핑
   - 변경 범위: `lock.rs` 단일 함수, DB 변경 없음 (Hotfix 수준 변경)

4. **[Sprint 2 이후] cipher on 빌드 기반 파일 DB 통합 시나리오 테스트 추가**
   - `#[cfg(feature = "cipher")]` + 임시 파일 기반 DB를 사용하는 통합 테스트 모듈 신설
   - 검증 범위: set_password → 재시작 시뮬레이션(pool 재초기화) → unlock_db → exit 백업 → quick_check
   - 목표: 모듈 연결부 버그 조기 발견 (Sprint 계획 시 T 항목으로 포함)

5. **[즉시] DEPLOY.md 수동 검증 항목 완료**
   - `pnpm tauri:dev` 실행 후 비밀번호 설정 → 복구 코드 발급 → 재시작 잠금 해제 흐름 수동 확인
   - cipher off 기준 < 3초 측정 (cipher on은 Action Item 1로 위임)
   - 완료 후 `develop` fast-forward merge 진행
