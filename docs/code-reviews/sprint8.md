# Sprint 8 코드 리뷰

> **대상**: Sprint 8 (`develop` 머지 후, `bffecb2` ~ `f100708`) — 출결 관리 + Sprint 7 carry-over 흡수
> **리뷰 일자**: 2026-05-24
> **수행**: sprint-review 에이전트 + 사용자 검증
> **체크리스트**: `.claude/skills/code-review.md` + `.claude/rules/backend.md`/`frontend.md`

## 범위

22개 커밋 (T1~T9 + T9 follow-up 3건 + sprint-close 문서화 + V107 hotfix):
- 신규 도메인: `regular_attendances` + `makeup_attendances` 테이블 + IPC 6종 + 프론트엔드 (`/attendance`)
- Sprint 7 carry-over 흡수 9건 (R39~R48-a, R51, A31)
- T9 follow-up: 출결표 sticky 4컬럼 + 너비 30% 감소 + 원생 검색 필터

## 자동 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` (cipher off) | ✅ **222 passed** / 3 ignored / 0 failed |
| `cargo test --lib --features cipher` (cipher on) | ✅ **133 passed** / 3 ignored / 0 failed |
| `cargo clippy --lib -- -D warnings` cipher off + on | ✅ clean |
| `pnpm lint` | ✅ clean |
| `pnpm tsc --noEmit` | ✅ clean |
| `pnpm build` | ✅ static export 성공 (out/ 정상) |

## 발견 사항 (5건)

### 🔴 F2 — `makeup_attendance_id` FK 제약 누락 (High, **해소됨**)

**위치**: `src-tauri/migrations/106__create_attendance_tables.sql` L30

V106 작성 시 `makeup_attendances` forward reference 회피 의도로 FK 절을 누락. 두 테이블 모두 CREATE 된 후라면 SQLite 도 FK 절 사용 가능했으나 단순 `INTEGER` 로만 선언됨.

**실패 시나리오**: Phase 3 보강 매칭에서 `makeup_attendances` 행 삭제 시 `regular_attendances.makeup_attendance_id` 가 dangling reference 로 남고, `compute_summary` 의 needed 계산식 `status='absent' AND makeup_attendance_id IS NULL` 이 해당 행을 제외 → 보강필요시간 0 오집계.

**해소**: V107 마이그레이션 추가 (`f100708`). 테이블 재생성 패턴으로 FK 절 보강 + 신규 단위 테스트 `regular_attendances_makeup_fk_enforced`. 기존 테스트의 dummy id `999` → `seed_makeup` 헬퍼로 실제 id 사용으로 변경.

### 🟡 F1 — `absent_count` 라벨 의미 혼동 (Medium, **차기 sprint 이연**)

**위치**: `src-tauri/src/commands/attendance.rs::compute_summary` + `AttendanceGrid` 헤더 "결석(일)"

`absent_count` 는 `status='absent' AND makeup_attendance_id IS NULL` 행만 카운트 (보강완료/소멸 제외). 사용자가 헤더 "결석(일)"을 "총 결석"으로 오해할 수 있음.

**실패 시나리오**: 원생이 3회 결석 → 1회 보강완료(makeup_done) + 1회 소멸(makeup_expired) + 1회 미처리 상태일 때, 출결표 요약 컬럼 "결석(일)"에 1만 표시. 원장이 "결석 1회만 발생"으로 오인.

**조치**: ROADMAP "차기 sprint 이연 후보" 등재. "미처리 결석(일)" 라벨 변경 또는 툴팁/도움말 검토.

### 🟡 F3 — `get_attendance_grid` N+1 쿼리 패턴 (Medium, **차기 sprint 이연**)

**위치**: `src-tauri/src/commands/attendance.rs::get_grid_impl` (L~395)

학생 루프 안에 4쿼리 (day_rows + cell_rows + `compute_summary` 2건) → 50명 기준 약 200 SQLite 쿼리.

**현재 영향**: PRD §5.7 "50명×31일 < 1초" 요건 통과 (T9 자동 검증). 그러나 데이터 누적이나 느린 HDD/네트워크 sync 환경에서 잠재 위험.

**조치**: ROADMAP 등재. JOIN 또는 단일 IN 쿼리로 batch 처리 검토.

### 🟢 F4 — `is_salt_corrupted` partial-NULL false positive (Low, **무시**)

**위치**: `src-tauri/src/commands/auth.rs::is_salt_corrupted` (L~385)

