//! Shared SQL/ISO timestamp helpers for catalog command and ranking persistence.

use chrono::{NaiveDateTime, TimeZone, Utc};
use sdkwork_utils_rust::{format_datetime, from_unix_millis, now};

const SQL_TIMESTAMP_PATTERN: &str = "%Y-%m-%d %H:%M:%S";

pub fn current_unix_seconds() -> i64 {
    now().timestamp()
}

pub fn start_of_day_unix_seconds(seconds: i64) -> i64 {
    seconds.div_euclid(86_400) * 86_400
}

pub fn sql_timestamp_now() -> String {
    format_datetime(now(), Some(SQL_TIMESTAMP_PATTERN))
}

pub fn sql_timestamp_from_unix(seconds: i64) -> String {
    from_unix_millis(seconds.saturating_mul(1_000))
        .map(|value| format_datetime(value, Some(SQL_TIMESTAMP_PATTERN)))
        .unwrap_or_default()
}

pub fn iso_timestamp_from_unix(seconds: i64) -> String {
    from_unix_millis(seconds.saturating_mul(1_000))
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_default()
}

pub fn date_string_from_unix_seconds(seconds: i64) -> String {
    from_unix_millis(seconds.saturating_mul(1_000))
        .map(|value| value.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub fn parse_sql_timestamp_to_seconds(value: &str) -> Option<i64> {
    let trimmed = value.trim().trim_end_matches('Z');
    NaiveDateTime::parse_from_str(trimmed, SQL_TIMESTAMP_PATTERN)
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S"))
        .ok()
        .map(|naive| Utc.from_utc_datetime(&naive).timestamp())
}

pub fn add_seconds_to_sql_timestamp(value: &str, seconds: i64) -> String {
    parse_sql_timestamp_to_seconds(value)
        .map(|base| iso_timestamp_from_unix(base + seconds.max(1)))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_timestamp_now_uses_space_separated_format() {
        let value = sql_timestamp_now();
        assert!(NaiveDateTime::parse_from_str(&value, SQL_TIMESTAMP_PATTERN).is_ok());
    }

    #[test]
    fn parse_and_round_trip_sql_timestamp() {
        let sample = "2026-06-29 12:34:56";
        let seconds = parse_sql_timestamp_to_seconds(sample).expect("seconds");
        assert_eq!(sample, sql_timestamp_from_unix(seconds));
    }
}
