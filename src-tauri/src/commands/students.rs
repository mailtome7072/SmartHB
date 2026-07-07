//! 원생 CRUD IPC + PI-05 자동 채번 (Sprint 2 T9, PRD §4.1·§6.2).
//!
//! ## 인터페이스 (6 IPC)
//!
//! - [`next_serial_number`] — UI 등록 폼 기본값 표시 (자동 채번 후보)
//! - [`create_student`] — 신규 원생 등록. `serial_no=None` 이면 자동 채번, `Some(s)` 이면 사용자 override
//! - [`get_student`] — ID 기반 단건 조회
//! - [`update_student`] — 전체 필드 PUT-like 업데이트
//! - [`withdraw_student`] — `withdraw_date` 설정 (퇴교)
//! - [`list_students`] — 다중 필터 + 정렬
//!
//! ## PI-05 자동 채번
//!
//! `serial_no` 는 TEXT 타입 (data-model §1.1) 이지만 자동 채번 시에는 숫자 문자열만 생성.
//! `SELECT COALESCE(MAX(CAST(serial_no AS INTEGER)), 0) + 1 FROM students WHERE serial_no GLOB '[0-9]*'`
//! 패턴으로 숫자 행만 대상. 사용자 override 시 영문 prefix 포함 임의 TEXT 허용.
//!
//! ## 트랜잭션
//!
//! `create_student` / `update_student` 는 `BEGIN IMMEDIATE` 트랜잭션 안에서 UNIQUE 검증 +
//! INSERT/UPDATE 를 수행한다. 단일 사용자 모델이라 race condition 실질 불가하지만 sqlx
//! 패턴 일관성 + 향후 멀티 사용자 확장 안전망.
//!
//! ## UNIQUE 위반 처리
//!
//! `serial_no UNIQUE` 충돌 시 한국어 사용자 메시지로 변환:
//! `"일련번호 '{n}'은(는) 이미 사용 중입니다. 다른 번호를 지정하거나 자동 채번을 사용해 주세요."`

use crate::commands::audit::{self, AuditEventType};
use crate::commands::db;
use crate::commands::pagination::clamp_list_limit;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

/// 성별 — DB 에는 'male'/'female' 로 저장. PRD §4.1.1.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    fn as_db_code(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
        }
    }

    fn from_db_code(s: &str) -> Result<Self, AppError> {
        match s {
            "male" => Ok(Self::Male),
            "female" => Ok(Self::Female),
            other => Err(AppError::Config(format!("알 수 없는 성별 코드: {}", other))),
        }
    }
}

/// 학교급 — DB 에는 'elementary'/'middle' 로 저장. PRD §4.1.1 (초·중 한정).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SchoolLevel {
    Elementary,
    Middle,
}

impl SchoolLevel {
    fn as_db_code(self) -> &'static str {
        match self {
            Self::Elementary => "elementary",
            Self::Middle => "middle",
        }
    }

    fn from_db_code(s: &str) -> Result<Self, AppError> {
        match s {
            "elementary" => Ok(Self::Elementary),
            "middle" => Ok(Self::Middle),
            other => Err(AppError::Config(format!(
                "알 수 없는 학교급 코드: {}",
                other
            ))),
        }
    }
}

/// 정렬 옵션 — PRD §4.1 화면 요구사항 (이름순/입교일 역순/학년순).
///
/// Sprint 19 T1(사용자 요청 1,2번): 기본 정렬을 학년별+이름 가나다순(`GradeAsc`)으로 통일.
/// 모든 정렬 기준은 동일 값 tie-break 로 `name ASC` 를 포함해 "동일 학년(또는 동일 값) 정렬 시
/// 이름 가나다순 2차 정렬 자동 적용"을 만족한다.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StudentSort {
    SerialAsc,
    SerialDesc,
    NameAsc,
    NameDesc,
    /// 학년(학교급 포함) 오름차순 — Sprint 19 T1 신규 디폴트
    #[default]
    GradeAsc,
    GradeDesc,
    EnrollDateAsc,
    EnrollDateDesc,
    GenderAsc,
    GenderDesc,
    /// 주간 수업시간 오름차순 — `list_students` 의 `weekly_hours` correlated subquery 별칭 참조
    WeeklyHoursAsc,
    WeeklyHoursDesc,
}

impl StudentSort {
    fn order_by_sql(self) -> &'static str {
        match self {
            // serial_no 는 TEXT 컬럼이라 CAST 후 정렬 (숫자 채번 정합성). 비숫자 serial 은 뒤로.
            Self::SerialAsc => {
                "ORDER BY CASE WHEN serial_no GLOB '[0-9]*' THEN 0 ELSE 1 END, \
                 CAST(serial_no AS INTEGER) ASC, serial_no ASC"
            }
            Self::SerialDesc => {
                "ORDER BY CASE WHEN serial_no GLOB '[0-9]*' THEN 0 ELSE 1 END, \
                 CAST(serial_no AS INTEGER) DESC, serial_no DESC"
            }
            Self::NameAsc => "ORDER BY name ASC",
            Self::NameDesc => "ORDER BY name DESC",
            Self::GradeAsc => "ORDER BY school_level ASC, grade ASC, name ASC",
            Self::GradeDesc => "ORDER BY school_level DESC, grade DESC, name ASC",
            Self::EnrollDateAsc => "ORDER BY enroll_date ASC, id ASC",
            Self::EnrollDateDesc => "ORDER BY enroll_date DESC, id DESC",
            Self::GenderAsc => "ORDER BY gender ASC, name ASC",
            Self::GenderDesc => "ORDER BY gender DESC, name ASC",
            Self::WeeklyHoursAsc => "ORDER BY weekly_hours ASC, name ASC",
            Self::WeeklyHoursDesc => "ORDER BY weekly_hours DESC, name ASC",
        }
    }
}

