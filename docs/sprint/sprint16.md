# Sprint Plan sprint16

## 기간
2026-06-09 ~ 2026-06-22 (2주)

## 목표
Sprint 15에서 이연된 양 OS 빌드 검증 + 양 PC 동기화 시나리오를 완수하고, CSV 가져오기로 원장 실데이터를 이관하며, DB 폴더 변경(경로 재지정 + salt.bin 이전)을 구현한다. 격식 UAT 없이 원장이 바로 실사용을 개시하고, 초기 피드백에 대응한다. v1.0 릴리즈 준비(CHANGELOG, 인스톨러 최종 확인)를 완료하되 실제 배포(태그 push)는 사용자 명시 지시까지 대기한다.

## ROADMAP 연계 기능
- Phase 6 마지막 스프린트: 실사용 개시 + 초기 피드백 반영 + v1.0 릴리즈 준비
- Sprint 15 이연: T7(양 OS 빌드 검증), T8(양 PC 동기화 시나리오), T9(통합검증 빌드부)
- Sprint 15 회고 액션: A98(즉시 적용 완료), A99(Ctrl+N 입력 방어), A100(미저장 이탈 경고), A96(복원 리허설 Low)
- Sprint 15 코드 리뷰: R105(교습소 정보 미저장 이탈 경고 — F3 Medium)
- 확정 포함: DB 폴더 변경 + salt.bin 이전 (PI-16 사용자 결정 2026-06-08)
- 이연 기능: CSV 가져오기(PRD SS4.13.1), 공지문 I/O 병렬화, 접근성 잔여(44px/gray-500/F1/Ctrl+S), A89 notices UI 구획화
- 기술 부채: 출결표 N+1 재설계, 셀 memo, makeup_attendances 인덱스(실측 후)

---

## Capacity 분석

### Velocity 참조 (과거 실적)

| Sprint | 계획(h) | 특성 | 비고 |
|--------|---------|------|------|
| 13 | 38h | PIN 옵션화 + carry-over | 소형, 검수 중 추가 2건 |
| 14 | 38h | 대시보드+자가진단+내보내기 | 검증-phase에서 기능 대폭 추가 |
| 15 | 38h | 안정화 + 접근성 감사 | T7~T9 이연, 38h 내 수용 완료 |

**패턴 분석**:
- 38h 계획이 3스프린트 연속 수용됨.
- DB 폴더 변경(PI-16 확정)이 +8h MUST로 추가되어 총 Capacity가 타이트하다. SHOULD/COULD 작업을 Post-MVP로 이연하여 MUST에 집중한다.
- 격식 UAT 제거(PI-18 확정)로 T4/T5가 축소되어 일부 여유 확보.

### Capacity 산정

| 항목 | 값 |
|------|-----|
| 팀 인원 | 1인 (AI 페어 프로그래밍) |
| 스프린트 일수 | 10일 |
| 실작업 시간/일 | 4시간 |
| 총 가용 시간 | 40시간 |

### 작업 소요 예상

| Task | 예상 소요 | 우선순위 | 비고 |
|------|----------|---------|------|
| T0 회고 액션 + carry-over | 3h | MUST | A99/A100/R105 통합 |
| T1 CSV 가져오기 | 6h | MUST | 실사용 데이터 이관 전제조건 |
| T2 DB 폴더 변경 + salt.bin 이전 | 8h | MUST | PI-16 확정. copy-then-switch + salt.bin/app.lock/백업 동반 |
| T3 양 OS 빌드 검증 | 4h | MUST | Sprint 15 이연 T7 |
| T4 양 PC 동기화 시나리오 | 3h | MUST | Sprint 15 이연 T8 |
| T5 실사용 개시 준비 | 2h | MUST | 양 OS 설치 + 데이터 이관 확인 + 기동 검증 |
| T6 초기 실사용 피드백 대응 (버퍼) | 4h | MUST | Critical/High 피드백 즉시 수정 |
| T7 접근성 잔여 개선 | 4h | SHOULD | 밀집 UI 44px, F1, Ctrl+S |
| T8 공지문 I/O 병렬화 | 3h | SHOULD | 50장 일괄 생성 성능 |
| T9 v1.0 릴리즈 준비 | 3h | MUST | CHANGELOG + 인스톨러 최종 확인 |
| T10 통합 검증 | 3h | MUST | cargo test + clippy + lint + build |
| **합계** | **43h** | | 가용 40h + 여유 3h 초과 — SHOULD 이연으로 조정 |

