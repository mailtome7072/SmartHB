//! 목록 조회 IPC 공통 페이지네이션 정책 (R14, Sprint 3 T3).
//!
//! `list_*` 계열 IPC 가 `limit`/`offset` 을 받을 때 동일한 정책으로 정규화하도록 한다.
//!
//! ## 정책
//!
//! - `limit` 미지정 → [`DEFAULT_LIST_LIMIT`] (100)
//! - `limit` 상한 → [`MAX_LIST_LIMIT`] (1000) — 메모리·UI 보호
//! - `limit` 하한 → 1 (0 으로 호출 시 1 로 보정 — 빈 결과 대신 최소 1건 반환)
//! - `offset` 은 정규화하지 않는다 (SQLite 가 범위 초과 시 빈 결과 반환).
//!
//! audit.rs 도 동일 정책을 따르되 본 모듈 도입 전 작성되어 인라인 `.unwrap_or(100).min(1000)`
//! 패턴을 사용한다. 별도 sweep 으로 통합 예정.

/// 기본 페이지 크기 — UI 가 limit 미지정 시 적용.
pub const DEFAULT_LIST_LIMIT: u32 = 100;
/// 페이지 크기 상한 — UI 입력 검증 실패 시 안전망.
pub const MAX_LIST_LIMIT: u32 = 1000;

/// `Option<u32>` 형태의 limit 입력을 정책 범위(`1..=MAX_LIST_LIMIT`) 로 정규화한다.
pub fn clamp_list_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIST_LIMIT).clamp(1, MAX_LIST_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_applies_default_when_none() {
        assert_eq!(clamp_list_limit(None), DEFAULT_LIST_LIMIT);
    }

    #[test]
    fn clamp_enforces_floor_and_ceiling() {
        assert_eq!(clamp_list_limit(Some(0)), 1, "0 → 1 보정");
        assert_eq!(clamp_list_limit(Some(1)), 1);
        assert_eq!(clamp_list_limit(Some(50)), 50, "범위 내 그대로");
        assert_eq!(clamp_list_limit(Some(MAX_LIST_LIMIT)), MAX_LIST_LIMIT);
        assert_eq!(clamp_list_limit(Some(9999)), MAX_LIST_LIMIT, "상한 초과 → MAX");
    }
}
