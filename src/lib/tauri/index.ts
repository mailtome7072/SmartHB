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
  StartupResult,
  SyncStatus,
} from '@/types'
import type {
  CodeEntry,
  CodeTable,
  CodeUpdate,
  NewCode,
} from '@/types/code'
import type {
  FeeUpdate,
  NewFee,
  StandardFee,
} from '@/types/fee'
import type {
  AttendanceGrid,
  AttendanceSummary,
  GenerateResult,
  ToggleResult,
} from '@/types/attendance'
import type {
  DiagnosisHistoryRow,
  DiagnosisResult,
} from '@/types/diagnosis'
import type { ExportResult } from '@/types/export'
import type {
  AcademyOverview,
  AttendanceProgress,
  BillingTrendPoint,
  DashboardAlert,
  MonthlySummary,
  TodaySchedule,
} from '@/types/dashboard'
import type {
  AbsenceHistoryItem,
  CreateMakeupPayload,
  EligibleDate,
  MakeupResult,
  PendingAbsence,
} from '@/types/makeup'
import type {
  CascadeDeletePreview,
  CreateScheduleCodePayload,
  CreateScheduleEventPayload,
  CreateStudyPeriodPayload,
  ScheduleCode,
  ScheduleEvent,
  ScheduleEventListItem,
  StudyPeriod,
  StudyPeriodResult,
  UpdateScheduleCodePayload,
  UpdateScheduleEventPayload,
  UpdateStudyPeriodPayload,
} from '@/types/academic'
import type { ExpirationReport } from '@/types/expiration'
import type {
  WithdrawalChoice,
  WithdrawalPendingMakeup,
} from '@/types/withdrawal'
import type {
  CalendarMonth,
  MakeupManagementStudent,
} from '@/types/calendar'
import type {
  ScheduleSet,
  StudentSchedule,
} from '@/types/schedule'
import type {
  NewStudent,
  Student,
  StudentFilter,
  StudentUpdate,
} from '@/types/student'

let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null

/**
 * OS 폴더 선택 다이얼로그를 띄우고 사용자가 선택한 경로를 반환한다 (Sprint 3 T7).
 *
 * 마법사(`/setup`)에서 클라우드 동기화 폴더(MYBOX/iCloud Drive/Dropbox) 선택용.
 * 사용자가 취소하면 `null` 반환. 개발 모드(Tauri 미동작)에서는 더미 경로 반환.
 *
 * 권한: `capabilities/default.json` 의 `dialog:allow-open` 필요.
 */
export async function selectFolder(): Promise<string | null> {
  if (typeof window === 'undefined') return null
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (selected === null) return null
    return typeof selected === 'string' ? selected : selected[0] ?? null
  } catch {
    return '[개발 모드] /Users/dev/MYBOX'
  }
}

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
 * 앱 종료 — AppHandle::exit(0) → RunEvent::ExitRequested → release_lock + exit 백업.
 *
 * 사이드바 "종료" 메뉴가 호출. capabilities 권한 우회 + macOS 닥 메뉴바 잔존 회피를 위해 백엔드 IPC 경유.
 */
export async function quitApp(): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('quit_app')
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
 * 현재 PIN 을 확인한 뒤 새 PIN 으로 변경한다 (잠금 해제 상태에서 설정 메뉴를 통해 호출).
 *
 * 현 PIN 불일치 / 새 PIN 형식 오류 시 throw — 호출자가 catch 하여 사용자 메시지 표시.
 */
export async function changePin(currentPin: string, newPin: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('change_pin', { currentPin, newPin })
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

/**
 * 앱 시작 시퀀스를 실행한다 (T10 PRD §5.6).
 *
 * 흐름: 락 + 무결성 quick_check 병렬 → 비밀번호 검증 → DB pool 초기화 → audit 1년 정리 →
 * 백그라운드 task spawn. UI 가 `checkSyncStatus` 로 동기화 대기를 먼저 처리한 후 본 함수를 호출한다.
 *
 * `forceLock=true` 는 사용자가 이전 화면에서 stale 락 강제 점유에 동의한 후에만 호출.
 * 브라우저 개발 모드에서는 더미 결과를 반환하여 UI 흐름만 검증 가능.
 *
 * 반환값 `elapsed_ms` 가 3000 미만이면 시작 예산 통과 — 임계 초과 시 UI 가 사용자에게
 * 환경 점검(클라우드 동기화 상태, 디스크 부하 등) 안내.
 */
export async function appStartupSequence(
  password: string,
  forceLock = false,
): Promise<StartupResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      elapsed_ms: 0,
      parallel_phase_ms: 0,
      password_verify_ms: 0,
      db_init_ms: 0,
      audit_cleanup_ms: 0,
      lock_force_used: forceLock,
      integrity_ok: true,
      audit_cleaned: 0,
      expiration_report: { transitionedCount: 0, details: [] },
    }
  }
  return inv('app_startup_sequence', {
    password,
    forceLock,
  }) as Promise<StartupResult>
}

/** 실행 시 PIN 인증 스킵 설정 조회 (ADR-008). 기본 false(인증 ON). 개발 모드는 false. */
export async function getPinSkipSetting(): Promise<boolean> {
  const inv = await getInvoke()
  if (!inv) return false
  return inv('get_pin_skip_setting') as Promise<boolean>
}

/** 실행 시 PIN 인증 스킵 설정 저장 (ADR-008). PC별 로컬(config.json). */
export async function setPinSkipSetting(skip: boolean): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('set_pin_skip_setting', { skip })
}

/**
 * 키체인 자동 잠금해제 (ADR-008). PIN 입력 없이 키체인 키로 진입.
 * 키체인에 키가 없으면 reject → 호출자가 LockScreen 폴백. 개발 모드는 미지원(reject).
 */
