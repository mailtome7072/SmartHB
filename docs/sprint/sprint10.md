# Sprint Plan sprint10

## 기간
2026-05-26 ~ 2026-06-08 (2주, 예상)

## Phase 위치
**Phase 3: 보강 + 소멸 (Sprint 9~10)** — 두 번째이자 마지막 스프린트. Phase 3 완료 후 Phase 4(청구+수납+공지문)로 전환한다.

## 목표
1. **소멸 자동 전이 완성** — `makeup_deadline` 도래 시 미보강 결석을 `makeup_expired` 상태로 자동 전이하는 batch 로직 구현 (앱 시작/출결 생성/교습기간 등록 3개 트리거)
2. **보강소멸 환원 + 퇴교 보강 처리** — 소멸 → 결석 수동 환원(AC-4.5-5) + 퇴교 시 미사용 보강 처리 다이얼로그(PRD §4.5.9)
3. **선행 수업 처리** — 미래 일자 결석 사전 등록 + 선행 보강 매칭(PRD §4.2.3)
4. **수업 관리 캘린더 뷰** — 일/주/월 뷰 + 원생 상세 팝업 + 보강 관리 전용 뷰(PRD §4.6)
5. **Sprint 9 dead code 정리** — `mark_makeup_absent` / `batch_create_makeups` 폐기 코드 제거

## ROADMAP 연계 기능
- PI-01 소멸 자동 전이 트리거 구현 (ROADMAP Sprint 10)
- 보강소멸 → 결석 수동 환원 §4.5.3 (ROADMAP Sprint 10)
- 퇴교 시 미사용 보강 처리 §4.5.9 (ROADMAP Sprint 10)
- 선행 수업 처리 §4.2.3 (ROADMAP Sprint 10)
- 수업 관리 캘린더 뷰 §4.6 (ROADMAP Sprint 10)

## 미결정 항목 (PI)
| ID | 항목 | 결정 시점 | 비고 |
|----|------|-----------|------|
| PI-03 | 캘린더 라이브러리 선택 (FullCalendar vs React Big Calendar) | T8 진입 시 ADR | 번들 크기, 라이선스, 커스텀 렌더러 지원도 비교. skill: brainstorming |
| PI-04 | 보강데이 일괄 등록 버튼 범위 | T11 진입 시 사용자 확인 | Sprint 9에서 BatchMakeupDialog 폐기(J7). 캘린더 보강관리뷰에서 진입점만 제공할지, 새 UI 필요한지 확인 |

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint9-retrospective.md` (A49~A57)

| ID | 항목 | 우선순위 | Sprint 10 반영 방법 |
|----|------|----------|---------------------|
| A49 | `mark_makeup_absent` + `batch_create_makeups` dead code 정리 | High | **T1**에서 처리 — IPC handler 등록 해제 + audit variant 정리 + 단위 테스트 폐기 |
| A50 | 시각 검증 capacity buffer 분리 | High | Capacity 계획에 "시각 검증 버퍼 6h" 별도 예약 (아래 Capacity 섹션) |
| A51 | 도메인 규칙 중 운용 관행 교차 항목 T1에서 사용자 확인 선행 | Medium | **T2** 설계 단계에서 소멸 전이 트리거 시점 + 선행 수업 운용 방식 사용자 확인 |
| A52 | `get_absence_history` pagination 적용 (R64) | Medium | 이번 sprint scope 외 — Phase 4 이후 (PRD 50명 규모 안전) |
| A53 | `create_makeup_with_absences_impl` JSON_EACH 전환 (R65) | Low | 이번 sprint scope 외 — 성능 영향 미미 |
| A54 | `get_attendance_grid` N+1 batch 쿼리 (R42 이월) | Medium | 이번 sprint scope 외 — Phase 4 이후 |
| A55 | salt buffer ZeroizeOnDrop (R48-b) | Medium | 이번 sprint scope 외 — 보안 도메인 |
| A56 | 반응형 폰트/셀 너비 clamp() (A45 이월) | Medium | 이번 sprint scope 외 — UX 전반 |
| A57 | 한글 자모 부분 일치 검색 (A46 이월) | Medium | 이번 sprint scope 외 — 검색 도메인 |

**이번 Sprint 적용**: A49(T1), A50(Capacity), A51(T2 설계 패턴)
**이연 사유**: A52~A57은 보강/소멸 도메인과 무관하거나 PRD 요건 충족 중이므로 Phase 4+ 이후 검토

---

## 작업 목록

### T1: Sprint 9 dead code 정리 (carry-over) — 2h ✅ (2026-05-26, `dde74aa`)
> A49 반영. 폐기된 IPC 2종 + audit variant + 단위 테스트 제거

**작업 내용**:
1. `src-tauri/src/commands/makeup.rs`: `mark_makeup_absent`, `batch_create_makeups` 함수 삭제
2. `src-tauri/src/lib.rs` invoke_handler: 해당 커맨드 등록 제거
3. `src-tauri/src/commands/audit.rs`: `MakeupAbsent` variant 삭제 (참조 없으면)
4. `src-tauri/src/commands/makeup.rs` 테스트: 관련 단위 테스트 삭제
5. (선택) `makeup_attendances.status` CHECK 제약에서 `makeup_absent` 값 제거 — V108 마이그레이션
6. `src/lib/tauri/index.ts`: 이미 제거 확인 (Sprint 9 T12에서 삭제됨)

**AC**:
- `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- dead code warning 0건

