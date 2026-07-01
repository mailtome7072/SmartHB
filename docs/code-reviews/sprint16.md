# Sprint 16 코드 리뷰

> 대상: Sprint 16 (develop~20...develop, 163파일 +16,033/-1,190) — v1.0.0 정식 출시
> 리뷰 일자: 2026-06-12
> 자동 검증 결과: cargo test 417 passed (cipher off) / clippy --all-targets clean / cargo check --features cipher OK / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 발견 사항 (8건 — Confirmed 2건 즉시 수정, 이연 6건)

### F1 — change_data_folder_impl pool 미교체 (High, 수정 완료)

- 위치: `src-tauri/src/commands/paths.rs` (change_data_folder_impl)
- 실패 시나리오: config.json만 새 경로로 갱신하고 OnceCell global pool은 그대로 유지 → 재시작 전 출결/청구 IPC 호출 시 구 경로 DB에 쓰기 → 재시작 후 신 경로 DB에서 해당 변경 누락
- 조치: `write_status` 완료 후 `db::pool().close()` 추가 (커밋 9b6034f). 후속 IPC가 pool 종료 에러를 반환하여 구 DB 오기입을 차단

### F2 — import_students_csv 부분 삽입 (High, 수정 완료)

- 위치: `src-tauri/src/commands/import.rs` (import_students_csv)
- 실패 시나리오: 단일 트랜잭션 없이 행마다 create_student 개별 호출 → 중간 DB 오류 시 1~N행 커밋, N+1~끝 누락 → 재시도 시 기삽입 행은 중복 skip → 데이터 불완전
- 조치: 단일 트랜잭션으로 묶음. `insert_student_tx` 헬퍼 추출(students.rs) + 원자성 단위 테스트 2건 추가 (커밋 9b6034f)

### F3 — MoveAttendanceDialog submitting 잔존 (Medium, v1.0 후 이연)

- 위치: `src/components/attendance/MoveAttendanceDialog.tsx:~89`
- 실패 시나리오: `handleSelect` 성공 분기에 `finally` 없고 catch에만 `setSubmitting(false)` → `void invalidateQueries` throw 시 submitting=true 잔존 → 달력 전체 비활성 영구화
- 조치: R117 등록. A105 — finally 블록으로 이동 (차기 안정화 스프린트)

### F4 — notices/page.tsx 구형 guard 패턴 (Medium, v1.0 후 이연)

- 위치: `src/app/notices/page.tsx:~986`
- 실패 시나리오: `setUnsavedNavTarget` 미호출로 시스템 공통 UnsavedNavDialog와 아키텍처 불일치. 현재 자체 pendingAction 모달로 기능적으로는 커버되나, UnsavedNavDialog 동작 변경 시 notices만 따라가지 않는 위험
- 조치: R118 등록. A104 — P2-4 notices 분리 스프린트 시 useUnsavedChanges 훅으로 마이그레이션

### F5 — move_attendance_impl TOCTOU 구조 (Low, v1.0 후 이연)

- 위치: `src-tauri/src/commands/attendance.rs:~1199`
- 실패 시나리오: status 체크와 UPDATE 사이 트랜잭션 미보호. 단독 운영 앱이라 실현 가능성 낮으나, MAX_CONNECTIONS=1 직렬화로 현재 운용 환경에서는 재현 불가
- 조치: R119 등록. 차기 attendance 도메인 리팩토링 시 명시적 트랜잭션 추가

### F6 — apply_schedule_change_impl 선행 조건 미검증 (Low, v1.0 후 이연)

- 위치: `src-tauri/src/commands/attendance.rs:~1282`
- 실패 시나리오: set_schedule 완료 여부를 DB 레벨 검증 없이 주석 계약에만 의존. 현재 schedule-editor.tsx가 항상 await set_schedule 선행 보장하므로 정상 경로에서 재현 불가. 미래 호출경로 추가 시 위험
- 조치: R120 등록. effective_from 존재 여부 DB 검증 추가 검토

### F7 — compute_summary / fetch_pending_absences SQL 술어 중복 (Low, v1.0 후 이연)

- 위치: `src-tauri/src/commands/attendance.rs:~971`
- 실패 시나리오: 동일 WHERE 조건이 2개 쿼리에 중복. 조건 변경 시 한 쪽 누락 시 집계-목록 불일치 + get_grid_impl 내 학생당 2쿼리(N+1) 성능 문제
- 조치: R121 등록. P2-1 출결 그리드 N+1 최적화 스프린트에서 통합 쿼리로 교체

### F8 — weekday_ko() 이중 정의 (Low, v1.0 후 이연)

- 위치: `src-tauri/src/commands/attendance.rs:~1072` (notice.rs에도 동일 구현)
- 실패 시나리오: 요일 표기 변경 시 한 쪽 누락 → 수업일 이동 메모와 공지문 요일 표기 불일치
- 조치: R122 등록. P2-10 백엔드 위생 스프린트에서 util_date.rs 공유 모듈로 추출

---

## 영역별 추가 점검

- 보안 (backend.md Critical) — SQL 인젝션 0건, 하드코딩 시크릿 0건. bind() 파라미터 일관 사용 확인
- 보안 (backend.md High) — unwrap()/expect() 프로덕션 코드 사용 없음. 마이그레이션(V306·V307) 적용 확인
- 프론트엔드 (frontend.md Critical/High) — dangerouslySetInnerHTML 사용 0건. invoke() 직접 호출 0건. localStorage 민감정보 저장 없음
- AI 생성 코드 추가 체크 — 전체 코드리뷰(full-review-2026-06.md)에서 치명 버그 없음, 트랜잭션 설계 13곳 정확, 비즈니스 규칙 테스트 충실 평가

---

## 결론

Critical 0건, High 2건(즉시 수정 완료), Medium 2건·Low 4건(risk-register 이연). 배포 차단 이슈 없음. v1.0.0 배포 진행 가능.
