---
name: tauri-window-confirm-blocked
description: Tauri WebView는 window.confirm/alert/prompt를 차단(dialog.confirm not allowed) — 확인창은 커스텀 모달로 구현
metadata: 
  node_type: memory
  type: reference
  originSessionId: 2378c691-d1e2-4779-9e75-29cb20834c60
---

Tauri v2 WebView에서 `window.confirm()` / `window.alert()` / `window.prompt()` 호출 시
**`Error: dialog.confirm not allowed. Command not found`** 로 차단된다. WebView가 이들을
dialog 플러그인 IPC로 가로채는데 capabilities 권한이 없으면 막힌다 (개발/실앱 모두).

**대응**: JS 네이티브 confirm/alert 쓰지 말고 프로젝트 표준 **커스텀 모달**로 구현한다.
이 프로젝트의 확인 다이얼로그 패턴 = plain `fixed inset-0 ... bg-black/50` 오버레이 +
`role="dialog" aria-modal="true"` + 버튼(min-h-44px). 예: `notices/page.tsx` pendingAction,
`settings/info` 이미지 삭제, Sprint 16 `UnsavedNavDialog`(공용 미저장 이동 확인).

동기 반환이 필요한 가드(예: `unsavedGuard: (href)=>boolean`)는 confirm 결과를 직접
못 받으므로, 차단 후 store에 대상값 세팅 → 별도 모달이 비동기로 결정하는 구조를 쓴다.

관련: [[sprint-next-session]]
