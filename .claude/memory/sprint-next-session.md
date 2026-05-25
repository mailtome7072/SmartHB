---
name: sprint-next-session
description: "Sprint 9 구현 완료 (T1~T12). 다음: sprint-close → sprint-review → develop 머지"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint9-t12-complete
---

Sprint 9 — Phase 3 핵심 도메인(보강) 완성. 모든 Task 완료. 다음 세션은 **sprint-close 에이전트**로 진입.

## Sprint 9 최종 현황

| Task | 내용 | 상태 | 커밋 |
|------|------|------|------|
| T1 | PI-02 + 스키마 검증 + scope.md | ✅ | `6494f2b` |
| T2 | 백엔드 IPC — 미처리 결석 + 보강 가능 일자 + A43 | ✅ | `14f583e` |
| T3 | 백엔드 IPC — 보강 등록 + 매칭 트랜잭션 | ✅ | `e0e3659` |
| T4 | 백엔드 IPC — 취소 + 미등원 + 일괄 | ✅ | `a62150d` |
| T5 | TS IPC 래퍼 + 도메인 타입 | ✅ | `6f761f5` |
| T6 | 보강 등록 (개별) UI | ✅ | `76c2ede` |
| T7 | 보강데이 일괄 + 보강 관리 + A41 라벨 | ✅ | `ef06b43` |
| T8 | 결석 이력 조회 + AbsenceHistoryDialog | ✅ | `f2a5689` |
| T9 | 통합 검증 + A39/A40 + AC 마킹 | ✅ | `70c856a` |
| **T10** | **I3 보강 가능일 정의 확장 + T3 검증 3 폐기** | ✅ | `4b21450` |
| **T11** | **I1/I2/I4-I8 시간 단위 + UX 보강** | ✅ | `a2e3169` |
| **T12** | **J1~J10 도메인 모델 정제 + tooltip 줄바꿈** | ✅ | `e6e3a39` |

## 최종 자동 검증 결과

- cargo test cipher off **254 passed** / cipher on **133 passed**
- cargo clippy cipher off/on clean
- pnpm lint / tsc --noEmit clean
- pnpm build 12 라우트 static export 성공

## 사용자 시각 검증 (7라운드 누적, 2026-05-24~26)

- 1차: I1~I8 (8건) — Sprint 9 확장 결정 (+10h, 누적 45h)
- 2/3차: J1~J10 (10건) — Session #11 흡수
- 최종 "검증완료" 보고 (2026-05-26)

## Sprint 9 핵심 도메인 결정

| 결정 | 영향 |
|------|------|
| **보강 가능일 (I3)** | 케이스 A (평일 + 보강불가 코드 없음) OR 케이스 B (allows_makeup_class=1 명시). study_periods 제약 제거 |
| **정규 수업 요일에도 보강 허용** | T3 검증 3 폐기 (수업 후 추가 보강 진행 가능) |
| **시간 단위 시간(hours)** | UI 입력/표시는 시간 단위, 백엔드 class_minutes(분) 유지 |
| **결석 셀 라벨 통일** | absent/makeup_done 모두 '결석' 표기. makeup_done 배경 emerald (보강 셀과 동일) |
| **보강 삭제 진입점** | 보강일(emerald) 셀 클릭 (기존 결석 셀에서 이동) |
| **보강 미등원 폐기** | 사용자 결정: 보강은 결과 기록 의미 — markMakeupAbsent UI 호출 제거 |
| **보강데이 일괄 기능 폐기** | BatchMakeupDialog 삭제 (헤더 버튼/타입/래퍼 모두 정리) |

## 다음 세션 (sprint-close) 진입 액션

새 대화에서:

> "sprint-close 실행해줘."

sprint-close 에이전트가 처리:
1. ROADMAP.md Sprint 9 → ✅ 완료 마킹
2. CHANGELOG.md `[0.4.x]` 항목 추가 (보강 도메인 + UX 보강 + 도메인 모델 정제)
3. DEPLOY.md 신규 체크리스트 작성
4. sprint9.md DoD 모든 항목 ✅ 확인
5. **PR 단계 생략** — 단일 개발자 정책 ([[workflow-no-pr]]). sprint9 → develop 직접 머지 안내

sprint-close 완료 후:

> "sprint-review 실행해줘."

sprint-review 에이전트가 4종 산출물 작성 (A40):
- `docs/test-reports/sprint9.md`
- `docs/risk-register/2026-05-26.md`
- `docs/sprint-retrospectives/sprint9-retrospective.md`
- `docs/code-reviews/sprint9.md`

## Sprint 10 carry-over 메모

- `mark_makeup_absent` 백엔드 IPC + audit variant — dead code 정리
- `batch_create_makeups` 백엔드 IPC + 관련 코드 정리
- `makeup_attendances.status='makeup_absent'` CHECK 제약 마이그레이션 정리 (선택)
- 원래 Sprint 10 마일스톤: 소멸 자동 전이 + 퇴교 보강 처리 + 캘린더 뷰

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint9 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- Capacity 실측 ~52h (38h 대비 +14h, 시각 검증 흡수 결과)
