---
name: project-status
description: SmartHB 프로젝트 현황 — 현재 스프린트, 기술 스택, 아키텍처 결정, 마이그레이션 현황
metadata:
  type: project
---

## 현재 스프린트
- Sprint 14: 대시보드 + 자가 진단 + 내보내기 (Phase 5, 구 Phase 6)
- 계획 문서: `docs/sprint/sprint14.md`

## 기술 스택
- Tauri 2 (Rust) + Next.js 15 (React 19) + SQLite (sqlx 0.8)
- UI: shadcn/ui + Tailwind CSS, Pretendard 18pt
- 상태: Zustand + TanStack Query
- 캘린더: FullCalendar (ADR-006)
- 차트: Sprint 14에서 Recharts 도입 예정 (대시보드 위젯용)

## 마이그레이션 현황
- 최신: V302 (`add_is_seeded_to_schedule_events`)
- Sprint 14 예정: V303 (`create_diagnosis_history`)
- 번호 정책: 100단위 블록 (V001-099 인프라, V101-199 도메인, V200-299 시드, V300+ 확장)

## 주요 마일스톤
- M7: 대시보드 (Sprint 14) -- 현재
- M8: v1.0 릴리즈 (Sprint 16)
- Sprint 15: 양 OS 빌드 + 최적화 + 접근성
- Sprint 16: UAT + v1.0

## 완료된 Phase
- Phase 1 (인프라+기반): Sprint 1-3
- Phase 1.5/1.5b (안정화): Sprint 4-5
- Phase 2 (학사+출결): Sprint 6-8
- Phase 3 (보강+소멸): Sprint 9-10
- Phase 4 (청구+공지문): Sprint 11-12
- Sprint 13 (PIN 옵션화): 독립

## 취소/이연 사항
- Phase 5 (단원평가+학습보고서): 전면 취소 (2026-05-31 원장 결정)
- CSV 가져오기: Sprint 15 이연
- Excel 비밀번호 보호 내보내기: Sprint 15 이연
