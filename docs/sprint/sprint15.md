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

### T0: Sprint 14 액션 아이템 해소 (3h)

- ⬜ **A95**: `dashboard.rs` `monthly_summary()` 쿼리를 `GROUP BY b.id` 패턴으로 리팩토링 — 부분 수납 확장 대비. 기존 테스트(`monthly_summary_totals_billing_and_paid`) 수정 + 엣지 케이스 1건 추가
- ⬜ **A97**: `DashboardView.tsx` 위젯 타이틀 `style={{ fontSize: '24px' }}` -> `text-2xl` Tailwind 클래스 통일 (3건). 포스트잇 동적 높이는 inline style 유지(정당)
- ⬜ A89 착수 여부 판단: `/notices/page.tsx` 1534줄 분리 필요성 vs Capacity. **결론을 T4 비고로 기록** (착수는 T4에서 Capacity 여유 시에만)

### T1: 교습소 정보 화면 (6h) · skill: frontend-design

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

- ⬜ **백엔드**: `app_settings` key `academy_info` JSON 저장 (텍스트 필드 7종)
  - `get_academy_info()` / `save_academy_info(info: AcademyInfo)` IPC 2종
  - 이미지는 기존 `save_notice_asset` / `delete_notice_asset` IPC 재사용 — 파일명을 `academy_logo.{ext}` / `academy_barcode.{ext}`로 고정
  - **DB 마이그레이션 없음** — app_settings key/value 패턴 + assets 파일 저장
- ⬜ **프론트엔드**: `/settings/info` 라우트 활성화 (현재 disabled stub → 실제 폼)
  - shadcn/ui Input + Form 구성, Pretendard 18pt/44px 준수
  - 텍스트 필드 7종 + 이미지 업로드 영역 2종
  - **이미지 업로드 UI**: Tauri Dialog `open` 파일 선택 → FileReader로 읽기 → `save_notice_asset` IPC → 미리보기 표시
  - **이미지 미리보기**: `<img>` 태그로 로컬 파일 경로 표시 (Tauri `convertFileSrc` 활용)
  - **이미지 교체**: 새 파일 선택 시 기존 파일 `delete_notice_asset` → 신규 `save_notice_asset`
  - **이미지 삭제**: "삭제" 버튼 + 확인 다이얼로그(PRD SS5.7) → `delete_notice_asset` → JSON에서 파일명 제거
  - 저장 버튼 + 성공 토스트 + 미저장 이탈 경고(PRD SS5.7)
- ⬜ **TypeScript IPC 래퍼 2종** (텍스트 저장/조회) + `src/types/settings.ts`에 `AcademyInfo` 타입 추가. 이미지 IPC는 기존 `saveNoticeAsset` / `deleteNoticeAsset` 래퍼 재사용
- ⬜ `commands/mod.rs` + `lib.rs` 커맨드 등록

### T2: 자가 진단 이력 수동 삭제 (2h)

- ⬜ **백엔드 IPC 2종** (`diagnosis.rs` 확장):
  - `delete_diagnosis_history(id: i64) -> Result<(), String>` — 행 단위 삭제
  - `clear_diagnosis_history() -> Result<(), String>` — 전체 이력 비우기
- ⬜ **프론트엔드**: `/settings/diagnosis` 이력 목록에 삭제 UI 추가
  - 행 단위: 각 이력 행에 "삭제" 아이콘 버튼 (Lucide `Trash2`)
  - 전체: 이력 목록 상단에 "이력 비우기" 버튼
  - **위험 동작 확인 다이얼로그 필수** (PRD SS5.7): "이 기록을 삭제하시겠습니까?" / "모든 진단 이력을 삭제하시겠습니까? 이 작업은 되돌릴 수 없습니다."
- ⬜ **단위 테스트**: 삭제 성공 / 존재하지 않는 id 삭제 / 빈 테이블 clear = 3건
- ⬜ `commands/mod.rs` + `lib.rs` 커맨드 등록

### T3: 접근성 감사 (5h)

- ⬜ **Pretendard 폰트 감사**:
  - 본문 18pt(16pt 하한) 전체 화면 확인 — `globals.css` 기본 폰트 사이즈 검증
  - 헤더 24pt+ 확인 (Sidebar 메뉴, 페이지 타이틀, 위젯 타이틀)
  - 행간 1.5 확인 (Tailwind `leading-relaxed` 또는 명시적 `line-height`)
- ⬜ **WCAG AA 명도 대비 검증** (4.5:1 이상):
  - 저자극 톤 배경(베이지/연그레이) 위 텍스트 대비 측정
  - 버튼 텍스트, 플레이스홀더, 비활성 상태 등 엣지 케이스
  - 알림 색상(빨강/주황/파랑) 위 텍스트 대비
  - 도구: Chrome DevTools Accessibility audit 또는 axe-core
