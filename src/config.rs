use std::time::Duration;

/// This is for all authentication sessions; users will need to log in again
/// every 30 days, since we basically have JWT authentication.
pub const SESSION_EXPIRY_TIME_DAYS: i64 = 30;

/// Password reset links will expire after 15 minutes.
pub const RESET_TOKEN_TIMEOUT_MINUTES: i64 = 15;

#[cfg(not(feature = "localhost_base_url"))]
pub const BASE_URL: &str = "https://beancount.bot";

#[cfg(feature = "localhost_base_url")]
pub const BASE_URL: &str = "http://localhost:8000";

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

#[cfg(feature = "use_stripe_test_instance")]
pub const BASIC_PLAN_STRIPE_ID: &str = "price_1OTyEXBhmccJFhTPvs01VoJf";

#[cfg(not(feature = "use_stripe_test_instance"))]
pub const BASIC_PLAN_STRIPE_ID: &str = "price_1OoJ2oBhmccJFhTPaTT6PdEp";

/// Page size for the food list view.
pub const FOOD_PAGE_SIZE: u8 = 50;

pub const MINIMUM_PASSWORD_LENGTH: u8 = 8;