export async function autoUnlockWithKeychain(forceLock = false): Promise<StartupResult> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] 자동 잠금해제 미지원')
  return inv('auto_unlock_with_keychain', { forceLock }) as Promise<StartupResult>
}

// ----------------------------------------------------------------------------
// Sprint 2 — 원생 도메인
// ----------------------------------------------------------------------------

/** PI-05 자동 채번 후보 — UI 등록 폼 기본값 표시용. */
export async function nextSerialNumber(): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return '1'
  return inv('next_serial_number') as Promise<string>
}

export async function createStudent(payload: NewStudent): Promise<Student> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id: 0,
      serial_no: payload.serial_no ?? '1',
      name: payload.name,
      gender: payload.gender,
      school_level: payload.school_level,
      grade: payload.grade,
      school_id: payload.school_id ?? null,
      phone_student: payload.phone_student ?? null,
      phone_mother: payload.phone_mother ?? null,
      phone_father: payload.phone_father ?? null,
      enroll_date: payload.enroll_date,
      withdraw_date: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('create_student', { payload }) as Promise<Student>
}

export async function updateStudent(id: number, payload: StudentUpdate): Promise<Student> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id,
      ...payload,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('update_student', { id, payload }) as Promise<Student>
}

export async function getStudent(id: number): Promise<Student> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] 원생 조회는 Tauri 환경에서만 동작합니다.')
  return inv('get_student', { id }) as Promise<Student>
}

export async function withdrawStudent(id: number, withdrawDate: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('withdraw_student', { id, withdrawDate })
}

/**
 * 퇴교 처리를 번복한다 (Sprint 4 T8 / 사용자 이슈 #8) — withdraw_date NULL 처리.
 *
 * 보강 잔여 처리 등 부수 효과는 본 IPC 범위 외 (Phase 3).
 */
export async function reinstateStudent(id: number): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('reinstate_student', { id })
}

/**
 * Sprint 10 T6 — 퇴교 시 미보강 결석 조회 (PRD §4.5.9).
 *
 * 빈 리스트(`absences.length === 0 && remainingMinutes === 0`) → 보강 잔여 없음 → 일반 `withdrawStudent` 흐름 사용.
 */
export async function getPendingMakeupForWithdrawal(
  studentId: number,
): Promise<WithdrawalPendingMakeup> {
  const inv = await getInvoke()
  if (!inv) {
    return { studentId, remainingMinutes: 0, absences: [] }
  }
  return inv('get_pending_makeup_for_withdrawal', {
    studentId,
  }) as Promise<WithdrawalPendingMakeup>
}

/**
 * Sprint 10 T6 — 퇴교 처리 (PRD §4.5.9, 3가지 선택지).
 *
 * 단일 트랜잭션: 보강 일괄 전이 + (external_expire 시) memo 일괄 저장 + withdraw_date 설정.
 * `defer_withdrawal` 은 IPC 호출 없이 UI 다이얼로그 닫기로 처리.
 */
export async function processWithdrawalMakeup(
  studentId: number,
  choice: WithdrawalChoice,
  withdrawDate: string,
): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('process_withdrawal_makeup', {
    studentId,
    choice,
    withdrawDate,
  })
}

/**
 * 원생 목록을 다중 필터·정렬·페이지네이션으로 조회한다.
 *
 * R14: `filter.limit` 미지정 시 백엔드 기본 100 (상한 1000), `filter.offset` 기본 0.
 * 페이지 UI 는 `countStudents(filter)` 로 총 건수를 별도 조회.
 * 개발 모드(Tauri 미동작)에서는 빈 배열을 반환한다.
 */
export async function listStudents(filter: StudentFilter = {}): Promise<Student[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_students', { filter }) as Promise<Student[]>
}

/**
 * 동일 필터에 매칭되는 총 원생 수를 반환한다 (R14 페이지네이션 UI 보조).
 *
 * `filter.limit` / `filter.offset` 은 백엔드에서 무시된다 — 필터 조합 자체의 총 건수.
 * 개발 모드에서는 0 을 반환.
 */
export async function countStudents(filter: StudentFilter = {}): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  return inv('count_students', { filter }) as Promise<number>
}

// ----------------------------------------------------------------------------
// Sprint 2 — 수업 스케줄
// ----------------------------------------------------------------------------

export async function setSchedule(payload: ScheduleSet): Promise<StudentSchedule> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id: 0,
      student_id: payload.student_id,
      day_of_week: payload.day_of_week,
      start_time: payload.start_time,
      duration_hours: payload.duration_hours,
      effective_from: payload.effective_from,
      effective_to: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('set_schedule', { payload }) as Promise<StudentSchedule>
}

/**
 * 원생의 특정 요일 스케줄을 마감한다 (Sprint 4 T9 / 사용자 이슈 #10).
 *
 * effective_to=today 로 설정 — 다음날부터 해당 요일에 수업 없음.
 */
export async function deleteSchedule(
  studentId: number,
  dayOfWeek: number,
  today: string,
): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('delete_schedule', { studentId, dayOfWeek, today })
}

export async function getSchedules(studentId: number): Promise<StudentSchedule[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_schedules', { studentId }) as Promise<StudentSchedule[]>
}

export async function getScheduleHistory(studentId: number): Promise<StudentSchedule[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_schedule_history', { studentId }) as Promise<StudentSchedule[]>
}

export async function getWeeklyHours(studentId: number): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  return inv('get_weekly_hours', { studentId }) as Promise<number>
}

// ----------------------------------------------------------------------------
// Sprint 2 — 표준 교습비
// ----------------------------------------------------------------------------

export async function listFees(): Promise<StandardFee[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_fees') as Promise<StandardFee[]>
}

