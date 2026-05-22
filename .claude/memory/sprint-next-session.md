---
name: sprint-next-session
description: "Sprint 6 완료 (Task 12/12, 2026-05-22). 다음: /sprint-dev 7 → Sprint 7 (출결 관리 — Phase 2 나머지)"
metadata: 
  node_type: memory
  type: project
  originSessionId: sprint6-close
---

Sprint 6 완료. **브랜치 `sprint6` → `develop` 머지 완료** (머지 커밋 `dc3139e`).
sprint-review 에이전트 실행 대기 중.

## Sprint 6 최종 현황

| 항목 | 내용 |
|------|------|
| 완료일 | 2026-05-22 |
| 세션 수 | 9 |
| 커밋 수 | 16 (feat 9 + docs 7) |
| Task | 12/12 완료 |
| cargo test | 146 passed |
| 버전 | 0.3.0 예정 (Unreleased) |

## Sprint 7 진입 시 우선 액션

1. sprint-review 에이전트 먼저 실행 (코드 리뷰 + 자동 검증)
2. DEPLOY.md `⬜ sprint-review` 완료 후 `/sprint-dev 7` 입력
3. Sprint 7: 출결 관리 (Phase 2 나머지) — Phase 2 마일스톤(M3)

## Sprint 7 핵심 작업 (참고)

- DB 마이그레이션 V005: regular_attendances + makeup_attendances 테이블
- 출결 생성 로직 (generate_attendances, get_attendance_grid)
- 출결표 UI (행×원생, 열×일자, 50명×31일 렌더링 1초 이내)
- 캘린더 라이브러리 ADR (FullCalendar vs React Big Calendar)
- **carry-over**: A17(salt.bin 이전 Keychain → cloud) 별도 hotfix 권고 — 사용자 데이터 생기기 전 처리 권장

## Sprint 6 발견 이슈 (Sprint 7 / 이후 참조)

1. **공휴일 시드 매년 1월 갱신**: V401+ 마이그레이션 정책 (ADR-005)
2. **Hook 정규식 수정 완료**: `.env.example` 차단 회귀 해소
3. **URL.searchParams base64 재인코딩**: 외부 API 인증키 raw string concat 필요
4. **SQLite VALUES...AS alias 미지원**: column1/column2 자동 명명 우회
5. **HTML button 중첩 회피**: 외부 button → div+role/tabIndex 패턴

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, 직접 머지
- **`/sprint-dev` 사용자 직접 입력** — 에이전트 호출 금지 (CLAUDE.md)
