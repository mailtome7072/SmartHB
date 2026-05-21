# Sprint Plan sprint4

## 기간
2026-05-22 ~ 2026-06-04 (2주, 예상)

## 목표
Sprint 3 스테이징 검증에서 발견된 **14개 이슈**(Critical Runtime Error 1건 + 사용자 보고 13건)를 모두 해결하여 Phase 1 품질을 확립하고, Phase 2(학사+출결) 진입 기반을 완성한다. 교습소 설정 화면 신설, 원생 관리 UX 개선, 코드 테이블 완성도 향상, 수업 스케줄 편집 기능 보강이 핵심이다.

## ROADMAP 연계 기능
- Phase 1 잔존 이슈 해소 (Sprint 3 스테이징 검증 14건)
- §4.0 초기 설정 마법사 — 마법사 외 영구 설정 화면(§4.12) 신설
- §4.1 원생 관리 — 퇴교일 필드, 퇴교 번복, 학교명 연동, 일련번호 보호
- §4.2 수업 스케줄 — 시작시간 콤보박스, 수정/삭제 기능
- §4.12 코드 테이블 — 드래그 순서 변경, 활성 상태 필터
- §5.7 50대 친화 UX — 연락처 자동 하이픈, 천단위 콤마, shadcn Dialog

> **참고**: ROADMAP.md의 Sprint 4 원래 범위(학사 스케줄 관리)는 Sprint 5로 이연된다. 본 Sprint 4는 Phase 1 품질 안정화 sprint이다.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint3-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 방법 |
|---------|------|----------------------|
| A7 | `paths::data_root()` 동적화 | **이미 완료** (`82eb1b2`). 추가 작업 불필요 |
| A8 | salt.bin 이전 (Keychain -> cloud/smarthb/) | Sprint 5 이후 별도 처리 (본 sprint 범위 외) |
| A9 | `dialog:allow-open` -> 디렉토리 전용 권한 좁히기 | **T1에 통합** — capabilities 파일 수정 시 함께 처리 |
| A10 | 출결 토글 1단계 Undo 스택 | Phase 2 (Sprint 5~6) 출결 구현 시 처리 |
| A11 | `window.confirm()` -> shadcn/ui Dialog 교체 | **T1 핵심 작업** — Critical Runtime Error 해결과 동일 |
| A12 | cipher on 환경 실측 | v0.3.0 배포 후 측정 예정. 본 sprint 범위 외 |
| A13 | simplify 기준 "사용처 2곳 이상 시 추출 권고" 명시 | 메타 작업. 본 sprint 범위 외 |

---

## 작업 목록

### T1: Critical — window.confirm 차단 해소 + shadcn/ui Dialog 도입 + capabilities 권한 정비
> 사용자 이슈: **Critical Runtime Error** (dialog.confirm not allowed) + **A9** + **A11**

- `src/components/ui/alert-dialog.tsx` — shadcn/ui AlertDialog 컴포넌트 추가 (npx shadcn@latest add alert-dialog)
- `src/app/students/edit/page.tsx:65` — `window.confirm()` -> AlertDialog 교체 (퇴교 확인)
- 프로젝트 전체에서 `window.confirm` / `window.alert` 사용 전수 조사 후 일괄 교체
- `src-tauri/capabilities/default.json` — `dialog:allow-open` 유지 (디렉토리 선택에 필요). Tauri 2.x에서 `dialog:allow-open`이 최소 단위인지 확인 후, 하위 권한이 존재하면 `dialog:allow-open-directory`로 교체 (A9)
- **scope 파일**: `src/components/ui/alert-dialog.tsx`, `src/app/students/edit/page.tsx`, `src-tauri/capabilities/default.json`
- **DoD**: `pnpm tauri:dev` 실행 후 퇴교 처리 시 shadcn AlertDialog 표시, 콘솔에 dialog 관련 에러 없음
- **skill**: systematic-debugging