- ⬜ **44x44px 최소 클릭 영역**:
  - 전체 버튼, 체크박스, 라디오, Select, 아이콘 버튼 점검
  - 캘린더 날짜 셀, 출결표 셀, 공지문 편집 도구 등 밀집 UI 집중 점검
- ⬜ **키보드 단축키 7종 동작 확인**:
  - F1(도움말) / Ctrl+F(검색 포커스) / Ctrl+N(신규 원생) / Ctrl+S(저장) / Ctrl+Z(Undo) / ESC(다이얼로그 닫기) / Ctrl+P(인쇄)
  - 미구현 단축키가 있으면 이번 스프린트에서 구현
- ⬜ **발견 이슈 수정**: 감사 결과 기준 미달 항목 즉시 수정
- ⬜ **접근성 감사 보고서**: `docs/sprint/sprint15/accessibility-audit.md`에 결과 기록

### T4: 기술 부채 정리 (4h)

- ⬜ **미사용 코드/import 정리**: `cargo clippy -- -D warnings` + `pnpm lint` 수정
- ⬜ **A89 판단**: `/notices/page.tsx` 1534줄 분리
  - Capacity 여유(2h+ 잔여) 시: 캔버스 / 편집 / 저장 섹션으로 3분할 시작 (완성은 Sprint 16)
  - Capacity 부족 시: Sprint 16 이연 기록
- ⬜ **코드 정리 대상 탐색**:
  - 사용되지 않는 Tauri IPC 커맨드 제거 (있을 경우)
  - `console.log` 디버깅 코드 정리
  - TypeScript `as` 타입 단언 최소화
  - 중복 유틸리티 함수 통합
- ⬜ **문서 정비**: CLAUDE.md 마이그레이션 현황 확인 (V305 최신 유지)

### T5: 마이너 기능 및 UI 개선 — 가변 버퍼 (3h)

> **성격**: 사용자 검증 병행 수집 Task. T0~T4 구현 결과를 사용자가 검증하면서 발견하는 마이너 기능 요청·UI 개선 사항을 이 Task에서 일괄 처리한다. Sprint 14 회고(A94)에서 "검증-phase에서 추가 요청이 대규모 발생"한 패턴을 선제적으로 수용한 버퍼 슬롯이다.

- ⬜ **요구사항 수집**: T0~T4 사용자 검증 시 발견되는 마이너 기능/UI 개선 항목을 scope.md `## T5 요구사항` 섹션에 실시간 기록
- ⬜ **착수 기준**: 개별 항목이 2h 이내, DB 마이그레이션 불필요, 신규 의존성 불필요인 경우에만 T5 범위에 포함. 이를 초과하면 Sprint 16 이연
- ⬜ **예상 후보** (확정 아님 — 사용자 검증 시 구체화):
  - 대시보드 위젯 레이아웃/문구 미세 조정
  - 설정 화면 UX 개선 (순서, 라벨, 안내 문구)
  - 접근성 감사(T3)에서 발견된 Medium/Low 항목 추가 수정
  - 기존 화면의 사소한 버그/불편 사항
- ⬜ **완료 시 기록**: 처리된 항목 목록을 scope.md에 최종 확정 (sprint-close 시 CHANGELOG 반영)

### T6: 성능 프로파일링 + 최적화 (5h)

> **배치 근거**: T0~T5의 모든 코드 변경이 반영된 최종 상태에서 측정해야 정확한 성능 기준선을 얻을 수 있다. 중간에 측정하면 이후 변경으로 재측정이 필요해 시간 낭비.

- ⬜ **PRD 성능 기준 5종 측정** (실측, 50명 데이터 기준):
  1. 화면 전환 300ms 이내
  2. 출결표 50명 x 31일 렌더링 1초 이내
  3. 청구 50명 생성 3초 이내
  4. 공지문 50장 생성 30초 이내
  5. 앱 시작 ~ 메인 화면 진입 3초 이내
- ⬜ **병목 프로파일링**:
  - Rust: `flamegraph` 또는 `cargo bench` 기반 핫스팟 탐색
  - Frontend: Chrome DevTools Performance 패널 (React 렌더 사이클, IPC 왕복 시간)
- ⬜ **개선 적용**: 측정 결과 기준 미달 항목에 대해:
  - SQL 쿼리 인덱스 추가 또는 쿼리 최적화
  - React 컴포넌트 memoization (React.memo, useMemo)
  - TanStack Query staleTime/cacheTime 조정
  - 불필요한 re-render 제거
- ⬜ **성능 측정 보고서**: `docs/sprint/sprint15/performance-report.md`에 기준 대비 실측치 기록

### T7: 양 OS 빌드 검증 (4h)

