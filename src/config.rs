/// This is for all authentication sessions; users will need to log in again
/// every 7 days, since we basically have JWT authentication.
pub const SESSION_EXPIRY_TIME_DAYS: i64 = 7;

/// Password reset links will expire after 15 minutes.
pub const RESET_TOKEN_TIMEOUT_MINUTES: i64 = 15;

/// Base URL of this website which is going to be wrong for local development,
/// but that's OK.
pub const DOMAIN: &str = "beancount.bot";