---

### T2: 교습소 설정 메뉴 화면 신설 (§4.12)
> 사용자 이슈: **#0** (교습소설정 메뉴 자체 누락)

PRD §4.12에 따른 설정 메뉴 통합 화면. 현재 `/settings/codes`만 존재하며, 상위 `/settings` 페이지와 교습소 운영 시간 설정(§4.12.2)이 누락.

**백엔드**:
- `src-tauri/src/commands/settings.rs` — 교습소 운영 시간 CRUD IPC 신규
  - `get_operating_hours()` — app_settings에서 운영 시간 JSON 로드
  - `save_operating_hours(hours: Vec<DayHours>)` — 요일별 시작/종료 저장
  - 디폴트: 월~금 13:00~19:00, 토/일 미운영
- `src-tauri/src/lib.rs` — invoke_handler에 신규 커맨드 등록

**프론트엔드**:
- `src/app/settings/page.tsx` — 설정 메뉴 허브 (교습소 정보 / 운영 시간 / 코드 테이블 / 마법사 재실행 링크)
- `src/app/settings/hours/page.tsx` — 교습소 운영 시간 편집 (요일별 시작/종료 1시간 단위 콤보)
- `src/lib/tauri/index.ts` — 운영 시간 IPC 래퍼 추가
- 사이드바에 "설정" 메뉴 항목 추가 (현재 코드 테이블만 직접 링크된 경우 상위 설정으로 변경)
- **scope 파일**: `src-tauri/src/commands/settings.rs`, `src-tauri/src/lib.rs`, `src/app/settings/page.tsx`, `src/app/settings/hours/page.tsx`, `src/lib/tauri/index.ts`, 사이드바 컴포넌트
- **DoD**: 설정 메뉴에서 운영 시간 조회/수정 가능, 코드 테이블 메뉴 접근 정상, 디폴트값 정상 표시
- **DB 변경**: 없음 (app_settings key/value 활용)

---

### T3: 상태바 점유 디바이스/백업/동기화 + 시작 시간 표시 수정
> 사용자 이슈: **#1** (상태바 "확인중..." 멈춤), **#2** (시작 시간 미표시)

- `src/app/page.tsx` — 상태바 컴포넌트의 lock status / backup status / sync status IPC 호출 디버깅
  - `check_lock_status` 응답을 상태바에 바인딩
  - 마지막 백업 시각 조회 IPC 호출 + 표시
  - 동기화 상태 표시 (클라우드 폴더 접근 가능 여부)
  - lastStartup 밀리초 표시 조건 점검 및 수정
- **scope 파일**: `src/app/page.tsx`, 상태바 관련 컴포넌트
- **DoD**: 상태바에 점유 디바이스명 / 마지막 백업 시각 / 시작 시간(ms) 정상 표시
- **skill**: systematic-debugging

---

### T4: 원생 등록/수정 — 학교명 선택란 추가 + 필터 연동
> 사용자 이슈: **#3** (학교명 선택란 누락)

학교는 V102 schools 테이블 + `list_codes('school')` IPC가 이미 존재. 프론트엔드 폼에 학교명 Select 컴포넌트 미구현 상태.

- `src/app/students/new/page.tsx` + `src/app/students/edit/page.tsx` — 학교명 Select 추가 (코드 테이블 schools 연동)
- `src/app/students/page.tsx` — 필터에 학교명 드롭다운 추가
- **scope 파일**: 원생 등록/수정/목록 페이지, student-form 관련 컴포넌트
- **DoD**: 원생 등록/수정 시 학교명 선택 가능, 목록 필터에 학교명 드롭다운 작동

---

### T5: 연락처 자동 하이픈 + 금액 천단위 콤마 유틸리티
> 사용자 이슈: **#4** (연락처 자동 하이픈), **#13** (금액 천단위 콤마)

