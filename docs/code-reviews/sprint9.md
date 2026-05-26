# Sprint 9 코드 리뷰

> 대상: Sprint 9 (fb710b3 → 6494f2b 범위, 22 커밋) — Phase 3 첫 마일스톤 (보강 도메인 전체)
> 리뷰 일자: 2026-05-26
> 자동 검증: cargo test cipher off 253 passed / cipher on 133 passed / clippy clean / lint clean / tsc clean / build 성공

---

## 발견 사항 (3건)

### F1 — dead code 잔존 (Medium, Sprint 10 carry-over)

- 위치: `src-tauri/src/commands/makeup.rs` — `mark_makeup_absent`, `batch_create_makeups` IPC 및 `audit::MakeupAbsent` variant
- 사유: 사용자 결정으로 J5(보강 미등원 UI 삭제)/J7(보강데이 일괄 폐기) 후 프론트엔드 호출 경로가 제거되었으나 백엔드 코드는 유지
- 실패 시나리오: 향후 `cargo clippy -- -D warnings` + `dead_code` lint 옵션 적용 시 경고 발생 가능. 현재는 IPC 등록으로 참조가 존재하여 `dead_code` 경고 미발생
- 조치: Sprint 10 carry-over (R63) — IPC handler 등록 제거 + 관련 코드 정리

### F2 — `get_absence_history` 페이지네이션 미적용 (Low, 이연)

- 위치: `src-tauri/src/commands/makeup.rs:742-780`
- 사유: `WHERE student_id = ?` + `ORDER BY event_date DESC` 전수 조회. `pagination.rs` 헬퍼 미적용
- 실패 시나리오: 장기 수강 원생(3년+, 결석 100건 이상)에서 응답 지연 가능성. 현재 단독 교습소 규모(50명 이하, 출결 12개월 기준)에서는 허용 범위
- 조치: Sprint 10~11 carry-over (R64)

### F3 — 결석 단건 루프 검증 (Low, 이연)

- 위치: `src-tauri/src/commands/makeup.rs:434-467` (`create_makeup_with_absences_impl` 검증 3)
- 사유: `for &aid in &payload.absence_ids` 루프에서 단건 SELECT 반복. `absence_ids` 20건+ 시 N회 쿼리
- 실패 시나리오: 한 원생이 20건 이상 결석을 한 번에 매칭하는 경우 미미한 성능 저하. 현재 실사용 패턴(결석 1~5건 선택)에서는 문제없음
- 조치: Sprint 10 carry-over (R65) — `JSON_EACH` IN 절 패턴으로 교체 가능

---

## 영역별 추가 점검

### 보안 (backend.md Critical 체크)

- SQL 인젝션: `sqlx::query!` 매크로 및 `.bind()` 패턴 일관 사용. raw query concat 없음 — **이상 없음**
- 하드코딩 시크릿: 암호화 키, 비밀번호, API 키 하드코딩 없음 — **이상 없음**
- Tauri 권한: `src-tauri/capabilities/default.json` 신규 권한 추가 없음 — **이상 없음**
- SQLCipher 키 노출: 키 관련 코드 변경 없음 (보강 도메인은 키 접근 불필요) — **이상 없음**

### 보안 (backend.md High 체크)

- `unwrap()` 남용: 테스트 fixture에서만 사용. 프로덕션 코드는 `?` 연산자 + `map_err` 일관 사용 — **이상 없음**
- 마이그레이션 없는 스키마 변경: 신규 마이그레이션 없음 (V106/V107 활용) — **이상 없음**
- `.sqlx/` 캐시: Sprint 9에서 신규 `query!` 매크로 추가 없음 (`sqlx::query()` + `Row::try_get` 패턴) — 캐시 갱신 불필요
- PRD §6.2 UNIQUE 제약: 보강 도메인은 UNIQUE 제약 없음 (1 결석 → 1 보강 매칭은 FK + status 로 관리) — **이상 없음**

### 프론트엔드 (frontend.md Critical 체크)

- XSS: `dangerouslySetInnerHTML` 사용 없음. 사용자 입력 직접 렌더링 없음 — **이상 없음**
- `invoke()` 직접 호출: 컴포넌트에서 `@tauri-apps/api/core` 직접 import 없음. 전부 `@/lib/tauri` 추상화 경유 — **이상 없음**
- 민감 정보 노출: localStorage 저장 없음 — **이상 없음**

### 프론트엔드 (frontend.md High 체크)

- TypeScript any 남용: `src/types/makeup.ts` strict 타입 정의. `any` 사용 0건 — **이상 없음**
- SSR 가드: `MakeupRegisterDialog`, `MakeupManageDialog`, `AbsenceHistoryDialog` 모두 `'use client'` 선언. `typeof window` 가드 별도 필요 없음 — **이상 없음**
- Pretendard/18pt/44px: 다이얼로그 버튼 `min-h-[44px]` 일관 적용 확인 — **이상 없음**
- 글로벌 검색바: 기존 layout.tsx 유지, 신규 라우트 없음 — **이상 없음**

### AI 생성 코드 추가 체크

- 트랜잭션 원자성: `BEGIN` + `COMMIT` 패턴 명시. `cancel_makeup_impl`은 결석 환원 후 DELETE 순서(FK 위반 회피) 준수 — **이상 없음**
- 멱등성: `mark_makeup_absent_impl`의 `status == "makeup_absent"` 조기 반환 확인 — **이상 없음**
- race condition 방지: `rows_affected() != 1` 검증으로 트랜잭션 중 상태 변경 감지 — **이상 없음**
- 에러 메시지 한국어: 전수 한국어 사용자 친화 메시지 — **이상 없음**

---

## 결론

Critical 0건 / High 0건 / Medium 1건 (F1, dead code) / Low 2건 (F2, F3).
Medium/Low 3건 모두 Sprint 10 carry-over 처리 예정 (R63~R65). 즉시 차단 이슈 없음.
Sprint 9 develop 머지 및 배포 진행 가능.
