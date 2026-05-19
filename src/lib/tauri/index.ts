/**
 * Tauri IPC 추상화 레이어
 * 컴포넌트에서 invoke() 직접 호출 금지 — 이 파일을 통해서만 Tauri 커맨드 호출
 */

import type {
  AuditLogEntry,
  AuthStatus,
  BackupLayer,
  BackupMetadata,
  IntegrityCheckResult,
  IntegrityMode,
  LockStatus,
  RestoreResult,
  SyncStatus,
} from '@/types'

let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null

async function getInvoke() {
  if (typeof window === 'undefined') return null
  if (!invoke) {
    try {
      const tauri = await import('@tauri-apps/api/core')
      invoke = tauri.invoke
    } catch {
      // 브라우저 환경 (Tauri 없이 실행 시) — 개발용 mock 가능
      invoke = null
    }
  }
  return invoke
}

export async function greet(name: string): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return `[개발 모드] 안녕하세요, ${name}!`
  return inv('greet', { name }) as Promise<string>
}

/**
 * 현재 인증 상태를 조회한다.
 *
 * - `'not-initialized'`: 비밀번호 미설정 — 최초 설정 모드 진입
 * - `'locked'`: 비밀번호 설정됨, 잠금 해제 모드 진입
 *
 * 브라우저 개발 모드(Tauri 없이)에서는 `'not-initialized'` 를 반환하여 UI 흐름 테스트 가능.
 */
export async function checkAuthStatus(): Promise<AuthStatus> {
  const inv = await getInvoke()
  if (!inv) return 'not-initialized'
  return inv('check_auth_status') as Promise<AuthStatus>
}

/**
 * 최초 비밀번호를 설정한다 (NotInitialized → Locked 전이).
 *
 * 이미 설정되어 있으면 백엔드에서 에러 반환 → throw.
 */
export async function setPassword(password: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) {
    // 개발 모드 — no-op
    return
  }
  await inv('set_password', { password })
}

/**
 * 비밀번호로 DB 잠금을 해제한다 (Locked → Unlocked 전이).
 *
 * 비밀번호 불일치 시 throw — 호출자가 catch 하여 사용자 친화 메시지 표시.
 */
export async function unlockDb(password: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) {
    // 개발 모드 — 어떤 비밀번호든 허용
    return
  }
  await inv('unlock_db', { password })
}

/**
 * 12자리 복구 코드를 발급한다 (PI-07 PRD v1.5.1).
 *
 * 평문 코드는 호출 직후 화면에 1회 표시하고 즉시 폐기해야 한다 — React state 보유는 표시
 * 중에만, 사용자가 "확인" 클릭 시 빈 문자열로 덮어쓰기 권장.
 *
 * 이미 발급된 코드가 있으면 무효화하고 새 코드를 반환 (재발급 정책).
 */
export async function generateRecoveryCode(): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return 'DEVM-ODEX-TEST'
  return inv('generate_recovery_code') as Promise<string>
}

/**
 * 사용자가 입력한 복구 코드를 검증한다 (constant-time).
 *
 * 공백·하이픈은 백엔드에서 자동 제거되며 대문자로 통일된다.
 */
export async function verifyRecoveryCode(code: string): Promise<boolean> {
  const inv = await getInvoke()
  if (!inv) return code.replace(/[-\s]/g, '').toUpperCase() === 'DEVMODEXTEST'
  return inv('verify_recovery_code', { code }) as Promise<boolean>
}

/**
 * 복구 코드로 비밀번호를 재설정한다.
 *
 * 코드 검증 실패 시 throw. 성공 시 keyring 의 salt + key 가 새 비밀번호로 갱신된다.
 * SQLCipher DB rekey 는 T9 통합 시점에 추가된다.
 */
export async function resetPasswordWithCode(code: string, newPassword: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('reset_password_with_code', { code, newPassword })
}

/**
 * 현재 app.lock 점유 상태를 조회한다 (T6 PRD §5.3).
 *
 * 브라우저 개발 모드에서는 `free` 를 반환하여 UI 흐름만 검증 가능.
 */
export async function checkLockStatus(): Promise<LockStatus> {
  const inv = await getInvoke()
  if (!inv) return { kind: 'free' }
  return inv('check_lock_status') as Promise<LockStatus>
}

/**
 * app.lock 점유를 시도한다.
 *
 * `force=true` 는 5분 이상 미갱신(stale) 락만 강제 점유 — 정상 동작 중인 다른 디바이스
 * 락은 백엔드가 보호한다. UI 가 사전 사용자 확인 후 force=true 호출.
 */
export async function acquireLock(force: boolean): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('acquire_lock', { force })
}

/**
 * 본 디바이스의 app.lock 을 해제한다 (다른 디바이스 락은 보호됨).
 */
export async function releaseLock(): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('release_lock')
}