> **MUST 합계**: 36h (T0+T1+T2+T3+T4+T5+T6+T9+T10)
> **SHOULD 합계**: 7h (T7+T8) — Capacity 초과분. MUST 완료 후 여유 시 착수, 미착수 시 Post-MVP 이연
> A89 notices UI 구획화(구 T8 COULD 2h)는 Post-MVP 이연 확정.
> MUST 36h는 가용 40h 이내이나 피드백 버퍼(T6 4h)가 가변적이므로 사실상 타이트. T2(DB 폴더 변경)가 8h 초과 시 SHOULD 전량 이연.

---

## 이전 회고 반영

출처: `docs/sprint-retrospectives/sprint15-retrospective.md`

| 액션 ID | 항목 | 이번 스프린트 반영 |
|---------|------|-------------------|
| A98 | self-verify `--all-targets` 추가 | **즉시 적용 완료** (Sprint 15 close 시 CLAUDE.md/harness-engineering.md 교정됨) |
| A99 | Ctrl+N 입력 필드 포커스 방어 로직 | T0에서 처리 — `GlobalShortcuts` INPUT/TEXTAREA/SELECT 가드 추가 |
| A100 | 미저장 이탈 경고 다이얼로그 공통 구현 | T0에서 처리 — `useUnsavedChanges` 훅 + 교습소 정보 화면 적용 (R105 해소) |
| A96 | 복원 리허설 dev 환경 개선 | Low — Capacity 여유 시 검토, 미포함 |

---

## 리스크 레지스터 반영

출처: `docs/risk-register/2026-06-07.md`

| ID | 설명 | 반영 방법 |
|----|------|-----------|
| R101 | Windows PC 물리 접근 제한 | T2(양 OS 빌드 검증)에서 GitHub Actions CI + 교습소 방문 설치 |
| R105 | 교습소 정보 미저장 이탈 경고 | T0(A100)에서 `useUnsavedChanges` 공통 훅 구현으로 해소 |

---

## 작업 목록

### T0: 회고 액션 + 코드 리뷰 carry-over (3h) — MUST

- ⬜ A99: `GlobalShortcuts` Ctrl+N 입력 필드 방어 — `e.target` tagName이 INPUT/TEXTAREA/SELECT이면 Ctrl+N 억제
- ⬜ A100 + R105: 미저장 이탈 경고 공통 훅 `useUnsavedChanges` 구현 — `beforeunload` + Next.js `routeChangeStart` 가드. `/settings/info`(교습소 정보) 적용
- ⬜ Ctrl+S 전역 저장 단축키 등록 — `GlobalShortcuts`에 추가, 현재 활성 폼의 저장 함수 실행

### T1: CSV 가져오기 (6h) — MUST

> PRD SS4.13.1 — 실사용 개시의 첫 번째 작업. 원생 실데이터 이관용.

**백엔드**
- ⬜ `import.rs` 신규 모듈 — `import_students_csv` IPC
  - CSV 파싱(BOM 처리 + EUC-KR/UTF-8 자동 감지)
  - 필수 컬럼: 이름, 학교명, 학년, 연락처 (선택: 일련번호, 입교일, 성별, 수업요일, 생년월일)
  - 중복 검사: 이름+연락처 동일 시 skip/overwrite 옵션
  - 마이그레이션 불필요 (기존 students/student_schedules 테이블 활용)
  - 단위 테스트: 정상 임포트, 중복 skip, 필수 컬럼 누락 에러, EUC-KR 처리

**프론트엔드**
- ⬜ TypeScript IPC 래퍼 + `src/types/import.ts` 타입
- ⬜ `/settings/import` 라우트 — 파일 선택(Tauri Dialog) + 미리보기 테이블 + 컬럼 매핑 + 임포트 실행 + 결과 요약

### T2: DB 폴더 변경 + salt.bin 이전 (8h) — MUST · PI-16 확정 (2026-06-08)

