# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

- [Sprint 현황](sprint-status.md) -- Sprint 13 계획 수립 완료, PIN 인증 옵션화 + Phase 5 취소 반영
- [마이그레이션 번호 정책](migration-numbering.md) -- V111 완료 (Sprint 13은 DB 마이그레이션 없음 -- config.json 활용)
- [Capacity 패턴](capacity-pattern.md) -- Sprint 9(38h계획/52h실측, +37%) → Sprint 10(실측 22.5h) → Sprint 11(계획 30.5h, 실측 16.5h)
- [Velocity 참조](velocity-reference.md) -- Sprint 11 실측 16.5h, Capacity 34h 기준
- [CSV 가져오기 이연](csv-import-defer.md) -- ROADMAP Sprint 12에 포함돼있던 CSV 가져오기를 Sprint 14(구 15)로 이연
- [Phase 5 취소 + 번호 재매핑](phase5-cancellation.md) -- Sprint 13에서 Phase 5 제거, Phase 6→5, Phase 7→6, Sprint 15→14, 16→15, 17→16

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 13 (2026-06-02)
- 다음 스프린트 번호: 14
- Sprint 13 계획 수립: 2026-06-02 (PIN 인증 옵션화 C안 + Phase 5 취소 반영 + carry-over 4건, 38h 예상)
- Sprint 12 계획 수립: 2026-05-30 (Phase 4 공지문 이미지 -- T0~T9 10개 Task, 31h+6h 예상, PI-14 react-rnd 사용자 결정 대기)
- Sprint 11 계획 수립: 2026-05-28 (Phase 4 청구+수납 -- T0~T9 10개 Task, 30.5h 예상, PI-10~12 사용자 결정 대기)
- Sprint 10 계획 수립: 2026-05-26 (Phase 3 소멸 자동 전이 + 퇴교 보강 + 선행 수업 + 캘린더 뷰 -- T1~T12 12개 Task, 44h 예상)
- Sprint 9 계획 수립: 2026-05-24 (Phase 3 보강 등록 + 매칭 -- T1~T9 9개 Task, 38h 예상, PI-02 사용자 결정 대기)
- Sprint 8 계획 수립: 2026-05-23 (Phase 2 출결 관리 + Sprint 7 carry-over 흡수)
- Sprint 7 계획 수립: 2026-05-22 (Phase 2 carry-over 해소)
- Sprint 6 계획 수립: 2026-05-22 (Phase 2 학사 스케줄)
- Sprint 5 계획 수립: 2026-05-22 (Phase 1.5b)
- Sprint 4 계획 수립: 2026-05-21 (Phase 1 스테이징 검증)
- Sprint 3 계획 수립: 2026-05-21 (마법사 + 원생 관리)
- Sprint 2 계획 수립: 2026-05-20 (기반 도메인 백엔드)

## Sprint 번호 시프트 이력

### 1차 (Sprint 4~5 삽입, Phase 1.5/1.5b)
Sprint 4가 "Phase 1.5 품질 안정화"로, Sprint 5가 "Phase 1.5b 추가 안정화"로 삽입:
- 원래 Sprint 4 (학사 스케줄) -> Sprint 6
- 원래 Sprint 5 (출결 관리) -> Sprint 7

### 2차 (Sprint 7 carry-over 삽입, 2026-05-22)
Sprint 6 시각 검증 carry-over 8건 해소를 위해 Sprint 7이 carry-over 전담으로 삽입:
- 원래 Sprint 7 (출결 관리) -> Sprint 8
- Phase 3: Sprint 8~9 -> Sprint 9~10
- Phase 4: Sprint 10~11 -> Sprint 11~12
- Phase 5: Sprint 12~13 -> Sprint 13~14
- Phase 6: Sprint 14 -> Sprint 15
- Phase 7: Sprint 15~16 -> Sprint 16~17
- 총 스프린트 수: 14 -> 17 (안정화 3개 삽입: Sprint 4, 5, 7)

### 3차 (Phase 5 전면 취소, 2026-06-02)
원장 결정으로 Phase 5(단원평가+학습보고서) 전면 취소:
- Sprint 13: 기존 단원평가 -> PIN 인증 옵션화 + Phase 5 취소 반영 + carry-over
- Sprint 14: 기존 학습보고서 -> Phase 5(구 Phase 6) 대시보드+유틸
- Sprint 15: 기존 Phase 6(Sprint 15) -> Phase 6(구 Phase 7) 양 OS 빌드+최적화
- Sprint 16: 기존 Phase 7(Sprint 17) -> Phase 6 UAT+v1.0 릴리즈
- Phase 6(구 Phase 6) -> Phase 5, Phase 7(구 Phase 7) -> Phase 6
- 총 스프린트 수: 17 -> 16 (Sprint 14 폐기, 번호 재매핑)

