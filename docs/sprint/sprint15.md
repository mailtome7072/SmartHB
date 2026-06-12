# Sprint Plan sprint15

## 기간
2026-06-07 ~ 2026-06-20 (2주)

## 목표
양 OS 빌드 산출물(인스톨러)의 설치/실행/삭제를 검증하고, 전체 화면 접근성 감사(Pretendard 18pt/WCAG AA/44x44px)를 완료하며, 교습소 정보 화면과 자가 진단 이력 수동 삭제 등 소규모 기능을 추가하여 v1.0 UAT 진입 준비를 완성한다.

## ROADMAP 연계 기능
- Phase 6: 양 OS 빌드 검증 + 최적화 + 접근성 감사 (ROADMAP Sprint 15)
- 이연 기능: 교습소 정보 화면 (ROADMAP 826행), 자가 진단 이력 수동 삭제 (ROADMAP 838행)
- Sprint 14 회고 액션 아이템: A95(monthly_summary 리팩토링), A97(위젯 inline style 통일)
- 리스크 R99: monthly_summary 1:1 청구-수납 암묵 의존 해소

## Capacity 분석

### Velocity 참조 (과거 실적)

| Sprint | 계획(h) | 특성 | 비고 |
|--------|---------|------|------|
| 11 | 30.5h | 청구+수납 전 스택 | 대규모, post-develop 보완 7커밋 |
| 12 | 37h | 공지문 Canvas + PIN | scope 초과 발생 |
| 13 | 38h | PIN 옵션화 + carry-over | 소형, stale 발견 |
| 14 | 38h | 대시보드+자가진단+내보내기 | 검증-phase에서 기능 대폭 추가(엑셀 전환, 생일 위젯 등) |

**패턴 분석**:
- 계획 38h 내외가 반복적으로 수용됨. 검증-phase 추가 요청이 2~3h 소진.
- Sprint 14는 검증-phase에서 기능 추가가 대규모였으나 수용 완료.
- **Sprint 15는 안정화 스프린트**: 새 비즈니스 로직 구현보다 검증/감사/리팩토링 비중이 높아 계획 대비 초과 확률이 낮음.

### Capacity 산정

| 항목 | 값 |
|------|-----|
| 팀 인원 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10일 |
| 실작업 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |

| 영역 | 예상 소요 | 비고 |
|------|----------|------|
| T0 Sprint 14 액션 아이템 해소 | 3h | A95 리팩토링 + A97 inline style + A89 검토 |
| T1 교습소 정보 화면 | 6h | 텍스트 필드 6종 + 이미지 2종(로고·바코드) 업로드/미리보기/삭제 |
| T2 자가 진단 이력 수동 삭제 | 2h | IPC 2종 + UI 삭제 버튼 + 확인 다이얼로그 |
| T3 접근성 감사 | 5h | 전체 화면 Pretendard/WCAG AA/44x44px 점검 + 수정 |
| T4 기술 부채 정리 | 4h | 미사용 코드 제거, lint/clippy 정리, A89 분리(선택) |
| T5 마이너 기능 및 UI 개선 (가변 버퍼) | 3h | 사용자 검증 병행 수집 — 범위 TBD |
| T6 성능 프로파일링 + 최적화 | 5h | PRD 성능 기준 5종 측정 + 병목 개선 |
| T7 양 OS 빌드 검증 | 4h | 최적화 확정 후 인스톨러 빌드·검증 |
| T8 양 PC 동기화 시나리오 테스트 | 3h | Win→Mac 전환 데이터 동기화 검증 |
| T9 통합 검증 | 3h | cargo test + clippy + cipher check + lint + tsc + build |
| **합계** | **38h** | 가용 40h 이내 (여유 2h) |

> **Capacity 변경 이력**:
> - rev1(2026-06-07): 초판 T8(내보내기 비밀번호 보호, 5h)을 Sprint 16으로 이연, 5h를 가변 버퍼로 재배분.
> - rev2(2026-06-07): 작업 순서 재배치 — 성능 프로파일링·빌드 검증·동기화 테스트를 후반에 배치.
> - rev3(2026-06-07): 내보내기 비밀번호 보호 **완전 취소** (어느 스프린트에도 배치하지 않음). T1(교습소 정보) 이미지 2종 추가로 4h→6h(+2h), T5(가변 버퍼) 5h→3h(-2h)로 재배분. 합계 38h 유지.

### 후보 작업 풀 우선순위 결정 (Capacity 재산정 결과)

