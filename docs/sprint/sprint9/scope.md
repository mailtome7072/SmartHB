---
Sprint: 9  |  Date: 2026-05-24  |  Session: #1
---

> Sprint 9 Session #1 — T1 (PI-02 결정 반영 + 보강 도메인 설계 검토).
> 예상 2h. 본 세션은 **순수 설계/검증** task — 코드 변경 없음, scope.md 작성으로 종료.

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T1** | PI-02 확정 + 기존 스키마 검증 + scope.md 작성 | 2h |

> 사용자 결정 (2026-05-24): PI-02 = 옵션 A (일 단위 매칭) 확정. sprint9.md L62 갱신 완료.

---

## PI-02 확정 사항

**옵션 A — 일 단위 매칭** (사용자 결정 2026-05-24).

| 항목 | 규칙 |
|------|------|
| 매칭 단위 | 일(day) 단위 — 보강 1일 = 결석 N일 충당 |
| 시간값 검증 | 없음 — `class_minutes` 비교 생략 |
| 보강필요시간 표시 | 기존 `compute_summary` 유지 — `SUM(absence.class_minutes WHERE makeup_attendance_id IS NULL) - SUM(makeup_attended.class_minutes)` |
| 변경 시 영향 | T3 `create_makeup_with_absences` 의 검증 3만 활성/비활성 (분 단위 전환 시 단순) |

→ T3 코드에 PI-02 결정 명시 (주석 + 분 단위 전환 시 활성 위치 표시) — R58 대응.

---

## 기존 스키마 검증 결과

### 결론: **V108 신규 마이그레이션 불필요**

보강 도메인 전체 흐름이 V106/V107 + V102/V301 스키마로 구현 가능. 신규 도메인 컬럼/테이블 추가 없음.

### 검증 매트릭스

| Sprint 9 요구 사항 | 기존 스키마 항목 | 검증 결과 |
|----------------|----------------|----------|
| 미처리 결석 조회 (T2 `get_pending_absences`) | `regular_attendances.status='absent' AND makeup_attendance_id IS NULL` (V106) | ✅ 가능 — 인덱스 `idx_regular_att_makeup` (V107) 활용 |
| 소멸기한 임박 정렬 | `regular_attendances.makeup_deadline` (V106, YYYY-MM TEXT) | ✅ `ORDER BY makeup_deadline ASC, event_date ASC` |
| 보강 가능 일자 판별 (T2 `get_makeup_eligible_dates`) | `schedule_codes.allows_makeup_class` (V102) + V301 공휴수업일 보정 | ✅ `JOIN schedule_events ON event_date` 으로 가능 |
| 보강 등록 + 매칭 (T3 `create_makeup_with_absences`) | `INSERT makeup_attendances` + `UPDATE regular_attendances` (V107 FK) | ✅ 트랜잭션 + FK 강제로 무결성 보장 |
| 보강 취소 → 결석 환원 (T4 `cancel_makeup`) | `UPDATE regular_attendances SET makeup_attendance_id=NULL, status='absent'` + `DELETE makeup_attendances` | ✅ 트랜잭션 내 순차 — FK 위반 없음 (DELETE 전 NULL 처리) |
| 보강 미등원 (T4 `mark_makeup_absent`) | `makeup_attendances.status='makeup_absent'` (V106 CHECK 2상태) | ✅ |
| 결석 이력 조회 (T8) | 기존 SELECT 만 | ✅ |

### audit::AuditEventType 추가 variants

코드 변경 (마이그레이션 아님). T3/T4 에서 도입:
- `MakeupCreated` → "makeup-created"
- `MakeupCancelled` → "makeup-cancelled"
- `MakeupAbsent` → "makeup-absent"

---

## 신규 모듈/파일 결정

| 결정 | 이유 |
|------|------|
| **모듈 분리**: `src-tauri/src/commands/makeup.rs` 신규 (attendance.rs 에 누적 X) | attendance.rs 이미 1000+ 줄. 보강은 별개 도메인이므로 모듈 분리가 가독성/유지보수 측면 유리 |
| `mod.rs` 에 `pub mod makeup;` 추가 | T2 작업 |
| `lib.rs` invoke_handler 등록 — 보강 IPC 6종 추가 예정 (T2 2종 + T3 1종 + T4 3종) | — |
| 프론트엔드 라우트 신규 없음 — 보강 등록은 `/attendance` 의 비수업일 셀 클릭 다이얼로그 | T6 작업. `MakeupRegisterDialog` 신규 컴포넌트 |
| 보강데이 일괄 — `/attendance` 헤더 "보강데이 일괄" 버튼 → 별도 페이지 `/attendance/makeup-batch` | T7. 다중 원생 선택 UI 복잡도 분리 |
| 결석 이력 — `/students/[id]` 상세 페이지에 섹션 추가 | T8 |

