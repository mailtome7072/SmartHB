# Test Report — Sprint 4 (Phase 1.5 품질 안정화)

> 검증 일시: 2026-05-22
> 브랜치: `develop`
> 기준 커밋: `b7e9ca6` (feat: Sprint 4 완료)

---

## 자동 검증 결과

| 검증 항목 | 결과 | 세부 |
|----------|------|------|
| `cargo test` | ✅ 통과 | 130 passed, 0 failed — Sprint 3 123건 대비 +7건 (settings 6 + serial_sort 1) |
| `cargo clippy -- -D warnings` | ✅ 통과 | 경고 0건 |
| `pnpm tsc --noEmit` | ✅ 통과 | 타입 오류 0건 |
| `pnpm lint` | ✅ 통과 | ESLint 경고/오류 0건 |
| `pnpm build` | ✅ 통과 | Next.js static export 성공 (12페이지, First Load JS 102kB) |

### cargo test 상세

```
test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
finished in 16.72s
```

Sprint 4에서 추가된 테스트 7건:
- `commands::settings::tests::default_has_seven_days`
- `commands::settings::tests::default_weekdays_open_13_to_20`
- `commands::settings::tests::default_weekend_is_closed`
- `commands::settings::tests::day_hours_serde_roundtrip_open`
- `commands::settings::tests::day_hours_serde_roundtrip_closed`
- `commands::settings::tests::vec_of_seven_serializes_compactly`
- `commands::students::tests::serial_asc_sql_uses_cast_integer`

### 알려진 flaky 테스트

- `paths::tests::init_from_config_ignores_empty_path` — 병렬 실행 시 OnceLock 격리 부족으로 간헐적 실패. `--test-threads=1` 직렬 실행 시 OK. Sprint 3 기존 결함이며 Sprint 4 변경과 무관. Sprint 5 carry-over (테스트 격리 강화 필요).

### pnpm build 라우트 결과

```
Route (app)                                 Size  First Load JS
┌ ○ /                                    1.35 kB         121 kB
├ ○ /settings                            1.17 kB         121 kB
├ ○ /settings/codes                      19.6 kB         139 kB
├ ○ /settings/hours                      2.14 kB         122 kB
├ ○ /students                            2.56 kB         122 kB
├ ○ /students/edit                       39.8 kB         159 kB
└ ○ /students/new                        2.58 kB         122 kB
```

Sprint 4 신규 라우트: `/settings`, `/settings/hours` 빌드 포함 확인.

---

## 수동 검증 항목

| 항목 | 상태 |
|------|------|
| `pnpm tauri:dev` 실행 후 앱 동작 확인 | ⬜ 미완료 (개발자 수행 필요) |
| 14개 이슈 매트릭스 재검증 (사용자 시각) | ✅ 완료 (2026-05-21 — scope.md T11 기록) |
| 교습소 설정 화면 → 운영 시간 저장 확인 | ⬜ 미완료 (개발자 수행 필요) |
| 스케줄 시작시간 콤보 → 운영시간 반영 확인 | ⬜ 미완료 (개발자 수행 필요) |
| 코드 테이블 DnD → 새로고침 후 순서 유지 확인 | ⬜ 미완료 (개발자 수행 필요) |

---

## 배포 준비도 사전 확인

| 항목 | 결과 |
|------|------|
| CHANGELOG.md `[Unreleased]` 섹션 Sprint 4 내용 기재 | ✅ 확인 — Added/Changed/Fixed/Security 4개 카테고리 기재 |
| 하드코딩 시크릿 패턴 (변경 파일 대상) | ✅ 없음 |
| Next.js CVE-2025-66478 | ⚠️ 미적용 — CHANGELOG Security 섹션에 "Sprint 5 또는 hotfix 업그레이드 필수"로 기록됨 |

---

## 결론

자동 검증 5개 항목 전체 통과. 코드 리뷰 Critical/High 이슈 없음. Medium 1건(DnD 필터 sort_order 충돌)은 Risk Register 등록 후 Sprint 5 처리 예정. `pnpm tauri:dev` 실행 수동 검증은 개발자가 별도 수행 필요.
