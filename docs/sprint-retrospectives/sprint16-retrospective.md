# Sprint Retrospective Sprint 16

> 대상: Sprint 16 (develop...sprint16) — v1.0.0 정식 출시 스프린트
> 리뷰 일자: 2026-06-12
> 코드 리뷰: Critical 0 / High 0 / Confirmed 2건(A6·C2, 즉시 수정) / Plausible 4건 + Low 2건(이연)
> 자동 검증: cargo test 417 passed (cipher off) / clippy --all-targets clean / cargo check --features cipher OK / pnpm lint clean / pnpm tsc clean / pnpm build 성공

---

## 이전 회고 액션 아이템 이행 결과

출처: `docs/sprint-retrospectives/sprint15-retrospective.md`

| 항목 ID | 항목 | 이행 여부 | 비고 |
|---------|------|-----------|------|
| A98 | self-verify 명령에 `--all-targets` 추가 | ✅ 완료 | Sprint 15 close 시 CLAUDE.md·harness-engineering.md 교정 완료 |
| A99 | `GlobalShortcuts` Ctrl+N 입력 필드 방어 로직 | ✅ 완료 | T1에서 INPUT/TEXTAREA/SELECT tagName 가드 추가 |
| A100 | 미저장 이탈 경고 공통 훅 `useUnsavedChanges` 구현 | ✅ 완료 | T1에서 beforeunload + 사이드바 guard 연결. 교습소 정보 화면(R105) 포함 적용 |
| A96 | 복원 리허설 dev 환경 개선 | ⏸️ 이연 | Low 우선순위. Post-MVP backlog. |

---

## 잘한 점

**v1.0.0 정식 출시 — T0(수업일 변경) 최우선 추가에도 전체 스프린트 완주**

Sprint 계획 수립 당일 사용자 이슈(수업일 변경 2종)가 긴급 추가되어 MUST 합계가 44h로 가용(40h)을 초과했다. 그러나 SHOULD 2건(T8·T9)을 Post-MVP로 전량 이연하고 T0에 집중한 전략이 적중 — 케이스1(1회성 이동 + V306/V307 마이그레이션), 케이스2(날짜 인식 generate 리팩토링 + 재생성 원자화), 실사용 피드백 대응 7건, 코드리뷰 P0/P1/P2 전수 반영을 포함해 출시 전 완주했다.

**v1.0 출시 직전 전체 코드리뷰로 P0 7건 + P1 11건 선제 제거**

출시 전에 6인 전문가 코드리뷰(`docs/code-review/full-review-2026-06.md`)를 통해 WAL 체크포인트 부재, config.json fsync 누락, UTC 날짜 오기입, 수납 draft 무경고 소실 등 실사용 시 데이터 손실이 발생할 P0 7건을 sprint16에서 모두 반영했다. "치명 버그 없음, SQL 인젝션 0건, invoke() 직접 호출 0건"이라는 종합 평가는 스프린트 초기부터 쌓아온 설계 원칙의 결과다.

**sprint-review 코드리뷰에서 Confirmed 2건을 출시 전 당일 수정·원자성 테스트 추가**

`change_data_folder_impl`의 pool 미교체(A6)는 재시작 전 구 DB 오기입 위험이었고, CSV import 부분 삽입(C2)은 중간 실패 시 데이터 불완전 상태가 남는 문제였다. 두 건 모두 sprint-review 당일 수정(pool close 추가 + insert_student_tx 헬퍼 추출 + 원자성 단위 테스트 2건)했고 develop 머지까지 완료하여 배포 게이트를 충족했다.

**실사용 시각검수를 체계적으로 병렬 수행**

sprint16은 수업일 변경(T0), CSV 가져오기(T2), DB 폴더 변경(T3), 백업 복원, 청구/수납 분리, 공지문 보강 등 기능 구현 직후 사용자 시각검수를 세션 단위로 병렬 수행했다. P2 선별 7건(원생 폼 UX 포함)이 시각검수에서 발견되어 출시 전 반영됐다.

**generate_impl 날짜 인식 리팩토링이 근본 해결**

기존 `load_weekly_schedule`이 `effective_to IS NULL`(현행 스케줄만)을 보는 구조를 `effective_from ≤ d AND (effective_to IS NULL OR d < effective_to)` 날짜 인식으로 교체함으로써, 케이스2뿐 아니라 기존 출결 생성 로직 전체가 스케줄 이력을 올바르게 인식하는 근본 개선이 이루어졌다. 밴드에이드가 아닌 인프라 수준의 수정이다.

---

## 개선할 점

**cipher 실동작은 dev 환경에서 검증 불가 — 첫 실검증이 배포 산출물(.exe)에서 일어난다**

SQLCipher 기반 백업/복원, DB 무결성, DB 폴더 변경은 `cipher` Cargo feature가 활성화된 프로덕션 빌드에서만 동작하고, dev 환경(`cargo test` 기본)은 stub으로 처리된다. 따라서 이번 sprint-review 자동 검증(`cargo test` cipher off, `cargo check --features cipher`)으로는 cipher 실동작이 검증되지 않았다. 배포 인스톨러(.exe) 설치 후 스모크 테스트가 첫 실검증이 된다 — 배포 후 즉시 수행 필요.

