# Sprint Plan sprint16

## 기간
2026-06-09 ~ 2026-06-22 (2주)

## 목표
**(최우선) 원장이 보고한 수업일 변경 2종(1회성 이동 + 특정일 이후 영구 변경)을 시스템이 정합성 손실 없이 수용하도록 출결·스케줄 도메인을 보강한다.** 이어 Sprint 15에서 이연된 양 OS 빌드 검증 + 양 PC 동기화 시나리오를 완수하고, CSV 가져오기로 원장 실데이터를 이관하며, DB 폴더 변경(경로 재지정 + salt.bin 이전)을 구현한다. 격식 UAT 없이 원장이 바로 실사용을 개시하고, 초기 피드백에 대응한다. v1.0 릴리즈 준비(CHANGELOG, 인스톨러 최종 확인)를 완료하되 실제 배포(태그 push)는 사용자 명시 지시까지 대기한다.

## ROADMAP 연계 기능
- **사용자 이슈(2026-06-08) — 수업일 변경 도메인 (T0, 최우선)**: 케이스1(특정일 1회성 이동) + 케이스2(특정일 이후 영구 스케줄 변경 + 출결 부분 재생성)
- Phase 6 마지막 스프린트: 실사용 개시 + 초기 피드백 반영 + v1.0 릴리즈 준비
- Sprint 15 이연: T7(양 OS 빌드 검증), T8(양 PC 동기화 시나리오), T9(통합검증 빌드부)
- Sprint 15 회고 액션: A98(즉시 적용 완료), A99(Ctrl+N 입력 방어), A100(미저장 이탈 경고), A96(복원 리허설 Low)
- Sprint 15 코드 리뷰: R105(교습소 정보 미저장 이탈 경고 — F3 Medium)
- 확정 포함: DB 폴더 변경 + salt.bin 이전 (PI-16 사용자 결정 2026-06-08)
- 이연 기능: CSV 가져오기(PRD SS4.13.1), 공지문 I/O 병렬화, 접근성 잔여(44px/gray-500/F1/Ctrl+S), A89 notices UI 구획화
- 기술 부채: 출결표 N+1 재설계, 셀 memo, makeup_attendances 인덱스(실측 후)

---

## Capacity 분석

### Velocity 참조 (과거 실적)

| Sprint | 계획(h) | 특성 | 비고 |
|--------|---------|------|------|
| 13 | 38h | PIN 옵션화 + carry-over | 소형, 검수 중 추가 2건 |
| 14 | 38h | 대시보드+자가진단+내보내기 | 검증-phase에서 기능 대폭 추가 |
| 15 | 38h | 안정화 + 접근성 감사 | T7~T9 이연, 38h 내 수용 완료 |

**패턴 분석**:
- 38h 계획이 3스프린트 연속 수용됨.
- **수업일 변경 도메인(T0, +8h MUST)이 사용자 이슈로 최우선 추가**되었고, DB 폴더 변경(PI-16, +8h MUST)까지 더해져 MUST만 44h로 가용 40h를 초과한다.
- 대응: **SHOULD(T8 접근성·T9 공지문 I/O) 전량 Post-MVP 이연을 기본 전제로 한다.** 피드백 버퍼(T7)는 가변. 그래도 MUST 44h가 타이트하므로, 필요 시 스프린트를 +2~3일 연장하거나 T10(릴리즈 준비)·T11(통합검증)을 스프린트 말 집중 배치한다.
- 격식 UAT 제거(PI-18 확정)로 실사용 개시 준비/피드백 Task가 축소되어 일부 여유 확보.

### Capacity 산정

| 항목 | 값 |
|------|-----|
| 팀 인원 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10일 |
| 실작업 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |

### 작업 소요 예상

| Task | 예상 소요 | 우선순위 | 비고 |
|------|----------|---------|------|
| **T0 수업일 변경 도메인 (케이스1+2)** | **8h** | **MUST · 최우선** | **사용자 이슈. 착수 시 가장 먼저. 케이스1 출결행 이동 + 케이스2 날짜인식 재생성 + V306** |
| T1 회고 액션 + carry-over | 3h | MUST | A99/A100/R105 통합 |
| T2 CSV 가져오기 | 6h | MUST | 실사용 데이터 이관 전제조건 |
| T3 DB 폴더 변경 + salt.bin 이전 | 8h | MUST | PI-16 확정. copy-then-switch + salt.bin/app.lock/백업 동반 |
| T4 양 OS 빌드 검증 | 4h | MUST | Sprint 15 이연 T7 |
| T5 양 PC 동기화 시나리오 | 3h | MUST | Sprint 15 이연 T8 |
| T6 실사용 개시 준비 | 2h | MUST | 양 OS 설치 + 데이터 이관 확인 + 기동 검증 |
| T7 초기 실사용 피드백 대응 (버퍼) | 4h | MUST | Critical/High 피드백 즉시 수정 |
| T8 접근성 잔여 개선 | 4h | SHOULD | 밀집 UI 44px, F1, Ctrl+S |
| T9 공지문 I/O 병렬화 | 3h | SHOULD | 50장 일괄 생성 성능 |
| T10 v1.0 릴리즈 준비 | 3h | MUST | CHANGELOG + 인스톨러 최종 확인 |
| T11 통합 검증 | 3h | MUST | cargo test + clippy + lint + build |
| **합계** | **51h** | | 가용 40h 초과 — SHOULD 전량 이연 + 필요 시 스프린트 연장으로 조정 |