## 프로젝트 기본 정보

- **기술 스택**: Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- **Phase 구조**: 6 Phase (Phase 5 단원평가 취소). Phase 1~4 완료, Sprint 13 진행 중
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)

## Sprint 13 주요 설계 결정 사항

- PIN 인증 옵션화: C안(키체인 자동 스킵) 채택 (사용자 메모리 sprint13-pin-optional.md)
- 토글 저장: config.json (DB 밖, app_config_dir, PC별)
- ADR-008 필수: PRD SS5.5 완화 결정 기록
- DB 마이그레이션: 없음 (V111 최신 유지)
- 신규 의존성: 없음

## Sprint 9 주요 도메인 결정 사항 (Sprint 10에 영향)

- PI-02 보강-결석 매칭: 일 단위 확정 (분 단위 전환은 T3 주석 위치로 가능 -- R58)
- 보강 가능일: 케이스 A(평일+보강불가 코드 없음) OR 케이스 B(allows_makeup_class=1 명시)
- 정규 수업 요일 보강 허용 (T3 검증 3 폐기)
- 보강 미등원 UI 폐기 (J5), 보강데이 일괄 폐기 (J7) -- dead code 정리 Sprint 10 T1
- 시간 표시: UI=시간(h), 백엔드=class_minutes(분)

## Sprint 7 주요 발견 사항

### Keychain 반복 다이얼로그 근본 원인 (Issue 1)
- `verify_password`가 keyring을 3회 개별 호출
- 해결: CredentialCache (OnceLock<Mutex<Option<CachedCredentials>>>) 도입

### device_id 영속화 (Issue 8)
- OS 로컬(app_config_dir) 저장 권장

## Sprint 4 완료 결과 (2026-05-21)

- **신규 의존성 (허가 완료)**: @base-ui/react, class-variance-authority, clsx, lucide-react, tailwind-merge, tw-animate-css, @dnd-kit/*
- **shadcn 트랩 (R23)**: globals.css CSS 변수 수동 추가 필요

## 미결정 항목 (PI)

- PI-03 (Sprint 10): 캘린더 라이브러리 선택 -- **결정 완료** (FullCalendar, ADR-006)
- PI-04 (Sprint 10): 보강데이 일괄 -- **결정 완료** (캘린더 보강관리뷰 진입점 제공)
- PI-05 (Medium): 일련번호 자동 채번 -- **확정** (MAX+1 + BEGIN IMMEDIATE)
- PI-07 (High): 복구 코드 -- **결정 완료** (Sprint 12에서 전면 제거)
- PI-10 (Sprint 11): 마감 후 수정 사유 UX -- **결정 완료** (모달 다이얼로그)
- PI-11 (Sprint 11): 마감 해제(reopen) -- **결정 완료** (불가, 개별 수정만)
- PI-12 (Sprint 11): 수납 테이블 분리 -- **결정 완료** (별도 payments 테이블)
- 마감(closed) 개념 폐기 (post-Sprint 11 원장 결정, V111)

## Sprint Planner 사전 검토 체크리스트 (A4 반영)

- 마이그레이션 대상 컬럼 타입은 data-model.md 기준으로 명시할 것
- 외부 라이브러리/매크로 사용 Task는 현재 코드 사용 현황 확인 후 포함할 것
- 이연 Task는 기술적 실현 가능성 사전 검증 후 계획에 포함할 것
- 스테이징 검증 이슈가 있으면 다음 sprint에 반드시 전수 포함 (임의 누락 금지)
- A51: 도메인 규칙 중 운용 관행 교차 항목은 T1/T2 설계 단계에서 사용자 확인 선행

## 기술 스택 및 프로젝트 특이사항

- Cargo.toml: sqlx 0.8 + tauri 2 + thiserror 2 + keyring, zeroize, rusqlite, fs2, uuid (argon2 Sprint 12에서 제거)
- `cargo` 명령에 `--manifest-path src-tauri/Cargo.toml` 필수 (루트에 Cargo.toml 없음)
- DB 개발: `./SmartHB-dev.db` (루트, SQLCipher 미적용 가능). 프로덕션: 클라우드 동기화 폴더 하위
- PR 단계 생략 정책: 단일 개발자, `gh pr create` 금지