**로컬 cipher release 빌드가 git-bash MSYS perl 충돌로 실패**

`cargo build --release --features cipher`를 git-bash에서 실행하면 Strawberry Perl이 아닌 MSYS perl이 OpenSSL 빌드에 개입해 충돌한다. PowerShell에서는 Strawberry Perl이 우선되어 성공한다. 로컬 release 빌드가 필요할 때는 PowerShell 전용임을 팀이 인식해야 한다. 단, 인스톨러 빌드는 GitHub Actions CI가 담당하므로 로컬 release 빌드는 실제로 불필요하다. 이 교훈을 `docs/setup-guide.md`에 기록할 것.

**`release_lock_atomic_is_idempotent_when_no_file` 테스트의 잠재적 flake**

T11 통합검증·sprint-review 재실행 모두에서 재현되지 않았으나, 이 테스트는 전역 락 경로(`app.lock`)를 공유한다. 병렬 테스트 실행 환경(`cargo test -- --test-threads=N`)에서 다른 테스트가 동일 경로를 건드리면 간헐 실패 가능성이 있다. 현재 `cargo test` 기본은 단일 프로세스 병렬 실행이라 재현 빈도가 낮지만, CI 확장 시 발현 가능성이 있다.

**notices/page.tsx가 구형 guard 패턴을 유지 — UnsavedNavDialog와 아키텍처 불일치**

`notices/page.tsx`는 `setUnsavedNavTarget` 대신 자체 `pendingAction` 모달로 미저장 이탈 경고를 처리한다. 현재는 사용자에게 기능적으로 동작하나, 향후 `UnsavedNavDialog` 동작 변경 시 notices만 자동으로 따라가지 않는다. `useUnsavedChanges` 훅으로 마이그레이션이 필요하다.

**MoveAttendanceDialog의 setSubmitting finally 누락**

`handleSelect` 성공 분기에서 `finally` 없이 `setSubmitting(false)`가 catch에만 존재한다. `void invalidateQueries`가 내부에서 throw하면 submitting=true가 잔존하여 달력 전체가 비활성 상태에 빠질 수 있다. v1.0 이후 개선 backlog에 등록.

**T4(양 OS 빌드 검증), T5(양 PC 동기화 시나리오)는 물리 환경 의존으로 미완료**

Sprint 15에서 이연됐던 T4·T5는 이번에도 물리 환경(교습소 PC, 양 PC 전환 시나리오) 의존으로 자동 검증 외 수동 검증 항목으로 남았다. 배포 인스톨러 설치 단계에서 함께 수행 예정.

---

## 액션 아이템

| ID | 항목 | 우선순위 | 대상 위치 | 기한 |
|----|------|----------|-----------|------|
| A101 | 배포 후 cipher 실동작 스모크 테스트 — v1.0.0 인스톨러(.exe/.dmg) 설치 직후: ① 앱 시작 → integrity_check 통과 ② 백업 생성(exit/hourly) 확인 ③ 수동 복원 흐름 ④ DB 폴더 변경 후 정상 기동. 실패 시 hotfix 즉시 | High | 배포 후 수동 검증 | deploy-prod 완료 직후 |
| A102 | `release_lock_atomic_is_idempotent_when_no_file` 테스트 직렬화 검토 — `lock.rs` 테스트에 임시 고유 경로(`tempdir()`) 또는 `#[serial]` crate으로 전역 경로 공유 제거. 병렬 CI 환경에서 flake 발현 전에 선제 대응 | Medium | `src-tauri/src/commands/lock.rs` 테스트 블록 | 차기 안정화 스프린트 T0 |
| A103 | cipher release 빌드 환경 기록 — PowerShell 전용, git-bash 사용 금지 명시. `docs/setup-guide.md`에 "cipher 빌드 시 PowerShell 필수" 섹션 추가 | Low | `docs/setup-guide.md` | 차기 스프린트 착수 전 |
| A104 | `notices/page.tsx` 미저장 경고 `useUnsavedChanges` 훅으로 마이그레이션 — 자체 `pendingAction` 패턴 제거, `setUnsavedNavTarget` 경로로 일원화. P2-4 분리 작업과 묶어 진행 가능 | Medium | `src/app/notices/page.tsx` | P2-4 notices 분리 스프린트 시 |
| A105 | `MoveAttendanceDialog.handleSelect`에 `finally` 블록 추가 — `setSubmitting(false)` 를 try/catch/finally의 finally로 이동하여 void invalidateQueries throw 시 submitting 잔존 방지 | Medium | `src/components/attendance/MoveAttendanceDialog.tsx:~90` | 차기 안정화 스프린트 |
| A106 | P2 1차 안정화 스프린트 범위 확정 — 실사용 피드백 2주 수집 후 P2-1(출결 그리드 N+1), P2-2(에러 한글화), P2-8(Undo·피드백 통일) 우선순위 최종 결정. `docs/code-review/full-review-2026-06.md` P2 목록을 ROADMAP.md Post-MVP 섹션에 이관 | Medium | ROADMAP.md, `full-review-2026-06.md` | 실사용 2주 후 sprint-planner 계획 시 |
