# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 1 (2026-05-19)
- 다음 스프린트 번호: 2

## 프로젝트 기본 정보

- **기술 스택**: Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- **Phase 구조**: 7 Phase, 14 Sprint (총 30주). Phase 1 진행 중 (Sprint 1~3)
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)
- **전문가 리뷰 4관점**: 보안/성능/UX/PO — docs/phase/phase1/ 하위 파일

## 미결정 항목 (PI)

- PI-05 (Medium): 일련번호 자동 채번 규칙 → Sprint 2 진입 전 사용자 결정 필요
- PI-07 (High): 복구 코드 — **결정 완료** (PRD v1.5.1). Argon2id 해시, Sprint 1 구현

## Sprint 1 핵심 리스크

- SQLCipher PoC: 첫 2일에 `bundled-sqlcipher-vendored-openssl` 양 OS 빌드 검증 필수
- CI Forbidden Area: .github/workflows/ 수정 시 사용자 허가 필요

## 반복 위반 패턴 (세션 로그 기반)

<!-- session-summary.md에서 3회 이상 반복 패턴 발생 시 여기에 기록 -->
<!-- 형식: - [YYYY-MM-DD] {패턴}: {파일 또는 규칙} → 스프린트 계획 시 주의 -->

## 기술 스택 및 프로젝트 특이사항

- Cargo.toml 현재: sqlx 0.8 + tauri 2 + thiserror 2. Sprint 1에서 keyring, argon2, zeroize, rusqlite, fs2, uuid 추가 예정
- `cargo` 명령에 `--manifest-path src-tauri/Cargo.toml` 필수 (루트에 Cargo.toml 없음)
- DB 개발: `./SmartHB-dev.db` (루트, SQLCipher 미적용 가능). 프로덕션: 클라우드 동기화 폴더 하위
