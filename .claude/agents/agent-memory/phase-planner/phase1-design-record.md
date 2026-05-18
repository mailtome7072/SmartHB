---
name: phase1-design-record
description: Phase 1 (인프라+기반도메인) 설계 결정 사항과 전문가 검토 핵심 권고
metadata:
  type: project
---

## Phase 1 설계 (2026-05-18)

- 3스프린트, ADR 6건 (SQLCipher, app.lock, 백업, Keychain, PI-05 일련번호, Pretendard)
- Sprint 1이 가장 리스크 높음: SQLCipher PoC 첫 2일 집중 필수
- 전문가 핵심 권고:
  - 보안: zeroize crate 필수, 복구 코드 Argon2id 해시, PBKDF2 600K iterations
  - 성능: PRAGMA quick_check 사용 (integrity_check 대비 10~100x 빠름), 시작 시퀀스 병렬화
  - UX: 마법사 9단계 유지하되 진행률 표시 필수, 잠금 화면 입력 필드 56px
  - 인프라: bundled-sqlcipher-vendored-openssl feature 확인 (CI OpenSSL 설치 불필요 가능)
- PI-05 (일련번호 자동 채번)는 Sprint 2 진입 전 사용자 결정 필요

**Why:** Phase 1은 후속 전 Phase의 기반. SQLCipher + app.lock + 백업이 실패하면 전체 일정에 영향.
**How to apply:** Sprint 1 sprint-planner 호출 시 PoC 우선 배치, ADR-001~004 Sprint 1 진입 시 작성.
