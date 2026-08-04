// Time-related utilities

use chrono::{DateTime, Utc, Duration};

/// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Get current timestamp in milliseconds
    pub fn now_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    /// Get current timestamp in seconds
    pub fn now_secs() -> i64 {
        Utc::now().timestamp()
    }

    /// Format duration in human-readable format
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.num_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Calculate duration between two timestamps
    pub fn duration_between(start: DateTime<Utc>, end: DateTime<Utc>) -> Duration {
        end.signed_duration_since(start)
    }

    /// Check if a timestamp is within the last N seconds
    pub fn is_within_last_seconds(timestamp: DateTime<Utc>, seconds: i64) -> bool {
        let duration = Utc::now().signed_duration_since(timestamp);
        duration.num_seconds() <= seconds
    }
}