export async function createFee(payload: NewFee): Promise<StandardFee> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id: 0,
      weekly_hours: payload.weekly_hours,
      amount: payload.amount,
      sort_order: payload.sort_order ?? 0,
      is_active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('create_fee', { payload }) as Promise<StandardFee>
}

export async function updateFee(id: number, payload: FeeUpdate): Promise<StandardFee> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id,
      ...payload,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('update_fee', { id, payload }) as Promise<StandardFee>
}

/** 주 수업시간 → 매칭 교습비 (정확 일치 우선, 없으면 이하 최댓값). */
export async function matchFeeByHours(weeklyHours: number): Promise<StandardFee | null> {
  const inv = await getInvoke()
  if (!inv) return null
  return inv('match_fee_by_hours', { weeklyHours }) as Promise<StandardFee | null>
}

// ----------------------------------------------------------------------------
// Sprint 2 — 코드 테이블
// ----------------------------------------------------------------------------

/**
 * 코드 항목 목록을 페이지네이션으로 조회한다 (R14).
 *
 * `limit` 미지정 시 백엔드 기본 100 (상한 1000), `offset` 기본 0.
 * 개발 모드에서는 빈 배열을 반환한다.
 */
export async function listCodes(
  table: CodeTable,
  limit?: number,
  offset?: number,
): Promise<CodeEntry[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_codes', {
    table,
    limit: limit ?? null,
    offset: offset ?? null,
  }) as Promise<CodeEntry[]>
}

/**
 * 코드 테이블 총 항목 수 (R14 페이지네이션 UI 보조).
 */
export async function countCodes(table: CodeTable): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  return inv('count_codes', { table }) as Promise<number>
}

export async function createCode(table: CodeTable, payload: NewCode): Promise<CodeEntry> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id: 0,
      code: payload.code,
      label: payload.label ?? payload.code,
      sort_order: payload.sort_order ?? 0,
      is_active: true,
    }
  }
  return inv('create_code', { table, payload }) as Promise<CodeEntry>
}

export async function updateCode(
  table: CodeTable,
  id: number,
  payload: CodeUpdate,
): Promise<CodeEntry> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id,
      code: payload.label,
      label: payload.label,
      sort_order: payload.sort_order,
      is_active: payload.is_active,
    }
  }
  return inv('update_code', { table, id, payload }) as Promise<CodeEntry>
}

/** 정렬 순서 일괄 변경 — (id, sort_order) 쌍 배열. */
export async function reorderCodes(
  table: CodeTable,
  orders: [number, number][],
): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('reorder_codes', { table, orders })
}

// ----------------------------------------------------------------------------
// Sprint 3 — 초기 설정 마법사 (T8/T9, PRD §4.0)
// ----------------------------------------------------------------------------

export interface SetupStatus {
  cloud_folder_path: string
  setup_completed: boolean
}

/** 마법사 진행 상태 — 미진입 시 빈 경로 + setup_completed=false. */
export async function getSetupStatus(): Promise<SetupStatus> {
  const inv = await getInvoke()
  if (!inv) return { cloud_folder_path: '', setup_completed: false }
  return inv('get_setup_status') as Promise<SetupStatus>
}

/** 클라우드 동기화 폴더 경로를 저장 — 폴더 안에 smarthb/ 디렉토리 생성. */
export async function saveCloudFolder(path: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('save_cloud_folder', { path })
}

/** 마법사 완료 표시 — 모든 단계 완료 후 호출. */
export async function completeSetup(): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('complete_setup')
}

// ============================================================================
// 영구 설정 (Sprint 4 T2, PRD §4.0/§4.12) — 교습소 운영 시간
// ============================================================================

/**
 * 요일별 운영 시간. open/close 가 모두 null 이면 미운영.
 *
 * `src-tauri/src/commands/settings.rs::DayHours` 와 정합.
 * day_of_week: 1=월, 2=화, 3=수, 4=목, 5=금, 6=토, 7=일 (ISO 8601, schedules.rs 와 일관)
 */
export interface DayHours {
  day_of_week: number
  open_time: string | null
  close_time: string | null
}

/** 운영 시간 조회 — 저장값 없으면 디폴트 (월~금 13:00~19:00, 토/일 미운영). */
export async function getOperatingHours(): Promise<DayHours[]> {
  const inv = await getInvoke()
  if (!inv) {
    // dev fallback — 디폴트 7일 반환
    return Array.from({ length: 7 }, (_, i) => {
      const day = i + 1
      return day <= 5
        ? { day_of_week: day, open_time: '13:00', close_time: '19:00' }
        : { day_of_week: day, open_time: null, close_time: null }
    })
  }
  return inv('get_operating_hours') as Promise<DayHours[]>
}

/** 운영 시간 저장 — 7개 요일 모두 포함 필수, 백엔드가 검증. */
export async function saveOperatingHours(hours: DayHours[]): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('save_operating_hours', { hours })
}

// ============================================================================
// Sprint 6 — 일정 관리 도메인 (T8, PRD §4.4)
// ============================================================================
// 백엔드: src-tauri/src/commands/academic.rs (T5/T6/T7).
// Tauri invoke args 는 자동 camelCase ↔ snake_case 변환 (예: Rust from_month ↔ TS fromMonth).

// ─── 교습기간 study_periods (T5) ─────────────────────────────────────

/** Sprint 10 T4: 응답이 `StudyPeriodResult` 로 wrapping — 소멸 자동 전이 결과 동봉. */
export async function createStudyPeriod(
  payload: CreateStudyPeriodPayload,
): Promise<StudyPeriodResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      studyPeriod: {
        id: 0,
        year_month: payload.year_month,
        start_date: payload.start_date,
        end_date: payload.end_date,
        is_confirmed: false,
        is_closed: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      expirationReport: { transitionedCount: 0, details: [] },
    }
  }
  return inv('create_study_period', { payload }) as Promise<StudyPeriodResult>
}

