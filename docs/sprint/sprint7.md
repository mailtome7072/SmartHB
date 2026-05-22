# Sprint Plan sprint7

## 기간
2026-05-22 ~ 2026-06-05 (예상, 2주)

## 목표
Sprint 6 시각 검증에서 발견된 carry-over 8건을 전수 해소하여 Phase 2 학사 도메인의 운영 신뢰도를 확보한다. 특히 macOS Keychain 반복 다이얼로그(Critical UX)를 근본 해결하고, 교습기간 UX 5건(Issue 3~7)을 재설계하며, device_id 영속화로 양 PC 동시성 안전성을 보장한다. 아울러 3회 이월된 A17(salt.bin 이전)을 auth.rs 리팩토링과 함께 처리한다.

## ROADMAP 연계 기능
- Phase 2: 학사 + 출결 (Sprint 6~7) -- Sprint 7은 학사 도메인 완성도 회복 + 인프라 안정화
- §5.3 양 PC 시점 분리 (device_id 영속화)
- §5.5 OS Keychain 통합 (Keychain 호출 최적화)
- §4.4.2 교습기간 설정 UX 재설계
- §4.4.4 학사 일정 배치 제약 강화
- §4.12 설정 메뉴 하위 학사 일정 코드 관리

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint6-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 방법 |
|---------|------|----------------------|
| A23 | `ScheduleEventListItem`에 `is_system_reserved` 필드 추가 -- 프론트엔드 codeBadgeClass/draggableEventIds 하드코딩 제거 | **T4에서 처리** -- 백엔드 JOIN 확장 + 프론트엔드 플래그 기반 판단으로 변경 (R33 해소) |
| A25 | 교습기간 외 드롭 UI 경고 여부 | **T7에서 처리** -- Issue 4(배치 제약 강화)로 교습기간 내 일자만 배치 허용하면 자연스럽게 해소 (R34 해소) |
| A27 | salt.bin 이전 (Keychain -> cloud/smarthb/) | **T2에서 처리** -- Keychain 캐싱 리팩토링과 동시 진행. auth.rs 동일 파일 수정이므로 효율적 |
| A26 | 2028 공휴일 데이터 수집 (매년 1월) | **범위 외** -- 2026-12 이후 시점 작업. ROADMAP 메모만 유지 |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-05-22.md`

| 리스크 ID | 항목 | 이번 스프린트 반영 |
|-----------|------|------------------|
| R30 잔여 | 기간성 일정 드래그 이동 (현재 단일 일자만 지원) | **범위 외** -- Sprint 7은 carry-over 해소 집중. 기간성 드래그는 Phase 3 이후 필요 시 구현 |
| R33 | codeBadgeClass/draggableEventIds 하드코딩 | **T4에서 해소** -- is_system_reserved JOIN 추가 |
| R34 | 교습기간 외 드롭 가드 없음 | **T7에서 해소** -- 배치 제약으로 교습기간 내만 허용 |

---

## 작업 목록

### T1: macOS Keychain 반복 다이얼로그 해소 -- keyring 호출 통합 캐싱 (Issue 1)
> **배경**: `verify_password`가 keyring을 3회 개별 호출(salt 조회 + key 유도 + stored key 조회)하여 macOS Keychain이 각 호출마다 "Allow" 다이얼로그를 표시. startup 31초 소요 (PRD §5.6 3000ms 예산 대비 10배 초과). 매일 앱 실행마다 발생하는 Critical UX 이슈.
>
> **원인 분석**: `keyring::Entry::get_password()` 호출마다 macOS Security Framework가 사용자 승인을 요청. 현재 auth.rs에서 salt/key/recovery_code 각각 별도 keyring 항목으로 관리하므로, `verify_password` 경로에서 최소 2회(salt + key), `check_auth_status`에서 1회(salt) 호출.

**백엔드**:
- `src-tauri/src/commands/auth.rs`:
  - `CredentialCache` 구조체 도입 (OnceLock<Mutex<Option<CachedCredentials>>>)
    - `CachedCredentials`: salt([u8; 32]) + key(DerivedKey) -- ZeroizeOnDrop
  - `load_credentials_to_cache()` 함수: 앱 시작 시 1회 호출, salt + key를 keyring에서 읽어 캐시
  - `verify_password`: 캐시에서 salt/key 조회 -> keyring 직접 호출 제거
  - `check_auth_status`: 캐시 또는 1회 keyring 조회로 결정
  - `set_password` / `reset_password_with_code`: keyring 저장 후 캐시 갱신
  - `delete_key_from_keyring`: 캐시 무효화
- `src-tauri/src/commands/db.rs`: `retrieve_key_from_keyring()` -> 캐시 경유로 변경
- `src-tauri/src/commands/integrity.rs`: 동일 패턴 변경 (2곳)
- `src-tauri/src/commands/backup.rs`: 동일 패턴 변경 (1곳)
- `src-tauri/src/commands/recovery.rs`: `reset_password_with_code` 후 캐시 갱신 호출 추가

**예상 변경 파일**: `auth.rs`, `db.rs`, `integrity.rs`, `backup.rs`, `recovery.rs` (5파일)
**예상 소요**: 5시간
**skill**: systematic-debugging
**AC (Acceptance Criteria)**:
- AC-T1-1: macOS에서 `pnpm tauri:dev` 실행 -> 비밀번호 입력 -> Keychain 다이얼로그 최대 1회 표시 (기존 3+회 -> 1회)
- AC-T1-2: startup 시간 3초 이내 복귀 (PRD §5.6)
- AC-T1-3: `verify_password` 경로에서 keyring 직접 호출 0회 (캐시 경유)
- AC-T1-4: `set_password` / `reset_password_with_code` 후 캐시가 즉시 갱신되어 후속 `verify_password` 정상 동작
- AC-T1-5: `CredentialCache`가 ZeroizeOnDrop 적용 -- 메모리 노출 최소화
- AC-T1-6: 기존 단위 테스트 전체 통과 + 캐시 무효화 테스트 추가

---

### T2: salt.bin 이전 (Keychain -> cloud/smarthb/) (A17/A27)
> **배경**: salt는 비밀이 아니므로 Keychain 대신 클라우드 동기화 폴더의 `smarthb/salt.bin`에 평문 저장 가능. 양 PC 간 salt 자동 동기화를 통해 Keychain 의존도 감소 + 양 PC에서 동일 salt 사용 보장. auth.rs 주석에도 "T9 마법사 통합 시점에 이전" 명시 (3번째 이월).

**백엔드**:
- `src-tauri/src/commands/auth.rs`:
  - `store_salt_in_keyring` -> `store_salt(path)`: cloud 폴더 `smarthb/salt.bin`에 저장
  - `retrieve_salt_from_keyring` -> `retrieve_salt(path)`: 파일에서 읽기
  - T1의 캐시와 통합: salt 로드 경로가 파일 -> 캐시 -> verify_password
  - 마이그레이션 로직: 기존 Keychain에 salt가 존재하면 파일로 복사 후 Keychain 항목 삭제 (1회 자동 마이그레이션)
  - fallback: 파일 없고 Keychain에도 없으면 첫 설정 상태(NotInitialized) 반환
- `src-tauri/src/commands/paths.rs`: `salt_file_path()` 헬퍼 추가 (data_root / "salt.bin")

**예상 변경 파일**: `auth.rs`, `paths.rs` (2파일)
**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- AC-T2-1: 신규 설치 시 salt가 `{cloud}/smarthb/salt.bin`에 평문 저장
- AC-T2-2: 기존 설치 시 Keychain salt -> 파일 자동 마이그레이션 (1회) + Keychain 항목 삭제
- AC-T2-3: 양 PC 시나리오: PC-A에서 설정 -> salt.bin 동기화 -> PC-B에서 동일 비밀번호로 잠금 해제 가능
- AC-T2-4: salt.bin 파일 권한 600 (Unix) / 소유자 전용 (macOS/Windows 기본)
- AC-T2-5: Keychain에 남아있는 항목: key + recovery_code_hash (2개) -- salt 항목만 제거

---

### T3: device_id 영속화 (Issue 8, stale lock 안전성)
> **배경**: `lock.rs:46` `OnceLock<Uuid> + Uuid::new_v4()` -- 매 프로세스 시작마다 새 UUID 생성. prod 비정상 종료 후 다음 시작 시 stale lock 자동 점유가 항상 발동 (다른 디바이스 락 오인 위험). PRD §5.3 양 PC 시점 분리 정책의 핵심.

**백엔드**:
- `src-tauri/src/commands/lock.rs`:
  - `device_id()` 변경: 클라우드 폴더 `smarthb/device.id` 파일에 UUID 저장/로드
    - 파일 존재 시: 저장된 UUID 로드
    - 파일 미존재 시: UUID v4 생성 -> 파일 저장 -> 반환
    - 파일 손상/파싱 실패 시: 새 UUID 재생성 (graceful fallback)
  - `device_id_file_path()` 헬퍼 추가
  - 기존 OnceLock 유지: 프로세스 내에서 1회 파일 읽기 후 캐시

**예상 변경 파일**: `lock.rs` (1파일)
**예상 소요**: 2시간
**AC (Acceptance Criteria)**:
- AC-T3-1: 앱 시작 -> 종료 -> 재시작 후 동일 device_id 유지 확인
- AC-T3-2: `smarthb/device.id` 파일에 UUID 문자열 저장 확인
- AC-T3-3: 비정상 종료 후 재시작 시 stale lock 자동 점유가 "본 디바이스" 락으로 올바르게 판정
- AC-T3-4: PC-A와 PC-B의 device.id가 서로 다른 UUID 확인 (양 PC 식별 정확)
- AC-T3-5: device.id 파일 손상 시 새 UUID 재생성 + 파일 재기록

---

### T4: is_system_reserved JOIN 추가 + 프론트엔드 하드코딩 제거 (A23, R33)
> **배경**: CalendarCell.tsx의 `codeBadgeClass`와 ThreeMonthCalendar.tsx의 `draggableEventIds`가 시스템 코드명 6종을 한국어 문자열로 하드코딩. Sprint 6 코드 리뷰 Medium 이슈(R33). 백엔드 `list_schedule_events` 응답에 `is_system_reserved` 필드를 JOIN으로 추가하면 프론트엔드가 플래그 기반으로 판단 가능.

**백엔드**:
- `src-tauri/src/commands/academic.rs`:
  - `list_schedule_events` 쿼리 수정: schedule_codes JOIN 시 `is_system_reserved` 필드 포함
  - 응답 구조체에 `is_system_reserved: bool` 필드 추가

**프론트엔드**:
- `src/types/academic.ts`: `ScheduleEventListItem` 타입에 `is_system_reserved` 필드 추가
- `src/components/academic/CalendarCell.tsx`:
  - `codeBadgeClass` 함수: 코드명 switch -> is_system_reserved 플래그 기반 분기
  - 시스템 코드는 `code_name`으로 색상 결정 (기존 동작 유지), 비시스템 코드는 디폴트 색상
- `src/components/academic/ThreeMonthCalendar.tsx`:
  - `draggableEventIds` 계산: 코드명 Set 리터럴 -> `!event.is_system_reserved` 조건으로 변경

**예상 변경 파일**: `academic.rs`, `src/types/academic.ts`, `CalendarCell.tsx`, `ThreeMonthCalendar.tsx` (4파일)
**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- AC-T4-1: `list_schedule_events` 응답에 `is_system_reserved` 필드 포함 확인
- AC-T4-2: 시스템 코드(공휴일/보강데이/공휴수업일/방학/휴원일/단원평가) 배지 색상이 기존과 동일
- AC-T4-3: 사용자 추가 코드 배지가 드래그 가능, 시스템 코드 배지는 드래그 불가 (기존 동작 유지)
- AC-T4-4: CalendarCell.tsx와 ThreeMonthCalendar.tsx에 시스템 코드명 한국어 문자열 리터럴 0개
- AC-T4-5: `pnpm tsc --noEmit` + `pnpm lint` 통과

---

### T5: 학사 일정 코드 관리 -> /settings 하위로 이동 (Issue 3)
> **배경**: 현재 `/academic` 페이지 우측에 `ScheduleCodePanel` 마운트. 사용자 요구: 설정성 작업(코드 CRUD)과 운영성 작업(일정 배치)의 동선 분리. 학사 일정 코드 관리는 `/settings/schedule-codes` 신규 페이지로 분리.

**프론트엔드**:
- `src/app/settings/schedule-codes/page.tsx` (신규): 학사 일정 코드 관리 전용 페이지
  - ScheduleCodePanel 컴포넌트를 이동/재사용
  - 설정 메뉴 사이드바 또는 `/settings` 페이지 내 탭/링크로 접근
- `src/app/academic/page.tsx`:
  - ScheduleCodePanel 마운트 제거
  - 대신: 일정 배치 모드에서 "현재 활성 코드 목록" 드롭다운/라디오 선택만 표시
  - 코드 관리가 필요하면 "설정에서 관리" 링크 제공
- `src/app/settings/page.tsx`: "학사 일정 코드 관리" 메뉴 항목 추가 (링크)

**예상 변경 파일**: `settings/schedule-codes/page.tsx` (신규), `academic/page.tsx`, `settings/page.tsx`, 사이드바 컴포넌트 (3~4파일)
**예상 소요**: 3시간
**skill**: frontend-design
**AC (Acceptance Criteria)**:
- AC-T5-1: `/settings/schedule-codes` 페이지에서 코드 CRUD(추가/수정/삭제/토글) 전체 동작
- AC-T5-2: `/academic` 페이지에서 ScheduleCodePanel 제거 확인
- AC-T5-3: `/academic` 일정 배치 시 활성 코드 목록에서 코드 선택 가능
- AC-T5-4: 설정 메뉴에서 "학사 일정 코드 관리" 항목 접근 가능
- AC-T5-5: Pretendard 18pt, WCAG AA 준수

---

### T6: 교습기간 설정 UX 재설계 (Issue 5)
> **배경**: 현재 "교습기간 설정" 토글 버튼 -> 모드 활성 -> 셀 클릭 -> "확정" 버튼. 사용자 요구: 토글 제거, 교습기간 미확정 월에서 캘린더가 기본 선택 모드로 동작, 시작/끝 클릭 -> "확정/취소" 버튼만.

**프론트엔드**:
- `src/components/academic/StudyPeriodEditor.tsx`:
  - 토글 버튼 제거
  - 교습기간 미확정 월: 안내 메시지 "캘린더에서 교습기간을 선택하세요" 상시 표시
  - 시작일 클릭 -> 종료일 클릭 -> "확정" / "취소" 버튼
  - 교습기간 확정 월: 읽기 전용 표시 (기존 동작 유지)
- `src/app/academic/page.tsx`:
  - 화면 최상단 캘린더 네비게이션 컨트롤 (이전/다음 월 화살표) 배치
  - 모드 state machine 단순화: study-period 모드가 미확정 월에서 기본 활성
- `src/components/academic/ThreeMonthCalendar.tsx`:
  - 미확정 월 셀 클릭 동작: 교습기간 선택 모드로 자동 진입 (별도 토글 불필요)

**예상 변경 파일**: `StudyPeriodEditor.tsx`, `academic/page.tsx`, `ThreeMonthCalendar.tsx` (3파일)
**예상 소요**: 4시간
**skill**: frontend-design
**AC (Acceptance Criteria)**:
- AC-T6-1: 토글 버튼이 제거되고, 미확정 월에서 캘린더 셀 클릭 즉시 교습기간 선택 시작
- AC-T6-2: 안내 메시지 "캘린더에서 교습기간을 선택하세요" 미확정 월에서만 표시
- AC-T6-3: 시작일 -> 종료일 클릭 -> "확정" / "취소" 버튼으로 교습기간 설정 완료
- AC-T6-4: 확정된 교습기간 월에서는 읽기 전용 표시 + 삭제 버튼(T8과 연동)
- AC-T6-5: 화면 최상단 이전/다음 캘린더 네비게이션 동작

---

### T7: 학사 일정 배치 제약 강화 (Issue 4, R34)
> **배경**: 현재 `create_schedule_event`의 중복불가 검증은 동일 `code_id` + 동일 `event_date`만 차단. 사용자 요구: (1) 중복불가 코드는 다른 학사 일정과도 동일 날짜 공존 불가, (2) 학사 일정은 교습기간 내 일자에만 배치 가능.

**백엔드**:
- `src-tauri/src/commands/academic.rs`:
  - `create_schedule_event` 가드 강화:
    - 제약 1: `is_duplicate_blocked=1` 코드 배치 시, 해당 일자에 **어떤 다른 일정**이 존재하면 차단 (기존: 동일 code_id만 차단)
    - 제약 1-역: 해당 일자에 이미 `is_duplicate_blocked=1` 일정이 있으면, **새 코드 배치도 차단**
    - 제약 2: `event_date`가 어떤 확정된 교습기간의 `start_date ~ end_date` 범위 안에 있어야 배치 허용
    - 한국어 에러 메시지: "중복불가 코드는 다른 일정이 있는 날짜에 배치할 수 없습니다" / "학사 일정은 교습기간 내 일자에만 배치할 수 있습니다"
  - `update_schedule_event` (이동): 동일 제약 적용
  - 기존 단위 테스트 갱신 + 신규 제약 테스트 추가

**프론트엔드**:
- `src/components/academic/ThreeMonthCalendar.tsx`:
  - 드롭 핸들러: 교습기간 외 셀로 드롭 시 시각적 거부 피드백 (드롭 불가 영역 표시)
  - 에러 토스트: 백엔드 에러 메시지 표시

**예상 변경 파일**: `academic.rs`, `ThreeMonthCalendar.tsx` (2파일)
**예상 소요**: 4시간
**AC (Acceptance Criteria)**:
- AC-T7-1: 중복불가 코드 배치 시 다른 일정 존재 일자에서 차단 + 한국어 에러 반환
- AC-T7-2: 역방향: 중복불가 일정 존재 일자에 새 코드 배치 차단
- AC-T7-3: 교습기간 외 일자에 일정 배치 시도 시 차단 + 한국어 에러 반환
- AC-T7-4: 드래그 이동 시에도 동일 제약 적용
- AC-T7-5: 공휴일(시스템 코드, 중복불가)이 이미 배치된 일자에 다른 일정 배치 차단 확인
- AC-T7-6: 단위 테스트: 중복불가 상호 차단, 교습기간 외 차단, 공휴일 보호

---

### T8: 교습기간 삭제 + cascade 삭제 (Issue 6)
> **배경**: 확정된 교습기간 중 삭제 가능한 경우(지난 달 아님 + 마감 안 됨)에 "삭제" 버튼 제공. 삭제 시 그 기간 내 공휴일 제외 모든 학사 일정을 cascade 삭제.

**백엔드**:
- `src-tauri/src/commands/academic.rs`:
  - `delete_study_period` 확장 (또는 `delete_study_period_cascade` 신규):
    - 삭제 가능 조건: `is_confirmed = 1` AND 현재 월 이후 (지난 달 아님)
    - cascade 대상: 해당 교습기간 `start_date ~ end_date` 범위 내 `schedule_events` 중 공휴일 코드 **제외** 전부 삭제
    - 공휴일은 시드 데이터이므로 보존 (교습기간과 무관하게 캘린더에 표시 필요)
    - 삭제 후 해당 월에 새 교습기간 재확정 가능 (기존 로직으로 충분)
  - `get_cascade_delete_preview(study_period_id)` 신규 IPC:
    - 삭제 시 영향받는 일정 건수 + 목록 미리보기 반환 (프론트엔드 AlertDialog용)

**프론트엔드**:
- `src/components/academic/StudyPeriodEditor.tsx`:
  - 확정 교습기간에 "삭제" 버튼 추가 (지난 달 아닌 경우만 표시)
  - AlertDialog: "교습기간을 삭제하면 공휴일을 제외한 N건의 학사 일정이 함께 삭제됩니다. 삭제하시겠습니까?"
  - 삭제 후 캘린더 데이터 무효화 (TanStack Query invalidation)

**예상 변경 파일**: `academic.rs`, `StudyPeriodEditor.tsx`, `src/lib/tauri/index.ts`, `src/types/academic.ts` (4파일)
**예상 소요**: 4시간
**AC (Acceptance Criteria)**:
- AC-T8-1: 확정 교습기간 삭제 시 공휴일 제외 학사 일정 cascade 삭제 확인
- AC-T8-2: 공휴일 이벤트는 삭제 후에도 캘린더에 잔존 확인
- AC-T8-3: AlertDialog에 삭제 대상 건수 표시 + 확인 후에만 삭제 실행
- AC-T8-4: 지난 달 교습기간의 "삭제" 버튼 비표시/비활성
- AC-T8-5: 삭제 후 동일 월 새 교습기간 재확정 가능
- AC-T8-6: 단위 테스트: cascade 삭제 대상 정확성, 공휴일 보존, 지난 달 차단

---

### T9: 확정 교습기간 내 공휴일 삭제 차단 (Issue 7)
> **배경**: 사용자가 실수로 공휴일 셀 배지 클릭 -> 삭제하는 동작 차단. 공휴일은 시스템 코드로 시드된 데이터이므로 사용자 삭제 방지.

**백엔드**:
- `src-tauri/src/commands/academic.rs`:
  - `delete_schedule_event` 가드 추가:
    - 삭제 대상 이벤트의 코드가 `is_system_reserved = 1` AND `code_name = '공휴일'`이면 차단
    - 한국어 에러: "공휴일은 삭제할 수 없습니다"
  - 또는 더 넓게: `is_system_reserved = 1`인 코드의 이벤트는 모두 삭제 차단 (시스템 코드 보호)
    - 단, 단원평가 자동 배치 이벤트는 수동 삭제 허용해야 하므로, 공휴일 코드만 차단이 적절

**프론트엔드**:
- `src/components/academic/CalendarCell.tsx`:
  - 공휴일 배지: 삭제 아이콘/동작 비표시 (is_system_reserved + 공휴일 코드 조건)
  - 또는 클릭 시 "공휴일은 삭제할 수 없습니다" 토스트 표시

**예상 변경 파일**: `academic.rs`, `CalendarCell.tsx` (2파일)
**예상 소요**: 2시간
**AC (Acceptance Criteria)**:
- AC-T9-1: 공휴일 이벤트 `delete_schedule_event` 호출 시 차단 + 에러 반환
- AC-T9-2: 공휴일 배지에 삭제 UI 비표시 (프론트엔드 가드)
- AC-T9-3: 비공휴일 시스템 코드(단원평가 등) 이벤트는 삭제 허용 (기존 동작 유지)
- AC-T9-4: 단위 테스트: 공휴일 삭제 차단, 비공휴일 시스템 코드 삭제 허용

---

### T10: 통합 검증
> 전체 변경사항 검증

- `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과 (T1~T4, T7~T9 백엔드 변경 포함)
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- `pnpm tauri:dev` 실행 후 전수 검증:
  - T1: 비밀번호 입력 시 Keychain 다이얼로그 1회 이하, startup < 3초
  - T2: salt.bin 파일 생성 확인 + 기존 Keychain salt 마이그레이션
  - T3: 앱 재시작 후 동일 device_id 확인
  - T4: 캘린더 배지 색상 정상 + 시스템 코드 드래그 차단 정상
  - T5: `/settings/schedule-codes` 코드 관리 동작 + `/academic` 코드 패널 제거 확인
  - T6: 교습기간 미확정 월에서 셀 클릭 즉시 선택 모드 진입
  - T7: 중복불가 일정 배치 상호 차단 + 교습기간 외 배치 차단
  - T8: 교습기간 삭제 -> cascade 삭제 -> 공휴일 보존
  - T9: 공휴일 배지 삭제 차단

