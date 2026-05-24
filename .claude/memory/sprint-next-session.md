---
name: sprint-next-session
description: "Sprint 8 Session #9 완료 (T1~T9 자동 검증, 9/9). 다음: 사용자 시각 검증 후 sprint-close 실행"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint8-session9-t9
---

Sprint 8 출결 관리 — **모든 T1~T9 자동 작업 완료**. 사용자 시각 검증 + sprint-close 대기.

## Sprint 8 진행 현황

| Task | 내용 | 상태 |
|------|------|------|
| T1 | V106 마이그레이션 | ✅ `f72778b` |
| T2 | 출결 생성 IPC | ✅ `366f880` |
| T3 | 출결 조회 + 토글 IPC | ✅ `4efc570` |
| T4 | 출결표 프론트엔드 UI | ✅ `0a20c18` |
| T4 follow-up | UX 보강 | ✅ `516758c` |
| T5 | 보강필요시간/소멸기한 단위 테스트 100% | ✅ `5f2f0fd` |
| T6 | carry-over High 4건 (R40~R43) | ✅ `14b9bfb` |
| T7 | carry-over Medium-High (R45) Keychain race | ✅ `e89c3a8` |
| T8 | carry-over Medium 6항목 | ✅ `069f435` |
| T9 | 통합 검증 (자동 7항목 통과) | ✅ `cda2745` |

검증 상태: cargo test cipher off **221 passed** / cipher on **133 passed** / clippy clean / pnpm lint/tsc/build clean.

## Session #9 (T9) 검증 결과

| 항목 | 결과 |
|------|------|
| cargo test cipher off | ✅ 221 passed |
| cargo test cipher on | ✅ 133 passed |
| clippy off + on | ✅ clean |
| pnpm lint | ✅ clean |
| pnpm tsc --noEmit | ✅ clean |
| pnpm build | ✅ static export 성공 (out/ 정상) |

## 다음 액션 (선택)

### A. 사용자 시각 검증 후 sprint-close
1. `pnpm tauri:dev` 로 앱 기동 → sprint8.md L353-360 항목별 확인
2. `docs/sprint/sprint8/scope.md` Session #9 사용자 검증 표에 ✅ 마킹
3. sprint8.md AC-T9-2/T9-3/T9-4 도 ✅ 마킹
4. 사용자 명령: `"sprint8 구현 완료했어. sprint-close 실행해줘."`

### B. 시각 검증 생략하고 바로 sprint-close
시각 검증을 다음 세션으로 미루고 문서화 + PR 생성만 진행.

## sprint-close 후 흐름

1. **sprint-close**: ROADMAP 업데이트 + (PR 단계 생략 정책상 PR 미생성 — 직접 머지) + CHANGELOG / DEPLOY.md 업데이트
2. **sprint-review**: 코드 리뷰 + 자동 검증 + 회고 작성
3. **deploy-prod**: develop QA 통과 후 main merge + `v{version}` 태그 push

## 잔여 후속 task (Sprint 8 범위 외)

- **R48-b**: salt buffer ZeroizeOnDrop 시그니처 광범위 변경 — `Zeroizing<[u8; SALT_LEN]>` 또는 wrapper struct 도입. 별도 후속 sprint task
- **반응형 폰트/셀 너비**: `--font-size-body: 18px` + h1~h6 + `AttendanceGrid` 셀(140/62/84px) 모두 px 고정 → 모니터 해상도 비례 조정 안 됨. `clamp()` viewport 또는 html font-size 미디어쿼리 + rem 전환. 폰트 변경 시 셀 너비도 동기 필요. ROADMAP.md Sprint 8 "차기 sprint 이연 후보" 에 기록됨

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지 (`gh pr create` 금지)
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지
- **사용자 메모리 미러 동기화 필수** — `.claude/memory/sprint-next-session.md` 동시 갱신