export async function updateStudyPeriod(
  id: number,
  payload: UpdateStudyPeriodPayload,
): Promise<StudyPeriodResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      studyPeriod: {
        id,
        year_month: payload.start_date.slice(0, 7),
        start_date: payload.start_date,
        end_date: payload.end_date,
        is_confirmed: false,
        is_closed: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      expirationReport: { transitionedCount: 0, details: [] },
    }
  }
  return inv('update_study_period', { id, payload }) as Promise<StudyPeriodResult>
}

/** 교습기간 목록 — "YYYY-MM" 범위(포함). */
export async function listStudyPeriods(
  fromMonth: string,
  toMonth: string,
): Promise<StudyPeriod[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_study_periods', { fromMonth, toMonth }) as Promise<StudyPeriod[]>
}

/** 특정 월의 교습기간 조회 — 없으면 null. */
export async function getStudyPeriod(yearMonth: string): Promise<StudyPeriod | null> {
  const inv = await getInvoke()
  if (!inv) return null
  return inv('get_study_period', { yearMonth }) as Promise<StudyPeriod | null>
}

export async function confirmStudyPeriod(id: number): Promise<StudyPeriodResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      studyPeriod: {
        id,
        year_month: '',
        start_date: '',
        end_date: '',
        is_confirmed: true,
        is_closed: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      expirationReport: { transitionedCount: 0, details: [] },
    }
  }
  return inv('confirm_study_period', { id }) as Promise<StudyPeriodResult>
}

/** 미확정 교습기간 삭제 — 확정/마감 시 백엔드가 throw. */
export async function deleteStudyPeriod(id: number): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('delete_study_period', { id })
}

/** 교습기간 cascade 삭제 미리보기 (Sprint 7 T8) — 영향 건수 + 가능 여부. */
export async function getCascadeDeletePreview(
  id: number,
): Promise<CascadeDeletePreview> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      affected_count: 0,
      holiday_count: 0,
      deletable: false,
      reason: '개발 모드: 백엔드 없이는 cascade 삭제 미리보기 불가',
    }
  }
  return inv('get_cascade_delete_preview', { id }) as Promise<CascadeDeletePreview>
}

/** 교습기간 cascade 삭제 — 공휴일 제외 학사 일정 + 교습기간 트랜잭션 삭제 (Sprint 7 T8). */
export async function deleteStudyPeriodCascade(id: number): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('delete_study_period_cascade', { id })
}

// ─── 학사 일정 코드 schedule_codes (T6) ──────────────────────────────

export async function listScheduleCodes(): Promise<ScheduleCode[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_schedule_codes') as Promise<ScheduleCode[]>
}

export async function createScheduleCode(
  payload: CreateScheduleCodePayload,
): Promise<ScheduleCode> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id: 0,
      code_name: payload.code_name,
      is_system_reserved: false,
      allows_regular_class: payload.allows_regular_class,
      allows_makeup_class: payload.allows_makeup_class,
      is_duplicate_blocked: payload.is_duplicate_blocked,
      is_period_type: payload.is_period_type,
      is_active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('create_schedule_code', { payload }) as Promise<ScheduleCode>
}

export async function updateScheduleCode(
  id: number,
  payload: UpdateScheduleCodePayload,
): Promise<ScheduleCode> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id,
      code_name: '',
      is_system_reserved: false,
      allows_regular_class: payload.allows_regular_class,
      allows_makeup_class: payload.allows_makeup_class,
      is_duplicate_blocked: payload.is_duplicate_blocked,
      is_period_type: payload.is_period_type,
      is_active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('update_schedule_code', { id, payload }) as Promise<ScheduleCode>
}

/** 활성/비활성 토글 — 시스템 예약 코드도 허용 (AC-T6-2). */
export async function toggleScheduleCodeActive(id: number): Promise<ScheduleCode> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id,
      code_name: '',
      is_system_reserved: false,
      allows_regular_class: false,
      allows_makeup_class: false,
      is_duplicate_blocked: true,
      is_period_type: false,
      is_active: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('toggle_schedule_code_active', { id }) as Promise<ScheduleCode>
}

// ─── 학사 일정 schedule_events (T7) ──────────────────────────────────

export async function createScheduleEvent(
  payload: CreateScheduleEventPayload,
): Promise<ScheduleEvent> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id: 0,
      code_id: payload.code_id,
      event_date: payload.event_date,
      period_end_date: payload.period_end_date,
      display_name: payload.display_name,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('create_schedule_event', { payload }) as Promise<ScheduleEvent>
}

export async function updateScheduleEvent(
  id: number,
  payload: UpdateScheduleEventPayload,
): Promise<ScheduleEvent> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      id,
      code_id: 0,
      event_date: payload.event_date,
      period_end_date: payload.period_end_date,
      display_name: payload.display_name,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
  }
  return inv('update_schedule_event', { id, payload }) as Promise<ScheduleEvent>
}

/** 학사 일정 삭제 — 지난 달이면 백엔드가 throw (AC-T7-3). */
export async function deleteScheduleEvent(id: number): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('delete_schedule_event', { id })
}

/** 기간 내 학사 일정 목록 (코드명 JOIN 평탄 응답) — "YYYY-MM-DD" 범위. */
export async function listScheduleEvents(
  fromDate: string,
  toDate: string,
): Promise<ScheduleEventListItem[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_schedule_events', { fromDate, toDate }) as Promise<ScheduleEventListItem[]>
}

