use chrono::prelude::*;
use chrono_tz::Tz;
use std::time::Duration;

#[cfg(not(test))]
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
pub fn utc_now() -> DateTime<Utc> {
    DateTime::from_timestamp(1719704853, 0)
        .expect("can construct dummy-now for testing")
}

/// Check if `time` is yesterday or before.
pub fn is_before_today(datetime: &DateTime<Utc>, user_timezone: Tz) -> bool {
    let local_dt = datetime.with_timezone(&user_timezone);
    let date = local_dt.date_naive();
    let yesterday = (utc_now() - Duration::from_secs(60 * 60 * 24))
        .with_timezone(&user_timezone)
        .date_naive();
    date <= yesterday
}

pub fn as_days(duration: Duration) -> u64 {
    duration.as_secs() / 24 / 60 / 60
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Days;

    #[test]
    fn test_yesterday_is_before_today() {
        let yesterday = utc_now().checked_sub_days(Days::new(1)).unwrap();
        let result = is_before_today(&yesterday, Tz::UTC);
        assert!(result);
    }

    #[test]
    fn test_today_is_not_before_today() {
        let today = utc_now();
        let result = is_before_today(&today, Tz::UTC);
        assert!(!result);
    }

    #[test]
    fn test_11_59_yesterday_est_is_not_today() {
        let barely_yesterday = utc_now()
            .with_timezone(&Tz::EST)
            .checked_sub_days(Days::new(1))
            .unwrap()
            .with_hour(23)
            .unwrap()
            .with_minute(59)
            .unwrap()
            .with_second(59)
            .unwrap();
        let result = is_before_today(&barely_yesterday.to_utc(), Tz::EST);
        assert!(result);
    }
    #[test]
    fn test_but_one_second_later_it_is() {
        let barely_yesterday = utc_now()
            .with_timezone(&Tz::EST)
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();
        let result = is_before_today(&barely_yesterday.to_utc(), Tz::EST);
        assert!(!result);
    }
}
