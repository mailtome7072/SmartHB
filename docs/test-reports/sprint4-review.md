# Code Review — Sprint 4 (Phase 1.5 품질 안정화)

> 기준 커밋: `a06dbd6^..b7e9ca6`
> 검토 파일 수: 34개 (Sprint 4 핵심 변경)
> 검토 완료: 2026-05-22

---

## 검토 범위

Sprint 4 핵심 Task 별 변경 파일:

| Task | 파일 | 등급 |
|------|------|------|
| T1 | `src/components/ui/alert-dialog.tsx`, `src/app/students/edit/page.tsx` | 통과 |
| T2 | `src-tauri/src/commands/settings.rs`, `src/app/settings/page.tsx`, `src/app/settings/hours/page.tsx` | 통과 |
| T3 | `src/app/page.tsx`, `src/components/layout/top-bar.tsx` | 통과 |
| T4/T5/T6 | `src/components/students/student-form.tsx`, `src/lib/format.ts` | 통과 |
| T7/T8 | `src/app/students/new/page.tsx`, `src/app/students/edit/page.tsx` | 통과 |
| T9 | `src/components/students/schedule-editor.tsx`, `src-tauri/src/commands/schedules.rs` | 통과 (Low 1건) |
| T10 | `src/app/settings/codes/page.tsx`, `src-tauri/src/commands/codes.rs` | 통과 (Medium 1건) |
| T11 | `src-tauri/src/commands/students.rs`, `src/app/students/page.tsx` | 통과 |
| 공통 | `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs`, `src/lib/tauri/index.ts`, `src/types/student.ts`, `src-tauri/src/commands/audit.rs` | 통과 |

---

## Critical 이슈 (0건)

없음.

---

## High 이슈 (0건)

없음.

---

## Medium 이슈 (1건)

### M1 — DnD 필터링 중 숨겨진 항목의 sort_order 충돌

- **파일**: `src/app/settings/codes/page.tsx` (L198~200)
- **내용**: 활성 필터(`전체/사용/미사용`)가 적용된 상태에서 보이는 행만 DnD 정렬하면, 보이는 행들에 1~N이 재부여되지만 숨겨진 행의 sort_order와 충돌 가능. 주석에서 이 문제를 인지하고 "단순화를 위해 전체 base 0부터 다시 부여"로 설명하고 있으나, 실제 구현은 보이는 행(visibleCodes)의 id만 reorder 호출에 포함하여 숨겨진 행과 sort_order 겹침이 발생한다.
- **영향**: 코드 테이블 단일 사용자 환경, 항목 수 소량(~30개)이므로 사용자 체감 오류는 발생하지 않음. 그러나 필터를 '사용'으로 두고 DnD 후 '전체'로 전환하면 순서가 혼재될 수 있다.
- **대응**: Risk Register에 등록. 필터 상태에 무관하게 전체 codes 배열을 기준으로 재정렬하거나, DnD 활성화 시 필터를 '전체'로 강제 전환하는 방식으로 개선. Sprint 5에서 처리.

---

## Low 이슈 (1건)

### L1 — 미운영 요일 스케줄 추가 폼 개선 여지

- **파일**: `src/components/students/schedule-editor.tsx`
- **내용**: 미운영 요일 선택 시 "미운영 요일" 텍스트와 disabled 버튼으로 안내하나, 운영시간 설정 페이지로의 직접 링크가 없어 50대 사용자가 해결 경로를 파악하기 어려울 수 있다. `title` 속성으로 안내는 제공되나 tooltip은 모바일/터치에서 표시되지 않음(데스크톱 앱이므로 영향 미미).
- **대응**: 기록만. 향후 온보딩 안내 개선 시 함께 처리.

---

## 체크리스트 결과

### 보안

