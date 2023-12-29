use std::time::Duration;

/// This is for all authentication sessions; users will need to log in again
/// every 7 days, since we basically have JWT authentication.
pub const SESSION_EXPIRY_TIME_DAYS: i64 = 7;

/// Password reset links will expire after 15 minutes.
pub const RESET_TOKEN_TIMEOUT_MINUTES: i64 = 15;

/// Base URL of this website which is going to be wrong for local development,
/// but that's OK.
pub const DOMAIN: &str = "beancount.bot";

/// Messages which exceed this length limit will not be forwarded to OpenAI,
/// and will instead provide the user with an error message.
pub const CHAT_MAX_LEN: usize = 200;

/// We advertise a 30-day free trial, but we'll make it 31 days in duration,
/// since the clock starts as soon as registration happens and we want to
/// ensure that everyone gets 30 full days. Consider -- in the moment after
/// registration, the user would be left with only 29 full days remaining,
/// so we don't want a UX where someone registers and then the trial counter
/// shows "29 days remaining."
pub const FREE_TRIAL_DURATION: Duration =
    Duration::from_secs(60 * 60 * 24 * 31);