/// 원생 — IPC 응답.
///
/// `weekly_hours`/`schedule_days_csv` 는 list_students 만 제공 (correlated subquery).
/// get_student / create_student / update_student RETURNING 절에는 없으며 None 으로 채워진다.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Student {
    pub id: i64,
    pub serial_no: String,
    pub name: String,
    pub gender: Gender,
    pub school_level: SchoolLevel,
    pub grade: i64,
    pub school_id: Option<i64>,
    pub phone_student: Option<String>,
    pub phone_mother: Option<String>,
    pub phone_father: Option<String>,
    /// 생년월일 (선택, YYYY-MM-DD). 미입력 시 None.
    pub birth_date: Option<String>,
    pub enroll_date: String,
    pub withdraw_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// T11 이슈 #4: 원생 목록에 주총 수업시간/요일 표시. list_students 만 제공.
    pub weekly_hours: Option<i64>,
    /// 현행 스케줄 요일 콤마 구분 — "1,3,5" (월/수/금). list_students 만 제공.
    pub schedule_days_csv: Option<String>,
}

impl Student {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        Ok(Self {
            id: row.try_get("id")?,
            serial_no: row.try_get("serial_no")?,
            name: row.try_get("name")?,
            gender: Gender::from_db_code(row.try_get::<&str, _>("gender")?)?,
            school_level: SchoolLevel::from_db_code(row.try_get::<&str, _>("school_level")?)?,
            grade: row.try_get("grade")?,
            school_id: row.try_get("school_id")?,
            phone_student: row.try_get("phone_student")?,
            phone_mother: row.try_get("phone_mother")?,
            phone_father: row.try_get("phone_father")?,
            birth_date: row.try_get("birth_date")?,
            enroll_date: row.try_get("enroll_date")?,
            withdraw_date: row.try_get("withdraw_date")?,
            // list_students 외 SELECT 에는 컬럼이 없으므로 try_get().ok() 로 None fallback
            weekly_hours: row.try_get("weekly_hours").ok(),
            schedule_days_csv: row.try_get("schedule_days_csv").ok(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// 신규 원생 등록 payload — `serial_no=None` 이면 자동 채번.
#[derive(Debug, Deserialize)]
pub struct NewStudent {
    pub serial_no: Option<String>,
    pub name: String,
    pub gender: Gender,
    pub school_level: SchoolLevel,
    pub grade: i64,
    pub school_id: Option<i64>,
    pub phone_student: Option<String>,
    pub phone_mother: Option<String>,
    pub phone_father: Option<String>,
    pub birth_date: Option<String>,
    pub enroll_date: String,
}

/// 원생 정보 수정 payload — PUT-like (전체 필드 받음). 부분 업데이트가 필요해지면 별도 IPC 추가.
#[derive(Debug, Deserialize)]
pub struct StudentUpdate {
    pub serial_no: String,
    pub name: String,
    pub gender: Gender,
    pub school_level: SchoolLevel,
    pub grade: i64,
    pub school_id: Option<i64>,
    pub phone_student: Option<String>,
    pub phone_mother: Option<String>,
    pub phone_father: Option<String>,
    pub birth_date: Option<String>,
    pub enroll_date: String,
    pub withdraw_date: Option<String>,
}

/// 원생 목록 조회 필터 — 모든 필드 Optional. 미지정 시 해당 조건 무시.
///
/// R14 페이지네이션: `limit`/`offset` 미지정 시 [`clamp_list_limit`] 정책으로 정규화된다.
/// `count_students` 는 동일 필터(`limit`/`offset` 제외)로 총 건수를 반환한다.
#[derive(Debug, Deserialize, Default)]
pub struct StudentFilter {
    pub active_only: Option<bool>,
    pub name_query: Option<String>,
    pub school_level: Option<SchoolLevel>,
    pub grade: Option<i64>,
    pub school_id: Option<i64>,
    pub gender: Option<Gender>,
    pub day_of_week: Option<i64>,
    pub sort: Option<StudentSort>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// ============================================================================
// 내부 헬퍼
// ============================================================================

/// UNIQUE 제약 위반 (특히 `serial_no UNIQUE`) 을 사용자 친화 한국어로 변환.
///
/// sqlx 의 `Error::Database` 에서 SQLite 에러 코드 2067 (`SQLITE_CONSTRAINT_UNIQUE`) 또는
/// 메시지 패턴으로 감지. 그 외 DB 오류는 그대로 전파.
fn map_serial_unique_violation(serial: &str, err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        let msg = db_err.message();
        if msg.contains("UNIQUE") && msg.contains("serial_no") {
            return AppError::UserFacing(format!(
                "일련번호 '{}'은(는) 이미 사용 중입니다. 다른 번호를 지정하거나 자동 채번을 사용해 주세요.",
                serial
            ));
        }
    }
    AppError::Db(err)
}

/// 숫자 문자열 행만 대상으로 `MAX(CAST AS INTEGER) + 1` 을 산출 (PI-05).
async fn compute_next_serial(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<String, AppError> {
    let row = sqlx::query(
        "SELECT COALESCE(MAX(CAST(serial_no AS INTEGER)), 0) + 1 AS next \
         FROM students WHERE serial_no GLOB '[0-9]*'",
    )
    .fetch_one(&mut **tx)
    .await?;
    let next: i64 = row.try_get("next")?;
    Ok(next.to_string())
}

// ============================================================================
// Tauri IPC commands
// ============================================================================

/// UI 등록 폼 기본값 표시용 — 다음 자동 채번 후보를 반환.
///
/// 실제 INSERT 와 race-free 보장은 [`create_student`] 의 트랜잭션이 담당. 본 IPC 는 advisory.
#[tauri::command]
pub async fn next_serial_number() -> Result<String, String> {
    let pool = db::pool().map_err(String::from)?;
    let mut tx = pool.begin().await.map_err(AppError::Db).map_err(String::from)?;
    let next = compute_next_serial(&mut tx).await.map_err(String::from)?;
    tx.rollback().await.map_err(AppError::Db).map_err(String::from)?;
    Ok(next)
}

/// 신규 원생을 등록한다. PI-05 자동 채번 또는 사용자 override.
#[tauri::command]
pub async fn create_student(payload: NewStudent) -> Result<Student, String> {
    let pool = db::pool().map_err(String::from)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    let student = insert_student_tx(&mut tx, &payload)
        .await
        .map_err(String::from)?;
    tx.commit().await.map_err(AppError::Db).map_err(String::from)?;

    // R13 PII 마스킹: 원생 이름은 details 에 기록하지 않는다 — event_subject(serial_no) 만으로 추적 가능.
    audit::try_record(AuditEventType::StudentCreated, Some(&student.serial_no), None).await;
    Ok(student)
}

/// 주어진 트랜잭션 안에서 원생 1건을 INSERT 하고 `Student` 를 반환한다 (commit·audit 미포함).
///
/// `create_student`(단건)와 CSV 일괄 가져오기(`import::import_students_csv`)가 공유한다.
/// 후자는 전체 행을 **하나의 트랜잭션**으로 묶어 중간 실패 시 부분 삽입을 방지한다(코드리뷰 C2).
pub(crate) async fn insert_student_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: &NewStudent,
) -> Result<Student, AppError> {
    // BEGIN IMMEDIATE 의 효과를 위해 즉시 쓰기 의도를 표시 — sqlx 의 begin() 은 deferred.
    // 단일 사용자 모델이라 실질 race 없음. PI-05 안전망으로 INSERT 전 MAX 조회를 같은 tx 안에서 수행.
    sqlx::query("SELECT 1 FROM students LIMIT 0")
        .execute(&mut **tx)
        .await
        .map_err(AppError::Db)?;

    let serial = match payload.serial_no.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => compute_next_serial(tx).await?,
    };

    let row = sqlx::query(
        "INSERT INTO students \
            (serial_no, name, gender, school_level, grade, school_id, \
             phone_student, phone_mother, phone_father, birth_date, enroll_date) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, serial_no, name, gender, school_level, grade, school_id, \
                   phone_student, phone_mother, phone_father, birth_date, enroll_date, withdraw_date, \
                   created_at, updated_at",
    )
    .bind(&serial)
    .bind(&payload.name)
    .bind(payload.gender.as_db_code())
    .bind(payload.school_level.as_db_code())
    .bind(payload.grade)
    .bind(payload.school_id)
    .bind(payload.phone_student.as_deref())
    .bind(payload.phone_mother.as_deref())
    .bind(payload.phone_father.as_deref())
    .bind(payload.birth_date.as_deref())
    .bind(&payload.enroll_date)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| map_serial_unique_violation(&serial, e))?;

    Student::from_row(&row)
}

/// 원생 정보를 PUT-like 로 갱신한다.
///
/// **T6 (사용자 이슈 #5)**: serial_no 는 PI-05 자동 채번/사용자 override 로 등록 시점에만
/// 결정되며 수정 불가능 — 본 UPDATE SQL 에서 serial_no 컬럼을 제외하여 payload 의 값을
/// 무시한다. 프론트는 readonly 표시하지만 백엔드도 가드(defense in depth).
#[tauri::command]
pub async fn update_student(id: i64, payload: StudentUpdate) -> Result<Student, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "UPDATE students SET \
            name = ?, gender = ?, school_level = ?, grade = ?, \
            school_id = ?, phone_student = ?, phone_mother = ?, phone_father = ?, \
            birth_date = ?, enroll_date = ?, withdraw_date = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, serial_no, name, gender, school_level, grade, school_id, \
                   phone_student, phone_mother, phone_father, birth_date, enroll_date, withdraw_date, \
                   created_at, updated_at",
    )
    .bind(&payload.name)
    .bind(payload.gender.as_db_code())
    .bind(payload.school_level.as_db_code())
    .bind(payload.grade)
    .bind(payload.school_id)
    .bind(payload.phone_student.as_deref())
    .bind(payload.phone_mother.as_deref())
    .bind(payload.phone_father.as_deref())
    .bind(payload.birth_date.as_deref())
    .bind(&payload.enroll_date)
    .bind(payload.withdraw_date.as_deref())
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| map_serial_unique_violation(&payload.serial_no, e))
    .map_err(String::from)?;

    let row = row.ok_or_else(|| {
        String::from(AppError::UserFacing(format!("원생을 찾을 수 없습니다 (id={}).", id)))
    })?;
    let student = Student::from_row(&row).map_err(String::from)?;
    // R13 PII 마스킹: 원생 이름 미기록 — event_subject(serial_no) 만 기록.
    audit::try_record(AuditEventType::StudentUpdated, Some(&student.serial_no), None).await;
    Ok(student)
}