ROADMAP 824행이 sprint-planner에게 Capacity 재산정 및 재배치를 명시. 후보 8건을 평가하여 Sprint 15 포함/이연을 결정한다.

| # | 후보 작업 | 우선순위 | 판정 | 근거 |
|---|----------|---------|------|------|
| 1 | 양 OS 빌드 검증 | 필수 | **Sprint 15** | ROADMAP 핵심 목표. v1.0 릴리즈 전 필수. |
| 2 | 성능 최적화 | 필수 | **Sprint 15** | ROADMAP 핵심 목표. PRD 성능 기준 검증. |
| 3 | 접근성 감사 | 필수 | **Sprint 15** | ROADMAP 핵심 목표. 50대 사용자 친화 필수. |
| 4 | 교습소 정보 화면 | 중간 | **Sprint 15** | 저위험, DB 변경 없음, 기존 assets 패턴 재사용. UAT 전 완성 필요. |
| 5 | DB 폴더 변경 (경로 재지정) | 높음 | **Sprint 16 이연** | 고위험, 독립 Task 규모(10h+). cipher ON + 양 PC + salt.bin 동반 이전 + 앱 재시작. 안정화 스프린트에 넣으면 Capacity 초과 + 리스크 누적. R12 salt.bin과 함께 Sprint 16에서 UAT 환경 준비와 병행 처리. |
| 6 | ~~내보내기 비밀번호 보호~~ | — | **취소** | 사용자 결정(2026-06-07)으로 작업 자체를 취소. 비보호 엑셀 내보내기(Sprint 14 완성)로 충분. |
| 7 | CSV 가져오기 | 높음 | **Sprint 16 이연** | PRD SS4.13.1. Sprint 16 UAT에서 원장 실데이터 이관(ROADMAP 876행 "CSV 가져오기로 원생 데이터 이관")에 직접 사용. Sprint 16 T1에 배치하여 UAT 환경 준비의 첫 번째 작업으로 실행. |
| 8 | monthly_summary 리팩토링 | 중간 | **Sprint 15** | A95 액션 아이템. 3h 이내 소규모 리팩토링. GROUP BY 패턴 전환으로 부분 수납 확장 대비. R99 해소. |

**이연/취소 판단 상세**:

