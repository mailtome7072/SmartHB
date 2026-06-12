# 성능 측정·최적화 보고서 — Sprint 15 T6

> 일자: 2026-06-07 | 기준: PRD §6 성능 요구 5종 (50명 데이터 기준)
> 범위 정책: 마이그레이션 없는 **안전 최적화만 적용**, 인덱스 추가·대규모 쿼리 재설계는 실측 후 Sprint 16.

---

## PRD 성능 기준 5종 & 측정 절차

실측 수치는 50명 시드 데이터 + 실제 사용 환경에서 측정해야 한다(원장 PC). 아래는 기준과 측정 방법.

| # | 기준 | 측정 방법 |
|---|------|----------|
| 1 | 화면 전환 300ms 이내 | Chrome DevTools(WebView) Performance — 메뉴 클릭~렌더 완료 |
| 2 | 출결표 50명×31일 렌더 1초 이내 | 출결 화면 진입~그리드 표시. Performance 탭 Scripting/Rendering |
| 3 | 청구 50명 생성 3초 이내 | 청구 생성 버튼~완료 토스트. 백엔드 `Instant` 로깅 가능 |
| 4 | 공지문 50장 생성 30초 이내 | 일괄 생성 진행률 0~100% 소요 |
| 5 | 앱 시작~메인 진입 3초 이내 | 앱 실행~대시보드 표시 |

> 자동 계측 하니스는 미도입(별도 인프라). 본 스프린트는 **정적 분석 + 안전 최적화**에 집중.

---

## 정적 분석 결과

### DB 인덱스 현황 (양호)
핵심 테이블에 인덱스가 대체로 구비됨:
- `regular_attendances`: student / year_month / event_date / makeup 인덱스 ✅
- `bills`: bill_year_month / student / status ✅, `payments`: bill / paid_date ✅
- `schedule_events`: date / code ✅, `students`: active(부분) ✅
- **미보유**: `makeup_attendances` 복합(student_id, year_month), `standard_fees`(weekly_hours), `study_periods`(year_month)

### 발견된 병목 후보
| 경로 | 유형 | 영향 | 처리 |
|------|------|------|------|
| `billing.rs` 청구 생성 — `standard_fees` 루프 내 조회 | N+1 (50회) | 소~중 | **✅ 이번 수정** (1회 로드) |
| `attendance.rs` 출결표 — 학생 루프 내 4쿼리×50명(~200회) | N+1 | 중 | Sprint 16 (쿼리 재설계, 회귀 위험) |
| `AttendanceGrid` — 1550셀 미메모이제이션 | 렌더 | 측정 필요 | Sprint 16 (측정 후 판단) |
| `notice.rs` 공지문 50장 — 동기 `std::fs::write` 순차 | 디스크 I/O | 중(체감 큼) | Sprint 16 (I/O 병렬화 검토) |

---

## 이번 스프린트 적용 (안전 최적화)

### ✅ 청구 생성 N+1 제거 (`billing.rs generate_bills`)
- **전**: 원생 50명 루프 안에서 `SELECT amount FROM standard_fees WHERE weekly_hours=?` 매번 실행(50회).
- **후**: 루프 진입 전 `standard_fees`(활성) 전체를 **1회 로드**해 `HashMap<weekly_hours, amount>`로 조회.
- **효과**: 쿼리 50회 → 1회. 원생 수 증가 시 선형 확장 비용 제거(확장성). standard_fees가 소규모라 인덱스 추가 불필요.
- **검증**: billing 테스트 42건 통과(동작 동일), clippy `--all-targets` 통과.

---

## 의도적 설계 — 변경하지 않음

### TanStack Query `staleTime: 0` (전역)
`src/providers/query-provider.tsx`의 `staleTime: 0` + `refetchOnMount: 'always'` + `refetchOnWindowFocus: true`는 **양 PC 동기화 환경에서 데이터 신선도를 보장하기 위한 의도적 선택**(주석 명시). 다른 PC에서 수정한 데이터를 화면 진입·재포커스 시 반영. 화면 전환 속도와의 trade-off이며, **신선도 우선이 맞다**(staleHB 데이터로 인한 오결정 방지) → **변경 권장하지 않음**. (정적 분석 도구의 'staleTime 상향' 제안은 본 앱 특성상 부적합)

---

## Sprint 16 이연 (실측 선행 권장)

먼저 50명 데이터로 5종 실측 → 기준 초과 항목만 선별 개선:
1. **출결표(기준 2)**: 실측 1초 초과 시 — ① `attendance.rs` 학생 루프 N+1을 JOIN/배치로 재설계 ② `makeup_attendances` 복합 인덱스(마이그레이션) ③ `AttendanceGrid` 셀 `React.memo`. (모두 회귀 위험이 있어 측정 근거 후 착수)
2. **공지문(기준 4)**: 30초 초과 시 — `save_notice_images_batch` 디스크 I/O 병렬화(`tokio` 비동기 쓰기). NTFS atomic write 안전성 동반 검토.
3. **인덱스(마이그레이션)**: `makeup_attendances(student_id, year_month)` 등 — Sprint 16 마이그레이션 묶음에 포함.

> 결론: 인덱스는 대체로 구비돼 있고, 명백·안전한 N+1(청구) 1건만 이번에 제거. 나머지는 **실측 근거 없는 선제 대수술을 피하고** Sprint 16에서 측정 후 처리.