> 클라우드 동기화 경로 재지정 UI/IPC + salt.bin 동반 이전. R12 salt.bin 이전의 최종 해소.
> 참조 메모리: `keyring-v3-features-trap`, `ntfs-power-loss-pattern`, `sqlite-migration-fk-rebuild`

**설계 검토 (ADR 권장)**
- ⬜ ADR 작성 — DB 폴더 변경 전략 결정: copy-then-switch vs move-then-update
  - copy-then-switch 권장: 원본 보존, 실패 시 원래 경로 즉시 복귀
  - 중간 실패(복사 중 강제 종료) 시 복구 전략 명시

**백엔드**
- ⬜ `paths.rs` 확장 — `change_data_folder` IPC 신규
  - 단계 1: 대상 폴더 유효성 검증 (쓰기 권한, 디스크 여유)
  - 단계 2: DB 파일(`app.db`) 복사 (`ntfs-power-loss-pattern` 적용 — fsync 호출)
  - 단계 3: salt.bin 복사 (손상 감지 `is_corrupted()` 적용)
  - 단계 4: 백업 폴더(`backup/`) 복사 (4계층 전체)
  - 단계 5: app.lock 해제 + 신규 경로에 app.lock 재생성
  - 단계 6: config.json `cloud_folder` 경로 업데이트
  - 단계 7: 원본 폴더에 이전 완료 마커 파일 생성 (역방향 참조)
  - 실패 시: 원래 config.json 복원 (copy-then-switch이므로 원본 무손상)
- ⬜ WAL 파일 처리 — `PRAGMA wal_checkpoint(TRUNCATE)` 실행 후 복사 (WAL/SHM 잔여 방지)
- ⬜ cipher ON 검증 — 암호화 DB 복사 후 정상 열기 확인 (키는 Keychain에서 동일 키 사용)
- ⬜ 단위 테스트: 정상 이전, 원본 보존 확인, 잘못된 경로 거부, 권한 없는 폴더 거부

**프론트엔드**
- ⬜ `/settings` 허브 — 'DB 폴더 변경' 카드 활성화 (현재 disabled 상태 → 활성)
- ⬜ `ChangeFolderDialog` — 폴더 선택(Tauri Dialog) + 진행률 표시 + 완료/실패 알림
- ⬜ 변경 완료 후 앱 재시작 안내 (Tauri `process::relaunch` 사용)
- ⬜ TypeScript IPC 래퍼 추가

**정합성 검증 항목**
- ⬜ salt.bin 이전 후 PIN 잠금해제 정상 동작 (`keyring-v3-features-trap` — Keychain 키 유지 확인)
- ⬜ 이전 후 백업 4계층 정상 동작 (경로 참조 갱신)
- ⬜ 양 PC 시나리오: 한 PC에서 폴더 변경 → 다른 PC에서 새 경로 인식 (config.json 동기화)

### T3: 양 OS 빌드 검증 (4h) — MUST · Sprint 15 이연 T7

- ⬜ macOS: `pnpm tauri:build` → `.dmg` 생성, 설치/실행/삭제, Apple Silicon arch 확인
- ⬜ Windows: GitHub Actions CI matrix(windows-latest) 빌드 확인, `.msi`/`.exe` 설치/실행/언인스톨
- ⬜ 인스톨러 체크리스트: 앱 아이콘, 시작 메뉴/Dock 등록, 기존 데이터 유지(업그레이드), 언인스톨 후 잔여 파일 없음
- ⬜ WebView2 런타임 확인(Windows), Xcode CLI 확인(macOS)

### T4: 양 PC 동기화 시나리오 테스트 (3h) — MUST · Sprint 15 이연 T8

- ⬜ 시나리오 1: Windows → Mac 전환 — Windows 앱 정상 종료 → 클라우드 동기화 대기 → Mac 앱 시작 → 데이터 정합 확인
- ⬜ 시나리오 2: Mac → Windows 역방향 — 동일 절차 역방향
- ⬜ 시나리오 3: 비정상 종료 후 5분 임계 강제 점유 — Windows 강제 종료 → Mac에서 5분 경과 후 강제 점유 → 데이터 정합 확인
- ⬜ 검증 항목: app.lock 해제/점유, DB 정합(원생/출결/청구), salt.bin 동기화, 백업 파일 동기화
- ⬜ T2(DB 폴더 변경) 후 양 PC 경로 인식 정합 확인