> **MUST 합계**: 44h (T0+T1+T2+T3+T4+T5+T6+T7+T10+T11)
> **SHOULD 합계**: 7h (T8+T9) — **Post-MVP 이연을 기본 전제**로 한다 (MUST 초과로 사실상 착수 불가).
> A89 notices UI 구획화(COULD 2h)는 Post-MVP 이연 확정.
> MUST 44h > 가용 40h. T0(수업일 변경 8h) 최우선 추가로 타이트. 피드백 버퍼(T7 4h)가 가변적이며, 초과 시 스프린트 +2~3일 연장 또는 T10/T11 말 집중.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint15-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 |
|---------|------|-------------------|
| A98 | self-verify `--all-targets` 추가 | **즉시 적용 완료** (Sprint 15 close 시 CLAUDE.md/harness-engineering.md 교정됨) |
| A99 | Ctrl+N 입력 필드 포커스 방어 로직 | T1에서 처리 — `GlobalShortcuts` INPUT/TEXTAREA/SELECT 가드 추가 |
| A100 | 미저장 이탈 경고 다이얼로그 공통 구현 | T1에서 처리 — `useUnsavedChanges` 훅 + 교습소 정보 화면 적용 (R105 해소) |
| A96 | 복원 리허설 dev 환경 개선 | Low — Capacity 여유 시 검토, 미포함 |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-06-07.md`

| ID | 설명 | 반영 방법 |
|----|------|-----------|
| R101 | Windows PC 물리 접근 제한 | T4(양 OS 빌드 검증)에서 GitHub Actions CI + 교습소 방문 설치 |
| R105 | 교습소 정보 미저장 이탈 경고 | T1(A100)에서 `useUnsavedChanges` 공통 훅 구현으로 해소 |
| R113 | 케이스2 출결 재생성 시 보강(makeup) 연결 행 처리 | T0 추천안(미처리 present만 삭제·처리행 보존)으로 고아 데이터 회피. 보존 후 옛 요일 잔존 가능성은 사용자 경고로 노출 |
| R114 | 케이스1 월 경계 이동 시 청구/요약 월 불일치 | T0에서 동월 한정으로 차단 + 친화적 안내. 월 경계 이동 요구는 Post-MVP 재검토 |
| R115 | 케이스2 주당시간 변동 시 청구 미반영 인지 부족 | `apply_schedule_change`가 변경 전/후 주당시간 반환 → 다르면 재확인 토스트. 금액은 청구 화면 수동 조정 |
| R116 | 케이스2 사후(과거) 변경 시 과거 출결 소급 변경 — 변경일을 과거로 지정하면 지난 날짜의 present 출결이 신 요일로 재생성되어 기존 기록이 바뀜 | 변경일·사전/사후 여부를 확인 다이얼로그에 명시 + 보존 건수 경고. 처리행(결석/보강/메모) 보존으로 핵심 데이터 손실 방지. audit에 사전/사후 구분 기록 |

---

## 작업 목록

### T0: 수업일 변경 도메인 — 1회성 이동 + 특정일 이후 영구 변경 (8h) — MUST · 최우선

> **사용자 이슈(2026-06-08)**: 원생 수업 진행 중 수업일을 변경하는 2가지 케이스를 시스템이 정합성 손실 없이 수용해야 한다. **`/sprint-dev 16` 착수 시 가장 먼저 수행한다.**
> **설계 결정 4건 확정(2026-06-08)**: ①케이스1 = 출결 행 이동(동월 한정·메모 기록) ②케이스2 = 변경일 이후 **미처리(present)만 재생성**, 결석/보강/메모 행 보존 + 보존 건수 경고 ③청구 = 현행 유지 + 주당시간 변동 시 재확인 안내만 ④우선순위 = T0 최우선.
> **현 구조 파악(조사 완료)**: `student_schedules`는 `effective_from`/`effective_to` 시계열 이력 보유, `set_schedule`=기존 마감(effective_to=신규 effective_from)+신규 INSERT. 그러나 `generate_attendances`(`load_weekly_schedule`)는 **현행(effective_to IS NULL) 요일만** 보고 날짜 범위를 무시 → 케이스2 출결 측 미구현. 케이스1은 개념 자체 부재. 청구(`billing.rs`)는 현행 스케줄 주당시간 기반이며 출결을 미참조.

**마이그레이션**
- ⬜ **V306** — `regular_attendances`에 `note TEXT` 컬럼 추가 (`ALTER TABLE ADD COLUMN`, 테이블 재구성 불필요 → `sqlite-migration-fk-rebuild` 함정 비해당). 케이스1 이동 메모 저장용 — `absence_memo`(결석 사유)와 의미 분리. `.sqlx` 오프라인 캐시 갱신 + 커밋. (현재 최신 V305 → V306)

**케이스 1 — 1회성 수업일 이동 (출결 행 재배치)**
- ⬜ `attendance.rs` 신규 IPC `move_attendance(student_id, from_date, to_date)`:
  - `from_date` 정규 출결 행 조회 — `status='present'`만 이동 허용 (결석/보강완료/소멸 행은 거부 + 친화적 안내)
  - **동월 한정** 검증: from/to의 `year_month` 동일 (월 경계 이동은 차단 + 안내 — 청구/요약 월 정합 보호)
  - **도착일 OFF 검증**: `to_date`가 공휴일 또는 `allows_regular_class=0`(정규수업 OFF) 일자면 **차단 + 안내** (PI-25). `load_off_dates` 동일 판정 재사용
  - 충돌 검증: `to_date`에 해당 원생 출결 기존재 시 거부 (UNIQUE(student_id, event_date))
  - `event_date`를 `to_date`로 UPDATE + `note`에 "M/D(요일)→M/D(요일) 이동" 자동 기록 (`class_minutes` 유지 → 청구·주당시간 불변)
  - `audit::AttendanceRescheduled` variant 추가
  - 단위 테스트: 정상 이동(비OFF·비수업 요일), **OFF일/공휴일 이동 차단**, 월경계 차단, to_date 충돌 차단, present 외 상태 거부
- ⬜ 출결표 UI (조작 방식 = 클릭 → 다이얼로그, PI-26):
  - **진입점 = 옮길 출결이 있는(present) 셀 클릭** → 액션 메뉴 "수업일 이동". (도착할 빈 셀을 직접 클릭하는 방식이 아님 — **비수업일 빈 셀 클릭은 기존 보강 등록 진입점**(Sprint 9 `MakeupRegisterDialog`)이라 구분 필요)
  - 도착일은 **다이얼로그 내 월(月) 달력 컨트롤에서 선택**(PI-27) — 출발일이 속한 달의 그리드 캘린더. **기존 `ThreeMonthCalendar`/`CalendarCell`(Tailwind grid) 패턴 재활용** (shadcn Calendar/react-day-picker 신규 도입 대신 — 학사 캘린더와 일관·신규 의존성 회피). 텍스트 입력(`type="date"`)이 아닌 시각적 달력 위젯
  - 달력에서 OFF일/공휴일/기존 출결 있는 날/타월(他月) 셀은 **비활성(클릭 불가) + 사유 표시**(PI-25 차단 규칙을 UI 단에서 선반영)
  - 확정 시 원본 셀은 비고, 도착 셀에 출석 + `note` 툴팁 표시. TanStack Query 무효화 + 1단계 Undo(PRD §5.7 위험행위 확인)

**케이스 2 — 특정일 이후 영구 변경 (변경일 양방향 지정 + 미처리만 재생성)**

> **변경일(effective_date D)은 원장이 직접 지정**하며, **오늘보다 미래(사전 예약)·과거(소급 정정) 양방향 모두 허용**한다 (사용자 요구 2026-06-08). 변경일부터 신 스케줄을 반영한다.

- ⬜ `generate_impl` **날짜 인식 리팩토링** (근본 개선): `load_weekly_schedule`(현행 only) → 원생별 스케줄 이력 전체 로드 후 각 날짜 `d`에 유효 스케줄 매칭 `effective_from ≤ d AND (effective_to IS NULL OR d < effective_to)`. **effective_to는 exclusive 확정** (set_schedule이 effective_to=신규 effective_from으로 마감하므로 `d<D`=옛 스케줄, `d≥D`=신 스케줄로 무경계 연결). 기존 정규 출결 생성도 동일 로직 적용.
- ⬜ 신규 IPC `apply_schedule_change(student_id, effective_date D)`:
  - **변경일 D 검증**: 현행 스케줄 `effective_from` 이후 + 원생 `enroll_date` 이후. 그 이전이면 차단 + 친화적 안내(이력 모순 방지). 미래/과거 자체는 허용.
  - **재생성 범위 = D ~ 확정 교습기간 말** (오늘 무관). 사후(D<오늘)면 이미 지난 날짜의 `present` 출결도 신 스케줄로 소급 재생성, 사전(D>오늘)이면 D 이후 미생성분은 향후 `generate`가 날짜 인식으로 자동 반영.
  - 해당 범위의 `regular_attendances` 중 `status='present'` 행만 DELETE → 날짜 인식 스케줄로 재생성 (`INSERT OR IGNORE`, off_date·입퇴교 범위 동일 적용)
  - 결석/보강완료/소멸/메모(`note` 존재) 행은 **보존** → 보존 건수 + 옛 요일 잔존 목록 반환
  - 여러 확정 교습기간 월에 걸치면 전부 반영, 미확정 월은 건너뜀
  - `audit::ScheduleChangedWithRegen` variant (변경일 + 사전/사후 구분 기록)
  - 단위 테스트: **사후 변경 소급 재생성 / 사전 변경(미래 출결 미생성 시 no-op + 향후 generate 반영) / 사전 변경(미래 출결 기생성 시 재생성)**, 결석·보강 보존, 변경일 이전 불변, D < effective_from 차단, 미확정월 skip, 주당시간 변동 감지
- ⬜ 스케줄 변경 UI(`ScheduleEditor`, 원생 관리 스케줄 수정): **변경일 날짜 선택 입력**(기본값 오늘, 과거·미래 모두 선택 가능) → `set_schedule(effective_from=D)` + `apply_schedule_change(D)` 연계. 확인 다이얼로그에 변경일·사전/사후 여부 + "변경일 이후 출결을 신 스케줄로 재생성합니다. 결석/보강 N건은 보존됩니다." 명시
- ⬜ 청구 연계: `apply_schedule_change` 결과에 변경 전/후 주당시간 동봉 → 다르면 프론트 토스트("이번 달 청구액 재확인 필요"). billing 생성 로직 자체는 불변(현행 스케줄 1값 + `adjusted_amount` 수동 조정 유지)

**TypeScript / 공통**
- ⬜ IPC 래퍼 추가(`moveAttendance`, `applyScheduleChange`) + `src/types` 타입 + dev-mode fallback

### T1: 회고 액션 + 코드 리뷰 carry-over (3h) — MUST

- ⬜ A99: `GlobalShortcuts` Ctrl+N 입력 필드 방어 — `e.target` tagName이 INPUT/TEXTAREA/SELECT이면 Ctrl+N 억제
- ⬜ A100 + R105: 미저장 이탈 경고 공통 훅 `useUnsavedChanges` 구현 — `beforeunload` + Next.js `routeChangeStart` 가드. `/settings/info`(교습소 정보) 적용
- ⬜ Ctrl+S 전역 저장 단축키 등록 — `GlobalShortcuts`에 추가, 현재 활성 폼의 저장 함수 실행

### T2: CSV 가져오기 (6h) — MUST

> PRD SS4.13.1 — 실사용 개시의 첫 번째 작업. 원생 실데이터 이관용.

**백엔드**
- ⬜ `import.rs` 신규 모듈 — `import_students_csv` IPC
  - CSV 파싱(BOM 처리 + EUC-KR/UTF-8 자동 감지)
  - 필수 컬럼: 이름, 학교명, 학년, 연락처 (선택: 일련번호, 입교일, 성별, 수업요일, 생년월일)
  - 중복 검사: 이름+연락처 동일 시 skip/overwrite 옵션
  - 마이그레이션 불필요 (기존 students/student_schedules 테이블 활용)
  - 단위 테스트: 정상 임포트, 중복 skip, 필수 컬럼 누락 에러, EUC-KR 처리

**프론트엔드**
- ⬜ TypeScript IPC 래퍼 + `src/types/import.ts` 타입
- ⬜ `/settings/import` 라우트 — 파일 선택(Tauri Dialog) + 미리보기 테이블 + 컬럼 매핑 + 임포트 실행 + 결과 요약

### T3: DB 폴더 변경 + salt.bin 이전 (8h) — MUST · PI-16 확정 (2026-06-08)

> 클라우드 동기화 경로 재지정 UI/IPC + salt.bin 동반 이전. R12 salt.bin 이전의 최종 해소.
> 참조 메모리: `keyring-v3-features-trap`, `ntfs-power-loss-pattern`, `sqlite-migration-fk-rebuild`

**설계 검토 (ADR 권장)**
- ⬜ ADR 작성 — DB 폴더 변경 전략 결정: copy-then-switch vs move-then-update
  - copy-then-switch 권장: 원본 보존, 실패 시 원래 경로 즉시 복귀
  - 중간 실패(복사 중 강제 종료) 시 복구 전략 명시

**백엔드**
- ⬜ `paths.rs` 확장 — `change_data_folder` IPC 신규
  - 단계 1: 대상 폴더 유효성 검증 (쓰기 권한, 디스크 여유)
  - 단계 2: DB 파일(`app.db`) 복사 (`ntfs-power-loss-pattern` 적용 — fsync 호출)
  - 단계 3: salt.bin 복사 (손상 감지 `is_corrupted()` 적용)
  - 단계 4: 백업 폴더(`backup/`) 복사 (4계층 전체)
  - 단계 5: app.lock 해제 + 신규 경로에 app.lock 재생성
  - 단계 6: config.json `cloud_folder` 경로 업데이트
  - 단계 7: 원본 폴더에 이전 완료 마커 파일 생성 (역방향 참조)
  - 실패 시: 원래 config.json 복원 (copy-then-switch이므로 원본 무손상)
- ⬜ WAL 파일 처리 — `PRAGMA wal_checkpoint(TRUNCATE)` 실행 후 복사 (WAL/SHM 잔여 방지)
- ⬜ cipher ON 검증 — 암호화 DB 복사 후 정상 열기 확인 (키는 Keychain에서 동일 키 사용)
- ⬜ 단위 테스트: 정상 이전, 원본 보존 확인, 잘못된 경로 거부, 권한 없는 폴더 거부

**프론트엔드**
- ⬜ `/settings` 허브 — 'DB 폴더 변경' 카드 활성화 (현재 disabled 상태 → 활성)
- ⬜ `ChangeFolderDialog` — 폴더 선택(Tauri Dialog) + 진행률 표시 + 완료/실패 알림
- ⬜ 변경 완료 후 앱 재시작 안내 (Tauri `process::relaunch` 사용)
- ⬜ TypeScript IPC 래퍼 추가

**정합성 검증 항목**
- ⬜ salt.bin 이전 후 PIN 잠금해제 정상 동작 (`keyring-v3-features-trap` — Keychain 키 유지 확인)
- ⬜ 이전 후 백업 4계층 정상 동작 (경로 참조 갱신)
- ⬜ 양 PC 시나리오: 한 PC에서 폴더 변경 → 다른 PC에서 새 경로 인식 (config.json 동기화)

### T4: 양 OS 빌드 검증 (4h) — MUST · Sprint 15 이연 T7

- ⬜ macOS: `pnpm tauri:build` → `.dmg` 생성, 설치/실행/삭제, Apple Silicon arch 확인
- ⬜ Windows: GitHub Actions CI matrix(windows-latest) 빌드 확인, `.msi`/`.exe` 설치/실행/언인스톨
- ⬜ 인스톨러 체크리스트: 앱 아이콘, 시작 메뉴/Dock 등록, 기존 데이터 유지(업그레이드), 언인스톨 후 잔여 파일 없음
- ⬜ WebView2 런타임 확인(Windows), Xcode CLI 확인(macOS)

### T5: 양 PC 동기화 시나리오 테스트 (3h) — MUST · Sprint 15 이연 T8

- ⬜ 시나리오 1: Windows → Mac 전환 — Windows 앱 정상 종료 → 클라우드 동기화 대기 → Mac 앱 시작 → 데이터 정합 확인
- ⬜ 시나리오 2: Mac → Windows 역방향 — 동일 절차 역방향
- ⬜ 시나리오 3: 비정상 종료 후 5분 임계 강제 점유 — Windows 강제 종료 → Mac에서 5분 경과 후 강제 점유 → 데이터 정합 확인
- ⬜ 검증 항목: app.lock 해제/점유, DB 정합(원생/출결/청구), salt.bin 동기화, 백업 파일 동기화
- ⬜ T2(DB 폴더 변경) 후 양 PC 경로 인식 정합 확인

### T6: 실사용 개시 준비 (2h) — MUST

> 격식 UAT 없이 원장이 바로 v1.0을 실사용 개시한다. T3(양 OS 빌드) + T4(양 PC 동기화) 통과 후 실행.

- ⬜ 교습소 PC(Windows)에 `.msi` 인스톨러 설치 확인
- ⬜ 자택 Mac에 `.dmg` 인스톨러 설치 확인
- ⬜ T1(CSV 가져오기)으로 원생 실데이터 이관 완료 확인
- ⬜ 클라우드 동기화 폴더(MYBOX) 정상 동작 확인
- ⬜ 양 OS 앱 기동 + PIN 잠금해제 + 대시보드 진입 확인

### T7: 초기 실사용 피드백 대응 버퍼 (4h) — MUST

> 실사용 개시 후 수집되는 피드백을 우선순위별로 대응한다. 2주 고정 기간 없이 지속적으로 수집.
> 피드백 분류 기준:
> - **Critical**: 기능 오류 (앱 크래시, 데이터 손실, 저장 실패) → 즉시 수정
> - **High**: UX 장애 (글씨 안 보임, 버튼 못 누름, 동선 혼란) → Sprint 내 수정
> - **Medium**: 미세 조정 (색상, 간격, 문구 변경) → Capacity 내 수정 또는 Post-MVP
> - **Low**: 희망 사항 → Post-MVP backlog 기록

- ⬜ 실사용 중 발견된 Critical/High 피드백 즉시 수정
- ⬜ 피드백 기록 + 분류 (화면별 사용성, 글씨 크기, 동선)

### T8: 접근성 잔여 개선 (4h) — SHOULD

- ⬜ 밀집 UI 클릭 영역 44x44px 미달 항목 수정 — 사이드바 메뉴, 테이블 셀 버튼, 필터 드롭다운 등
- ⬜ `text-gray-500` 잔여 항목 → WCAG AA 대비 수정 (Sprint 15 T3에서 gray-400→600 수정 완료, gray-500 잔여 점검)
- ⬜ F1 도움말 단축키 구현 — 현재 화면 컨텍스트에 따른 도움말 다이얼로그 또는 가이드 표시
- ⬜ A99 Ctrl+N 방어 로직 추가 시 함께 입력 필드 방어 통합 검증

### T9: 공지문 I/O 병렬화 (3h) — SHOULD

> 50장 일괄 생성 시 I/O 병목 개선. `ntfs-power-loss-pattern` 메모리 참조.

- ⬜ `notice-generator.ts` 일괄 생성 엔진에 `Promise.allSettled` 기반 병렬 저장 (동시성 4~8개 제한)
- ⬜ 개별 저장 실패 시 부분 성공 보고 (전체 실패 아닌 건별 결과)
- ⬜ NTFS power-loss 대응: `fs::write` 후 `fsync` 호출 검토 (Tauri fs 플러그인 제약 확인)

### T10: v1.0 릴리즈 준비 (3h) — MUST

> 실제 배포(태그 push)는 사용자 명시 지시까지 대기. 준비만 완료.

- ⬜ `CHANGELOG.md` v1.0.0 릴리즈 노트 작성 — Sprint 1~16 전체 기능 요약
- ⬜ `package.json` + `src-tauri/Cargo.toml` 버전 → `1.0.0` 업데이트
- ⬜ `README.md` 프로덕션 정보 갱신 (스크린샷, 설치 방법, 시스템 요구사항)
- ⬜ GitHub Actions `deploy.yml` 최종 확인 — `v*` 태그 push 시 양 OS 인스톨러 빌드 정상 동작 확인
- ⬜ 배포 대기 상태 문서화 — `DEPLOY.md` 체크리스트 준비, 사용자 지시 대기 명시

### T11: 통합 검증 (3h) — MUST

- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ⬜ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ⬜ 마이그레이션 self-check: 신규 마이그레이션 여부 확인 + `.sqlx/` 캐시 갱신
- ⬜ develop HEAD에 Sprint 16 전체 변경 반영 확인

---

## 실사용 개시 전 검증 체크리스트

실사용 개시(T6) 전에 아래 핵심 흐름이 양 OS에서 정상 동작하는지 확인한다.

### 핵심 흐름 검증 (T4 빌드 + T5 동기화 시 수행)
- ⬜ 앱 시작 → PIN 잠금해제 → 대시보드 진입 (양 OS)
- ⬜ 원생 등록/수정/조회 전체 흐름
- ⬜ 출결 생성 + 출결표 + 토글
- ⬜ **수업일 변경 케이스1**: 특정일 1회성 이동 → 원래 날 비고 + 새 날 출석 + 메모, 청구 불변, OFF일/공휴일 도착 차단
- ⬜ **수업일 변경 케이스2**: 특정일 이후 영구 변경 → 변경일 이전 출결 유지 + 이후 신 스케줄 재생성 + 결석/보강 보존 경고
- ⬜ 청구 생성 + 확정 + 수납
- ⬜ 공지문 이미지 일괄 생성
- ⬜ CSV 가져오기 (이관 데이터 정합)
- ⬜ DB 폴더 변경 + 변경 후 정상 기동
- ⬜ 양 PC 전환 데이터 정합 (Win→Mac, Mac→Win)

---

## 확정 사항 (사용자 결정 완료)

| # | 항목 | 결정 | 결정일 |
|---|------|------|--------|
| PI-16 | DB 폴더 변경(경로 재지정) | **Sprint 16 포함 (MUST)** — T3로 배정. salt.bin 이전 + copy-then-switch 구현 | 2026-06-08 |
| PI-18 | UAT 방식 | **격식 2주 파일럿 폐기** — 원장이 바로 실사용 개시, 피드백은 실사용 중 수집 | 2026-06-08 |
| PI-20 | 수업일 변경 케이스1 구현 방식 | **출결 행 이동** — `event_date` UPDATE + 메모 기록, 동월 한정, 청구 불변 (T0) | 2026-06-08 |
| PI-21 | 수업일 변경 케이스2 처리행 보존 | **미처리(present)만 재생성** — 결석/보강/메모 행 보존 + 보존 건수 경고 (데이터 손실 0) (T0) | 2026-06-08 |
| PI-24 | 케이스2 변경일 지정 방식 | **원장이 변경일 직접 지정 + 사전(미래)·사후(과거) 양방향 허용** — 변경일부터 신 스케줄 반영. 사후는 소급 재생성, 사전은 향후 generate 자동 반영. 변경일 하한=현행 effective_from·입교일 (T0) | 2026-06-08 |
| PI-25 | 케이스1 도착일이 OFF/공휴일일 때 | **차단** — 공휴일·정규수업 OFF 일자로의 1회성 이동은 거부 + 안내 (운영 규칙 일관성) (T0) | 2026-06-08 |
| PI-26 | 케이스1 이동 UI 조작 방식 | **present 셀 우클릭 → [수업일 이동 / 보강 등록] 메뉴 → 수업일 이동 → 달력 다이얼로그**. 좌클릭(출결 토글)은 기존 유지. 드래그앤드롭 아님. 기존 우클릭 단일 보강등록 동작을 메뉴로 확장 (T0) | 2026-06-08 |
| PI-27 | 케이스1 도착일 선택 컨트롤 | **월 달력 컨트롤(시각적 달력 위젯)** — `type="date"` 텍스트 입력이 아닌 그리드 달력. 기존 `ThreeMonthCalendar`/`CalendarCell` 패턴 재활용(신규 의존성 회피). OFF일/공휴일/충돌일/타월은 비활성 셀 (T0) | 2026-06-08 |
| PI-28 | 케이스1 이동 시 시작시간 | **이동 다이얼로그에서 수업 시작시간 입력** — 시각 검증 발견: 이동 출결은 시간 부재로 캘린더 주/일 뷰 표시 불가(크래시). V307 `regular_attendances.start_time` 추가, move 시 입력 시간 저장, calendar는 `COALESCE(ra.start_time, ss.start_time)` (T0) | 2026-06-08 |
| PI-22 | 케이스2 월 중 변경 청구 여파 | **현행 유지 + 안내만** — 주당시간 변동 시 재확인 토스트, 금액은 청구 화면에서 수동 조정 (T0) | 2026-06-08 |
| PI-23 | 수업일 변경 우선순위 | **T0 최우선** — `/sprint-dev 16` 착수 시 가장 먼저, 기존 Task 한 칸씩 밀림 | 2026-06-08 |

## 미결정 항목 (Pending Items)

사용자 결정이 필요한 항목입니다. 스프린트 진행 중 확인 요청합니다.

| # | 항목 | 필요 시점 | 옵션 | 기본값 |
|---|------|----------|------|--------|
| PI-17 | 출결표 N+1 쿼리 재설계 실행 여부 | 실사용 피드백 후 | A: Sprint 16 T7 버퍼에서 수행 / B: Post-MVP | **B: Post-MVP** — 현재 PRD 성능 기준(50명x31일 < 1초) 충족. 실데이터에서 성능 저하 확인 시 A로 전환 |
| PI-19 | 셀 memo (출결표 셀별 메모) | 실사용 피드백 | A: Sprint 16 T7에서 구현 / B: Post-MVP | **B: Post-MVP** — 명시적 요청 시 A로 전환. T0에서 추가하는 `note` 컬럼을 향후 재활용 가능 |

---

## 이연 확정 항목 (Post-MVP backlog)

아래 항목은 Sprint 16 범위에서 제외하며 Post-MVP로 이연한다.

| 항목 | 사유 |
|------|------|
| 출결표 N+1 재설계 | 현재 성능 기준 충족. PI-17으로 실측 후 재판단 |
| makeup_attendances 인덱스 | 출결표 N+1과 연계. 실측 데이터 필요 |
| 셀 memo | 실사용 피드백 없으면 불필요. PI-19 |
| A89 notices UI 구획화 | 로직 분리 완료, UI 3분할만 잔여. Capacity 부족으로 이연 |
| E2E 자동화(UC-1~UC-5) | Tauri WebDriver 별도 인프라 세팅 8~12h |
| 한글 자모 부분 일치 검색 | hangul-js 라이브러리 또는 직접 분해 알고리즘. Nice-to-have |
| 반응형 폰트/셀 너비 | clamp() viewport 패턴. 현재 18px 고정으로 충분 |
| A96 복원 리허설 dev 환경 개선 | Low 우선순위 |
| query!() 매크로 전환 | 동적 query+bind 패턴 유지 중. 별도 backlog |

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ **수업일 변경 케이스1**: 1회성 출결 행 이동 동작 — 동월 한정·메모 기록·present 외 거부·to_date 충돌 차단, 청구/주당시간 불변
- ⬜ **수업일 변경 케이스2**: 변경일 이후 미처리(present)만 재생성 + 결석/보강/메모 행 보존 + 보존 건수 경고, 변경일 이전 출결 불변
- ⬜ **출결 생성 날짜 인식**: `generate`가 날짜별 유효 스케줄(effective_from/to) 반영
- ⬜ V306 마이그레이션 적용 + `.sqlx/` 캐시 갱신 + self-check 1:1 일치
- ⬜ 양 OS 인스톨러(.dmg / .msi) 설치/실행/삭제 정상
- ⬜ 양 PC 동기화 시나리오 최소 2종 통과 (Win→Mac, Mac→Win)
- ⬜ CSV 가져오기로 원생 실데이터 이관 성공
- ⬜ DB 폴더 변경(경로 재지정) 정상 동작 — copy-then-switch + salt.bin/백업 동반 이전
- ⬜ DB 폴더 변경 후 양 PC 경로 인식 정합 확인
- ⬜ 실사용 개시 완료 (양 OS 기동 + 데이터 이관 + 핵심 흐름 확인)
- ⬜ 초기 실사용 피드백 Critical/High 전수 반영
- ⬜ `cargo test` 전체 통과 (예상 390+ tests — T0 신규 단위 테스트 포함)
- ⬜ `cargo clippy --all-targets -- -D warnings` clean
- ⬜ `cargo check --features cipher` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ⬜ CHANGELOG.md v1.0.0 작성 완료
- ⬜ 버전 번호 1.0.0 업데이트 (package.json + Cargo.toml)

**배포 대기 (사용자 명시 지시 후)**
- ⬜ v1.0.0 태그 push → GitHub Actions 인스톨러 빌드
- ⬜ GitHub Release 생성 + 양 OS 인스톨러 첨부
- ⬜ 배포 후 CV 체크리스트 통과

**프로세스**
- ⬜ ROADMAP.md Sprint 16 상태 업데이트
- ⬜ sprint-close 에이전트: 문서화 + develop 머지
- ⬜ sprint-review 에이전트: 코드 리뷰 + 회고

---

## 참고 사항

### 의존성
- **T0(수업일 변경)** → 독립·최우선. 기존 출결/스케줄 도메인 위에서 동작, 다른 Task에 선행 의존 없음. 착수 시 가장 먼저 단독 수행
- T2(CSV 가져오기) → T6(실사용 개시): CSV 임포트 완료 후 데이터 이관 확인
- T3(DB 폴더 변경) → T5(양 PC 동기화): 폴더 변경 후 양 PC 경로 인식 검증
- T4(양 OS 빌드) → T6(실사용 개시): 인스톨러 확보 후 교습소 PC 설치
- T5(양 PC 동기화) → T6(실사용 개시): 동기화 검증 통과 후 실사용 개시
- T6(실사용 개시) → T7(피드백 대응): 실사용 시작 후 피드백 발생

### 작업 순서 (권장)
1. **T0(수업일 변경 도메인) — 최우선 단독 집중 (8h)**. V306 → 케이스1 → 케이스2 → UI 순
2. T1(회고 액션) + T2(CSV 가져오기) — 병렬 착수
3. T3(DB 폴더 변경) — 핵심 MUST, 단독 집중 (8h)
4. T4(양 OS 빌드)
5. T5(양 PC 동기화) — T3 완료 후 폴더 변경 시나리오 포함
6. T6(실사용 개시 준비) — T2/T4/T5 통과 후
7. T7(피드백 대응) — 실사용 개시 후 지속
8. T8(접근성)·T9(공지문 I/O) — SHOULD, MUST 초과로 **Post-MVP 이연 기본**. 잔여 Capacity 시에만 착수
9. T10(릴리즈 준비) + T11(통합 검증) — 스프린트 마지막

### 기술 고려사항
- **수업일 변경(T0)**: `student_schedules`의 effective_from/to 시계열 구조 활용. `generate_impl`의 스케줄 조회를 날짜 인식(`effective_from ≤ d AND (effective_to IS NULL OR d < effective_to)`)으로 확장 — effective_to는 **exclusive**. 케이스1은 `regular_attendances` 1행의 `event_date` UPDATE(+`note`), 케이스2는 변경일 이후 present 행 DELETE 후 재생성(처리행 보존). 보강 연결 행은 보존 대상이므로 고아 데이터 미발생. 청구는 현행 스케줄 주당시간 기반이라 케이스2 주당시간 변동 시 토스트 안내만, 금액은 수동 조정
- CSV 가져오기 인코딩: 한국어 엑셀 기본 CSV는 EUC-KR. `encoding_rs` crate으로 자동 감지 필요 (신규 의존성)
- DB 폴더 변경: copy-then-switch 전략. WAL checkpoint 후 복사 필수. `ntfs-power-loss-pattern` 적용 (fsync). cipher ON 환경에서 복사 후 정상 열기 검증 필수
- DB 폴더 변경 양 PC 정합: config.json이 클라우드 동기화 대상이므로, 한 PC에서 경로 변경 시 다른 PC도 새 경로 인식 필요. config.json 저장 위치(app_config_dir = PC별 로컬)와 cloud_folder 경로의 분리 확인 필수
- salt.bin 이전: Keychain 키는 유지 (`keyring-v3-features-trap` — features 명시 확인). salt.bin 손상 감지 `is_corrupted()` 적용
- 공지문 I/O 병렬화: `ntfs-power-loss-pattern` 메모리 참조 — atomic write 시 `fsync` 호출 검토
- `cipher` feature: 프로덕션 빌드에서만 활성화. 인스톨러는 `--features cipher`로 빌드
- 배포(deploy-prod): 사용자 명시 지시 전까지 절대 진행하지 않음

### 신규 의존성 (예상)
- T0(수업일 변경): **신규 의존성 없음** (기존 `chrono` + `sqlx` 활용, 도착일 달력은 기존 `ThreeMonthCalendar`/`CalendarCell` grid 패턴 재활용 — react-day-picker 미도입). 마이그레이션 V306 1건 추가
- `encoding_rs` (Rust) — CSV EUC-KR 자동 감지 (T2에서 필요 시)
- `csv` (Rust) — CSV 파싱 (이미 sqlx 의존성 트리에 포함 가능성 확인 필요)
