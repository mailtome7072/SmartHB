# ADR-011: 부분 보강(分 단위 부분 차감) 스키마 설계

- **상태**: Accepted  (2026-07-21 사용자 확정)
- **날짜**: 2026-07-21
- **결정자**: 원장(사용자) + Claude (Sprint 22 T0)

## Context (배경)

기존 보강 시스템은 **PI-02 "옵션 A — 일(日) 단위 매칭"**으로 설계되어, 결석 1건은 보강 1건에 매칭되면 시간과 무관하게 통째로 `makeup_done`으로 전이된다. 이로 인해 2시간(120분) 결석 원생이 60분만 보강해도 결석 레코드 전체가 소진 처리되어 **잔여 60분이 유실되고 보강 대상 목록에서 사라지는 버그**가 실사용 중 확인되었다 (Sprint 22 목표).

이를 **분(分) 단위 부분 차감** 모델로 전환한다. 요구 사항:
1. 결석 시간의 일부만 보강 등록 가능, 잔여분이 남으면 계속 보강 대상에 노출
2. 여러 번에 걸친 부분 보강으로 결석 점진 소진 (잔여 0일 때만 `makeup_done`)
3. **보강 취소 시** 해당 보강이 각 결석에 배분한 분만큼만 정확히 되돌려야 함 (T3)

### 현재 스키마 제약

- `regular_attendances`: `class_minutes`(원 결석분), `status`(present/absent/makeup_done/makeup_expired), `makeup_attendance_id`(보강 FK, 단일)
- `makeup_attendances`: `class_minutes`(실 보강분), 독립 레코드
- 결석 → 보강은 결석 쪽 **단일 FK `makeup_attendance_id`로 1:N** (보강 1건이 결석 N건 충당). 부분 보강을 도입하면 **1결석 : N보강**도 발생하므로 단일 FK로는 관계 표현이 불가능하다.
- 취소 로직(`makeup.rs:537-546`)은 `WHERE makeup_attendance_id = ?`로 연결 결석 전체를 all-or-nothing 환원 — 부분 배분 취소를 표현할 수 없다.

### 후보

- **A안**: `regular_attendances`에 `makeup_attended_minutes` 누적 컬럼 추가 → 잔여 = `class_minutes - makeup_attended_minutes`
- **B안**: 결석-보강 **배분(allocation) 링크 테이블** `makeup_allocations(makeup_id, absence_id, allocated_minutes)` 신설 (N:M + 배분량)

## 1단계: Weighted Decision Matrix

> 기준 해석은 본 결정 맥락에 맞춤: "성능"=잔여 계산 쿼리 비용(데이터 규모 50명 수준), "비용"=구현+마이그레이션 초기 비용, "관리 복잡도"=취소/감사/백필 로직의 견고성, "팀 경험"=기존 코드 패턴 친숙도, "확장성"=부분 취소·배분 조회 등 향후 요구 대응.

| 기준 | 가중치 | 선택지 A (누적 컬럼) | A 점수 | 선택지 B (배분 테이블) | B 점수 |
|------|--------|----------|--------|----------|--------|
| 성능 | 0.25 | 컬럼 직접 뺄셈, 조인 불필요 | 5 | `SUM(allocations)` 조인·집계 필요하나 50명 규모라 실용상 무영향 | 4 |
| 비용 | 0.20 | 컬럼 1개 추가로 단순하나, 정확한 취소 지원 위해 결국 배분 정보가 필요해 숨은 비용 발생 | 3 | 신규 테이블 + 기존 매칭 이전 + 쿼리 조인 변경으로 초기 비용 큼 | 3 |
| 관리 복잡도 | 0.20 | "어느 보강이 몇 분 채웠나" 정보 부재 → 취소·감사가 취약 | 3 | 관계가 명시적, 취소·감사·백필이 자연스러움 | 4 |
| 팀 경험 | 0.20 | `ADD COLUMN` 선례 다수(V302/V305~307) | 4 | 링크 테이블·조인 패턴(bills↔payments) 및 마이그레이션 경험 보유 | 4 |
| 확장성 | 0.15 | 1결석:N보강(N:M) 표현 불가 — 부분 취소·배분 조회 확장 제약 | 2 | N:M 배분량 완전 표현 | 5 |
| **총점** | | | **3.55** | | **3.95** |

총점 차이 0.40 (> 0.30) → **B안이 정량적으로 우세**.

## 2단계: SWOT + Trade-off

**선택지 A: 누적 컬럼 (`makeup_attended_minutes`)**
- Strengths: 최소 스키마 변경(ALTER ADD COLUMN 1개), 잔여 계산 쿼리 최속, 마이그레이션 단순
- Weaknesses: 어느 보강이 어느 결석을 얼마 채웠는지 추적 불가 → **T3 부분 취소 요구를 정확히 만족 못 함**, 감사 추적 취약
- Opportunities: 잔여 계산 술어 변경만으로 대부분 쿼리 대응 가능
- Threats: 취소 정확도를 확보하려면 결국 배분 정보를 별도 보관해야 해 A안 단독으론 요구 미충족 → 재작업 위험

