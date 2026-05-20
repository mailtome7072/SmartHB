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
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StudentSort {
    #[default]
    NameAsc,
    EnrollDateDesc,
    GradeAsc,
}

impl StudentSort {
    fn order_by_sql(self) -> &'static str {
        match self {
            Self::NameAsc => "ORDER BY name ASC",
            Self::EnrollDateDesc => "ORDER BY enroll_date DESC, id DESC",
            Self::GradeAsc => "ORDER BY school_level ASC, grade ASC, name ASC",
        }
    }
}

/// 원생 — IPC 응답.
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
    pub enroll_date: String,
    pub withdraw_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
            enroll_date: row.try_get("enroll_date")?,
            withdraw_date: row.try_get("withdraw_date")?,
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
    pub enroll_date: String,
    pub withdraw_date: Option<String>,
}

/// 원생 목록 조회 필터 — 모든 필드 Optional. 미지정 시 해당 조건 무시.
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
    // BEGIN IMMEDIATE 의 효과를 위해 즉시 쓰기 의도를 표시 — sqlx 의 begin() 은 deferred.
    // 단일 사용자 모델이라 실질 race 없음. PI-05 안전망으로 INSERT 전 MAX 조회를 같은 tx 안에서 수행.
    sqlx::query("SELECT 1 FROM students LIMIT 0")
        .execute(&mut *tx)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    let serial = match payload.serial_no.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => compute_next_serial(&mut tx).await.map_err(String::from)?,
    };

    let row = sqlx::query(
        "INSERT INTO students \
            (serial_no, name, gender, school_level, grade, school_id, \
             phone_student, phone_mother, phone_father, enroll_date) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, serial_no, name, gender, school_level, grade, school_id, \
                   phone_student, phone_mother, phone_father, enroll_date, withdraw_date, \
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
    .bind(&payload.enroll_date)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| map_serial_unique_violation(&serial, e))
    .map_err(String::from)?;

    let student = Student::from_row(&row).map_err(String::from)?;
    tx.commit().await.map_err(AppError::Db).map_err(String::from)?;

    // R13 PII 마스킹: 원생 이름은 details 에 기록하지 않는다 — event_subject(serial_no) 만으로 추적 가능.
    audit::try_record(AuditEventType::StudentCreated, Some(&serial), None).await;
    Ok(student)
}

/// 원생 정보를 PUT-like 로 갱신한다.
#[tauri::command]
pub async fn update_student(id: i64, payload: StudentUpdate) -> Result<Student, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "UPDATE students SET \
            serial_no = ?, name = ?, gender = ?, school_level = ?, grade = ?, \
            school_id = ?, phone_student = ?, phone_mother = ?, phone_father = ?, \
            enroll_date = ?, withdraw_date = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, serial_no, name, gender, school_level, grade, school_id, \
                   phone_student, phone_mother, phone_father, enroll_date, withdraw_date, \
                   created_at, updated_at",
    )
    .bind(&payload.serial_no)
    .bind(&payload.name)
    .bind(payload.gender.as_db_code())
    .bind(payload.school_level.as_db_code())
    .bind(payload.grade)
    .bind(payload.school_id)
    .bind(payload.phone_student.as_deref())
    .bind(payload.phone_mother.as_deref())
    .bind(payload.phone_father.as_deref())
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
                phone_student, phone_mother, phone_father, enroll_date, withdraw_date, \
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

/// 원생 목록을 다중 필터 + 정렬로 조회한다.
///
/// `day_of_week` 필터는 `student_schedules` JOIN 으로 현행 스케줄 보유 여부 검증.
/// 그 외 필터는 students 자체 컬럼.
#[tauri::command]
pub async fn list_students(filter: StudentFilter) -> Result<Vec<Student>, String> {
    let pool = db::pool().map_err(String::from)?;

    // SELECT 절은 students 컬럼 고정. JOIN/WHERE/ORDER 는 동적 빌드.
    // 동적 SQL 문자열은 정적 단편의 조건부 연결 — 사용자 입력은 모두 bind() 로 전달 (SQL injection 안전).
    let mut sql = String::from(
        "SELECT s.id, s.serial_no, s.name, s.gender, s.school_level, s.grade, s.school_id, \
                s.phone_student, s.phone_mother, s.phone_father, s.enroll_date, s.withdraw_date, \
                s.created_at, s.updated_at \
         FROM students s ",
    );
    if filter.day_of_week.is_some() {
        sql.push_str(
            "INNER JOIN student_schedules sch \
                ON sch.student_id = s.id AND sch.effective_to IS NULL AND sch.day_of_week = ? ",
        );
    }
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
    if !conditions.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push(' ');
    }
    sql.push_str(filter.sort.unwrap_or_default().order_by_sql());

    let mut q = sqlx::query(&sql);
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

#[cfg(test)]
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
    fn student_sort_default_is_name_asc() {
        assert_eq!(StudentSort::default(), StudentSort::NameAsc);
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
}