/**
 * 지정 계층에 SQLCipher DB 백업을 생성한다 (T7 PRD §5.3/§5.4, ADR-003).
 *
 * 백엔드가 4계층 순환 삭제까지 자동 수행한다 — 호출자는 계층 정책 미관여.
 * `cipher` feature off 개발 빌드에서는 백엔드가 사용자 친화 안내 메시지로 reject.
 * 브라우저 개발 모드에서는 더미 메타데이터를 반환하여 UI 흐름만 검증 가능.
 */
export async function createBackup(layer: BackupLayer): Promise<BackupMetadata> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      path: `[개발 모드] ./SmartHB-data/backup/${layer}/app_dev.db`,
      layer,
      created_at: new Date().toISOString(),
      size_bytes: 0,
    }
  }
  return inv('create_backup', { layer }) as Promise<BackupMetadata>
}

/**
 * 백업 파일 목록을 시간 역순으로 조회한다.
 *
 * `layer` 미지정 시 4계층 전체. 브라우저 개발 모드에서는 빈 배열.
 */
export async function listBackups(layer?: BackupLayer): Promise<BackupMetadata[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_backups', { layer: layer ?? null }) as Promise<BackupMetadata[]>
}

/**
 * 지정 백업 파일로 현재 DB 를 복원한다 (T8).
 *
 * `integrity::restore_from_path` 안전망 공유 — 후보 백업이 무결한지 quick_check 통과 확인 후
 * 현재 DB 를 `restore_rollback/` 에 보존한 뒤 복사. 복사 실패 시 자동으로 rollback 되돌림.
 *
 * 브라우저 개발 모드에서는 더미 결과를 반환하여 UI 흐름만 검증 가능.
 */
export async function restoreBackup(path: string): Promise<RestoreResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      restored_from: `[개발 모드] ${path}`,
      rollback_path: '[개발 모드] ./SmartHB-data/restore_rollback/rollback_dev.db',
    }
  }
  return inv('restore_backup', { path }) as Promise<RestoreResult>
}

/**
 * 현재 DB 의 무결성을 검증한다 (T8 PRD §5.3/§5.4).
 *
 * - `'quick'`: PRAGMA quick_check (~50ms, 앱 시작 시 사용)
 * - `'full'`: PRAGMA integrity_check (일일 백업 시점 또는 사용자 수동 실행)
 *
 * 결과는 discriminated union `{ kind: 'ok' }` 또는 `{ kind: 'failed', detail }`.
 * 브라우저 개발 모드에서는 항상 `{ kind: 'ok' }` 반환.
 */
export async function checkIntegrity(mode: IntegrityMode): Promise<IntegrityCheckResult> {
  const inv = await getInvoke()
  if (!inv) return { kind: 'ok' }
  return inv('check_integrity', { mode }) as Promise<IntegrityCheckResult>
}

/**
 * `backup/exit/` 의 가장 최신 무결한 백업으로 자동 복원한다 (T8).
 *
 * 백엔드가 시간 역순으로 후보를 검증하며 quick_check 통과한 첫 백업을 선택. 모든 후보가
 * 손상되었으면 사용자에게 명확한 에러로 throw — UI 가 사용자에게 daily/weekly 수동 선택 안내.
 */
export async function autoRestore(): Promise<RestoreResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      restored_from: '[개발 모드] ./SmartHB-data/backup/exit/app_dev.db',
      rollback_path: '[개발 모드] ./SmartHB-data/restore_rollback/rollback_dev.db',
    }
  }
  return inv('auto_restore') as Promise<RestoreResult>
}

/**
 * 클라우드 동기화 상태를 조회한다 (T9 PRD §5.3).
 *
 * `'waiting'` 응답 시 UI 가 일정 간격으로 본 함수를 재호출 — 30초 대기 후에도 `'waiting'`
 * 이면 "새로고침" 옵션 노출. 브라우저 개발 모드에서는 항상 `'ready'` 반환.
 */
export async function checkSyncStatus(): Promise<SyncStatus> {
  const inv = await getInvoke()
  if (!inv) return { kind: 'ready' }
  return inv('check_sync_status') as Promise<SyncStatus>
}

/**
 * 감사 로그를 시간 역순으로 조회한다 (T9 PRD §6.6).
 *
 * @param since ISO8601 UTC 시각 (선택). 본 시각 이후 항목만 조회.
 * @param limit 페이지당 최대 항목 수. 기본 100, 최대 1000.
 *
 * 백엔드 DB pool 미초기화 상태(unlock 미수행)에서 호출 시 사용자 친화 메시지로 throw.
 * 브라우저 개발 모드에서는 빈 배열 반환.
 */
export async function getAuditLogs(
  since?: string,
  limit?: number,
): Promise<AuditLogEntry[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_audit_logs', {
    since: since ?? null,
    limit: limit ?? null,
  }) as Promise<AuditLogEntry[]>
}
