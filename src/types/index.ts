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
 * 보관 정책: exit(5) / hourly(12) / daily(14) / weekly(4) — 초과 시 가장 오래된 파일 삭제 (PRD §5.4 v1.5.2).
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

/**
 * 복원 리허설로 검증한 테이블 행 수 — `backup.rs::TableCount` 와 정합.
 */
export interface TableCount {
  table: string
  count: number
}

/**
 * 복원 리허설 결과 — `src-tauri/src/commands/backup.rs::RehearsalResult` 와 정합 (PRD §5.4).
 *
 * 백업 파일을 격리된 임시 사본으로 복사해 `PRAGMA integrity_check` + 주요 테이블 행 수를
 * 검증한 결과다. **운영 DB 에는 영향이 없다.**
 *
 * - `success`: 무결성 통과 + 주요 테이블 열람 성공 시 true.
 * - `integrity_detail`: 실패 시 사유(손상 메시지 또는 열기/복호화 실패). 성공 시 null.
 * - `table_counts`: 검증된 주요 테이블 행 수. 성공 시에만 채워진다.
 */
export interface RehearsalResult {
  backup_path: string
  size_bytes: number
  success: boolean
  integrity_detail: string | null
  table_counts: TableCount[]
  total_rows: number
}

/**
 * 감사 로그 이벤트 종류 — T9 PRD §6.6.
 *
 * `src-tauri/src/commands/audit.rs::AuditEventType` 와 kebab-case 직렬화 정합.
 */
export type AuditEventType =
  | 'password-change'
  | 'backup-created'
  | 'backup-restored'
  | 'lock-forced'
  | 'integrity-check-failed'

/**
 * 감사 로그 항목 — `src-tauri/src/commands/audit.rs::AuditLogEntry` 와 정합.
 *
 * `created_at`: ISO8601 UTC. `event_type`: AuditEventType 문자열 또는 향후 추가될 신규 코드.
 * `details`: JSON 문자열 (호출자가 사전 마스킹 — 민감 데이터 미포함).
 */
export interface AuditLogEntry {
  id: number
  created_at: string
  event_type: string
  event_subject: string | null
  details: string | null
}

/**
 * 앱 시작 시퀀스 결과 — T10 PRD §5.6 < 3초 예산 검증 + Sprint 2 T4 timing breakdown.
 *
 * `src-tauri/src/startup.rs::StartupResult` 와 정합. `elapsed_ms` 가 3000 미만이면
 * 시작 예산 통과 — UI 가 임계 초과 시 사용자에게 환경 점검 안내.
 *
 * 각 `*_ms` 필드는 R8 cipher on 실측 디버깅용 timing breakdown — 3초 초과 시 어느 단계가
 * 병목인지 식별 가능. `password_verify_ms` 가 보통 가장 큰 비중 (PBKDF2 600K iter, ~500ms).
 *
 * `integrity_ok=false` 는 cipher off 개발 빌드 또는 stub 케이스 — startup 자체는 성공.
 * `audit_cleaned` 는 1년 이전 audit_logs 삭제 행 수 (정보성).
 */
export interface StartupResult {
  elapsed_ms: number
  parallel_phase_ms: number
  password_verify_ms: number
  db_init_ms: number
  audit_cleanup_ms: number
  lock_force_used: boolean
  integrity_ok: boolean
  audit_cleaned: number
  /** Sprint 10 T4 (PI-05/PI-09): 앱 시작 직후 소멸 자동 전이 결과.
   *  내부는 camelCase, 본 필드는 snake_case 유지 (기존 StartupResult 패턴).
   *  `transitioned_count > 0` 시 메인 페이지에서 토스트 표시 (T9). */
  expiration_report: import('./expiration').ExpirationReport
  /** Sprint 16: 시작 시 DB 손상이 감지되어 자동 복원된 경우의 결과 (없으면 null).
   *  null 이 아니면 메인 페이지에서 "최근 정상 백업으로 복원됨" 고지. */
  auto_restored: RestoreResult | null
}
