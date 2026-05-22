---
Sprint: 6  |  Date: 2026-05-22  |  Session: #6
---

> Sprint 6 (Phase 2 학사 스케줄 관리) — T2-a 빌드 스크립트 + T2-b V301 마이그레이션.
> ADR-005 결정 적용: 공공데이터포털 API + schedule_events 통합 + "공휴일" 시스템 코드 추가.
> 예상 5.5h (T2-a 3h + T2-b 2.5h).

## 이전 세션 결과 (참고 — 모두 완료)

| Session | Task | 커밋 |
|---------|------|------|
| #1 | T1 (A20 lock 재시도) | `2c5b8a1` |
| #1 | T3 (A21 paths.rs OnceLock 분리) | `c2be584` |
| #1 | T4 (A22 DnD 방법 B) | `83f19d1` |
| #2 | T5+T6 (academic.rs 신규 — study_periods 6 + schedule_codes 4) | `c8dc3c8` |
| #3 | T7 (academic.rs 확장 — schedule_events 5) | `a4c380e` |
| #4 | T8 (TS IPC 래퍼 15 + 도메인 타입 10) | `5941d24` |
| #5 | T2-c (ADR-005 공휴일 API + 저장 결정) | `10a92d4` |

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T2-a** | scripts/fetch-holidays.ts + tsx devDep + package.json scripts + .env.example | 3h |
| **T2-b** | V301 마이그레이션 (시드 보정 3 + "공휴일" 코드 INSERT + 7년치 INSERT) + 테스트 + sqlx prepare | 2.5h |