/**
 * 단원평가 응시일 자동 배치 (§4.4.7).
 *
 * 해당 month 2주차 월~금 + 4주차 월~금 10건 INSERT. 이미 단원평가 1건 이상 존재 시 빈 배열 반환(No-op, AC-4.4-6).
 */
export async function autoPlaceAssessmentDates(yearMonth: string): Promise<ScheduleEvent[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('auto_place_assessment_dates', { yearMonth }) as Promise<ScheduleEvent[]>
}

// ──────────────────── 출결 도메인 (Sprint 8 T2·T3) ────────────────────

/** 해당 월에 정규 출결이 이미 생성되어 있는지 확인. */
export async function checkAttendanceExists(yearMonth: string): Promise<boolean> {
  const inv = await getInvoke()
  if (!inv) return false
  return inv('check_attendance_exists', { yearMonth }) as Promise<boolean>
}

/** 해당 월 재원 원생 × 수업 요일 일자에 정규 출결 일괄 생성 (AC-4.5-1). */
export async function countUngeneratedAttendanceStudents(yearMonth: string): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  return inv('count_ungenerated_attendance_students', { yearMonth }) as Promise<number>
}

export async function generateAttendances(yearMonth: string): Promise<GenerateResult> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      yearMonth,
      studentCount: 0,
      attendanceCount: 0,
      expirationReport: { transitionedCount: 0, details: [] },
    }
  }
  return inv('generate_attendances', { yearMonth }) as Promise<GenerateResult>
}

/**
 * Sprint 10 T4 (PI-05): 소멸 자동 전이 수동 호출.
 *
 * 트리거 3개소(앱 시작/출결 생성/교습기간 등록)에서 응답에 자동 동봉되지만,
 * UI 에서 명시적으로 한 번 더 호출하고 싶을 때 사용 (예: 디버깅, 수동 점검 메뉴).
 */
export async function expireOverdueAbsences(): Promise<ExpirationReport> {
  const inv = await getInvoke()
  if (!inv) return { transitionedCount: 0, details: [] }
  return inv('expire_overdue_absences') as Promise<ExpirationReport>
}

/**
 * Sprint 10 T8/T11 (PRD §4.6.1): 수업 관리 캘린더 — 일자별 정규/보강 수업.
 *
 * 백엔드는 raw 일자별 목록만 제공 — 시간대별 합산(AC-4.6-1)은 캘린더 UI 책임.
 */
export async function getCalendarData(yearMonth: string): Promise<CalendarMonth> {
  const inv = await getInvoke()
  if (!inv) return { yearMonth, days: [] }
  return inv('get_calendar_data', { yearMonth }) as Promise<CalendarMonth>
}

/**
 * Sprint 10 T8/T11 (PRD §4.6.3): 보강 관리 뷰 — 보강 필요 원생(소멸 임박 순).
 *
 * 정렬·임박 판정은 백엔드(`calendar.rs`)에서 수행. UI 는 표시만.
 */
export async function getMakeupManagementData(
  yearMonth: string,
): Promise<MakeupManagementStudent[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_makeup_management_data', { yearMonth }) as Promise<
    MakeupManagementStudent[]
  >
}

/** 출결표 그리드 — 원생 × 일자 + 월간 요약. 50명×31일 < 1초 (PRD §5.7). */
export async function getAttendanceGrid(yearMonth: string): Promise<AttendanceGrid> {
  const inv = await getInvoke()
  if (!inv) return { yearMonth, students: [], daySchedules: [] }
  return inv('get_attendance_grid', { yearMonth }) as Promise<AttendanceGrid>
}

/** 출석↔결석 토글. 보강필요시간/소멸기한 자동 갱신, audit 기록. */
export async function toggleAttendance(
  attendanceId: number,
  newStatus: 'present' | 'absent',
): Promise<ToggleResult> {
  const inv = await getInvoke()
  if (!inv) {
    throw new Error('Tauri 환경에서만 사용 가능')
  }
  return inv('toggle_attendance', { attendanceId, newStatus }) as Promise<ToggleResult>
}

/** 결석 사유 메모 set/clear. memo=null 이면 NULL 로 환원. */
export async function updateAbsenceMemo(
  attendanceId: number,
  memo: string | null,
): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('update_absence_memo', { attendanceId, memo })
}

/** 원생 월간 요약 단일 조회. */
export async function getAttendanceSummary(
  studentId: number,
  yearMonth: string,
): Promise<AttendanceSummary> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      studentId,
      yearMonth,
      presentCount: 0,
      absentCount: 0,
      makeupNeededMinutes: 0,
      makeupCompletedMinutes: 0,
    }
  }
  return inv('get_attendance_summary', { studentId, yearMonth }) as Promise<AttendanceSummary>
}

// ──────────────────── 보강 도메인 (Sprint 9 T2~T4) ────────────────────

/** 원생의 미처리 결석 목록 — 소멸기한 임박순 (NULL 마지막). PRD §4.5.4 다이얼로그 소스. */
export async function getPendingAbsences(studentId: number): Promise<PendingAbsence[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_pending_absences', { studentId }) as Promise<PendingAbsence[]>
}

/** 보강 가능 일자 — year_month 내 `allows_makeup_class=1` 학사일정 + 학생 입퇴교 범위. */
export async function getMakeupEligibleDates(
  studentId: number,
  yearMonth: string,
): Promise<EligibleDate[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_makeup_eligible_dates', { studentId, yearMonth }) as Promise<EligibleDate[]>
}

/** 보강 등록 + 매칭 (트랜잭션 검증 5종). PI-02 일 단위 채택 — class_minutes 비교 없음. */
export async function createMakeupWithAbsences(
  payload: CreateMakeupPayload,
): Promise<MakeupResult> {
  const inv = await getInvoke()
  if (!inv) {
    throw new Error('Tauri 환경에서만 사용 가능')
  }
  return inv('create_makeup_with_absences', { payload }) as Promise<MakeupResult>
}

