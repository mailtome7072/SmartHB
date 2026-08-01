---
name: actual-pcs-both-windows
description: "실사용 PC 2대(교습소 원장 PC + 자택 PC) 모두 Windows 환경. 문서/메모리의 '자택 Mac' 가정은 틀림"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2de8b3ba-b8a7-4d3d-bec8-87adf8a9081b
  modified: 2026-08-01T08:44:36.334Z
---

실사용 배포 대상 PC 2대는 **둘 다 Windows** 환경이다 (교습소 원장 PC + 자택 PC).

**Why:** 그동안 문서·메모리(DEPLOY.md 실 PC 검증, [[sprint-next-session]] 등)가 "교습소 Windows + 자택 Mac"으로 가정해 왔으나, 사용자가 2026-08-01 실사용 검증 중 양쪽 모두 Windows임을 확정했다.

**How to apply:**
- 배포 후 실 PC 검증은 Windows 2대 기준으로 작성한다 (macOS .dmg 실환경 검증 항목 불필요).
- 단, 앱 자체의 macOS 지원(CLAUDE.md "macOS 12+ 동시 지원", GitHub Actions dmg 빌드)은 별개 — 사용자 지시 없이 macOS 지원을 제거하지 말 것. 실사용 환경 ≠ 지원 플랫폼.
- Mac 전용 트랩 메모리([[keyring-v3-features-trap]]의 apple-native 등)는 크로스빌드 유지 목적상 계속 유효.
