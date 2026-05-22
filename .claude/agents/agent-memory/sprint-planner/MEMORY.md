# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 6 (2026-05-22)
- 다음 스프린트 번호: 7
- Sprint 6 계획 수립: 2026-05-22 (Phase 2 학사 스케줄 — 3개월 캘린더 + 교습기간 + 일정 코드 + 배치)
- Sprint 5 계획 수립: 2026-05-22 (Phase 1.5b — Node 25 호환 + single-instance + 시드)
- Sprint 4 계획 수립: 2026-05-21 (Phase 1 스테이징 검증 14개 이슈 해소 + 교습소 설정)
- Sprint 3 계획 수립: 2026-05-21 (마법사 + R12/R13/R14 해소 + 원생 관리 프론트 + 앱 셸)
- Sprint 2 계획 수립: 2026-05-20 (sprint1 잔여 + 기반 도메인 백엔드 통합)

## ROADMAP 번호 이동 (Sprint 4, 5 삽입)

Sprint 4가 "Phase 1.5 품질 안정화"로, Sprint 5가 "Phase 1.5b 추가 안정화"로 삽입됨에 따라:
- 원래 Sprint 4 (학사 스케줄) -> Sprint 6
- 원래 Sprint 5 (출결 관리) -> Sprint 7
- Phase 3 이후 번호는 sprint-close 시 일괄 정리 필요

## 프로젝트 기본 정보

- **기술 스택**: Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- **Phase 구조**: 7 Phase + 1.5(안정화) + 1.5b(추가 안정화). Phase 1 완료, Phase 1.5 완료, Phase 1.5b 진행 중
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)

## Sprint 4 완료 결과 (2026-05-21)

- **마이그레이션 V201 불필요 확정**: withdraw_date 컬럼 V101 부터 존재 — 사전 검토에서 발견. 사용자 보고 이슈 #7은 백엔드 OK, 프론트 입력/표시만 누락이었음
- **신규 의존성 (허가 완료, 모두 사용 중)**: @base-ui/react + class-variance-authority + clsx + lucide-react + tailwind-merge + tw-animate-css (shadcn init 부산물) + @dnd-kit/core + @dnd-kit/sortable + @dnd-kit/utilities
- **shadcn 트랩 (R23)**: init 이 globals.css CSS 변수를 안 넣어 AlertDialog 가 투명 렌더링. 13개 토큰(popover/muted/primary/secondary/destructive/input/ring/card/radius/foreground 변형) 을 globals.css `:root` + `@theme` 에 수동 추가하여 해소
- **shadcn add 누락**: `src/lib/utils.ts` 의 cn 헬퍼를 init/add 가 생성 안 함 → 수동 작성 필요 (clsx + tailwind-merge)
- **R24 검증 통과**: @dnd-kit × React 19 peer dep 경고 없음
- **post-T11 4건 추가 fix**: 사용자 시각 검증에서 발견 — 스케줄 폼 위치/시간 단위, 운영시간 디폴트 20:00, 컬럼 헤더 정렬 + SerialAsc 디폴트, 원생 목록 주총/요일 컬럼 (correlated subquery)
- **알려진 flaky (Sprint 5 carry-over)**: `paths::tests::init_from_config_ignores_empty_path` — 병렬 실행 시 OnceLock 격리 부족. `--test-threads=1` 직렬 OK

## Sprint 5 계획 요약 (Phase 1.5b, 2026-05-22)

Sprint 5는 Phase 2 진입 전 최종 안정화로 재정의됨:
- **T0**: cross-env + --no-experimental-webstorage (Node 25 호환) + CVE-2025-66478 영향 분석
- **T1**: tauri-plugin-single-instance 도입 (동일 PC 다중 인스턴스 차단)
- **T1-sub**: 양 PC 간 강제 점유 버튼 동작 검증
- **T2**: 마법사 완료 redirect `/` -> `/settings`
- **T3+T4**: 표준교습비(4종, 16~26만원) + 결제수단(5종, 현금 비활성) 시드 변경 (V201)
- **마이그레이션 V201 사용**: Sprint 4에서 미사용 확인 (withdraw_date 기존 컬럼 활용)

## Sprint 6 계획 요약 (Phase 2 학사 스케줄, 2026-05-22)

- **Task 12개**: T1(A20 lock 재시도) + T2(V301 시드 보정 + 공휴일) + T3(A21 flaky 테스트) + T4(A22 DnD) + T5~T7(교습기간/일정코드/배치 IPC) + T8(래퍼/타입) + T9~T11(캘린더/교습기간UI/배치UI) + T12(통합검증)
- **마이그레이션 V301**: schedule_codes 시드 3속성 보정 (보강데이/공휴수업일/단원평가)
- **기존 테이블 활용**: study_periods(V102), schedule_codes(V102), schedule_events(V103) -- 스키마 변경 없음
- **신규 의존성 없음**: 3개월 캘린더는 shadcn/ui Calendar 기반 커스텀
- **주요 리스크**: R30(캘린더 복잡도), R32(OnceLock 리팩토링 부작용)
- **A17(salt.bin 이전)**: 이번에도 범위 외 -- 별도 hotfix 권고

## Sprint 7 진입 시 카드

- **출결 관리** (Phase 2 나머지): 출결 생성 + 출결표 UI + 상태 토글 + 캘린더 ADR
- **회고 carry-over (Sprint 3 A6/A12)**: cipher on 환경 실측 (인스톨러 배포 후)

## 미결정 항목 (PI)

- PI-05 (Medium): 일련번호 자동 채번 — **확정 (2026-05-20)**: `MAX+1` + `BEGIN IMMEDIATE` + override 허용
- PI-07 (High): 복구 코드 — **결정 완료** (PRD v1.5.1). Argon2id 해시, Sprint 1 구현

## Sprint Planner 사전 검토 체크리스트 (A4 반영)

- 마이그레이션 대상 컬럼 타입은 data-model.md 기준으로 명시할 것
- 외부 라이브러리/매크로 사용 Task는 현재 코드 사용 현황 확인 후 포함할 것
- 이연 Task는 기술적 실현 가능성 사전 검증 후 계획에 포함할 것
- 스테이징 검증 이슈가 있으면 다음 sprint에 반드시 전수 포함 (임의 누락 금지)

## 기술 스택 및 프로젝트 특이사항

- Cargo.toml: sqlx 0.8 + tauri 2 + thiserror 2 + keyring, argon2, zeroize, rusqlite, fs2, uuid
- `cargo` 명령에 `--manifest-path src-tauri/Cargo.toml` 필수 (루트에 Cargo.toml 없음)
- DB 개발: `./SmartHB-dev.db` (루트, SQLCipher 미적용 가능). 프로덕션: 클라우드 동기화 폴더 하위
- PR 단계 생략 정책: 단일 개발자, `gh pr create` 금지
