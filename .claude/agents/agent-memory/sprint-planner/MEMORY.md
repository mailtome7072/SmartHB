# Sprint Planner 메모리

이 파일은 sprint-planner 에이전트의 영구 메모리입니다.
프로젝트 진행 상황, 기술 스택, 패턴 등을 기록합니다.

## 스프린트 현황

<!-- sprint-close 완료 시 업데이트 -->
- 마지막 완료 스프린트: Sprint 7 (2026-05-22)
- 다음 스프린트 번호: 8
- Sprint 7 계획 수립: 2026-05-22 (Phase 2 carry-over 해소 — Keychain 캐싱 + salt 이전 + device_id 영속화 + 교습기간 UX + 배치 제약)
- Sprint 6 계획 수립: 2026-05-22 (Phase 2 학사 스케줄 — 3개월 캘린더 + 교습기간 + 일정 코드 + 배치)
- Sprint 5 계획 수립: 2026-05-22 (Phase 1.5b — Node 25 호환 + single-instance + 시드)
- Sprint 4 계획 수립: 2026-05-21 (Phase 1 스테이징 검증 14개 이슈 해소 + 교습소 설정)
- Sprint 3 계획 수립: 2026-05-21 (마법사 + R12/R13/R14 해소 + 원생 관리 프론트 + 앱 셸)
- Sprint 2 계획 수립: 2026-05-20 (sprint1 잔여 + 기반 도메인 백엔드 통합)

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
- **Phase 구조**: 7 Phase + 1.5(안정화) + 1.5b(추가 안정화). Phase 1 완료, Phase 1.5 완료, Phase 1.5b 완료, Phase 2 진행 중
- **핵심 참조 문서**: PRD.md (v1.5.1), ROADMAP.md (SSOT), docs/phase/phase1.md (Phase 설계)
- **데이터 모델**: docs/data-model.md v1.5 (V001~V008 마이그레이션 가이드)

## Sprint 7 주요 발견 사항

### Keychain 반복 다이얼로그 근본 원인 (Issue 1)
- `verify_password`가 keyring을 3회 개별 호출: `retrieve_salt_from_keyring()` + `derive_key_async()` + `retrieve_key_from_keyring()`
- macOS Security Framework가 각 `keyring::Entry::get_password()` 호출마다 사용자 승인 요청
- 해결: CredentialCache (OnceLock<Mutex<Option<CachedCredentials>>>) 도입 + 앱 시작 시 1회 로드
- 영향 파일: auth.rs, db.rs, integrity.rs, backup.rs, recovery.rs (5파일)

### Keychain 항목 현황 (auth.rs 분석)
- KEYRING_SERVICE: "SmartHB"
- KEYRING_USER_KEY: "db_encryption_key" (AES-256 32byte hex)
- KEYRING_USER_SALT: "pbkdf2_salt" (32byte hex) -- T2에서 salt.bin 파일로 이전 대상
- KEYRING_USER_RECOVERY: "recovery_code_hash" (Argon2id PHC 문자열)
- Sprint 7 이후 Keychain 잔여: key + recovery_code_hash (2항목), salt 제거

### device_id 영속화 (Issue 8)
- lock.rs:46 `OnceLock<Uuid> + Uuid::new_v4()` -- 매 프로세스 새 UUID
- R37 리스크: 클라우드 폴더에 저장하면 양 PC 동일 ID 복제 -- OS 로컬(app_config_dir) 저장 권장

## Sprint 4 완료 결과 (2026-05-21)

- **신규 의존성 (허가 완료, 모두 사용 중)**: @base-ui/react + class-variance-authority + clsx + lucide-react + tailwind-merge + tw-animate-css (shadcn init 부산물) + @dnd-kit/core + @dnd-kit/sortable + @dnd-kit/utilities
- **shadcn 트랩 (R23)**: init이 globals.css CSS 변수를 안 넣어 AlertDialog 투명 렌더링 -- 13개 토큰 수동 추가
- **R24 검증 통과**: @dnd-kit x React 19 peer dep 경고 없음

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