- **DB 폴더 변경**: ROADMAP 830~837행에 "고위험, 독립 Task 규모"로 명시. copy-then-switch + salt.bin/WAL/backup 동반 이전 + 앱 재시작 + cipher ON 검증 + 양 PC 시나리오가 필요하며, 중간 강제 종료 시 옛 경로 복귀 확인까지 요구. 보수적으로 10~15h 예상. Sprint 15(안정화)에 넣으면 38h+10h=48h로 Capacity 초과이며, 안정화 취지(검증/감사)와 기능 추가(DB 폴더 변경)가 충돌. Sprint 16 UAT 환경 준비(양 PC 설치)와 시너지가 높으므로 Sprint 16 초반에 배치.
- **내보내기 비밀번호 보호**: 사용자 결정(2026-06-07)으로 **작업 자체를 완전 취소**. 비보호 엑셀 내보내기가 Sprint 14에서 완성되어 충분. 어느 스프린트에도 배치하지 않음. R100(rust_xlsxwriter 비번 API)도 취소 종결.
- **CSV 가져오기**: Sprint 16 UAT의 첫 단계가 "원장 실데이터 일부 마이그레이션 — CSV 가져오기로 원생 데이터 이관"(ROADMAP 876행). Sprint 16 T1에 배치하면 UAT 흐름과 자연스럽게 연결.
- **자가 진단 이력 수동 삭제**: Sprint 14에서 "완전 0건" 정책 + `reconcile_resolved_issues` 자동 정리가 도입되어 실질적 필요성이 낮아졌으나(ROADMAP 838행 비고), B안(수동 삭제)은 원장 결정 사항이므로 Sprint 15에서 소규모(2h) 구현.
- **E2E 테스트(UC-1~UC-5)**: ROADMAP 812행에 명시되어 있으나, Tauri WebDriver 설정이 프로젝트에 아직 도입되지 않았고 UC 5종 전체 자동화는 별도 인프라 세팅(8~12h)이 필요. Sprint 15 Capacity에 넣으면 초과. Sprint 16 UAT에서 원장 수동 테스트가 실질적 E2E 역할을 하므로, 자동화 E2E는 Post-MVP backlog로 이연.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint14-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 |
|---------|------|-------------------|
| A94 | 계획 시 위젯 동작·UI를 더 구체적으로 확정 | **적용** — Sprint 15는 안정화 스프린트로 신규 위젯 없음. T1(교습소 정보)의 필드 목록·이미지 저장 방식을 계획 단계에서 확정 |
| A95 | `monthly_summary` GROUP BY 리팩토링 | T0에서 처리 (R99 해소) |
| A96 | 복원 리허설 dev 환경 개선 | Sprint 15 범위 외 — Low 우선순위, 기회 발생 시 T4에서 검토 |
| A97 | 대시보드 위젯 inline fontSize -> Tailwind 통일 | T0에서 처리 |
| A89 | 공지문 페이지 분리 검토 | T4(기술 부채)에서 판단 — Capacity 여유 시 착수, 아니면 Sprint 16 이연 |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-06-02.md`

| 리스크 ID | 설명 | 반영 방법 |
|-----------|------|----------|
| R99 | monthly_summary 1:1 청구-수납 암묵 의존 | T0에서 GROUP BY b.id 리팩토링으로 해소 |

---

## 작업 목록

> **실행 순서**: T0~T3(기능 구현+감사) → T4(부채 정리) → T5(사용자 검증 병행 가변 작업) → T6(성능 프로파일링 — 코드 변경 완료 후 측정) → T7(빌드 검증 — 최적화 확정 후 인스톨러 빌드) → T8(양 PC 동기화 — 최종 안정 상태에서 테스트) → T9(통합 검증).

### T0: Sprint 14 액션 아이템 해소 (3h) ✅ 완료

- ✅ **A95**: `dashboard.rs` `monthly_summary()` 쿼리를 GROUP BY 서브쿼리 패턴으로 리팩토링 (R99 해소). 기존 테스트 수정 + 엣지 케이스 추가
- ✅ **A97**: `DashboardView.tsx` 위젯 타이틀 `style={{ fontSize: '22px' }}` → Tailwind `text-2xl` 통일
- ✅ A89 착수 여부 판단: 로직은 이미 `notice-generator.ts` 등 별도 모듈로 분리 완료 확인 → UI 구획화(`/notices/page.tsx` 3분할)만 Sprint 16 이연 결정

### T1: 교습소 정보 화면 (6h) ✅ 완료 · skill: frontend-design

#### 필드 목록 (확정)

| 필드 | 타입 | 필수 | 비고 |
|------|------|------|------|
| 교습소명 | 문자열 | 필수 | |
| 대표자(원장명) | 문자열 | 필수 | |
| 연락처 | 문자열 | 필수 | |
| 주소 | 문자열 | 필수 | |
| 사업자등록번호 | 문자열 | 선택 | |
| 교습 최대인원 수 | 정수 | 선택 | |
| 교습소 면적 | 숫자 | 선택 | 단위: 제곱미터(m2) |
| 교습소 로고 이미지 | 이미지 파일 | 선택 | PNG/JPG, 파일 저장 |
| 교습소 2D바코드 이미지 | 이미지 파일 | 선택 | PNG/JPG, 파일 저장 |

- **공지문 헤더 노출 없음** — `academy_info`는 교습소 정보 화면 전용 데이터이며 공지문(SS4.10)에 연동하지 않는다. 향후 공지문 헤더에 교습소명을 넣는 요구가 발생하면 별도 작업으로 처리.

#### 이미지 저장 방식 (권장안: 파일 저장 — `assets/` 패턴 재사용)

**트레이드오프 검토**:

| 방안 | 장점 | 단점 |
|------|------|------|
| A: `app_settings` JSON에 base64 인라인 | 구현 단순, 단일 key로 관리 | DB(app_settings 행) 비대화 — 로고+바코드 200KB~1MB. `get_settings`/`save_settings` IPC 호출마다 이미지를 직렬화/역직렬화. 모든 설정 읽기에 부하 전파. |
| **B: `{data_root}/assets/` 파일 저장 + 경로만 JSON** | 기존 `save_notice_asset` IPC 패턴 재사용. DB 경량 유지. 양 PC 클라우드 동기화 폴더에 파일이 자동 동기화. | 파일 참조 무결성 관리 필요(삭제 시 파일도 제거). |

**결정: B안 (파일 저장)** — 기존 Sprint 12 공지문 배경서식(`save_notice_asset`)과 동일 패턴. `{data_root}/assets/academy_logo.png`, `{data_root}/assets/academy_barcode.png` 고정 파일명으로 저장. 경로는 `academy_info` JSON에 파일명만 기록(전체 경로는 `paths::assets_dir()` + 파일명으로 조합). 삭제 시 `delete_notice_asset` 패턴 재사용.

#### 구현 상세

- ✅ **백엔드**: `app_settings` key `academy_info` JSON 저장 (텍스트 필드 9종)
  - `get_academy_info()` / `save_academy_info(info: AcademyInfo)` IPC 2종
  - 이미지는 기존 `save_notice_asset` / `delete_notice_asset` IPC 재사용
  - DB 마이그레이션 없음
- ✅ **프론트엔드**: `/settings/info` 라우트 활성화 — 텍스트 9필드 + 이미지 2종 업로드/미리보기/삭제. 설정 허브 카드 활성화
- ✅ **TypeScript IPC 래퍼 2종** + `AcademyInfo` 타입 추가
- ✅ `commands/mod.rs` + `lib.rs` 커맨드 등록

### T2: 자가 진단 이력 수동 삭제 (2h) ✅ 완료

- ✅ **백엔드 IPC 2종** (`diagnosis.rs` 확장): `delete_diagnosis_history(id)` / `clear_diagnosis_history()`
- ✅ **프론트엔드**: 행 단위 삭제 버튼 + 전체 비우기 버튼 + 확인 모달 (PRD §5.7)
- ✅ **단위 테스트**: 삭제 성공 / 존재하지 않는 id / 빈 테이블 clear — 3건
- ✅ `commands/mod.rs` + `lib.rs` 커맨드 등록

### T3: 접근성 감사 (5h) ✅ 완료

- ✅ **WCAG AA 명도 대비 수정**: `text-gray-400` → `text-gray-600` 17건 (Critical, 전체 화면)
- ✅ **GlobalShortcuts 컴포넌트 신설**: Ctrl+F(검색 포커스) / Ctrl+N(신규 원생) 전역 등록
- ✅ **접근성 감사 보고서**: `docs/sprint/sprint15/accessibility-audit.md`
- ⚠️ **Sprint 16 이연**: 밀집 UI 44px 기준 미달(캘린더 셀·출결표 셀), gray-500 잔존, F1/Ctrl+S 미구현 — Medium/Low 항목

### T4: 기술 부채 정리 (4h) ✅ 완료

- ✅ **clippy --all-targets 부채 6건 해소**: 테스트 코드 및 비프로덕션 경고 수정
- ✅ **A89 판단**: 로직 분리 이미 완료(notice-generator.ts 등) 확인 → UI 구획화만 Sprint 16 이연 결정. 기록 완료
- ⬜ A89 `/notices` UI 구획화 → Sprint 16 이연

### T5: 마이너 기능 및 UI 개선 — 가변 버퍼 (3h) ✅ 완료 (사용자 시각 검증 완료)

처리된 항목:
- ✅ 설정 허브 카드 순서 변경 (PIN 위치 조정)
- ✅ '마법사 재실행' 카드 → 'DB 폴더 변경(예정)' disabled 카드 + Sprint 16 안내 힌트
- ✅ 원생 상세 화면 '원생 관리 메인으로' 버튼 추가
- ✅ 전역 `GlobalTooltip` 컴포넌트 도입 (브라우저 `title` 툴팁 → 20px 커스텀 팝업 통일)
- ✅ 대시보드 위젯 폰트 미세 조정 (월 보기 수업 인원 hover 팝업, 설정 힌트 텍스트)

### T6: 성능 프로파일링 + 최적화 (5h) ✅ 완료

- ✅ **청구 생성 `standard_fees` N+1 쿼리 제거**: 학생 루프 안 개별 조회 → IN 쿼리 단일 배치
- ✅ **성능 측정 보고서**: `docs/sprint/sprint15/performance-report.md`
- ⚠️ 출결표 N+1 재설계(`get_attendance_grid` 학생 루프 200쿼리) — 실측 1초 이내 통과 중. 데이터 누적 대비 예방적 최적화는 Sprint 16 이연 결정

### T7: 양 OS 빌드 검증 (4h) ⬜ Sprint 16 이연 (사용자 결정 2026-06-07)

물리 환경 의존(Windows PC 교습소 방문 필요). Sprint 16 UAT(양 PC 설치)와 통합하여 진행.

### T8: 양 PC 동기화 시나리오 테스트 (3h) ⬜ Sprint 16 이연 (사용자 결정 2026-06-07)

물리 환경 의존(양 PC + 클라우드 동기화). Sprint 16 UAT와 통합하여 진행.

### T9: 통합 검증 (3h) ✅ 코드 검증 완료 / 빌드 산출물 Sprint 16 이연

코드 자동 검증 전수 통과:
- ✅ `cargo test --lib` 375 passed
- ✅ `cargo clippy --all-targets -- -D warnings` clean
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` clean
- ✅ `pnpm tsc --noEmit` clean
- ✅ `pnpm build` static export 성공
- ✅ 교습소 정보 · 자가진단 이력 삭제 · T5 마이너 기능 시각 검증 완료

