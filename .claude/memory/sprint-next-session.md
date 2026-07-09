---
name: sprint-next-session
description: "✅ Sprint 19 완료 + develop 머지 완료(2026-07-09, b5bf1b6) — v1.1.0은 이미 master 배포됨. 남은 일: Sprint 19 프로덕션 배포(버전 태그 미정, v1.2.0 후보). 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint19-manual-feature-2026-07-09
---

## ✅ 2026-07-09 세션 — Sprint 19 → develop 머지 + 사용 매뉴얼 기능 추가

### 배경
Sprint 19(그리드 정렬/스크롤 통일, 인쇄 팝업 아키텍처 전환, 수업관리 캘린더 재설계, 학년 자동승급,
학교급 필터링, 수강생대장 출력)는 이미 `sprint19-close`/`sprint19-review` 완료 상태였으나,
그 이후에도 실사용 피드백 기반 `sprint19-followup` 커밋이 대량으로 쌓이며(90+ 커밋) develop에는
전혀 머지되지 않은 채 로컬 `sprint19` 브랜치에만 존재하던 상태였음 — v1.1.0(마스터 배포 완료)은
Sprint 17+18 범위였고 Sprint 19는 별도로 미배포 상태였음.

### 이번 세션 작업
1. **사용 매뉴얼 기능 신규 개발** (사용자가 sprint-planner 없이 대화 중 직접 구현 요청, "지금 이 대화에서 바로 구현" 선택):
   - `src-tauri/resources/manual/index.html` — 11개 화면 스크린샷 포함 HTML 매뉴얼(목차+노트+FAQ)
   - `tauri.conf.json`: `bundle.resources`로 번들, `plugins.shell.open`을 매뉴얼 파일 경로로 한정(최소 권한)
   - `capabilities/default.json`: `core:path:allow-resolve-directory` 권한 추가
   - `src/lib/tauri/index.ts`: `openManual()` 래퍼, `sidebar.tsx`: "설정"↔"종료" 사이 "매뉴얼" 버튼
   - **시행착오**: `shell.open()` 기본 정규식(`mailto:`/`tel:`/`http(s)://`만 허용)이 로컬 파일 경로를 거부 →
     `tauri.conf.json`의 `plugins.shell.open`에 커스텀 정규식(매뉴얼 파일 경로만 허용) 추가로 해결.
     필드명은 `plugins.shell.open` (❌ `plugins.shell.scope.open` 아님 — deserialize 에러로 즉시 발견됨)
2. **sprint19 → develop 직접 머지** (`--no-ff`, PR 없음, [[workflow-no-pr]] 정책):
   - 머지 전 develop에 `Cargo.lock` 버전 필드(1.0.0→1.1.0) 로컬 미커밋 변경 있었음(사전 `cargo check` 재동기화 결과) — `git stash`로 치우고 머지 후 동일 결과라 stash drop
   - 병합 커밋 b5bf1b6, `git push origin develop` 완료
   - 머지 후 self-verify 전부 통과: lint / tsc / clippy -D warnings / cargo test(431 passed) / pnpm build
   - `pnpm build`가 간헐적으로 `Cannot find module for page: /settings` 등 stale `.next` 캐시 오류 발생 — `.next`+`out` 삭제 후 재실행하면 해결되는 **일시적 캐시 플레이크**(코드 문제 아님), 재현되면 재시도만 하면 됨
3. **CDP 기반 실기동 검증**: `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"` + `chrome-remote-interface`로 사이드바 "매뉴얼" 버튼 실클릭 → 기본 브라우저(Edge)에 매뉴얼 열림 확인 (스크린샷으로도 렌더링 확인)
   - 잠금 화면(`/lock`, `check_auth_status`='locked')에 걸리면 `auto_unlock_with_keychain` invoke 후 `location.reload()` 필요 (raw invoke만으론 React 상태 미갱신 → `/lock`으로 되돌아감)

## ⬜ 남은 일 (다음 세션)

1. **Sprint 19 프로덕션 배포**: develop은 이미 push 완료(b5bf1b6). "프로덕션 배포 준비해줘" → deploy-prod 에이전트로 develop→master 직접 머지 + 버전 태그(v1.2.0 후보, 사용자 확인 필요) + GitHub Release
2. DEPLOY.md 현재 배포 현황 섹션은 "2026-07-08 | Post-v1.1 | Sprint 19"로 스테이징 검증 ✅ 완료 상태 — 프로덕션 배포 시 CV 체크리스트 이어서 진행

## 마이그레이션 현황
최신 **V310** (schools.school_type 자동 보정). develop에 반영 완료.

관련: [[workflow-no-pr]]
