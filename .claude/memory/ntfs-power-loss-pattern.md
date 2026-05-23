---
name: ntfs-power-loss-pattern
description: NTFS power-loss 시 fs::write+rename 후 데이터가 NULL 바이트로 손상되는 패턴 — 모든 atomic write 헬퍼에 손상 감지+fallback 필요
metadata: 
  node_type: memory
  type: project
  originSessionId: 39e31ca0-8b92-444b-9a34-e09f9c9fb022
---

PC 강제 종료(2026-05-21 사고) 시 `fs::write(tmp) + fs::rename(tmp, path)` 원자적 쓰기 패턴이 깨질 수 있다. Windows NTFS 가 메타데이터(파일 길이·이름)는 저널에 커밋했으나 데이터 페이지는 캐시에만 있고 디스크에 미반영된 상태에서 전원이 끊기면, 재부팅 후 파일은 정상 길이(예: 90 바이트)로 존재하지만 내용은 **전부 NULL(0x00)** 로 남는다.

실측 사고: `%APPDATA%\co.kr.ubcare.smarthb\config.json` 90 바이트 = `00 00 00 00 ...`. `serde_json::from_str` 파싱 실패 → `AppError::Config` → 사용자는 "설정 정보를 불러오는 중 오류" 만 보고 앱 사용 불가.

**Why:** atomic rename 은 두 파일이 디스크에 sync 됐다는 보장은 하지 않는다. `fs::sync` / `fsync` 호출이 없으면 NTFS journal 의 메타데이터만 안전하다.

**How to apply:** 본 프로젝트에서 사용자 데이터/설정을 파일로 영속화하는 모든 모듈에 손상 감지 + 자동 복구 fallback 을 추가한다.

## 적용 완료
- `src-tauri/src/commands/setup.rs` `read_status_from_path()` (`ec7ffbf`, 2026-05-21) — `is_corrupted()` (빈 파일 또는 all-zero) + 파싱 실패 → `config.json.corrupted-{unix_ts}` 백업 후 `Default` fallback. 단위 테스트 6 건.
- `src-tauri/src/commands/lock.rs` `parse_lock_info()` (`fb513b8`, 2026-05-21) — `is_lock_corrupted()` + 파싱 실패 → `Ok(None)` 반환 (절대 에러 X) → `acquire_lock_atomic` 이 새 락 자동 작성. `read_lock_info` 가 `app.lock.corrupted-{ts}` 백업. **트랩**: 파싱 실패를 `AppError::Lock` 으로 wrap 하면 `error.rs::user_message` 가 "다른 컴퓨터에서 사용 중" 으로 잘못 표시 — 절대 에러 던지지 말 것. 단위 테스트 5 건.

## 적용 후보 (Sprint 4 이후)
- `salt.bin` 이전 후 (R12, R18) — keychain → cloud 폴더 이전 시 동일 패턴 적용 필수. salt 손상 = 모든 사용자 데이터 영구 잠금 위험.
- 백업 메타데이터 파일 (`backup/*.meta` 등) — 백업 인덱스 손상 시 복구 흐름 차단 방지.
- 락 파일 `app.lock` — heartbeat 갱신 중 손상되면 stale 락 강제 해제 분기에 영향. 단, 락은 짧은 주기로 재생성되므로 우선도 낮음.

## 코드 패턴
관련: [[workflow-no-pr]]

```rust
fn read_xxx_from_path(path: &Path) -> XxxStatus {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return XxxStatus::default(),
        Err(e) => { eprintln!("[xxx] 읽기 실패 (default fallback): {}", e); return XxxStatus::default(); }
    };
    if is_corrupted(&bytes) {
        backup_corrupted(path);  // .corrupted-{unix_ts} rename
        return XxxStatus::default();
    }
    match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(e) => { backup_corrupted(path); XxxStatus::default() }
    }
}

fn is_corrupted(bytes: &[u8]) -> bool {
    bytes.is_empty() || bytes.iter().all(|&b| b == 0)
}
```

`is_corrupted` 휴리스틱은 빈 파일/all-zero 만 컷. 부분 손상(예: 일부 페이지만 NULL)은 파싱 단계에서 자연스럽게 fallback 분기로 들어간다.