**예상 소요**: 3시간
**AC (Acceptance Criteria)**:
- AC-T10-1: 위 검증 항목 전수 통과
- AC-T10-2: 콘솔에 에러/경고 없음
- AC-T10-3: UC-2(월말 학사 일정 수립) 전체 흐름이 재설계된 UX로 완주 가능

---

## Task 의존성 그래프

```
T1 (Keychain 캐싱) ── 최우선 (Critical UX, 다른 Task 테스트 시 매번 영향)
  |
T2 (salt.bin 이전) ── T1 완료 필요 (auth.rs 캐시 구조 위에 salt 경로 변경)
  |
T3 (device_id 영속화) ── T2 완료 필요 (paths.rs data_root 경로 활용)

T4 (is_system_reserved JOIN) ── 독립 (백엔드+프론트 모두 수정)
T5 (코드 관리 이동) ── T4 완료 후 권장 (is_system_reserved 플래그 활용)
T6 (교습기간 UX 재설계) ── 독립 (StudyPeriodEditor 수정)
T7 (배치 제약 강화) ── T4 완료 후 권장 (is_system_reserved 활용)
T8 (교습기간 삭제 cascade) ── T6 완료 후 (삭제 버튼 UI 통합)
T9 (공휴일 삭제 차단) ── T4 완료 후 (is_system_reserved 플래그 활용)

T10 (통합 검증) ── 모든 Task 완료 후 최종
```