### T5: 실사용 개시 준비 (2h) — MUST

> 격식 UAT 없이 원장이 바로 v1.0을 실사용 개시한다. T3(양 OS 빌드) + T4(양 PC 동기화) 통과 후 실행.

- ⬜ 교습소 PC(Windows)에 `.msi` 인스톨러 설치 확인
- ⬜ 자택 Mac에 `.dmg` 인스톨러 설치 확인
- ⬜ T1(CSV 가져오기)으로 원생 실데이터 이관 완료 확인
- ⬜ 클라우드 동기화 폴더(MYBOX) 정상 동작 확인
- ⬜ 양 OS 앱 기동 + PIN 잠금해제 + 대시보드 진입 확인

### T6: 초기 실사용 피드백 대응 버퍼 (4h) — MUST

> 실사용 개시 후 수집되는 피드백을 우선순위별로 대응한다. 2주 고정 기간 없이 지속적으로 수집.
> 피드백 분류 기준:
> - **Critical**: 기능 오류 (앱 크래시, 데이터 손실, 저장 실패) → 즉시 수정
> - **High**: UX 장애 (글씨 안 보임, 버튼 못 누름, 동선 혼란) → Sprint 내 수정
> - **Medium**: 미세 조정 (색상, 간격, 문구 변경) → Capacity 내 수정 또는 Post-MVP
> - **Low**: 희망 사항 → Post-MVP backlog 기록

- ⬜ 실사용 중 발견된 Critical/High 피드백 즉시 수정
- ⬜ 피드백 기록 + 분류 (화면별 사용성, 글씨 크기, 동선)

### T7: 접근성 잔여 개선 (4h) — SHOULD

- ⬜ 밀집 UI 클릭 영역 44x44px 미달 항목 수정 — 사이드바 메뉴, 테이블 셀 버튼, 필터 드롭다운 등
- ⬜ `text-gray-500` 잔여 항목 → WCAG AA 대비 수정 (Sprint 15 T3에서 gray-400→600 수정 완료, gray-500 잔여 점검)
- ⬜ F1 도움말 단축키 구현 — 현재 화면 컨텍스트에 따른 도움말 다이얼로그 또는 가이드 표시
- ⬜ A99 Ctrl+N 방어 로직 추가 시 함께 입력 필드 방어 통합 검증

### T8: 공지문 I/O 병렬화 (3h) — SHOULD

> 50장 일괄 생성 시 I/O 병목 개선. `ntfs-power-loss-pattern` 메모리 참조.

- ⬜ `notice-generator.ts` 일괄 생성 엔진에 `Promise.allSettled` 기반 병렬 저장 (동시성 4~8개 제한)
- ⬜ 개별 저장 실패 시 부분 성공 보고 (전체 실패 아닌 건별 결과)
- ⬜ NTFS power-loss 대응: `fs::write` 후 `fsync` 호출 검토 (Tauri fs 플러그인 제약 확인)

### T9: v1.0 릴리즈 준비 (3h) — MUST

> 실제 배포(태그 push)는 사용자 명시 지시까지 대기. 준비만 완료.

- ⬜ `CHANGELOG.md` v1.0.0 릴리즈 노트 작성 — Sprint 1~16 전체 기능 요약
- ⬜ `package.json` + `src-tauri/Cargo.toml` 버전 → `1.0.0` 업데이트
- ⬜ `README.md` 프로덕션 정보 갱신 (스크린샷, 설치 방법, 시스템 요구사항)
- ⬜ GitHub Actions `deploy.yml` 최종 확인 — `v*` 태그 push 시 양 OS 인스톨러 빌드 정상 동작 확인
- ⬜ 배포 대기 상태 문서화 — `DEPLOY.md` 체크리스트 준비, 사용자 지시 대기 명시

### T10: 통합 검증 (3h) — MUST

