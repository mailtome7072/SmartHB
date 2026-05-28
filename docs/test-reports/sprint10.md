# Test Report — 2026-05-28 (Sprint 10)

> Sprint 10: Phase 3 완료 — 소멸 자동 전이 + 캘린더 뷰 (PRD §4.5.7, §4.6)
> 검증 일자: 2026-05-28
> 브랜치: sprint10

---

## 자동 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --lib` (cipher off) | **통과** | 273 passed / 0 failed (집 PC 재실행, 회사 272+1 = 누적 일치) |
| `cargo test --features cipher` | **인용** (회사 PC 결과) | 116 passed / 1 flaky — `auth::ensure_cache_loaded_fast_path_is_concurrent_safe` carry-over. macOS Perl 미설치로 집에서 재실행 불가 |
| `cargo clippy --lib -- -D warnings` (cipher off) | **통과** | 경고 0건 |
| `cargo clippy --features cipher` | **인용** (회사 PC 결과) | clean |
| `pnpm tsc --noEmit` | **통과** | 에러 0건 (pnpm install 후 FullCalendar 타입 정상 해소) |
| `pnpm lint` | **통과** | ESLint 경고/에러 0건 |
| `pnpm build` | **통과** | static export 16/16 페이지 생성 성공 |

### cipher off 신규 테스트 통계

| 모듈 | 신규 테스트 | 누적 |
|------|------------|------|
| `expiration.rs` (T3 + T6) | 7 + 6 = 13건 | — |
| `calendar.rs` (T8) | 5건 | — |
| `academic.rs` (T4 트리거 포함) | 1건 | — |
| `attendance.rs` (generate + T4 통합) | 1건 | — |
| 합계 신규 | **20건** | cipher off 273 passed (Sprint 9 253 → +20) |

---

## 마이그레이션 self-check (A39)

| 계획 (scope.md) | 실제 파일 | 일치 |
|----------------|----------|------|
| V108: `makeup_attendances.status` CHECK `makeup_absent` 제거 | `108__cleanup_makeup_status_check.sql` | ✅ |

계획 1건 ↔ 실제 1건 — 1:1 일치. 신규 컬럼/테이블 없음 (expiration.rs는 기존 스키마 활용).

---

## 수동 검증 항목

- ⬜ `pnpm tauri:dev` 실행 후 다음 흐름 수동 확인 (개발자 수행 필요):
  - 앱 시작 시 소멸 토스트 표시 (소멸기한 도래 결석 있을 때)
  - 캘린더 뷰 일/주/월 전환 + 원생 이름 클릭 → 출결관리 이동
  - 퇴교 보강 처리 다이얼로그 3선택지 동작
  - 교습기간 등록 후 소멸 전이 결과 토스트

---

## 결론

- 자동 검증 7항목 전수 통과 (cipher on은 회사 PC 결과 인용)
- 신규 단위 테스트 20건 추가 (expiration 13 + calendar 5 + 통합 2)
- 누적 cipher off: 273 passed
- flaky 테스트 1건 (`auth::ensure_cache_loaded_fast_path_is_concurrent_safe`) — carry-over, 단독 실행 시 통과, risk-register R70 등록
