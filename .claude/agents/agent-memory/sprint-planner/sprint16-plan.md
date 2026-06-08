---
name: sprint16-plan
description: "Sprint 16 계획 요약(rev2) — Phase 6 마지막. 격식UAT 폐기→바로 실사용 개시. DB폴더변경+salt.bin이전 MUST 확정. Sprint 15 이연(양OS빌드/양PC동기화) + CSV가져오기 + 피드백 대응. 배포는 사용자 지시 대기."
metadata:
  type: project
---

**Sprint 16 (2026-06-09 ~ 06-22)**: Phase 6 마지막 스프린트. v1.0 직행 확정(v0.6.0 폐기).

**핵심 Task (MUST, 36h)**:
- T0: 회고 액션(A99 Ctrl+N 방어, A100 미저장 경고, Ctrl+S) 3h
- T1: CSV 가져오기(실사용 데이터 이관, encoding_rs 신규 의존성) 6h
- T2: DB 폴더 변경 + salt.bin 이전(PI-16 확정, copy-then-switch, ADR) 8h
- T3: 양 OS 빌드 검증(Sprint 15 이연 T7) 4h
- T4: 양 PC 동기화(Sprint 15 이연 T8) 3h
- T5: 실사용 개시 준비(양 OS 설치+데이터 이관 확인) 2h
- T6: 초기 실사용 피드백 대응 버퍼 4h
- T9: v1.0 릴리즈 준비(CHANGELOG, 버전 1.0.0) 3h
- T10: 통합 검증 3h

**SHOULD (7h)**: T7(접근성 44px/F1 4h), T8(공지문 I/O 병렬화 3h)
A89 notices UI 구획화 → Post-MVP 이연 확정.

**확정(PI)**: PI-16(DB 폴더 변경 → Sprint 16 MUST, 2026-06-08), PI-18(격식 UAT 폐기 → 바로 실사용 개시, 2026-06-08)
**미결정(PI)**: PI-17(출결표 N+1 → 실측 후), PI-19(셀 memo)

**배포 금지**: deploy-prod는 사용자 명시 지시 전까지 절대 진행하지 않음.

관련: [[exam-feature-cancelled]], Phase 5 취소로 /exams, /reports 일절 미포함.