**검증**: `cargo test` + `cargo clippy`

---

### T2: 소멸 자동 전이 설계 + 사용자 확인 — 2h ✅ (2026-05-26)
> A51 반영. 도메인 규칙 중 운용 관행 교차 항목을 사전에 사용자와 수렴

**작업 내용**: scope.md Session #2 참조

**사용자 결정 (2026-05-26, PI-05~PI-09)**:
| ID | 결정 |
|----|------|
| PI-05 | 트리거 3개소(앱 시작 + 출결 생성 + 교습기간 등록) |
| PI-06 | 소멸 판정 기준일 = 오늘 (chrono::Local::now), 테스트는 Option<NaiveDate> 주입 |
| PI-07 | V108 마이그레이션 진행 → **T1' 신규 task** |
| PI-08 | 선행 수업 = 기존 상태 토글 흐름 활용 → **T7 범위 축소** |
| PI-09 | 자동 전이 알림 = 토스트 (건수 > 0일 때만) |

**AC**: ✅ 모든 PI 결정 + scope.md 기록 완료

---

### T1': V108 마이그레이션 — makeup_attendances.status CHECK 정리 (PI-07 결정) — 0.5h ✅ (2026-05-26)

**작업 내용**:
1. `src-tauri/migrations/108__cleanup_makeup_status_check.sql` 신규
2. `makeup_attendances.status` CHECK 제약에서 `'makeup_absent'` 값 제거
3. SQLite CHECK ALTER 미지원 → 테이블 rename + 재생성 + INSERT SELECT
4. `.sqlx/` 오프라인 캐시 갱신 후 커밋
5. cargo test 통과 확인 (데이터 0건이므로 안전)

**AC**:
- V108 적용 후 CHECK 제약 단순화 (`status = 'makeup_attended'`)
- 기존 `cargo test` 통과 (Sprint 9 J5 폐기 결정 후 makeup_absent 사용 코드 0건)

**검증**: `cargo test` + `cargo clippy` + `.sqlx/` 캐시 검증

---

### T3: 소멸 자동 전이 백엔드 IPC — 4h
> PI-01 구현. 독립 모듈 `src-tauri/src/commands/expiration.rs` 신규

**작업 내용**:
1. `expiration.rs` 신규 모듈 생성 — `expire_overdue_absences` 핵심 함수
   - 입력: DB pool, 기준일(Option — None이면 오늘)
   - 로직: `regular_attendances` WHERE `status='absent'` AND `makeup_attendance_id IS NULL` AND `makeup_deadline` 도래 → `status='makeup_expired'` UPDATE
   - 소멸기한 판정: `makeup_deadline`(년월) ≤ 기준월 AND 해당 교습기간 종료일 ≤ 기준일
   - 반환: 전이된 레코드 수 + 전이 상세 리스트 (원생명, 결석일, 소멸기한)