/** 보강 취소 — 연결 결석을 absent 환원 + makeup_attendances DELETE. */
export async function cancelMakeup(makeupId: number): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('cancel_makeup', { makeupId })
}

/** 원생 결석 이력 — absent/makeup_done/makeup_expired 모두 포함, event_date DESC (T8). */
export async function getAbsenceHistory(
  studentId: number,
): Promise<AbsenceHistoryItem[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_absence_history', { studentId }) as Promise<AbsenceHistoryItem[]>
}

// ─────────────────────── Sprint 11 청구·수납 도메인 ───────────────────────

import type {
  Bill,
  BillingPeriodStats,
  BillingSearchResult,
  BillingSummary,
  GenerateBillsResult,
  Payment,
  PaymentInput,
  PaymentViewRow,
  UnpaidBill,
} from '@/types/billing'

export async function generateBills(yearMonth: string): Promise<GenerateBillsResult> {
  const inv = await getInvoke()
  if (!inv) return { yearMonth, generatedCount: 0, skippedCount: 0 }
  return inv('generate_bills', { yearMonth }) as Promise<GenerateBillsResult>
}

export async function listBills(yearMonth: string): Promise<Bill[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_bills', { yearMonth }) as Promise<Bill[]>
}

export async function getBill(id: number): Promise<Bill> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] getBill 호출 불가')
  return inv('get_bill', { id }) as Promise<Bill>
}

export async function updateBill(id: number, adjustedAmount: number): Promise<Bill> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] updateBill 호출 불가')
  return inv('update_bill', { id, adjustedAmount }) as Promise<Bill>
}

export async function confirmBill(id: number): Promise<Bill> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] confirmBill 호출 불가')
  return inv('confirm_bill', { id }) as Promise<Bill>
}

export async function confirmAllBills(yearMonth: string): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  return inv('confirm_all_bills', { yearMonth }) as Promise<number>
}

export async function createPayment(input: PaymentInput): Promise<Payment> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] createPayment 호출 불가')
  return inv('create_payment', { input }) as Promise<Payment>
}

export async function updatePayment(id: number, input: PaymentInput): Promise<Payment> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] updatePayment 호출 불가')
  return inv('update_payment', { id, input }) as Promise<Payment>
}

export async function listUnpaidBills(yearMonth: string): Promise<UnpaidBill[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_unpaid_bills', { yearMonth }) as Promise<UnpaidBill[]>
}

export async function batchUpdatePayments(items: PaymentInput[]): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  return inv('batch_update_payments', { items }) as Promise<number>
}

export async function getBillingSummary(yearMonth: string): Promise<BillingSummary> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      yearMonth,
      totalBillableStudents: 0,
      billCount: 0,
      totalBilled: 0,
      totalPaid: 0,
      totalUnpaid: 0,
      paidCount: 0,
      unpaidCount: 0,
    }
  }
  return inv('get_billing_summary', { yearMonth }) as Promise<BillingSummary>
}

export async function listPaymentView(yearMonth: string): Promise<PaymentViewRow[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_payment_view', { yearMonth }) as Promise<PaymentViewRow[]>
}

export async function listBilledMonths(): Promise<string[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_billed_months') as Promise<string[]>
}

export async function getBillingPeriodStats(period: string): Promise<BillingPeriodStats> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      period,
      billCount: 0,
      totalBilled: 0,
      paidCount: 0,
      totalPaid: 0,
      totalUnpaid: 0,
      unpaidCount: 0,
      byMethod: [],
    }
  }
  return inv('get_billing_period_stats', { period }) as Promise<BillingPeriodStats>
}

export async function searchStudentsForBilling(
  query: string,
): Promise<BillingSearchResult[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('search_students_for_billing', { query }) as Promise<BillingSearchResult[]>
}

export async function getDefaultBillingYearMonth(): Promise<string | null> {
  const inv = await getInvoke()
  if (!inv) return null
  return inv('get_default_billing_year_month') as Promise<string | null>
}

// ─────────────────────── Sprint 12 공지문(이미지) 도메인 ───────────────────────

import type { NoticeAsset, NoticeImageItem, NoticeLayout, NoticeMonthInfo } from '@/types/notice'

export async function listNoticeAssets(): Promise<NoticeAsset[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_notice_assets') as Promise<NoticeAsset[]>
}

/** 배경서식 바이트 읽기 (미리보기/생성용). number[] 반환. */
export async function readNoticeAsset(filename: string): Promise<number[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('read_notice_asset', { filename }) as Promise<number[]>
}

/** 배경서식 저장. data 는 이미지 바이트 배열(number[]). 저장된 파일명 반환. */
export async function saveNoticeAsset(filename: string, data: number[]): Promise<string> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] saveNoticeAsset 호출 불가')
  return inv('save_notice_asset', { filename, data }) as Promise<string>
}

export async function deleteNoticeAsset(filename: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  return inv('delete_notice_asset', { filename }) as Promise<void>
}

export async function saveNoticeLayout(layout: NoticeLayout): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  return inv('save_notice_layout', { layout }) as Promise<void>
}

export async function getNoticeLayout(): Promise<NoticeLayout> {
  const inv = await getInvoke()
  if (!inv) {
    return { backgroundAsset: null, textboxes: [] }
  }
  return inv('get_notice_layout') as Promise<NoticeLayout>
}

/** 저장된 공지문 템플릿 이름 목록. */
export async function listNoticeLayouts(): Promise<string[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('list_notice_layouts') as Promise<string[]>
}

