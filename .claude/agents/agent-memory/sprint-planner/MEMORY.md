# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 3 (2026-05-21)
- 다음 스프린트 번호: 4 (Phase 1.5 품질 안정화 — 진행 중)
- Sprint 4 계획 수립: 2026-05-21 (Phase 1 스테이징 검증 14개 이슈 해소 + 교습소 설정)
- Sprint 3 계획 수립: 2026-05-21 (마법사 + R12/R13/R14 해소 + 원생 관리 프론트 + 앱 셸)
- Sprint 2 계획 수립: 2026-05-20 (sprint1 잔여 + 기반 도메인 백엔드 통합)

## ROADMAP 번호 이동 (Sprint 4 삽입)

Sprint 4가 "Phase 1.5 품질 안정화"로 삽입됨에 따라:
- 원래 Sprint 4 (학사 스케줄) -> Sprint 5
- 원래 Sprint 5 (출결 관리) -> Sprint 6
- Phase 3 이후(Sprint 6~14)는 아직 미조정 — sprint-close 시 일괄 정리 필요

## 프로젝트 기본 정보

- **기술 스택**: Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- **Phase 구조**: 7 Phase + 1.5(안정화), 15 Sprint (총 32주 예상). Phase 1 완료, Phase 1.5 진행 중
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)

## Sprint 4 핵심 사항

- 마이그레이션 번호: V201 (students.withdrawn_at 추가)
- 신규 의존성 후보: @dnd-kit/core + @dnd-kit/sortable (사용자 허가 필요)
- shadcn/ui AlertDialog 도입 (src/components/ui/ 현재 비어있음 — shadcn init 필요)
- R22 해소: window.confirm -> shadcn AlertDialog
- 교습소 운영 시간 설정 (app_settings key/value 방식)
- 원래 ROADMAP Sprint 4 범위(학사 스케줄)는 Sprint 5로 이연

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