2. `expire_overdue_absences` IPC 커맨드 (수동 호출용)
3. `MakeupExpired` audit variant 추가
4. 단위 테스트 (최소 6건):
   - 소멸기한 도래 + 미보강 → 전이 성공
   - 소멸기한 미도래 → 전이 없음
   - 이미 `makeup_done` → 전이 대상 아님
   - 이미 `makeup_expired` → 중복 전이 없음
   - 교습기간 미등록 월 → 전이 보류 (대기)
   - 복수 원생 batch 전이

**AC**:
- `expire_overdue_absences` 단위 테스트 6건+ 통과
- 트랜잭션 내 원자적 실행
- audit 로그 기록

**검증**: `cargo test` + `cargo clippy`

---

### T4: 소멸 전이 트리거 통합 — 3h
> 3개 트리거 지점에 `expire_overdue_absences` 호출 삽입

**작업 내용**:
1. **앱 시작 시**: `src-tauri/src/lib.rs` 또는 별도 startup 모듈에서 DB 연결 직후 호출
2. **출결 생성 시**: `attendance.rs::generate_attendances` 종료 직전에 해당 월 소멸 체크
3. **교습기간 등록 직후**: `schedules.rs` 교습기간 생성/수정 커맨드 종료 직전에 해당 월 소멸 체크
4. TypeScript IPC 래퍼: `expireOverdueAbsences` 추가 (`src/lib/tauri/index.ts`)
5. 프론트엔드 앱 초기화 시 호출 (로딩 화면 또는 레이아웃 mount)

**AC**:
- 앱 시작 → 소멸기한 도래 결석이 자동 전이됨
- 출결 생성 → 소멸기한 도래 결석이 전이됨 (PRD §4.5.1 4번째 동작)
- 교습기간 등록 → 해당 월 소멸기한 확정 + 전이 실행
- 전이 결과가 사용자에게 토스트/알림으로 표시 (건수 > 0일 때)

**검증**: 통합 시나리오 (출결 생성 후 소멸 체크 → 토글 차단 확인)

---

### T5: 보강소멸 → 결석 수동 환원 IPC — 3h
> PRD §4.5.3, AC-4.5-5 구현

**작업 내용**:
1. `expiration.rs` 또는 `makeup.rs`에 `revert_expired_to_absent` 함수
   - 입력: `attendance_id` (makeup_expired 상태인 정규 출결 ID)
   - 로직: `status='absent'`, `makeup_deadline` 재설정 (현재 월 + 1 또는 원래 값 복원)
   - audit 로그: `MakeupExpiredReverted` variant
2. IPC 커맨드 등록
3. TypeScript 래퍼: `revertExpiredToAbsent`
4. 단위 테스트 (최소 4건):
   - makeup_expired → absent 환원 성공
   - absent/present/makeup_done 상태에서 호출 시 거부
   - 환원 후 makeup_deadline 재설정 확인
   - audit 로그 기록 확인

**AC**:
- AC-4.5-5: 보강소멸 → 결석 환원 시 확인 다이얼로그 필수 (UI는 T9)
- 환원 후 보강필요시간 재산출 정합성 유지

**검증**: `cargo test` + `cargo clippy`

---

### T6: 퇴교 시 미사용 보강 처리 IPC — 3h
> PRD §4.5.9 구현

**작업 내용**:
1. `students.rs` 또는 `expiration.rs`에 퇴교 보강 처리 함수:
   - `get_pending_makeup_for_withdrawal(student_id)`: 미보강 결석 리스트 + 잔여 보강필요시간 조회
   - `process_withdrawal_makeup(student_id, choice)`: 3가지 선택지 처리
     - `immediate_expire`: 전체 미보강 → `makeup_expired` 전이
     - `defer_withdrawal`: 퇴교일 보류 (퇴교 취소)
     - `external_expire(memo)`: 사유 메모 + 전체 → `makeup_expired` 전이