#[tauri::command]
pub async fn get_student(id: i64) -> Result<Student, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "SELECT id, serial_no, name, gender, school_level, grade, school_id, \
                phone_student, phone_mother, phone_father, birth_date, enroll_date, withdraw_date, \
                created_at, updated_at \
         FROM students WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    let row = row.ok_or_else(|| {
        String::from(AppError::UserFacing(format!("원생을 찾을 수 없습니다 (id={}).", id)))
    })?;
    Student::from_row(&row).map_err(String::from)
}

/// 원생을 퇴교 처리한다 — `withdraw_date` 만 설정.
#[tauri::command]
pub async fn withdraw_student(id: i64, withdraw_date: String) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let result = sqlx::query(
        "UPDATE students SET \
            withdraw_date = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ?",
    )
    .bind(&withdraw_date)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    if result.rows_affected() == 0 {
        return Err(String::from(AppError::UserFacing(format!(
            "원생을 찾을 수 없습니다 (id={}).",
            id
        ))));
    }
    // R13 PII 마스킹: 퇴교 일자도 민감 정보로 분류 — event_subject(student id) 만 기록.
    audit::try_record(AuditEventType::StudentWithdrawn, Some(&id.to_string()), None).await;
    Ok(())
}

/// 퇴교 처리를 번복한다 — withdraw_date 를 NULL 로 되돌리고 퇴교 시 강제 소멸된
/// 미보강 결석(자연 만기 전) 을 absent 상태로 환원한다.
///
/// **T8 (Sprint 4)**: 퇴교 번복 — withdraw_date NULL 복귀.
/// **hotfix (Sprint 10 post-merge)**: 퇴교 처리에서 `process_withdrawal_makeup` 으로 강제
/// 전이된 `makeup_expired` 결석 중 자연 만기 전(`makeup_deadline >= 현재 YYYY-MM`) 항목만
/// `absent` 로 환원. 자연 만기 소멸은 T5 폐기 정책에 따라 환원 대상 외.
#[tauri::command]
pub async fn reinstate_student(id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let revived_ids = reinstate_student_impl(pool, id).await?;
    let details = if revived_ids.is_empty() {
        None
    } else {
        Some(format!(r#"{{"revivedAbsenceIds":{:?}}}"#, revived_ids))
    };
    audit::try_record(
        AuditEventType::StudentReinstated,
        Some(&id.to_string()),
        details.as_deref(),
    )
    .await;
    Ok(())
}

pub(crate) async fn reinstate_student_impl(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> Result<Vec<i64>, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    // absence_memo 도 함께 NULL 로 클리어 — 퇴교 외부 처리 메모(`ExternalExpire`)가
    // 일괄 덮어쓴 결과이므로 환원 시 의미가 사라진다. `attendance.rs::toggle_attendance`
    // 의 결석 → 출석 전환 시 동일 패턴(absence_memo=NULL).
    let revived_ids: Vec<i64> = sqlx::query_scalar(
        "UPDATE regular_attendances \
         SET status = 'absent', \
             absence_memo = NULL, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE student_id = ? \
           AND status = 'makeup_expired' \
           AND makeup_attendance_id IS NULL \
           AND makeup_deadline IS NOT NULL \
           AND makeup_deadline >= strftime('%Y-%m', 'now') \
         RETURNING id",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("결석 환원 실패: {}", e))?;

    let result = sqlx::query(
        "UPDATE students SET \
            withdraw_date = NULL, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    if result.rows_affected() == 0 {
        return Err(String::from(AppError::UserFacing(format!(
            "원생을 찾을 수 없습니다 (id={}).",
            id
        ))));
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;
    Ok(revived_ids)
}

/// `list_students` / `count_students` 가 공유하는 SQL fragment 빌더.
///
/// 반환값: `(WHERE 절, JOIN 절)`. 각각 비어 있으면 빈 문자열.
/// 호출자는 두 단편을 SQL 본문에 그대로 push 하면 된다 — 추가 분기 불요.
/// SQL 동적 빌드는 정적 단편의 조건부 연결로만 구성 — 사용자 입력은 모두 bind 로 전달.
fn build_filter_clause(filter: &StudentFilter) -> (String, String) {
    let join_sql = if filter.day_of_week.is_some() {
        "INNER JOIN student_schedules sch \
            ON sch.student_id = s.id AND sch.effective_to IS NULL AND sch.day_of_week = ? "
            .to_string()
    } else {
        String::new()
    };

    let mut conditions: Vec<&'static str> = Vec::new();
    if filter.active_only.unwrap_or(false) {
        conditions.push("s.withdraw_date IS NULL");
    }
    if filter.name_query.is_some() {
        conditions.push("s.name LIKE ?");
    }
    if filter.school_level.is_some() {
        conditions.push("s.school_level = ?");
    }
    if filter.grade.is_some() {
        conditions.push("s.grade = ?");
    }
    if filter.school_id.is_some() {
        conditions.push("s.school_id = ?");
    }
    if filter.gender.is_some() {
        conditions.push("s.gender = ?");
    }
    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {} ", conditions.join(" AND "))
    };
    (where_sql, join_sql)
}

/// 공유 헬퍼: 필터 bind 순서를 SQL fragment 순서와 일치시킨다.
/// `day_of_week` → `name_query` → `school_level` → `grade` → `school_id` → `gender`.
fn bind_filter<'q>(
    mut q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    filter: &'q StudentFilter,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    if let Some(d) = filter.day_of_week {
        q = q.bind(d);
    }
    if let Some(ref n) = filter.name_query {
        q = q.bind(format!("%{}%", n));
    }
    if let Some(l) = filter.school_level {
        q = q.bind(l.as_db_code());
    }
    if let Some(g) = filter.grade {
        q = q.bind(g);
    }
    if let Some(s) = filter.school_id {
        q = q.bind(s);
    }
    if let Some(g) = filter.gender {
        q = q.bind(g.as_db_code());
    }
    q
}