- ⬜ `cargo test --manifest-path src-tauri/Cargo.toml` 전체 통과
- ⬜ `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` clean
- ⬜ `cargo check --manifest-path src-tauri/Cargo.toml --features cipher` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ⬜ 마이그레이션 self-check: 신규 마이그레이션 여부 확인 + `.sqlx/` 캐시 갱신
- ⬜ develop HEAD에 Sprint 16 전체 변경 반영 확인

---

## 실사용 개시 전 검증 체크리스트

실사용 개시(T5) 전에 아래 핵심 흐름이 양 OS에서 정상 동작하는지 확인한다.

### 핵심 흐름 검증 (T3 빌드 + T4 동기화 시 수행)
- ⬜ 앱 시작 → PIN 잠금해제 → 대시보드 진입 (양 OS)
- ⬜ 원생 등록/수정/조회 전체 흐름
- ⬜ 출결 생성 + 출결표 + 토글
- ⬜ 청구 생성 + 확정 + 수납
- ⬜ 공지문 이미지 일괄 생성
- ⬜ CSV 가져오기 (이관 데이터 정합)
- ⬜ DB 폴더 변경 + 변경 후 정상 기동
- ⬜ 양 PC 전환 데이터 정합 (Win→Mac, Mac→Win)

---

## 확정 사항 (사용자 결정 완료)

| # | 항목 | 결정 | 결정일 |
|---|------|------|--------|
| PI-16 | DB 폴더 변경(경로 재지정) | **Sprint 16 포함 (MUST)** — T2로 배정. salt.bin 이전 + copy-then-switch 구현 | 2026-06-08 |
| PI-18 | UAT 방식 | **격식 2주 파일럿 폐기** — 원장이 바로 실사용 개시, 피드백은 실사용 중 수집 | 2026-06-08 |

## 미결정 항목 (Pending Items)

사용자 결정이 필요한 항목입니다. 스프린트 진행 중 확인 요청합니다.

| # | 항목 | 필요 시점 | 옵션 | 기본값 |
|---|------|----------|------|--------|
| PI-17 | 출결표 N+1 쿼리 재설계 실행 여부 | 실사용 피드백 후 | A: Sprint 16 T6 버퍼에서 수행 / B: Post-MVP | **B: Post-MVP** — 현재 PRD 성능 기준(50명x31일 < 1초) 충족. 실데이터에서 성능 저하 확인 시 A로 전환 |
| PI-19 | 셀 memo (출결표 셀별 메모) | 실사용 피드백 | A: Sprint 16 T6에서 구현 / B: Post-MVP | **B: Post-MVP** — 명시적 요청 시 A로 전환 |

---

## 이연 확정 항목 (Post-MVP backlog)

아래 항목은 Sprint 16 범위에서 제외하며 Post-MVP로 이연한다.

| 항목 | 사유 |
|------|------|
| 출결표 N+1 재설계 | 현재 성능 기준 충족. PI-17으로 실측 후 재판단 |
| makeup_attendances 인덱스 | 출결표 N+1과 연계. 실측 데이터 필요 |
| 셀 memo | 실사용 피드백 없으면 불필요. PI-19 |
| A89 notices UI 구획화 | 로직 분리 완료, UI 3분할만 잔여. Capacity 부족으로 이연 |
| E2E 자동화(UC-1~UC-5) | Tauri WebDriver 별도 인프라 세팅 8~12h |
| 한글 자모 부분 일치 검색 | hangul-js 라이브러리 또는 직접 분해 알고리즘. Nice-to-have |
| 반응형 폰트/셀 너비 | clamp() viewport 패턴. 현재 18px 고정으로 충분 |
| A96 복원 리허설 dev 환경 개선 | Low 우선순위 |
| query!() 매크로 전환 | 동적 query+bind 패턴 유지 중. 별도 backlog |

---

## 완료 기준 (Definition of Done)

