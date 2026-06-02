# Phase 5 전면 취소 + Sprint/Phase 번호 재매핑

> 기록일: 2026-06-02 (Sprint 13 계획 수립 시)

## 결정 사항

원장 결정(2026-05-31)으로 Phase 5(단원평가 /exams + 학습보고서 /reports) 개발 전면 취소.

## 번호 재매핑

| 변경 전 | 변경 후 | 내용 |
|---------|---------|------|
| Phase 5 (Sprint 13~14) | 취소 | 단원평가 + 학습보고서 |
| Sprint 13 | Sprint 13 (내용 변경) | 단원평가 -> PIN 인증 옵션화 + 취소 반영 |
| Sprint 14 (학습보고서) | 폐기 | 번호 소멸 |
| Phase 6 Sprint 15 | Phase 5 Sprint 14 | 대시보드 + 유틸 |
| Phase 7 Sprint 16 | Phase 6 Sprint 15 | 양 OS 빌드 + 최적화 |
| Phase 7 Sprint 17 | Phase 6 Sprint 16 | UAT + v1.0 릴리즈 |

## 총 스프린트 수

17 -> 16 (1개 폐기)

## 마일스톤 변경

- M6(평가+보고서) 취소
- M6 -> PIN 옵션화 (Sprint 13)
- M7(대시보드) -> Sprint 14
- M8(v1.0) -> Sprint 16

## 영향 받는 문서

- ROADMAP.md: Phase 5 취소 표기, Phase 6->5, Phase 7->6, Sprint 번호 재매핑, 의존성 맵, 마일스톤
- menu-config.ts: /exams, /reports 메뉴 제거 (Sprint 13 T1-a)
- PRD.md: SS4.7/SS4.8/SS6.1 폐기 표기 (Sprint 13 T1-c)
