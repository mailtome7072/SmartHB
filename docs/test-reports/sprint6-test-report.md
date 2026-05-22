# Test Report — Sprint 6 (2026-05-22)

> Sprint 6: Phase 2 학사 스케줄 관리 — 교습기간 + 학사 일정 코드 + 학사 일정 + 3개월 캘린더
> 브랜치: `sprint6` → `develop` 머지 완료 (`dc3139e`)
> 검증 실행: 2026-05-22

---

## 자동 검증 결과

| 항목 | 결과 | 상세 |
|------|------|------|
| `cargo test --manifest-path src-tauri/Cargo.toml` | ✅ 통과 | 146 passed / 0 failed (전 Sprint 대비 +5, V301 검증 테스트 신규) |
| `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | ✅ 통과 | 경고 0건 |
| `pnpm tsc --noEmit` | ✅ 통과 | TypeScript strict 오류 0건 |
| `pnpm lint` | ✅ 통과 | ESLint 경고/오류 0건 |
| `pnpm build` | ✅ 통과 | `/academic` 9.49kB / First Load 176kB. 전체 11개 라우트 Static export 성공 |

### cargo test 신규 테스트 (Sprint 6 추가분 +5)

| 테스트 | 검증 항목 |
|--------|----------|
| `overlap_detection_blocks_intersecting_range` | AC-T5-1 일자 중첩 검증 SQL 정확성 |
| `closed_period_is_blocked_by_flag` | AC-T5-2 마감 플래그 조회 정합성 |
| `confirm_sets_flag_to_one` | AC-T5-3 확정 후 is_confirmed=1 반영 |
| `system_reserved_codes_seeded` | AC-T6-1 V102 5종 + V301 공휴일 = 6종 확인 |
| `code_name_unique_violation_detected` | AC-T6-4 UNIQUE 위반 감지 |
| `user_code_crud_and_system_toggle` | AC-T6-5 사용자 코드 CRUD + 시스템 코드 토글 허용 |
| `year_month_of_extracts_valid_prefix` | 날짜 파서 헬퍼 정상 케이스 |
| `year_month_of_rejects_invalid_format` | 날짜 파서 헬퍼 비정상 케이스 |
| `assessment_dates_returns_two_groups_of_five_weekdays` | AC-T7-4 2+4주차 월~금 10건 정확성 |

> 단위 테스트는 `#[cfg(not(feature = "cipher"))]` 조건부 — 인메모리 SQLite 사용. cipher 빌드에서는 생략.

---

## 배포 준비도 사전 확인

| 항목 | 상태 | 비고 |
|------|------|------|
| CHANGELOG.md `[Unreleased]` 섹션 | ✅ 갱신됨 | Sprint 6 Added/Changed/Fixed 각 항목 기재 완료 |
| 하드코딩 시크릿 스캔 (변경 파일) | ✅ 미발견 | `grep -E` 패턴 스캔 결과 없음 |

---

## 수동 검증 항목 (개발자 수행 필요)

- ⬜ `pnpm tauri:dev` 실행 후 `/academic` 진입 — 3개월 캘린더 렌더링 확인
- ⬜ 교습기간 설정: 셀 두 곳 클릭 → 확정 → amber 배경 표시 확인
- ⬜ 학사 일정 배치: 코드 선택 → 셀 클릭 → 배지 등록 확인
- ⬜ 드래그 이동: 단일 일자 배지를 다른 날짜로 드래그 → 이동 확인
- ⬜ 단원평가 자동 배치: "자동 배치" 버튼 → 2/4주차 월~금 배지 10건 확인
- ⬜ 지난 달 셀 클릭 차단, 지난 달 일정 삭제 시도 → 한국어 에러 AlertDialog 확인
- ⬜ `sqlx migrate run` — V301 마이그레이션 적용 확인 (공휴일 64건 schedule_events 확인)
- ⬜ 앱 시작 시 `PRAGMA integrity_check` 통과 확인

---

## 결론

자동 검증 5개 항목 전부 통과. 신규 IPC 15개, 신규 컴포넌트 4개, 백엔드 테스트 146건(+5) 모두 정상. CHANGELOG 및 시크릿 사전 점검 이상 없음. 수동 검증 8개 항목은 개발자 직접 수행 필요.