**권장 실행 순서**: T1 -> T2 -> T3 -> T4 -> T5 -> T6 -> T7 -> T8 -> T9 -> T10

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ macOS Keychain 다이얼로그 최대 1회 + startup < 3초 (Issue 1 해소)
- ⬜ salt.bin 클라우드 동기화 폴더 이전 완료 (A17/A27 해소)
- ⬜ device_id 영속화 + stale lock 정확한 디바이스 식별 (Issue 8 해소)
- ⬜ is_system_reserved JOIN + 프론트 하드코딩 제거 (R33 해소)
- ⬜ 학사 일정 코드 관리 `/settings` 하위 이동 (Issue 3 해소)
- ⬜ 교습기간 설정 UX 재설계 -- 토글 제거 + 기본 선택 모드 (Issue 5 해소)
- ⬜ 학사 일정 배치 제약: 중복불가 상호 차단 + 교습기간 내만 (Issue 4, R34 해소)
- ⬜ 교습기간 삭제 cascade + 공휴일 보존 (Issue 6 해소)
- ⬜ 공휴일 이벤트 삭제 차단 (Issue 7 해소)
- ⬜ cargo test 전체 통과
- ⬜ cargo clippy -- -D warnings 통과
- ⬜ pnpm build 성공 (Next.js static export)
- ⬜ pnpm lint + pnpm tsc --noEmit 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트 (Sprint 7 완료 반영)
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 신규 의존성

