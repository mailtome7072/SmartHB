# Sprint 14 코드 리뷰

> 대상: Sprint 14 (develop...sprint14) — 대시보드 위젯 + 자가 진단 + 엑셀 내보내기 + 복원 리허설 + 원생 생년월일
> 리뷰 일자: 2026-06-06
> 자동 검증 결과: cargo test 369 passed / clippy clean / cargo check --features cipher 통과 / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 발견 사항 (3건)

### F1 — monthly_summary 청구 집계 시 결제 미존재 원생 청구 누락 가능 (Medium, 기록)

- 위치: `src-tauri/src/commands/dashboard.rs:306~324` `monthly_summary()`
- 내용: `bills LEFT JOIN payments` 집계에서 `payments.is_paid=1` 케이스만 `paid_total`에 합산하는 로직은 올바르나, `bill_count`가 `COUNT(*)` (LEFT JOIN 포함)이므로 미수납 청구도 정상 카운트된다. 현재 설계 의도와 일치하므로 오류가 아니나, 장래 여러 결제 건이 한 청구에 결합되는 요구사항 변경 시 `GROUP BY b.id` 없이 집계하면 중복 합산 위험이 있다.
- 실패 시나리오: 현재 스키마(청구:수납 = 1:1)에서는 문제 없음. 향후 부분 수납 지원 시 `paid_total` 이중 합산 가능.
- 조치: ROADMAP 이연. Sprint 15 청구 마감 워크플로우 확장 시 GROUP BY 검토 포함.

### F2 — 대시보드 위젯 title prop에 inline `fontSize` style 4건 (Low, 기록)

- 위치: `src/components/dashboard/DashboardView.tsx:131, 155, 219` 및 메모 포스트잇 높이
- 내용: `당일 수업`, `이달의 생일`, `월 요약` 위젯 타이틀에 `style={{ fontSize: '24px' }}` inline style 사용. frontend.md 규칙("인라인 style prop 지양")에 해당하나, scope.md에 "정밀 사이징 목적의 의도적 선택"으로 명시되어 있음. 포스트잇 높이(사용자 드래그 리사이즈 값 — 동적 수치)는 inline style이 불가피하므로 정당함.
- 실패 시나리오: 기능 영향 없음. 단, 향후 다크모드/테마 확장 시 Tailwind 클래스 기반과 혼용돼 유지보수 복잡도 증가 가능.
- 조치: 기록만. `text-2xl`(24px) 상수 클래스로 통일은 Sprint 15 UI 정비 시 검토.

### F3 — run_diagnosis run_type 파라미터 유효성 검사 위치 (Low, 기록)

- 위치: `src-tauri/src/commands/diagnosis.rs:568`
- 내용: `run_type`이 "auto"/"manual"이 아닌 경우 `UserFacing` 에러를 반환하는 검사가 IPC 진입부에 있어 올바르다. 다만 내부 `run_and_record`가 `run_type`을 DB에 그대로 바인드하므로, 검사를 우회하면 CHECK 제약(`run_type IN ('auto', 'manual')`)이 2차 방어선이 된다. 이중 방어 구조는 의도적이고 올바름.
- 조치: 기록만. 현재 구조 유지.

---

## 영역별 추가 점검

### 보안 (backend.md Critical 체크)

| 항목 | 결과 |
|------|------|
| SQL 인젝션 — raw query concat | 이상 없음. `REHEARSAL_TABLES` allowlist 상수 보간(사용자 입력 아님) 명시 주석 확인. 모든 파라미터는 `bind()` 처리. |
| 하드코딩 시크릿/암호화 키 | 이상 없음. 시크릿 패턴 스캔 결과 0건. |
| Tauri 권한 과다 허용 | 미변경 (`capabilities/default.json` 수정 없음). |
| SQLCipher 키 Keychain 외부 저장 | 이상 없음. cipher feature 게이트 정상 동작. |

### 보안 (backend.md High 체크)

| 항목 | 결과 |
|------|------|
| `unwrap()` 남용 | 이상 없음. 프로덕션 코드에서 `unwrap()`/`expect()` 없음. 테스트 코드 내 `expect()` 사용만 존재 (의도적). |
| 마이그레이션 없는 스키마 변경 | 이상 없음. V303~V305 마이그레이션 정상 생성. `students.birth_date` nullable ALTER TABLE은 V305로 보관. |
| PRD §6.2 UNIQUE 제약 누락 | 이상 없음. V303 `diagnosis_history`는 UNIQUE 제약 불필요(이력성 테이블). `birth_date` 추가는 기존 UNIQUE(`serial_no`) 영향 없음. |
| `PRAGMA integrity_check` 누락 | 이상 없음. 복원 리허설 흐름에서 `integrity_check` 실행 확인. |

### 보안 (frontend.md Critical 체크)

| 항목 | 결과 |
|------|------|
| XSS (`dangerouslySetInnerHTML`) | 이상 없음. 신규 컴포넌트 전체 스캔 결과 0건. |
| `invoke()` 직접 호출 | 이상 없음. 모든 IPC 호출이 `src/lib/tauri/index.ts` 래퍼 경유. |
| 민감정보 localStorage/sessionStorage | 이상 없음. 신규 코드에서 사용 없음. |

### 프론트엔드 (frontend.md High 체크)

| 항목 | 결과 |
|------|------|
| TypeScript `any` 남용 | 이상 없음. 신규 타입 파일(`diagnosis.ts`, `dashboard.ts`, `export.ts`) 전수 any 미사용. |
| SSR 가드 누락 | 이상 없음. Recharts는 `next/dynamic(ssr:false)`로 적용. 브라우저 API 직접 접근 없음. |
| 글로벌 검색바 누락 | 이상 없음. 신규 라우트(`/settings/diagnosis`, `/settings/data`, `/settings/backup`) 모두 AppShell 내부에서 렌더링되어 검색바 자동 포함. |
| Pretendard/18pt/44×44px 접근성 | 신규 버튼(내보내기, 진단 실행, 리허설)이 `h-10 px-4`(Tailwind) 기반으로 44×44px 충족 확인. 폰트 크기는 글로벌 globals.css 기반 유지. |

### AI 생성 코드 추가 체크

| 항목 | 결과 |
|------|------|
| 검사 로직 오탐 — 보강필요시간 음수 | 세션 #2 사용자 검증에서 오탐(성춘향 케이스) 발견 → 원인 분석 후 `absent`+`makeup_done` 합산으로 수정. 회귀 테스트 2건 추가. 이력 있음. |
| 공휴일 오탐 — 출결 진행률 | 세션 #2 사용자 검증에서 발견 → 위젯·알림 전체 제거로 해결(근본 원인: 출결 모델과 "미입력" 개념 불일치). |
| 자가 진단 이력 중복 누적 | 세션 #4 사용자 검증에서 발견 → `is_same_as_latest` 가드 추가. |
| 퇴교생 보강 소멸 알림 오탐 | 세션 #3 사용자 검증에서 발견 → 알림 쿼리 퇴교 제외 + V304 백필 마이그레이션. |

---

## 결론

Critical/High 이슈 없음. Medium 1건(F1 — 청구 집계 장래 확장성)은 현재 스키마에서 문제 없음. Low 2건(F2 inline style, F3 유효성 검사 이중 방어)은 기록만. 코드 품질 양호. 사용자 실DB 검증이 4~5회 수행되어 오탐·중복·퇴교생 예외처리 등 AI 생성 코드의 비즈니스 로직 오류가 조기에 발견·수정됐다. 이는 스프린트의 신뢰성을 높이는 긍정적 패턴이다.