/** 현재 레이아웃을 이름 붙여 템플릿으로 저장. */
export async function saveNoticeLayoutNamed(name: string, layout: NoticeLayout): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  return inv('save_notice_layout_named', { name, layout }) as Promise<void>
}

/** 이름으로 저장된 템플릿 조회. */
export async function getNoticeLayoutNamed(name: string): Promise<NoticeLayout> {
  const inv = await getInvoke()
  if (!inv) return { backgroundAsset: null, textboxes: [] }
  return inv('get_notice_layout_named', { name }) as Promise<NoticeLayout>
}

/** 이름 템플릿 삭제. */
export async function deleteNoticeLayoutNamed(name: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  return inv('delete_notice_layout_named', { name }) as Promise<void>
}

/** 청구년월의 교습기간·보강데이 표기 텍스트. */
export async function getNoticeMonthInfo(yearMonth: string): Promise<NoticeMonthInfo> {
  const inv = await getInvoke()
  if (!inv) return { teachingPeriodText: null, makeupDayText: null }
  return inv('get_notice_month_info', { yearMonth }) as Promise<NoticeMonthInfo>
}

/**
 * 한글 파일명/경로를 NFC(완성형)로 정규화한다.
 * macOS 저장 다이얼로그·일부 입력은 NFD(자모 분리형)를 돌려줄 수 있어, 파일시스템에 쓰기 전
 * NFC 로 통일한다. (APFS 는 작성 시 전달한 형태 그대로 저장 — NFC 로 전달하면 NFC 로 저장)
 */
const nfc = (s: string): string => s.normalize('NFC')

/** 단건 공지문 PNG 저장 — output/{공지문이름}/{청구년월}/{공지문이름}_{청구년월}_{원생명}.png. 저장 경로 반환. */
export async function saveNoticeImage(
  noticeName: string,
  yearMonth: string,
  studentName: string,
  image: number[],
): Promise<string> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] saveNoticeImage 호출 불가')
  return inv('save_notice_image', {
    noticeName: nfc(noticeName),
    yearMonth,
    studentName: nfc(studentName),
    image,
  }) as Promise<string>
}

/** 다건 공지문 PNG 일괄 저장. 저장 완료 건수 반환. */
export async function saveNoticeImagesBatch(
  noticeName: string,
  yearMonth: string,
  items: NoticeImageItem[],
): Promise<number> {
  const inv = await getInvoke()
  if (!inv) return 0
  const normalized = items.map((it) => ({ ...it, studentName: nfc(it.studentName) }))
  return inv('save_notice_images_batch', {
    noticeName: nfc(noticeName),
    yearMonth,
    items: normalized,
  }) as Promise<number>
}

/** 해당 공지문/청구년월 출력 폴더에 이미 PNG가 있는지 (덮어쓰기 확인용). */
export async function checkNoticeOutputExists(noticeName: string, yearMonth: string): Promise<boolean> {
  const inv = await getInvoke()
  if (!inv) return false
  return inv('check_notice_output_exists', { noticeName: nfc(noticeName), yearMonth }) as Promise<boolean>
}

/** 미리보기 저장 다이얼로그 기본 경로 — output/공지문/{공지문이름}.png (폴더 미리 생성). */
export async function noticePreviewDefaultPath(noticeName: string): Promise<string> {
  const inv = await getInvoke()
  if (!inv) return `output/공지문/${noticeName}.png`
  return inv('notice_preview_default_path', { noticeName: nfc(noticeName) }) as Promise<string>
}

/** 미리보기 PNG를 지정 경로에 저장. 저장된 경로 반환. (경로는 NFC 로 정규화하여 저장) */
export async function saveNoticePreview(path: string, image: number[]): Promise<string> {
  const inv = await getInvoke()
  if (!inv) throw new Error('[개발 모드] saveNoticePreview 호출 불가')
  return inv('save_notice_preview', { path: nfc(path), image }) as Promise<string>
}

/** 생성 출력 폴더(output/{공지문이름}/{YYMM}/)를 생성 후 OS 탐색기로 연다. 이름 비면 output 루트. */
export async function openNoticeOutputDir(noticeName: string, yearMonth: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('open_notice_output_dir', { noticeName: nfc(noticeName), yearMonth })
}

/** 미리보기 저장 폴더(output/공지문/)를 생성 후 OS 탐색기로 연다. */
export async function openNoticePreviewDir(): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('open_notice_preview_dir')
}

/** 파일 저장 다이얼로그 — 사용자가 선택한 경로 반환(취소 시 null). */
export async function showSaveDialog(defaultPath: string): Promise<string | null> {
  if (typeof window === 'undefined') return null
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const selected = await save({ defaultPath, filters: [{ name: 'PNG 이미지', extensions: ['png'] }] })
    return selected ?? null
  } catch {
    return null
  }
}

// ============================================================================
// Sprint 14 — 데이터 자가 진단 (T1/T2, PRD §6.6)
// ============================================================================
// 백엔드: src-tauri/src/commands/diagnosis.rs (검사 7종 + 이력).

/** 자가 진단 실행 (수동/자동). 7종 검사 + 이력 저장 + 12개월 초과 정리. */
export async function runDiagnosis(runType: 'auto' | 'manual'): Promise<DiagnosisResult> {
  const inv = await getInvoke()
  if (!inv) {
    // dev fallback — 이상 0건
    return { run_date: '', run_type: runType, total_checks: 7, issues_found: 0, issues: [] }
  }
  return inv('run_diagnosis', { runType }) as Promise<DiagnosisResult>
}

/** 진단 이력 조회 (최신순 limit 건). */
export async function getDiagnosisHistory(limit: number): Promise<DiagnosisHistoryRow[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_diagnosis_history', { limit }) as Promise<DiagnosisHistoryRow[]>
}

