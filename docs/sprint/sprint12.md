# Sprint Plan sprint12

## 기간
2026-05-30 ~ 2026-06-13 (2주, 예상)

## 목표
카카오톡 교습비 공지문 이미지 일괄 생성(PRD §4.10)을 완성하여 UC-5의 마지막 단계를 달성한다. 청구 데이터(원생명/청구액/청구년월)를 소스로, 배경서식 위에 텍스트박스를 배치하고 원생별 PNG를 일괄 생성하여 카카오톡 수동 발송에 사용한다. Phase 4 마지막 마일스톤(M5: 청구 완성)이다.

**ROADMAP 대시보드의 "다음 마일스톤"에 언급된 "CSV 가져오기"는 본 스프린트 범위에서 제외한다.** ROADMAP Sprint 12 작업 목록에 포함되어 있으나, PRD §4.13.1 데이터 가져오기는 Phase 6 Sprint 15(대시보드+유틸리티)의 주요 기능이다. Sprint 12에서는 공지문 이미지 생성에 집중하고, CSV 가져오기는 Sprint 15로 이연한다.

## ROADMAP 연계 기능
- PRD §4.10 교습비 공지문(이미지) 작성 (Feature 4.10.1 + 4.10.2)
- PRD §4.12.1 코드 테이블 — 공지문 저장 폴더 경로 (설정 화면 연동)
- PRD §5.6 성능 — 공지문 이미지 50장 일괄 생성 < 30초
- AC-4.10-1: 천단위 콤마 표기
- AC-4.10-2: 동일 월 재생성 시 덮어쓰기 확인
- AC-4.10-3: 텍스트박스 레이아웃 보존

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint11-retrospective.md` + `sprint11-post-retro.md` + `sprint11-post-retro2.md`

| 액션 ID | 항목 | 반영 방법 |
|---------|------|-----------|
| A70 | PaymentsView dirtyEntries payerName 필터 | T0 carry-over |
| A71 | 50명 generate_bills 3초 이내 실측 | T0 carry-over (수동 검증) |
| A72 | PaymentsView is_card_type 기반 전환 | Low, 공지문 범위 외 — 이연 |
| A73 | seed_student .bind(withdraw) 전환 | Low, T0 carry-over |
| A74~A79 | 이월 항목 6건 (pagination, JSON_EACH, N+1, salt, 반응형, 자모) | 공지문 범위 외 — 이연 유지 |
| A82 | verify_password validate_pin 미적용 주석 | T0 carry-over (1줄) |
| A85 | R86 update_bill_impl LEFT JOIN 통합 | T0 carry-over |
| A86 | R87 BillingSummaryView radio 전환 | T0 carry-over (1줄) |

## 작업 목록

### T0: Sprint 11 carry-over 정리 (4h)

Sprint 11 + post-Sprint 11 회고 액션 아이템 중 공지문 구현 전에 정리할 항목을 일괄 처리한다.

- ⬜ A70 — `PaymentsView` dirtyEntries 필터에 `d.payerName !== ''` 추가
- ⬜ A71 — 50명 시드 환경에서 `generate_bills` 3초 이내 수동 실측 (DEPLOY.md 기록)
- ⬜ A73 — `billing.rs` seed_student 테스트 헬퍼 `.bind(withdraw)` 전환
- ⬜ A82 — `auth.rs:664` verify_password에 validate_pin 미적용 의도 주석 1줄 추가
- ⬜ A85 — `billing.rs` update_bill_impl status + is_paid 단일 LEFT JOIN 통합 (R86 해소)
- ⬜ A86 — `BillingSummaryView.tsx` 년/월 토글 `type="radio"` + `role="radiogroup"` 전환 (R87 해소)

**INVEST**: Independent(이전 회고 항목 독립), Negotiable, Valuable(기술 부채 해소), Estimable(각 1줄~10줄), Small(4h), Testable(cargo test + pnpm lint 통과)

---

### T1: 백엔드 경로 헬퍼 + 배경서식/출력 디렉토리 (2h)

`paths.rs`에 공지문 관련 경로 헬퍼 함수를 추가한다. PRD §5.2 아키텍처에 명시된 `smarthb/assets/`(배경서식) + `smarthb/output/`(생성된 PNG) 경로를 `data_root()` 기반으로 구성한다.

- ⬜ `paths.rs`에 `assets_dir()`, `notice_output_dir(year_month: &str)` 함수 추가
  - `assets_dir()` → `{data_root}/assets/`
  - `notice_output_dir("2026-05")` → `{data_root}/output/202605/`
- ⬜ 디렉토리 자동 생성 (`std::fs::create_dir_all`)
- ⬜ 단위 테스트 2건: 경로 조합 정확성

**INVEST**: Independent, Negotiable(경로 구조), Valuable(이후 T4/T6 의존), Estimable, Small(2h), Testable(단위 테스트)

---

### T2: 배경서식 관리 IPC (3h)

배경서식 이미지를 `smarthb/assets/` 에 저장/조회하는 백엔드 IPC를 구현한다. 배경서식은 DB 테이블 없이 파일 시스템으로 관리한다 (이미지 파일 자체가 데이터, 메타데이터는 파일명과 수정일).

- ⬜ `notice.rs` 신규 모듈 — `commands/mod.rs`에 `pub mod notice` 등록
- ⬜ IPC `list_notice_assets` — `assets/` 디렉토리 내 PNG/JPG 파일 목록 반환 (파일명, 크기, 수정일)
- ⬜ IPC `save_notice_asset` — base64 인코딩된 이미지 데이터를 `assets/{filename}` 에 저장 (동일 파일명 덮어쓰기 확인)
- ⬜ IPC `delete_notice_asset` — 배경서식 파일 삭제 (확인 다이얼로그는 프론트에서 처리)
- ⬜ `lib.rs` invoke_handler 등록
- ⬜ 단위 테스트 3건: 저장/목록/삭제 (tempdir 사용)

**INVEST**: Independent(T1 경로 헬퍼 완료 후), Negotiable(파일 vs DB), Valuable(배경서식 관리), Estimable, Small(3h), Testable(단위 테스트)

---

### T3: 레이아웃 설정 저장/로드 IPC (2.5h)

텍스트박스 위치/크기/폰트 속성을 JSON 형태로 저장/로드하는 IPC를 구현한다. AC-4.10-3("텍스트박스 레이아웃 보존")을 충족한다. 단일 사용자 앱이므로 `app_settings` 테이블의 JSON 값으로 저장한다.

- ⬜ IPC `save_notice_layout` — 레이아웃 JSON을 `app_settings` 테이블에 key='notice_layout'으로 저장
- ⬜ IPC `get_notice_layout` — 저장된 레이아웃 JSON 조회 (없으면 기본값 반환)
- ⬜ 레이아웃 타입 정의 (Rust struct):
  ```
  NoticeLayout {
    background_asset: Option<String>,    // 선택된 배경서식 파일명
    textboxes: Vec<TextboxConfig>,       // 3종 텍스트박스 설정
  }
  TextboxConfig {
    field_type: String,   // "bill_month" | "student_name" | "bill_amount"
    x: f64, y: f64,       // 위치 (px, 배경 이미지 기준)
    width: f64, height: f64,
    font_size: f64,
    font_weight: String,  // "normal" | "bold"
    font_color: String,   // hex color
    text_align: String,   // "left" | "center" | "right"
  }
  ```
- ⬜ 단위 테스트 2건: 저장/조회 + 기본값 반환

**INVEST**: Independent, Negotiable(저장 방식), Valuable(AC-4.10-3), Estimable, Small(2.5h), Testable(단위 테스트)

---

### T4: 공지문 이미지 저장 IPC (3h)

프론트엔드에서 HTML5 Canvas로 렌더링한 PNG 바이너리를 백엔드 IPC를 통해 파일 시스템에 저장한다. 저장 경로 규칙: `[설정폴더]/output/[YYYYMM]/[YYYYMM]_[원생이름].png` (PRD §4.10.2, §10.3 (11)).

- ⬜ IPC `save_notice_image` — (year_month, student_name, image_base64) → 파일 저장, 경로 반환
  - 파일명: `{YYYYMM}_{student_name}.png` (공백 → 언더스코어, 특수문자 제거)
  - 디렉토리 자동 생성
- ⬜ IPC `save_notice_images_batch` — Vec<(student_name, image_base64)> + year_month → 일괄 저장
  - 진행률 반환 (저장 완료 건수 / 총 건수)
- ⬜ IPC `check_notice_output_exists` — 해당 year_month 폴더에 기존 파일 존재 여부 확인 (AC-4.10-2 지원)
- ⬜ 단위 테스트 3건: 단건 저장, 일괄 저장, 기존 파일 확인

**INVEST**: Independent(T1 경로 완료 후), Negotiable, Valuable(핵심 저장 기능), Estimable, Small(3h), Testable(단위 테스트)

---

### T5: TypeScript IPC 래퍼 + 도메인 타입 (2h) · skill: frontend-design

`src/lib/tauri/index.ts`에 T2~T4의 IPC 래퍼를 추가하고, `src/types/notice.ts`에 도메인 타입을 정의한다.

- ⬜ IPC 래퍼 7종: `listNoticeAssets`, `saveNoticeAsset`, `deleteNoticeAsset`, `saveNoticeLayout`, `getNoticeLayout`, `saveNoticeImage`, `saveNoticeImagesBatch`, `checkNoticeOutputExists`
- ⬜ 도메인 타입: `NoticeAsset`, `NoticeLayout`, `TextboxConfig`, `SaveNoticeResult`
- ⬜ dev mode fallback (Tauri 미실행 시 mock 데이터)

**INVEST**: Independent(백엔드 완료 후), Negotiable, Valuable(프론트-백엔드 연결), Estimable, Small(2h), Testable(타입 검증 + tsc)

---

### T6: 공지문 편집 화면 UI (6h) · skill: frontend-design

PRD §4.10.1 공지문 편집 화면을 구현한다. `/notice` 라우트 신설.

- ⬜ `/notice` 라우트 + 사이드바 "공지문" 메뉴 활성화
- ⬜ **좌측 패널**: 청구 대상 원생 리스트
  - `list_bills` IPC로 해당 월 청구 데이터 조회 (confirmed 상태만)
  - 원생명, 주 수업시간, 청구액 표시
  - 전체 선택 / 개별 선택 체크박스
  - 년/월 선택 드롭다운 (`list_billed_months` 재사용)
- ⬜ **우측 패널**: 배경서식 미리보기 + 텍스트박스 3종 오버레이
  - 배경서식 이미지 로드 (`list_notice_assets` → 선택 → base64 표시)
  - 텍스트박스 3종: 청구월 / 원생이름 / 청구액
  - **드래그로 위치 조정**: `react-rnd` (resize + drag) — PI-14 확정
  - **크기 조절**: 핸들 드래그
  - **폰트 속성 변경**: 크기/굵기/색상/정렬 — 간단한 툴바
  - 실시간 미리보기: 원생 리스트에서 선택한 원생의 데이터로 텍스트박스 내용 채움
- ⬜ **배경서식 관리**: 업로드 버튼 (파일 선택 → `saveNoticeAsset`), 목록에서 선택/삭제
- ⬜ **레이아웃 저장/로드**: 위치/속성 변경 시 `saveNoticeLayout` 자동 호출 (debounce 500ms), 페이지 진입 시 `getNoticeLayout` 로드

**INVEST**: Independent, Negotiable(드래그 라이브러리), Valuable(핵심 편집 UX), Estimable, Small(6h — 복잡하지만 기존 UI 패턴 재사용), Testable(화면 렌더링 + 드래그 동작)

---

### T7: 일괄 이미지 생성 엔진 (5h) · skill: frontend-design

`html-to-image` 라이브러리를 사용하여 HTML/CSS 기반 공지문을 PNG로 변환하는 엔진을 구현한다. PRD §4.10.2, AC-4.10-1, AC-4.10-2.

- ⬜ `pnpm add html-to-image` — 신규 의존성 추가 (package.json)
- ⬜ `src/lib/notice-generator.ts` 신규 — 이미지 생성 코어 로직
  - 숨겨진 렌더링 컨테이너(offscreen div)에 배경서식 + 텍스트박스 3종을 배치
  - 원생별로 텍스트박스 내용만 교체 (청구월, 원생이름, 청구액)
  - `toPng()` 호출 → base64 PNG → `saveNoticeImage` IPC로 저장
  - 청구액 천단위 콤마 표기: `Intl.NumberFormat('ko-KR')` (AC-4.10-1)
  - 일괄 처리: 순차 실행 (Promise 체인) — UI 블로킹 방지를 위해 `requestAnimationFrame` 또는 `setTimeout(0)` 인터리브
- ⬜ **"발송용 공지문 생성" 버튼** — 선택된 원생 목록 기반 일괄 생성
  - 생성 전: `checkNoticeOutputExists` → 기존 파일 존재 시 덮어쓰기 확인 다이얼로그 (AC-4.10-2)
  - 생성 중: 진행률 표시 (X/N건 완료)
  - 생성 완료: 토스트 알림 + 저장 경로 안내
- ⬜ **성능 목표**: 50장 < 30초 (PRD §5.6)
  - 순차 처리 기본, 성능 미달 시 Web Worker 도입 검토

**INVEST**: Independent(T5/T6 완료 후), Negotiable(렌더링 방식), Valuable(핵심 기능), Estimable, Small(5h), Testable(50장 성능 실측)

---

### T8: Tauri capabilities 권한 추가 (0.5h)

공지문 기능에 필요한 Tauri 파일 시스템 권한을 `capabilities/default.json`에 추가한다.

- ⬜ `tauri-plugin-fs` 사용 여부 판단: 현재 파일 접근은 모두 Rust IPC 경유이므로 추가 플러그인 불필요할 가능성 높음. T2/T4 IPC가 `std::fs` 직접 사용이면 capabilities 변경 불필요
- ⬜ 배경서식 업로드 시 `dialog:allow-open` 이미 허용됨 — 파일 선택 다이얼로그 사용 가능
- ⬜ 필요 시 추가 권한만 최소 범위로 추가 (최소 권한 원칙)

**INVEST**: Independent, Negotiable, Valuable(보안 정합), Estimable, Small(0.5h), Testable(앱 실행 시 권한 오류 없음)

---

### T9: 통합 검증 + AC 전수 마킹 (3h)

- ⬜ `cargo test --lib --manifest-path src-tauri/Cargo.toml` — cipher off 전수 통과
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` — clean
- ⬜ `pnpm lint` — clean
- ⬜ `pnpm tsc --noEmit` — clean
- ⬜ `pnpm build` — static export 성공 (라우트 수 확인)
- ⬜ 공지문 편집 화면 수동 검증: 배경서식 업로드 → 텍스트박스 배치 → 위치/속성 저장 → 재진입 시 복원
- ⬜ 일괄 이미지 생성 수동 검증: 50명 미만 실 데이터 → PNG 파일 생성 → 이미지 품질 확인
- ⬜ 성능 실측: 50장 생성 시간 < 30초
- ⬜ AC 전수 마킹:
  - AC-4.10-1: 청구액 천단위 콤마 확인
  - AC-4.10-2: 동일 월 재생성 시 덮어쓰기 확인 다이얼로그 동작
  - AC-4.10-3: 레이아웃 보존 동작 (재진입 시 위치/속성 유지)

