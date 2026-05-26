# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

- [Sprint 현황](sprint-status.md) -- Sprint 10 계획 수립 완료, Phase 3 마지막 sprint (소멸 자동 전이 + 캘린더 뷰)
- [마이그레이션 번호 정책](migration-numbering.md) -- V107 완료, Sprint 10에서 V108 사용 (dead code CHECK 제약 정리 또는 소멸 관련)
- [Capacity 패턴](capacity-pattern.md) -- Sprint 9(38h계획/52h실측, +37%) → Sprint 10(38h구현+6h시각검증버퍼=44h)
- [Velocity 데이터](velocity.md) -- 시각 검증 버퍼 6h 별도 예약 패턴 정착 (A50)

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 9 (2026-05-26)
- 다음 스프린트 번호: 10
- Sprint 10 계획 수립: 2026-05-26 (Phase 3 소멸 자동 전이 + 퇴교 보강 + 선행 수업 + 캘린더 뷰 — T1~T12 12개 Task, 44h 예상)
- Sprint 9 계획 수립: 2026-05-24 (Phase 3 보강 등록 + 매칭 — T1~T9 9개 Task, 38h 예상, PI-02 사용자 결정 대기)
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

## 프로젝트 기본 정보

- **기술 스택**: Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- **Phase 구조**: 7 Phase + 1.5(안정화) + 1.5b(추가 안정화). Phase 1~2 완료, Phase 3 진행 중 (Sprint 10이 마지막)
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)

## Sprint 9 주요 도메인 결정 사항 (Sprint 10에 영향)

- PI-02 보강-결석 매칭: 일 단위 확정 (분 단위 전환은 T3 주석 위치로 가능 — R58)
- 보강 가능일: 케이스 A(평일+보강불가 코드 없음) OR 케이스 B(allows_makeup_class=1 명시)
- 정규 수업 요일 보강 허용 (T3 검증 3 폐기)
- 보강 미등원 UI 폐기 (J5), 보강데이 일괄 폐기 (J7) — dead code 정리 Sprint 10 T1
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

- PI-03 (Sprint 10): 캘린더 라이브러리 선택 — T8 진입 시 ADR (brainstorming 스킬)
- PI-04 (Sprint 10): 보강데이 일괄 등록 버튼 범위 — T11 진입 시 사용자 확인
- PI-05 (Medium): 일련번호 자동 채번 — **확정** (MAX+1 + BEGIN IMMEDIATE)
- PI-07 (High): 복구 코드 — **결정 완료** (PRD v1.5.1)

## Sprint Planner 사전 검토 체크리스트 (A4 반영)

- 마이그레이션 대상 컬럼 타입은 data-model.md 기준으로 명시할 것
- 외부 라이브러리/매크로 사용 Task는 현재 코드 사용 현황 확인 후 포함할 것
- 이연 Task는 기술적 실현 가능성 사전 검증 후 계획에 포함할 것
- 스테이징 검증 이슈가 있으면 다음 sprint에 반드시 전수 포함 (임의 누락 금지)
- A51: 도메인 규칙 중 운용 관행 교차 항목은 T1/T2 설계 단계에서 사용자 확인 선행

## 기술 스택 및 프로젝트 특이사항

- Cargo.toml: sqlx 0.8 + tauri 2 + thiserror 2 + keyring, argon2, zeroize, rusqlite, fs2, uuid
- `cargo` 명령에 `--manifest-path src-tauri/Cargo.toml` 필수 (루트에 Cargo.toml 없음)
- DB 개발: `./SmartHB-dev.db` (루트, SQLCipher 미적용 가능). 프로덕션: 클라우드 동기화 폴더 하위
- PR 단계 생략 정책: 단일 개발자, `gh pr create` 금지
