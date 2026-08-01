---
name: sprint-next-session
description: "v1.5.1(Sprint24) 배포 완료. 2026-08-01 프로덕션 사고=v1.5.1 첫 실행 마이그레이션 hang → 실 DB에 V313 선적용 우회로 정상화(완료). 다음=v1.5.2로 근본원인(migrator.run Windows 무한블록) 견고화. 사고 상세=[[v151-migration-hang]]. 새 세션 진입 시 가장 먼저 확인"
metadata: 
  node_type: memory
  type: project
  modified: 2026-08-01T11:46:22.144Z
  originSessionId: 2de8b3ba-b8a7-4d3d-bec8-87adf8a9081b
---

## ✅ 완료 상태
- **v1.5.0 / v1.5.1 배포 완료** (GitHub Release). Sprint 23(데이터 소실 재발방지)·Sprint 24(다월 교습기간 year_month 오태깅 일괄수정+V313).
- **2026-08-01 프로덕션 사고 → 정상화 완료**: v1.5.0→v1.5.1 첫 실행이 PIN 인증 직후 **무한 멈춤**. 원인 = **V313 pending 상태에서 `db::initialize`의 `migrator.run`이 실 Windows 환경에서 무한 블록**(pre-migration 백업은 완료됨). 우회 = 실 프로덕션 DB에 **V313을 오프라인 선적용+재암호화 후 교체**(절차 [[offline-migration-apply]]) → pending 제거 → v1.5.1이 그 구간을 건너뛰어 정상 실행. 양 PC 정상, 데이터 무손실 확인. 상세·근거 = [[v151-migration-hang]].
- 안전본: 교체 직전 원본이 스크래치패드 `PREREPLACE-app.db.bak` (세션 종료 시 소멸 — 필요하면 영구 위치로 이동).

## ⬜ 다음 세션 / 남은 작업
1. **[최우선] v1.5.2 — 마이그레이션 hang 근본원인 대응** (사고는 우회했으나 **다음 마이그레이션 V314 배포 시 재발**).
   - **계획 상세는 [[v151-migration-hang]] "v1.5.2 계획" 섹션에 기록됨** (트랙 A 진단 + 트랙 B timeout·fail-soft 하드닝, 범위, DoD, 재현 자산). 착수 시 `docs/sprint/sprint25.md`로 정식화.
   - Mac 독립 harness로는 재현 불가 → **실 Windows 환경 필요**(원장 PC 접근 가능할 때 착수).
2. **양 PC 안정성 관찰**: v1.5.1 며칠 정상 사용 확인 후 `PREREPLACE` 안전본 정리.
3. (선택) 이번 오프라인 마이그레이션 도구/재현 harness는 스크래치패드에 있음 — v1.5.2 착수 시 재사용.

관련: [[v151-migration-hang]], [[offline-migration-apply]], [[data-loss-recovery-method]], [[multi-month-period-pitfall]], [[workflow-no-pr]], [[deploy-version-three-files]], [[dev-pc-db-is-test-data]]
