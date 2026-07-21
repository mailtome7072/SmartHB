# Sprint 22 코드 리뷰

> 대상: Sprint 22 (3ad9b1f~93ec011) — 보강 분 단위 부분 차감 전환 + 출결 그리드 z-index 수정 (ADR-011)
> 리뷰 일자: 2026-07-21
> 자동 검증 결과: cargo test 457 passed / clippy clean / tsc clean / lint clean / build 성공

## 발견 사항 (3건: Critical 0 / High 0 / Medium 1 / Low 2)

### F1 — cancel_makeup_impl 루프 내 N쿼리 패턴 (Medium, 수용)

- 위치: `src-tauri/src/commands/makeup.rs:635-661` (`cancel_makeup_impl`)
- 내용: 취소된 보강에 배분된 결석 수(N)만큼 루프에서 잔여 재계산 쿼리를 개별 실행. 결석당 1 SELECT + 조건부 1 UPDATE = 최대 2N 쿼리.
- 영향: 50명 1인 교습소 규모라 실용적 성능 영향 미미. 단일 보강에 매칭되는 결석은 통상 1~3건.
- 조치: 현재 규모에서 수용. 향후 원생 100명+ 확장 시 배치 쿼리 전환 고려 (Medium 리스크로 등록).

### F2 — cancel_makeup docstring이 구 로직 설명 유지 (Low, 무시)

- 위치: `src-tauri/src/commands/makeup.rs:575-581` (IPC 함수 주석)
- 내용: `cancel_makeup` 함수 docstring이 구 V107 FK 로직("makeup_attendance_id=NULL, status='absent'")을 설명하고 있음. 실제 구현은 ADR-011 allocation 기반으로 올바르게 교체됨. 주석만 stale 상태.
- 조치: 기능 영향 없음. 다음 해당 파일 수정 시 주석 갱신.

### F3 — MakeupRegisterDialog 보강 시간 입력 UI (Low, 수용)

- 위치: `src/components/attendance/MakeupRegisterDialog.tsx:230-243`
- 내용: Sprint22.md T7 계획은 "1h/2h/3h 드롭다운 또는 라디오 버튼"이었으나 `<input type="number" step={1} min={0} max={10}>`으로 구현됨. step=1로 정수만 입력 권장되나 소수 직접 입력도 허용.
- 영향: 0.5h 등 소수 입력 시 `hoursToMinutes` 변환이 실수(예: 30분)로 처리되어 기능 이상은 없으나 계획된 UX(1시간 단위 선택)와 차이. 백엔드가 `classMinutes > totalRemaining`을 차단하므로 데이터 오염 없음.
- 조치: 기능상 문제 없음. 향후 스프린트에서 드롭다운 전환 고려.

## 중점 리뷰 항목 검토 결과

### R139 회귀 위험 — T4 전수 전환 검증

체크리스트 8개 항목 전수 확인:
1. `calendar.rs` — `remaining_minutes_expr("ra")` HAVING 기준 ✅ (line 204-220)
2. `attendance.rs` 월간 요약 — `remaining_minutes_expr("ra")` 잔여분 합계 ✅ (line 1020, 1068)
3. `expiration.rs` 소멸 대상 조회 — `status='absent'` 조건 + `makeup_attendance_id` 제거 ✅ (line 99-112)
4. `expiration.rs` 소멸 전이 — `status='absent'` 전이 ✅
5. `expiration.rs` 퇴교 보강 처리 — `remaining_minutes_expr` 잔여분 > 0 기준 ✅ (line 200-205)
6. `diagnosis.rs` 고아보강 — `makeup_allocations` 기반 ✅ (line 284-313)
7. `students.rs:498` 재원 로직 — T4 체크리스트에서 "present/status 전이라 변경 불필요" 판정. 보강 잔여분 집계가 아닌 재원 상태 전이(enroll→withdraw) 판단 로직이라 ADR-011 영향 없음 ✅
8. `attendance.rs:857` 토글 가드 — `makeup_allocations` 잔여분 재계산 사용 ✅ (line 886)

판정: 회귀 위험(R139) 해소 확인. 8개 위치 전수 전환 완료.

### R140 백필 정확성 — V312 윈도우 함수

V312 배분 규칙이 등록 로직(`create_makeup_with_absences_impl`)과 동일한지:
- 정렬 기준: `(makeup_deadline IS NULL), makeup_deadline, event_date, id` — 등록 로직의 `deadline ASC (NULL 마지막), event_date ASC`와 동일 ✅
- 배분량: `min(absence_minutes, max(0, makeup_minutes - prev_sum))` — 등록 로직의 `remaining_makeup.min(info.remaining)`과 동일 논리 ✅
- 멱등성: NOT EXISTS로 중복 INSERT 차단 + UPDATE는 `makeup_attendance_id IS NOT NULL` 조건으로 신규 등록 데이터 보호 ✅
- `makeup_expired` 제외: `status='makeup_done'` 조건으로 자동 제외 (expired는 absent도 makeup_done도 아님) ✅
- 초과 보강분: 배분할 결석 없으면 alloc=0이라 INSERT 제외 → 버림 ✅

### R142 배포 안전성 — 테이블 재구성 회피

- V311: 순수 `CREATE TABLE + CREATE INDEX` — 기존 테이블 미수정 ✅
- `regular_attendances.makeup_attendance_id` DROP 없이 레거시 유지 ✅
- 신규 등록 로직에서 `makeup_attendance_id` 미설정(레거시 컬럼 사용 안 함) ✅
- deferred FK 카운터 함정(V108 code 787) 재발 없음 ✅

### SQL 인젝션 — remaining_minutes_expr

`format!` 사용이지만 `alias` 파라미터는 8개 호출 위치 모두 코드 상수 리터럴(`"ra"`, `"regular_attendances"`)만 전달. 함수 docstring에 "사용자 입력을 넘기면 안 된다" 명시. SQL 인젝션 위험 없음 ✅

## 영역별 추가 점검 (backend.md / frontend.md rules)

### 보안 (backend.md Critical)
- SQL 인젝션: bind() 파라미터 사용 전수 확인 ✅
- 하드코딩 시크릿: 없음 (git diff grep 확인) ✅
- Tauri 권한 과다: 신규 capability 없음 ✅

### 보안 (backend.md High)
- `unwrap()` 프로덕션 코드: 없음 (테스트 코드만 사용) ✅
- 마이그레이션 없는 스키마 변경: V311/V312 정상 작성 ✅
- 새 쿼리 단위 테스트: makeup.rs 36건 + diagnosis.rs 31건 신규 커버 ✅
- PRD §6.2 UNIQUE 제약: makeup_allocations UNIQUE(makeup_id, absence_id) ✅

### 프론트엔드 (frontend.md Critical/High)
- XSS: `dangerouslySetInnerHTML` 없음 ✅
- invoke() 직접 호출: `@/lib/tauri` 래퍼 경유 ✅
- TypeScript any: 없음 ✅
- SSR 가드: Tauri IPC 컴포넌트 `'use client'` 적용 ✅

### AI 생성 코드 추가 체크
- 비즈니스 로직 ↔ sprint22.md 요구사항 일치 ✅
- 의도치 않은 파일 변경 없음 (변경 파일 목록 확인) ✅
- 하드코딩 테스트 데이터 프로덕션 코드 없음 ✅
- 신규 의존성 없음 ✅

## 결론

Critical·High 이슈 없음. Medium 1건(cancel_makeup N쿼리)은 현재 규모에서 수용 가능 수준으로 배포 진행에 문제 없음. V312 백필 정확성, R139 전수 전환, R142 배포 안전성 모두 검증 완료.