**공통 유틸리티**:
- `src/lib/format.ts` — 포맷 유틸리티 신규
  - `formatPhone(value: string): string` — `01012345678` -> `010-1234-5678`
  - `formatCurrency(amount: number): string` — `Intl.NumberFormat('ko-KR')` 래퍼
  - `parsePhone(formatted: string): string` — 하이픈 제거 (저장용)

**적용**:
- 원생 등록/수정 폼의 전화번호 3개 필드에 onChange 자동 하이픈
- 표준 교습비 코드 테이블 금액 표시에 천단위 콤마
- 원생 상세 / 목록에서 금액 표시 위치에 콤마 적용
- 향후 Phase 4(청구/수납) 화면에서도 이 유틸리티 재사용
- **scope 파일**: `src/lib/format.ts`, 원생 폼 컴포넌트, 코드 테이블 페이지
- **DoD**: 전화번호 입력 시 실시간 하이픈 삽입, 모든 금액 표시에 천단위 콤마 적용

---

### T6: 원생 일련번호 수정 차단 (데이터 무결성)
> 사용자 이슈: **#5** (일련번호가 수정 가능)

PRD §6.2 — 일련번호 UNIQUE + PI-05 자동 채번. 수정 화면에서 편집 불가해야 한다.

- `src/app/students/edit/page.tsx` — 일련번호 input에 `readOnly` 속성 추가, 시각적으로 비활성 표시
- 백엔드 `update_student` IPC — serial_no 파라미터를 무시하거나 변경 시 에러 반환 (이중 가드)
- **scope 파일**: 원생 수정 페이지, `src-tauri/src/commands/students.rs`
- **DoD**: 수정 화면에서 일련번호 편집 불가, 백엔드에서도 serial_no 변경 시도 차단

---

### T7: 원생 등록 후 수업 스케줄 등록 UX 개선
> 사용자 이슈: **#6** (등록 직후 스케줄 등록이 별도 경로)

- 신규 원생 등록 완료 후 `/students/edit?id={newId}` 또는 원생 상세 화면으로 자동 리다이렉트
- 등록 완료 토스트에 "수업 스케줄을 등록하세요" 안내 메시지 포함
- 원생 상세/수정 화면에서 스케줄 편집 UI가 자연스럽게 접근 가능하도록 UI 배치 확인
- **scope 파일**: `src/app/students/new/page.tsx`, 라우팅 로직
- **DoD**: 원생 등록 완료 -> 수정 화면 자동 진입 -> 스케줄 편집 섹션 즉시 접근 가능

---

### T8: 퇴교일 필드 추가 + 퇴교 번복 기능 (DB 스키마 변경)
> 사용자 이슈: **#7** (퇴교일 정보 없음), **#8** (퇴교 번복 불가)

**DB 마이그레이션 V201**:
- `src-tauri/migrations/201__add_withdrawn_at_to_students.sql`
  - `ALTER TABLE students ADD COLUMN withdrawn_at TEXT;` — 퇴교 처리 시점 기록
  - 기존 `withdraw_date`는 "퇴교 예정일/퇴교일" (사용자 입력), `withdrawn_at`는 "시스템 퇴교 처리 일시" (자동)
  - 기존 퇴교 처리된 원생(withdraw_date NOT NULL)에 대해 `UPDATE students SET withdrawn_at = withdraw_date WHERE withdraw_date IS NOT NULL`

**백엔드**:
- `withdraw_student(id, withdraw_date)` IPC 수정 — withdraw_date + withdrawn_at 동시 설정
- `reinstate_student(id)` IPC 신규 — withdraw_date = NULL, withdrawn_at = NULL로 복원 (퇴교 번복)
- audit_log에 퇴교/번복 기록

**프론트엔드**:
- 퇴교 처리 시 퇴교일자 DatePicker 입력 (AlertDialog 내)
- 퇴교 원생 목록에 퇴교일 표시
- 퇴교 원생 상세 화면에 "퇴교 번복" 버튼 (AlertDialog 확인 필수)

