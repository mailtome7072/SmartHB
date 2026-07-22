---
name: sprint-next-session
description: "✅ Sprint 22 완료 + develop 머지 + v1.4.0 프로덕션 배포 완료(2026-07-22). Sprint 23 계획 대기. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint22-makeup-partial-2026-07-22
---

## ✅ 2026-07-22 — Sprint 22: 보강 분 단위 부분 차감 + 출결 그리드 버그 + UX 개선 → v1.4.0 배포

### 배경 (실사용 버그 2건)
1. **보강 부분 차감 유실**: 2시간 결석에 1시간만 보강해도 잔여 1시간이 보강 대상 목록에서 사라짐. 원인 = PI-02 "옵션 A 일 단위 매칭"(시간 비교 없이 결석 레코드를 통째 makeup_done 전이).
2. **출결 그리드 상하 스크롤 시 고정 헤더 깨짐** = thead(z-10)와 좌측 고정 셀(z-10) z-index 동률 충돌.

### 핵심 설계 (ADR-011, `docs/arch/adr-011-partial-makeup-schema.md`)
- **`makeup_allocations` 배분 링크 테이블(V311)** — (makeup_id, absence_id, allocated_minutes). 결석↔보강 N:M + 분 단위 배분량.
- 잔여 = `class_minutes - SUM(allocated_minutes)`. 잔여 0일 때만 makeup_done, 잔여>0이면 absent 유지.
- **`makeup_attendance_id` 컬럼은 레거시로 남김(DROP 안 함)** → 테이블 재구성 회피 = deferred FK 함정(V108 code 787) 원천 차단. V311은 순수 CREATE TABLE.
- **V312 백필**: 기존 makeup_done을 SQLite 윈도우 함수로 소멸기한 임박순 순차 배분 이전, 부분보강 잔여를 absent 복원. 멱등(NOT EXISTS + 조건부 UPDATE). **완전 자동·무알림**(사용자 결정 — 원장 UI 안내 없음).
- 마이그레이션 직전 **사전 스냅샷 백업**(exit 백업 계층 재사용, `db.rs::has_pending_migrations`).

### 회귀 주의 (R139) — 잔여분 계산 공유 헬퍼
`makeup::remaining_minutes_expr(alias)` 로 8곳 통일: calendar/attendance/expiration/makeup/diagnosis. 기존 `makeup_attendance_id IS NULL` 조건 전부 제거 → **`status='absent'`(부분소진 포함) 기준**. students.rs 재원(revive) 로직만 변경 불필요(status 전이라 잔여 무관). diagnosis 고아보강(→allocations 기준)/과보강(→결석별 배분 초과) 검사도 재작성.

### 시각 검증 중 추가된 UX 개선 (배포 전 함께 반영)
- 결석 이력에 **'부분보강'** 상태 구분(주황 + 'N시간 보강 · 잔여 M시간')
- 출결관리 상단 **'보강 관리' 버튼** → 수업관리 보강 탭 (Zustand `schedulesInitialMakeupTab` 프리셋 — static export useSearchParams Suspense 이슈 회피, attendanceSearchPreset과 대칭)
- 보강 목록 원생 이름 클릭 → 결석 이력, 출결 그리드/보강 목록 원생 이름 표시 통일(상시 accent 링크 + '결석 이력 보기' 힌트)
- 원생관리 재원상태 필터 3분화(재원중/전체/퇴원, 택1 세그먼트, 백엔드 `StudentFilter.withdrawn_only`) + 퇴교생 행 `(퇴교 YYYY-MM-DD)` 표시

### 배포
- develop→master ff 머지(723214c), v1.4.0 태그, GitHub Actions 성공, 릴리스 발행: https://github.com/mailtome7072/SmartHB/releases/tag/v1.4.0
- 아티팩트: `SmartHB_1.4.0_x64-setup.exe` / `SmartHB_1.4.0_aarch64.dmg`
- develop 역머지+push 완료. local/remote 동기화.
- 버전 3파일+lock 동기화 확인([[deploy-version-three-files]]).
- **cipher-on 실 DB 백필 스모크는 원장님 PC 인스톨러 설치 후 확인**으로 대체(개발 PC는 테스트 DB — [[dev-pc-db-is-test-data]]). 백필 로직은 인메모리 5건 테스트(원장님 케이스 120분→60분 잔여복원 포함)로 검증됨.

## 마이그레이션 현황
최신 **V312** (V311 makeup_allocations 테이블 + V312 유실 데이터 백필). develop+master 반영 완료.

## ⬜ 다음 세션 / 이연 항목
- Sprint 23 계획 대기 — 남은 필수 작업 없음.
- **이연**: A114(sync_single_date 이력 패턴 통일 — 보강과 무관한 별도 리팩터), A127(cancel_makeup_impl N+1 쿼리 — 50명 규모 영향 미미).

관련: [[workflow-no-pr]], [[deploy-version-three-files]], [[dev-pc-db-is-test-data]]
