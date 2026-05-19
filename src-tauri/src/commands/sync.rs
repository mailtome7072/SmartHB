//! 클라우드 동기화 대기 (T9, PRD §5.3).
//!
//! 양 PC 시점 분리 사용 시 한 디바이스가 종료한 직후 다른 디바이스가 앱을 시작하면 클라우드
//! 동기화가 완료되기 전이라 DB / 락 파일이 stale 상태일 수 있다. 본 모듈은 mtime 변화를 관찰하여
//! 동기화 완료 시점을 판단하고 시작 시퀀스 진입을 게이트한다.
//!
//! ## 판단 기준 (advisory)
//!
//! - DB 파일 mtime 이 최근 30초 이내 변경 — 동기화 진행 중일 가능성
//! - 락 파일 mtime 이 최근 30초 이내 변경 — 동기화 진행 중일 가능성
//! - 위 둘 다 30초 이상 안정 — `Ready`
//! - 30초 대기 후에도 변화가 멈추지 않으면 `Timeout` — 사용자가 새로고침 결정
//!
//! mtime 은 advisory — 클라우드 클라이언트별로 동작 다름. 본 IPC 는 시작 시퀀스 게이트의
//! soft signal 이며, 사용자 새로고침 옵션이 보완한다.

use crate::commands::{lock, paths};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;
use std::time::SystemTime;

/// 동기화 안정성 판단 임계 — 본 시간 이내에 mtime 이 변경되면 "동기화 중" 으로 본다.
const STABILITY_THRESHOLD_SECONDS: i64 = 30;

/// 동기화 대기 상태 — IPC 응답.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum SyncStatus {
    /// 두 파일 모두 안정 — 시작 시퀀스 진입 가능.
    Ready,
    /// mtime 이 최근 변경됨 — 동기화 진행 중일 가능성. UI 가 안내 + 일정 시간 후 재호출.
    Waiting { seconds_since_change: i64 },
}

/// 파일 mtime 을 UTC `DateTime` 으로 변환. 파일이 없으면 `None`.
///
/// 동기화 임계는 30초 단위 비교라 ns 정밀도 불필요 — 초 단위로 단순화.
fn file_mtime(path: &PathBuf) -> Option<DateTime<Utc>> {
    let modified: SystemTime = std::fs::metadata(path).ok()?.modified().ok()?;
    let secs = modified.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs() as i64;
    DateTime::<Utc>::from_timestamp(secs, 0)
}

fn determine_status(now: DateTime<Utc>, latest_mtime: Option<DateTime<Utc>>) -> SyncStatus {
    let Some(mtime) = latest_mtime else {
        return SyncStatus::Ready;
    };
    let elapsed = (now - mtime).num_seconds();
    if elapsed >= STABILITY_THRESHOLD_SECONDS {
        SyncStatus::Ready
    } else {
        SyncStatus::Waiting {
            seconds_since_change: elapsed.max(0),
        }
    }
}

/// 현재 DB / 락 파일 mtime 으로 동기화 상태를 판단한다.
///
/// UI 는 `Waiting` 일 때 일정 간격으로 본 IPC 를 재호출하며 진행 안내를 표시한다.
/// 30초 사용자 대기 후 여전히 `Waiting` 이면 UI 는 "새로고침" 옵션을 노출한다.
#[tauri::command]
pub async fn check_sync_status() -> Result<SyncStatus, String> {
    tokio::task::spawn_blocking(|| {
        let db_mtime = file_mtime(&paths::db_path());
        let lock_mtime = file_mtime(&lock::lock_path());
        let latest = match (db_mtime, lock_mtime) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        determine_status(Utc::now(), latest)
    })
    .await
    .map_err(|e| AppError::Config(format!("동기화 상태 작업 실패: {}", e)))
    .map_err(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_when_no_files_exist() {
        let now = Utc::now();
        assert_eq!(determine_status(now, None), SyncStatus::Ready);
    }

    #[test]
    fn ready_when_mtime_older_than_threshold() {
        let now = Utc::now();
        let mtime = now - chrono::Duration::seconds(STABILITY_THRESHOLD_SECONDS + 5);
        assert_eq!(determine_status(now, Some(mtime)), SyncStatus::Ready);
    }

    #[test]
    fn waiting_when_mtime_within_threshold() {
        let now = Utc::now();
        let mtime = now - chrono::Duration::seconds(5);
        let status = determine_status(now, Some(mtime));
        match status {
            SyncStatus::Waiting { seconds_since_change } => {
                assert!((4..=6).contains(&seconds_since_change));
            }
            _ => panic!("Waiting 기대"),
        }
    }

    #[test]
    fn waiting_clamps_negative_elapsed_to_zero() {
        // 시계 차이로 mtime 이 미래로 보이는 경우(NTP 보정 직후 등) 음수 elapsed 방어.
        let now = Utc::now();
        let mtime = now + chrono::Duration::seconds(2);
        let status = determine_status(now, Some(mtime));
        match status {
            SyncStatus::Waiting { seconds_since_change } => {
                assert_eq!(seconds_since_change, 0);
            }
            _ => panic!("Waiting 기대 — 미래 mtime 도 임계 이내"),
        }
    }

    #[test]
    fn sync_status_serializes_with_kind_tag() {
        let ready = SyncStatus::Ready;
        let json = serde_json::to_string(&ready).unwrap();
        assert_eq!(json, r#"{"kind":"ready"}"#);

        let waiting = SyncStatus::Waiting { seconds_since_change: 10 };
        let json = serde_json::to_string(&waiting).unwrap();
        assert!(json.contains(r#""kind":"waiting""#));
        assert!(json.contains(r#""seconds_since_change":10"#));
    }
}