**INVEST**: Independent(T0~T8 완료 후), Negotiable, Valuable(품질 보증), Estimable, Small(3h), Testable(자동+수동 검증)

---

## 설계 결정 필요 사항

### PI-13: 이미지 생성 라이브러리 선택 (ADR 불필요 — 기존 결정 존재)

`frontend.md`와 ROADMAP에 **html-to-image** 가 명시되어 있다. 대안(`dom-to-image`, `html2canvas`, 직접 Canvas API)과 비교하지 않고 기존 결정을 따른다.

- `html-to-image` v1.11.13 (MIT) — DOM 요소를 SVG foreignObject → Canvas → PNG로 변환
- 장점: API 단순 (`toPng(node)`), 외부 렌더링 서버 불필요, CSS 스타일 충실 재현
- 제약: foreignObject 기반이라 복잡한 CSS (filter, clip-path) 일부 미지원 가능 — 공지문 용도에서는 문제 없음

### PI-14: 텍스트박스 드래그/리사이즈 구현 방식

**방안 A**: `react-rnd` (npm, 드래그+리사이즈 통합)
- 장점: 드래그+리사이즈 동시 지원, 경계 제한(bounds), 그리드 snap
- 단점: 추가 의존성 1개

**방안 B**: 자체 구현 (onMouseDown/onMouseMove)
- 장점: 의존성 없음
- 단점: 경계 처리, 리사이즈 핸들, 터치 지원 등 구현 비용

