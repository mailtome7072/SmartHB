# Sprint 19 코드 리뷰

> 대상: Sprint 19 (develop...sprint19, 69개 파일) — UX 개선 + 인쇄 아키텍처 재설계 + 수강생대장 출력
> 리뷰 일자: 2026-07-08
> 자동 검증 결과: cargo test 431 passed / clippy clean / pnpm lint clean / pnpm tsc clean / pnpm build 성공

## 발견 사항 (4건)

### F1 — GradePromotionDialog.tsx promoteGrades() 에러 미처리 (High, 미수정)

- 위치: `src/components/layout/GradePromotionDialog.tsx:74-79`
- 내용: `onClick` 핸들러에 `try/finally`만 있고 `catch` 블록이 없음. `promoteGrades()` IPC 호출이 예외(DB 쓰기 실패, 타임아웃 등)를 던지면 `setPrompt(null)`이 실행되지 않아 다이얼로그는 열린 채 유지되지만, 사용자에게 어떤 에러 피드백도 표시되지 않음.
- 실패 시나리오: DB 잠금 상태에서 "진행" 클릭 → IPC throw → finally로 `setPromoting(false)` 복원 → 버튼만 "진행"으로 돌아오고 학년 승급이 실패했음을 사용자가 알 수 없음
- 조치: 다음 스프린트 T0 수정 권고 — `catch (err)` 블록에서 toast 또는 에러 상태 노출

### F2 — useTableSort.ts desc 정렬 시 tiebreak 역전 (Medium, 미수정)

- 위치: `src/hooks/useTableSort.ts:54`
- 내용: `copy.reverse()`가 배열 전체를 뒤집으므로, tiebreak(이름 가나다순)도 함께 역순이 됨. 예: 출결 횟수 내림차순 정렬 시 동일 출석 횟수 학생들이 이름 역순(Z→A)으로 표시됨. 계획 문서 T1 "동일 값 정렬 시 이름 자동 가나다순"(사용자 요청 2번) 요구사항과 불일치.
- 실패 시나리오: 출결 그리드에서 "출석" 컬럼 ▼(내림차순) 클릭 → 동일 출석 횟수 학생이 가나다 역순으로 표시됨
- 조치: comparator에 방향을 적용하되 tiebreak는 항상 오름차순 유지하는 방식으로 수정. `const compare = (a, b) => dir * primaryCompare(a,b) || tiebreak(a,b)`

### F3 — students.rs EnrollDateAsc/Desc tiebreak가 이름 대신 id 사용 (Low, 미수정)

- 위치: `src-tauri/src/commands/students.rs:131-132`
- 내용: `EnrollDateAsc → "ORDER BY enroll_date ASC, id ASC"`, `EnrollDateDesc → "ORDER BY enroll_date DESC, id DESC"`. 다른 정렬 기준(GenderAsc, WeeklyHoursAsc 등)이 모두 `name ASC`로 tiebreak하는 것과 달리 `id ASC/DESC`를 사용. 단위 테스트 `gender_and_weekly_hours_sort_have_name_tiebreak`에서 EnrollDate를 검사하지 않아 미검출.
- 실패 시나리오: 같은 날 등록한 원생 2명이 id 순서(등록 순서)로 표시됨 — 사용자 요청 2번 "동일 값 내 이름 가나다순" 일관성 위반
- 조치: `ORDER BY enroll_date ASC, name ASC` / `ORDER BY enroll_date DESC, name ASC`로 수정

### F4 — attendance.rs AttendanceGridStudent.school_level raw String 미검증 (Low, 미수정)

- 위치: `src-tauri/src/commands/attendance.rs:607`
- 내용: `get_grid_impl`에서 `school_level`을 raw `String`으로 fetch. `students.rs`는 `SchoolLevel::from_db_code()` 검증 계층을 사용하지만, 이 경로는 없음. DB에 예상 외 값이 있으면 프론트엔드 `SCHOOL_LEVEL_ORDER[a.schoolLevel]`이 `undefined`를 반환해 정렬 결과가 비결정적이 될 수 있음.
- 실패 시나리오: V310 마이그레이션 전 DB에 `초등학교` 같은 한글 원시값이 남아있는 경우 출결 그리드 정렬이 학년순이 아닌 임의 순서로 표시됨 — V310 마이그레이션 적용 후에는 실제 위험 낮음
- 조치: 프론트엔드 `SCHOOL_LEVEL_ORDER`에 fallback 값 추가 또는 백엔드에서 unknown → "unknown"으로 정규화

## 영역별 추가 점검

### 보안 (backend.md Critical)
- SQL 인젝션: 없음 — 모든 쿼리 `bind()` 파라미터 일관 사용
- 하드코딩 시크릿/암호화 키: 없음
- Tauri 권한 과다 허용: 없음 — `capabilities/default.json` 인쇄 창 생성 권한 최소 범위로 추가됨

### 보안 (backend.md High)
- `unwrap()` 프로덕션 코드: 없음 (테스트에서만 사용)
- 마이그레이션 없는 스키마 변경: 없음 — V310 마이그레이션 동반
- 새 쿼리 단위 테스트: 출결 그리드 school_level 필드 추가 테스트 존재 (`grid_orders_students_by_school_level_grade_then_name`)

### 보안 (frontend.md Critical)
- XSS (`dangerouslySetInnerHTML`): 없음
- `invoke()` 직접 호출: 없음 — 모든 IPC는 `src/lib/tauri/index.ts` 래퍼 경유
- 인쇄 HTML 생성 (`academic-print-html.ts`, `student-roster-print-html.ts`): `escapeHtml()` 함수로 사용자 데이터 escape 처리 — XSS 안전

### AI 생성 코드 추가 체크
- on_window_event "main" 창 가드: `window.label() == "main"` 조건 정상 — academic-print/roster-print 창 닫기 시 exit_hook 미실행 확인
- `GradePromotionDialog.tsx` 모듈 레벨 플래그 `gradePromotionAttempted`: 세션당 1회 체크 정상 작동 (StrictMode 이중 마운트에서도 AtomicBool 없이 모듈 클로저로 관리 — 허용 범위)
- `promote_grades` 연도 중복 방지: `last_grade_promotion_year` 키로 정상 보호 — 단위 테스트 5건 커버

## 결론

Critical 없음. High 1건(F1, 에러 처리 누락)과 Medium 1건(F2, tiebreak 역전)은 다음 스프린트 T0 수정 권고. Low 2건은 risk-register 등록 후 모니터링. 전반적 코드 품질 양호 — Sprint 19 규모(69파일, 11개 Task) 대비 findings 수 적음.
