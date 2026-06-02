---
name: sprint13-pin-optional
description: "Sprint 13 예정 — 실행 시 PIN 인증 옵션화(C안: 키체인 자동 스킵). 설계·제약·ADR 요건 확정됨"
metadata: 
  node_type: memory
  type: project
  originSessionId: d4a8a2df-8629-4fbd-a1c6-d51d1d4eaaeb
---

**결정(2026-05-31)**: "실행 시 PIN 입력" 을 설정으로 옵션화한다. **C안(키체인 자동 스킵)** 채택.
Sprint 12(공지문) 검증·마무리 후 **Sprint 13 이후 작업**으로 진행. (단원평가 등 Phase 5 일정과 우선순위는 그때 조율)

**Why:** 매 실행 PIN 입력 부담을 줄이되, 하드코딩 시크릿/평문 DB/양 PC 공유 깨짐 없이 데이터 보호를 유지하는 유일한 안. (대안 비교: 하드코딩 PIN=암호화 무력화+규칙위반, 평문+무인증=PII 노출+대규모 제거 → 모두 기각)

## 설계 (확정)
- **동작**: 설정 토글 `실행 시 PIN 인증 사용` 기본 **ON**.
  - ON: 현행대로 LockScreen에서 PIN 입력 → verify → 키 확보.
  - OFF: startup에서 **OS Keychain에 이미 저장된 유도 키를 그대로 로드**해 잠금 해제, **PIN 입력 단계 스킵**. (`verify_password` 의 비교 단계만 생략 — 키는 이미 키체인에 있음, 암호화 로직 변경 없음)
- **토글 저장 위치 = DB 밖**: 암호화 DB(app_settings)에 두면 닭-달걀(키 없이 못 읽음). → `app_config_dir/config.json`(현 cloud_folder_path 보관처)에 `skip_pin_on_launch` 같은 플래그로 저장. **PC별(per-device)** 설정 (클라우드 동기화 X).
- **새 PC/키체인 비어있음**: 키체인에 키가 없으면 스킵 불가 → **최초 1회 PIN 입력 필수**(그 PC 키체인 시딩). 즉 "기기당 한 번"만 입력, 이후 스킵.
- **최초 설치**: PIN 설정은 그대로 유지(키 출처 확보).

## 구현 범위 (Sprint급)
- 백엔드: config.json 플래그 get/set IPC(언락 전 호출 가능), startup에 "키체인 키로 무입력 잠금해제" 경로 추가(`app_startup_sequence` 가 password 없이 키체인 키로 동작하는 변종 또는 분기). `auth::get_cached_or_load_key`(cipher) 재사용 검토.
- 프론트: 설정 화면에 토글, 앱 진입 흐름(LockScreen 렌더 분기)에서 플래그+키존재 확인 후 스킵.
- **ADR 필수**: PRD §5.5(인증 의무) 완화 결정 기록 — "기기별 선택적 PIN 게이트, 데이터 보호는 OS 계정+키체인 ACL로 위임" 트레이드오프 명시. ADR-007(PIN) 후속.
- 복구코드(12자리) 흐름 유지 — PIN 분실 위험 누적 대비 안내 강화.

## 주의 (구현 시 함정)
- macOS 키체인 접근 다이얼로그: 스킵해도 첫 접근 시 OS 프롬프트 가능 → "항상 허용" 필요(서명 안정 빌드). dev 빌드는 재컴파일마다 재프롬프트 가능.
- 토글을 절대 DB 안에 저장하지 말 것(닭-달걀).
- 키는 키체인에 PC별 저장(동기화 X) — "한 번 끄면 모든 PC 무입력"이 아니라 "키 있는 그 PC에서만".

관련: [[sprint-next-session]]