**선택지 B: 배분 링크 테이블 (`makeup_allocations`)**
- Strengths: 1결석:N보강 / 1보강:N결석 모두 표현, **부분 취소 시 해당 allocation만 정확 제거**, 배분 이력 감사 가능, 백필 결과를 배분 레코드로 그대로 표현
- Weaknesses: 신규 테이블 + 조인 쿼리로 초기 구현량 증가, 기존 `makeup_attendance_id` 매칭을 allocation으로 이전 필요
- Opportunities: 향후 보강 배분 상세 조회·리포트 확장 용이
- Threats: 8개 쿼리를 조인 기반으로 일괄 변경(R139), 백필 정확도(R140)

| 선택 시 | 개선 (↑) | 저하 (↓) |
|---------|----------|----------|
| A 선택 | 스키마·쿼리 단순, 성능 | 취소 정확도, 감사 추적, 확장성 (요구 미충족) |
| B 선택 | 취소 정확도, 확장성, 감사 | 초기 구현량, 쿼리 조인 복잡도 |

### Risk

| 리스크 | 관련 선택지 | 영향도 | 완화 방법 |
|--------|------------|--------|----------|
| 8개 쿼리 조인 전환 누락 → 잔여 계산 불일치 | B (R139) | 높음 | T4 회귀 체크리스트 전수 + 쿼리별 단위 테스트 |
| 기존 매칭→allocation 백필 오류 | B (R140) | 높음 | 멱등 설계 + 마이그레이션 직전 사전 스냅샷 + cipher-on 스모크 |
| 취소 부정확으로 데이터 오염 | A | 높음 | (A 채택 시 회피 불가 — B 채택으로 해소) |
| 테이블 재구성 시 deferred FK 함정(code 787) | 공통 (R142) | 중간 | `makeup_attendance_id` 컬럼 **DROP 하지 않고 레거시 유지** → 재구성 회피 |

## Decision (결정)

**B안 (배분 링크 테이블 `makeup_allocations`)을 채택한다.**

> 1단계 총점: A = 3.55, B = 3.95 → B안 채택
> 핵심 Trade-off: 초기 구현량과 쿼리 조인 복잡도를 감수하는 대신, 이 스프린트의 필수 요구인 **부분 보강의 정확한 취소·추적**과 향후 확장성을 확보한다. A안은 취소 정확도를 근본적으로 만족하지 못해 재작업 위험이 크다.

### 스키마 설계 (V311)

```sql
-- 신규: 결석-보강 배분 링크 테이블
CREATE TABLE makeup_allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    makeup_id INTEGER NOT NULL REFERENCES makeup_attendances(id),
    absence_id INTEGER NOT NULL REFERENCES regular_attendances(id),
    allocated_minutes INTEGER NOT NULL CHECK (allocated_minutes > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_makeup_alloc_makeup ON makeup_allocations(makeup_id);
CREATE INDEX idx_makeup_alloc_absence ON makeup_allocations(absence_id);
```

- **잔여 계산**: `class_minutes - COALESCE(SUM(alloc.allocated_minutes), 0)` (absence_id 기준 LEFT JOIN)
- **status 판정**: 잔여 == 0 → `makeup_done`, 잔여 > 0 → `absent` 유지 (기존 4상태 모델 유지, R141 해소)
- **배포 안전성 (R142)**: `regular_attendances.makeup_attendance_id` 컬럼은 **DROP 하지 않고 레거시로 남긴다** (신규 로직 미사용). 테이블 재구성이 불필요해져 deferred FK 함정을 원천 회피. 단순 `CREATE TABLE`(신규)만 수행.
- **기존 매칭 이전**: V107 방식으로 매칭된 기존 `makeup_done` 데이터의 `(makeup_id, absence_id, class_minutes)`를 V312 백필에서 `makeup_allocations`로 이전하며 동시에 부분 보강 잔여분을 복원.

## Consequences (영향)

**긍정적 영향:**
- 부분 보강의 정확한 소진·잔여 추적, 취소 시 해당 배분만 정밀 제거 가능
- status 4상태 모델 그대로 유지 → 기존 UI/집계 로직 호환성 유지 (R141 완화)
- `makeup_attendance_id` 유지로 테이블 재구성 회피 → 마이그레이션 안전성 확보 (R142)
- 백필 결과를 allocation 레코드로 자연스럽게 표현 → 감사·검증 용이

**부정적 영향 / 주의사항:**
- 보강 대상 목록/월간 요약/소멸/진단/원생 상세 등 8개 쿼리를 `makeup_allocations` 조인 기반 잔여 계산으로 일괄 전환 필요 (R139) — T4 회귀 체크리스트로 관리
- `makeup_attendance_id` 레거시 컬럼과 신규 `makeup_allocations`가 공존 → 신규 코드는 반드시 allocations 사용, 레거시 컬럼 참조 잔재 없도록 T4에서 전수 확인
- 잔여 계산이 매 쿼리에서 집계 조인 발생 (50명 규모라 성능 영향 미미)

**후속 액션:**
- T1: 위 V311 마이그레이션 작성 + 인메모리 단위 테스트 (테이블 생성·제약)
- T2/T3: 등록/취소 로직을 allocation 기반으로 구현
- T4: 8개 쿼리 잔여 계산 조인 전환 (회귀 체크리스트)
- T5: V312 백필 — 기존 매칭을 allocation으로 이전 + 부분 보강 잔여 복원. **완전 자동·무알림**(앱 첫 실행 시 조용히 자동 보정, 원장에게 UI 안내 없음, 보정 건수는 `audit_logs`에만 기록 — 2026-07-21 사용자 확정)