---

## 데이터 흐름도

```
[보강 등록 — 개별 (UC-4 핵심)]
사용자: /attendance 비수업일 셀 클릭
   ↓
프론트엔드: MakeupRegisterDialog 오픈
   ↓
백엔드: get_makeup_eligible_dates(student_id, year_month) — 가능 일자 사전 검증
   ↓
백엔드: get_pending_absences(student_id) — 충당 결석 목록 (소멸기한 임박 순)
   ↓
사용자: 결석 N건 다중 선택 → "확정"
   ↓
백엔드: create_makeup_with_absences(student_id, event_date, class_minutes, absence_ids)
   ├── 트랜잭션 BEGIN
   ├── 검증 1: event_date 보강 가능 일자
   ├── 검증 2: absence_ids 모두 미처리 결석
   ├── 검증 3: (PI-02 일 단위 — 생략) / (분 단위 — class_minutes 합산 비교)
   ├── INSERT makeup_attendances → makeup_id
   ├── UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=makeup_id WHERE id IN (absence_ids)
   ├── audit::MakeupCreated 기록
   └── COMMIT
   ↓
프론트엔드: 출결표 invalidate → 결석 셀 빨강 → "보강" 표시로 전환

[보강 취소]
사용자: 보강 행 우클릭 → "취소"
   ↓
백엔드: cancel_makeup(makeup_id)
   ├── 트랜잭션
   ├── UPDATE regular_attendances SET makeup_attendance_id=NULL, status='absent' WHERE makeup_attendance_id=?
   ├── DELETE makeup_attendances WHERE id=?
   ├── audit::MakeupCancelled
   └── COMMIT

[보강 미등원]
사용자: 보강 행 마킹 → "미등원"
   ↓
백엔드: mark_makeup_absent(makeup_id)
   ├── 트랜잭션
   ├── UPDATE makeup_attendances SET status='makeup_absent' WHERE id=?
   ├── UPDATE regular_attendances SET makeup_attendance_id=NULL, status='absent' (연결된 결석 환원)
   │     ※ 결석 상태 유지 — 새 결석 미생성, 다음 보강 매칭 대상으로 재진입
   ├── audit::MakeupAbsent
   └── COMMIT
```

---

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| docs/sprint/sprint9/scope.md | [1회] | 본 세션 — 신규 |

> T1 은 순수 설계 task — 코드 변경 없음. 다음 세션(T2)부터 백엔드 신규.

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/`, `SETUP.sh`, `docs/harness-engineering/` — Forbidden
- [ ] `src-tauri/migrations/` — V108 신규 마이그레이션 불필요 결정. 기존 V106/V107/V102/V301 활용
- [ ] `src/` 전체 — T1 범위 외 (T5~T8 에서 다룸)
- [ ] `src-tauri/src/` 전체 — T1 범위 외 (T2~T4 에서 다룸)

## 완료 기준 (이번 세션) — T1 AC (sprint9.md L80)

- ✅ AC-T1-1: PI-02 확정 기록 (옵션 A 일 단위 매칭) — "PI-02 확정 사항" 섹션
- ✅ AC-T1-2: 기존 스키마 검증 + V108 불필요 결정 명문화 — "검증 매트릭스" 표
- ✅ AC-T1-3: 보강 도메인 데이터 흐름도 작성 — "데이터 흐름도" 섹션
- ✅ AC-T1-4: 모듈 분리 결정 (`makeup.rs` 신규) — "신규 모듈/파일 결정" 표

## 세션 종료 조건

- ✅ scope.md 완성 (본 파일)
- ⬜ 단일 커밋 (sprint9 브랜치 첫 커밋)

## 발견된 이슈
(없음 — T1 은 설계 task)

## 다음 세션 (T2) 미리보기

- 신규 모듈 `src-tauri/src/commands/makeup.rs` 생성
- IPC 2종: `get_pending_absences`, `get_makeup_eligible_dates`
- A43 흡수: `validate_year_month` 월 범위(01-12) 검증 강화 — 기존 `attendance.rs::validate_year_month` 수정
- `src-tauri/src/commands/mod.rs` 에 `pub mod makeup;` 추가
- 단위 테스트: 소멸기한 정렬 + 보강 가능 일자 필터 + validate 무효 입력 거부

---

## carry-over

- A39/A40 프로세스 개선이 본 sprint 부터 강제 — T9 통합 검증에서 검증
- T1 코드 변경 없음으로 self-verify 단계 생략 가능 (scope.md 단일 커밋)