⬜ 빌드 산출물(.dmg/.msi) 설치 검증 → Sprint 16
⬜ 양 OS 실기동 검증 → Sprint 16

---

## 신규 의존성

없음 — 기존 의존성(`rust_xlsxwriter`, `recharts` 등)만 활용. 이미지 업로드는 기존 Tauri Dialog 플러그인 + `save_notice_asset` IPC 재사용.

## 신규 마이그레이션

없음 — 교습소 정보는 `app_settings` key/value JSON 저장 + `assets/` 파일 저장 (V305 최신 유지).

## scope 경계 (포함/제외)

### 포함
- 양 OS 빌드 산출물 검증 (macOS .dmg + Windows .msi/.exe)
- 성능 프로파일링 + 최적화 (PRD 5종 기준)
- 접근성 감사 (Pretendard/WCAG AA/44x44px/키보드 단축키)
- 교습소 정보 화면 신설 (`/settings/info`) — 텍스트 7필드 + 이미지 2종(로고·바코드)
- 자가 진단 이력 수동 삭제 (행 단위 + 전체 비우기)
- 마이너 기능 및 UI 개선 (사용자 검증 병행, 가변 버퍼 3h)
- Sprint 14 액션 아이템 해소 (A95/A97/A89 판단)
- 양 PC 동기화 시나리오 테스트
- 기술 부채 정리 (미사용 코드, lint, 문서 정비)
- R99 리스크 해소 (monthly_summary 리팩토링)