**✅ 확정 (2026-05-30 사용자 결정)**: 방안 A (`react-rnd`) — 드래그+리사이즈 통합, 경계 제한/그리드 snap 지원.

### PI-15: 배경서식 업로드 방식

파일 선택 다이얼로그(`dialog:allow-open`)로 이미지를 선택한 뒤, 프론트엔드에서 FileReader로 base64 변환 → IPC `save_notice_asset`으로 `smarthb/assets/`에 저장. Tauri dialog 플러그인이 이미 활성화되어 있으므로 추가 설정 불필요.

---

## Capacity 검증

| Task | 예상 소요 |
|------|----------|
| T0: carry-over 정리 | 4h |
| T1: 경로 헬퍼 | 2h |
| T2: 배경서식 IPC | 3h |
| T3: 레이아웃 IPC | 2.5h |
| T4: 이미지 저장 IPC | 3h |
| T5: TS 래퍼 + 타입 | 2h |
| T6: 편집 화면 UI | 6h |
| T7: 일괄 생성 엔진 | 5h |
| T8: capabilities | 0.5h |
| T9: 통합 검증 | 3h |
| **합계** | **31h** |
| 시각 검증 버퍼 (A50) | 6h |
| **총계** | **37h** |

Capacity 40h 이내 (93%). Sprint 11 실측 16.5h/계획 30.5h 패턴을 고려하면 충분한 여유가 있다. T6(편집 UI)와 T7(일괄 생성)이 핵심 복잡도이며, 기존 Sprint 10~11 UI 패턴 재사용으로 속도를 높일 수 있다.

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ 배경서식 업로드 → 선택 → 미리보기 동작
- ⬜ 텍스트박스 3종 드래그 위치 조정 + 크기 조절 + 폰트 속성 변경 동작
- ⬜ 위치/속성 저장 → 재진입 시 자동 로드 (AC-4.10-3)
- ⬜ 원생별 PNG 일괄 생성 → 파일 시스템 저장 동작
- ⬜ 저장 경로: `{data_root}/output/{YYYYMM}/{YYYYMM}_{원생이름}.png`
- ⬜ 청구액 천단위 콤마 표기 (AC-4.10-1)
- ⬜ 동일 월 재생성 시 덮어쓰기 확인 다이얼로그 (AC-4.10-2)
- ⬜ 50장 생성 < 30초 (PRD §5.6)
- ⬜ `cargo test --lib` 전수 통과
- ⬜ `cargo clippy -- -D warnings` clean
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 통과

