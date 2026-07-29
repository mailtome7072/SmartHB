---
name: multi-month-period-pitfall
description: "다월 교습기간(달력월과 다른 study_period, 예 8월=7/30~9/2) 함정 모음 — year_month는 소속 교습기간 기준, 청구 시수는 기간 유효 스케줄(이력) 기준. 날짜→월/스케줄→시수 로직 건드릴 때 필독"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9497daac-3ea6-45b1-b7c3-59d8beed3d8d
  modified: 2026-07-29T09:08:43.139Z
---

교습기간(`study_periods`)은 달력월과 다르게 여러 달에 걸칠 수 있다(예: "8월" = 2026-07-30 ~ 2026-09-02). `year_month`는 **사용자 지정 라벨(SSOT)** 이지 날짜에서 파생되는 값이 아니다. 단월 기간에선 우연히 일치해 버그가 안 보이고, **경계일(7/30·7/31·9/1·9/2)에서만** 터진다. Sprint 21 R136 → Sprint 24 에서 계열 전수 해소.

**날짜/스케줄 관련 로직을 건드릴 때 항상 자문**: "이 로직이 (a) 달력월을 넘는 교습기간, (b) 교습기간이 스케줄 변경 적용일보다 앞선 경우에도 맞는가?"

## 규칙 (정답 패턴)
- **출결/보강 `year_month` 태깅** = event_date가 속한 확정 교습기간의 year_month. 달력월(`date[..7]`) 금지. 공유 헬퍼 `commands/periods::period_year_month_for_date` 사용.
- **청구 주당시간(weekly_hours)** = 교습기간에 **유효했던 스케줄**(`effective_from <= period_end < effective_to`, 대표일=기간 종료일). **현행 스케줄(`effective_to IS NULL`)만 보면 틀림** — 변경 적용일이 기간보다 뒤/앞이면 오산(7월 3h를 2h로). `generate_bills`·`get_billing_summary` 둘 다 동일 기준이어야 유령 버튼 안 생김. ⚠️ 초기 감사가 billing을 "SAFE 미변경"으로 판정했으나 이 케이스를 놓쳤다 — billing도 예외 아님.
- **교습기간 경계/확정 변경 시 기존 데이터 재태깅** = `periods::reconcile_attendance_year_month`(V313과 동일 로직 idempotent). academic 교습기간 생성/수정/확정 직후 + **앱 startup**에서 호출(자가 치유, fail-soft). V313 마이그레이션은 1회성이라 그 이후 생긴 드리프트를 못 잡으므로 런타임 reconcile 필수.
- **수업일 이동(move_attendance)** = "같은 달력월"이 아니라 "같은 교습기간" 판정. UI 달력도 `periodStart~periodEnd` 범위로 그려야 이웃 달(9/1·9/2) 선택 가능. 다른 교습기간(9/30 등)은 이동이 아니라 보강.
- **schedule_events(공휴일/방학/휴원)** 는 예외: year_month 없는 **순수 달력-날짜 엔티티**. "지난 달 수정 차단" 가드는 달력월 기준이 맞다(B7 설계 결정).

## 검출/공유 자산
- 자가진단 검사 8번: `regular/makeup_attendances.year_month ↔ 소속 교습기간` 불일치 검출(diagnosis.rs).
- 공유 헬퍼는 `src-tauri/src/commands/periods.rs`(`period_year_month_for_date`, `load_confirmed_period`, `reconcile_attendance_year_month`) — attendance/academic 양방향 결합 회피용 중립 모듈.

관련: [[sprint-next-session]], [[dev-pc-db-is-test-data]]