2. IPC 커맨드 2종 등록
3. TypeScript 래퍼 2종
4. 단위 테스트 (최소 5건):
   - 미보강 결석 조회 정확성
   - 즉시 소멸 → 전체 makeup_expired
   - 보강 진행 후 퇴교 → 퇴교일 미변경
   - 외부 처리 → memo 저장 + 전체 makeup_expired
   - 보강필요시간 0인 원생 → 다이얼로그 미표시 (조회 결과 빈 리스트)

**AC**:
- 퇴교 처리 시 미사용 보강 보유 원생에게 처리 다이얼로그 표시 (UI는 T10)
- 3가지 선택지 각각 정확히 동작
- audit 로그 기록

**검증**: `cargo test` + `cargo clippy`

---

### T7: 선행 수업 처리 IPC — 2h
> PRD §4.2.3 구현

**작업 내용**:
1. 기존 `toggle_attendance` 확장 또는 별도 `register_advance_absence` 함수:
   - 미래 일자 결석 사전 등록 (출결 생성 전 월의 특정 일자에 결석 레코드 수동 생성)
   - 선행 보강 매칭: 기존 `create_makeup_with_absences`로 현재 일자 보강 + 미래 결석 매칭
2. IPC 커맨드 등록 (필요 시)
3. TypeScript 래퍼
4. 단위 테스트 (최소 3건):
   - 미래 일자 결석 등록 성공
   - 미래 결석 + 현재 보강 매칭 성공
   - 이미 출결 생성된 일자에 중복 등록 방지

**AC**:
- 미래 일자 결석 사전 등록 → 선행 보강 매칭 동작
- 기존 보강 흐름과 충돌 없음

**검증**: `cargo test` + `cargo clippy`

---

### T8: 캘린더 라이브러리 ADR + 백엔드 집계 IPC — 4h · skill: brainstorming
> PI-03 결정 + 캘린더 뷰용 백엔드 데이터 집계

**작업 내용**:
1. **ADR 작성**: FullCalendar vs React Big Calendar
   - 비교 기준: 라이선스 (MIT vs 상용), 번들 크기, 일/주/월 뷰 지원도, 커스텀 렌더러 (원생 이름+시간 셀), TypeScript 지원, Tauri static export 호환성
   - `docs/arch/adr-{NNN}-calendar-library.md` 저장
2. **캘린더 데이터 집계 IPC** — `get_calendar_data(year_month)`:
   - 일별 시간대별 수업 원생 목록 (원생명, 시작/종료 시간, 정규/보강 구분)
   - AC-4.6-1: 시간대별 인원 = 시작 원생 + 진행 중 원생 합산
3. **보강 관리 뷰 IPC** — `get_makeup_management_data(year_month)`:
   - 보강 필요 원생 리스트 (소멸기한 임박 순 정렬)
   - 소멸 임박 판정: 교습기간 종료일 - 7일 이내
4. IPC 등록 + TypeScript 래퍼
5. 단위 테스트 (최소 4건)

**AC**:
- ADR 문서 작성 완료 + 사용자 확인
- 캘린더 데이터 집계 IPC 정확성
- 보강 관리 데이터 정렬/강조 기준 일치

**검증**: `cargo test` + ADR 리뷰

---

### T9: 소멸 환원 UI + 소멸 알림 UI — 3h · skill: frontend-design
> T5 백엔드 연결 + 소멸 전이 결과 알림

**작업 내용**:
1. **소멸 환원 다이얼로그** — 출결표에서 `makeup_expired` 셀 클릭 시 표시
   - 확인 다이얼로그: "보강소멸 상태를 결석으로 환원하시겠습니까?" (AC-4.5-5)
   - 환원 성공 시 TanStack Query 무효화 → 출결표 갱신
2. **소멸 전이 결과 토스트** — 앱 시작/출결 생성 후 전이 건수 표시
   - "소멸 처리된 결석이 N건 있습니다" 토스트 (건수 > 0일 때만)
3. 출결표 `makeup_expired` 셀 스타일 확인 (Sprint 8에서 이미 구현 — gray 배경)

**AC**:
- AC-4.5-5: 환원 시 확인 다이얼로그 필수
- 환원 후 출결표 즉시 반영
- 소멸 전이 결과 사용자에게 시각적 피드백

**검증**: 시각 검증

---