**프로세스 (sprint-close 에이전트가 처리)**
- ⬜ ROADMAP.md Sprint 12 상태 업데이트
- ⬜ CHANGELOG.md 업데이트

---

## 예상 산출물

| 산출물 | 경로 |
|--------|------|
| 백엔드 모듈 | `src-tauri/src/commands/notice.rs` (신규) |
| 경로 헬퍼 | `src-tauri/src/commands/paths.rs` (수정) |
| 프론트엔드 라우트 | `src/app/notice/page.tsx` (신규) |
| 이미지 생성 엔진 | `src/lib/notice-generator.ts` (신규) |
| 도메인 타입 | `src/types/notice.ts` (신규) |
| IPC 래퍼 | `src/lib/tauri/index.ts` (수정) |
| 레이아웃 컴포넌트 | `src/components/notice/` (신규 디렉토리) |

## 의존성 및 리스크

| ID | 리스크 | 영향도 | 대응 |
|----|--------|--------|------|
| R88 | html-to-image의 foreignObject 방식이 배경서식 이미지 해상도를 왜곡할 가능성 | 중간 | T7 초반에 단일 이미지 PoC로 품질 확인. pixelRatio 옵션으로 해상도 조정 가능. 최악의 경우 html2canvas 대안 |
| R89 | 50장 순차 생성 시 30초 초과 가능성 | 중간 | toPng의 pixelRatio를 1x로 제한, 이미지 크기 최적화. 미달 시 Web Worker 병렬 또는 배치 크기 조절 |
| R90 | 텍스트박스 드래그 라이브러리(react-rnd) 번들 크기 영향 | 낮음 | tree-shaking 확인. 대안: 자체 구현(비용 증가) |
| R91 | 클라우드 동기화 폴더의 output/ 디렉토리에 대량 PNG 저장 시 동기화 지연 | 낮음 | 50명 x 100KB = ~5MB 수준으로 MYBOX 30GB 한도 대비 미미. 경고 불필요 |

