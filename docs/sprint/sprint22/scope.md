---
Sprint: 22  |  Date: 2026-07-21  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

### 백엔드 (Rust)
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/311__*.sql | [0회] | T1: 부분 보강 스키마 변경 (ADR-011 결정 반영, 신규 파일) |
| src-tauri/migrations/312__*.sql | [0회] | T5: 유실 데이터 자동 백필 (신규 파일, 멱등) |
| src-tauri/src/commands/makeup.rs | [0회] | T2 등록 부분차감 전환 / T3 취소 부분차감 대응 |
| src-tauri/src/commands/calendar.rs | [0회] | T4-1: 보강 대상 목록 잔여분>0 기준 |
| src-tauri/src/commands/attendance.rs | [0회] | T4-2 월간 요약 잔여분 / T4-8 A114 이력 패턴 통일 |
| src-tauri/src/commands/expiration.rs | [0회] | T4-3~5: 소멸/퇴교 잔여분 기준 |
| src-tauri/src/commands/diagnosis.rs | [0회] | T4-6: 부분 보강 부족분 감지 재검토 |
| src-tauri/src/commands/students.rs | [0회] | T4-7: 원생 상세 집계 잔여분 기준 |
| src-tauri/src/startup.rs | [0회] | T5: 마이그레이션 직전 사전 스냅샷 백업 안전장치 |
| src-tauri/src/commands/audit.rs | [0회] | T5: 백필 보정 건수 audit_logs 기록 (UI 비노출, 필요 시) |

### 프론트엔드 (TS/React)
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/components/attendance/MakeupRegisterDialog.tsx | [0회] | T7: 1시간 단위 선택 UI + 잔여분 표시 |
| src/components/attendance/AttendanceGrid.tsx | [0회] | T8: sticky z-index 층위 재정렬 |
| src/lib/tauri/index.ts | [0회] | 잔여분 조회 래퍼 시그니처 변경 (필요 시) |
| src/types/*.ts | [0회] | 잔여분 응답 타입 (필요 시) |
| src/components/attendance/MoveAttendanceDialog.tsx | [0회] | T7: A126 yearMonth→invalidationYm prop 명확화 |

### 문서
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/arch/adr-011-partial-makeup-schema.md | [0회] | T0: 스키마 설계 ADR (신규) |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] .github/workflows/ — CI/CD 파이프라인 (hook이 차단)
- [ ] SETUP.sh — 초기화 스크립트 (hook이 차단)
- [ ] 기존 마이그레이션 파일 V001~V310 — sqlx 체크섬 불일치 위험 (R143), 신규 V311+만 추가
- [ ] docker/, docker-compose*.yml
- [ ] docs/harness-engineering/

## 완료 기준 (이번 세션)
- [ ] T0: ADR-011 부분 보강 스키마 설계 결정 (테이블 재구성 회피 우선)
- [ ] T1: V311 마이그레이션 (스키마 변경) + 인메모리 단위 테스트
- [ ] T2: 보강 등록 분 단위 부분 차감 전환 + 비즈니스 규칙 100% 테스트
- [ ] T3: 보강 취소 부분 차감 대응 + 테스트
- [ ] T4: 8개 쿼리 술어 일괄 변경 (회귀 체크리스트 전수) + 테스트
- [ ] T5: V312 백필(완전 자동·무알림, audit만 기록) + 마이그레이션 직전 사전 스냅샷 백업 + 멱등성 테스트
- [ ] T6: (취소됨) 백필 결과 안내 UI — 무알림 결정으로 제거
- [ ] T7: 보강 등록 1시간 단위 선택 UI + A126 prop 명확화
- [ ] T8: 출결 그리드 z-index 수정 + 시각 검증
- [ ] T9: 통합 검증 (자동 + cipher-on 백필 스모크 + 시각)

## 발견된 이슈
<!-- Step-back 프로토콜: 구조적 충돌/설계 오류 발견 시 여기 기록 후 사용자 보고 -->
(없음)

## 세션 로그
- #1 (2026-07-21): 브랜치 생성, scope 선언, 계획 문서 커밋. T0(ADR) 착수.
- #1 계속: T0~T9 구현 완료 (커밋 10건).
  - T0 ADR-011(B안) / T1 V311 makeup_allocations / T2·T3 등록·취소 분단위 부분차감
  - T4 조회·집계·소멸·진단 쿼리 전수 잔여분 전환 (calendar/attendance/expiration/makeup/diagnosis;
    students 재원로직은 변경 불필요 판정)
  - T5 V312 백필(원장님 케이스 포함 5건 검증) + 마이그레이션 직전 사전 스냅샷
  - T7 보강 UI 잔여분 + A126 rename / T8 그리드 z-index 층위 재정렬
  - A114 이연 유지(범위 외)
  - 자동검증 전부 통과: cargo test 457 / clippy / lint / tsc / build / cargo check --features cipher
  - 잔여: 시각 검증(개발서버) + cipher-on 실DB 백필 스모크(배포 단계) — sprint-close/review 대기
