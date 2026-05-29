---
name: sprint-next-session
description: "post-Sprint 11 hotfix 다수 + 3건 일괄처리 사용자 검수 대기 중. 다음 세션: 검수 + deploy-prod"
metadata:
  node_type: memory
  type: project
  originSessionId: post-sprint11-pending-user-review
---

Sprint 11 완료 (develop 머지 `dfc5925`) 이후 사용자 시각 검증 진행 중. 다수 hotfix가
develop 에 적용됐고, **2026-05-29 진행 마지막 commit (`419ea36`) 의 3건 일괄처리는
사용자 검수 대기 상태** — 다음 세션 진입 시 가장 먼저 확인.

## ⏳ 다음 세션 시작 시 — 사용자 검수 대기 (3건)

**commit**: `419ea36 feat(billing): 수납완료 라벨 + 청구/수납 상태 필터 라디오`

| # | 검수 항목 | 확인 위치 | 기대 동작 |
|---|---------|---------|---------|
| 1 | 청구 그리드 **수납완료 라벨** | `/billing` 청구 탭의 청구 행 상태 컬럼 | `Bill.isPaid=true` 청구 옆에 emerald 배지 "수납완료" 표시 |
| 2 | 청구 탭 **상태 필터** | `/billing` 청구년월 우측 (검색 input 옆) | 라디오 3개: 전체/확정/미확정. 디폴트 전체. 선택 시 청구 그리드 필터 |
| 3 | 수납 탭 **상태 필터 + 수납완료 행 표시** | `/billing` 수납 탭 동일 위치 | 라디오 3개: 전체/수납완료/미수납. 디폴트 전체. 수납완료 행은 read-only(✓ + 날짜·입금자·결제수단·카드사 텍스트) + emerald-50 배경 |

검수 시작 시: `pnpm tauri:dev` 실행 → `/billing` 진입 → 청구 데이터 생성 후 일부 수납 처리하여 위 3건 확인.

검수 통과 시 → deploy-prod 진입. 미통과면 후속 hotfix.

## develop 누적 hotfix 요약 (Sprint 11 머지 후 → 현재)

| 커밋 | 내용 |
|------|------|
| `419ea36` ⏳ **검수 대기** | 수납완료 라벨 + 청구/수납 상태 필터 라디오 + PaymentsView refactor (list_payment_view IPC) |
| `b7a59d1` | 청구·수납 통합 검색 + 미수납 자동 채움 (search_students_for_billing + 입금일=오늘) |
| `3398e3c` | 메뉴 '청구 관리' → '청구/수납 관리' |
| `c1fbfdd` | V110 마이그레이션 — payment_methods 중복 시드 정리 (other/locCash 삭제) |
| `33f3962` | docs: R82/A80 — 마감 후 추가 청구 정책 carry-over |
| `4871e2b` | 출결 '추가 출결 데이터 생성' 버튼을 검색 input 우측으로 이동 |
| `cd1a233` | 출결 추가 데이터 생성 UX (generate_attendances INSERT OR IGNORE + count_ungenerated IPC) |
| `9244034` | 디버그 정리 — ErrorDialog 정상 동작 확인 후 |
| `b0f760a` | ErrorDialog autoFocus 제거 — Enter 키 즉시 닫기 race 해소 (핵심 fix) |
| `7d89a0c` | 인라인 빨간 박스 → ErrorDialog 모달 (createPortal + inline style) |
| `9141658` | mutation onMutate 에서 setError(null) 자동 클리어 |
| `f5a9cc5` | error 박스 빈 문자열 렌더 차단 |
| `67d05ae` | 청구년월 디폴트 = 오늘 / "추가 청구 데이터 생성" 버튼 + 총수업원생수 표시 (BillingSummary.totalBillableStudents) |
| (이전) `dfc5925` | Sprint 11 완료 머지 |

## 핵심 학습 (메모리 후보)

1. **ErrorDialog autoFocus + Enter 키 race**: confirm 버튼 autoFocus → Enter 키의 KeyUp 이 자동 click 트리거 → 모달이 떴다 즉시 닫힘. 해결: autoFocus 제거. 일반 패턴이라 별도 메모리 등재 검토.
2. **createPortal + inline style**: AppShell 의 main.overflow-y-auto stacking context 우회. Tailwind JIT 누락 위험 차단.
3. **dev 서버 HMR 갱신 지연**: 새 컴포넌트 추가 후 사용자 환경에서 강제 새로고침 필요할 수 있음.

## Sprint 12 carry-over (R82/A80 외)

- A69: F1 `CloseMonthDialog` summaryQuery 의존 게이팅
- A70: F3 PaymentsView dirtyEntries payerName-only 소실
- A71: 청구 50명 3초 이내 실측
- **A80: 마감 후 추가 청구 정책 결정** ← post-Sprint 11 신규 등록
- (외 Sprint 10 이연 항목들 A74~A79)

## 다음 단계 (검수 통과 후)

1. **deploy-prod** — develop → main 머지 + 다음 버전 태그 (`v0.5.1` 또는 `v0.6.0`)
2. **Sprint 12 계획** — `phase-planner` 또는 `sprint-planner` 진입
   - Phase 4 마무리: 공지문 이미지 생성 (PRD §4.10)
   - 대시보드 위젯 §4.11.3 본격화
   - A80 마감 정책 결정 포함

## 정책 (재확인)

- **PR 단계 생략** — develop 머지 / develop → main 머지 모두 직접 ([[workflow-no-pr]])
- **메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
- 사용자 메모리 미러: `/Users/skyang/.claude/projects/-Users-skyang-Projects-SmartHB/memory/`