## 신규 의존성 (package.json 추가 필요)

| 패키지 | 버전 | 용도 | 라이선스 |
|--------|------|------|---------|
| `html-to-image` | ^1.11.13 | DOM → PNG 변환 (공지문 이미지 생성 핵심) | MIT |
| `react-rnd` | ^10.x (최신) | 텍스트박스 드래그 + 리사이즈 (PI-14 ✅ 확정) | MIT |

## 참고 사항

- Sprint 12는 Phase 4 마지막 마일스톤. 완료 시 Phase 4(청구+수납+공지문) 전체 완료.
- ROADMAP Sprint 12의 "데이터 가져오기 기초 (§4.13.1)"는 범위 제외 — Sprint 15(Phase 6)로 이연.
- 배경서식 이미지 저장 위치는 `smarthb/assets/` (클라우드 동기화 폴더) — 양 PC 간 공유됨.
- 생성된 PNG 저장 위치는 `smarthb/output/{YYYYMM}/` (클라우드 동기화 폴더) — 양 PC 간 공유됨.
- 마감(closed) 개념은 post-Sprint 11에서 폐기됨. 공지문은 confirmed 상태의 청구 데이터만 대상.
- 청구 데이터 소스: 기존 `list_bills` IPC 재사용 (status='confirmed' 필터).
- `dialog:allow-open` 권한이 이미 활성화되어 있어 파일 선택 다이얼로그 추가 설정 불필요.
- PR 단계 생략 정책(단일 개발자) — develop 직접 머지.
