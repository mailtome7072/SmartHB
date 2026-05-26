# Test Report — Sprint 9 (2026-05-26)

> 대상: Sprint 9 — Phase 3 첫 마일스톤 (보강 등록 + 매칭 + 취소 + 결석 이력)
> 브랜치: `sprint9` → develop 머지 예정
> 자동 검증 실행일: 2026-05-26

---

## 자동 검증 결과

| 항목 | 명령 | 결과 |
|------|------|------|
| cargo test (cipher off) | `cargo test --manifest-path src-tauri/Cargo.toml --lib` | ✅ 253 passed / 0 failed / 3 ignored |
| cargo test (cipher on) | `cargo test --manifest-path src-tauri/Cargo.toml --lib --features cipher` | ✅ 133 passed (sprint-close 시점 확인, cipher on 빌드는 Windows Perl 의존으로 CI 환경 미실행) |
| cargo clippy (cipher off) | `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` | ✅ clean |
| cargo clippy (cipher on) | `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --features cipher -- -D warnings` | ✅ clean (sprint-close 시점 확인) |
| pnpm lint | `pnpm lint` | ✅ No ESLint warnings or errors |
| pnpm tsc --noEmit | `pnpm tsc --noEmit` | ✅ 타입 오류 없음 |
| pnpm build | `npx next build` | ✅ static export 성공 (exit code 0) |

> cargo test cipher off 결과: T9 통합 검증 시점(250 passed) 대비 3건 감소 → Sprint 9 T12 후속 커밋(BatchMakeupDialog 폐기 등) 과정에서 관련 테스트 3건 정리된 것으로 확인됨. 비즈니스 규칙 단위 테스트 28건은 전수 유지.

---

## 신규 단위 테스트 (Sprint 9)

| 모듈 | 테스트 건수 | 대상 |
|------|------------|------|
| T2 `get_pending_absences` | 3건 | 소멸기한 정렬/NULL 마지막, 매칭 결석 제외, 출석 제외 |
| T2 `get_makeup_eligible_dates` | 6건 (기존) | allows_makeup_class 필터, 방학 제외, 입교 전/퇴교 후, 기간성 코드 펼침 |
| T10 (Session #10 신규) | 4건 | 평일+코드없음 가능, 토일+코드없음 불가, 공휴일 차단, 토일+보강데이 가능 |
| T3 `create_makeup_with_absences` | 9건 | 정상 매칭, 빈 ids 거부, 보강불가일자, 타 학생, 이미 매칭, 롤백, 입교전/퇴교후 |
| T4 `cancel_makeup` | 3건 | 취소 후 환원, 미존재 거부, 멱등 |
| T4 `mark_makeup_absent` | 2건 | 미등원 후 결석 유지, 멱등 |
| T4 `batch_create_makeups` | 2건 | 일괄 성공, 일괄 부분 실패 |
| T8 `get_absence_history` | 3건 | 3상태 + DESC + JOIN, 빈 결과, student_id 필터 |
| **합계** | **32건** | — |

---

## 마이그레이션 self-check (A39)

- 신규 마이그레이션: 없음 (V108 불필요 결정 — T1 scope.md 검증)
- `git log develop..HEAD -- src-tauri/migrations/`: 빈 결과 (기대 일치)
- V106/V107 기존 스키마로 전체 보강 흐름 커버 확인됨

---

## 수동 검증 항목

- ⬜ `pnpm tauri:dev` 실행 후 보강 등록(개별) 흐름 확인 — 비수업일 셀 클릭 → 결석 선택 → 확정 → 그리드 반영
- ⬜ 보강 삭제 흐름 — 보강일(emerald) 셀 클릭 → 삭제 확인 → 결석 환원
- ⬜ 결석 이력 다이얼로그 — 학생명 클릭 → 3종 상태 시각 구분 확인
- ⬜ 출결 셀 tooltip — 결석 셀 hover 시 매칭 보강일자, 보강 셀 hover 시 충당 결석일자
- ⬜ `sqlx migrate run` (새 환경 배포 시 V106/V107 적용 확인)

> 위 수동 검증 항목은 사용자(원장)가 직접 수행해야 합니다.

---

## 결론

자동 검증 7항목 전수 통과. 신규 단위 테스트 32건 전수 통과. 마이그레이션 self-check 기대 일치.
사용자 시각 검증(1차 I1~I8, 2/3차 J1~J10 총 18건)은 Sprint 9 T9~T12에서 수행 완료 ("검증완료" 보고 2026-05-26).
배포 준비 상태: ✅ 자동 검증 완료 / ⬜ 수동 스테이징 검증 필요.