/// 원생 목록을 다중 필터 + 정렬 + 페이지네이션으로 조회한다.
///
/// `day_of_week` 필터는 `student_schedules` JOIN 으로 현행 스케줄 보유 여부 검증.
/// 그 외 필터는 students 자체 컬럼.
/// R14: 페이지네이션 정책은 [`clamp_list_limit`] 참조.
#[tauri::command]
pub async fn list_students(filter: StudentFilter) -> Result<Vec<Student>, String> {
    let pool = db::pool().map_err(String::from)?;
    let (where_sql, join_sql) = build_filter_clause(&filter);
    let limit = clamp_list_limit(filter.limit);
    let offset = filter.offset.unwrap_or(0);

    // T11 이슈 #4: correlated subquery 로 현행 스케줄 요약 동봉 — N+1 IPC 회피.
    // SQLite 가 자동 최적화 (사용자 ~100명 규모에서 PRAGMA cache_size 만으로 충분).
    let mut sql = String::from(
        "SELECT s.id, s.serial_no, s.name, s.gender, s.school_level, s.grade, s.school_id, \
                s.phone_student, s.phone_mother, s.phone_father, s.birth_date, s.enroll_date, s.withdraw_date, \
                s.created_at, s.updated_at, \
                (SELECT COALESCE(SUM(duration_hours), 0) FROM student_schedules \
                 WHERE student_id = s.id AND effective_to IS NULL) AS weekly_hours, \
                (SELECT GROUP_CONCAT(day_of_week) FROM \
                 (SELECT day_of_week FROM student_schedules \
                  WHERE student_id = s.id AND effective_to IS NULL \
                  ORDER BY day_of_week)) AS schedule_days_csv \
         FROM students s ",
    );
    sql.push_str(&join_sql);
    sql.push_str(&where_sql);
    sql.push_str(filter.sort.unwrap_or_default().order_by_sql());
    sql.push_str(" LIMIT ? OFFSET ?");

    let mut q = sqlx::query(&sql);
    q = bind_filter(q, &filter);
    q = q.bind(limit).bind(offset);

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    rows.iter()
        .map(Student::from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(String::from)
}

/// 동일 필터에 매칭되는 총 원생 수를 반환한다 (R14 페이지네이션 UI 보조).
///
/// `limit`/`offset` 필드는 무시한다 — 필터 조합 자체의 총 건수를 반환.
#[tauri::command]
pub async fn count_students(filter: StudentFilter) -> Result<i64, String> {
    let pool = db::pool().map_err(String::from)?;
    let (where_sql, join_sql) = build_filter_clause(&filter);

    let mut sql = String::from("SELECT COUNT(*) AS cnt FROM students s ");
    sql.push_str(&join_sql);
    sql.push_str(&where_sql);

    let q = bind_filter(sqlx::query(&sql), &filter);
    let row = q
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    row.try_get::<i64, _>("cnt")
        .map_err(AppError::Db)
        .map_err(String::from)
}

// ============================================================================
// Sprint 19 T8 — 학년 자동 승급 (매년 1월 이후 최초 실행, 사용자 확인 후 일괄 적용)
// ============================================================================

/// `app_settings` 키 — 마지막으로 학년 승급을 적용한 연도(YYYY). `diagnosis.rs`의
/// `LAST_AUTO_DIAGNOSIS_KEY` 패턴과 동일 — 값은 실제 승급을 실행했을 때만 갱신한다
/// (조회만 하는 `check_grade_promotion`은 이 키를 쓰지 않음).
const LAST_GRADE_PROMOTION_KEY: &str = "last_grade_promotion_year";

/// 학년 승급 대상 WHERE 절 — 재원생(withdraw_date IS NULL) 중 학교급별 최대 학년
/// 미만(초등 <6, 중등 <3)만 대상. `check_grade_promotion`/`promote_grades` 공유.
const GRADE_PROMOTION_WHERE: &str = "withdraw_date IS NULL \
     AND ((school_level = 'elementary' AND grade < 6) \
          OR (school_level = 'middle' AND grade < 3))";

/// 자동 승급 기능 도입 첫 해(2026) 는 대상에서 제외 — 사용자 결정(2026-07-07).
/// 학사 연도 중간에 배포되어 기존 원생 학년이 이미 현재 상태로 정리돼 있으므로,
/// 이 해에는 승급을 건너뛰고 다음 해(2027)부터 정상 적용한다.
const EXCLUDED_PROMOTION_YEAR: &str = "2026";

fn current_year() -> String {
    chrono::Local::now().format("%Y").to_string()
}

/// 학년 승급 필요 여부 조회 결과.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GradePromotionCheck {
    /// 올해 아직 승급을 실행하지 않았고, 대상 원생이 1명 이상 존재하면 true.
    pub needed: bool,
    /// 승급 대상 원생 수 (재원생 + 학교급별 최대 학년 미만).
    pub count: i64,
}