**이슈 #8-1 포함**: 퇴교 원생 수정 화면에서 수업 스케줄 추가/변경 비활성화

- **scope 파일**: 마이그레이션 V201, `commands/students.rs`, 원생 수정 페이지, IPC 래퍼
- **DoD**: 퇴교 시 일자 입력 가능, 퇴교일 표시, 퇴교 번복 동작, 퇴교 원생 스케줄 편집 차단
- **DB 변경**: V201 마이그레이션 신규

---

### T9: 수업 스케줄 시작시간 콤보박스 + 수정/삭제 기능
> 사용자 이슈: **#9** (시작시간 콤보박스), **#10** (스케줄 수정/삭제 불가)

**#9 — 시작시간 콤보박스**:
- 교습소 운영 시간(T2에서 구현)을 기반으로 해당 요일의 가능한 시작 시간을 1시간 단위로 콤보박스 제공
- AC-4.1.1-2 준수: 시작 시간 + 1회 수업 시간 <= 운영 종료 시간
- AC-4.1.1-5 준수: 시작 시간에 따라 1회 수업 시간 옵션 동적 제한
- T2 의존: 운영 시간 데이터 로드 필요

**#10 — 수정/삭제**:
- 스케줄 편집 UI에 기존 스케줄 행별 수정/삭제 버튼 추가
- `update_schedule(id, ...)` IPC — 기존 스케줄 수정 (변경 이력 자동 생성, §4.2.2)
- `delete_schedule(id)` IPC — 스케줄 삭제 (AlertDialog 확인 필수)
- 삭제 시 effective_to 설정 (소프트 삭제) + 변경 이력 기록

- **scope 파일**: 스케줄 편집 컴포넌트, `commands/students.rs` 또는 `commands/schedules.rs`, IPC 래퍼
- **DoD**: 시작시간이 운영시간 기반 콤보로 표시, 기존 스케줄 수정/삭제 가능, 변경 이력 생성
- **의존성**: T2 (운영 시간 데이터)

---

### T10: 코드 테이블 드래그앤드롭 순서 변경 + 활성 상태 필터
> 사용자 이슈: **#11** (드래그 순서 변경 + 자동 sort_order), **#12** (전체/사용/미사용 필터)

**신규 의존성 필요**: `@dnd-kit/core` + `@dnd-kit/sortable` (드래그앤드롭 라이브러리)
> dnd-kit 선정 이유: React 18/19 호환, 접근성(키보드 DnD) 내장, 번들 크기 경량 (~15KB gzip), 사용자 허가 필요

**백엔드**:
- `reorder_codes(table: String, ids: Vec<i64>)` IPC 신규 — 전달받은 ID 순서대로 sort_order 일괄 업데이트 (트랜잭션)
- 신규 코드 추가 시 `MAX(sort_order) + 1` 자동 부여 (기존 로직 확인 후 보완)

**프론트엔드**:
- `src/app/settings/codes/page.tsx` — dnd-kit으로 행 드래그 순서 변경
  - 드래그 핸들(grabber icon) + 시각적 피드백
  - 드래그 완료 시 `reorder_codes` IPC 호출
- 상단에 "전체 / 사용 / 미사용" 라디오 버튼 그룹 (is_active 필터)
  - 프론트엔드 필터링 (데이터 소량이므로 서버 필터 불필요)

- **scope 파일**: `src/app/settings/codes/page.tsx`, `commands/codes.rs` 또는 관련 모듈, IPC 래퍼
- **DoD**: 코드 행 드래그로 순서 변경 후 새로고침해도 유지, 라디오 필터로 사용/미사용 분리 표시
- **신규 의존성**: `@dnd-kit/core`, `@dnd-kit/sortable` (사용자 허가 필요)

---

### T11: 통합 검증 + 이슈 매트릭스 재검증
> 전체 14개 이슈 재검증

