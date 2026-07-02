---
name: sprint-next-session
description: "✅ Sprint 18 완료(2026-07-01) + 수동검증 후속 버그 10건 수정(2026-07-02, 커밋 60ccdb4) — master. 남은 일: v1.1.0 프로덕션 배포. 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint18-dev-2026-06-30
---

## ✅ 2026-07-01 세션 — Sprint 18 완료

### 구현 내용 (7개 커밋 + review 수정 1개 커밋, master 머지 완료)

| 커밋 | 내용 |
|------|------|
| fix(sprint18-T0) | lock.rs STALE_THRESHOLD 86400, integrity rollback 유니크, startup spawn_blocking, setup WAL pool.close |
| feat(sprint18-T3) | 결제선생 카드사 드롭다운 optional 활성화 |
| feat(sprint18-T4+T5) | 수업 캘린더 기본 뷰 timeGridWeek + 일요일 시작(firstDay=0) |
| feat(sprint18-T6) | 주보기 수업시간 4색 + 원생별 개별 이벤트(균등 열) + 다중 슬롯 칩 |
| feat(sprint18-T7) | 월보기 셀 원생 이름 직접 표기(2열 그리드, 4색) |
| feat(sprint18-T8) | 학사 일정 변경 시 출결 자동 동기화 (fail-soft) |
| feat(sprint18-T9) | 교습일정 인쇄 버튼 + A4 HTML/CSS 출력 |
| fix(sprint18-review) | 코드리뷰 이슈 5건 수정 (d86c25b) |

## ✅ 2026-07-02 세션 — 수동 검증 + 후속 버그 10건 수정 (커밋 60ccdb4, master 직접 커밋)

pnpm tauri:dev 수동 검증 중 발견된 이슈를 그 자리에서 바로 수정. Hotfix 기준(파일3/50줄) 초과(9파일 +302/-125)라 원칙상 Sprint급이었으나, 미배포 상태의 Sprint18 후속 안정화라 "프로덕션 긴급"은 아니었음 — 사용자가 "master 직접 커밋" 선택(단일 개발자 간소화).

### 수정 내역
1. **ClassCalendar.tsx 대형 리팩터**: 주보기 hover 시 `Cannot read properties of undefined (reading 'split')` 크래시 — background 이벤트가 eventContent에서 extendedProps 없이 처리되던 버그. 겹침 열 배정을 하루 단위 interval greedy packing(`assignColumns`)으로 재구현 — 다중 슬롯(여러 시간대 걸친) 원생이 항상 동일 열에 표시. 주보기는 3명↑ 2열×N행 재배치, 일보기는 폭이 넓어 한 행 유지(view별 분기 필요). 월보기 배지 2열→3열, 색상은 `colorForDuration` SSOT로 주보기와 통일. 3시간(180분) 색상 amber→violet(교습일 셀배경 amber와 가독성 구분). 다중슬롯 화살표(↑/↓) 위치를 이름 뒤로.
2. **ThreeMonthCalendar.tsx(일정관리)**: 요일 시작 일요일 변경 누락 보정 — sprint18 T5는 ClassCalendar(수업관리)만 고쳤고 이 컴포넌트는 빠져있었음. 같은 종류의 UI가 여러 컴포넌트에 흩어져 있으면 일괄 변경 시 누락 위험 — 향후 "요일 순서" 등 유사 변경 시 grep으로 전체 컴포넌트 확인 필요.
3. **교습일정 인쇄**: 클릭 시 빈 미리보기 — **Next.js App Router에는 Pages Router 전용 `#__next` 래퍼가 없어**, 인쇄 CSS(`#__next > *:not(.wrapper){display:none}`)가 매칭 안 되고 대신 `body > *:not(#__next)`가 전체 앱 트리(인쇄 wrapper 포함)를 display:none 처리 — 조상이 숨겨지면 후손 `!important`도 무력화. `createPortal`로 `document.body` 직속 렌더링으로 해결. **App Router 프로젝트에서 프린트 CSS 작성 시 `#__next` 셀렉터 사용 금지 — Pages Router 전용 관례.** A4 세로→가로, 여백 15mm→0(브라우저 기본 머리글/바닥글 억제 목적, 완전 제어는 불가), 달력 셀 flex+percentage로 균등 배분. 인쇄 버튼도 교습기간 미등록 시 완전히 숨기지 않고 비활성화+툴팁으로 변경(발견성 개선).
4. **출결 동기화 실사용 버그**: `sync_attendance_on_schedule_change`가 "allows_regular_class=0 이벤트가 하나라도 있으면 OFF" 로직이라, 공휴일(OFF)+공휴수업일(ON)이 같은 날짜에 공존(V309에서 의도적으로 허용)해도 무조건 OFF 판정 → 출결 미생성. `allows_regular_class=1` 이벤트 존재 시 ON 우선하도록 수정 + 회귀 테스트(`sync_attendance_inserts_when_on_event_coexists_with_off_event`) 추가. **이미 등록된 기존 데이터는 소급 미적용 — 이벤트 삭제 후 재등록 필요.**
5. **결제선생 카드사 매칭**: 실사용 DB에 라벨 중복("결제선생"/오타 "결재선생" 등, code 값이 다른 레거시 항목) 존재 — `code==='pay_teacher'` 외 `label==='결제선생'` 폴백 매칭 추가. 근본 정리는 설정>코드관리에서 사용자가 직접 비활성화.
6. **수납관리/월별집계 UI 통일**: 년월/연도 select 드롭다운 → 일정관리와 동일한 "◀ 이전 / 년월 / 다음 ▶" 버튼 방식.

### 작업 중 이슈 (해결됨)
- 반복적인 `pnpm tauri:dev` 재시작 중 이전 프로세스(smarthb.exe, node.exe)가 완전히 종료되지 않고 좀비로 남아 포트 1420 충돌 발생 다회 — `taskkill //F //IM smarthb.exe`, `taskkill //F //IM node.exe`로 정리 후 재기동. Windows에서 Bash `run_in_background`로 띄운 pnpm/tauri 프로세스는 트리 종료가 불완전할 수 있음 인지.
- Bash 도구 cwd가 `cd src-tauri &&` 이후 세션 내내 유지되어 이후 git 명령이 repo root 기준으로 실패한 적 있음 — git 명령은 `git -C /c/Projects/SmartHB`로 절대경로 지정하거나 cwd 확인 후 사용.

## ⬜ 남은 일 (다음 세션)

1. **프로덕션 배포**: `"프로덕션 배포 준비해줘"` → deploy-prod 에이전트 (master → v1.1.0 태그)
2. **7/17 출결 재생성**: 사용자가 직접 학사일정 화면에서 7/17 공휴수업일 이벤트를 삭제 후 재등록 필요 (수정된 동기화 로직 적용을 위해)
3. **결제선생 중복 정리 확인**: 설정 > 코드 테이블 관리 > 결제 수단에서 코드값이 `pay_teacher`가 아닌 중복 "결제선생" 라벨 항목이 남아있다면 "사용" 체크 해제 안내 완료됨 — 사용자 후속 조치 여부 미확인

## 마이그레이션 현황
최신 **V309** (V308: 보강데이 중복허용, V309: 공휴일 중복허용). 이번 세션 스키마 변경 없음.
CLAUDE.md "현재 상태"는 sprint-close 시 V307로 표기 중 — 다음 문서 갱신 시 V309로 동기화 필요.

관련: [[workflow-no-pr]], [[exam-feature-cancelled]]
