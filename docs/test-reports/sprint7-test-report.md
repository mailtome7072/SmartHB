# Test Report — Sprint 7 (2026-05-22)

> 대상 브랜치: `sprint7` → `develop` 머지 완료 (61e7bc3, --no-ff)
> 검증 시점: 2026-05-22 sprint-review 단계
> T2는 Session #2에서 high-effort code review 완료 (S-T2-1~6 동행 패치). 본 보고서는 T1, T3~T10 대상.

---

## 자동 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --lib` (cipher off) | **통과** | 177 passed, 0 failed |
| `cargo test --lib --features cipher` | **통과** | 127 passed, 0 failed |
| `cargo clippy -- -D warnings` (cipher off) | **통과** | 0 warnings |
| `cargo clippy --features cipher -- -D warnings` | **통과** | 0 warnings |
| `pnpm tsc --noEmit` | **통과** | 오류 없음 |
| `pnpm lint` | **통과** | ESLint 경고/오류 0건 |
| `pnpm build` | **통과** | 12개 페이지 prerendered, `/settings/schedule-codes` 신규 포함 |

### Flaky 테스트 기록

`commands::lock::tests::release_lock_atomic_removes_self_owned_lock` (cipher off) — 1차 실행 시 간헐적 실패, 재실행 시 통과.

**원인**: `lock_path()`가 `paths::data_root()`에 의존하는 process-wide 전역 경로를 반환하므로, `cargo test --lib`의 병렬 테스트 실행 중 다른 테스트가 동일 lock 파일을 점유할 때 race 발생.

**영향 범위**: 단위 테스트 환경에 한정. 프로덕션은 단일 인스턴스 모델 (`tauri-plugin-single-instance`)로 lock 동시 접근 시나리오 자체가 존재하지 않음.

**대응**: carry-over I-S2-2~10과 함께 후속 스프린트에서 테스트 격리 강화 검토.

---

## 코드 리뷰 결과 (T1, T3~T10)

> T2는 Session #2 high-effort code review에서 S-T2-1~6 전수 패치 완료. 본 섹션은 T1, T3~T10 대상.

| 등급 | 건수 | 내용 |
|------|------|------|
| Critical | 0 | — |
| High | 0 | — |
| Medium | 1 | StudyPeriodEditor T6: createStudyPeriod 성공 + confirmStudyPeriod 실패 시 orphan 미확정 period 잔존 → retry 시 overlap 오류로 차단 |
| Low | 0 | — |

**Medium 이슈 상세**:

- 파일: `src/components/academic/StudyPeriodEditor.tsx:104`
- 시나리오: `createStudyPeriod` 성공 후 `confirmStudyPeriod` 실패(DB 락, IPC 오류 등) → `onError`에서 에러 메시지 표시 + `confirmOpen=false`. 그러나 DB에 미확정 교습기간이 잔존. 사용자가 같은 날짜로 재시도 시 `create_study_period` 백엔드의 overlap 검사(is_confirmed 필터 없음)에 의해 "다른 교습기간과 일자가 중첩됩니다" 오류 발생. `delete_study_period` IPC를 수동 호출하지 않으면 재시도 불가.
- 대응: R39로 리스크 등록. 다음 스프린트에서 `create+confirm`을 단일 트랜잭션 IPC로 통합하거나 overlap 검사에 `is_confirmed` 필터 추가 검토.

---

## 배포 준비도 사전 확인

| 항목 | 결과 |
|------|------|
| `CHANGELOG.md [Unreleased]` 섹션 | **업데이트됨** — Sprint 7 Added/Changed 항목 전수 기재 확인 |
| 하드코딩 시크릿 패턴 스캔 | **없음** — `*.rs`, `*.ts`, `*.tsx` 변경 파일 전수 확인 |

---

## 수동 검증 항목 (개발자 수행 필요)

아래 항목은 `pnpm tauri:dev` 실행 후 수동으로 확인해야 합니다. DEPLOY.md의 시각 검증 체크리스트를 참조하세요.

| 항목 | 검증 내용 | 상태 |
|------|----------|------|
| T1 | 비밀번호 입력 시 Keychain 다이얼로그 최대 1회, startup < 3초 | ⬜ 미완료 |
| T2 | `{cloud}/smarthb/salt.bin` 파일 생성 확인 + Keychain salt 마이그레이션 | ⬜ 미완료 |
| T3 | 앱 재시작 후 동일 device_id 유지 (`app_config_dir/device.id`) | ⬜ 미완료 |
| T4 | 캘린더 배지 색상 정상(시스템 코드 하드코딩 제거) + 시스템 코드 드래그 차단 | ⬜ 미완료 |
| T5 | `/settings/schedule-codes` 코드 CRUD 동작 + `/academic` 코드 패널 제거 확인 | ⬜ 미완료 |
| T6 | 교습기간 미확정 월에서 셀 클릭 즉시 selection 모드 진입 | ⬜ 미완료 |
| T7 | 중복불가 일정 상호 차단 + 교습기간 외 배치 차단 확인 | ⬜ 미완료 |
| T8 | 교습기간 삭제 → cascade 삭제 → 공휴일 보존 확인 | ⬜ 미완료 |
| T9 | 공휴일 배지 삭제 버튼 비표시 + 삭제 시도 차단 확인 | ⬜ 미완료 |
| UC-2 | 월말 학사 일정 수립 전체 흐름 완주 가능 여부 | ⬜ 미완료 |

---

## 결론

자동 검증(cargo test, clippy, tsc, lint, build) 전수 통과. 코드 리뷰 Medium 1건(R39 등록). 수동 시각 검증은 개발자가 `pnpm tauri:dev` 실행 후 DEPLOY.md 체크리스트 기준으로 수행 필요.
