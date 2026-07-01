---
Sprint: 18  |  Date: 2026-07-01  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `src-tauri/src/commands/lock.rs` | [0회] | T0 A107: STALE_THRESHOLD_SECONDS 86400 상향 |
| `src-tauri/src/commands/integrity.rs` | [0회] | T0 A108·A109: rollback 파일명 고유성 + retry 테스트 |
| `src-tauri/src/startup.rs` | [0회] | T0 A110: cleanup_stale_tmp_backups spawn_blocking |
| `src-tauri/src/commands/setup.rs` | [0회] | T0 A111: WAL 실패 시 pool.close() |
| `src/app/fees/page.tsx` | [0회] | T3: 결제선생 카드사 optional |
| `src/components/schedules/ClassCalendar.tsx` | [0회] | T4·T5·T6·T7: 뷰/요일/색상/슬롯/월보기 |
| `src/lib/calendar-image.ts` | [0회] | T5: 요일 순서 동기화 |
| `src-tauri/src/commands/attendance.rs` | [0회] | T8: sync_attendance_on_schedule_change |
| `src-tauri/src/commands/academic.rs` | [0회] | T8: 3개 IPC 함수 수정 |
| `src/app/academic/` (신규 컴포넌트) | [0회] | T9: 교습일정 인쇄 |

## 이미 완료된 파일 (T1·T2)
| 파일 | 상태 |
|------|------|
| `src-tauri/migrations/308__allow_duplicate_supplement_day.sql` | ✅ 커밋됨 |
| `src-tauri/migrations/309__allow_duplicate_holiday.sql` | ✅ 커밋됨 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [x] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [x] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [x] DB 마이그레이션 추가 없음 (T8은 스키마 변경 없음)
- [x] 새 의존성 추가 없음

## 완료 기준 (이번 세션)
- [ ] T0: A107~A111 5건 전수 해소, cargo test 통과
- [ ] T3: 결제선생 카드사 드롭다운 optional 활성화
- [ ] T4: 수업관리 기본 뷰 timeGridWeek
- [ ] T5: FullCalendar firstDay=0 + calendar-image.ts 요일 동기화
- [ ] T6: 수업 시간 기준 4색 + 2열 균등 너비 + 다중 슬롯 칩
- [ ] T7: 월 보기 원생 이름 직접 표기 (Nx2 그리드 + hover)
- [ ] T8: sync_attendance_on_schedule_change + IPC 3곳 연동 + 테스트 3건
- [ ] T9: 교습일정 인쇄 버튼 + A4 HTML/CSS 출력
- [ ] T10: 통합 검증 전수 통과