확률 1.4e-17 (256^-7) — 정상 random salt 의 첫 8바이트가 우연히 모두 동일할 확률. 발생 시 salt.bin 백업·삭제 후 `migrate_keyring_salt_to` 호출, legacy keyring 부재 시 `NotInitialized` 에러로 잠금 가능.

**조치**: 무시. 실용적 확률 무시 가능 수준 (우주 나이 동안 발생 가능성 ~0). 복구 코드 흐름이 있어 잠금 시에도 우회 가능. 회고에 명시만.

### 🟢 F5 — `validate_year_month` 월 범위 미검증 (Low, **차기 sprint 이연**)

**위치**: `src-tauri/src/commands/attendance.rs::validate_year_month` (L~165)

`2026-00` / `2026-13` 같은 의미론적 무효 입력이 GLOB 패턴은 통과. `next_month_str` 단계에서 `NaiveDate::parse_from_str` 실패로 "날짜 파싱 실패" 비친화적 에러.

**조치**: ROADMAP 등재. 정규식/범위 검증 강화 (사용자 친화 메시지 즉시 반환).

## 영역별 추가 점검 (.claude/rules 기반)

### 보안 (backend.md / Critical 항목)
- ✅ SQL 인젝션 — 모든 쿼리 `?` 바인딩, raw string concat 없음
- ✅ 하드코딩된 시크릿 — 없음 (`auth.rs` 검토)
- ✅ Tauri 권한 — 신규 capability 추가 없음
- ✅ SQLCipher 키 Keychain 외부 저장 — 없음 (`CachedCredentials` ZeroizeOnDrop 메모리만)

### 보안 (backend.md / High 항목)
- ✅ `unwrap()`/`expect()` 프로덕션 코드 — 없음 (테스트 모듈만)
- ✅ 마이그레이션 — V106/V107 모두 추가됨
- ✅ `.sqlx/` 캐시 — raw `sqlx::query` 만 사용으로 매크로 미사용 → 갱신 불필요
- ✅ `PRAGMA integrity_check` — startup.rs 의 `check_integrity_quick_for_startup` 유지
- ✅ 락 파일 heartbeat — 변경 없음
- ✅ PRD §6.2 UNIQUE 제약 — `(student_id, event_date)` 적용 / makeup 은 의도적 미적용

### 성능 (backend.md / Medium 항목 + F3)
- ⚠️ F3 — `get_grid_impl` N+1 쿼리 패턴 (위 finding 참조)
- ✅ `app_data_dir()` 사용 — 없음
- ✅ 백업 트랜잭션 — 변경 없음

### 프론트엔드 (frontend.md / Critical 항목)
- ✅ XSS — `dangerouslySetInnerHTML` 사용 없음, 사용자 입력 직접 렌더링 없음
- ✅ `invoke()` 직접 호출 — 없음 (모두 `src/lib/tauri/` 추상화 경유)
- ✅ SQLCipher 키 프론트 메모리 — 없음

### 프론트엔드 (frontend.md / High 항목)
- ✅ TypeScript any — 없음 (`pnpm tsc --noEmit` clean)
- ✅ SSR 가드 — `typeof window` 분기 필요 위치 없음 (Tauri IPC 만)
- ✅ `'use client'` 과다 사용 — `/attendance/page.tsx` 단일 (IPC 사용)
- ✅ 글로벌 검색바 — `AttendancePage` 의 `<AppShell topBarSlot={<GlobalSearch />}>` 유지
- ✅ Pretendard 18pt / WCAG AA / 44×44px — `min-h-[44px]` 적용 확인 (출결 셀 / 검색 input)

### AI 생성 코드 추가 체크 (code-review.md)
- ✅ 미사용 변수/함수 — 없음 (clippy clean)
- ✅ 과도한 추상화 — 없음 (`SetPasswordGuard` / `cred_cache_lock` / `seed_makeup` 모두 단일 책임)
- ✅ 회복 가능한 에러를 panic 으로 처리 — 없음 (테스트 모듈만 `expect`)
- ✅ 주석 정합성 — V107 마이그레이션 / `cred_cache_lock` 헬퍼 / `ensure_cache_loaded` 주석 적절

## 결론

자동 검증 7항목 + 사용자 시각 검증 + 코드 리뷰 모두 완료. F2 는 본 review 직후 V107 hotfix 로 해소되었고, 나머지 F1/F3/F5 는 영향도 Medium/Low 로 차기 sprint 이연. F4 는 무시 결정.

**Sprint 8 출시 준비 완료** — 다음 단계는 회고 작성 + `deploy-prod` 에이전트 (사용자가 별도 신호 후).