### T10: 퇴교 보강 처리 UI — 3h · skill: frontend-design
> T6 백엔드 연결

**작업 내용**:
1. **퇴교 보강 처리 다이얼로그** — 원생 퇴교 버튼 클릭 시 미사용 보강 보유 시 표시
   - 표시: 원생명, 남은 보강필요시간, 미보강 결석 일자 리스트
   - 3가지 선택지 버튼: "즉시 소멸" / "보강 후 퇴교" / "외부 처리 후 소멸"
   - "외부 처리 후 소멸" 선택 시 사유 메모 입력 textarea
2. 기존 원생 관리 페이지의 퇴교 흐름에 통합
3. TanStack Query 무효화

**AC**:
- 퇴교 시 미사용 보강 보유 원생에게만 다이얼로그 표시
- 3가지 선택지 각각 정상 동작
- 보강필요시간 0인 원생은 다이얼로그 없이 바로 퇴교 처리

**검증**: 시각 검증

---

### T11: 캘린더 뷰 UI (일/주/월) — 6h · skill: frontend-design
> PRD §4.6.1~4.6.3 구현. T8 ADR 결과 라이브러리 적용

**작업 내용**:
1. 캘린더 라이브러리 설치 (ADR 결과에 따라)
2. `/calendar` 페이지 신규 — 사이드바 메뉴 추가
3. **일/주/월 뷰 전환**: Outlook 스타일
   - 날짜 셀 상단: 시간대별 총 수업 인원수
   - 시간대 구분선(1시간) 아래 원생 이름 + 당일 수업 시간 (한 줄 3명씩)
4. **원생 상세 팝업** (§4.6.2):
   - 이름, 학년, 정규/보강 구분, 시작·종료 시간
   - 해당 월 결석일, 미수업 시간, 소멸기한, 잔여 보강필요시간
   - "출결/보강관리" 이동 버튼
5. **보강 관리 전용 뷰** (§4.6.3):
   - 보강 필요 원생 리스트 (소멸기한 임박 순)
   - 소멸 임박(7일 이내) 행 강조 (색상/아이콘)
   - 보강데이 일괄 진입 버튼 → PI-04에서 연결 대상 결정
6. TanStack Query로 데이터 페칭

**AC**:
- AC-4.6-1: 시간대별 인원수 정확 (시작 + 진행 중 합산)
- AC-4.6-2: 소멸 임박 데이터 시각 식별 가능
- 일/주/월 뷰 전환 동작
- 원생 상세 팝업 → 출결관리 이동 동작

**검증**: 시각 검증

---

### T12: 통합 검증 + 자동 검증 — 3h

**작업 내용**:
1. 자동 검증 7항목:
   - `cargo test` cipher off / cipher on
   - `cargo clippy` cipher off / cipher on
   - `pnpm lint`
   - `pnpm tsc --noEmit`
   - `pnpm build`
2. 마이그레이션 self-check (A39): scope.md 설계 vs 실제 migrations 1:1 대조
3. 통합 시나리오 검증:
   - 결석 → 소멸기한 도래 → 자동 전이 → 소멸 환원 → 재보강 등록
   - 퇴교 보강 처리 3가지 선택지 각각
   - 선행 수업: 미래 결석 → 현재 보강 매칭
   - 캘린더 뷰: 일/주/월 전환 + 원생 팝업 + 보강관리 뷰
4. sprint-review 산출물 경로 확인 (A40)

**AC**:
- 자동 검증 7/7 통과
- 통합 시나리오 4개 전수 통과
- sprint-review 산출물 경로 명시

**검증**: 자동 검증 + 시각 검증

---

## Capacity