> **배치 근거**: T6(성능 최적화)에서 SQL 인덱스·쿼리·React 렌더 등 코드 변경이 확정된 후에 인스톨러를 빌드해야 재빌드를 줄일 수 있다. 최적화 전에 빌드하면 최적화 적용 후 다시 빌드·검증해야 하므로 시간 낭비.

- ⬜ **macOS .dmg 빌드 및 검증** (집 Mac에서 직접):
  - `pnpm tauri:build` 실행 → `.dmg` 생성 확인
  - 설치 → 실행 → 정상 동작(대시보드 진입, IPC 응답) → 삭제
  - Apple Silicon 바이너리 확인 (`file` 명령으로 arch 검증)
- ⬜ **Windows .msi/.exe 빌드 검증** (GitHub Actions CI):
  - `v0.7.0-beta` 태그 push → CI matrix(windows-latest) 빌드 확인
  - 산출물 다운로드 → Windows PC에서 설치/실행/언인스톨 테스트
  - WebView2 런타임 확인
- ⬜ **인스톨러 체크리스트**:
  - 앱 아이콘 정상 표시
  - 시작 메뉴/Dock 등록
  - 업데이트 설치 시 기존 데이터 유지 확인
  - 언인스톨 후 잔여 파일 확인
- ⬜ **빌드 이슈 기록**: 발견 이슈는 `docs/sprint/sprint15/build-issues.md`에 기록 + 즉시 수정

### T8: 양 PC 동기화 시나리오 테스트 (3h)

> **배치 근거**: 전체 기능이 안정되고 빌드 검증까지 완료된 후 실행해야 재테스트를 줄일 수 있다. T7(빌드 검증)에서 발견된 이슈 수정이 동기화 동작에 영향을 줄 수 있으므로 T7 이후에 배치.

- ⬜ **시나리오 1: Windows -> Mac 전환**:
  - Windows에서 원생 등록/출결 입력/청구 생성 → 앱 종료 (app.lock 해제 확인)
  - 클라우드 동기화 완료 대기
  - Mac에서 앱 시작 → app.lock 획득 → 데이터 정합성 확인
- ⬜ **시나리오 2: Mac -> Windows 전환** (역방향):
  - 동일 흐름 역방향 실행
- ⬜ **시나리오 3: 비정상 종료 후 전환**:
  - 한쪽 PC에서 강제 종료 (app.lock 미해제)
  - 다른 PC에서 5분 임계값 경과 후 강제 점유
  - 데이터 무결성 확인 (`PRAGMA integrity_check`)
- ⬜ **동기화 테스트 보고서**: `docs/sprint/sprint15/sync-test-report.md`

### T9: 통합 검증 (3h)

- ⬜ `cargo test --lib --manifest-path src-tauri/Cargo.toml` 전수 통과 (370+ 예상)
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- ⬜ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ⬜ `pnpm lint` clean
- ⬜ `pnpm tsc --noEmit` clean
- ⬜ `pnpm build` static export 성공
- ⬜ 교습소 정보 저장/조회 + 이미지 업로드/미리보기/삭제 시각 검증
- ⬜ 자가 진단 이력 삭제 시각 검증
- ⬜ T5(마이너 기능) 처리 항목 시각 검증
- ⬜ 접근성 감사 수정 항목 재검증
- ⬜ 성능 측정 기준 전수 통과 확인

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

**필수**
- ⬜ macOS .dmg 빌드 설치/실행/삭제 정상
- ⬜ Windows .msi/.exe CI 빌드 성공 (설치/실행은 Windows PC 방문 시)
- ⬜ PRD 성능 기준 5종 중 최소 4종 충족 (미달 1종은 원인 분석 + Sprint 16 대응 계획)
- ⬜ 접근성 감사 — Pretendard 18pt 본문 / WCAG AA 4.5:1 / 44x44px 클릭 영역 전체 통과
- ⬜ 키보드 단축키 7종 정상 동작
- ⬜ 교습소 정보 저장/조회 정상 — 텍스트 7필드 + 이미지 2종 업로드/미리보기/삭제
- ⬜ 자가 진단 이력 행 단위 삭제 + 전체 비우기 정상
- ⬜ 양 PC 동기화 시나리오 3종 중 최소 시나리오 1 통과 (시나리오 2~3은 Windows PC 의존)
- ⬜ monthly_summary GROUP BY 리팩토링 완료 (R99 해소)
- ⬜ `cargo test --lib` 전수 통과 (370+)
- ⬜ `cargo clippy -- -D warnings` clean
- ⬜ `cargo check --features cipher` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md Sprint 15 상태 업데이트
- ⬜ CHANGELOG.md 업데이트
- ⬜ DEPLOY.md 업데이트

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
