# Sprint 18 코드 리뷰

> 대상: Sprint 18 (5cd6a8a → d86c25b) — 사용자 피드백 10건 + 캘린더 UX 개선 + 출결 동기화
> 리뷰 일자: 2026-07-01
> 자동 검증 결과: cargo test 414 passed / clippy --all-targets clean / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 발견 사항 (5건, 수정 완료)

### F1 — LockWarning.tsx STALE_THRESHOLD_SECONDS 불일치 (Medium, ✅ 수정 완료 d86c25b)

- 위치: `src/components/LockWarning.tsx:17`
- 내용: 프론트엔드 `STALE_THRESHOLD_SECONDS = 300`이 Sprint 18 T0에서 백엔드 lock.rs를 86400으로 변경한 것과 불일치.
- 실패 시나리오: 비정상 종료 후 5분~24시간 사이에 다른 PC에서 앱 시작 시 백엔드는 stale=true 반환하지만 UI 버튼 disabled 유지로 강제 점유 불가.
- 조치: `STALE_THRESHOLD_SECONDS = 86400`으로 수정, 주석 및 UI 텍스트 "5분" → "24시간" 동기화.

### F2 — academic.rs T8 sync fail-soft 정책 위반 (Medium, ✅ 수정 완료 d86c25b)

- 위치: `src-tauri/src/commands/academic.rs:1013, 1085, 1151`
- 내용: `sync_attendance_on_schedule_change(...).await?`로 에러 전파. sprint18.md T8 설계의 "fail-soft: 동기화 실패 시 eprintln! 로그만 남기고 IPC 흐름 차단하지 않음" 위반.
- 실패 시나리오: 일정 생성/수정/삭제 DB 커밋 성공 후 sync 단계에서 오류 발생 시 사용자에게 에러 노출, 재시도 시 중복 이벤트 생성 위험.
- 조치: 3곳 모두 `if let Err(e) = ... { eprintln!(...) }` fail-soft 패턴으로 변경.

### F3 — attendance.rs OFF 분기 DELETE 조건 누락 (Medium, ✅ 수정 완료 d86c25b)

- 위치: `src-tauri/src/commands/attendance.rs:1460`
- 내용: `DELETE FROM regular_attendances WHERE event_date = ?`가 결석(absent) 기록과 보강 매칭(makeup_attendance_id IS NOT NULL) 행까지 삭제.
- 실패 시나리오: 결석 처리된 날에 공휴일 이벤트 추가 시 결석 기록, makeup_deadline, makeup_attendance_id FK 참조 소실로 보강 매칭 관계 파괴.
- 조치: `AND status = 'present' AND makeup_attendance_id IS NULL` 조건 추가.

### F4 — academic/page.tsx 인쇄 race condition (Low, ✅ 수정 완료 d86c25b)

- 위치: `src/app/academic/page.tsx:284`
- 내용: `setTimeout(() => window.print(), 300)` 고정 딜레이로 printEventsQuery 완료를 보장하지 않음.
- 실패 시나리오: IPC 응답 지연 시 달력 데이터 로드 전 인쇄 다이얼로그 실행 → 빈 인쇄물 출력.
- 조치: useEffect로 `printEventsQuery.isSuccess` 감지 후 `window.print()` 호출로 변경.

### F5 — ClassCalendar.tsx 월보기 색상 상수 중복 (Low, ✅ 수정 완료 d86c25b)

- 위치: `src/components/schedules/ClassCalendar.tsx:285`
- 내용: useEffect 내부 `COLORS` 객체가 파일 상단 `DURATION_COLORS`의 border 값을 하드코딩 중복.
- 실패 시나리오: 색상 변경 시 두 곳 중 한 곳만 수정하면 주보기/월보기 색상 불일치.
- 조치: `DURATION_COLORS[min]?.border` 참조로 단일화.

---

## 영역별 추가 점검

### 보안 (backend.md Critical)
- SQL 인젝션: 모든 쿼리 bind() 파라미터 사용 확인 — 이상 없음
- 하드코딩된 시크릿: 스캔 결과 없음 — 이상 없음
- 인증/인가 누락: 신규 IPC(sync_attendance_on_schedule_change)는 내부 함수로 pub(crate) 노출 없음 — 이상 없음

### 보안 (backend.md High)
- `unwrap()` 사용: 신규 코드에서 사용 없음 — 이상 없음
- 마이그레이션 누락 스키마 변경: V308, V309 정상 적용. T8은 스키마 변경 없음 — 이상 없음
- 새 쿼리 단위 테스트: sync_single_date 단위 테스트 2건 추가 — 충족

### 프론트엔드 (frontend.md Critical)
- XSS (dangerouslySetInnerHTML): 신규 코드에서 사용 없음 — 이상 없음
- invoke() 직접 호출: AcademicSchedulePrint, PaymentsView, ClassCalendar 모두 src/lib/tauri 래퍼 사용 — 이상 없음

### 프론트엔드 (frontend.md High)
- TypeScript any 사용: 신규 파일에서 사용 없음 — 이상 없음
- SSR 가드 누락: `window.print()` 호출이 academic/page.tsx 클라이언트 컴포넌트('use client') 내에서만 실행 — 이상 없음

### AI 생성 코드 추가 체크
- 프론트엔드 useEffect 의존성: 수정된 printMode useEffect `[printMode, printEventsQuery.isSuccess]` 의존성 배열 정확
- Rust 에러 처리 일관성: fail-soft 수정 후 3곳 모두 동일 패턴 적용 확인

---

## 결론

Critical 0 / High 0 / Medium 3 / Low 2. 수정 5건 모두 당일 완료. 재검증(cargo test 414, clippy clean, pnpm lint/tsc/build 통과) 확인.
