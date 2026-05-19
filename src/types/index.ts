/**
 * 공유 TypeScript 타입 정의
 * Tauri 커맨드 요청/응답 타입은 여기에 정의
 */

export interface ApiResult<T> {
  data: T
  error?: string
}

/**
 * 사용자 인증 상태.
 *
 * `src-tauri/src/commands/auth.rs::AuthStatus` 와 serde rename_all="kebab-case" 정합.
 * Rust enum 의 `NotInitialized` / `Locked` 가 각각 `"not-initialized"` / `"locked"` 로
 * 직렬화되어 IPC 응답으로 전달된다.
 *
 * `Unlocked` 는 메모리 상태로만 관리되어 IPC 응답에 포함되지 않으므로 본 타입에 없다.
 */
export type AuthStatus = 'not-initialized' | 'locked'

/**
 * app.lock 점유 상태 — T6 PRD §5.3.
 *
 * `src-tauri/src/commands/lock.rs::LockStatus` 와 serde `tag = "kind"` + `rename_all = "kebab-case"` 정합.
 * - `free`: 락 파일 없음, 즉시 점유 가능
 * - `owned-by-self`: 본 디바이스가 점유 중
 * - `owned-by-other`: 다른 디바이스 점유 중 (`stale: true` 면 5분 이상 미갱신 — 강제 점유 가능)
 */
export type LockStatus =
  | { kind: 'free' }
  | { kind: 'owned-by-self'; last_heartbeat_seconds_ago: number }
  | { kind: 'owned-by-other'; stale: boolean; last_heartbeat_seconds_ago: number }

/**
 * 백업 계층 — T7 PRD §5.3/§5.4 (ADR-003).
 *
 * `src-tauri/src/commands/backup.rs::BackupLayer` 와 serde rename_all="kebab-case" 정합.
 * 보관 정책: exit(10) / hourly(24) / daily(30) / weekly(4) — 초과 시 가장 오래된 파일 삭제.
 */
export type BackupLayer = 'exit' | 'hourly' | 'daily' | 'weekly'

/**
 * 백업 파일 메타데이터 — IPC 응답.
 *
 * `created_at` 은 ISO8601 UTC 문자열 (chrono::DateTime<Utc> serde 직렬화). UI 표시 시
 * 사용자 로컬 타임존으로 변환 필요.
 */
export interface BackupMetadata {
  path: string
  layer: BackupLayer
  created_at: string
  size_bytes: number
}

/**
 * 무결성 검증 모드 — T8 PRD §5.3/§5.4.
 *
 * - `quick`: PRAGMA quick_check, ~50ms — 앱 시작 시 사용 (PRD §5.6 < 3초 예산)
 * - `full`: PRAGMA integrity_check — 일일 백업 시점 또는 사용자 수동 실행
 */
export type IntegrityMode = 'quick' | 'full'

/**
 * 무결성 검증 결과 — `src-tauri/src/commands/integrity.rs::IntegrityCheckResult` 와 정합.
 *
 * - `ok`: quick_check / integrity_check 가 "ok" 단일 행 반환
 * - `failed`: 손상 — 다중 행 메시지가 `\n` 으로 결합된 `detail` 포함
 */
export type IntegrityCheckResult =
  | { kind: 'ok' }
  | { kind: 'failed'; detail: string }

/**
 * 자동 복원 / 백업 복원 결과 — `src-tauri/src/commands/integrity.rs::RestoreResult` 와 정합.
 *
 * `rollback_path`: 복원 직전 현재 DB 가 보존된 위치. 사용자가 복원이 실패했다고 판단할 경우
 * 수동으로 되돌릴 수 있도록 안내한다.
 */
export interface RestoreResult {
  restored_from: string
  rollback_path: string
}
