---
name: sprint-next-session
description: "post-Sprint 11 develop 보완 다수 완료 + sprint-close/review 완료. 다음 세션: 수동검증 잔여 항목 후 deploy-prod 또는 Sprint 12 계획"
metadata:
  node_type: memory
  type: project
  originSessionId: post-sprint11-develop-batch-2026-05-30
---

Sprint 11 머지(`dfc5925`) 이후 develop 에 직접 보완을 계속 누적 중. **PR 단계 생략** ([[workflow-no-pr]]).
2026-05-30 세션에서 청구/수납·인증 대규모 보완 완료 + sprint-close/sprint-review 2회 완료.

## 2026-05-30 세션 커밋 (develop, 검증 완료)

| 커밋 | 내용 |
|------|------|
| `945e4a7` | 청구/수납 검수 후속 8건 (마감필터·건수표기·확정버튼버그·달력포커스·수납취소·결제수단필수·수납완료마감잠금) |
| `c93399e` | 앱 잠금 인증 **6자리 숫자 PIN** 전환 (ADR-007, validate_pin) |
| `70c59a1` | 청구 관리 **'월별 집계' 탭** (년/월 토글, 결제수단별 수납총액 열 배치, get_billing_period_stats IPC) |
| `c1ae063` | **청구 '마감(closed)' 개념 전면 폐기** (원장 결정). V111 마이그레이션 — bills 재구성(status 2단계, close_reason/closed_at 제거). PRD §4.9.7/AC 갱신 |
| `2a964b0` | 월별 집계 기간 선택을 청구 생성된 년월로 한정 (list_billed_months IPC) |
| `29fbe93` | 월별 집계 — 청구 0건 시 디폴트 년월=현재 년월 |
| `fb2a491` | review F1 — update_bill 존재확인+수납여부 1쿼리 통합 |
| (docs) | sprint-close/review 산출물 2회: CHANGELOG/ROADMAP/DEPLOY, test-report, risk-register(R83~R87), 회고 2건, code-review |

## 핵심 도메인 변경 (다음 세션 주의)

- **청구 상태 2단계**: 미확정(draft) → 확정(confirmed). **마감(closed) 폐기**. `close_billing_month`/CloseMonthDialog/CloseReasonDialog 제거됨. 다시 마감 언급 금지.
- **수납완료 잠금**: `payments.is_paid=1` 청구는 status 무관 금액 수정 거부(`update_bill_impl`) + 프론트 편집 비활성. (구 "마감+수납완료" 규칙 대체)
- **인증 = 6자리 숫자 PIN** (ADR-007). 보안 트레이드오프(10^6 키스페이스, 클라우드 DB 오프라인 브루트포스) 명시 수용. 복구코드 12자리 유지. 마이그레이션: dev 자격증명 재설정 필요(평문 DB라 데이터 무손실), prod 미출시.
- **마이그레이션 현황**: V111 추가 (bills 재구성). 다음 번호는 V112~ 또는 도메인 블록.

## ⏳ 다음 세션 시작 시 — 남은 일

1. **DEPLOY.md 수동 검증 잔여 항목** (sprint-review 가 ⬜ 로 추가):
   - 월별 집계 탭(년/월 토글·결제수단별 열), 드롭다운 생성년월 한정, 0건 현재년월 디폴트, 마감 제거 회귀, V111 적용 확인
   - (사용자는 세션 중 시각검증 완료 의사 밝힘 — DEPLOY.md 항목 ✅ 정리만 남았을 수 있음)
2. **Notion 업데이트 필요** (sprint-review 권고, 사용자 확인 후): 데이터 모델(V111 bills 재구성/status 2단계/close 컬럼 제거), 기능 명세(마감 폐기 §4.9.7). [[notion]] 규칙 — 페이지 ID 미입력 상태일 수 있음.
3. **deploy-prod** 또는 **Sprint 12 계획** 진입.

## carry-over (이전 리뷰)

- review F2 (Low): 월별 집계 년/월 토글이 radio 아닌 checkbox — **사용자가 '체크박스' 명시 요청**이라 의도된 설계(수정 안 함).
- A69/A70/A71/A80(마감 정책은 폐기로 무효화)/A74~A79 (Sprint 10~11 이연)
- A81(Medium, update_bill 트랜잭션 분리 — fb2a491 에서 1쿼리 통합으로 일부 해소), A82, A84

## 다음 단계 (배포)

1. **deploy-prod** — develop → main 직접 머지 + 다음 버전 태그. 누적 변경 규모상 `v0.6.0` 권장 (월별집계 신규 + 마감 제거 + PIN). `deploy-prod` 가 `[Unreleased]` → 버전 전환.
2. **Sprint 12 계획** — 공지문 이미지 생성(§4.10), 대시보드 위젯(§4.11.3).

## 정책 (재확인)

- **PR 단계 생략** — develop/main 머지 모두 직접 ([[workflow-no-pr]])
- **메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
- 사용자 메모리 미러: `/Users/skyang/.claude/projects/-Users-skyang-Projects-SmartHB/memory/`
