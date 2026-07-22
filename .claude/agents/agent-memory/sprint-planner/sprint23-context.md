---
name: sprint23-context
description: Sprint 23 계획 수립 컨텍스트 -- 프로덕션 데이터 소실 사고 재발방지 (A 데이터 안전 + B 2번째 PC 로그인). ADR-012 A안 확정(클라우드 유지+접근 강화, 데이터 이전 없음). B 항목 MUST 승격
metadata:
  type: project
---

## Sprint 23 요약

- **목표**: 프로덕션 데이터 소실 사고(2026-07-22) 재발방지
- **버전**: v1.4.0 → v1.5.0
- **RCA SSOT**: `docs/incidents/2026-07-22-data-loss-rca.md`
- **ADR**: ADR-012 A안 확정 (`docs/arch/adr-012-db-live-location.md`) -- **클라우드 유지 + 접근 강화, 데이터 이전 없음**
- **Task 수**: T0~T9 (10개), 전부 MUST (T0 완료, B 항목 MUST 승격)
- **Capacity**: 29h 예상 (실용 34h 대비 5h 여유)
- **DB 마이그레이션**: 없음 (V312 유지)
- **신규 의존성**: 없음

## ADR-012 개정 영향

ADR-012 A안 확정(2026-07-22)으로 초기 계획 대비 변경:
- T0: 완료(0h). 초기 3h → 0h 절감
- T6: "데이터 위치 이전"(8h) → "클라우드 안전 접근 강화 -- 유휴 close + WAL 체크포인트 + 재연결"(5h). 3h 절감
- 합산 6h 절감 → B 항목(T7+T8, 6h) SHOULD→MUST 승격
- R142(이전 실패), R144(FK rebuild) 폐기. R149(유휴 재연결 지연), R150(잔여 torn-sync) 신규 등록
- ROADMAP에 B안(로컬+핸드오프)을 다음 Phase 후보로 등록 (승격 트리거: A 배포 후 클라우드 간섭 손상/복원 이벤트 관찰 시)

## 결함 → Task 매핑

| 결함 | 심각도 | Task |
|------|--------|------|
| C1 create_if_missing | Critical | T2 |
| C2 빈 DB 정상 판정 | Critical | T2 |
| C3 PRAGMA key 유실 | Critical | T1 |
| H1 stale WAL/SHM | High | T3 |
| H2 빈 DB 백업 | High | T4 |
| H3 exit-only restore | High | T3 |
| H4 소스 미검증 | High | T3 |
| H5 체크포인트 커넥션 | High | T1 (간접) |
| M1 config 불일치 | Medium | T5 |
| M2 salt 가드 없음 | Medium | T5 |
| M3 device.id 오판 | Medium | T8 |
| M4 STALE 기준 | Medium | T8 |
| B 2nd PC 키 없음 | Medium | T7 |

## T1/T6 역할 분담

- T1 (after_connect): **커넥션 단위** PRAGMA 보장 -- 새 커넥션마다 키 재적용
- T6 (유휴 close): **풀 라이프사이클** 관리 -- 유휴 시 풀 close + WAL 체크포인트 → 클라우드 동기화 안전화 → 활동 시 풀 재생성 (T1이 자동 적용)

**Why:** ADR-012 A안으로 데이터 이전이 불필요해져 Sprint 규모가 축소, 모든 항목 MUST 포함 가능.
**How to apply:** T0 완료 상태이므로 T1부터 바로 착수. T6은 T1 완료 후(after_connect 훅 의존).
