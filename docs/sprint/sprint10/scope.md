---
Sprint: 10  |  Date: 2026-05-26  |  Session: #1
---

> Sprint 10 Session #1 — T1 (Sprint 9 dead code 정리).
> 예상 2h. 단순 삭제 작업 + 단위 테스트 제거.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1** | `mark_makeup_absent` + `batch_create_makeups` 폐기 코드 완전 제거 (A49 carry-over) | 2h |

> Sprint 9에서 사용자 결정으로 폐기되었으나 dead code 상태로 남은 항목 정리.

---

## T1 작업 범위

### 백엔드 정리
1. `src-tauri/src/commands/makeup.rs`:
   - `mark_makeup_absent` 함수 + IPC `#[tauri::command]` 핸들러 삭제
   - `batch_create_makeups` 함수 + IPC 핸들러 삭제
   - 관련 payload struct(BatchMakeupEntry 등) 삭제
   - 관련 단위 테스트 삭제 (`mark_makeup_absent_*`, `batch_create_makeups_*` 등)
2. `src-tauri/src/lib.rs`:
   - `invoke_handler!`에서 `makeup::mark_makeup_absent`, `makeup::batch_create_makeups` 제거
3. `src-tauri/src/commands/audit.rs`:
   - `AuditEventType::MakeupAbsent` variant 삭제 (다른 참조 0건 확인 후)
   - 관련 직렬화 string 매핑 정리
4. (선택) V108 마이그레이션:
   - `makeup_attendances.status` CHECK 제약에서 `'makeup_absent'` 제거
   - 데이터 행 없음 (Sprint 9 J5에서 폐기, 운용 데이터 없음) — 안전한 변경
   - 본 세션에서는 코드 정리에만 집중, 마이그레이션은 별도 판단

### 프론트엔드 정리
5. `src/lib/tauri/index.ts`:
   - `markMakeupAbsent`, `batchCreateMakeups` 래퍼 이미 제거됨 (Sprint 9 T12) — 재확인만
6. `src/types/makeup.ts`:
   - `BatchMakeupEntry`, `BatchCreateMakeupsPayload`, `BatchFailure`, `BatchResult` 타입 이미 제거됨 — 재확인만

---

## 이번 세션에서 수정할 파일

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/makeup.rs | [0회] | 함수/IPC 핸들러/단위 테스트 삭제 |
| src-tauri/src/commands/audit.rs | [0회] | MakeupAbsent variant 삭제 |
| src-tauri/src/lib.rs | [0회] | invoke_handler 정리 |
| docs/sprint/sprint10/scope.md | [0회] | 본 파일 |

> 프론트엔드 파일은 Sprint 9 T12에서 이미 정리됨 — 확인만, 수정 없음 예상.

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [x] .github/workflows/ — CI/CD 파이프라인
- [x] SETUP.sh — 초기화 스크립트
- [x] src-tauri/migrations/ — 본 세션에서 마이그레이션 추가하지 않음 (V108은 별도 판단)
- [x] src/components/attendance/ — Sprint 9 UI는 이미 정리 완료

---

## 완료 기준 (이번 세션) — T1 AC

- ✅ `cargo test --manifest-path src-tauri/Cargo.toml` 251 passed / 0 failed (256 → -5, 삭제한 단위 테스트 5건과 일치)
- ✅ `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` clean
- ✅ dead code warning 0건
- ✅ TS 영향 없음 — Sprint 9 T12에서 이미 정리됨 확인

## 세션 종료 조건

- ✅ T1 AC 모두 통과
- ⬜ 단일 커밋
- ⬜ 사용자 메모리 미러 동기화 (Session #1 → #2 갱신)
- ⬜ 다음 세션(T2 — 소멸 자동 전이 설계 + 사용자 확인) 진입점 준비

## 실제 수정 결과

| 파일 | 변동 |
|------|------|
| src-tauri/src/commands/makeup.rs | -343 라인 (mark_makeup_absent 함수/impl + batch 함수/impl + payload struct 4종 + 단위 테스트 5건 + 모듈 헤더 주석 정리) |
| src-tauri/src/commands/audit.rs | -2 라인 (`MakeupAbsent` variant + string 매핑) |
| src-tauri/src/lib.rs | -2 라인 (invoke_handler 2건) |
| docs/sprint/sprint10/scope.md | 신규 — Session #1 |

## 발견된 이슈

(없음 — 진행 중 발견 시 기록)

## 다음 세션 (T2) 미리보기

- 소멸 전이 트리거 3개소 설계 (앱 시작 / 출결 생성 / 교습기간 등록)
- 소멸기한 판정 로직 확정
- 사용자 확인: 교습기간 미등록 월의 소멸 처리 방식 (A51 패턴)
- V108 마이그레이션 필요 여부 최종 판단

---

## carry-over

(Session #1 시작 시점에 carry-over 없음 — Sprint 9 완전 종료 + develop 머지 완료)
