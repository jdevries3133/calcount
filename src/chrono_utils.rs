use chrono::prelude::*;
use chrono_tz::Tz;
use std::time::Duration;

/// Check if `time` is yesterday or before.
pub fn is_before_today(datetime: &DateTime<Utc>, user_timezone: Tz) -> bool {
    let local_dt = datetime.with_timezone(&user_timezone);
    let date = local_dt.date_naive();
    let yesterday = (Utc::now() - Duration::from_secs(60 * 60 * 24))
        .with_timezone(&user_timezone)
        .date_naive();
    date <= yesterday
}

pub fn as_days(duration: Duration) -> u64 {
    duration.as_secs() / 24 / 60 / 60
}
