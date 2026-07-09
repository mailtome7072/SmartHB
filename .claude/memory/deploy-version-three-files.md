---
name: deploy-version-three-files
description: "프로덕션 배포 시 버전 번호를 갱신해야 하는 파일은 3곳 — package.json, Cargo.toml, tauri.conf.json"
metadata:
  type: feedback
  originSessionId: sprint19-deploy-v1.2.0-2026-07-09
---

배포 시 버전 번호는 `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` **3개 파일 모두** 동기화해야 한다.

**Why**: v1.2.0 배포 중 deploy-prod 에이전트가 `package.json`과 `Cargo.toml`만 올리고 `tauri.conf.json`을 빠뜨려, 첫 빌드의 인스톨러 아티팩트 파일명이 `SmartHB_1.1.0_*`로 잘못 생성됨(태그를 재생성해 수정). `tauri.conf.json`의 `version` 필드가 실제 인스톨러 파일명/앱 버전 표시에 쓰이는 SSOT라서 누락되면 CI가 통과해도 결과물이 틀어진다.

**How to apply**: 다음 배포부터 버전 bump 시 세 파일을 함께 grep해서 확인 — `grep -rn '"version"' package.json src-tauri/tauri.conf.json; grep -n '^version' src-tauri/Cargo.toml`. deploy-prod 에이전트에게 배포를 맡길 때도 이 3개 파일을 명시적으로 상기시키는 것이 안전.
