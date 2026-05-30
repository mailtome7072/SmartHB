---
name: exam-feature-cancelled
description: 단원평가 메뉴(/exams) 관련 개발 요구사항·개발 전면 취소 결정. 다음 sprint 계획 시 Phase 5 범위에서 제외
metadata: 
  node_type: memory
  type: project
  originSessionId: d4a8a2df-8629-4fbd-a1c6-d51d1d4eaaeb
---

**결정(2026-05-31)**: **단원평가('단원 평가' 메뉴, `/exams`) 관련 개발 요구사항과 개발을 전면 취소**한다.
사용자(원장) 지시. 다음 sprint 계획 수립 시 반드시 반영.

**Why:** 운영 상 불필요하다고 판단 — 더 이상 개발하지 않기로 함.

**How to apply (다음 계획·문서 반영):**
1. **Phase 5 범위 축소**: ROADMAP 기준 Phase 5 = 단원평가(Sprint 13: 점수 입력+추이 조회) + 학습보고서(Sprint 14). 이 중 **단원평가 부분을 계획에서 제외**한다. sprint-planner 진입 시 Phase 5 재정의.
2. **메뉴 처리**: `src/lib/menu-config.ts` 의 '단원 평가'(`/exams`, 현재 `disabledHint: 'Phase 5 에서 제공'`) 항목을 **제거**(또는 영구 비활성)로 결정 — 계획 시 확정.
3. **⚠️ 학습보고서 종속성 확인 필수**: PRD §4.8.3 — 분기 학습보고서는 **단원평가 점수에 직접 참조**(점수표·추이 차트). 단원평가를 취소하면 학습보고서가 데이터 소스를 잃는다. → **학습보고서(§4.8)도 같이 취소할지, 아니면 점수 입력만 별도 경량 유지/재정의할지** 다음 계획 때 사용자와 확정해야 함. (사용자는 단원평가만 명시 취소 — 학습보고서는 미언급)
4. PRD §4.8(학습보고서)·§6.1 분기 보고서 도메인, frontend.md/backend.md 의 단원평가·학습보고서 관련 제약은 계획 확정 후 정리.

관련: [[sprint13-pin-optional]] (PIN 옵션화도 Sprint 13+ 예정 — 단원평가 취소로 Phase 5 일정 재조정 여지)