/// `year` 를 인자로 받아 테스트에서 실제 시스템 연도(및 `EXCLUDED_PROMOTION_YEAR`)와
/// 무관하게 검증할 수 있게 한다. IPC 래퍼만 `current_year()`를 호출.
async fn check_grade_promotion_impl(
    pool: &sqlx::SqlitePool,
    year: &str,
) -> Result<GradePromotionCheck, AppError> {
    if year == EXCLUDED_PROMOTION_YEAR {
        return Ok(GradePromotionCheck { needed: false, count: 0 });
    }

    let last: Option<String> = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?")
        .bind(LAST_GRADE_PROMOTION_KEY)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)?;
    if last.as_deref() == Some(year) {
        return Ok(GradePromotionCheck { needed: false, count: 0 });
    }

    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM students WHERE {GRADE_PROMOTION_WHERE}"
    ))
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(GradePromotionCheck { needed: count > 0, count })
}

/// 올해 학년 승급이 필요한지 조회한다 (IPC — 조회 전용, DB 변경 없음).
///
/// 이미 올해 승급을 실행했으면(`last_grade_promotion_year`=올해) 대상이 있어도 `needed=false`.
/// 프론트엔드는 `needed && count > 0` 일 때만 확인 다이얼로그를 표시한다.
#[tauri::command]
pub async fn check_grade_promotion() -> Result<GradePromotionCheck, String> {
    let pool = db::pool().map_err(String::from)?;
    check_grade_promotion_impl(pool, &current_year())
        .await
        .map_err(String::from)
}

