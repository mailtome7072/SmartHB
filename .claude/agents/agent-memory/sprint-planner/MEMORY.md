# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 3 (2026-05-21)
- 다음 스프린트 번호: 4
- Sprint 3 계획 수립: 2026-05-21 (마법사 + R12/R13/R14 해소 + 원생 관리 프론트 + 앱 셸)
- Sprint 2 계획 수립: 2026-05-20 (sprint1 잔여 + 기반 도메인 백엔드 통합)

## 프로젝트 기본 정보

- **기술 스택**: Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- **Phase 구조**: 7 Phase, 14 Sprint (총 30주). Phase 1 진행 중 (Sprint 1~3)
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)
- **전문가 리뷰 4관점**: 보안/성능/UX/PO — docs/phase/phase1/ 하위 파일

## 미결정 항목 (PI)

- PI-05 (Medium): 일련번호 자동 채번 — **확정 (2026-05-20)**: `MAX+1` + `BEGIN IMMEDIATE` + override 허용
- PI-07 (High): 복구 코드 — **결정 완료** (PRD v1.5.1). Argon2id 해시, Sprint 1 구현

## Sprint 1 핵심 리스크 (해소 완료)

- SQLCipher PoC: bundled-sqlcipher-vendored-openssl 양 OS 빌드 성공 (ADR-001)
- CI Forbidden Area: 사용자 허가 받아 ci.yml/deploy.yml 수정 완료

## Sprint 2 핵심 사항

- 마이그레이션 번호 예약: V100~V199
- PI-05 (일련번호 자동 채번): 확정 — `MAX+1` + `BEGIN IMMEDIATE` + override. T5/T9/T13/T14에 반영
- Sprint 1 잔여 3건: R6(salt 이전), R7(release_lock), R8(cipher on 실측)
- 루트 라우팅 + 인증 게이트: Sprint 1 범위 외였으나 PRD SS5.6 인수 기준 필수
- PR 단계 생략 정책: 단일 개발자, `gh pr create` 금지

## Sprint 3 핵심 사항

- 마이그레이션 번호 예약: V200~V299
- 신규 의존성: tauri-plugin-dialog + @tauri-apps/plugin-dialog (사용자 허가 필요)
- 신규 의존성: zustand + @tanstack/react-query
- ADR-006 Pretendard: self-host 방식 확정 (public/fonts/)
- Sprint 2 backlog 해소: R12(salt 이전), R13(PII 마스킹), R14(페이지네이션)
- A4 액션 아이템 반영: data-model.md SSOT 대조 + 코드 현황 사전 확인 적용

## Sprint Planner 사전 검토 체크리스트 (A4 반영)

- 마이그레이션 대상 컬럼 타입은 data-model.md 기준으로 명시할 것
- 외부 라이브러리/매크로 사용 Task는 현재 코드 사용 현황 확인 후 포함할 것
- 이연 Task는 기술적 실현 가능성 사전 검증 후 계획에 포함할 것

## 반복 위반 패턴 (세션 로그 기반)

<!-- session-summary.md에서 3회 이상 반복 패턴 발생 시 여기에 기록 -->
<!-- 형식: - [YYYY-MM-DD] {패턴}: {파일 또는 규칙} → 스프린트 계획 시 주의 -->

## 기술 스택 및 프로젝트 특이사항

- Cargo.toml 현재: sqlx 0.8 + tauri 2 + thiserror 2. Sprint 1에서 keyring, argon2, zeroize, rusqlite, fs2, uuid 추가 예정
- `cargo` 명령에 `--manifest-path src-tauri/Cargo.toml` 필수 (루트에 Cargo.toml 없음)
- DB 개발: `./SmartHB-dev.db` (루트, SQLCipher 미적용 가능). 프로덕션: 클라우드 동기화 폴더 하위
