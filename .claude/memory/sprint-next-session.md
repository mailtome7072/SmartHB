---
name: sprint-next-session
description: "Sprint 12 진행 중 — 회사 PC 작업 분량 commit/push 완료, 집에서 사용자 검증 이어서. 잔여: 검증 피드백 처리 → sprint-close → sprint-review"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint12-company-pc-session
---

Sprint 12 (공지문) 본업 + 회사 PC 환경 scope-외 추가 작업 6개 commit 으로 정리되어 `origin/sprint12` 에 push 완료 (2026-06-01). **사용자 검증은 회사 PC 에서 시작했고 집에서 이어진다.**

## 회사 PC 세션 (2026-06-01) — 추가 작업 6 commit

| 커밋 | 영역 | 핵심 |
|------|------|------|
| `abccef3` | refactor(auth) | 복구 코드(12자리) 시스템 전면 제거 + `change_pin` IPC + 설정 → 'PIN 번호 변경' 메뉴 |
| `c26f99d` | feat(attendance,billing) | 대상월/청구년월 콤보를 `study_periods` 기반으로 (없으면 현재 년월 fallback) + mount 마다 invalidateQueries 강제 갱신 |
| `0cd04fd` | fix(schedules) | 월보기 캘린더 일자별 인원수 초기 누락 hotfix — `dayCellDidMount` 1회 한계 → `useEffect` 로 dayInfo 변경 반응 |
| `39794bc` | feat(notice) | 좌우 패널 swap (order-* 유틸) + 배경서식 미선택 시: 공지문 이름 자동 공란, 저장 비활성, 데이터 필드 체크박스 unchecked 강제 |
| `cf92ebc` | chore(menu) | 출결관리/학사 순서 swap, '학사 스케줄' → '학사 관리' |
| `c93287b` | docs(sprint12) | scope.md 갱신 — 본 세션 scope 외 변경 일괄 기록 |

## 회사 PC 한정 진행 — 데이터/계정 측면

- **회사 PC 진입 차단 해소**: 옛 텍스트 암호로 설정된 회사 DB → 6자리 PIN UI 와 충돌. devtools console 에서 `await window.__TAURI_INTERNALS__.invoke('set_password', { password: '<새 PIN>' })` 1회 호출로 강제 재설정. DB 평문(SQLCipher off 빌드)이라 데이터 보존, salt+keychain 만 새 PIN 기준으로 덮어씀.
- 회사 DB 와 집 DB 는 **다른 cloud path** 라 서로 독립. 양쪽 PIN 을 동일하게 설정함.

## 보안 정책 결정 (사용자 직권)

- 복구 코드 시스템 사실상 보호 효과 없음 + PRAGMA rekey 미구현이라 cipher on 빌드에서 위험 → 전체 제거
- PIN 변경은 **반드시 현 PIN 확인** 후 `change_pin` IPC 호출 (`set_password` 는 사용자 화면에 노출되지 않음 — devtools 우회 가능성은 단일 사용자 모델에서 수용)
- `argon2` crate 의존 제거, `RecoveryCodeIssued` audit variant 제거, `auth.rs` 의 recovery 관련 주석 정리

## 사용자 검증 진행 중 — 집에서 이어갈 항목

회사 PC 에서 일부 항목까지 검증 진행. 집 PC 에서는:

1. **PIN 변경 흐름**: 설정 → 'PIN 번호 변경' 카드 → 현 PIN + 새 PIN×2 → 저장. 잘못된 현 PIN / 형식 오류 / 확인 불일치 에러 메시지 확인.
2. **대상월/청구년월 콤보**: 학사 관리에서 교습기간 추가/삭제 → 출결관리/청구관리 메뉴 클릭 시 즉시 반영되는지.
3. **수업관리 월보기 캘린더**: 초기 진입 시 일자별 인원수 배지 표시 / 주·일 전환 / 데이터 갱신 시 즉시 반영.
4. **공지문 페이지**:
   - 좌우 패널 swap 적용 확인 (좌: 공지문 이름/저장/템플릿, 우: 청구년월/원생 리스트)
   - 배경서식 선택 해제 시 공지문 이름 공란 + 저장 비활성 + 데이터 체크박스 모두 해제
   - 배경서식 다시 선택 후 자유롭게 체크 가능
5. **메뉴 순서/라벨**: 사이드바·글로벌 검색에 '출결 관리/수업 관리/학사 관리' 순서, '학사 관리' 라벨 적용.
6. **공지문 본업(Sprint 12 메인)**: 일괄 이미지 생성, 템플릿 저장/불러오기/삭제 등 회귀.

## 집에서 시작 절차

```
git fetch origin
git checkout sprint12
git pull
pnpm install   # 의존성(argon2 제거 + html-to-image / react-rnd) 동기화
pnpm tauri:dev # 또는 /restart
```

집 DB 는 영향 없음 (cloud path 별도). PIN 도 그대로.

## 다음 단계 액션

1. 사용자 검증 완료 + 잔여 이슈 처리
2. `sprint-close` 실행 — ROADMAP / CHANGELOG 갱신 + PR 대신 develop 직접 머지 ([[workflow-no-pr]])
3. `sprint-review` 실행 — 코드 리뷰 + 자동 검증 + 회고

## 정책 재확인

- **PR 단계 생략** ([[workflow-no-pr]])
- **메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 갱신 후 commit
- **cipher on 검증**: 본 세션은 dev (cipher off) 로 진행. release 빌드 / CI 는 cipher on 으로 검증되는 게이트 유지 ([[cipher-test-gate-trap]])

관련: [[workflow-no-pr]], [[cipher-test-gate-trap]], [[keyring-v3-features-trap]]
