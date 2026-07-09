---
name: sprint19-context
description: Sprint 19 계획 수립 시 발견한 주요 컨텍스트 -- UX 개선 8건 + 학년 자동승급 + 학교급 필터링, V310, 2xN 버그 미확정
metadata:
  type: project
---

## Sprint 19 컨텍스트

- **목표**: 실사용 원장님 UI/UX 피드백 8건 + 학년 자동승급 + 학교급 필터링 (총 10건)
- **특성**: 프론트엔드 중심. V310 마이그레이션 1건 (schools.school_type 보정). 신규 의존성 없음.
- **Capacity**: 33h 예상 / 40h 가용 (18% 버퍼)
- **주요 리스크**: T5 주보기 2xN 버그 원인 미확정 (R126), T4 인쇄 밴드 `@media print` 호환성 (R127)

## Sprint 18 회고 반영

| 항목 | 반영 |
|------|------|
| A113 (상수 쌍 목록화) | T0 |
| A116 (인쇄 동적 행 수) | T4 흡수 |
| A114 (sync_single_date) | Post-MVP 유지 |
| A115 (cipher 스모크) | 배포 후 수동 검증 |

## 사전 코드 조사 주요 발견

- 원생 목록: 부분 정렬 구현 있음(`SORTABLE_COLUMNS`), 학교급/성별/수업시간 컬럼 미지원. 서버사이드 정렬
- 출결 그리드: 정렬 없음, sticky 4컬럼 있음, 이중 스크롤 컨테이너(부모+자식 모두 `overflow-auto`)
- 청구 그리드: 정렬 없음, sticky 없음
- 인쇄 캘린더: 기간성 코드 셀별 반복 렌더링(밴드 없음), Red 테두리 없음, `calendar-image.ts`에 참조 가능 패턴 존재
- 수업관리 주보기: `needSplit` 3명 이상 트리거, `rowGroup = Math.floor(column / 2)` 30분 행 분할, 화살표(↓/↑) 존재
- 수업관리 일보기: `needSplit` 항상 false, FullCalendar 자동 폭 분배만
- 대시보드: `sm:flex-row` 좌우 배치, 각 `sm:w-1/2`
- `students.grade`는 초등/중등이 같은 숫자 범위 공유 → `school_level` 우선 정렬 필수

## 추가 요구 (rev2)

- T8 학년 자동승급: `diagnosis.rs`의 `last_auto_diagnosis` 패턴 재사용. `app_settings`에 `last_grade_promotion_year` 키. 초등 grade<6, 중등 grade<3만 승급. 퇴교생 제외. 확인 다이얼로그 필수
- T9 학교급 필터: V310(`310__auto_correct_school_type.sql`) — `schools.school_type` 텍스트 기반 보정(초등학교→elementary, 중학교→middle). 코드 관리 UI에 school_type 셀렉트 추가. student-form의 `.includes('중학교')` 임시 로직 → school_type 기반 필터로 교체
- 최신 마이그레이션: V309 → V310 (Sprint 19)
