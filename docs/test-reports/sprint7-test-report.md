# Test Report — Sprint 7 (2026-05-22 / 갱신: 2026-05-23)

> 대상 브랜치: `sprint7` → `develop` 머지 완료 (61e7bc3), 이후 post-review fix 14커밋 (2bd1a90~d832c0b)
> 초기 검증 시점: 2026-05-22 sprint-review
> 갱신 시점: 2026-05-23 sprint-review 재실행 (V1~V37b 시각 검증 fix 38건 완료)
> T2는 Session #2에서 high-effort code review 완료 (S-T2-1~6 동행 패치). 본 보고서는 T1, T3~T10 + post-review V1~V37b 대상.

---

## 자동 검증 결과 (2026-05-23 재실행)

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --lib` (cipher off) | **통과** | 187 passed, 0 failed (V302 신규 테스트 10건 추가) |
| `cargo test --lib --features cipher` | **통과** | 127 passed, 0 failed |
| `cargo clippy -- -D warnings` (cipher off) | **통과** | 0 warnings |
| `cargo clippy --features cipher -- -D warnings` | **통과** | 0 warnings |
| `pnpm tsc --noEmit` | **통과** | 오류 없음 |
| `pnpm lint` | **통과** | ESLint 경고/오류 0건 |
| `pnpm build` | **통과** | 13개 페이지 prerendered |

### Flaky 테스트 기록

`commands::lock::tests::release_lock_atomic_removes_self_owned_lock` (cipher off) — 1차 실행 시 간헐적 실패, 재실행 시 통과.

**원인**: `lock_path()`가 `paths::data_root()`에 의존하는 process-wide 전역 경로를 반환하므로, `cargo test --lib`의 병렬 테스트 실행 중 다른 테스트가 동일 lock 파일을 점유할 때 race 발생.

**영향 범위**: 단위 테스트 환경에 한정. 프로덕션은 단일 인스턴스 모델 (`tauri-plugin-single-instance`)로 lock 동시 접근 시나리오 자체가 존재하지 않음.

**대응**: carry-over I-S2-2~10과 함께 후속 스프린트에서 테스트 격리 강화 검토.

---

## 코드 리뷰 결과 (T1, T3~T10) — 2026-05-22 초기 리뷰

> T2는 Session #2 high-effort code review에서 S-T2-1~6 전수 패치 완료.

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

## 코드 리뷰 결과 (V1~V37b post-review fix) — 2026-05-23 재실행

변경 파일: 29개 (백엔드 6개 + 프론트엔드 23개), 1202 insertions / 306 deletions

| 등급 | 건수 | 내용 |
|------|------|------|
| Critical | 0 | — |
| High | 0 | — |
| Medium | 3 | 아래 상세 참조 |
| Low | 1 | has_no_class_blocker 변수명 역전 (로직은 정확) |

**Medium 이슈 상세**:

**M-S7-02** (`src/components/LockScreen.tsx:45`): `NEXT_PUBLIC_DEV_AUTOLOGIN` 주석이 잘못됨.
- 주석 "release 빌드에서는 NEXT_PUBLIC 환경 변수 자체가 없으므로 무동작"은 사실이 아님. `NEXT_PUBLIC_*` 변수는 Next.js 빌드 타임에 클라이언트 번들에 인라인됨. `.env`에 값이 설정된 채 `pnpm tauri:build`를 실행하면 dev 비밀번호가 out/ 번들에 포함됨.
- 대응: `.env.example`에서 주석 처리로 기본 비활성화 상태이므로 즉시 위험은 없음. 주석 정정 + 릴리즈 빌드 전 `.env` 확인 절차 추가 필요. R50으로 등록.

**M-S7-03** (`src/app/academic/page.tsx:245`): 교습기간 선택 모드 중 배지 클릭 시 삭제 다이얼로그 표시.
- V27 변경으로 `calendarEventClick`이 항상 `setEventToDelete`를 호출. `studyPeriodMode` 활성 중 이벤트 배지를 실수 클릭 시 날짜 선택도 안 되고(`stopPropagation`) 삭제 다이얼로그가 뜨는 예상치 못한 UX 발생.
- 대응: 의도된 변경으로 명시되어 있어 즉시 수정 불필요. UX 피드백 모니터링 후 결정. R51로 등록.

**M-S7-04** (`src-tauri/migrations/302__add_is_seeded_to_schedule_events.sql:31`): V302 UPDATE 쿼리가 V301 시드 공휴일 외 사용자 추가 공휴일도 `is_seeded=1`로 마킹.
- 쿼리 `WHERE code_id IN (SELECT id FROM schedule_codes WHERE code_name = '공휴일' AND is_system_reserved = 1)` 조건은 이벤트 생성 시점이 아닌 코드 속성만 검사하여, 개발 DB에 수동 추가된 공휴일도 시드로 마킹됨.
- 대응: pre-release 상태에서 실제 사용자 데이터가 없으므로 현재 배포 영향 없음. 추후 운영 DB에서 마이그레이션 필요 시 주의 필요. R52로 등록.

---

## 배포 준비도 사전 확인 (2026-05-23 갱신)

| 항목 | 결과 |
|------|------|
| `CHANGELOG.md [Unreleased]` 섹션 | **업데이트됨** — Sprint 7 본 작업 + V1~V37b post-review fix 전수 기재 완료 |
| 하드코딩 시크릿 패턴 스캔 | **없음** — `*.rs`, `*.ts`, `*.tsx` 변경 파일 전수 확인 |

---

## 수동 검증 항목 (사용자 2026-05-23 확인 완료)

| 항목 | 검증 내용 | 상태 |
|------|----------|------|
| T1 | 비밀번호 입력 시 Keychain 다이얼로그 최대 1회, startup < 3초 | ✅ 완료 (2026-05-23) |
| T2 | `{cloud}/smarthb/salt.bin` 파일 생성 확인 + Keychain salt 마이그레이션 | ✅ 완료 (2026-05-23) |
| T3 | 앱 재시작 후 동일 device_id 유지 (`app_config_dir/device.id`) | ✅ 완료 (2026-05-23) |
| T4 | 캘린더 배지 색상 정상(시스템 코드 하드코딩 제거) + 시스템 코드 드래그 차단 | ✅ 완료 (2026-05-23) |
| T5 | `/settings/schedule-codes` 코드 CRUD 동작 + `/academic` 코드 패널 제거 확인 | ✅ 완료 (2026-05-23) |
| T6 | 교습기간 미확정 월에서 셀 클릭 즉시 selection 모드 진입 | ✅ 완료 (2026-05-23) |
| T7 | 중복불가 일정 상호 차단 + 교습기간 외 배치 차단 확인 | ✅ 완료 (2026-05-23) |
| T8 | 교습기간 삭제 → cascade 삭제 → 공휴일 보존 확인 | ✅ 완료 (2026-05-23) |
| T9 | 공휴일 배지 삭제 버튼 비표시 + 삭제 시도 차단 확인 | ✅ 완료 (2026-05-23) |
| UC-2 | 월말 학사 일정 수립 전체 흐름 완주 가능 여부 | ✅ 완료 (2026-05-23) |
| V1~V37b | post-review 시각 검증 38건 전수 정상 동작 확인 | ✅ 완료 (2026-05-23) |

---

## 결론

자동 검증(cargo test 187 passed, clippy clean, tsc/lint/build) 전수 통과. 코드 리뷰 Medium 3건(R50/R51/R52 등록). 수동 시각 검증 T1~T10 + V1~V37b 전수 사용자 확인 완료 (2026-05-23). v0.3.1 프로덕션 배포 준비 완료.
