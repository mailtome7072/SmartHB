# Test Report — Sprint 5 (2026-05-22)

> 검토 대상: sprint5 브랜치 (e717a78) → develop 머지 + sprint-review 수정 커밋 (89431ff)
> 변경 파일: 9개 (백엔드 3개, 프론트엔드 2개, 마이그레이션 1개, 문서 1개, lock 파일 2개)

---

## 코드 리뷰 결과 요약

| 등급 | 건수 | 내용 |
|------|------|------|
| Critical | 0 | 없음 |
| High | 0 | 없음 |
| Medium | 1 | lock/page.tsx 에러 화면 재시도 버튼 누락 (A20 등록) |
| Low | 1 | lock/page.tsx refresh() 후 순간 스테일 렌더링 가능성 (A20에서 동시 해소 예정) |

### Medium 이슈 상세

**파일**: `src/app/lock/page.tsx` (line 50)

`checkLockStatus()` IPC 호출 실패 시 에러 메시지 박스만 렌더되고 재시도 버튼이 없다. 앱 시작 직후 일시적 IPC 타이밍 오류가 발생할 경우 사용자가 앱을 강제 종료해야만 복구할 수 있다. Sprint 6 초반 또는 hotfix로 처리 예정 (A20).

---

## 자동 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| cargo test | ✅ 통과 | 130건 통과 (sprint-review에서 3건 수정 후) |
| cargo clippy | ✅ 통과 | 경고 0건 |
| pnpm tsc --noEmit | ✅ 통과 | 타입 오류 0건 |
| pnpm lint | ✅ 통과 | ESLint 경고/오류 0건 |
| pnpm build | ✅ 통과 | Next.js static export 성공 |

### 테스트 초기 실패 및 수정 내역

sprint-review 단계에서 cargo test 최초 실행 시 3건 실패:

| 테스트 | 실패 원인 | 수정 내용 |
|--------|-----------|-----------|
| `db::tests::in_memory_pool_runs_migrations` | V104 시드 5건 가정 → V201 적용 후 4건 | 검증 값 `5` → `4` |
| `fees::tests::match_fee_returns_exact_match` | 4시간 금액 250000 가정 → V201 이후 200000 | 기대 금액 수정 |
| `fees::tests::weekly_hours_unique_violation_returns_korean` | 2시간 행이 V201에서 삭제됨 → INSERT 성공 | 3시간 행으로 변경 |

수정 커밋: `89431ff` — 3건 모두 V201 시드 변경 반영.

**근본 원인**: V201 마이그레이션 작성 시 관련 테스트를 동일 커밋에 함께 업데이트하지 않음.
**대응**: A19 액션 아이템으로 규칙화 (CLAUDE.md 또는 backend.md 명문화).

---

## 배포 준비도 사전 확인

| 항목 | 결과 |
|------|------|
| CHANGELOG.md [Unreleased] 섹션 | ✅ Sprint 5 변경사항 기재됨 |
| 하드코딩된 시크릿 패턴 스캔 | ✅ 발견 없음 |

---

## 수동 검증 항목

- ⬜ `pnpm tauri:dev` 실행하여 앱 동작 수동 확인 (DEPLOY.md 스테이징 검증)

---

## 결론

자동 검증 5개 항목 전부 통과 (sprint-review 초기 테스트 실패 3건 수정 완료). 코드 리뷰 Critical/High 없음. Medium 1건(재시도 버튼 누락)은 위험도 낮으나 A20으로 추적 관리.