async fn promote_grades_impl(pool: &sqlx::SqlitePool, year: &str) -> Result<i64, AppError> {
    if year == EXCLUDED_PROMOTION_YEAR {
        return Ok(0);
    }
    let mut tx = pool.begin().await.map_err(AppError::Db)?;

    let result = sqlx::query(&format!(
        "UPDATE students SET grade = grade + 1 WHERE {GRADE_PROMOTION_WHERE}"
    ))
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)?;
    let promoted = result.rows_affected() as i64;

    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(LAST_GRADE_PROMOTION_KEY)
    .bind(year)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)?;

    tx.commit().await.map_err(AppError::Db)?;

    audit::try_record(
        AuditEventType::GradesPromoted,
        Some(year),
        Some(&format!(r#"{{"promoted_count":{promoted}}}"#)),
    )
    .await;

    Ok(promoted)
}

/// 학년 승급을 일괄 실행한다 (사용자가 확인 다이얼로그에서 승인한 후에만 호출).
///
/// 재원생 중 학교급별 최대 학년 미만인 원생 전원의 `grade`를 1 증가시키고,
/// `last_grade_promotion_year`를 올해로 기록해 같은 해 중복 승급을 방지한다.
#[tauri::command]
pub async fn promote_grades() -> Result<i64, String> {
    let pool = db::pool().map_err(String::from)?;
    promote_grades_impl(pool, &current_year()).await.map_err(String::from)
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    fn sample_payload(serial: Option<&str>) -> NewStudent {
        NewStudent {
            serial_no: serial.map(String::from),
            name: "홍길동".to_string(),
            gender: Gender::Male,
            school_level: SchoolLevel::Elementary,
            grade: 3,
            school_id: None,
            phone_student: None,
            phone_mother: Some("010-0000-0000".to_string()),
            phone_father: None,
            birth_date: Some("2017-05-10".to_string()),
            enroll_date: "2026-03-01".to_string(),
        }
    }

    #[test]
    fn gender_round_trip_serde() {
        for g in [Gender::Male, Gender::Female] {
            let json = serde_json::to_string(&g).unwrap();
            let back: Gender = serde_json::from_str(&json).unwrap();
            assert_eq!(g, back);
        }
        assert_eq!(serde_json::to_string(&Gender::Male).unwrap(), r#""male""#);
    }

    #[test]
    fn school_level_round_trip_serde() {
        for s in [SchoolLevel::Elementary, SchoolLevel::Middle] {
            let json = serde_json::to_string(&s).unwrap();
            let back: SchoolLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
        assert_eq!(
            serde_json::to_string(&SchoolLevel::Elementary).unwrap(),
            r#""elementary""#
        );
    }

    #[test]
    fn student_sort_default_is_grade_asc() {
        // Sprint 19 T1 사용자 요청: 원생 목록 디폴트 정렬은 학년별+이름 가나다순.
        assert_eq!(StudentSort::default(), StudentSort::GradeAsc);
    }

    #[test]
    fn serial_asc_sql_uses_cast_integer() {
        // PI-05 자동 채번이 숫자 문자열이라 TEXT 정렬이 아닌 CAST INTEGER 필수.
        let sql = StudentSort::SerialAsc.order_by_sql();
        assert!(sql.contains("CAST(serial_no AS INTEGER)"));
        assert!(sql.contains("ASC"));
    }

    #[test]
    fn grade_asc_sorts_school_level_then_grade_then_name() {
        // Sprint 19 T1: 학년 정렬은 학교급(초→중) 우선, 동일 학년 내 이름 가나다순 tie-break.
        let sql = StudentSort::GradeAsc.order_by_sql();
        assert_eq!(sql, "ORDER BY school_level ASC, grade ASC, name ASC");
    }

    #[test]
    fn gender_and_weekly_hours_sort_have_name_tiebreak() {
        // Sprint 19 T1(사용자 요청 2번): 모든 정렬 기준은 동일 값 tie-break 로 이름순을 포함한다.
        for sql in [
            StudentSort::GenderAsc.order_by_sql(),
            StudentSort::GenderDesc.order_by_sql(),
            StudentSort::WeeklyHoursAsc.order_by_sql(),
            StudentSort::WeeklyHoursDesc.order_by_sql(),
        ] {
            assert!(sql.ends_with("name ASC"), "tie-break 누락: {sql}");
        }
    }

    #[tokio::test]
    async fn insert_student_tx_rollback_discards_all() {
        // 코드리뷰 C2: import 가 여러 행을 단일 트랜잭션으로 묶을 때, 중간 롤백 시
        // 이전에 삽입한 행도 남지 않아야 한다(부분 삽입 방지).
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let mut tx = pool.begin().await.expect("tx 시작");
        insert_student_tx(&mut tx, &sample_payload(Some("100")))
            .await
            .expect("첫 행 삽입");
        insert_student_tx(&mut tx, &sample_payload(Some("101")))
            .await
            .expect("둘째 행 삽입");
        tx.rollback().await.expect("롤백");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0, "롤백 후 어떤 행도 커밋되지 않아야 함");
    }

    #[tokio::test]
    async fn insert_student_tx_commit_persists_all() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let mut tx = pool.begin().await.expect("tx 시작");
        insert_student_tx(&mut tx, &sample_payload(Some("100")))
            .await
            .expect("삽입");
        insert_student_tx(&mut tx, &sample_payload(Some("101")))
            .await
            .expect("삽입");
        tx.commit().await.expect("커밋");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2, "커밋 후 2행 모두 존재");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn auto_serial_increments_continuously() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");

        async fn create(pool: &sqlx::SqlitePool, serial: Option<&str>) -> String {
            let p = super::tests::sample_payload(serial);
            let mut tx = pool.begin().await.unwrap();
            let s = match p.serial_no.as_deref() {
                Some(v) if !v.is_empty() => v.to_string(),
                _ => compute_next_serial(&mut tx).await.unwrap(),
            };
            sqlx::query(
                "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&s)
            .bind(&p.name)
            .bind(p.gender.as_db_code())
            .bind(p.school_level.as_db_code())
            .bind(p.grade)
            .bind(&p.enroll_date)
            .execute(&mut *tx)
            .await
            .unwrap();
            tx.commit().await.unwrap();
            s
        }

        assert_eq!(create(&pool, None).await, "1");
        assert_eq!(create(&pool, None).await, "2");
        assert_eq!(create(&pool, None).await, "3");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn override_then_auto_continues_from_max() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");

        // 1, 2 자동 채번
        for _ in 0..2 {
            let mut tx = pool.begin().await.unwrap();
            let s = compute_next_serial(&mut tx).await.unwrap();
            sqlx::query(
                "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
                 VALUES (?, '학생', 'male', 'elementary', 1, '2026-03-01')",
            )
            .bind(&s)
            .execute(&mut *tx)
            .await
            .unwrap();
            tx.commit().await.unwrap();
        }
        // 사용자 100 override
        sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES ('100', '학생', 'male', 'elementary', 1, '2026-03-01')",
        )
        .execute(&pool)
        .await
        .unwrap();
        // 다음 자동 채번 = 101 (MAX(1,2,100)+1)
        let mut tx = pool.begin().await.unwrap();
        let next = compute_next_serial(&mut tx).await.unwrap();
        assert_eq!(next, "101");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn unique_violation_returns_korean_message() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES ('A-001', '학생A', 'male', 'elementary', 1, '2026-03-01')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let err = sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES ('A-001', '학생B', 'female', 'middle', 1, '2026-03-01')",
        )
        .execute(&pool)
        .await
        .expect_err("UNIQUE 위반");
        let mapped = map_serial_unique_violation("A-001", err);
        let msg: String = mapped.into();
        assert!(msg.contains("일련번호"), "msg={}", msg);
        assert!(msg.contains("이미 사용"));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn alphanumeric_serial_excluded_from_auto_max() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // 영문 prefix 행 — 자동 채번 대상에서 제외
        sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES ('A-001', '학생', 'male', 'elementary', 1, '2026-03-01')",
        )
        .execute(&pool)
        .await
        .unwrap();
        // 숫자 행 5
        sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES ('5', '학생', 'male', 'elementary', 1, '2026-03-01')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let next = compute_next_serial(&mut tx).await.unwrap();
        assert_eq!(next, "6", "숫자 행 MAX=5 → 다음 6 (A-001 무시)");
    }

    #[test]
    fn check_constraints_reject_invalid_values() {
        // CHECK 제약 자체는 마이그레이션이 보장 — db::tests::in_memory_pool_runs_migrations 가 적용 검증.
        // 본 테스트는 enum from_db_code 검증.
        assert!(Gender::from_db_code("invalid").is_err());
        assert!(SchoolLevel::from_db_code("high").is_err()); // PRD §4.1.1 초·중 한정
    }

    // ------------------------------------------------------------------------
    // R14 페이지네이션 — 필터 빌더 + COUNT 정확성
    // (limit 정규화 정책 자체는 `pagination::clamp_list_limit` 단위 테스트에서 검증)
    // ------------------------------------------------------------------------

    #[test]
    fn build_filter_clause_empty_filter_returns_empty_strings() {
        let (where_sql, join_sql) = build_filter_clause(&StudentFilter::default());
        assert_eq!(where_sql, "");
        assert_eq!(join_sql, "");
    }

    #[test]
    fn build_filter_clause_with_day_of_week_emits_join() {
        let filter = StudentFilter {
            day_of_week: Some(1),
            ..Default::default()
        };
        let (where_sql, join_sql) = build_filter_clause(&filter);
        assert!(join_sql.starts_with("INNER JOIN student_schedules"));
        assert!(join_sql.contains("sch.day_of_week = ?"));
        assert_eq!(where_sql, "", "JOIN ON 조건은 WHERE 가 아니라 별도 절");
    }

    #[test]
    fn build_filter_clause_combines_multiple_conditions() {
        let filter = StudentFilter {
            active_only: Some(true),
            name_query: Some("홍".to_string()),
            school_level: Some(SchoolLevel::Elementary),
            ..Default::default()
        };
        let (where_sql, _) = build_filter_clause(&filter);
        assert!(where_sql.starts_with("WHERE "));
        assert!(where_sql.contains("s.withdraw_date IS NULL"));
        assert!(where_sql.contains("s.name LIKE ?"));
        assert!(where_sql.contains("s.school_level = ?"));
        assert_eq!(
            where_sql.matches(" AND ").count(),
            2,
            "조건 3개 → AND 2번"
        );
    }

    #[cfg(not(feature = "cipher"))]
    async fn seed_students(pool: &sqlx::SqlitePool, count: i64) {
        for i in 1..=count {
            sqlx::query(
                "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
                 VALUES (?, ?, 'male', 'elementary', 1, '2026-03-01')",
            )
            .bind(i.to_string())
            .bind(format!("학생{}", i))
            .execute(pool)
            .await
            .unwrap();
        }
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn list_students_respects_limit_and_offset() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_students(&pool, 7).await;

        // limit 적용 — name_asc 정렬 (학생1, 학생2, ...) 가정
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM students ORDER BY name ASC LIMIT ? OFFSET ?",
        )
        .bind(3u32)
        .bind(0u32)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 3, "limit=3 → 3건");

        // offset=3 → 4번째부터
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM students ORDER BY name ASC LIMIT ? OFFSET ?",
        )
        .bind(3u32)
        .bind(3u32)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 3, "offset=3, limit=3 → 다음 3건");

        // 마지막 페이지 — 잔여 1건
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM students ORDER BY name ASC LIMIT ? OFFSET ?",
        )
        .bind(3u32)
        .bind(6u32)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 1, "offset=6, limit=3 → 마지막 1건");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn count_matches_filtered_total() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_students(&pool, 5).await;

        // 1건 퇴교 처리 (active_only=true 시 4건 기대)
        sqlx::query("UPDATE students SET withdraw_date = '2026-04-01' WHERE serial_no = '1'")
            .execute(&pool)
            .await
            .unwrap();

        // 전체 COUNT
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(total.0, 5, "전체 5건");

        // active_only 시뮬레이션 — build_filter_clause + 직접 실행
        let filter = StudentFilter {
            active_only: Some(true),
            ..Default::default()
        };
        let (where_sql, _) = build_filter_clause(&filter);
        let sql = format!("SELECT COUNT(*) AS cnt FROM students s {}", where_sql);
        let row = sqlx::query(&sql).fetch_one(&pool).await.unwrap();
        let active_count: i64 = row.try_get("cnt").unwrap();
        assert_eq!(active_count, 4, "퇴교 1건 제외 → 4건");
    }

    /// hotfix (Sprint 10 post-merge): 퇴교 번복 시 강제 소멸된 결석 중 자연 만기 전인 항목만
    /// absent 로 환원한다. 자연 만기 항목(과거 makeup_deadline)은 그대로 유지.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn reinstate_revives_only_pre_natural_deadline_expired_absences() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");

        // 학생 1명 — 일단 퇴교 상태
        sqlx::query(
            "INSERT INTO students (id, serial_no, name, gender, school_level, grade, enroll_date, withdraw_date) \
             VALUES (1, '1', '홍길동', 'male', 'elementary', 4, '2026-03-01', '2026-05-28')",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 보강 완료 케이스 FK 대상 — id=1
        sqlx::query(
            "INSERT INTO makeup_attendances (id, student_id, event_date, year_month, status, class_minutes) \
             VALUES (1, 1, '2026-05-15', '2026-05', 'makeup_attended', 60)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 결석 A: 자연 만기 전 (이번 달) + 외부 처리 메모 — 환원 대상, memo 도 클리어
        // 결석 B: 자연 만기 후 (옛 달) — 환원 대상 외
        // 결석 C: makeup_expired 이지만 makeup_attendance_id 채워짐 → 환원 대상 외 (보강 완료)
        sqlx::query(
            "INSERT INTO regular_attendances \
                 (student_id, event_date, year_month, status, class_minutes, makeup_deadline, makeup_attendance_id, absence_memo) \
             VALUES \
                 (1, '2026-05-22', '2026-05', 'makeup_expired', 60,  strftime('%Y-%m','now'),  NULL, '환불 처리 완료'), \
                 (1, '2025-12-10', '2025-12', 'makeup_expired', 60, '2026-01',                 NULL, NULL), \
                 (1, '2026-04-15', '2026-04', 'makeup_expired', 60, '2026-06',                    1, NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let revived = super::reinstate_student_impl(&pool, 1).await.unwrap();
        assert_eq!(revived.len(), 1, "환원 대상은 1건 (결석 A)");

        let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT event_date, status, absence_memo FROM regular_attendances WHERE student_id = 1 ORDER BY event_date",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows[0].0, "2025-12-10", "자연 만기는 그대로");
        assert_eq!(rows[0].1, "makeup_expired", "자연 만기 status 유지");
        assert_eq!(rows[1].0, "2026-04-15", "보강 완료 expired 는 그대로");
        assert_eq!(rows[1].1, "makeup_expired", "보강 완료 expired status 유지");
        assert_eq!(rows[2].0, "2026-05-22", "퇴교 강제 expired 가 환원 대상");
        assert_eq!(rows[2].1, "absent", "status 가 absent 로 환원");
        assert!(rows[2].2.is_none(), "absence_memo 도 NULL 로 클리어");

        let withdraw_date: Option<String> = sqlx::query_scalar(
            "SELECT withdraw_date FROM students WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(withdraw_date.is_none(), "withdraw_date 도 NULL 로 복귀");
    }

    // ─────── Sprint 19 T8 — 학년 자동 승급 ───────

    #[cfg(not(feature = "cipher"))]
    async fn seed_grade_student(
        pool: &sqlx::SqlitePool,
        serial: &str,
        school_level: &str,
        grade: i64,
        withdraw_date: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, \
             enroll_date, withdraw_date) VALUES (?, ?, 'male', ?, ?, '2026-01-01', ?)",
        )
        .bind(serial)
        .bind(format!("학생{}", serial))
        .bind(school_level)
        .bind(grade)
        .bind(withdraw_date)
        .execute(pool)
        .await
        .expect("학생 INSERT");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn promote_grades_increments_elementary_and_middle() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_grade_student(&pool, "1", "elementary", 3, None).await;
        seed_grade_student(&pool, "2", "middle", 1, None).await;

        let promoted = promote_grades_impl(&pool, "2027").await.expect("승급");
        assert_eq!(promoted, 2);

        let grades: Vec<(i64,)> =
            sqlx::query_as("SELECT grade FROM students ORDER BY serial_no")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(grades[0].0, 4, "초등 3→4학년");
        assert_eq!(grades[1].0, 2, "중등 1→2학년");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn promote_grades_excludes_max_grade() {
        // 초6/중3(각 학교급 최대 학년)은 승급 대상에서 제외.
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_grade_student(&pool, "1", "elementary", 6, None).await;
        seed_grade_student(&pool, "2", "middle", 3, None).await;

        let promoted = promote_grades_impl(&pool, "2027").await.expect("승급");
        assert_eq!(promoted, 0, "최대 학년은 승급 대상 아님");

        let grades: Vec<(i64,)> =
            sqlx::query_as("SELECT grade FROM students ORDER BY serial_no")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(grades[0].0, 6);
        assert_eq!(grades[1].0, 3);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn promote_grades_excludes_withdrawn_students() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_grade_student(&pool, "1", "elementary", 3, Some("2026-02-01")).await;

        let promoted = promote_grades_impl(&pool, "2027").await.expect("승급");
        assert_eq!(promoted, 0, "퇴교생은 승급 대상 아님");

        let grade: i64 = sqlx::query_scalar("SELECT grade FROM students WHERE serial_no = '1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(grade, 3, "퇴교생 학년 불변");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn check_grade_promotion_skips_already_processed_year() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_grade_student(&pool, "1", "elementary", 3, None).await;

        let before = check_grade_promotion_impl(&pool, "2027").await.expect("조회");
        assert!(before.needed, "처음엔 대상 존재 → 승급 필요");
        assert_eq!(before.count, 1);

        promote_grades_impl(&pool, "2027").await.expect("승급 실행");

        let after = check_grade_promotion_impl(&pool, "2027").await.expect("재조회");
        assert!(!after.needed, "같은 해 재실행 후에는 스킵");
        assert_eq!(after.count, 0);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn check_grade_promotion_needed_false_when_no_eligible_students() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_grade_student(&pool, "1", "elementary", 6, None).await;

        let result = check_grade_promotion_impl(&pool, "2027").await.expect("조회");
        assert!(!result.needed, "대상 없으면 needed=false");
        assert_eq!(result.count, 0);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn check_and_promote_grade_excluded_for_2026() {
        // 사용자 결정(2026-07-07): 자동 승급 도입 첫 해(2026)는 대상 제외.
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        seed_grade_student(&pool, "1", "elementary", 3, None).await;

        let check = check_grade_promotion_impl(&pool, "2026").await.expect("조회");
        assert!(!check.needed, "2026년은 대상 있어도 needed=false");
        assert_eq!(check.count, 0);

        let promoted = promote_grades_impl(&pool, "2026").await.expect("승급 시도");
        assert_eq!(promoted, 0, "2026년은 방어적으로도 승급 0건");

        let grade: i64 = sqlx::query_scalar("SELECT grade FROM students WHERE serial_no = '1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(grade, 3, "2026년엔 학년 불변");
    }
}