- `cargo test` 전체 통과 (V201 마이그레이션 포함)
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 통과
- `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과
- `pnpm tauri:dev` 실행 후 14개 이슈 전수 재검증 (아래 매트릭스 기준)
- `.sqlx/` 오프라인 캐시 갱신 + 커밋

- **DoD**: 14개 이슈 모두 해소 확인, CI 전체 통과

---

## 사용자 보고 이슈 -> Task 매트릭스 (누락 검증용)

| 이슈 # | 이슈 설명 | Task | 상태 |
|--------|----------|------|------|
| Critical | dialog.confirm not allowed (runtime error) | T1 | ⬜ |
| #0 | 교습소설정 메뉴 자체 누락 | T2 | ⬜ |
| #1 | 상태바 점유/백업/동기화 "확인중..." 멈춤 | T3 | ⬜ |
| #2 | 상태바 시작 시간(ms) 미표시 | T3 | ⬜ |
| #3 | 원생 등록에 학교명 선택란 없음, 필터 누락 | T4 | ⬜ |
| #4 | 연락처 자동 하이픈 | T5 | ⬜ |
| #5 | 원생 일련번호 수정 가능 (데이터 무결성) | T6 | ⬜ |
| #6 | 등록 직후 스케줄 등록 별도 경로 불편 | T7 | ⬜ |
| #7 | 퇴교 처리 시 퇴교일 정보 없음 | T8 | ⬜ |
| #8 | 퇴교 번복 불가 | T8 | ⬜ |
| #8-1 | 퇴교 원생 스케줄 추가/변경 가능 (차단 필요) | T8 | ⬜ |
| #9 | 시작시간 자유입력 -> 운영시간 기반 콤보 | T9 | ⬜ |
| #10 | 수업 스케줄 수정/삭제 불가 | T9 | ⬜ |
| #11 | 코드 테이블 드래그 순서 변경 + 자동 sort_order | T10 | ⬜ |
| #12 | 코드 테이블 전체/사용/미사용 필터 | T10 | ⬜ |
| #13 | 모든 금액 표시 천단위 콤마 | T5 | ⬜ |

**14개 이슈 -> 11개 Task에 모두 매핑 완료. 누락 없음.**

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ cargo test 전체 통과 (V201 마이그레이션 포함, Rust 변경 시)
- ⬜ cargo clippy -- -D warnings 통과
- ⬜ pnpm build 성공 (Next.js static export)
- ⬜ pnpm lint + pnpm tsc --noEmit 통과
- ⬜ `pnpm tauri:dev` 실행 후 14개 사용자 보고 이슈 전수 재검증 통과
- ⬜ .sqlx/ 오프라인 캐시 갱신 및 커밋

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

---

## 신규 의존성

| 패키지 | 용도 | 사용자 허가 |
|--------|------|-----------|
| `@dnd-kit/core` | 코드 테이블 행 드래그앤드롭 (T10) | **필요** |
| `@dnd-kit/sortable` | 정렬 가능한 리스트 DnD 헬퍼 (T10) | **필요** |
| shadcn/ui AlertDialog | 확인 다이얼로그 (T1) | 불필요 (shadcn CLI로 생성, 별도 npm 패키지 아님) |

> shadcn/ui 컴포넌트는 `npx shadcn@latest add alert-dialog`로 프로젝트 내 소스로 복사되므로 별도 npm 의존성이 아니다. 단, shadcn CLI 최초 실행 시 `components.json` 설정이 필요할 수 있다 (Sprint 3에서 shadcn 초기화 여부 확인 필요).

---

## DB 마이그레이션

| 번호 | 파일명 | 내용 |
|------|--------|------|
| V201 | `201__add_withdrawn_at_to_students.sql` | students 테이블에 `withdrawn_at TEXT` 컬럼 추가 + 기존 퇴교 원생 백필 |

> Sprint 4 마이그레이션 예약 범위: V201~V299. V200은 Sprint 3에서 사용 완료.

---

## Task 의존성 그래프

```
T1 (Dialog 차단 해소) ── 최우선, 다른 Task에서 AlertDialog 사용
  |
  v
