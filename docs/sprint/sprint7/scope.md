---
Sprint: 7  |  Date: 2026-05-22  |  Session: #3
---

> Sprint 7 Session #3 — T3 단독 (device_id 영속화: stale lock 안전성).
> Issue 8 carry-over 해소 + R37 (device.id 경로 정책) 결정. 예상 2h.

## 이전 세션 결과

- Session #1 (2026-05-22, `8eb1c92`): T1 — macOS Keychain 호출 통합 캐싱 + CredentialCache 도입
- Session #2 (2026-05-22, `4178324`): T2 — salt.bin 이전 + Critical 보안 패치 6건 (S-T2-1~6) + I-S2-1 동행 패치
  - cargo test 160 passed (cipher off) / 121 passed (cipher on), clippy clean

## 이번 세션의 Task 선정

| Task | 작업 | 예상 소요 |
|------|------|---------|
| **T3** | device_id 영속화 — `OnceLock<Uuid>::new_v4()` → 파일 저장 + `app_config_dir/device.id` | 2h |

> 사용자 결정 (2026-05-22): Session #3 = T3 단독. T4~T10 + carry-over 9건은 후속 세션.

## 설계 결정 (T3)

### 경로 정책 (R37 — 사용자 결정 2026-05-22)
- **채택**: **OS `app_config_dir`** 하위 `device.id` — macOS `~/Library/Application Support/SmartHB/device.id`, Windows `%APPDATA%\SmartHB\device.id`
- **이유**: device.id 는 양 PC **구분** 용도 — 클라우드 동기화 폴더에 두면 양 PC 가 동일 UUID 로 sync 되어 식별 불가. OS 로컬은 클라우드 매니페스트에서 자동 제외되어 안전.
- **기각된 대안**:
  - 클라우드 폴더 + `.nosync` 마킹: 서비스별 일관성 없음 (MYBOX/Dropbox/iCloud 제각각)
  - `~/.smarthb/device.id`: OS 관례 위반

### 영속화 절차
1. 앱 시작 시 `lib.rs::setup` hook 이 `app.path().app_config_dir()` 를 lock 모듈에 전달 (`init_device_id_path`).
2. 첫 `device_id()` 호출 시 파일 로드 시도:
   - 파일 존재 + 유효 UUID 파싱 → 해당 UUID 캐시
   - 파일 부재 → 새 UUID v4 생성 + atomic write (tmp → rename + sync_all, T2 패턴 답습)
   - 파일 손상 (UUID 파싱 실패) → 새 UUID 재생성 + 파일 재기록 (graceful fallback)
3. 후속 호출은 process-내 OnceLock 캐시 hit (현재 동작 보존).

### `init_device_id_path` 미호출 fallback
테스트 환경 또는 setup 진입 전 호출되면 임시 UUID v4 1회 생성하여 메모리에만 보관 (파일 미작성). lib.rs setup 이 정상 실행되면 production 에는 영향 없음 — `lock::lock_info_is_self_when_device_id_matches` 류 테스트 호환.

### 신규 의존성
- 없음 — 기존 `uuid` + `std::fs` 사용.

## 이번 세션에서 수정할 파일

<!-- 수정 횟수가 [3회 ⚠️]에 도달하면 loop-detection 스킬 즉시 실행 -->

| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| src-tauri/src/commands/lock.rs | [3회 ⚠️] | `device_id()` 파일 영속화 + `init_device_id_path` 신규 |
| src-tauri/src/lib.rs | [1회] | setup hook 에서 `app_config_dir/device.id` 전달 |
| docs/sprint/sprint7/scope.md | [1회] | 본 세션 추적 |

## 수정하지 않을 파일 (Forbidden Areas 포함)

- [ ] `.github/workflows/` — CI/CD 파이프라인 (hook 차단)
- [ ] `SETUP.sh` — 초기화 스크립트 (hook 차단)
- [ ] `docs/harness-engineering/` — Harness 정책 (경고)
- [ ] `src/` — 본 세션 프론트엔드 변경 없음
- [ ] `src-tauri/Cargo.toml` — 신규 의존성 없음
- [ ] `src-tauri/migrations/` — DB 스키마 변경 없음
- [ ] `src-tauri/src/commands/auth.rs`, `recovery.rs`, `paths.rs` — T2 완료, T3 영향 없음
- [ ] `src-tauri/src/commands/setup.rs` — `app_config_dir` 사용 패턴만 참고 (수정 없음)

## 완료 기준 (이번 세션)

### T3 — device_id 영속화 (sprint7.md L101-120)
- ✅ AC-T3-1: 앱 시작 → 종료 → 재시작 후 동일 device_id 유지 (`device_id_persists_across_load_calls`)
- ✅ AC-T3-2: `{app_config_dir}/device.id` 파일에 UUID 문자열 저장 (`device_id_file_contains_parseable_uuid`)
- ✅ AC-T3-3: 비정상 종료 후 재시작 시 stale lock 자동 점유가 "본 디바이스" 락으로 판정 — 동일 UUID 유지 + 기존 `acquire_lock_atomic` stale 분기 + `is_self()` 로 자동 충족
- ✅ AC-T3-4: PC-A 와 PC-B 의 device.id 가 서로 다른 UUID — app_config_dir 분리 (`device_id_differs_across_app_config_dirs`)
- ✅ AC-T3-5: device.id 파일 손상 시 새 UUID 재생성 + 파일 재기록 (`device_id_regenerates_on_corruption`, `_on_empty_file`)

### 세션 종료 조건
- ✅ Self-verify: cipher off 166 passed / cipher on 127 passed / clippy clean (양쪽)
- ✅ simplify 검토 — `device_id()` 로드 분기 단일 책임 유지, T2 atomic write 패턴 일관성. 빈 `reset_device_id_for_tests` 헬퍼 제거.
- ⬜ 단일 커밋 (2파일 + scope.md)

## 발견된 이슈

(없음 — Step-back 트리거 발생 시 여기에 기록)

## carry-over (Session #2 발견 9건, 후속 세션 처리)

I-S2-1 은 Session #2 에서 동행 패치 완료. I-S2-2 ~ I-S2-10 은 후속 세션 또는 hotfix 로 처리 — 상세는 git 이력 `4178324` 또는 sprint7 마무리 sprint-close 시 통합 정리.