| 항목 | 결과 |
|------|------|
| SQL 인젝션 (raw query concat 금지) | 통과 — `reorder_codes`의 `format!()` 사용이 있으나 `table_name()`이 `&'static str` match 반환으로 안전. 사용자 입력 미포함 |
| 하드코딩 시크릿/암호화 키 | 통과 — 스캔 결과 없음 |
| Tauri 권한 최소화 (capabilities) | 통과 — Sprint 4에서 capabilities 변경 없음 |
| `dangerouslySetInnerHTML` 미사용 | 통과 |
| localStorage에 민감 정보 저장 금지 | 통과 — localStorage는 임시저장(draft) 용도로만 사용 |
| `window.confirm`/`window.alert` 제거 | 통과 — T1에서 shadcn AlertDialog로 완전 교체 |

### 코드 품질

| 항목 | 결과 |
|------|------|
| `unwrap()`/`expect()` 프로덕션 코드 미사용 | 통과 — 추가된 `unwrap()`은 전부 `#[cfg(test)]` 블록 |
| TypeScript `any` 타입 남용 | 통과 — Sprint 4 변경 파일 전수 확인, `any` 사용 없음 |
| Tauri IPC `invoke()` 직접 호출 금지 | 통과 — 모든 IPC 호출이 `src/lib/tauri/index.ts` 래퍼 경유 |
| SSR 가드(`typeof window`) | 통과 — `student-form.tsx`(localStorage), `tauri/index.ts`에 가드 적용 |
| `'use client'` 사용 최소화 | 통과 — IPC 호출 필요 페이지에만 선언 (4개 페이지) |

### 패턴 준수

| 항목 | 결과 |
|------|------|
| Tauri IPC 커맨드 3단계 패턴 (정의 → 등록 → 래퍼) | 통과 — `reinstate_student`, `delete_schedule`, `get_operating_hours`, `save_operating_hours`, `reorder_codes` 모두 3단계 완비 |
| SQLx `query!` 매크로 사용 | 통과 — `reorder_codes`만 dynamic query 사용하나 table명이 enum 매핑이므로 인젝션 위험 없음 |
| 에러 타입 (`thiserror::Error`) 사용 | 통과 — `AppError::Db`, `AppError::UserFacing` 패턴 일관 적용 |
| 감사 로그 기록 | 통과 — `StudentReinstated` 이벤트 추가 |
| PRD §6.2 UNIQUE 제약 준수 | 통과 — Sprint 4 DB 마이그레이션 없음. 기존 제약 그대로 유지 |

### AI 생성 코드 추가 체크

| 항목 | 결과 |
|------|------|
| `schedule_days_csv` correlated subquery 성능 | 통과 — 100명 규모에서 SQLite 최적화로 충분. 주석에 근거 명시 |
| `formatPhone` 패턴 커버리지 | 통과 — 02 지역번호, 3자리 특수번호, 10~11자리 휴대폰 패턴 포괄. 비숫자 입력 방어 처리 완비 |
| 운영시간 검증 이중 적용 (FE+BE) | 통과 — 프론트엔드 사전 검증 + 백엔드 `save_operating_hours` 검증 모두 구현 |

---

## 이전 회고 액션 아이템 이행 결과

| ID | 항목 | 이행 |
|----|------|------|
| A7 | `paths::data_root()` 동적화 | ✅ 이미 완료(Sprint 3 직후 hotfix `82eb1b2`) |
| A8 | salt.bin 이전 | ⏸️ Sprint 5 이후 처리 (sprint4.md 명시) |
| A9 | `dialog:allow-open` 최소 권한 | ✅ T1에서 capabilities 검토. `dialog:allow-open`이 Tauri 2에서 최소 단위임 확인 후 유지 |
| A10 | 출결 토글 Undo 스택 | ⏸️ Phase 2 처리 |
| A11 | `window.confirm()` → shadcn Dialog | ✅ T1에서 완전 교체 |
| A12 | cipher on 환경 실측 | ⏸️ 인스톨러 배포 후 측정 |
| A13 | simplify 기준 명시 | ⏸️ 메타 작업 이연 |