| 항목 | 시간 |
|------|------|
| T1: dead code 정리 ✅ | 2h (실측 1.5h) |
| T2: 소멸 설계 + 사용자 확인 ✅ | 2h |
| T1': V108 makeup_status CHECK 정리 (PI-07) | 0.5h |
| T3: 소멸 자동 전이 IPC | 4h |
| T4: 소멸 트리거 통합 | 3h |
| T5: 소멸 환원 IPC | 3h |
| T6: 퇴교 보강 처리 IPC | 3h |
| T7: 선행 수업 IPC | 2h |
| T8: 캘린더 ADR + 집계 IPC | 4h |
| T9: 소멸 환원/알림 UI | 3h |
| T10: 퇴교 보강 UI | 3h |
| T11: 캘린더 뷰 UI | 6h |
| T12: 통합 검증 | 3h |
| **소계 (구현)** | **38.5h** (T1' +0.5h) |
| **시각 검증 버퍼 (A50)** | **6h** |
| **총계** | **44.5h** |

- 팀: 1인 개발 + AI 보조
- 2주 스프린트, 일 4h 실작업 = 40h 기본 + 6h 버퍼 = 46h 가용
- **44h / 46h = 95.6%** — 적정 (Sprint 9 교훈: 시각 검증 버퍼 분리로 초과 방지)

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ 소멸 자동 전이가 앱 시작 / 출결 생성 / 교습기간 등록 3개 트리거에서 정상 발동
- ⬜ 보강소멸 → 결석 환원 시 확인 다이얼로그 동작 (AC-4.5-5)
- ⬜ 퇴교 처리 다이얼로그 3개 선택지 모두 동작 (PRD §4.5.9)
- ⬜ 선행 수업: 미래 결석 → 현재 보강 매칭 동작 (PRD §4.2.3)
- ⬜ 캘린더 뷰 일/주/월 전환 + 원생 팝업 + 보강관리 뷰 동작 (PRD §4.6)
- ✅ Sprint 9 dead code 0건 (mark_makeup_absent + batch_create_makeups 완전 제거) — T1 완료 (Session #1, 2026-05-26)
- ⬜ `cargo test` 전체 통과 (cipher off/on)
- ⬜ `cargo clippy -- -D warnings` clean (cipher off/on)
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- ⬜ 마이그레이션 self-check 통과 (A39)
- ⬜ 소멸 + 보강 비즈니스 규칙 단위 테스트 신규 18건+ 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md Phase 3 완료 표기
- ⬜ CHANGELOG.md 업데이트

---

## 의존성 및 리스크

| ID | 설명 | 영향도 | 대응 |
|----|------|--------|------|
| R66 | 소멸 전이 batch가 앱 시작 시 DB lock 경합 — Single Instance(Sprint 5) + app.lock(ADR-002)으로 동시 접근 차단되어 있으나, startup 지연 우려 | 중간 | batch를 spawn_blocking + 진행 표시 UI로 비동기 실행 |
| R67 | 캘린더 라이브러리가 static export(Next.js output: 'export')에서 SSR 관련 에러 | 중간 | dynamic import + 'use client' 적용. ADR에서 static export 호환성 검증 포함 |
| R68 | 교습기간 미등록 월의 소멸기한 판정 — 실제 소멸일 = 해당 월 교습기간 종료일인데 교습기간 없으면 판정 불가 | 중간 | T2 설계에서 "교습기간 미등록 → 소멸 보류" 정책 확정 (PRD §4.5.7 "소멸기한 미확정" 표시) |
| R69 | 선행 수업(§4.2.3) 구현 시 미래 일자 결석이 출결 생성과 충돌 가능 | 낮음 | 출결 생성 시 기존 레코드 존재하면 skip (중복 방지 이미 구현) |

---

## 참고 사항
- **Phase 3 완료 조건**: Sprint 10 모든 DoD 충족 시 Phase 3(보강 + 소멸) 완료. ROADMAP에서 Phase 3 상태를 ✅ 완료로 전환
- **캘린더 뷰 범위**: PRD §4.6의 일/주/월 뷰 + 원생 팝업 + 보강관리 뷰를 모두 포함. 학사 캘린더(§4.4)는 이미 Sprint 7에서 구현 완료 — 연동만 필요
- **보강데이 일괄 등록**: Sprint 9에서 BatchMakeupDialog 폐기(J7). 캘린더 보강관리 뷰에서 진입점만 제공하되, 구체적 UI는 PI-04로 사용자 확인 후 결정
- **마이그레이션 정책**: V108부터 (V107은 Sprint 8). T1 CHECK 제약 정리 or T3 소멸 관련 컬럼 추가 시 사용