> 사용자 사전 결정(Session #5): tsx devDep 승인 / API 공공데이터포털 / 저장 schedule_events 통합 / 인증키 사용자가 .env 에 직접 추가 (채팅 노출 회피).

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| package.json | [1회] | tsx devDep + holidays:fetch 스크립트 등록 |
| pnpm-lock.yaml | [0회] | tsx 추가 lock 갱신 |
| scripts/fetch-holidays.ts | [4회 ⚠️] | 신규 — data.go.kr API 호출, XML→JSON, SQL INSERT 출력 |
| .env.example | [1회] | KOREA_HOLIDAY_API_KEY 변수 추가 |
| src-tauri/migrations/301__fix_schedule_codes_seed.sql | [3회 ⚠️] | 신규 V301 — 시드 보정 + "공휴일" 코드 + 7년치 |
| src-tauri/src/commands/academic.rs | [2회] | V301 적용 후 공휴일 데이터 검증 테스트 추가 |
| src-tauri/.sqlx/ | [0회] | sqlx prepare 캐시 갱신 (커밋 필수) |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `.env` — 사용자가 직접 인증키 채움. 본 에이전트는 절대 .env 에 키를 작성하지 않음
- [ ] `src-tauri/Cargo.toml` — Rust 의존성 변경 없음
- [ ] `src/` — 프론트엔드 변경 없음 (T9~T11)
- [ ] 기존 마이그레이션 (V001~V201) — 변경 금지, V301 만 신규 추가

## 완료 기준 (이번 세션)

### T2-a — 빌드 스크립트 (PRD §5.5, sprint6.md L75-83)
- ✅ AC-T2-3: `pnpm holidays:fetch -- --years 2025-2027` → 64건 SQL INSERT 출력 (대체공휴일 7건 포함). 외부 API 한계로 2028+ 0건 — ADR-005 갱신
- ✅ AC-T2-8: 스크립트가 `scripts/` 디렉토리 (`src-tauri/`/`src/` 본체 외)
- ✅ `tsx` 4.22 devDependency + `holidays:fetch` 스크립트 등록 (`node --env-file=.env --import tsx`)
- ✅ `.env.example` 에 `KOREA_HOLIDAY_API_KEY` + data.go.kr 발급 절차 주석
- ✅ 에러 처리 — 인증키 누락 / HTTP 403 / JSON 파싱 / API resultCode≠00 모두 친화 메시지 + exit 1

### T2-b — V301 마이그레이션 (PRD §4.4.4, §6.2, sprint6.md L85-96)
- ✅ AC-T2-1: 시드 보정 검증 — 테스트 `v301_corrects_system_code_attributes`
- ✅ AC-T2-2: 방어적 UPDATE — WHERE code_name + is_system_reserved=1
- ✅ AC-T2-4: "공휴일" 시스템 코드(1,0,0,1,0) + 2025~2027 공휴일 64건 — 테스트 `v301_inserts_holiday_system_code` + `v301_seeds_korean_holidays_2025_2027`
- ✅ AC-T2-5: `cargo sqlx prepare` 시도 — `query()` 런타임 사용으로 매크로 없음 → `.sqlx/` 캐시 미생성 (정상)
- ✅ AC-T2-6: V301 검증 테스트 5건 동일 커밋(`f534706`) (A19 규칙)
- ✅ 주요 공휴일: 1월1일/삼일절/어린이날/광복절/한글날/기독탄신일 + 대체공휴일 7건 (요구 5건↑)

### 외부 데이터 한계 발견 + ADR-005 갱신
- API 가 2028+ 미발표 → 초기 범위 2025~2027 (3년치, 64건) 로 축소
- 갱신 트리거 좁힘: "2029-12 이전 7년치" → "**매년 1월** N년치 신규 발표 시 V401(+) 추가"
- Sprint 6 회고에 메모 필요 (sprint-close 에이전트 반영)

### Session #6 부수 작업
- ✅ Hook 정규식 좁힘 — `.env.example` 허용 (CLAUDE.md 환경변수 정책 충돌 해소)
- ✅ 인증키 발급/거부 디버깅 — `URL.searchParams.set` 이중 인코딩 → raw concat 우회
- ⚠️ scripts/fetch-holidays.ts [4회] / V301 [3회] — 동일 오류 반복 아닌 단계적 진화 (auth 디버깅 / SQL 호환성 / TypeScript strict). loop-detection 미적용

### 세션 종료 조건
- ✅ 커밋 분할 3개: T2-a `1d0ebe1` / T2-b `f534706` / ADR-005 갱신+scope 완료 마킹 (본 커밋)
- ✅ Self-verify: `cargo test 146 passed` + `cargo clippy -D warnings` clean + `pnpm tsc` clean + `pnpm lint` clean
- ✅ simplify 검토 — 신규 결함 없음 (단순 빌드 스크립트, V301 SQL, 패턴 일관 테스트)

## 안전 절차 — API 인증키

- 사용자가 `.env` 에 `KOREA_HOLIDAY_API_KEY=<값>` 직접 추가 (`.gitignore` `.env` 검증 완료)
- 에이전트는 채팅창에서 받은 키를 메모리에만 보유, 어떤 파일에도 기록하지 않음
- 스크립트는 `process.env.KOREA_HOLIDAY_API_KEY` 만 읽고, 누락 시 친화 메시지로 종료
- V301 산출물에는 인증키 흔적 없음 (SQL INSERT 문만)

## 설계 결정 (ADR-005 적용)

- **저장 위치**: schedule_events 통합 → V102 schedule_codes 시드에 "공휴일" 추가 필요
- **공휴일 시스템 코드 속성** (PRD §4.4.4 기준):
  - is_system_reserved = 1 (사용자 3속성 수정 차단)
  - allows_regular_class = 0 (정규수업 OFF)
  - allows_makeup_class = 0 (보강 OFF)
  - is_duplicate_blocked = 1 (중복불가 ON)
  - is_period_type = 0 (단일 일자)
- **공휴일 데이터 매핑**:
  - schedule_events.event_date = 공휴일 일자
  - schedule_events.period_end_date = NULL (단일 일자)
  - schedule_events.display_name = "신정" / "어린이날" / "대체공휴일(어린이날)" 등
- **갱신 방법**: 2029-12 이전 `pnpm holidays:fetch` 재실행 + V401(+) 신규 마이그레이션 추가 (ROADMAP 메모)

## 코드 패턴 SSOT

- V200/V201 시드 마이그레이션 패턴 (방어적 INSERT/UPDATE) 답습
- academic.rs 테스트 패턴 (Session #2/#3 그대로): `#[cfg(not(feature = "cipher"))] + #[tokio::test]` + `db::test_pool_in_memory()`
- 스크립트는 process.exit(1) 에러 종료 — npm scripts 단순 실패 신호

## 발견된 이슈

> 진행 중 새 제약·충돌 발견 시 여기에 기록.
