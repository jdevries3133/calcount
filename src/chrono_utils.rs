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

fn into_date(datetime: &DateTime<Utc>, user_timezone: Tz) -> NaiveDate {
    let local_dt = datetime.with_timezone(&user_timezone);
    local_dt.date_naive()
}

fn get_yesterday(user_timezone: Tz) -> NaiveDate {
    (utc_now() - Duration::from_secs(60 * 60 * 24))
        .with_timezone(&user_timezone)
        .date_naive()
}

/// Check if `time` is yesterday or before.
pub fn is_before_today(datetime: &DateTime<Utc>, user_timezone: Tz) -> bool {
    let date = into_date(datetime, user_timezone);
    let yesterday = get_yesterday(user_timezone);
    date <= yesterday
}

fn is_today(datetime: &DateTime<Utc>, user_timezone: Tz) -> bool {
    let date = into_date(datetime, user_timezone);
    let today = utc_now().with_timezone(&user_timezone).date_naive();

    date == today
}

fn is_yesterday(datetime: &DateTime<Utc>, user_timezone: Tz) -> bool {
    let date = into_date(datetime, user_timezone);
    let yesterday = get_yesterday(user_timezone);

    date == yesterday
}

pub fn fmt_date(datetime: &DateTime<Utc>, user_timezone: Tz) -> String {
    if is_today(datetime, user_timezone) {
        "Today".to_string()
    } else if is_yesterday(datetime, user_timezone) {
        "Yesterday".to_string()
    } else {
        datetime
            .with_timezone(&user_timezone)
            .format("%b %e")
            .to_string()
    }
}

pub fn as_days(duration: Duration) -> u64 {
    duration.as_secs() / 24 / 60 / 60
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Days;

    fn get_yesterday() -> DateTime<Utc> {
        utc_now().checked_sub_days(Days::new(1)).unwrap()
    }

    fn get_barely_yesterday() -> DateTime<Utc> {
        utc_now()
            .checked_sub_days(Days::new(1))
            .unwrap()
            .with_hour(23)
            .unwrap()
            .with_minute(59)
            .unwrap()
            .with_second(59)
            .unwrap()
    }

    fn get_barely_today() -> DateTime<Utc> {
        utc_now()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    }

    #[test]
    fn test_yesterday_is_before_today() {
        let yesterday = get_yesterday();
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
        let barely_yesterday = get_barely_yesterday();
        let result = is_before_today(&barely_yesterday, Tz::UTC);
        assert!(result);
    }
    #[test]
    fn test_but_one_second_later_it_is() {
        let barely_today = get_barely_today();
        let result = is_before_today(&barely_today, Tz::UTC);
        assert!(!result);
    }

    #[test]
    fn test_yesterday_is_not_today() {
        let yesterday = get_yesterday();
        let result = is_today(&yesterday, Tz::UTC);
        assert!(!result);
    }

    #[test]
    fn test_today_is_today() {
        let today = utc_now();
        let result = is_today(&today, Tz::UTC);
        assert!(result);
    }

    #[test]
    fn test_11_59_yesterday_est_is_not_today_via_is_today() {
        let barely_yesterday = get_barely_yesterday();
        let result = is_today(&barely_yesterday, Tz::UTC);
        assert!(!result);
    }
    #[test]
    fn test_but_one_second_later_it_is_today() {
        let barely_today = get_barely_today();
        let result = is_today(&barely_today, Tz::UTC);
        assert!(result);
    }

    #[test]
    fn test_yesterday_is_yesterday() {
        let yesterday = get_yesterday();
        let result = is_yesterday(&yesterday, Tz::UTC);
        assert!(result);
    }

    #[test]
    fn test_today_is_not_yesterday_via_is_yesterday() {
        let today = utc_now();
        let result = is_yesterday(&today, Tz::UTC);
        assert!(!result);
    }

    #[test]
    fn test_11_59_yesterday_est_is_not_today_via_is_yesterday() {
        let barely_yesterday = get_barely_yesterday();
        let result = is_yesterday(&barely_yesterday, Tz::UTC);
        assert!(result);
    }
    #[test]
    fn test_but_one_second_later_it_is_yesterday() {
        let barely_today = get_barely_today();
        let result = is_yesterday(&barely_today, Tz::UTC);
        assert!(!result);
    }

    #[test]
    fn test_fmt_today() {
        let today = get_barely_today();
        let text = fmt_date(&today, Tz::UTC);
        assert_eq!(text, "Today");
    }

    #[test]
    fn test_fmt_yesterday() {
        let today = get_barely_yesterday();
        let text = fmt_date(&today, Tz::UTC);
        assert_eq!(text, "Yesterday");
    }

    #[test]
    fn test_fmt_old_date() {
        let date =
            DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200")
                .unwrap();
        let text = fmt_date(&date.to_utc(), Tz::UTC);
        assert_eq!(text, "Jul  1");
    }
}