T2 (설정 메뉴) ── T9가 운영 시간 데이터에 의존
  |
  v
T9 (스케줄 콤보 + 수정/삭제) ── T2 완료 후
  
T1 -> T8 (퇴교 AlertDialog 사용)

T3, T4, T5, T6, T7, T10 ── 독립적, 병렬 가능

T11 (통합 검증) ── 모든 Task 완료 후 최종
```

**권장 실행 순서**: T1 -> T2 -> (T3~T8 병렬) -> T9 -> T10 -> T11

---

## Capacity 확인

- 팀: AI 페어 프로그래밍 1인 개발
- 스프린트 기간: 2주 (10 영업일)
- 실작업 가능 시간: 하루 4시간 = 총 40시간
- Task 수: 11개 (T11 통합 검증 포함)
- 예상 평균 소요: T1(3h) + T2(6h) + T3(3h) + T4(3h) + T5(3h) + T6(2h) + T7(2h) + T8(6h) + T9(5h) + T10(5h) + T11(3h) = **41시간**
- 여유율: -2.5% (약간 초과하나 T3~T8 병렬 가능 + AI 페어로 효율적)
- 결론: **수용 가능** (tight하지만 범위 축소 없이 진행)

---

## 위험 및 대응

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R23 | shadcn/ui 초기 설정 미완료 — `components.json` 부재 시 AlertDialog 추가 실패 | 중간 | T1 착수 시 shadcn init 여부 확인. 미설정 시 `npx shadcn@latest init` 선행 |
| R24 | dnd-kit과 React 19 호환성 — dnd-kit이 React 19를 공식 지원하는지 미확인 | 중간 | T10 착수 전 호환성 확인. 비호환 시 네이티브 HTML Drag API로 대체 |
| R25 | 상태바 IPC 응답 미연결 (T3) — lock/backup/sync 모듈의 IPC 구조 파악 필요 | 중간 | systematic-debugging으로 IPC 호출 흐름 추적. 최악 시 상태바를 비동기 폴링으로 전환 |
| R26 | V201 마이그레이션 — 기존 퇴교 원생 데이터 백필 시 edge case | 낮음 | 인메모리 DB 테스트에서 백필 로직 검증 |

---

## ROADMAP 변경 사항

본 Sprint 4는 ROADMAP.md의 원래 Sprint 4 범위(학사 스케줄 관리)와 다르다. ROADMAP 업데이트 시:
- Sprint 4 설명을 "Phase 1 품질 안정화 — 스테이징 검증 14개 이슈 해소"로 변경
- 원래 Sprint 4 범위(학사 스케줄)는 Sprint 5로 이연
- Phase 2 시작이 Sprint 5로 조정됨을 명시

> 이 변경은 sprint-close 에이전트가 ROADMAP.md를 공식 업데이트할 때 반영한다. sprint-planner는 계획 문서에 의도만 기록한다.

---

## 참고 사항

- **PRD 확인 완료**: §4.0(마법사 + 설정 재실행), §4.1(퇴교일 필드 명시 확인 — `입교일, 퇴교일`), §4.2(시작시간 1시간 단위 콤보 확인), §4.12(코드 테이블 + 운영 시간), §5.7(50대 친화 UX)
- **마이그레이션 번호**: V200 (Sprint 3) 사용 완료 -> V201부터 시작
- **shadcn/ui**: `src/components/ui/` 디렉토리가 현재 비어있음. AlertDialog 추가 전 shadcn init 필요 가능성
- **Phase 2 영향**: Sprint 4 완료로 교습소 운영 시간이 확립되면, Sprint 5(학사 스케줄) 구현 시 운영 시간 기반 제약을 바로 활용 가능