**필수**
- ⬜ 양 OS 인스톨러(.dmg / .msi) 설치/실행/삭제 정상
- ⬜ 양 PC 동기화 시나리오 최소 2종 통과 (Win→Mac, Mac→Win)
- ⬜ CSV 가져오기로 원생 실데이터 이관 성공
- ⬜ DB 폴더 변경(경로 재지정) 정상 동작 — copy-then-switch + salt.bin/백업 동반 이전
- ⬜ DB 폴더 변경 후 양 PC 경로 인식 정합 확인
- ⬜ 실사용 개시 완료 (양 OS 기동 + 데이터 이관 + 핵심 흐름 확인)
- ⬜ 초기 실사용 피드백 Critical/High 전수 반영
- ⬜ `cargo test` 전체 통과 (예상 385+ tests)
- ⬜ `cargo clippy --all-targets -- -D warnings` clean
- ⬜ `cargo check --features cipher` 통과
- ⬜ `pnpm lint` + `pnpm tsc --noEmit` + `pnpm build` 전수 통과
- ⬜ CHANGELOG.md v1.0.0 작성 완료
- ⬜ 버전 번호 1.0.0 업데이트 (package.json + Cargo.toml)

**배포 대기 (사용자 명시 지시 후)**
- ⬜ v1.0.0 태그 push → GitHub Actions 인스톨러 빌드
- ⬜ GitHub Release 생성 + 양 OS 인스톨러 첨부
- ⬜ 배포 후 CV 체크리스트 통과

**프로세스**
- ⬜ ROADMAP.md Sprint 16 상태 업데이트
- ⬜ sprint-close 에이전트: 문서화 + develop 머지
- ⬜ sprint-review 에이전트: 코드 리뷰 + 회고

---

## 참고 사항

### 의존성
- T1(CSV 가져오기) → T5(실사용 개시): CSV 임포트 완료 후 데이터 이관 확인
- T2(DB 폴더 변경) → T4(양 PC 동기화): 폴더 변경 후 양 PC 경로 인식 검증
- T3(양 OS 빌드) → T5(실사용 개시): 인스톨러 확보 후 교습소 PC 설치
- T4(양 PC 동기화) → T5(실사용 개시): 동기화 검증 통과 후 실사용 개시
- T5(실사용 개시) → T6(피드백 대응): 실사용 시작 후 피드백 발생

### 작업 순서 (권장)
1. T0(회고 액션) + T1(CSV 가져오기) — 병렬 착수
2. T2(DB 폴더 변경) — 핵심 MUST, 단독 집중 (8h)
3. T3(양 OS 빌드)
4. T4(양 PC 동기화) — T2 완료 후 폴더 변경 시나리오 포함
5. T5(실사용 개시 준비) + T7(접근성, SHOULD) — 여유 시 병렬
6. T6(피드백 대응) — 실사용 개시 후 지속
7. T8(공지문 I/O, SHOULD) — T6 여유 시
8. T9(릴리즈 준비) + T10(통합 검증) — 스프린트 마지막

### 기술 고려사항
- CSV 가져오기 인코딩: 한국어 엑셀 기본 CSV는 EUC-KR. `encoding_rs` crate으로 자동 감지 필요 (신규 의존성)
- DB 폴더 변경: copy-then-switch 전략. WAL checkpoint 후 복사 필수. `ntfs-power-loss-pattern` 적용 (fsync). cipher ON 환경에서 복사 후 정상 열기 검증 필수
- DB 폴더 변경 양 PC 정합: config.json이 클라우드 동기화 대상이므로, 한 PC에서 경로 변경 시 다른 PC도 새 경로 인식 필요. config.json 저장 위치(app_config_dir = PC별 로컬)와 cloud_folder 경로의 분리 확인 필수
- salt.bin 이전: Keychain 키는 유지 (`keyring-v3-features-trap` — features 명시 확인). salt.bin 손상 감지 `is_corrupted()` 적용
- 공지문 I/O 병렬화: `ntfs-power-loss-pattern` 메모리 참조 — atomic write 시 `fsync` 호출 검토
- `cipher` feature: 프로덕션 빌드에서만 활성화. 인스톨러는 `--features cipher`로 빌드
- 배포(deploy-prod): 사용자 명시 지시 전까지 절대 진행하지 않음

### 신규 의존성 (예상)
- `encoding_rs` (Rust) — CSV EUC-KR 자동 감지 (T1에서 필요 시)
- `csv` (Rust) — CSV 파싱 (이미 sqlx 의존성 트리에 포함 가능성 확인 필요)