> Sprint 7은 신규 외부 패키지 추가 없음. 기존 crate/npm 패키지만 사용.

없음.

---

## DB 마이그레이션

없음. Sprint 7은 기존 테이블 스키마 변경 없이 IPC 레벨 가드 강화 + 프론트엔드 UX 재설계 중심.

---

## Capacity 확인

- 팀: AI 페어 프로그래밍 1인 개발
- 스프린트 기간: 2주 (10 영업일)
- 실작업 가능 시간: 하루 4시간 = 총 40시간
- Task 수: 10개 (T10 통합 검증 포함)
- 예상 소요: T1(5h) + T2(3h) + T3(2h) + T4(3h) + T5(3h) + T6(4h) + T7(4h) + T8(4h) + T9(2h) + T10(3h) = **33시간**
- 여유율: +17.5% (40h 대비 33h -- 7h 여유)
- 결론: **적정** -- Sprint 6(45h, -12.5%)과 달리 여유 확보. Keychain 캐싱(T1)이 예상보다 복잡할 경우 버퍼 충분. T5~T6 프론트 작업은 기존 컴포넌트 재배치이므로 생산성 높음.

---

## 위험 및 대응

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R35 | Keychain 캐싱(T1) 도입 후 메모리 내 key 노출 시간 증가 -- 앱 실행 전체 시간 동안 key가 메모리에 존재 | 중간 | ZeroizeOnDrop으로 프로세스 종료 시 자동 폐기. 캐시를 Mutex<Option<>>으로 감싸 명시적 invalidation 가능. PRD §5.5 보안 모델(단일 사용자, 로컬 앱) 대비 수용 가능한 trade-off |
| R36 | salt.bin 이전(T2) 시 기존 Keychain salt와 파일 salt 동시 존재 기간에 불일치 가능 -- 마이그레이션 중 crash 시 | 낮음 | 마이그레이션은 원자적 수행: (1) Keychain에서 salt 읽기 -> (2) 파일 쓰기 -> (3) Keychain 삭제. (2) 실패 시 다음 시작에 재시도. (3) 실패 시 다음 시작에 파일 존재 확인 후 Keychain 삭제만 재시도 |
| R37 | device_id 영속화(T3) 후 클라우드 동기화로 양 PC에 동일 device.id가 복제 -- 동일 ID로 양 PC가 인식되면 lock 구분 불가 | 중간 | device.id를 클라우드 동기화 폴더가 아닌 **OS 로컬 전용 경로**(Tauri app_config_dir 또는 OS temp)에 저장. 클라우드 동기화 대상에서 제외. 또는 smarthb/ 하위에 두되 `.nosync` (macOS) 마킹 검토 |
| R38 | 교습기간 배치 제약 강화(T7) 후 기존 배치 데이터와 불일치 -- Sprint 6에서 교습기간 외에 배치한 일정이 있을 경우 수정 불가 | 낮음 | pre-release 상태(실사용자 데이터 없음)이므로 기존 데이터 불일치 가능성 극히 낮음. 만약 존재 시 개발 DB 리셋으로 해소 |

---

## 참고 사항

- **PRD 확인**: §5.3(양 PC 시점 분리), §5.5(OS Keychain), §5.6(startup 성능 예산), §4.4.2(교습기간), §4.4.4(배치 제약)
- **Issue 2 (종료 메뉴)**: Sprint 6 후속 develop 직접 패치 완료 (`2bb0f6c`). Sprint 7 범위 아님
- **출결 도메인**: Sprint 8로 이연. carry-over 해소 후 Phase 2 학사 도메인 완성도 확보가 선행 조건
- **R30 잔여 (기간성 드래그)**: Sprint 7 범위 아님. Phase 3 이후 필요 시 구현
- **R37 대응 상세**: T3 구현 시 device.id 저장 경로를 결정해야 함. 클라우드 폴더(양 PC 복제 위험) vs OS 로컬(동기화 안 됨, 안전). 구현 시점에 최종 결정 (Negotiable)
- **A17(salt.bin) 마이그레이션**: 기존 설치 사용자가 이론적으로 존재(개발 환경)하므로 Keychain -> 파일 자동 마이그레이션 로직 필수. 신규 설치는 파일에 직접 저장