### 취소
- ~~**내보내기 비밀번호 보호**~~ (AC-4.13-4) — 사용자 결정(2026-06-07)으로 완전 취소. 비보호 엑셀로 충분.

### 제외 (Sprint 16 이연)
- **DB 폴더 변경(경로 재지정)** — 고위험, 10~15h. R12 salt.bin 이전과 함께 Sprint 16 설계. ADR 필요
- **CSV 가져오기** (PRD SS4.13.1) — Sprint 16 UAT 환경 준비의 첫 번째 작업으로 배치
- **E2E 테스트 자동화** (UC-1~UC-5, Tauri WebDriver) — 인프라 세팅 8~12h, Post-MVP backlog
- **A89 공지문 페이지 분리** — T4에서 Capacity 여유 시만 착수, 아니면 Sprint 16 이연
- **A96 복원 리허설 dev 환경 개선** — Low, 기회 발생 시

### 제외 (Post-MVP backlog)
- 반응형 폰트/셀 너비 (Sprint 8 backlog)
- 한글 자모 검색 (Sprint 8 backlog)
- N+1 쿼리 최적화 (Sprint 8 backlog)
- AC-4.11-2 새벽 자동 갱신 (Sprint 14 이연)

---

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R101 | Windows PC 물리 접근 제한 — 집 Mac에서만 개발하므로 Windows 빌드 직접 테스트 불가 | 중간 | GitHub Actions CI matrix로 Windows 빌드 자동화. 실제 Windows 설치/실행 테스트는 교습소 PC 방문 시 수행. CI 산출물 다운로드 + VM 테스트를 대안으로 검토 |
| R102 | 접근성 감사에서 대규모 CSS 수정 발생 — Pretendard 18pt/44x44px 미달 화면이 예상보다 많을 경우 T3 시간 초과 | 중간 | T3를 5h로 넉넉하게 배정. 수정 범위가 8h 이상이면 Critical 항목만 Sprint 15에서 수정하고 Medium/Low는 T5 버퍼 또는 Sprint 16 이연 |
| R103 | 성능 기준 미달 항목 최적화 난이도 — 50명 x 31일 출결표 렌더링 등에서 React 렌더 병목이 예상보다 심할 수 있음 | 중간 | React.memo + 가상화(react-window) 검토. 가상화 도입 시 신규 의존성 필요 → 사용자 확인 후 진행 |
| R104 | T5(가변 버퍼) 범위 초과 — 사용자 검증에서 3h 이상의 요청이 발생하여 후속 T6~T9에 영향 | 중간 | T5 착수 기준(개별 2h 이내, DB 변경 없음, 신규 의존성 없음) 엄수. 초과분은 Sprint 16 이연. 3h 소진 시점에서 즉시 중단하고 T6로 전환 |

