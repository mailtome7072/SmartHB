# ADR-006: 캘린더 라이브러리 선택 (PI-03)

| 항목 | 내용 |
|------|------|
| 상태 | **Accepted** (2026-05-26) |
| Sprint | Sprint 10 T8 |
| 결정자 | 사용자 (원장) |
| 관련 PRD | §4.6 수업 관리 캘린더 (일/주/월 뷰 + 원생 상세 + 보강 관리) |

## 배경

PRD §4.6 수업 관리 캘린더 뷰를 위한 React 라이브러리 선택. 후보:

- **FullCalendar** (`@fullcalendar/react` + 일/주/월 플러그인)
- **React Big Calendar** (`react-big-calendar`)

요구사항:

- 일/주/월 뷰 전환
- 시간대별 수업 원생 표시 (시작 + 진행 중 합산 — AC-4.6-1)
- 원생 상세 팝업 (PRD §4.6.2)
- 보강 관리 뷰 (소멸 임박 강조 — AC-4.6-2)
- Next.js 15 static export 호환
- React 19 호환
- 50대 운영자 친화 (시각적 익숙함)

## Weighted Decision Matrix

| 기준 | 가중치 | FullCalendar | 점수 | React Big Calendar | 점수 |
|------|-------|--------------|------|---------------------|------|
| 라이선스 (MVP 범위) | 0.20 | MIT (premium 기능 상용, MVP 미사용) | 4 | MIT 완전 | 5 |
| 번들 크기 (50대 PC 로딩) | 0.15 | ~150KB+ | 2 | ~80KB | 4 |
| 일/주/월 뷰 완성도 | 0.20 | 표준, 매우 강력 | 5 | 보통 | 3 |
| 커스텀 렌더러 (원생 + 시간 셀) | 0.15 | `eventContent` prop 강력 | 5 | `components` prop 자유도 | 4 |
| TypeScript 지원 | 0.05 | 공식 @types 풍부 | 5 | 공식 .d.ts | 4 |
| Next.js static export 호환 | 0.10 | `'use client'` + dynamic import | 3 | `'use client'` 충분 | 4 |
| 한국어 i18n / 운영 안정성 | 0.05 | 한국 사용 사례 풍부 | 5 | 보통 | 3 |
| React 19 호환 | 0.05 | `@fullcalendar/react` 호환 보고됨 | 3 | 호환 보고됨 | 4 |
| 50대 친화 검증 사례 | 0.05 | Google Calendar 류 UI 친숙 | 5 | 보통 | 3 |
| **총점** |        |              | **3.95** |                | **3.85** |

→ 차이 0.10 — 통계적 동등. 2단계 SWOT 분석 중요.

## SWOT — FullCalendar

| 강점 | 약점 |
|------|------|
| 시각적 완성도 (Google Calendar 류 익숙함) | 번들 크기 큼 (~150KB+) |
| 풍부한 한국어 자료 + 한국 사용 사례 | premium 기능 분리 (시간 그리드 일부 — MVP 범위 외) |
| `eventContent` 커스터마이징 강력 | React 19 호환은 보고된 수준, 검증 필요 |

| 기회 | 위협 |
|------|------|
| 50대 사용자에게 친숙한 UI 패턴 | premium 라이선스 (미래 확장 시 비용) |
| 풍부한 community 지원 | 번들 추가 의존성 plugin 다수 |

## SWOT — React Big Calendar

| 강점 | 약점 |
|------|------|
| 가벼움 (~80KB) | 한국어 자료 부족 |
| MIT 완전 (라이선스 부담 0) | 시간 그리드 디자인 직접 보강 필요 |
| 컴포넌트 자유도 높음 | 50대 친화 UI 직접 구현 필요 |

| 기회 | 위협 |
|------|------|
| 완전한 MIT — 미래 확장 시 라이선스 비용 0 | 커뮤니티 작아 신뢰성 검증 부담 |
| 커스터마이징 자유도 | 학습 곡선 |

## 결정

**FullCalendar 채택** (사용자 결정 2026-05-26).

### 채택 사유

1. **50대 운영자 친화** — Google Calendar 류 UI 패턴은 원장에게 시각적으로 익숙. PRD §5.7 50대 사용자 친화 기준과 일치.
2. **시각 완성도** — 일/주/월 뷰 표준 구현이 직접 보강 작업 없이 즉시 사용 가능. PRD §4.6 요구사항 (시간대별 인원수, 원생 상세 팝업, 보강 관리 뷰) 충족도 높음.
3. **MVP 범위에서 MIT** — 사용 예정 기능 (`dayGrid`, `timeGrid`, `interaction`)은 모두 MIT. premium 기능 (resource timeline 등) 미사용.
4. **번들 크기 영향 미미** — 50명 미만 규모 + 데스크톱 앱 + 첫 로딩 1회. 50대 PC 환경에서 ~150KB 추가는 체감 없음.

### 패키지 구성

```json
{
  "@fullcalendar/react": "^6.x",
  "@fullcalendar/daygrid": "^6.x",
  "@fullcalendar/timegrid": "^6.x",
  "@fullcalendar/interaction": "^6.x",
  "@fullcalendar/core": "^6.x"
}
```

## 결과 (Consequences)

### 긍정

- 사용자 친숙도 + 시각 완성도 + 커스터마이징 강력 → T11 UI 구현 부담 감소
- 한국어 i18n + 한국 사용 사례 → 운영 안정성 확보

### 부정

- 번들 크기 증가 (~150KB) → static export 빌드 사이즈 영향. Next.js dynamic import 로 코드 분할 권장
- React 19 호환은 보고된 수준 — Sprint 11+ React 버전 업데이트 시 재검증 필요

### 정책

- premium 기능 (resource timeline, calendar feed 등) 도입 금지 — MIT 범위 유지
- T11 UI 구현 시 `'use client'` + `dynamic(() => import(...), { ssr: false })` 패턴 강제
- 한국어 로케일은 `@fullcalendar/core/locales/ko` 사용

## 미해결 사항

- **React 19 호환 검증**: T11 구현 시 실제 동작 확인. 호환성 문제 시 `@fullcalendar/react@6.1.x` 핀 + 후속 sprint 에서 업그레이드 검토.

## 참고

- FullCalendar 공식: https://fullcalendar.io/
- React Big Calendar 공식: https://jquense.github.io/react-big-calendar/
- 본 ADR 의 PI-03 사용자 결정 컨텍스트: `docs/sprint/sprint10/scope.md` Session #9
