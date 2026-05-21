# Test Report — 2026-05-21 (Sprint 3)

> Sprint 3: 원생 관리 프론트 + 초기 설정 마법사 + 글로벌 검색 + 접근성 기반
> 브랜치: develop (sprint3 머지 완료)
> 변경 파일: 47개 (백엔드 7 / 프론트엔드 24 / 문서·폰트·설정 16)

---

## 자동 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test` | 통과 | 109 passed (Sprint 2 97건 → +12건) |
| `cargo clippy -- -D warnings` | 통과 | 경고 0건 |
| `pnpm tsc --noEmit` | 통과 | 타입 에러 0건 |
| `pnpm lint` | 통과 | ESLint 경고·에러 0건 |
| `pnpm build` | 통과 | Next.js static export, 전 라우트 성공 |

### cargo test 신규 테스트 케이스 (+12건)

| 모듈 | 신규 케이스 |
|------|------------|
| `commands::pagination` | `clamp_applies_default_when_none`, `clamp_enforces_floor_and_ceiling` (2건) |
| `commands::students` | `build_filter_clause_*` 3건, `list_students_respects_limit_and_offset`, `count_matches_filtered_total` (5건) |
| `commands::codes` | `list_codes_limit_offset_respected`, `count_codes_matches_seed_total` (2건) |
| `commands::setup` | `setup_status_default_is_empty_and_not_completed`, `setup_status_serde_round_trip`, `setup_status_parses_when_field_missing` (3건) |

---

## 수동 검증 항목

- ⬜ `pnpm tauri:dev` 실행하여 앱 동작 수동 확인 (스테이징 검증) — 개발자 수행 필요
- ⬜ 초기 설정 마법사 4단계 전체 완주 확인
- ⬜ 원생 등록/수정/조회/퇴교 전체 흐름 확인
- ⬜ 글로벌 검색바 원생 이름 검색 + 1클릭 이동 확인
- ⬜ 코드 테이블 관리 CRUD 확인
- ⬜ 수업 스케줄 편집 UI + 표준 교습비 자동 매칭 확인
- ⬜ Pretendard 18pt / 44×44px 클릭 영역 / WCAG AA 명도 대비 시각 확인

---

## 결론

자동 검증 5개 항목 전체 통과. 수동 검증 7개 항목은 개발자 스테이징 환경 실행 후 확인 필요.
배포 준비도 사전 확인: CHANGELOG.md `[Unreleased]` 섹션 업데이트 완료, 하드코딩 시크릿 패턴 없음.