> ~~R100(rust_xlsxwriter 비번 API)~~: 내보내기 비밀번호 보호 작업 완전 취소에 따라 **종결(드롭)**. 대응 불필요.

---

## 완료 기준 (Definition of Done)

**완료 (2026-06-07)**
- ✅ 교습소 정보 저장/조회 정상 — 텍스트 9필드 + 이미지 2종 업로드/미리보기/삭제
- ✅ 자가 진단 이력 행 단위 삭제 + 전체 비우기 정상
- ✅ monthly_summary GROUP BY 리팩토링 완료 (R99 해소)
- ✅ 접근성 Critical 수정 — WCAG AA 명도 대비 17건, 전역 단축키 Ctrl+F/Ctrl+N
- ✅ 청구 생성 N+1 쿼리 제거 (T6)
- ✅ `cargo test --lib` 375 passed
- ✅ `cargo clippy --all-targets -- -D warnings` clean
- ✅ `cargo check --features cipher` 통과
- ✅ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과

**Sprint 16 이연 (사용자 결정 2026-06-07)**
- ⬜ macOS .dmg / Windows .msi 빌드 설치/실행 검증 (T7)
- ⬜ 양 PC 동기화 시나리오 (T8)
- ⬜ 접근성 Medium/Low — 밀집UI 44px, gray-500, F1, Ctrl+S

**프로세스 (sprint-close 완료)**
- ✅ ROADMAP.md Sprint 15 상태 업데이트
- ✅ CHANGELOG.md 업데이트
- ✅ DEPLOY.md 업데이트

---

## 예상 산출물

| 산출물 | 경로 |
|--------|------|
| 스프린트 계획 | `docs/sprint/sprint15.md` |
| 접근성 감사 보고서 | `docs/sprint/sprint15/accessibility-audit.md` |
| 성능 측정 보고서 | `docs/sprint/sprint15/performance-report.md` |
| 동기화 테스트 보고서 | `docs/sprint/sprint15/sync-test-report.md` |
| 빌드 이슈 기록 | `docs/sprint/sprint15/build-issues.md` |
| 교습소 정보 IPC | `src-tauri/src/commands/settings.rs` 확장 또는 별도 함수 |
| 진단 이력 삭제 IPC | `src-tauri/src/commands/diagnosis.rs` 확장 |
| 교습소 정보 UI | `src/app/settings/info/page.tsx` (stub -> 실제 폼 + 이미지 업로드) |
| 진단 이력 삭제 UI | `src/app/settings/diagnosis/` 확장 |
| 교습소 이미지 파일 | `{data_root}/assets/academy_logo.{ext}`, `academy_barcode.{ext}` |
| TypeScript 타입 | `src/types/settings.ts` 확장 |
| IPC 래퍼 | `src/lib/tauri/index.ts` 확장 (4종+) |

---

## 참고 사항

- **v0.6.0 배포가 보류 중**: Sprint 14 완료 후 develop->master 머지 + v0.6.0 태그 push가 아직 미수행(사용자 보류). Sprint 15 진입 전 또는 병행으로 deploy-prod 에이전트 실행이 필요. Sprint 15 브랜치는 develop 기반으로 생성하므로 배포 보류 상태와 무관하게 진행 가능.
- **Node 25 환경**: 집 Mac에 Node 25만 설치되어 있음. `pnpm tauri:dev` 중 `pnpm build` 금지, dev 화면 깨지면 `.next` 삭제 후 재기동 (Sprint 14 회고).
- **cipher feature**: 개발 빌드는 cipher off, 프로덕션/CI는 cipher on. `cargo check --features cipher`로 컴파일 확인만 수행.
- **Sprint 16 예고**: DB 폴더 변경(R12 salt.bin 동반) + CSV 가져오기 + UAT 환경 준비 + 원장 2주 파일럿. Sprint 15 완료 후 Sprint 16 계획 수립 시 ROADMAP 871~907행 참조.
