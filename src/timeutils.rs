use std::time::Duration;

pub fn as_days(duration: Duration) -> u64 {
    duration.as_secs() / 24 / 60 / 60
}