/** 대시보드 알림용 최신 진단 결과 1건 (없으면 null). */
export async function getLatestDiagnosis(): Promise<DiagnosisHistoryRow | null> {
  const inv = await getInvoke()
  if (!inv) return null
  return inv('get_latest_diagnosis') as Promise<DiagnosisHistoryRow | null>
}

/** 당월 자동 진단 필요 여부 (매월 1일 첫 실행 판단, AC-6.6-1). */
export async function checkAutoDiagnosisNeeded(): Promise<boolean> {
  const inv = await getInvoke()
  if (!inv) return false
  return inv('check_auto_diagnosis_needed') as Promise<boolean>
}

// ============================================================================
// Sprint 14 — 데이터 내보내기 (T5/T6, PRD §4.13.2)
// ============================================================================
// 백엔드: src-tauri/src/commands/export.rs (CSV + UTF-8 BOM).

/** CSV 저장 다이얼로그 — 사용자가 선택한 경로 반환(취소 시 null). */
export async function showCsvSaveDialog(defaultPath: string): Promise<string | null> {
  if (typeof window === 'undefined') return null
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const selected = await save({ defaultPath, filters: [{ name: 'CSV 파일', extensions: ['csv'] }] })
    return selected ?? null
  } catch {
    return null
  }
}

/** dev 모드(브라우저) fallback 결과 — 실제 저장 없이 0건. */
const EXPORT_DEV_FALLBACK: ExportResult = { file_path: '', row_count: 0, byte_size: 0 }

/** 원생 명단을 CSV 로 내보낸다. */
export async function exportStudents(filePath: string): Promise<ExportResult> {
  const inv = await getInvoke()
  if (!inv) return EXPORT_DEV_FALLBACK
  return inv('export_students', { filePath }) as Promise<ExportResult>
}

/** 출결 데이터를 CSV 로 내보낸다. `yearMonth` 가 null 이면 전체 기간. */
export async function exportAttendances(
  yearMonth: string | null,
  filePath: string,
): Promise<ExportResult> {
  const inv = await getInvoke()
  if (!inv) return EXPORT_DEV_FALLBACK
  return inv('export_attendances', { yearMonth, filePath }) as Promise<ExportResult>
}

/** 청구-수납 데이터를 CSV 로 내보낸다. `yearMonth` 가 null 이면 전체 기간. */
export async function exportBilling(
  yearMonth: string | null,
  filePath: string,
): Promise<ExportResult> {
  const inv = await getInvoke()
  if (!inv) return EXPORT_DEV_FALLBACK
  return inv('export_billing', { yearMonth, filePath }) as Promise<ExportResult>
}

// ============================================================================
// Sprint 14 — 대시보드 (T3/T4, PRD §4.11)
// ============================================================================
// 백엔드: src-tauri/src/commands/dashboard.rs.

/** 4.11.1 교습소 현황 (재원/성별/학년/학교 분포 + 분기별 입퇴교). */
export async function getAcademyOverview(): Promise<AcademyOverview> {
  const inv = await getInvoke()
  if (!inv) {
    return { total_active: 0, by_gender: [], by_grade: [], by_school: [], quarterly: [] }
  }
  return inv('get_academy_overview') as Promise<AcademyOverview>
}

/** 4.11.2 당일 수업 — 시간대별 명단. */
export async function getTodaySchedule(): Promise<TodaySchedule> {
  const inv = await getInvoke()
  if (!inv) return { weekday: 1, slots: [] }
  return inv('get_today_schedule') as Promise<TodaySchedule>
}

/** 4.11.3 월 핵심 요약 (청구/입금/미납 + 당월 입퇴교 + 출결 기록일수). */
export async function getMonthlySummary(yearMonth: string): Promise<MonthlySummary> {
  const inv = await getInvoke()
  if (!inv) {
    return {
      year_month: yearMonth,
      bill_total: 0,
      paid_total: 0,
      unpaid_total: 0,
      bill_count: 0,
      paid_count: 0,
      enrolled_this_month: 0,
      withdrawn_this_month: 0,
      attendance_recorded_days: 0,
    }
  }
  return inv('get_monthly_summary', { yearMonth }) as Promise<MonthlySummary>
}

/** 4.11.5 출결 입력 진행률 — 미입력 일자 목록. */
export async function getAttendanceProgress(yearMonth: string): Promise<AttendanceProgress> {
  const inv = await getInvoke()
  if (!inv) return { year_month: yearMonth, expected_days: 0, recorded_days: 0, missing_dates: [] }
  return inv('get_attendance_progress', { yearMonth }) as Promise<AttendanceProgress>
}

/** 4.11.4 알림 5종 (조건 충족분만 반환). */
export async function getDashboardAlerts(): Promise<DashboardAlert[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_dashboard_alerts') as Promise<DashboardAlert[]>
}

/** 교습소 월별 청구총액 추이 (마지막 청구월 기준 최근 12개월, 빈 달 0). */
export async function getBillingTrend(): Promise<BillingTrendPoint[]> {
  const inv = await getInvoke()
  if (!inv) return []
  return inv('get_billing_trend') as Promise<BillingTrendPoint[]>
}

/** 4.11.6 메모 조회 (없으면 null). */
export async function getDashboardMemo(): Promise<string | null> {
  const inv = await getInvoke()
  if (!inv) return null
  return inv('get_dashboard_memo') as Promise<string | null>
}

/** 4.11.6 메모 저장 (디바운스 자동 저장). */
export async function saveDashboardMemo(content: string): Promise<void> {
  const inv = await getInvoke()
  if (!inv) return
  await inv('save_dashboard_memo', { content })
}
