---
Sprint: 5  |  Date: 2026-05-22  |  Session: #1
---

## 이번 세션에서 수정할 파일
<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

### T0 — Node 25/20 cross-OS 환경 호환
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| package.json | [1회] | dev script + cross-env devDep 추가 |

### T1 — tauri-plugin-single-instance 도입
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/Cargo.toml | [0회] | tauri-plugin-single-instance 의존성 |
| src-tauri/src/lib.rs | [1회] | plugin 등록 + 두 번째 인스턴스 핸들러 |
| ~~src-tauri/capabilities/default.json~~ | - | **불필요** — 플러그인이 JS API 미제공 (Tauri 공식 문서 확인) |
| ~~package.json (JS 바인딩)~~ | - | **불필요** — @tauri-apps/plugin-single-instance npm 미발행 |

### T1-sub — 강제 점유 버튼 검증/수정
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/app/lock/page.tsx | [1회] | 강제 점유 버튼 onClick 핸들러 검증 |
| src-tauri/src/commands/(lock 관련) | [0회] | force_acquire_lock IPC 동작 확인 |

### T2 — 마법사 redirect 수정
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src/app/setup/page.tsx | [0회] | handleComplete redirect /  → /settings |

### T3 + T4 — 시드 데이터 마이그레이션
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/migrations/201__update_seed_data.sql | [0회] | 표준교습비 + 결제수단 시드 변경 (V201 신규) |
| src-tauri/.sqlx/ | [0회] | 오프라인 캐시 갱신 (sqlx prepare 결과) |

### T5 — 통합 검증
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| (검증만, 코드 변경 없음) | - | cargo test / clippy / pnpm lint / build / tauri:dev |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 문서 (정책 약화 방지)
- [ ] PRD.md, CLAUDE.md — 본 sprint 범위 외
- [ ] V001~V200 기존 마이그레이션 — 수정 금지 (신규 V201로만 보정)

## 완료 기준 (이번 세션)
- [ ] T0: Node 25 + Node 20 양 환경 dev 서버 정상 기동
- [ ] T1: 동일 PC 다중 인스턴스 차단 동작 확인
- [ ] T1-sub: 양 PC 강제 점유 버튼 동작 확인
- [ ] T2: 마법사 완료 후 /settings 진입
- [ ] T3: standard_fees 시드 3/4/5/6시간 4종 (16/20/23/26만원)
- [ ] T4: payment_methods 시드 5종 (현금 비활성 + 4종 활성)
- [ ] T5: cargo test + clippy + pnpm lint/tsc/build 전수 통과
- [ ] .sqlx/ 오프라인 캐시 갱신 + 커밋

## 발견된 이슈
<!-- Step-back 프로토콜 발동 시 여기에 기록 -->

### A16: Next.js 15.3.6 추가 CVE 4건 (2025-12-11)
- CVE-2025-66478 (RCE, Critical), CVE-2025-55183 (Source Exposure, Medium), CVE-2025-55184 (DoS, High), CVE-2025-67779 (CVE-55184 완전 패치)
- 모두 RSC / 서버 응답 경로 취약점
- **SmartHB 영향: 없음** — `output: 'export'` + Tauri WebView 로컬 로드, 외부 네트워크 미수신, RSC 직렬화 경로 미사용
- 조치: T0 scope에서 업그레이드 제외 (환경 호환만 처리). 향후 별도 hotfix 또는 Sprint 6+ 진입 시 업그레이드 검토

## 신규 의존성 (사용자 허가 완료)
| 패키지 | 구분 | Task |
|--------|------|------|
| cross-env | npm devDep | T0 |
| tauri-plugin-single-instance | Rust crate | T1 |
| @tauri-apps/plugin-single-instance | npm dep | T1 |
