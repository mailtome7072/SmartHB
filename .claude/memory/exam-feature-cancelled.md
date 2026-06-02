---
name: exam-feature-cancelled
description: Phase 5 전체(단원평가 /exams + 학습보고서 /reports) 개발 전면 취소. 다음 sprint 계획에서 Phase 5 제외
metadata: 
  node_type: memory
  type: project
  originSessionId: d4a8a2df-8629-4fbd-a1c6-d51d1d4eaaeb
---

**결정(2026-05-31)**: **단원평가('단원 평가' `/exams`) + 학습보고서('학습 보고서' `/reports`) 개발 요구사항·개발을 전면 취소**한다. → 사실상 **Phase 5 전체 취소**.
사용자(원장) 지시. 다음 sprint 계획 수립 시 반드시 반영.

**Why:** 운영 상 불필요하다고 판단 — 더 이상 개발하지 않기로 함. (단원평가 취소 → 단원평가 점수에 종속된 학습보고서도 함께 취소)

**How to apply (다음 계획·문서 반영):**
1. **Phase 5 전체 제외**: ROADMAP 기준 Phase 5 = 단원평가(Sprint 13) + 학습보고서(Sprint 14). **둘 다 계획에서 제외**. sprint-planner 진입 시 Phase 5 통째로 드롭하고 Phase 6(대시보드+유틸) 등으로 일정 재정렬.
2. **메뉴 처리**: `src/lib/menu-config.ts` 의 '단원 평가'(`/exams`)·'학습 보고서'(`/reports`) 두 항목(현재 `disabledHint: 'Phase 5 에서 제공'`)을 **제거** 결정 — 계획 시 확정.
3. **문서 정리(계획 시)**: PRD §4.8(학습보고서)·§6.1 분기 보고서 도메인, AC-4.8-*, frontend.md/backend.md 의 단원평가·학습보고서 관련 제약을 폐기/취소선 처리(마감 폐기 때와 동일 방식).
4. DB·IPC 신규 없음 — 미구현 상태라 제거할 코드 거의 없음(메뉴 항목 + 문서 위주).

관련: [[sprint13-pin-optional]] — Phase 5 취소로 공지문(Sprint 12) 이후 가용 작업은 Phase 6(대시보드·가져오기/내보내기·자가진단)로 이동.
