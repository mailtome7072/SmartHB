---
name: prd-v1.5-roadmap-decisions
description: PRD v1.5 기반 첫 ROADMAP 생성 시 주요 결정사항 — Phase 분할 근거, 인프라 우선순위, PI 미결정 항목 배치
metadata:
  type: project
---

## ROADMAP 생성 결정사항 (2026-05-18, PRD v1.5)

**인프라 우선순위 결정**: SQLCipher / app.lock / 4계층 백업 / 무결성 검증을 Phase 1 Sprint 1에 배치. 이유: 모든 도메인 데이터 작업이 암호화된 DB 위에서 동작해야 하며, 후반에 SQLCipher를 도입하면 마이그레이션 전체를 재작성해야 함.

**Why:** PRD §5.3~§5.5가 데이터 보안을 MVP 필수로 정의하며, 양 PC 동기화 폴더 구조(smarthb/app.db + app.lock + backup/)가 모든 기능의 기반.

**How to apply:** Sprint 1 착수 전 ADR-001(SQLCipher) 완료 필수. PI-07(복구 코드) 결정도 Sprint 1 착수 전 사용자 확인.

### Phase 분할 근거 (7 Phase, 14 Sprint)

| Phase | 스프린트 수 | 분할 근거 |
|-------|-----------|-----------|
| 1. 인프라+기반 | 3 | SQLCipher/lock/백업(인프라) + 원생/코드(기반 도메인) + 프론트+마법사 |
| 2. 학사+출결 | 2 | UC-2/UC-3 달성, Phase 3~5의 선행 조건 |
| 3. 보강+소멸 | 2 | PRD에서 가장 복잡한 도메인(4상태+매칭+소멸), 별도 Phase 분리 |
| 4. 청구+공지문 | 2 | UC-5 달성, 비즈니스 가치 높은 미납률 KPI 직접 기여 |
| 5. 단원평가+보고서 | 2 | v1.5 분기 단위 재설계 반영, A4 4분할 인쇄 복잡도 |
| 6. 대시보드+유틸 | 1 | 모든 도메인 데이터 집계, 자가 진단(§6.6) |
| 7. 안정화+UAT | 2 | 양 OS 빌드 검증 + 성능 + 접근성 + 원장 2주 UAT |

### 미해결 PI 항목 배치

- PI-01 (소멸 트리거): Sprint 7에서 앱 시작 batch로 구현
- PI-02 (보강 시간값): Sprint 6 착수 전 사용자 결정 필수, 미결정 시 보수적 채택
- PI-05 (자동 채번): Sprint 2 착수 전 결정, 미결정 시 수동 입력만
- PI-07 (복구 코드): Sprint 1 착수 전 결정 필수

### PRD v1.5 핵심 변경 반영

- 학습보고서: 월→분기 단위, 종합의견 1종, 점수 종속(복사 보관 금지)
- 청구 마감: 3단계(미확정→확정→마감)
- E2E: Tauri WebDriver(tauri-driver)
- 백업 복원 리허설: "필요시 수행 모드"로 단순화
